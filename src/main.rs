use std::env;

use serenity::all::{GatewayEvent, ScheduledEvent};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {why:?}");
            }
        }
    }

    async fn guild_scheduled_event_create(&self, ctx: Context, event: ScheduledEvent) {
        handle_scheduled_event(ctx, event)
    }

    async fn guild_scheduled_event_update(&self, ctx: Context, event: ScheduledEvent) {
        handle_scheduled_event(ctx, event)
    }

    async fn guild_scheduled_event_delete(&self, ctx: Context, event: ScheduledEvent) {
        handle_scheduled_event(ctx, event)
    }
}

fn handle_scheduled_event(ctx: Context, event: ScheduledEvent) {
    dbg!("Did smth: ", event);
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_SCHEDULED_EVENTS
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
