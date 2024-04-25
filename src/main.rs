use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use icalendar::{Calendar, Component, DatePerhapsTime, Event, EventLike};
use serenity::all::{
    ActivityData, ActivityType, Guild, GuildId, GuildScheduledEventUserAddEvent,
    GuildScheduledEventUserRemoveEvent, Ready, ScheduledEvent, UnavailableGuild,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use tokio::fs;

const EVENT_USER_LIMIT: u64 = 5;
const STATUS: &str = "Ping me for a calendar link";
const INSTRUCTIONS: &str = "Add to your calendar: [Google](<https://calendar.google.com/calendar/r/settings/addbyurl>) | [Apple](<https://support.apple.com/102301>) | [Outlook](<https://support.microsoft.com/office/import-or-subscribe-to-a-calendar-in-outlook-com-or-outlook-on-the-web-cff1429c-5af6-41ec-a5b4-74f2c278e98c>) | [Proton](<https://proton.me/support/subscribe-to-external-calendar#subscribe-external-link>)";

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        setup_calendars(&ctx).await;

        ctx.set_activity(Some(ActivityData {
            name: STATUS.to_string(),
            kind: ActivityType::Custom,
            state: Some(STATUS.to_string()),
            url: None,
        }));
    }

    async fn message(&self, ctx: Context, message: Message) {
        if !message.mentions_me(ctx.http()).await.unwrap_or(false) {
            return;
        }

        if let Some(guild_id) = message.guild_id {
            let url = get_calendar_url(guild_id);
            let reply = format!("1. Copy this link: {url}\n2. {INSTRUCTIONS}");
            let _ = message.channel_id.say(ctx.http(), reply).await;
        }
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, _: Option<bool>) {
        publish_calendar(&ctx, guild.id).await;
    }

    async fn guild_delete(&self, _: Context, guild: UnavailableGuild, _: Option<Guild>) {
        delete_calendar(guild.id).await;
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

    async fn guild_scheduled_event_user_add(
        &self,
        ctx: Context,
        subscribed: GuildScheduledEventUserAddEvent,
    ) {
        publish_calendar(&ctx, subscribed.guild_id).await;
    }

    async fn guild_scheduled_event_user_remove(
        &self,
        ctx: Context,
        unsubscribed: GuildScheduledEventUserRemoveEvent,
    ) {
        publish_calendar(&ctx, unsubscribed.guild_id).await;
    }
}

async fn publish_calendar(ctx: &Context, guild_id: GuildId) {
    let calendar = build_calendar(&ctx, guild_id).await.unwrap();

    let Ok(mut file) = File::create(get_calendar_path(guild_id)) else {
        return;
    };

    let _ = file.write_all(calendar.to_string().as_bytes());
}

async fn delete_calendar(guild_id: GuildId) {
    let _ = fs::remove_file(get_calendar_path(guild_id)).await;
}

async fn build_calendar(ctx: &Context, guild_id: GuildId) -> Result<Calendar, SerenityError> {
    let mut calendar = Calendar::new()
        .name(
            &guild_id
                .name(ctx.cache.clone())
                .unwrap_or("Discord".to_string()),
        )
        .done();

    for event in guild_id.scheduled_events(ctx.http(), true).await? {
        calendar.push(build_event(&ctx, &event).await);
    }

    Ok(calendar.done())
}

async fn build_event(ctx: &Context, event: &ScheduledEvent) -> Event {
    println!("{}", event.start_time);

    let mut calendar_event = Event::new()
        .uid(&event.id.to_string())
        .summary(&event.name)
        .description(&build_description(ctx, event).await)
        .url(&get_event_url(event))
        .starts::<DatePerhapsTime>(event.start_time.to_utc().into())
        .ends::<DatePerhapsTime>(event.end_time.unwrap_or(event.start_time).to_utc().into())
        .done();

    if let Some(location) = get_location(ctx, &event).await {
        calendar_event = calendar_event.location(&location).done();
    }

    calendar_event
}

async fn build_description(ctx: &Context, event: &ScheduledEvent) -> String {
    let mut paragraphs = Vec::new();
    let description = event.description.clone().unwrap_or("".to_string());

    if description.len() > 0 {
        paragraphs.push(description);
    }

    let user_count = event.user_count.unwrap_or(0);

    if user_count > 0 {
        let noun = match user_count {
            1 => "person",
            _ => "people",
        };

        let mut user_info = format!("{user_count} {noun} interested");

        let mut users = ctx
            .http
            .get_scheduled_event_users(
                event.guild_id,
                event.id,
                Some(EVENT_USER_LIMIT),
                None,
                Some(true),
            )
            .await
            .unwrap_or(Vec::new())
            .iter()
            .map(|event_user| {
                event_user
                    .member
                    .clone()
                    .and_then(|member| member.nick)
                    .or(event_user.user.global_name.clone())
                    .unwrap_or(event_user.user.name.clone())
            })
            .collect::<Vec<_>>();

        if users.len() > 0 {
            if users.len() < user_count as usize {
                users.push("and others".to_string());
            }

            user_info += &format!(": {}", users.join(", "));
        }

        paragraphs.push(user_info);
    }

    paragraphs.push(get_event_url(event));

    paragraphs.join("\n\n")
}

async fn get_location(ctx: &Context, event: &ScheduledEvent) -> Option<String> {
    match (event.channel_id, &event.metadata) {
        (Some(channel), None) => channel
            .name(ctx.http())
            .await
            .ok()
            .map(|name| format!("🔊 {name}")),
        (None, Some(metadata)) => metadata.location.clone(),
        _ => None,
    }
}

fn get_event_url(event: &ScheduledEvent) -> String {
    format!("https://discord.com/events/{}/{}", event.guild_id, event.id)
}

fn get_calendar_url(id: GuildId) -> String {
    format!("https://icaly.xyz/{id}")
}

fn get_calendar_path(id: GuildId) -> PathBuf {
    let public_dir = env::args().nth(1).unwrap_or(".".into());
    Path::new(&public_dir).join(id.to_string())
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
