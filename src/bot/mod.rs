mod commands;

use std::sync::{Arc, Mutex};

use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};

use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::channel::Message;

#[derive(Clone)]
pub enum BotStateMode {
    Normal,
    CallsignLesson(Arc<Mutex<crate::modes::lesson::CallLessonModeState>>),
}

impl std::default::Default for BotStateMode {
    fn default() -> Self {
        BotStateMode::Normal
    }
}

struct BotState {
    txt_ch: Option<serenity::model::id::ChannelId>,
    mode: Arc<Mutex<BotStateMode>>,
}

impl std::default::Default for BotState {
    fn default() -> Self {
        BotState {
            txt_ch: None,
            mode: Arc::new(Mutex::new(BotStateMode::default())),
        }
    }
}

pub struct Bot {
    db: sqlx::SqlitePool,

    states: Arc<Mutex<std::collections::HashMap<u64, BotState>>>,
}

impl Bot {
    pub async fn new(db: sqlx::SqlitePool) -> Self {
        Bot {
            db,
            states: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn switch_mode(&self, guild_id: u64, mode: BotStateMode) -> Result<(), ()> {
        let mut mode = Arc::new(Mutex::new(mode));
        std::mem::swap(
            &mut self
                .states
                .lock()
                .map_err(|_| log::error!("lock failed"))?
                .entry(guild_id)
                .or_default()
                .mode,
            &mut mode,
        );

        match &*mode.lock().map_err(|_| log::error!("lock failed"))? {
            BotStateMode::Normal => {}
            BotStateMode::CallsignLesson(s) => {
                log::info!("terminating callsign lesson");
                let _ = crate::modes::call_lesson::end(s.clone());
            }
        }
        Ok(())
    }
}

#[async_trait]
impl EventHandler for Bot {
    // Botが起動したときに走る処理
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(serenity::model::gateway::Activity::listening("7.074MHz"))
            .await;
        log::info!("{} is connected!", ready.user.name);

        let _ = self.register_command_neko(&ctx).await;
        let _ = self.register_commands_vc(&ctx).await;
        let _ = self.register_commands_cw(&ctx).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            log::info!("got command: {}", command.data.name);

            let content = match command.data.name.as_str() {
                "neko" => self
                    .run_command_neko(&command.data.options)
                    .unwrap_or_else(|e| e),
                "cw-join" => self
                    .run_command_join(&ctx, &command)
                    .await
                    .unwrap_or_else(|e| e),
                "cw-leave" => self
                    .run_command_leave(&ctx, &command)
                    .await
                    .unwrap_or_else(|e| e),
                "cw-speed" => self
                    .run_command_speed(&ctx, &command)
                    .await
                    .unwrap_or_else(|e| e),
                "cw-freq" => self
                    .run_command_freq(&ctx, &command)
                    .await
                    .unwrap_or_else(|e| e),
                "cw-start-lesson" => self
                    .run_command_lesson_start(&ctx, &command)
                    .await
                    .unwrap_or_else(|e| e),
                "cw-end-lesson" => self
                    .run_command_lesson_end(&ctx, &command)
                    .await
                    .unwrap_or_else(|e| e),
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
                log::error!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn message(&self, ctx: Context, message: Message) {
        if message.author.bot {
            return;
        }

        let mode = {
            let mut states = match self.states.lock() {
                Ok(s) => s,
                Err(_) => {
                    log::error!("lock failed");
                    return;
                }
            };

            let gid = match message.guild_id {
                Some(gid) => gid.0,
                None => {
                    log::info!("not guild; ignore");
                    return;
                }
            };

            let state = states.entry(gid).or_default();

            if Some(message.channel_id) != state.txt_ch {
                return;
            }

            state.mode.clone()
        };

        log::info!("got message: {}", message.content);

        // NOTE: actually clone not needed but compiler complains: https://github.com/rust-lang/rust/issues/104883
        let mode = (*mode.lock().unwrap()).clone();
        match mode {
            BotStateMode::Normal => {
                let _ = crate::modes::normal::on_message(&ctx, &message, &self.db).await;
            }
            BotStateMode::CallsignLesson(s) => {
                let _ = crate::modes::lesson::on_message(&ctx, &message, s.clone()).await;
            }
        }
    }
}
