use std::env;

use icalendar::{Calendar, Component, Event};
use serenity::all::{Guild, Ready, ScheduledEvent, UnavailableGuild};
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

    async fn ready(&self, ctx: Context, _: Ready) {
        setup_calendars(ctx).await
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, _: Option<bool>) {
        //update_calendar(ctx)
        println!("guild create");

        // let mut calendar = Calendar::new();

        // for scheduled_event in guild.scheduled_events(ctx.http(), false).await {
        //     // let cal_event = Event::new().summary(scheduled_event.).done();

        //     // calendar.push(cal_event);
        // }
    }

    async fn guild_delete(&self, ctx: Context, _: UnavailableGuild, _: Option<Guild>) {
        update_calendar(ctx)
    }

    async fn guild_scheduled_event_create(&self, ctx: Context, event: ScheduledEvent) {
        update_calendar(ctx)
    }

    async fn guild_scheduled_event_update(&self, ctx: Context, event: ScheduledEvent) {
        update_calendar(ctx)
    }

    async fn guild_scheduled_event_delete(&self, ctx: Context, event: ScheduledEvent) {
        update_calendar(ctx)
    }
}

fn update_calendar(ctx: Context) {
    dbg!("Update calendar");
}

async fn setup_calendars(ctx: Context) {
    dbg!("Setup calendars");

    for guild_id in ctx.cache.guilds() {
        dbg!(guild_id);

        if let Ok(events) = guild_id.scheduled_events(ctx.http(), false).await {
            for event in events {
                println!("{}", event.name);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILDS
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
