use log::error;

use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, 
    CreateInteractionResponse, CreateInteractionResponseMessage, ResolvedOption, ResolvedValue
};

use uuid::Uuid;

use crate::{
    bot::{
        create_response_with_content, 
        utilities::{
            generate_random_colour, get_user_id_by_login, resolve_user_name
        }
    }, 
        database::{
            PgDatabase, ServerBan, ServerBanShort
        }
};

static SHORT_MSG_LEN_SYMBOLS: usize = 50;

pub fn get_registration() -> CreateCommand {
    CreateCommand::new("bans")
        .description("User notes in game")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "list", "Lists all bans of this user")
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::String, "login", "In-game login")
                        .required(true),
                ),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "pardon", "Pardons specified ban ID")
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::Integer, "id", "ID of the ban from the 'list' subcommand")
                        .required(true),
                ),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "info", "Lists info about this ban")
                .add_sub_option(
                    CreateCommandOption::new(CommandOptionType::Integer, "id", "ID of the ban from the `list` subcommand")
                        .required(true)
                ),
        )
}

fn parse_list_options(opt: &ResolvedOption) -> Result<BanSubcommand, String> {
    if let ResolvedValue::SubCommand(suboptions) = &opt.value {
        if let Some(ResolvedOption {
            value: ResolvedValue::String(login), ..
        }) = suboptions.first()
        {
            return Ok(BanSubcommand::List { login: login.to_string() });
        }
    }
    Err("Invalid or missing 'login' option".to_string())
}

fn parse_pardon_options(opt: &ResolvedOption) -> Result<BanSubcommand, String> {
    if let ResolvedValue::SubCommand(suboptions) = &opt.value {
        if let Some(ResolvedOption {
            value: ResolvedValue::Integer(id), ..
        }) = suboptions.first()
        {
            return Ok(BanSubcommand::Pardon { id: *id as i32 });
        }
    }
    Err("Invalid or missing 'id' option".to_string())
}

fn parse_info_options(opt: &ResolvedOption) -> Result<BanSubcommand, String> {
    if let ResolvedValue::SubCommand(suboptions) = &opt.value {
        if let Some(ResolvedOption {
            value: ResolvedValue::Integer(id), ..
        }) = suboptions.first()
        {
            return Ok(BanSubcommand::Info { id: *id as i32 });
        }
    }
    Err("Invalid or missing 'id' option".to_string())
}

fn parse_ban_options(opt: &ResolvedOption) -> Result<BanSubcommand, String> {
    if let ResolvedValue::SubCommand(suboptions) = &opt.value {
        let mut admin_user_id = None;
        let mut player_user_id = None;
        let mut reason = None;
        let mut severity = None;
        let mut minutes = None;

        // Parse suboptions
        for suboption in suboptions {
            match (&suboption.name[..], &suboption.value) {
                ("admin_user_id", ResolvedValue::String(value)) => {
                    admin_user_id = Some(value.parse::<Uuid>().map_err(|_| "Invalid UUID for admin_user_id".to_string())?)
                }
                ("player_user_id", ResolvedValue::String(value)) => {
                    player_user_id = Some(value.parse::<Uuid>().map_err(|_| "Invalid UUID for player_user_id".to_string())?)
                }
                ("reason", ResolvedValue::String(value)) => reason = Some(value.to_string()),
                ("severity", ResolvedValue::Integer(value)) => {
                    severity = Some(*value as u8)
                }
                ("minutes", ResolvedValue::Integer(value)) => {
                    minutes = Some(*value as u64)
                }
                _ => return Err(format!("Unknown option: {}", suboption.name)),
            }
        }

        if admin_user_id.is_none()  || 
           player_user_id.is_none() ||
           severity.is_none()       ||
           minutes.is_none() {
           
           return Err("Some of the options is not supplied".to_string())
        }

        let reason = reason.unwrap_or_else(|| "No reason supplied".to_string());

        return Ok(BanSubcommand::Ban { 
            admin_user_id: admin_user_id.unwrap(),
            player_user_id: player_user_id.unwrap(),
            reason: reason.to_string(),
            severity: severity.unwrap(),
            minutes: minutes.unwrap()
        })
    }

    Err("Invalid type of the subcommand".to_string())
}

pub fn get_options(options: &[ResolvedOption]) -> Result<BanSubcommand, String> {
    if options.len() != 1 {
        return Err("Invalid options count".to_string());
    }

    let subcommand = options.first().unwrap();

    match subcommand.name {
        "list" => return parse_list_options(subcommand),
        "pardon" => return parse_pardon_options(subcommand),
        "ban" => return parse_ban_options(subcommand),
        "info" => return parse_info_options(subcommand),
        _ => return Err("invalid subcommand".to_string())
    }
}

pub async fn execute(cmd: BanSubcommand, database: &PgDatabase) -> CreateInteractionResponse {
    match cmd {
        BanSubcommand::Ban { 
            admin_user_id, 
            player_user_id, 
            reason, 
            severity, 
            minutes 
        } => execute_ban_cmd(admin_user_id, player_user_id, reason, severity, minutes).await,
        BanSubcommand::Pardon { id } => execute_pardon_cmd(id).await,
        BanSubcommand::List { login } => execute_list_cmd(&login, database).await,
        BanSubcommand::Info { id } => execute_info_cmd(id, database).await,
    }
}

