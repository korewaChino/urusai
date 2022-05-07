use std::env;
use std::sync::Arc;
mod handler;
mod config;
mod tts;
use handler::Handler;
use songbird::SerenityInit;

use serenity::async_trait;
use pretty_env_logger;
use serenity::prelude::*;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};
use sqlx::Executor;
use crate::handler::GENERAL_GROUP;
use crate::handler::Database;


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("tts!")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    let database = sqlx::sqlite::SqlitePoolOptions::new()
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename("database.db")
                .create_if_missing(true),
        )
        .await
        .expect("Couldn't connect to database");
    let schema = include_str!("schema.sql");
    let executor = database.clone();
    executor.execute(schema).await.unwrap();

    pretty_env_logger::init();

    let token = env::var("DISCORD_TOKEN").expect("token");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Error creating client");

    // Make a new folder called tts if none exists
    if !std::path::Path::new("tts").exists() {
        std::fs::create_dir("tts").unwrap();
    }

    // Set data for the client
    {
        let mut data = client.data.write().await;

        data.insert::<Database>(Arc::new(RwLock::new(database)));
    }
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
