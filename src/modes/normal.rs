use serenity::model::channel::Message;
use serenity::prelude::Context;
use sqlx::Row;

pub async fn on_message(ctx: &Context, msg: &Message, db: &sqlx::SqlitePool) {
    let s = &msg.content;
    if s.starts_with(";") { return; }

    let speed_cfgs = sqlx::query("select * from cw_speed where id = ?")
        .bind(msg.author.id.to_string())
        .fetch_all(db)
        .await
        .unwrap();

    let speed = speed_cfgs.first().map(|row| row.get::<f32, _>("speed")).unwrap_or(20.0);

    let man = songbird::get(&ctx).await
        .expect("init songbird").clone();

    let handler = man.get(msg.guild_id.unwrap());
    if let Some(handler) = handler {
        let mut handler = handler.lock().await;
        let source = crate::cw_audio::CWAudioPCM::new(s.to_string(), speed, 800.0).to_input();
        handler.play_source(source);
    }
}
