use bot::DiscordBot;
use config::Config;
use error::Error;

pub mod bot;
pub mod api;
pub mod config;
pub mod error;
pub mod database;

pub async fn run(config: Config) -> Result<(), Error> {
    let bot = DiscordBot::new(&config)?;

    bot.start().await;
    Ok(())
}