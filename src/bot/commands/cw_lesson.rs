use anyhow::Context as _;
use serenity::model::application::command::Command;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;
use std::sync::{Arc, Mutex};

use crate::{bot::BotStateMode, modes::lesson::LessonGen};

pub fn get_lesson_gen(probset: &str) -> anyhow::Result<LessonGen> {
    // TODO: use braces to support nesting
    let (probset_name, probset_args_str) = probset.split_once(':').unwrap_or((probset, ""));

    use crate::modes::lesson;

    let gen: LessonGen = match probset_name {
        "call_ja" => Box::new(lesson::callsign::JaCallsignGen {}),
        "file" => Box::new(lesson::file::FileSourceGen::new(probset_args_str)?),
        "nr_allja" => Box::new(lesson::allja_number::AllJANumberGen::new()),
        "nr_acag" => Box::new(lesson::acag_number::ACAGNumberGen::new()),
        "rand5_jp" => Box::new(lesson::japanese::JapaneseFiveCharGen {}),
        _ => {
            anyhow::bail!(
                "unknown probset.\n".to_owned()
                    + "available selections are: call_ja, file, nr_allja, nr_acag, rand5_jp"
            )
        }
    };
    Ok(gen)
}

impl crate::bot::Bot {
    pub async fn run_command_lesson_start(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> anyhow::Result<String> {
        let mut min_speed = None;
        let mut max_speed = None;
        let mut min_freq = None;
        let mut max_freq = None;
        let mut probset = "call_ja".to_string();

        command
            .data
            .options
            .iter()
            .try_fold::<_, _, anyhow::Result<()>>((), |_, x| {
                let v = x.value.as_ref().context("value empty")?;

                let vf = v.as_f64().map(|x| x as f32).context("value is not f64");
                let vs = v.as_str().context("value is not string");

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

        anyhow::ensure!(min_speed <= max_speed, "min_speed > max_speed");

        let min_freq = min_freq.unwrap_or(500.0_f32.min(max_freq.unwrap_or(std::f32::NAN)));
        let max_freq = max_freq.unwrap_or(1000.0_f32.max(min_freq));

        anyhow::ensure!(min_freq <= max_freq, "min_freq > max_freq");

        let speed_range = min_speed..=max_speed;
        let freq_range = min_freq..=max_freq;

        probset.make_ascii_lowercase();
        let gen = get_lesson_gen(&probset)?;

        let gid = command.guild_id.context("not in guild")?;
        let state = Arc::new(Mutex::new(crate::modes::lesson::LessonModeState::new(
            speed_range,
            freq_range,
            gen,
        )));
        crate::modes::lesson::start(ctx, gid, state.clone())
            .await
            .context("internal error")?;
        self.switch_mode(gid.0, BotStateMode::Lesson(state))?;

        Ok("let's start lesson".to_string())
    }

    pub async fn run_command_lesson_end(
        &self,
        _ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> anyhow::Result<String> {
        let r = self.switch_mode(
            command.guild_id.context("no guild")?.0,
            BotStateMode::Normal,
        )?;

        Ok(r)
    }

    pub async fn register_commands_cw_lesson(&self, ctx: &Context) -> anyhow::Result<()> {
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
        .context("command cw-start-lesson registration failed")?;

        Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-end-lesson")
                .description("end callsign lesson")
        })
        .await
        .context("command cw-end-lesson registration failed")?;

        Ok(())
    }
}
