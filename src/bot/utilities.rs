use serde::Deserialize;
use uuid::Uuid;

use crate::{database::PgDatabase, error::Error};

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
        return Err(Error::AuthNotFound)
    }

    Err(Error::AuthNotFound)
}

// helper method to avoid error checks bloating
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