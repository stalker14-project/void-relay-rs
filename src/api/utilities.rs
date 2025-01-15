use std::time::Duration;

use log::error;
use reqwest::Method;
use serde::{Deserialize, Serialize};

use crate::{bot::commands::ban::BanSubcommand, error::Error};

static TIMEOUT: Duration = Duration::from_secs(1);
static USER_AGENT: &str = "VoidRelay Discord Bot";

pub struct DiscordAuthClient {
    inner: reqwest::Client,
    auth_url: String,
    auth_token: String,
}

impl DiscordAuthClient {
    pub fn new(auth_url: &str, auth_token: &str) -> Result<Self, Error> {
        let client = reqwest::Client::builder()
            .connect_timeout(TIMEOUT)
            .http1_only()
            .user_agent(USER_AGENT)
            .build()?;

        Ok(Self {
            inner: client,
            auth_url: auth_url.to_string(),
            auth_token: auth_token.to_string()
        })
    }

    pub async fn get_uuid_by_discord_id(&self, discord_uid: &str) -> Result<Option<DiscordAuthUuidResponse>, Error> {
        let request = self.inner
            .get(&format!("{}/api/uuid?method=discord&id={}", self.auth_url, discord_uid))
            .bearer_auth(&self.auth_token)
            .build()?;

        let response = self.inner.execute(request).await?;
        if response.status() != 200 {
            if response.content_length().is_some() {
                let error = response.bytes().await?.into_iter().collect::<Vec<_>>();
                error!("Error trying to get uuid by discord_id: {}", String::from_utf8_lossy(&error));
                return Err(Error::api_err("Error happened on typeauthd side, check logs for more info."))
            } else {
                error!("Unknown error happened at typeauthd trying to get uuid by discord_uid.");
                return Err(Error::api_err("Unknown error happened on typeauthd side."))
            }
        }

        let response_body = response.bytes().await?.into_iter().collect::<Vec<_>>();
        let uuid_response: DiscordAuthUuidResponse = serde_json::from_slice(&response_body)?;

        Ok(Some(uuid_response))
    }
}

// data structs

#[derive(Debug, Deserialize)]
pub struct DiscordAuthUuidResponse {
    pub uuid: String,
}


// I had to create this, so because I dont want to fuck around with SS14 database, I just don't want to fuck up db, so let me just talk to server :)
#[derive(Debug)]
pub struct SS14ApiClient {
    inner: reqwest::Client,
    api_url: String,
    auth_token: String,
}

// TODO: Implement following routes on server
impl SS14ApiClient {
    pub fn new(api_url: &str, auth_token: &str) -> Result<Self, Error> {
        let inner = reqwest::ClientBuilder::new()
            .connect_timeout(TIMEOUT)
            .http1_only()
            .user_agent(USER_AGENT)
            .build()?;

        Ok(Self {
            inner,
            api_url: api_url.to_string(),
            auth_token: auth_token.to_string()
        })
    }

    pub async fn post_ban(&self, cmd: BanSubcommand) -> Result<(), Error> {
        let request = self.inner.request(Method::POST, format!("{}/actions/ban", self.api_url))
            .bearer_auth(self.auth_token.to_owned())
            .body(serde_json::to_string(&BanCommand::from(cmd)).unwrap())
            .build()?;

        let response = self.inner.execute(request).await?;
        
        if response.status() != 200 {
            Err(Error::api_err("Error happened while posting ban command."))
        } else {
            Ok(())
        }
    }

    pub async fn post_pardon(&self, cmd: BanSubcommand) -> Result<(), Error> {
        let request = self.inner.request(Method::POST, format!("{}/actions/pardon", self.api_url))
            .bearer_auth(self.auth_token.to_owned())
            .body(serde_json::to_string(&PardonCommand::from(cmd)).unwrap())
            .build()?;

        let response = self.inner.execute(request).await?;
        
        if response.status() != 200 {
            Err(Error::api_err("Error happened while posting pardon command."))
        } else {
            Ok(())
        }
    }
}

// data structs

#[derive(Debug, Serialize)]
pub struct BanCommand {
    #[serde(rename = "adminGuid")]
    admin_guid: String,
    #[serde(rename = "playerGuid")]
    player_guid: String,
    reason: String,
    severity: u8,
    minutes: u64,
}

impl From<BanSubcommand> for BanCommand {
    fn from(value: BanSubcommand) -> Self {
        if let BanSubcommand::Ban { 
            admin_user_id, 
            player_user_id, 
            reason, 
            severity, 
            minutes 
        } = value {
            BanCommand {
                admin_guid: admin_user_id.to_string(),
                player_guid: player_user_id.to_string(),
                reason,
                severity,
                minutes
            }
        } else {
            panic!("Not a ban command passed.")
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PardonCommand {
    #[serde(rename = "banId")]
    ban_id: i32,
}

impl From<BanSubcommand> for PardonCommand {
    fn from(value: BanSubcommand) -> Self {
        if let BanSubcommand::Pardon { id } = value {
            PardonCommand { ban_id: id}
        } else {
            panic!("Invalid ban subcommand passed.")
        }
    }
}