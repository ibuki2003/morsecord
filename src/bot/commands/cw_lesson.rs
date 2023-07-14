use serenity::model::application::command::Command;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;
use std::sync::{Arc, Mutex};

use crate::bot::BotStateMode;

pub fn get_lesson_gen(probset: &str) -> Result<Box<dyn Iterator<Item = String> + Send>, String> {
    let (probset_name, _probset_args_str) =
        probset.split_once(':').unwrap_or((probset, ""));

    use crate::modes::lesson;

    let gen: Box<dyn Iterator<Item = String> + Send> = match probset_name {
        "call_ja" => Box::new(lesson::callsign::JaCallsignGen {}),
        "nr_allja" => Box::new(lesson::allja_number::AllJANumberGen::new()),
        _ => return Err(
            "unknown probset.\n".to_owned()
                + "available selections are: call_ja, nr_allja",
            ),
    };
    Ok(gen)
}

impl crate::bot::Bot {
    pub async fn run_command_lesson_start(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<String, String> {
        let mut min_speed = None;
        let mut max_speed = None;
        let mut min_freq = None;
        let mut max_freq = None;
        let mut probset = "call_ja".to_string();

        command
            .data
            .options
            .iter()
            .try_fold::<_, _, Result<(), String>>((), |_, x| {
                let v = x.value.as_ref().ok_or("value not found".to_string())?;

                let vf = v
                    .as_f64()
                    .map(|x| x as f32)
                    .ok_or("value is not f64".to_string());
                let vs = v.as_str().ok_or("value is not string".to_string());

                match x.name.as_str() {
                    "min_speed" => min_speed = Some(vf?),
                    "max_speed" => max_speed = Some(vf?),
                    "min_freq" => min_freq = Some(vf?),
                    "max_freq" => max_freq = Some(vf?),
                    "probset" => probset = vs?.to_string(),
                    _ => (),
                };
                Ok(())
            })?;

        let min_speed = min_speed.unwrap_or(15.0_f32.min(max_speed.unwrap_or(std::f32::NAN)));
        let max_speed = max_speed.unwrap_or(20.0_f32.max(min_speed));

        if min_speed > max_speed {
            return Err("min_speed > max_speed".to_string());
        }

        let min_freq = min_freq.unwrap_or(500.0_f32.min(max_freq.unwrap_or(std::f32::NAN)));
        let max_freq = max_freq.unwrap_or(1000.0_f32.max(min_freq));

        if min_freq > max_freq {
            return Err("min_freq > max_freq".to_string());
        }

        let speed_range = min_speed..=max_speed;
        let freq_range = min_freq..=max_freq;

        probset.make_ascii_lowercase();
        let gen = get_lesson_gen(&probset)?;

        let gid = command.guild_id.ok_or("not in guild")?;
        let state = Arc::new(Mutex::new(crate::modes::lesson::LessonModeState::new(
            speed_range,
            freq_range,
            gen,
        )));
        crate::modes::lesson::start(&ctx, gid, state.clone())
            .await
            .map_err(|_| "error occured")?;
        let _ = self.switch_mode(gid.0, BotStateMode::Lesson(state))?;

        Ok("let's start lesson".to_string())
    }

    pub async fn run_command_lesson_end(
        &self,
        _ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<String, String> {
        let _ = self.switch_mode(command.guild_id.ok_or("no guild")?.0, BotStateMode::Normal)?;

        Ok("good job!".to_string())
    }

    pub async fn register_command_cw_lesson(&self, ctx: &Context) -> Result<(), ()> {
        Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-start-lesson")
                .description("start callsign lesson")
                .create_option(|option| {
                    option
                        .name("min_speed")
                        .description("minimum speed")
                        .kind(serenity::model::prelude::command::CommandOptionType::Number)
                        .min_number_value(5.0)
                        .required(false)
                })
                .create_option(|option| {
                    option
                        .name("max_speed")
                        .description("maximum speed")
                        .kind(serenity::model::prelude::command::CommandOptionType::Number)
                        .min_number_value(5.0)
                        .required(false)
                })
                .create_option(|option| {
                    option
                        .name("min_freq")
                        .description("minimum freq")
                        .kind(serenity::model::prelude::command::CommandOptionType::Number)
                        .min_number_value(200.0)
                        .required(false)
                })
                .create_option(|option| {
                    option
                        .name("max_freq")
                        .description("maximum freq")
                        .kind(serenity::model::prelude::command::CommandOptionType::Number)
                        .min_number_value(200.0)
                        .required(false)
                })
                .create_option(|option| {
                    option
                        .name("probset")
                        .description("problem set name (can be followed by colon and args)")
                        .kind(serenity::model::prelude::command::CommandOptionType::String)
                        .required(false)
                })
        })
        .await
        .map_err(|e| log::error!("error: {:?}", e))?;

        Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-end-lesson")
                .description("end callsign lesson")
        })
        .await
        .map_err(|e| log::error!("error: {:?}", e))?;

        Ok(())
    }
}
