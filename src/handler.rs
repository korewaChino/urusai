
use std::sync::Arc;

use serenity::async_trait;
use sqlx::query;
use crate::config::{Server, self, User};
use serenity::http::CacheHttp;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::model::prelude::Ready;
use serenity::{framework::standard::macros::{command, group}, client::EventHandler};
use serenity::framework::standard::{StandardFramework, CommandResult};
use lazy_static::lazy_static;
use tokio::join;
use log::{debug, info, warn, error};
use crate::tts::TTS;
pub struct Database;

impl TypeMapKey for Database {
    type Value = Arc<RwLock<sqlx::SqlitePool>>;
}




pub struct Handler;
use serenity::model::{channel, guild};


#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let database = {
            let data = ctx.data.read().await;
            data.get::<Database>().unwrap().clone().read().await.clone()
        };


        for server in ready.guilds {
            let serverid = server.id.0 as i64;
            sqlx::query!("INSERT OR IGNORE INTO servers (id) VALUES (?)", serverid).execute(&database).await.unwrap();
        }
        println!("{} is connected!", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let database = {
            let data = ctx.data.read().await;
            data.get::<Database>().unwrap().clone().read().await.clone()
        };
        let guild = msg.guild(&ctx.cache).unwrap();
        
        let db = config::Server::from_db(&ctx, &msg).await;
        let manager = songbird::get(&ctx).await
                        .expect("Could not get songbird manager");
        // Check the channel in which the message was sent
        let server_id = msg.guild_id.unwrap().0 as i64;
        let channel_id = msg.channel_id.0 as i64;
        // check if the message is in the server's text channel
        let channel_id_query = sqlx::query!("SELECT text_channel FROM servers WHERE id = ?", server_id).fetch_one(&database).await.unwrap();
        // check if the message is in the server's voice channel
        if channel_id_query.text_channel.is_some() {
            // Match channel_id to the text channel
            if channel_id_query.text_channel.unwrap() == channel_id {
                // if bot
                if msg.author.bot {
                    return;
                // else if command
                } else if msg.content.starts_with("tts!") {
                    return;
                } else {
                    println!("{}: {}", msg.author.name, msg.content);
                    let database = User::from_db(&ctx, &msg).await;
                    let tts_file = TTS::request(&database.voice, &msg.content, Some(&msg)).await.download().await.unwrap();
                    // TODO: Put the message in the queue
                    if let Some(handler_lock) = manager.get(guild.id) {
                        let mut handler = handler_lock.lock().await;
                        let source = match songbird::ffmpeg(&tts_file).await {
                            Ok(source) => source,
                            Err(e) => {
                                println!("Error Starting Source: {}", e);

                                return;
                            }
                        };
                        handler.play_source(source);
                        // Then delete the file
                        //std::fs::remove_file(&tts_file).unwrap();

                    }
                }
            }
        }
    }
}



#[group]
#[commands(join, setvoice, leave)]
struct General;






#[command]
#[description("Join a voice channel")]
#[usage("<channel>")]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let user = msg.author.clone();
    let guild = msg.guild(&ctx.cache).unwrap();
    let server_id = guild.id.0 as i64;


    // Get the handler's database connection
    /* let database = {
        let data = ctx.data.read().await;
        data.get::<Database>().unwrap().clone().read().await.clone()
    }; */

    let voice_state = guild.voice_states.get(&user.id);
    if voice_state.is_some() {
        // Join the voice channel
        let channel = voice_state.unwrap().channel_id.unwrap();
        // Join the vc
        let manager = songbird::get(ctx).await.unwrap();
        let _handler = manager.join(guild.id, channel).await;

        // Update the database with the new voice channel and text channel
        let channel_id = msg.channel_id.0 as i64;
        let voice_channel = channel.0 as i64;
        // Set the database
        //sqlx::query!("UPDATE servers SET voice_channel = ? WHERE id = ?", voice_channel, server_id).execute(&database).await.unwrap();
        //sqlx::query!("UPDATE servers SET text_channel = ? WHERE id = ?", channel_id, server_id).execute(&database).await.unwrap();
        // Reply
        msg.reply(&ctx, format!("Joining Channel <#{}>",&channel)).await?;
        let db = config::Server::from_db(ctx, &msg).await
            .update_text_channel(channel_id)
            .await
            .update_voice_channel(voice_channel)
            .await;
    } else {
        msg.reply(&ctx, "You are not in a voice channel! Please join a channel.").await?;
    }
    Ok(())
}

#[command]
#[description("Leave a voice channel")]
#[usage("")]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            msg.reply(&ctx, format!("Error: {}", e)).await?;
        }
        // Clear the database entry
        msg.reply(&ctx, "Left voice channel.").await?;
        let database = Server::from_db(&ctx, &msg).await
            .update_voice_channel(0)
            .await;
    } else {
        msg.reply(&ctx, "Currently not in a voice channel!").await?;
    }

    Ok(())
}

#[command]
#[description("Set the TTS voice")]
#[usage("<voice>")]
async fn setvoice(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();

    // Get the argument
    let args = msg.content.split_whitespace().collect::<Vec<&str>>();
    let voice = args[1];

    let _database = User::from_db(&ctx, &msg).await
        .update_voice(voice)
        .await;

    msg.reply(&ctx, format!("Set voice to `{}`", voice)).await?;
    Ok(())
}

