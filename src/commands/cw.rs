use serenity::model::application::command::Command;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::Context;

pub async fn run_speed(ctx: &Context, command: &ApplicationCommandInteraction, db: &sqlx::SqlitePool) -> String {
    let new_speed = command.data.options
        .iter()
        .find(|option| option.name == "speed")
        .map(|option| option.value.as_ref().unwrap().as_f64().unwrap());

    if let Err(e) = sqlx::query("insert or replace into cw_speed(id, speed) values(?, ?)")
        .bind(command.user.id.to_string())
        .bind(new_speed.unwrap())
        .execute(db)
        .await {

        return format!("error: {}", e);
    }

    "ok!".to_string()
}

pub async fn run_freq(ctx: &Context, command: &ApplicationCommandInteraction, db: &sqlx::SqlitePool) -> String {
    let new_freq = command.data.options
        .iter()
        .find(|option| option.name == "freq")
        .map(|option| option.value.as_ref().unwrap().as_f64().unwrap());

    if let Err(e) = sqlx::query("insert or replace into cw_speed(id, freq) values(?, ?)")
        .bind(command.user.id.to_string())
        .bind(new_freq.unwrap())
        .execute(db)
        .await {

        return format!("error: {}", e);
    }

    "ok!".to_string()
}

pub async fn register(ctx: &Context) {
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

    }).await.unwrap();

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
    }).await.unwrap();
}
