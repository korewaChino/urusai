use sqlx::Executor;
use sqlx::SqlitePool;
use sqlx::Connection;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use sqlx::query;
use crate::Database;

// Let's make a macro to automate the tedious task of unwrapping the database context
macro_rules! db {
    ($ctx: expr) => {
        $ctx.data.read().await.get::<Database>().unwrap().clone().read().await.clone()
    }
}


// Config structs for interacting with the SQLite database

pub struct Server {
    ctx: Context,
    msg: Message,
    pub id: i64,
    pub voice_channel: Option<i64>,
    pub text_channel: Option<i64>,
}

impl Server {
    pub async fn from_db(ctx: &Context, msg: &Message) -> Self {
        // query
        let database = db!(ctx);
        let server_id = msg.guild_id.unwrap().0 as i64;
        let server_query = query!("SELECT * FROM servers WHERE id = ?", server_id).fetch_one(&database).await;
        if server_query.is_err() {
            // Then it probably doesn't exist
            // Add it to the database
            query!("INSERT INTO servers (id) VALUES (?)", server_id).execute(&database).await.unwrap();
            // Then return the defaults
            return Server {
                ctx: ctx.clone(),
                msg: msg.clone(),
                id: server_id,
                voice_channel: None,
                text_channel: None,
            };
        }
        let server = server_query.unwrap();
        Server {
            ctx: ctx.clone(),
            msg: msg.clone(),
            id: server.id,
            voice_channel: server.voice_channel,
            text_channel: server.text_channel,
        }
    }

    pub async fn update_voice_channel(mut self, channel_id: i64) -> Self {
        let database = db!(&self.ctx);

        let server_id = self.id;
        query!("UPDATE servers SET voice_channel = ? WHERE id = ?", channel_id, server_id).execute(&database).await.unwrap();

        self.voice_channel = Some(channel_id);
        self
    }

    pub async fn update_text_channel(mut self, channel_id: i64) -> Self {
        let database = db!(&self.ctx);

        let server_id = self.id;
        query!("UPDATE servers SET text_channel = ? WHERE id = ?", channel_id, server_id).execute(&database).await.unwrap();
        self.text_channel = Some(channel_id);
        self
    }
}

pub struct User {
    ctx: Context,
    msg: Message,
    pub id: i64,
    pub server_id: i64,
    pub voice: String,
}

impl User {
    pub async fn from_db(ctx: &Context, msg: &Message) -> Self {
        // query
        let database = db!(ctx);
        let user_id = msg.author.id.0 as i64;
        let server_id = msg.guild_id.unwrap().0 as i64;
        let user_query = query!("SELECT * FROM users WHERE id = ? AND server_id = ?", user_id, server_id).fetch_one(&database).await;
        if user_query.is_err() {
            // Then it probably doesn't exist
            // Add it to the database
            let voice = "tiktok-en_us_002";
            query!("INSERT INTO users (id, server_id, voice) VALUES (?, ?, ?)", user_id, server_id, voice).execute(&database).await.unwrap();
            // Then return the defaults
            User {
                ctx: ctx.clone(),
                msg: msg.clone(),
                id: user_id,
                server_id: server_id,
                voice: voice.to_string(),
            }
        } else {
            let user = user_query.unwrap();
            User {
                ctx: ctx.clone(),
                msg: msg.clone(),
                id: user.id.unwrap(),
                server_id: user.server_id.unwrap(),
                voice: user.voice.unwrap(),
            }
        }
    }

    pub async fn update_voice(mut self, voice: &str) -> Self {
        let database = db!(&self.ctx);
        let user_id = self.id;
        let server_id = self.server_id;
        query!("UPDATE users SET voice = ? WHERE id = ? AND server_id = ?", voice, user_id, server_id).execute(&database).await.unwrap();
        self.voice = voice.to_string();
        self
    }
}