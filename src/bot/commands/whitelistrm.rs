use log::error;
use serenity::all::{CreateCommand, CreateCommandOption, CreateInteractionResponse, ResolvedOption, ResolvedValue};

use crate::{bot::{create_response_with_content, utilities::get_user_id_by_login}, database::PgDatabase};

pub fn get_registration() -> CreateCommand {
    CreateCommand::new("whitelistrm")
        .description("Removes user from whitelist, literally `whitelistrm` in-game")
        .add_option(CreateCommandOption::new(
            serenity::all::CommandOptionType::String, 
            "login", 
            "In-game login of the player"
        )
        .required(true)
    )
}

pub fn get_options(options: &Vec<ResolvedOption>) -> Result<String, String> {
    if options.len() != 1 {
        return Err("Invalid options count".to_string());
    }

    let login = options.first();
    if let Some(login) = login {
        if let ResolvedValue::String(login) = login.value {
            return Ok(login.to_string());
        }
    }

    Err("Unable to retrieve login option".to_string())
}

pub async fn execute(login: String, database: &PgDatabase) -> CreateInteractionResponse {
    let user_id = match get_user_id_by_login(&login, database).await {
        Some(user_id) => user_id,
        None => return create_response_with_content(&format!("User {} is not found", login))
    };

    // execute db command
    match database.whitelistrm(user_id).await {
        Ok(rows) => {
            if rows == 0 {
                create_response_with_content(&format!("User {} is not whitelisted.", login))
            } else {
                create_response_with_content(&format!("Successfully removed {} from whitelist.", login))
            }
        }
        Err(e) => {
            error!("Error removing player from whitelist: {}", e);
            create_response_with_content(&format!("Unable to remove {} from whitelist.", login))
        }
    }
}