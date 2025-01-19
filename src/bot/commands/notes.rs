use log::error;
use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateInteractionResponseFollowup, ResolvedOption, ResolvedValue
};

use crate::{
    bot::{
        create_response_with_content,
        utilities::{generate_random_colour, get_user_id_by_login, resolve_user_name},
    },
    database::{AdminNote, AdminNoteShort, PgDatabase},
};

static SHORT_MSG_LEN_SYMBOLS: usize = 50;

pub fn get_registration() -> CreateCommand {
    CreateCommand::new("notes")
        .description("User notes in game")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "list", "Lists all notes of this user")
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::String, "login", "In-game login")
                        .required(true),
                ),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "note", "Gets a specific note by ID")
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::Integer, "id", "ID of the note from the 'list' subcommand")
                        .required(true),
                ),
        )
}

pub fn get_options(options: &[ResolvedOption]) -> Result<NotesSubcommand, String> {
    if options.len() != 1 {
        return Err("Invalid options count".to_string());
    }

    let subcommand = options.first().unwrap();

    match subcommand.name {
        "list" => parse_list_subcommand(subcommand),
        "note" => parse_note_subcommand(subcommand),
        _ => Err("Invalid subcommand type".to_string()),
    }
}

fn parse_note_subcommand(opt: &ResolvedOption) -> Result<NotesSubcommand, String> {
    if let ResolvedValue::SubCommand(suboptions) = &opt.value {
        if let Some(ResolvedOption {
            value: ResolvedValue::Integer(id), ..
        }) = suboptions.first()
        {
            return Ok(NotesSubcommand::Note { id: *id });
        }
    }
    Err("Invalid or missing 'id' option".to_string())
}

fn parse_list_subcommand(opt: &ResolvedOption) -> Result<NotesSubcommand, String> {
    if let ResolvedValue::SubCommand(suboptions) = &opt.value {
        if let Some(ResolvedOption {
            value: ResolvedValue::String(login), ..
        }) = suboptions.first()
        {
            return Ok(NotesSubcommand::List { login: login.to_string() });
        }
    }
    Err("Invalid or missing 'login' option".to_string())
}

pub async fn execute(command: NotesSubcommand, db: &PgDatabase) -> CreateInteractionResponseFollowup {
    match command {
        NotesSubcommand::Note { id } => execute_note_by_id(id as i32, db).await,
        NotesSubcommand::List { login } => execute_list_by_login(login, db).await,
    }
}

async fn execute_list_by_login(login: String, db: &PgDatabase) -> CreateInteractionResponseFollowup {
    let uuid = match get_user_id_by_login(&login, db).await {
        Some(id) => id,
        None => return create_response_with_content("No such player found.", true),
    };

    match db.get_notes_list(&uuid).await {
        Ok(notes) => {
            let description = notes
                .iter()
                .map(format_short_note_summary)
                .collect::<Vec<String>>()
                .join("\n");

            let embed = CreateEmbed::new()
                .title(format!("Notes for `{}`", login))
                .description(description)
                .color(generate_random_colour())
                .footer(CreateEmbedFooter::new("VoidRelay by JerryImMouse"));

            CreateInteractionResponseFollowup::new().add_embed(embed).ephemeral(true)
        }
        Err(err) => {
            error!("Error retrieving notes for {}: {}", login, err);
            create_response_with_content("Failed to retrieve notes.", true)
        }
    }
}

async fn execute_note_by_id(id: i32, db: &PgDatabase) -> CreateInteractionResponseFollowup {
    match db.get_note_by_id(id).await {
        Ok(Some(note)) => {
            let created_by = resolve_user_name(db, &note.created_by_id).await;
            let last_edited_by = resolve_user_name(db, &note.last_edited_by_id).await;
            let player_user = resolve_user_name(db, &note.player_user_id).await;

            let formatted_note = format_admin_note(&note, &created_by, &last_edited_by);

            let embed = CreateEmbed::new()
                .title(format!("Note `{}` for `{}`", id, player_user))
                .description(formatted_note)
                .color(generate_random_colour())
                .footer(CreateEmbedFooter::new("VoidRelay by JerryImMouse"));

            CreateInteractionResponseFollowup::new().add_embed(embed).ephemeral(true)
        }
        Ok(None) => create_response_with_content(&format!("Note with ID `{}` not found.", id), true),
        Err(err) => {
            error!("Error fetching note with ID {}: {}", id, err);
            create_response_with_content("Error occurred while fetching the note.", true)
        }
    }
}

fn format_short_note_summary(note: &AdminNoteShort) -> String {
    let short_msg = if note.message.chars().count() <= SHORT_MSG_LEN_SYMBOLS {
        note.message.clone()
    } else {
        let end_index = note
            .message
            .char_indices()
            .nth(SHORT_MSG_LEN_SYMBOLS)
            .map(|(idx, _)| idx)
            .unwrap_or_else(|| note.message.len());
        format!("{}...", &note.message[..end_index])
    };

    format!("**{}**. {}", note.admin_notes_id, short_msg)
}

fn format_admin_note(note: &AdminNote, created_by: &str, last_edited_by: &str) -> String {
    let mut formatted = format!(
        r#"✨ **Round ID:** {}
👤 **Created By:** {}
📅 **Created At:** {}
✍️ **Last Edited By:** {}
🕒 **Last Edited At:** {}
🗑️ **Deleted:** {}
"#,
        note.round_id,
        created_by,
        note.created_at,
        last_edited_by,
        note.last_edited_at,
        if note.deleted { "Yes" } else { "No" },
    );

    if let Some(deleted_at) = note.deleted_at {
        formatted.push_str(&format!("🗓️ **Deleted At:** {}\n", deleted_at));
    }

    formatted.push_str(if note.secret { 
        "🔒 **Secret:** Yes\n" 
    } else { 
        "🔓 **Secret:** No\n" 
    });

    if let Some(expiration_time) = note.expiration_time {
        formatted.push_str(&format!("⏳ **Expiration Time:** {}\n", expiration_time));
    }

    formatted.push_str(&format!("\n📝 **Message:**\n{}", note.message));

    formatted
}

#[derive(Debug)]
pub enum NotesSubcommand {
    List { login: String },
    Note { id: i64 },
}
