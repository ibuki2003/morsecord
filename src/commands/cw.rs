use serenity::model::application::command::Command;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::Context;

pub async fn run_play(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let man = songbird::get(&ctx).await
        .expect("init songbird").clone();
    let gid = command.guild_id.unwrap();

    let s = command.data.options
        .iter()
        .find(|option| option.name == "str")
        .map(|option| option.value.as_ref().unwrap().as_str().unwrap());
    if let Some(s) = s {
        let handler = man.get(gid);
        if let Some(handler) = handler {
            let mut handler = handler.lock().await;
            let source = crate::cw_audio::CWAudioPCM::new(s.to_string(), 20.0, 800.0).to_input();
            handler.play_source(source);
            "-.-.".to_string()
        } else {
            "Not in a voice channel".to_string()
        }
    } else {
        "no string".to_string()
    }
}

pub async fn register(ctx: &Context) {
    Command::create_global_application_command(&ctx.http, |command| {
        command
            .name("cw-play")
            .description("play")
            .create_option(|option| {
                option
                    .name("str")
                    .description("string")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    }).await.unwrap();
}
