use serenity::all::{CreateCommand, CreateCommandOption, CreateInteractionResponse, ResolvedOption, ResolvedValue};

use crate::{bot::{create_response_with_content, utilities::get_user_id_by_login}, database::PgDatabase, error::Error};

pub fn get_registration() -> CreateCommand {
    CreateCommand::new("whitelistadd")
        .description("Add user to whitelist, literally `whitelistadd` in-game")
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
    match database.whitelistadd(user_id).await {
        Ok(_) => create_response_with_content(&format!("Successfully added {} to whitelist.", login)),
        Err(e) => {
            if let Error::SqlxError(sqlx_err) = e {
                if let Some(db_err) = sqlx_err.into_database_error() {
                    if db_err.is_unique_violation() {
                        return create_response_with_content("Such player is already whitelisted.");
                    }
                }
            }
            create_response_with_content(&format!("Unable to add {} to whitelist.", login))
        }
    }
}