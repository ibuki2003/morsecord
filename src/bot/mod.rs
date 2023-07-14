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
    Lesson(Arc<Mutex<crate::modes::lesson::LessonModeState>>),
}

impl std::default::Default for BotStateMode {
    fn default() -> Self {
        BotStateMode::Normal
    }
}

impl BotStateMode {
    // NOTE: call this when you want to discard the state
    // Drop trait is not implemented because BotStateMode is cloned at Bot::message. :(
    pub fn discard(&self) {
        match self {
            BotStateMode::Normal => {}
            BotStateMode::Lesson(s) => {
                log::info!("terminating callsign lesson");
                let _ = crate::modes::lesson::end(s.clone());
            }
        }
    }
}

struct BotState {
    txt_ch: serenity::model::id::ChannelId,
    mode: Arc<Mutex<BotStateMode>>,
}

impl Clone for BotState {
    fn clone(&self) -> Self {
        BotState {
            txt_ch: self.txt_ch,
            mode: self.mode.clone(),
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

    // called when bot joins to a call
    pub fn add_call_state(
        &self,
        guild_id: u64,
        ch: serenity::model::prelude::ChannelId,
    ) -> Result<(), String> {
        let mut states = self.states.lock().map_err(|_| {
            log::error!("lock failed");
            "internal error".to_string()
        })?;

        if states.contains_key(&guild_id) {
            return Err("already in call".to_string());
        }

        states.insert(
            guild_id,
            BotState {
                txt_ch: ch,
                mode: Arc::new(Mutex::new(BotStateMode::Normal)),
            },
        );

        Ok(())
    }

    pub fn switch_mode(&self, guild_id: u64, mode: BotStateMode) -> Result<(), String> {
        let mut mode = Arc::new(Mutex::new(mode));
        std::mem::swap(
            &mut self
                .states
                .lock()
                .map_err(|_| {
                    log::error!("lock failed");
                    "internal error".to_string()
                })?
                .get_mut(&guild_id)
                .ok_or("not in call".to_string())?
                .mode,
            &mut mode,
        );

        mode.lock()
            .map_err(|_| {
                log::error!("lock failed");
                "internal error".to_string()
            })?
            .discard();

        Ok(())
    }

    pub fn get_call_txt_ch(&self, guild_id: u64) -> Result<serenity::model::id::ChannelId, String> {
        let states = self.states.lock().map_err(|_| {
            log::error!("lock failed");
            "internal error".to_string()
        })?;

        Ok(states
            .get(&guild_id)
            .ok_or("not in call".to_string())?
            .txt_ch)
    }

    pub fn get_call_mode(&self, guild_id: u64) -> Result<Arc<Mutex<BotStateMode>>, String> {
        let states = self.states.lock().map_err(|_| {
            log::error!("lock failed");
            "internal error".to_string()
        })?;

        Ok(states
            .get(&guild_id)
            .ok_or("not in call".to_string())?
            .mode
            .clone())
    }

    pub fn erase_call_state(&self, guild_id: u64) -> Result<(), String> {
        let mut states = self.states.lock().map_err(|_| {
            log::error!("lock failed");
            "internal error".to_string()
        })?;

        let state = states.remove(&guild_id).ok_or("not in call".to_string())?;

        state
            .mode
            .lock()
            .map_err(|_| {
                log::error!("lock failed");
                "internal error".to_string()
            })?
            .discard();

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
        let _ = self.register_commands_cw_lesson(&ctx).await;
        log::info!("commands registered");
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            log::info!("got command: {}", command.data.name);

            let content = match command.data.name.as_str() {
                "neko" => self.run_command_neko(&command.data.options),
                "cw-join" => self.run_command_join(&ctx, &command).await,
                "cw-leave" => self.run_command_leave(&ctx, &command).await,
                "cw-speed" => self.run_command_speed(&ctx, &command).await,
                "cw-freq" => self.run_command_freq(&ctx, &command).await,
                "cw-start-lesson" => self.run_command_lesson_start(&ctx, &command).await,
                "cw-end-lesson" => self.run_command_lesson_end(&ctx, &command).await,
                _ => Ok("not implemented :(".to_string()),
            }
            .unwrap_or_else(|e| e);

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

        let Some(gid) = message.guild_id else {
            log::info!("message not in guild");
            return;
        };

        let Ok(cid) = self.get_call_txt_ch(gid.0) else {
            return;
        };

        if cid != message.channel_id {
            return;
        }

        let mode = match self.get_call_mode(gid.0) {
            Ok(m) => m,
            Err(_) => {
                return;
            }
        };

        log::info!("got message: {}", message.content);

        // NOTE: actually clone not needed but compiler complains: https://github.com/rust-lang/rust/issues/104883
        let mode = (*mode.lock().unwrap()).clone();
        match mode {
            BotStateMode::Normal => {
                let _ = crate::modes::normal::on_message(&ctx, &message, &self.db).await;
            }
            BotStateMode::Lesson(s) => {
                let _ = crate::modes::lesson::on_message(&ctx, &message, s.clone()).await;
            }
        }
    }
}
