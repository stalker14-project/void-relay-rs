use serde::Deserialize;
use uuid::Uuid;

use crate::{database::PgDatabase, error::Error};

#[derive(Debug, Deserialize)]
pub struct AuthServerResponse { userId: String }

#[derive(Debug, Deserialize)]
pub struct AuthErrorResponse { status: usize }

static AUTH_SERVER: &str = "https://auth.spacestation14.com";

pub async fn lookup_user_id_by_login(login: &str) -> Result<Uuid, Error> {
    let response = reqwest::get(&format!("{AUTH_SERVER}/api/query/name?name={login}")).await?;
    let bdata = response.bytes().await?.into_iter().collect::<Vec<_>>();

    if let Ok(response) = serde_json::from_slice::<AuthServerResponse>(&bdata) {
        let uuid = Uuid::parse_str(&response.userId)?;
        return Ok(uuid)
    }

    if let Ok(_) = serde_json::from_slice::<AuthErrorResponse>(&bdata) {
        return Err(Error::AuthNotFound)
    }

    return Err(Error::AuthNotFound)
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