async fn execute_list_cmd(login: &str, db: &PgDatabase) -> CreateInteractionResponse {
    let uuid = match get_user_id_by_login(&login, db).await {
        Some(id) => id,
        None => return create_response_with_content("No such player found."),
    };

    match db.get_bans_list(uuid).await {
        Ok(bans) => {
            let description = bans
                .iter()
                .map(|ban| format_short_ban_summary(ban))
                .collect::<Vec<String>>()
                .join("\n");

            let embed = CreateEmbed::new()
                .title(format!("Bans for `{}`", login))
                .description(description)
                .color(generate_random_colour())
                .footer(CreateEmbedFooter::new("VoidRelay By JerryImMouse"));
            
            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().add_embed(embed).ephemeral(true))
        },
        Err(e) => {
            error!("Error retrieving bans for {}. Error {}", login, e);
            create_response_with_content("Failed to retrieve bans.")
        }
    }
}

// them both are waiting for me to implement admin API
async fn execute_pardon_cmd(id: i32) -> CreateInteractionResponse {
    todo!()
}

async fn execute_ban_cmd(admin_user_id: Uuid, player_user_id: Uuid, reason: String, severity: u8, minutes: u64) -> CreateInteractionResponse {
    todo!()
}

async fn execute_info_cmd(id: i32, db: &PgDatabase) -> CreateInteractionResponse {
    match db.get_ban_by_id(id).await {
        Ok(Some(ban)) => {
            let created_by = resolve_user_name(db, &ban.banning_admin).await;
            let last_edited_by = if ban.last_edited_by_id.is_none() { None } else { Some(resolve_user_name(db, &ban.last_edited_by_id.unwrap()).await) };
            let player = resolve_user_name(db, &ban.player_user_id).await;


            let fmt = format_ban_summary(&ban, &created_by, last_edited_by);
            let embed = CreateEmbed::new()
                .title(format!("Ban of `{}`", player))
                .description(fmt)
                .colour(generate_random_colour())
                .footer(CreateEmbedFooter::new("VoidRelay By JerryImMouse"));

            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().add_embed(embed).ephemeral(true))
        },
        Ok(None) => {
            return create_response_with_content(&format!("Ban with id: {} is not found", id))
        }
        Err(e) => {
            error!("Error retrieving ban by id: {}. Error: {}", id, e);
            return create_response_with_content("Error happened retrieving ban.")
        }
    }
}

fn format_short_ban_summary(ban: &ServerBanShort) -> String {
    let short_msg = if ban.reason.chars().count() <= SHORT_MSG_LEN_SYMBOLS {
        ban.reason.clone()
    } else {
        let end_index = ban
            .reason
            .char_indices()
            .nth(SHORT_MSG_LEN_SYMBOLS)
            .map(|(idx, _)| idx)
            .unwrap_or_else(|| ban.reason.len());
        format!("{}...", &ban.reason[..end_index])
    };

    format!("**{}**. {}", ban.server_ban_id, short_msg)
}

fn format_ban_summary(ban: &ServerBan, created_by: &str, last_edited_by: Option<String>) -> String {
    let mut formatted = format!(
        r#"🔒 **Ban ID:** {}
📅 **Ban Time:** {}
📍 **Address:** {}
✍️ **Banning Admin:** {}
"#,
        ban.server_ban_id,
        ban.ban_time.to_string(),
        ban.address,
        created_by,
    );

    // Optional Fields
    if let Some(expiration_time) = ban.expiration_time {
        formatted.push_str(&format!("⏳ **Expiration Time:** {}\n", expiration_time.to_string()));
    } else {
        formatted.push_str("⏳ **Expiration Time:** Never\n");
    }

    if ban.hwid.len() != 0 {
        formatted.push_str(&format!("  **HWID:** {}\n", String::from_utf8_lossy(&ban.hwid)));
    }

    if let Some(last_edited_at) = ban.last_edited_at {
        formatted.push_str(&format!("🕒 **Last Edited At:** {}\n", last_edited_at.to_string()));
    }

    if let Some(last_edited_by) = last_edited_by {
        formatted.push_str(&format!("✍️ **Last Edited By:** {}\n", last_edited_by));
    }

    if let Some(round_id) = ban.round_id {
        formatted.push_str(&format!("✨ **Round ID:** {}\n", round_id));
    }

    formatted.push_str(&format!(
        "{}",
        if ban.auto_delete {
            "🗑️ **Auto Delete:** Yes\n"
        } else {
            "🗑️ **Auto Delete:** No\n"
        }
    ));

    formatted.push_str(&format!("\n📝 **Reason:**\n{}", ban.reason));

    formatted
}

pub enum BanSubcommand {
    List {login: String},
    Pardon {id: i32},
    Info {id: i32},
    Ban {
        admin_user_id: Uuid,
        player_user_id: Uuid,
        reason: String,
        severity: u8,
        minutes: u64,
    }
}
