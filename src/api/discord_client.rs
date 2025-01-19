use std::{str::FromStr, time::Duration};

use log::error;
use reqwest::Method;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::Error;

static TIMEOUT: Duration = Duration::from_secs(5);
static USER_AGENT: &str = "VoidRelay Discord Bot";

pub struct DiscordApiClient {
    inner: reqwest::Client,

    api_url: String,
    api_key: String,
}

// assume this tuple is (api_url, api_key)
impl TryFrom<(&str, &str)> for DiscordApiClient {
    type Error = Error;
    
    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        Self::new(value.0, value.1)
    }
}

impl DiscordApiClient {
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

    pub async fn uuid(&self, discord_id: &str) -> Option<Uuid> {
        let request = self.inner
            .request(Method::GET, format!("{}/api/uuid?method=discord&id={}", self.api_url, discord_id))
            .bearer_auth(&self.api_key)
            .build();

        if let Err(e) = request {
            error!("Error occurred constructing auth request: {}", e);
            return None;
        }

        let request = request.unwrap();

        let response = self.inner.execute(request).await;
        if let Err(e) = &response {
            if e.is_connect() {
                error!("Connecting error in auth client. Check if auth service is up.");
                return None;
            }
        }
        let response = response.unwrap();
        let status = response.status();

        if status.is_success() {
            if let Ok(bytes) = response.bytes().await {
                let body = bytes.into_iter().collect::<Vec<_>>();
                let uuid = serde_json::from_slice::<DiscordAuthUuidResponse>(&body);
                if let Err(e) = uuid {
                    error!("Error deserializing UUID response from auth client. Error: {}", e);
                    return None;
                } else {
                    let uuid = uuid.unwrap().uuid;
                    let uuid = Uuid::from_str(&uuid);
                    match uuid {
                        Ok(uuid) => return Some(uuid),
                        Err(e) => {
                            error!("Error occured during parsing auth response UUID to UUID struct. Error: {}", e);
                            return None;
                        }
                    }
                }
            } else {
                error!("Request has been successfully proceed, but response is empty.");
                return None;
            }
        }

        if let Ok(bytes) = response.bytes().await {
            let body = bytes.into_iter().collect::<Vec<_>>();
            let err_response = serde_json::from_slice::<ErrorResponse>(&body);
            if let Err(e) = err_response {
                error!("Error deserializing error response from auth client. Error: {}", e);
            } else {
                let err = err_response.unwrap();
                error!("Error occured during fetching UUID from auth client. Error: {}", err.error);
            }
        }

        None
    }
}

// responses

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    error: String,
}

#[derive(Debug, Deserialize)]
pub struct DiscordAuthUuidResponse {
    pub uuid: String,
}