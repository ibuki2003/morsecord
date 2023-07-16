use std::{fs::File, io::BufReader, usize};

use serenity::framework::StandardFramework;
use serenity::prelude::Client;

use songbird::SerenityInit;

use morsecord::bot::Bot;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Token {
    token: String,
}

fn get_token(file_name: &str) -> anyhow::Result<String> {
    let file = File::open(file_name)?;
    let reader = BufReader::new(file);
    let t: Token = serde_json::from_reader(reader)?;
    Ok(t.token)
}

fn init_logger() {
    let base_config = fern::Dispatch::new();

    let stderr_config = fern::Dispatch::new()
        .level(log::LevelFilter::Warn)
        .level_for("morsecord", log::LevelFilter::Info)
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::io::stderr());

    base_config.chain(stderr_config).apply().unwrap();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger();
    let token = get_token("config.json")?;
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
        .event_handler(Bot::new(db).await)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    tokio::spawn(async move {
        if let Err(why) = client.start().await {
            log::error!("Client error: {:?}", why);
        }
    });

    tokio::signal::ctrl_c().await.unwrap();
    log::info!("Received Ctrl-C, shutting down.");

    Ok(())
}
