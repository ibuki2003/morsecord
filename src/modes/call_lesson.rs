use std::sync::{Arc, Mutex};

use rand::Rng;
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::model::prelude::{ChannelId, GuildId};
use serenity::prelude::Context;

pub struct CallLessonModeState {
    speed_range: std::ops::Range<f32>,
    freq_range: std::ops::Range<f32>,

    txt_ch: ChannelId,

    last_str: Option<String>,
    last_freq: f32,
    last_speed: f32,

    answered: bool, // to check 1st AC
    is_advancing: bool,
    next_ftr_token: Option<tokio_util::sync::CancellationToken>,
}

impl CallLessonModeState {
    pub fn new(
        speed_range: std::ops::Range<f32>,
        freq_range: std::ops::Range<f32>,
        txt_ch: ChannelId,
    ) -> Self {
        Self {
            speed_range,
            freq_range,
            txt_ch,
            last_str: None,
            last_freq: 0.,
            last_speed: 0.,
            answered: false,
            is_advancing: false,
            next_ftr_token: None,
        }
    }
}

impl Drop for CallLessonModeState {
    fn drop(&mut self) {
        println!("dropped, {:?}", self.next_ftr_token);
        self.next_ftr_token.take().map(|t| t.cancel());
    }
}

pub async fn start(
    ctx: &Context,
    guild: GuildId,
    state: Arc<Mutex<CallLessonModeState>>,
) -> Result<(), ()> {
    let man = songbird::get(&ctx).await.expect("init songbird").clone();

    let call = man.get(guild).ok_or_else(|| log::error!("not in call"))?;
    play_next(call, state).await?;
    Ok(())
}

pub fn end(state: Arc<Mutex<CallLessonModeState>>) -> Result<(), ()> {
    state
        .lock()
        .map_err(|_| log::error!("lock failed"))?
        .next_ftr_token
        .take()
        .map(|t| t.cancel());
    Ok(())
}

pub async fn on_message(
    ctx: &Context,
    msg: &Message,
    state: Arc<Mutex<CallLessonModeState>>,
) -> Result<(), ()> {
    let (s, ans, answered) = {
        let st = state.lock().map_err(|_| log::error!("lock failed"))?;

        if msg.channel_id != st.txt_ch {
            return Ok(());
        }

        let s = msg.content.to_uppercase();

        let ans = match &st.last_str {
            None => return Ok(()),
            Some(ans) => ans.to_uppercase(),
        };

        let answered = st.answered;

        drop(st);
        (s, ans, answered)
    };

    if s == ans {
        msg.react(
            &ctx.http,
            if answered {
                ReactionType::from('⭕')
            } else {
                ReactionType::from('🥇')
            },
        )
        .await
        .map_err(|_| log::error!("react failed"))?;

        let next_token = {
            let mut st = state.lock().map_err(|_| log::error!("lock failed"))?;

            st.answered = true;

            if !st.is_advancing {
                let token = tokio_util::sync::CancellationToken::new();
                st.next_ftr_token.replace(token.clone()).map(|t| t.cancel());
                st.is_advancing = true;
                Some(token)
            } else {
                None
            }
        };

        if let Some(token) = next_token {
            let man = songbird::get(&ctx).await.expect("init songbird").clone();

            let call = man
                .get(msg.guild_id.ok_or_else(|| log::error!("no guild"))?)
                .ok_or_else(|| log::error!("not in call"))?;

            tokio::spawn(async move {
                tokio::select! {
                    _ = token.cancelled() => {},
                    _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                        state.lock().map_err(|_| log::error!("lock failed"))?.is_advancing = false;
                        play_next(call, state.clone()).await?;
                    },
                };
                Ok::<(), ()>(())
            });
        }
    } else {
        msg.react(&ctx.http, ReactionType::from('❌'))
            .await
            .map_err(|_| log::error!("react failed"))?;
    }
    Ok(())
}

async fn play(
    call: Arc<serenity::prelude::Mutex<songbird::Call>>,
    state: Arc<Mutex<CallLessonModeState>>,
) -> Result<(), ()> {
    let mut st = state.lock().map_err(|_| log::error!("lock failed"))?;
    let speed = st.last_speed;
    let freq = st.last_freq;
    let s = &st.last_str;
    let s = match s {
        None => return Ok(()),
        Some(s) => " ".to_string() + &s, // to keep margin between last playback
    };

    let token = tokio_util::sync::CancellationToken::new();
    st.next_ftr_token.replace(token.clone()).map(|t| t.cancel());

    drop(st);

    tokio::spawn(async move {
        loop {
            {
                let mut handler = call.lock().await;
                let source = crate::cw_audio::CWAudioPCM::new(s.clone(), speed, freq).to_input();
                handler.play_only_source(source);
            }

            tokio::select! {
                _ = token.cancelled() => { break; }
                _ = tokio::time::sleep(std::time::Duration::from_secs(20)) => {}
            };
        }
        Ok::<(), ()>(())
    });

    Ok(())
}

pub async fn play_next(
    call: Arc<serenity::prelude::Mutex<songbird::Call>>,
    state: Arc<Mutex<CallLessonModeState>>,
) -> Result<(), ()> {
    let next_str = generate_callsign();

    {
        let mut state = state.lock().map_err(|_| log::error!("lock failed"))?;
        state.last_str = Some(next_str.clone());
        state.last_speed = rand::thread_rng().gen_range(state.speed_range.clone());
        state.last_freq = rand::thread_rng().gen_range(state.freq_range.clone());
        state.answered = false;
    }

    play(call, state).await
}

const ALPHA: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALNUM: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const NUM: &'static str = "0123456789";
const JA_PRF: &'static str = "ADEFGHJKLMNPQRS";

fn rand_char(s: &str) -> &str {
    let i = rand::random::<usize>() % s.len();
    &s[i..i + 1]
}

fn generate_callsign() -> String {
    // TODO: improve algorithm
    let s = match rand::random::<u8>() {
        0..=13 => {
            "7".to_string()
                + rand_char("JKLMN")
                + rand_char(NUM)
                + rand_char(ALPHA)
                + rand_char(ALPHA)
                + rand_char(ALPHA)
        }
        14 => {
            "8".to_string()
                + rand_char("JN")
                + rand_char(NUM)
                + rand_char(ALNUM)
                + rand_char(ALNUM)
                + rand_char(ALNUM)
        }
        15..=18 => "JA".to_owned() + rand_char(NUM) + rand_char(ALPHA) + rand_char(ALPHA),
        19 => "JR6".to_owned() + rand_char(ALPHA) + rand_char(ALPHA),
        20..=255 => {
            "J".to_string()
                + rand_char(JA_PRF)
                + rand_char(NUM)
                + rand_char(ALPHA)
                + rand_char(ALPHA)
                + rand_char(ALPHA)
        }
    };

    if rand::random::<u8>() < 50 {
        s + "/" + rand_char(NUM)
    } else {
        s
    }
}
