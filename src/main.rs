use poise::serenity_prelude as serenity;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::time::Duration;

pub struct Data {
    pool: PgPool,
}

pub struct GuildChannel {
    id: i64,
    guild_id: i64,
    channel_id: i64,
    inactive_time: Duration,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[derive(Debug, poise::ChoiceParameter)]
pub enum Period {
    Sec,
    Min,
    Hours,
    Days
}

impl Period {
    fn to_secs(&self) -> u64 {
        match self {
            Period::Sec => 1,
            Period::Min => 60,
            Period::Hours => 60 * 60,
            Period::Days => 60 * 60 * 24,
        }
    }

    fn duration(&self, amount: u64) -> Duration {
        Duration::from_secs( amount * self.to_secs()) 
    }
}

#[poise::command(slash_command, prefix_command)]
async fn consume(
    ctx: Context<'_>,
    #[description = "amount"] amount: Option<u64>,
    #[description = "period"] period: Option<Period>,
) -> Result<(), Error> {
    let period = period.unwrap_or(Period::Days);
    let amount = amount.unwrap_or(30);

    let response = format!("setup to devour messages older than {:?}", period.duration(amount));
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").expect("missing `DATABASE_URL` env variable"))
        .await
        .expect("error connecting to the db");

    sqlx::migrate!().run(&pool).await.expect("failed to run migrations");

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(),consume()],
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { pool })
            })
        });

    framework.run().await.unwrap();
}
