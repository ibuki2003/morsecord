pub mod cw;
pub mod cw_lesson;
pub mod neko;
pub mod vc;

use serenity::json::Value;

pub fn get_value_f64(v: &Option<Value>) -> Result<f64, String> {
    v.as_ref()
        .ok_or_else(|| "empty value error".to_string())
        .and_then(|v| v.as_f64().ok_or_else(|| "type error".to_string()))
}

pub fn get_value_i64(v: &Option<Value>) -> Result<i64, String> {
    v.as_ref()
        .ok_or_else(|| "empty value error".to_string())
        .and_then(|v| v.as_i64().ok_or_else(|| "type error".to_string()))
}
