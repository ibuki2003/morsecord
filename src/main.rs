use std::{fs::File, io::BufReader, usize};

use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::gateway::Ready;
use serenity::prelude::{Client, Context, EventHandler};

use songbird::SerenityInit;

use serde::{Deserialize, Serialize};
use serde_json::Result;

use serenity::model::application::interaction::{Interaction, InteractionResponseType};

use morsecord::commands;

struct Handler;

fn get_ch(cmd: &serenity::model::prelude::application_command::ApplicationCommandInteraction) -> u64 {
    let ch = cmd.data.options
        .iter()
        .find(|opt| opt.name == "ch")
        .expect("no ch option");

    let ch = ch.value.as_ref().unwrap().as_str().unwrap().parse::<u64>().unwrap();
    println!("{:?}", ch);
    ch
}

#[async_trait]
impl EventHandler for Handler {
    // Botが起動したときに走る処理
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(serenity::model::gateway::Activity::listening("7.074MHz")).await;
        println!("{} is connected!", ready.user.name);

        use serenity::model::application::command::Command;
        let guild_command = Command::create_global_application_command(&ctx.http, |command| {
            commands::neko::register(command)
        })
        .await.unwrap();

        println!("I created the following global slash command: {:#?}", guild_command);

        let a = Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-join")
                .description("join")
                .create_option(|option| {
                    use serenity::model::application::command::CommandOptionType;
                    use serenity::model::channel::ChannelType;
                    option
                        .name("ch")
                        .description("channel to join")
                        .kind(CommandOptionType::Channel)
                        .channel_types(&[ChannelType::Voice])
                        .required(true)
                })
        }).await.unwrap();
        dbg!(a);

        let a = Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-leave")
                .description("leave")
        }).await.unwrap();
        dbg!(a);

        let a = Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-play")
                .description("play")
                .create_option(|option| {
                    use serenity::model::application::command::CommandOptionType;
                    option
                        .name("str")
                        .description("string")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
        }).await.unwrap();
        dbg!(a);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "neko" => commands::neko::run(&command.data.options),
                "cw-join" => {
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
                "cw-play" => {
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
                            let source = morsecord::cw_audio::CWAudioPCM::new(s.to_string(), 20.0, 800.0).to_input();
                            handler.play_source(source);
                            "-.-.".to_string()
                        } else {
                            "Not in a voice channel".to_string()
                        }
                    } else {
                        "no string".to_string()
                    }
                }

                "cw-leave" => {
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
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Token {
    token: String,
}

fn get_token(file_name: &str) -> Result<String> {
    let file = File::open(file_name).unwrap();
    let reader = BufReader::new(file);
    let t: Token = serde_json::from_reader(reader).unwrap();
    Ok(t.token)
}

#[tokio::main]
async fn main() {
    let token = get_token("config.json").expect("no token found");
    let framework = StandardFramework::new()
        // .configure(|c| c.prefix("~")) // コマンドプレフィックス
        ;


    use serenity::model::gateway::GatewayIntents;
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
