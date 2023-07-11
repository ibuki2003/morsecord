use serenity::model::application::command::Command;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;
use std::sync::{Arc, Mutex};

use crate::bot::BotStateMode;

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

        command
            .data
            .options
            .iter()
            .try_fold::<_, _, Result<(), String>>((), |_, x| {
                let v = x
                    .value
                    .as_ref()
                    .map_or(Err("value not found".to_string()), |v| {
                        v.as_f64().ok_or("value is not f64".to_string())
                    })? as f32;
                match x.name.as_str() {
                    "min_speed" => min_speed = Some(v),
                    "max_speed" => max_speed = Some(v),
                    "min_freq" => min_freq = Some(v),
                    "max_freq" => max_freq = Some(v),
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
        let gid = command.guild_id.ok_or("not in guild")?;
        let state = Arc::new(Mutex::new(crate::modes::lesson::LessonModeState::new(
            speed_range,
            freq_range,
            Box::new(crate::modes::lesson::callsign::JaCallsignGen {}),
        )));
        crate::modes::lesson::start(&ctx, gid, state.clone())
            .await
            .map_err(|_| "error occured")?;
        let _ = self.switch_mode(gid.0, BotStateMode::Lesson(state)).await;

        Ok("let's start lesson".to_string())
    }

    pub async fn run_command_lesson_end(
        &self,
        _ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<String, String> {
        let _ = self
            .switch_mode(command.guild_id.ok_or("no guild")?.0, BotStateMode::Normal)
            .await;

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
