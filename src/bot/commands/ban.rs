use std::str::FromStr;

use log::{error, warn};
use serenity::all::{CommandInteraction, CommandOptionType, CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage, ResolvedOption, ResolvedValue, UserId};

use crate::{api::utilities::{BanCommand, DiscordAuthClient, PardonCommand, SS14Actor, SS14ApiClient}, bot::{create_response_with_content, utilities::{generate_random_colour, get_user_id_by_login, resolve_user_name}}, config::Config, database::{PgDatabase, ServerBan, ServerBanShort}};

static SHORT_MSG_LEN_SYMBOLS: usize = 50;

pub fn get_registration() -> CreateCommand {
    CreateCommand::new("bans")
        .description("Manages bans at SS14 server")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "list",
                "Lists all bans for specified player",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "login",
                    "In-Game login",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "info",
                "Info about specific ban by ID",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "id",
                    "Ban ID",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "pardon",
                "Pardon specific ban",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "id",
                    "Ban ID",
                )
                .required(true),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                "ban",
                "Bans specific player",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "login",
                    "In-Game Login",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "minutes",
                    "Minutes to ban player for",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "reason",
                    "Reason to ban for",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    "severity",
                    "Severity to ban for",
                )
                .required(true),
            ),
        )
}

fn parse_info_options(opt: &ResolvedOption) -> Result<BansSubcommand, String> {
    if let ResolvedValue::SubCommand(suboptions) = &opt.value {
        if let Some(ResolvedOption {
            value: ResolvedValue::Integer(id), ..
        }) = suboptions.first()
        {
            return Ok(BansSubcommand::Info(*id as i32));
        }
    }
    Err("Invalid or missing 'id' option".to_string())
}

fn parse_list_options(opt: &ResolvedOption) -> Result<BansSubcommand, String> {
    if let ResolvedValue::SubCommand(suboptions) = &opt.value {
        if let Some(ResolvedOption {
            value: ResolvedValue::String(login), ..
        }) = suboptions.first()
        {
            return Ok(BansSubcommand::List(login.to_string()));
        }
    }
    Err("Invalid or missing 'login' option".to_string())
}

fn parse_pardon_options(opt: &ResolvedOption, cmd: &CommandInteraction) -> Result<BansSubcommand, String> {
    if let ResolvedValue::SubCommand(suboptions) = &opt.value {
        let caller = cmd.user.id.to_owned();
        if let Some(ResolvedOption {
            value: ResolvedValue::Integer(id), ..
        }) = suboptions.first()
        {
            return Ok(BansSubcommand::Pardon {id: *id, caller_discord_id: caller});
        }
    }
    Err("Invalid or missing 'id' option".to_string())
}

fn parse_ban_options(opt: &ResolvedOption, cmd: &CommandInteraction) -> Result<BansSubcommand, String> {
    let caller_discord_id = cmd.user.id.clone();

    if let ResolvedValue::SubCommand(suboptions) = &opt.value {
        let mut banning_player_login: Option<String> = None;
        let mut minutes: Option<i64> = None;
        let mut reason: Option<String> = None;
        let mut severity: Option<u16> = None;
        
        for option in suboptions {
            match (option.name, &option.value) {
                ("login", ResolvedValue::String(login)) => banning_player_login = Some(login.to_string()),
                ("minutes", ResolvedValue::Integer(min)) => minutes = Some(*min),
                ("reason", ResolvedValue::String(r)) => reason = Some(r.to_string()),
                ("severity", ResolvedValue::Integer(s)) => severity = Some(*s as u16),
                _ => return Err("Invalid options passed".to_string())
            }
        }

        if banning_player_login.is_none() {
            return Err("Banning player couldn't be none".to_string())
        }

        let banning_player_login = banning_player_login.unwrap();
        let minutes = minutes.unwrap_or_else(|| 0);
        let reason = reason.unwrap_or_else(|| "No reason provided".to_string());
        let severity = severity.unwrap_or_else(|| 2);

        return Ok(BansSubcommand::Ban { 
            caller_discord_id,
            banning_player_login, 
            minutes, 
            reason, 
            severity 
        })
    }

    todo!()
}

