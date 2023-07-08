use serenity::model::application::command::Command;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::prelude::Context;

fn get_ch(cmd: &serenity::model::prelude::application_command::ApplicationCommandInteraction) -> u64 {
    let ch = cmd.data.options
        .iter()
        .find(|opt| opt.name == "ch")
        .expect("no ch option");

    let ch = ch.value.as_ref().unwrap().as_str().unwrap().parse::<u64>().unwrap();
    ch
}

pub async fn run_join(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let ch = get_ch(&command);

    let man = songbird::get(&ctx).await
        .expect("init songbird").clone();
    let handler = man.join(command.guild_id.unwrap(), ch).await;
    handler.1.unwrap();
    {
        let mut handler = handler.0.lock().await;
        handler.deafen(true).await.unwrap();
    }

    "got it!".to_string()
}

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
            let source = crate::cw_audio::CWAudioPCM::new(s.to_string(), 40.0, 800.0).to_input();
            handler.play_source(source);
            "-.-.".to_string()
        } else {
            "Not in a voice channel".to_string()
        }
    } else {
        "no string".to_string()
    }
}

pub async fn run_leave(ctx: &Context, command: &ApplicationCommandInteraction) -> String {
    let man = songbird::get(&ctx).await
        .expect("init songbird").clone();
    let gid = command.guild_id.unwrap();
    let cid = command.channel_id;
    let has_handler = man.get(gid).is_some();

    if has_handler {
        if let Err(e) = man.remove(gid).await {
            cid.say(&ctx.http, format!("Failed: {:?}", e)).await.unwrap();
        }

        cid.say(&ctx.http, "Left voice channel").await.unwrap();
    } else {
        cid.say(&ctx.http, "Not in a voice channel").await.unwrap();
    }

    "bye!".to_string()
}

pub async fn register(ctx: &Context) {
    Command::create_global_application_command(&ctx.http, |command| {
        command
            .name("cw-join")
            .description("join")
            .create_option(|option| {
                use serenity::model::channel::ChannelType;
                option
                    .name("ch")
                    .description("channel to join")
                    .kind(CommandOptionType::Channel)
                    .channel_types(&[ChannelType::Voice])
                    .required(true)
            })
    }).await.unwrap();

    Command::create_global_application_command(&ctx.http, |command| {
        command
            .name("cw-leave")
            .description("leave")
    }).await.unwrap();

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
