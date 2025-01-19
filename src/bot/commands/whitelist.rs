use log::error;
use serenity::all::{CommandOptionType, CreateCommand, CreateCommandOption, CreateInteractionResponseFollowup, ResolvedOption, ResolvedValue};

use crate::{bot::{create_response_with_content, utilities::get_user_id_by_login}, database::PgDatabase, error::Error};

pub fn get_registration() -> CreateCommand {
    CreateCommand::new("whitelist")
        .description("Add user to whitelist, literally `whitelistadd` in-game")
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "add", "Adds to whitelist")
            .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "login", "In-Game Login")
                .required(true)
        )
    )
        .add_option(CreateCommandOption::new(CommandOptionType::SubCommand, "rm", "Removes from whitelist")
            .add_sub_option(CreateCommandOption::new(CommandOptionType::String, "login", "In-Game Login")
                .required(true)
            )
    )
}

fn parse_rm_options(opt: &ResolvedOption) -> Result<WhitelistSubCommand, String> {
    let opt = opt.clone();
    if let ResolvedValue::SubCommand(opt) = opt.value {
        if let Some(ResolvedOption { value: ResolvedValue::String(login), .. }) = opt.first() {
            return Ok(WhitelistSubCommand::Rm { login: login.to_string() })
        }
    }
    Err("Invalid options provided.".to_string())
}
fn parse_add_options(opt: &ResolvedOption) -> Result<WhitelistSubCommand, String> {
    let opt = opt.clone();
    if let ResolvedValue::SubCommand(opt) = opt.value {
        if let Some(ResolvedOption { value: ResolvedValue::String(login), .. }) = opt.first() {
            return Ok(WhitelistSubCommand::Add { login: login.to_string() })
        }
    }
    Err("Invalid options provided.".to_string())
}

pub fn get_options(options: &Vec<ResolvedOption>) -> Result<WhitelistSubCommand, String> {
    if options.len() != 1 {
        return Err("Invalid options count".to_string());
    }

    let subcommand = options.first().unwrap();

    match subcommand.name {
        "add" => parse_add_options(subcommand),
        "rm" => parse_rm_options(subcommand),
        _ => Err("Invalid subcommand.".to_string())
    }
}

pub async fn execute(cmd: WhitelistSubCommand, database: &PgDatabase) -> CreateInteractionResponseFollowup {
    match cmd {
        WhitelistSubCommand::Add { login } => execute_add_cmd(login, database).await,
        WhitelistSubCommand::Rm { login } => execute_rm_cmd(login, database).await,
    }
}

async fn execute_rm_cmd(login: String, db: &PgDatabase) -> CreateInteractionResponseFollowup {
    let uuid = match get_user_id_by_login(&login, db).await {
        Some(id) => id,
        None => return create_response_with_content("No such player found.", true),
    };

    match db.whitelistrm(&uuid).await {
        Ok(rows) => {
            if rows == 0 {
                create_response_with_content(&format!("User {} is not whitelisted.", login), true)
            } else {
                create_response_with_content(&format!("Successfully removed {} from whitelist.", login), true)
            }
        }
        Err(e) => {
            error!("Error removing player from whitelist: {}", e);
            create_response_with_content(&format!("Unable to remove {} from whitelist.", login), true)
        }
    }
}

async fn execute_add_cmd(login: String, db: &PgDatabase) -> CreateInteractionResponseFollowup {
    let uuid = match get_user_id_by_login(&login, db).await {
        Some(id) => id,
        None => return create_response_with_content("No such player found.", true),
    };

    match db.whitelistadd(&uuid).await {
        Ok(_) => create_response_with_content(&format!("Successfully added {} to whitelist.", login), true),
        Err(e) => {
            if let Error::SqlxError(sqlx_err) = e {
                if let Some(db_err) = sqlx_err.into_database_error() {
                    if db_err.is_unique_violation() {
                        return create_response_with_content("Such player is already whitelisted.", true);
                    }
                }
            }
            create_response_with_content(&format!("Unable to add {} to whitelist.", login), true)
        }
    }
}

pub enum WhitelistSubCommand {
    Add { login: String },
    Rm { login: String },
}