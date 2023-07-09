use serenity::model::channel::Message;
use serenity::prelude::Context;
use sqlx::Row;

pub async fn on_message(ctx: &Context, msg: &Message, db: &sqlx::SqlitePool) -> Result<(), ()> {
    let s = &msg.content;
    if s.starts_with(";") { return Ok(()); }

    let speed_cfgs = sqlx::query("select * from cw_speed where id = ?")
        .bind(msg.author.id.to_string())
        .fetch_all(db)
        .await
        .map_err(|e| log::error!("query failed: {}", e))?;

    let (speed, freq) = speed_cfgs.first().map(|row| (row.get::<f32, _>("speed"), row.get::<f32, _>("freq"))).unwrap_or((20.0, 800.0));

    let man = songbird::get(&ctx).await
        .expect("init songbird").clone();

    let handler = man.get(msg.guild_id.ok_or_else(|| log::error!("no guild"))?);
    if let Some(handler) = handler {
        let mut handler = handler.lock().await;
        let source = crate::cw_audio::CWAudioPCM::new(s.to_string(), speed, freq).to_input();
        handler.play_source(source);
    }
    Ok(())
}
