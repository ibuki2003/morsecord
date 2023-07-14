pub mod allja_number;
pub mod callsign;
use std::iter::Iterator;
use std::sync::{Arc, Mutex};

use rand::Rng;
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::model::prelude::GuildId;
use serenity::prelude::Context;

pub struct LessonModeState {
    speed_range: std::ops::RangeInclusive<f32>,
    freq_range: std::ops::RangeInclusive<f32>,

    last_str: Option<String>,
    last_freq: f32,
    last_speed: f32,

    gen: Box<dyn Iterator<Item = String> + Send>,

    answered: bool, // to check 1st AC
    is_advancing: bool,
    next_ftr_token: Option<tokio_util::sync::CancellationToken>,
}

impl LessonModeState {
    pub fn new(
        speed_range: std::ops::RangeInclusive<f32>,
        freq_range: std::ops::RangeInclusive<f32>,
        gen: Box<dyn Iterator<Item = String> + Send>,
    ) -> Self {
        Self {
            speed_range,
            freq_range,
            last_str: None,
            last_freq: 0.,
            last_speed: 0.,
            gen,
            answered: false,
            is_advancing: false,
            next_ftr_token: None,
        }
    }
}

impl Drop for LessonModeState {
    fn drop(&mut self) {
        log::info!("lesson state dropped, {:?}", self.next_ftr_token);
        self.next_ftr_token.take().map(|t| t.cancel());
    }
}

pub async fn start(
    ctx: &Context,
    guild: GuildId,
    state: Arc<Mutex<LessonModeState>>,
) -> Result<(), ()> {
    let man = songbird::get(&ctx).await.expect("init songbird").clone();

    let call = man.get(guild).ok_or_else(|| log::error!("not in call"))?;
    play_next(call, state).await?;
    Ok(())
}

pub fn end(state: Arc<Mutex<LessonModeState>>) -> Result<(), ()> {
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
    state: Arc<Mutex<LessonModeState>>,
) -> Result<(), ()> {
    let (s, ans, answered) = {
        let mut st = state.lock().map_err(|_| log::error!("lock failed"))?;

        let s = msg.content.to_uppercase();

        let ans = match &st.last_str {
            None => return Ok(()),
            Some(ans) => ans.to_uppercase(),
        };

        let answered = st.answered;
        if s == ans {
            st.answered = true;
        }

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

        let man = songbird::get(&ctx).await.expect("init songbird").clone();
        let call = man
            .get(msg.guild_id.ok_or_else(|| log::error!("no guild"))?)
            .ok_or_else(|| log::error!("not in call"))?;
        {
            let mut handler = call.lock().await;
            handler.stop();
        }

        let next_token = {
            let mut st = state.lock().map_err(|_| log::error!("lock failed"))?;

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
    } else if s == "||".to_owned() + &ans + "||" {
        msg.react(&ctx.http, ReactionType::from('⭕'))
            .await
            .map_err(|_| log::error!("react failed"))?;
    } else {
        msg.react(&ctx.http, ReactionType::from('❌'))
            .await
            .map_err(|_| log::error!("react failed"))?;
    }
    Ok(())
}

async fn play(
    call: Arc<serenity::prelude::Mutex<songbird::Call>>,
    state: Arc<Mutex<LessonModeState>>,
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

    let delay_time =
        crate::cw_audio::CWAudioPCM::get_duration(&s, speed) + std::time::Duration::from_secs(10);

    tokio::spawn(async move {
        loop {
            {
                let mut handler = call.lock().await;
                let source = crate::cw_audio::CWAudioPCM::new(s.clone(), speed, freq).to_input();
                handler.play_only_source(source);
            }

            tokio::select! {
                _ = token.cancelled() => { break; }
                _ = tokio::time::sleep(delay_time) => {}
            };
        }
        Ok::<(), ()>(())
    });

    Ok(())
}

pub async fn play_next(
    call: Arc<serenity::prelude::Mutex<songbird::Call>>,
    state: Arc<Mutex<LessonModeState>>,
) -> Result<(), ()> {
    {
        let mut state = state.lock().map_err(|_| log::error!("lock failed"))?;

        let next_str = match state.gen.next() {
            Some(s) => s,
            None => {
                // TODO: switch mode no normal
                return Ok(()); // no more
            }
        };

        log::info!("next: {}", next_str);

        state.last_str = Some(next_str.clone());
        state.last_speed = rand::thread_rng().gen_range(state.speed_range.clone());
        state.last_freq = rand::thread_rng().gen_range(state.freq_range.clone());
        state.answered = false;
    }

    play(call, state).await
}

fn rand_char(s: &str) -> &str {
    let i = rand::thread_rng().gen_range(0..s.len());
    &s[i..i + 1]
}
