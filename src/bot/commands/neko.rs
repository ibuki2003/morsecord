use crate::bot::commands::get_value_i64;
use serenity::model::prelude::command::{Command, CommandOptionType};
use serenity::model::prelude::interaction::application_command::CommandDataOption;

impl crate::bot::Bot {
    pub async fn register_command_neko(&self, ctx: &serenity::client::Context) -> Result<(), ()> {
        Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("neko")
                .description("猫のように鳴く")
                .create_option(|option| {
                    option
                        .name("count")
                        .description("にゃーんの回数")
                        .kind(CommandOptionType::Integer)
                        .min_int_value(1)
                        .max_int_value(32)
                        .required(false)
                })
        })
        .await
        .map_err(|e| log::error!("error: {:?}", e))?;
        Ok(())
    }

    pub fn run_command_neko(&self, options: &[CommandDataOption]) -> Result<String, String> {
        let count = options
            .iter()
            .find(|option| option.name == "count")
            .map(|option| get_value_i64(&option.value))
            .unwrap_or(Ok(1))?;

        let count = count.clamp(1, 32);

        Ok("にゃーん".to_string().repeat(count as usize))
    }
}
