use std::convert::Infallible;

use log::warn;
use rand::Rng;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serenity::all::Colour;
use uuid::Uuid;

use crate::{database::PgDatabase, error::Error};

use super::commands::ban::BanSubcommand;

#[derive(Debug, Deserialize)]
pub struct AuthServerResponse { 
    #[serde(rename = "userId")] 
    user_id: String 
}

#[derive(Debug, Deserialize)]
pub struct AuthErrorResponse { 
    #[allow(dead_code)]
    status: usize  // needed as flag, that we got into error response
}

static AUTH_SERVER: &str = "https://auth.spacestation14.com";

pub async fn lookup_user_id_by_login(login: &str) -> Result<Uuid, Error> {
    let response = reqwest::get(&format!("{AUTH_SERVER}/api/query/name?name={login}")).await?;
    let bdata = response.bytes().await?.into_iter().collect::<Vec<_>>();

    if let Ok(response) = serde_json::from_slice::<AuthServerResponse>(&bdata) {
        let uuid = Uuid::parse_str(&response.user_id)?;
        return Ok(uuid)
    }

    if serde_json::from_slice::<AuthErrorResponse>(&bdata).is_ok() {
        return Err(Error::api_err("SS14 Authorization server was unable to find such user"))
    }

    Err(Error::api_err("SS14 Authorization server was unable to find such user"))
}

// helper methods

pub async fn get_user_id_by_login(login: &str, db: &PgDatabase) -> Option<Uuid> {
    match db.get_uuid_by_login(login).await {
        Ok(uuid) => {
            uuid
        },
        Err(_) => {
            lookup_user_id_by_login(login).await.ok()
        }
    }
}

pub async fn resolve_user_name(db: &PgDatabase, user_id: &Uuid) -> String {
    match db.get_login_by_uuid(*user_id).await {
        Ok(Some(login)) => login,
        Ok(None) => user_id.to_string(),
        Err(err) => {
            warn!("Failed to fetch user name for {}: {}", user_id, err);
            user_id.to_string()
        }
    }
}

pub fn generate_random_colour() -> Colour {
    let mut rng_thread = rand::thread_rng();
    let r = rng_thread.gen::<u8>();
    let g = rng_thread.gen::<u8>();
    let b = rng_thread.gen::<u8>();

    Colour::from_rgb(r, g, b)
}