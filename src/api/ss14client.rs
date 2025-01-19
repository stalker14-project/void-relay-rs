use std::time::Duration;

use reqwest::{self, Method};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Error;

static TIMEOUT: Duration = Duration::from_secs(5);
static USER_AGENT: &str = "VoidRelay Discord Bot";

pub struct SS14ApiClient {
    inner: reqwest::Client,

    api_url: String,
    api_key: String,
}

// assume this tuple is (api_url, api_key)
impl TryFrom<(&str, &str)> for SS14ApiClient {
    type Error = Error;
    
    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        Self::new(value.0, value.1)
    }
}

impl SS14ApiClient {
    pub fn new(api_url: &str, api_key: &str) -> Result<Self, Error> {
        let inner = reqwest::Client::builder()
            .connect_timeout(TIMEOUT)
            .user_agent(USER_AGENT)
            .build()?;

        Ok(Self {
            inner,
            api_key: api_key.to_owned(),
            api_url: api_url.to_owned()
        })
    }

    pub async fn pardon(&self, req: PardonRequest) -> Result<(), Error> {
        let actor = serde_json::to_string(&req.actor)?;
        let body = serde_json::to_string(&req)?;

        let request = self.inner.request(Method::POST, format!("{}/admin/actions/pardon", self.api_url))
            .header("Authorization", format!("SS14Token {}", self.api_key))
            .header("Actor", actor)
            .body(body)
            .build()?;

        let response = self.inner.execute(request).await?;
        let status = response.status().to_owned();

        if status == 200 {
            return Ok(())
        }

        if let Ok(bytes) = response.bytes().await {
            let body = bytes.into_iter().collect::<Vec<_>>();
            let err_response = serde_json::from_slice::<ErrorResponse>(&body);
            match err_response {
                Err(_) => Err(Error::ss14_api(&format!("Invalid error: {}", String::from_utf8_lossy(&body)))),
                Ok(val) => Err(Error::from(val))
            }
        } else {
            Err(Error::ss14_api(&format!("Unknown error occured at SS14 api. Status: {}", status)))
        }
    }

    pub async fn ban(&self, req: BanRequest) -> Result<(), Error> {
        let actor = serde_json::to_string(&req.actor)?;
        let body = serde_json::to_string(&req)?;

        let request = self.inner.request(Method::POST, format!("{}/admin/actions/ban", self.api_url))
            .header("Authorization", format!("SS14Token {}", self.api_key))
            .header("Actor", actor)
            .body(body)
            .build()?;

        let response = self.inner.execute(request).await?;
        let status = response.status().to_owned();

        if status == 200 {
            return Ok(())
        }

        if let Ok(bytes) = response.bytes().await {
            let body = bytes.into_iter().collect::<Vec<_>>();
            let err_response = serde_json::from_slice::<ErrorResponse>(&body);
            match err_response {
                Err(_) => Err(Error::ss14_api(&format!("Invalid error: {}", String::from_utf8_lossy(&body)))),
                Ok(val) => Err(Error::from(val))
            }
        } else {
            Err(Error::ss14_api(&format!("Unknown error occured at SS14 api. Status: {}", status)))
        }
    }
}

// responses

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    #[serde(rename = "Message")]
    message: String,
    #[serde(rename = "ErrorCode")]
    error_code: i32,
    #[serde(rename = "Exception")]
    exception: Option<SS14ExceptionData>
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message: {}. ErrorCode: {}. Exception Message: {}", 
            self.message, 
            self.error_code,
            if let Some(ex) = &self.exception { &ex.message } else { "none" }
        )
    }
}

// requests

#[derive(Serialize, Debug)]
pub struct PardonRequest {
    #[serde(rename = "BanId")]
    ban_id: i32,

    #[serde(skip)]
    actor: SS14ApiActor
}

impl PardonRequest {
    pub fn new(ban_id: i32, actor: SS14ApiActor) -> Self {
        Self {
            ban_id,
            actor
        }
    }
}

#[derive(Serialize, Debug)]
pub struct BanRequest {
    #[serde(rename = "Username")]
    username: String,
    #[serde(rename = "PlayerGuid")]
    player_guid: String,
    #[serde(rename = "Reason")]
    reason: String,
    #[serde(rename = "Minutes")]
    minutes: i32,
    #[serde(rename = "Severity")]
    severity: u16,

    #[serde(skip)]
    actor: SS14ApiActor
}

impl BanRequest {
    pub fn new(
        username: String,
        player_guid: Uuid,
        reason: String,
        minutes: i32,
        severity: u16,

        actor: SS14ApiActor
    ) -> Self {
        Self {
            username,
            player_guid: player_guid.to_string(),
            reason,
            minutes,
            severity,
            actor
        }
    }
}

// helper structs

#[derive(Serialize, Debug)]
pub struct SS14ApiActor {
    #[serde(rename = "Guid")]
    pub guid: String,
    #[serde(rename = "Name")]
    pub name: String,
}

impl SS14ApiActor {
    pub fn new(guid: uuid::Uuid, name: &str) -> Self {
        Self {
            guid: guid.to_string(),
            name: name.to_string()
        }
    }
}

impl From<(&str, &str)> for SS14ApiActor {
    fn from(value: (&str, &str)) -> Self {
        Self {
            guid: value.0.to_string(),
            name: value.1.to_string()
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct SS14ExceptionData {
    #[serde(rename = "Message")]
    message: String,
}