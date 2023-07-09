use std::sync::{Arc, Mutex};
use std::{fs::File, io::BufReader, usize};

use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::gateway::Ready;
use serenity::prelude::{Client, Context, EventHandler};

use songbird::SerenityInit;

use serde::{Deserialize, Serialize};
use serde_json::Result;

use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;

use morsecord::commands;

enum BotStateMode {
    Normal,
    CallsignLesson(Arc<Mutex<morsecord::modes::call_lesson::CallLessonModeState>>),
}

struct BotState {
    mode: BotStateMode,
}
impl std::default::Default for BotState {
    fn default() -> Self {
        Self {
            mode: BotStateMode::Normal,
        }
    }
}

struct Bot {
    db: sqlx::SqlitePool,

    states: Arc<Mutex<std::collections::HashMap<u64, Arc<Mutex<BotState>>>>>,
}

#[async_trait]
impl EventHandler for Bot {
    // Botが起動したときに走る処理
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(serenity::model::gateway::Activity::listening("7.074MHz")).await;
        println!("{} is connected!", ready.user.name);

        use serenity::model::application::command::Command;
        Command::create_global_application_command(&ctx.http, |command| {
            commands::neko::register(command)
        }).await.unwrap();

        commands::vc::register(&ctx).await;
        commands::cw::register(&ctx).await;

        Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-start-lesson")
                .description("start callsign lesson")
                .create_option(|option| { option.name("min_speed").description("minimum speed").kind(serenity::model::prelude::command::CommandOptionType::Number).min_number_value(5.0).required(false) })
                .create_option(|option| { option.name("max_speed").description("maximum speed").kind(serenity::model::prelude::command::CommandOptionType::Number).min_number_value(5.0).required(false) })
                .create_option(|option| { option.name("min_freq").description("minimum freq").kind(serenity::model::prelude::command::CommandOptionType::Number).min_number_value(200.0).required(false) })
                .create_option(|option| { option.name("max_freq").description("maximum freq").kind(serenity::model::prelude::command::CommandOptionType::Number).min_number_value(200.0).required(false) })
        }).await.unwrap();

        Command::create_global_application_command(&ctx.http, |command| {
            command
                .name("cw-end-lesson")
                .description("end callsign lesson")
        }).await.unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = match command.data.name.as_str() {
                "neko" => commands::neko::run(&command.data.options),
                "cw-join" => commands::vc::run_join(&ctx, &command).await,
                "cw-leave" => commands::vc::run_leave(&ctx, &command).await,
                "cw-speed" => commands::cw::run_speed(&ctx, &command, &self.db).await,
                "cw-freq" => commands::cw::run_freq(&ctx, &command, &self.db).await,
                "cw-start-lesson" => {
                    let mut min_speed = 15.0;
                    let mut max_speed = 30.0;
                    let mut min_freq = 400.0;
                    let mut max_freq = 1000.0;

                    command.data.options.iter().for_each(|x| {
                        match x.name.as_str() {
                            "min_speed" => min_speed = x.value.as_ref().unwrap().as_f64().unwrap() as f32,
                            "max_speed" => max_speed = x.value.as_ref().unwrap().as_f64().unwrap() as f32,
                            "min_freq" => min_freq = x.value.as_ref().unwrap().as_f64().unwrap() as f32,
                            "max_freq" => max_freq = x.value.as_ref().unwrap().as_f64().unwrap() as f32,
                            _ => (),
                        };
                    });

                    let speed_range = min_speed..max_speed;
                    let freq_range = min_freq..max_freq;
                    let state = Arc::new(Mutex::new(morsecord::modes::call_lesson::CallLessonModeState::new(speed_range, freq_range, command.channel_id)));
                    morsecord::modes::call_lesson::start(&ctx, command.guild_id.unwrap(), state.clone()).await;
                    let old = self.states.lock().unwrap().insert(command.guild_id.unwrap().0, Arc::new(Mutex::new(BotState {
                        mode: BotStateMode::CallsignLesson(state),
                    })));

                    if let Some(state) = old {
                        match &state.lock().unwrap().mode {
                            BotStateMode::Normal => {},
                            BotStateMode::CallsignLesson(s) => {
                                morsecord::modes::call_lesson::end(s.clone());
                            },
                        }
                    }

                    "let's start lesson".to_string()
                },
                "cw-end-lesson" => {
                    let old = self.states.lock().unwrap().insert(command.guild_id.unwrap().0, Arc::new(Mutex::new(BotState {
                        mode: BotStateMode::Normal,
                    })));
                    if let Some(state) = old {
                        match &state.lock().unwrap().mode {
                            BotStateMode::Normal => {},
                            BotStateMode::CallsignLesson(s) => {
                                morsecord::modes::call_lesson::end(s.clone());
                            },
                        }
                    }

                    "good job!".to_string()
                },
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

    async fn message(&self, ctx: Context, message: Message) {
        {
            // FIXME: match statement gives error "future cannot be sent between threads safely"
            let (is_normal, csls) = {
                let mut states = self.states.lock().unwrap();
                let state = states.entry(message.guild_id.unwrap().0).or_default().clone();
                drop(states);

                let mode = &state.lock().unwrap().mode;

                let is_normal = match mode {
                    BotStateMode::Normal => true,
                    _ => false,
                };

                let csls = match mode {
                    BotStateMode::CallsignLesson(s) => Some(s.clone()),
                    _ => None,
                };

                (is_normal, csls)
            };

            if is_normal {
                morsecord::modes::normal::on_message(&ctx, &message, &self.db).await;
            }

            if let Some(csls) = csls {
                morsecord::modes::call_lesson::on_message(&ctx, &message, csls.clone()).await;
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

    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("db.sqlite3")
                .create_if_missing(true),
        )
        .await
        .expect("failed to connect to sqlite3");

    // migration
    sqlx::query("create table if not exists cw_speed (id text primary key, speed REAL not null default 20, freq REAL not null default 800)")
        .execute(&db)
        .await
        .expect("failed to create table");

    use serenity::model::gateway::GatewayIntents;
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Bot {
            db,
            states: Arc::new(Mutex::new(std::collections::HashMap::new())),
        })
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    });

    tokio::signal::ctrl_c().await.unwrap();
    println!("Received Ctrl-C, shutting down.");
}
