use serenity::model::application::command::Command;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::Context;

use crate::bot::commands::get_value_f64;

impl crate::bot::Bot {
    pub async fn run_command_speed(
        &self,
        _ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<String, String> {
        let new_speed = command
            .data
            .options
            .iter()
            .find(|option| option.name == "speed")
            .map(|option| get_value_f64(&option.value))
            .ok_or("no argument".to_string())??;

        if let Err(e) = sqlx::query("insert or replace into cw_speed(id, speed) values(?, ?)")
            .bind(command.user.id.to_string())
            .bind(new_speed)
            .execute(&self.db)
            .await
        {
            return Err(format!("error: {}", e));
        }

        Ok("ok!".to_string())
    }

    pub async fn run_command_freq(
        &self,
        _ctx: &Context,
        command: &ApplicationCommandInteraction,
    ) -> Result<String, String> {
        let new_freq = command
            .data
            .options
            .iter()
            .find(|option| option.name == "freq")
            .map(|option| get_value_f64(&option.value))
            .ok_or("no argument".to_string())??;

        if let Err(e) = sqlx::query("insert or replace into cw_speed(id, freq) values(?, ?)")
            .bind(command.user.id.to_string())
            .bind(new_freq)
            .execute(&self.db)
            .await
        {
            return Err(format!("error: {}", e));
        }

        Ok("ok!".to_string())
    }

    pub async fn register_commands_cw(&self, ctx: &Context) -> Result<(), ()> {
        Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-speed")
                .description("set cw speed")
                .create_option(|option| {
                    option
                        .name("speed")
                        .description("speed(wpm)")
                        .kind(CommandOptionType::Number)
                        .min_number_value(5.0)
                        .required(true)
                })
        })
        .await
        .map_err(|e| log::error!("error: {:?}", e))?;

        Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-freq")
                .description("set cw freq")
                .create_option(|option| {
                    option
                        .name("freq")
                        .description("freq (Hz)")
                        .kind(CommandOptionType::Number)
                        .min_number_value(10.0)
                        .required(true)
                })
        })
        .await
        .map_err(|e| log::error!("error: {:?}", e))?;
        Ok(())
    }
}