pub fn get_options(_options: &[ResolvedOption], cmd: &CommandInteraction) -> Result<BansSubcommand, String> {
    if _options.len() != 1 {
        return Err("Invalid options length".to_string());
    }

    let subcommand = _options.first().unwrap();
    match subcommand.name {
        "list" => parse_list_options(subcommand),
        "info" => parse_info_options(subcommand),
        "pardon" => parse_pardon_options(subcommand, cmd),
        "ban" => parse_ban_options(subcommand, cmd),
        _ => return Err("Invalid subcommand".to_string())
    }
}

pub async fn execute(cmd: BansSubcommand, db: &PgDatabase, config: &Config) -> CreateInteractionResponse {
    match cmd {
        BansSubcommand::Ban { .. } => execute_ban_cmd(cmd).await,
        BansSubcommand::List(_) => execute_list_cmd(cmd, db).await,
        BansSubcommand::Pardon { .. } => execute_pardon_cmd(cmd, db, config).await,
        BansSubcommand::Info(_) => execute_info_cmd(cmd, db).await,
    }
}

async fn execute_list_cmd(cmd: BansSubcommand, db: &PgDatabase) -> CreateInteractionResponse {
    let login = match cmd {
        BansSubcommand::List(login) => login,
        _ => panic!("Invalid subcommand passed.")
    };

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

async fn execute_info_cmd(cmd: BansSubcommand, db: &PgDatabase) -> CreateInteractionResponse {
    let id = match cmd {
        BansSubcommand::Info(id) => id,
        _ => panic!("Invalid subcommand passed.")
    };

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

async fn execute_pardon_cmd(cmd: BansSubcommand, db: &PgDatabase, config: &Config) -> CreateInteractionResponse {
    let (caller, id) = match cmd {
        BansSubcommand::Pardon {caller_discord_id, id} => (caller_discord_id.to_string(), id),
        _ => panic!("Invalid subcommand passed")
    };

    let auth_client = DiscordAuthClient::new(config.auth_url(), config.auth_token());
    if let Err(e) = auth_client {
        error!("Error creating auth client: {e}");
        return create_response_with_content("Unable to setup authorization client.");
    }
    let auth_client = auth_client.unwrap();
    
    let admin_uuid = auth_client.get_uuid_by_discord_id(&caller).await;
    let admin_uuid = match admin_uuid {
        Ok(Some(uuid)) => uuid.uuid,
        Ok(None) => return create_response_with_content("You're probably unauthorized to perform this action."),
        Err(e) => {
            error!("Error occured during fetching uuid by discord_id: {}", e);
            return create_response_with_content("Error happened in auth client.")
        }
    };

    let admin_name = match db.get_login_by_uuid(uuid::Uuid::from_str(&admin_uuid).unwrap()).await {
        Ok(Some(login)) => login,
        Ok(None) => return create_response_with_content("Unable to fetch admin name."),
        Err(err) => {
            warn!("Failed to fetch user name for {}: {}", admin_uuid, err);
            return create_response_with_content("Unable to fetch admin name.")
        }
    };

    let ss14_client = SS14ApiClient::new(config.api_url(), config.server_token());
    if let Err(e) = ss14_client {
        error!("Error creating ss14 client: {e}");
        return create_response_with_content("Unable to setup ss14 client.");
    }
    let ss14_client = ss14_client.unwrap();

    let actor = SS14Actor::new(uuid::Uuid::from_str(&admin_uuid).unwrap(), &admin_name);
    let ban_cmd = PardonCommand::new(id);

    let result = ss14_client.post_pardon(ban_cmd, actor).await;

    match result {
        Ok(_) => create_response_with_content(&format!("Successfully pardoned ban with id: {}", id)),
        Err(e) => {
            error!("Unable to pardon ban. Id: {id}. Error: {e}");
            create_response_with_content("Error occured during ban pardon.")
        }
    }
}

async fn execute_ban_cmd(cmd: BansSubcommand) -> CreateInteractionResponse {
    if !matches!(cmd, BansSubcommand::Ban {..}) {
        panic!("Invalid bans subcommand passed!")
    }

    todo!()
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

pub enum BansSubcommand {
    List(String),
    Info(i32),
    Pardon {
        caller_discord_id: UserId,
        id: i64
    },
    Ban {
        caller_discord_id: UserId,
        banning_player_login: String,
        minutes: i64,
        reason: String,
        severity: u16,
    }
}