use std::str::FromStr;

use commands::{ban, notes, whitelist, DiscordCommandType};
use log::{
    debug,
    info,
    error
};

use serenity::{all::{CommandInteraction, Context, CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, EventHandler, GatewayIntents, GuildId, Http, Interaction, Ready}, async_trait, Client};

use crate::{config::Config, database::PgDatabase, error::Error};

pub mod utilities;
pub mod commands;

pub struct DiscordBot {
    config: Config,
    db: PgDatabase, // ss14 database connection
}

#[async_trait]
impl EventHandler for DiscordBot {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            debug!("Recieved command interaction: {} {:?}", command.data.name, command.data.options);
            self.handle_command_interaction(ctx, command).await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Bot {} connected and ready to handle interactions!", ready.user.name);
        let result = Self::register_commands(&ctx.http, self.config.guild(), vec![
            whitelist::get_registration(),
            notes::get_registration(),
            ban::get_registration(),
        ]).await;

        if let Err(e) = result {
            error!("Error happened registering interaction commands: {}", e);
        }
    }
}

impl DiscordBot {
    pub fn new(config: &Config) -> Result<Self, Error> {
        let db = PgDatabase::new(config.cstr())?;
        info!("Connected to SS14 database.");
        Ok(Self { db, config: config.clone() })
    }

    pub async fn start(self) {
        let token = self.config.token();

        let mut client = Client::builder(token, GatewayIntents::empty())
            .event_handler(self).await.expect("Unable to create serenity client");

        if let Err(e) = client.start().await {
            error!("Serenity client error: {}", e);
        }
    }

    async fn register_commands(http: &Http, guild_id: &str, commands: Vec<serenity::all::CreateCommand>) -> Result<(), Error> {
        let guild_id = GuildId::new(guild_id.parse().expect("Invalid guild_id type"));
        let commands = guild_id.set_commands(http, commands).await?;

        info!("Registered {} commands: {:?}", commands.len(), commands.iter().map(|c| c.name.to_string()).collect::<Vec<_>>());

        Ok(())
    }

    async fn handle_command_interaction(&self, ctx: Context, command: CommandInteraction) {
        let command_type = DiscordCommandType::from_str(&command.data.name);
        if command_type.is_err() {
            error!("Invalid interaction type provided: {}", command.data.name);
            create_response("Invalid interaction type!", ctx, command).await;
            return;
        }

        if let Err(e) = command.create_response(&ctx.http, CreateInteractionResponse::Defer(
            CreateInteractionResponseMessage::new().content("Your request is processing...").ephemeral(true)
        )).await {
            error!("Error creating defer response: {e}");
            return;
        }

        let command_type = command_type.unwrap();
        let response = match command_type {
            DiscordCommandType::Whitelist => {
                let result = whitelist::get_options(&command.data.options());
                match result {
                    Ok(options) => whitelist::execute(options, &self.db).await,
                    Err(e) => create_response_with_content(&e, true)
                }
            },
            DiscordCommandType::Notes => {
                let result = notes::get_options(&command.data.options());
                match result {
                    Ok(options) => notes::execute(options, &self.db).await,
                    Err(e) => create_response_with_content(&e, true)
                }
            }
            DiscordCommandType::Ban => {
                let result = ban::get_options(&command.data.options(), &command);
                match result {
                    Ok(options) => ban::execute(options, &self.db, &self.config).await,
                    Err(e) => create_response_with_content(&e, true)
                }
            }
        };

        if let Err(e) = command.create_followup(&ctx.http, response).await {
            error!("Error creating response: {e}");
        }
    }
}

fn create_response_with_content(s: &str, eph: bool) -> CreateInteractionResponseFollowup {
    CreateInteractionResponseFollowup::new().content(s).ephemeral(eph)
}

async fn create_response(s: &str, ctx: Context, command: CommandInteraction) {
    let builder = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content(s).ephemeral(true));
    if let Err(e) = command.create_response(&ctx.http, builder).await {
        error!("Error creating response: {e}");
    }
}