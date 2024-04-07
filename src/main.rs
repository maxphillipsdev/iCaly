use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use icalendar::{Calendar, CalendarDateTime, Component, DatePerhapsTime, Event, EventLike};
use serenity::all::{Guild, GuildId, Ready, ScheduledEvent, UnavailableGuild};
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
        setup_calendars(&ctx).await;
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, _: Option<bool>) {
        publish_calendar(&ctx, guild.id).await;
    }

    async fn guild_delete(&self, ctx: Context, _: UnavailableGuild, _: Option<Guild>) {
        // let _ = publish_calendar(ctx, guild.id).await;
    }

    async fn guild_scheduled_event_create(&self, ctx: Context, event: ScheduledEvent) {
        publish_calendar(&ctx, event.guild_id).await;
    }

    async fn guild_scheduled_event_update(&self, ctx: Context, event: ScheduledEvent) {
        publish_calendar(&ctx, event.guild_id).await;
    }

    async fn guild_scheduled_event_delete(&self, ctx: Context, event: ScheduledEvent) {
        publish_calendar(&ctx, event.guild_id).await;
    }
}

async fn publish_calendar(ctx: &Context, guild_id: GuildId) {
    let calendar = build_calendar(&ctx, guild_id).await.unwrap();

    let mut file = File::create(get_calendar_path(guild_id)).unwrap();
    // let Ok(mut file) = File::create(get_calendar_path(guild_id)) else {
    //     return;
    // };

    let _ = file.write_all(calendar.to_string().as_bytes());
}

async fn build_calendar(ctx: &Context, guild_id: GuildId) -> Result<Calendar, SerenityError> {
    let mut calendar = Calendar::new();

    for event in guild_id.scheduled_events(ctx.http(), false).await? {
        calendar.push(build_event(event));
    }

    Ok(calendar.done())
}

fn build_event(event: ScheduledEvent) -> Event {
    println!("{}", event.start_time);

    Event::new()
        .summary(event.name.as_str())
        .description(event.description.unwrap_or("".to_string()).as_str())
        .starts::<DatePerhapsTime>(event.start_time.to_utc().into())
        .ends::<DatePerhapsTime>(event.end_time.unwrap_or(event.start_time).to_utc().into())
        .done()
}

fn get_calendar_path(id: GuildId) -> PathBuf {
    let public_dir = env::args().nth(1).unwrap_or(".".into());
    Path::new(public_dir.as_str()).join(id.to_string())
}

async fn setup_calendars(ctx: &Context) {
    for guild_id in ctx.cache.guilds() {
        publish_calendar(&ctx, guild_id).await;
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
