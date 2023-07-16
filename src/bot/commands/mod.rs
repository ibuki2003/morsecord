pub mod cw;
pub mod cw_lesson;
pub mod neko;
pub mod vc;

use anyhow::Context as _;
use serenity::json::Value;

pub fn get_value_f64(v: &Option<Value>) -> anyhow::Result<f64> {
    v.as_ref()
        .context("empty value error")
        .and_then(|v| v.as_f64().context("type error"))
}

pub fn get_value_i64(v: &Option<Value>) -> anyhow::Result<i64> {
    v.as_ref()
        .context("empty value error")
        .and_then(|v| v.as_i64().context("type error"))
}
