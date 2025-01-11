use serenity::all::{CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, ResolvedOption, ResolvedValue};

use crate::{bot::{create_response_with_content, utilities::get_user_id_by_login}, database::PgDatabase};

pub fn get_registration() -> CreateCommand {
    CreateCommand::new("notes")
        .description("Get user notes in game")
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

    let notes = database.get_notes(user_id).await.unwrap();

    let mut message = CreateInteractionResponseMessage::new();
    let mut embed = CreateEmbed::new().title(format!("Notes for `{}`", login));
    let mut grouped_notes = String::new();
    let mut count = 0;

    for (idx, note) in notes.iter().enumerate() {
        if idx == 50 {
            break;
        }

        let created_by_id = match database.get_login_by_uuid(note.created_by_id).await {
            Ok(Some(uuid)) => uuid,
            _ => return create_response_with_content("Unable to find `created by` user") 
        };

        let last_edited_by = match database.get_login_by_uuid(note.last_edited_by_id).await {
            Ok(Some(uuid)) => uuid,
            _ => return create_response_with_content("Unable to find `last edited by` user") 
        };
        
        let note_entry = format!(
            "**📋 Round ID**: {}\n**👤 Created By**: {}\n**✏️ Last Edited By**: {}\n**🗑️ Deleted**: {}\n💬 **Message**: {}\n\n",
            note.round_id, created_by_id, last_edited_by, note.deleted, note.message
        );

        grouped_notes.push_str(&note_entry);
        count += 1;

        if count == 2 || idx == notes.len() - 1 {
            embed = embed.field(
                " ",
                grouped_notes.trim(),
                false,
            );
            grouped_notes.clear();
            count = 0;
        }
    }

    embed = embed.footer(CreateEmbedFooter::new("Only the first 50 notes are listed, grouped by 2."));

    message = message.embed(embed);
    CreateInteractionResponse::Message(message)
}