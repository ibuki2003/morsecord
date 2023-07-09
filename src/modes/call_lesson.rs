use std::sync::{Arc, Mutex};

use serenity::model::channel::Message;
use serenity::model::prelude::{GuildId, ChannelId};
use serenity::prelude::Context;
use serenity::model::channel::ReactionType;
use rand::Rng;

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
    pub fn new(speed_range: std::ops::Range<f32>, freq_range: std::ops::Range<f32>, txt_ch: ChannelId) -> Self {
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

pub async fn start(ctx: &Context, guild: GuildId, state: Arc<Mutex<CallLessonModeState>>) {
    let man = songbird::get(&ctx).await
        .expect("init songbird").clone();

    let call = man.get(guild).unwrap();
    play_next(call, state).await;
}

pub fn end(state: Arc<Mutex<CallLessonModeState>>) {
    state.lock().unwrap().next_ftr_token.take().map(|t| t.cancel());
}

pub async fn on_message(ctx: &Context, msg: &Message, state: Arc<Mutex<CallLessonModeState>>) {
    if msg.channel_id != state.lock().unwrap().txt_ch { return; }

    let s = msg.content.to_uppercase();

    let ans = state.lock().unwrap().last_str.clone();
    let ans = match ans {
        None => return,
        Some(ans) => ans.to_uppercase(),
    };

    if s == ans {
        msg.react(&ctx.http,
                if state.lock().unwrap().answered {
                    ReactionType::from('⭕')
                } else {
                    ReactionType::from('🥇')
                }
            ).await.unwrap();

        state.lock().unwrap().answered = true;

        let next_token = {
            let mut s = state.lock().unwrap();
            if !s.is_advancing {
                s.next_ftr_token.take().map(|t| t.cancel());
                let token = tokio_util::sync::CancellationToken::new();
                s.next_ftr_token = Some(token.clone());
                s.is_advancing = true;
                Some(token)
            } else {
                None
            }
        };

        if next_token.is_some() {
            let token = next_token.unwrap();
            let man = songbird::get(&ctx).await
                .expect("init songbird").clone();

            let call = man.get(msg.guild_id.unwrap());
            if let Some(call) = call {

                tokio::spawn(async move {
                    tokio::select! {
                        _ = token.cancelled() => {
                            state.lock().unwrap().next_ftr_token = None;
                        },
                        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                            state.lock().unwrap().is_advancing = false;
                            play_next(call, state.clone()).await;
                        },
                    };
                });
            }
        }

    } else {
        msg.react(&ctx.http, ReactionType::from('❌')).await.unwrap();
        return;
    }
}

async fn play(call: Arc<serenity::prelude::Mutex<songbird::Call>>, state: Arc<Mutex<CallLessonModeState>>) {
    let (speed, freq, s) = {
        let state = state.lock().unwrap();
        let speed = state.last_speed;
        let freq = state.last_freq;
        let s = &state.last_str;
        let s = match s {
            None => return,
            Some(s) => " ".to_string() + &s, // to keep margin between last playback
        };
        (speed, freq, s)
    };


    state.lock().unwrap().next_ftr_token.take().map(|t| t.cancel());

    let token = tokio_util::sync::CancellationToken::new();
    state.lock().unwrap().next_ftr_token = Some(token.clone());

    tokio::spawn(async move {
        loop {
            {
                let mut handler = call.lock().await;
                let source = crate::cw_audio::CWAudioPCM::new(s.clone(), speed, freq).to_input();
                handler.play_only_source(source);
            }

            tokio::select! {
                _ = token.cancelled() => {
                    state.lock().unwrap().next_ftr_token = None;
                    break;
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(20)) => {}
            };
        }
    });

}

pub async fn play_next(call: Arc<serenity::prelude::Mutex<songbird::Call>>, state: Arc<Mutex<CallLessonModeState>>) {
    let next_str = generate_callsign();

    {
        let mut state = state.lock().unwrap();
        state.last_str = Some(next_str.clone());
        state.last_speed = rand::thread_rng().gen_range(state.speed_range.clone());
        state.last_freq = rand::thread_rng().gen_range(state.freq_range.clone());
    }

    play(call, state).await
}

const ALPHA: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALNUM: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const NUM: &'static str = "0123456789";
const JA_PRF: &'static str = "ABCDEFGHJKLMNPQRS";

fn rand_char(s: &str) -> &str {
    let i = rand::random::<usize>() % s.len();
    &s[i..i + 1]
}

fn generate_callsign() -> String {
    // TODO: improve algorithm
    let s = match rand::random::<u8>() {
        0..=13 => "7".to_string()
                + rand_char("JKLMN")
                + rand_char(NUM)
                + rand_char(ALPHA)
                + rand_char(ALPHA)
                + rand_char(ALPHA),
        14 => "8".to_string()
                + rand_char("JN")
                + rand_char(NUM)
                + rand_char(ALNUM)
                + rand_char(ALNUM)
                + rand_char(ALNUM),
        15..=255 => "J".to_string()
                + rand_char(JA_PRF)
                + rand_char(NUM)
                + rand_char(ALPHA)
                + rand_char(ALPHA)
                + rand_char(ALPHA),
    };

    if rand::random::<u8>() < 50 {
        s + "/" + rand_char(NUM)
    } else {
        s
    }
}
