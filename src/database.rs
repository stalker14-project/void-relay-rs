use chrono::{DateTime, Utc};
use sqlx::{Executor, PgPool};
use uuid::Uuid;
use sqlx::Row;

use crate::error::Error;

pub struct PgDatabase {
    inner_pool: PgPool
}

impl PgDatabase {
    pub fn new(cstr: &str) -> Result<Self, Error> {
        let pg_pool = PgPool::connect_lazy(cstr)?;

        Ok(Self { inner_pool: pg_pool })
    }

    pub async fn get_uuid_by_login(&self, login: &str) -> Result<Option<Uuid>, Error> {
        let query = sqlx::query("SELECT user_id FROM player WHERE last_seen_user_name = $1");
        let query = query.bind(login);

        let row = self.inner_pool.fetch_optional(query).await?;
        if row.is_none() {
            return Ok(None)
        }
        let row = row.unwrap();

        let uuid: Uuid = row.try_get("user_id")?;

        Ok(Some(uuid))
    }

    pub async fn whitelistadd(&self, user_id: &Uuid) -> Result<u64, Error> {
        let query = sqlx::query("INSERT INTO whitelist (user_id) VALUES ($1)");
        let query = query.bind(user_id);
        
        let affected_rows = self.inner_pool.execute(query).await?.rows_affected();
        Ok(affected_rows)
    }

    pub async fn whitelistrm(&self, user_id: &Uuid) -> Result<u64, Error> {
        let query = sqlx::query("DELETE FROM whitelist WHERE user_id = $1");
        let query = query.bind(user_id);

        let affected_rows = self.inner_pool.execute(query).await?.rows_affected();
        Ok(affected_rows)
    }

    pub async fn get_login_by_uuid(&self, uuid: &Uuid) -> Result<Option<String>, Error> {
        let query = sqlx::query("SELECT last_seen_user_name FROM player WHERE user_id = $1");
        let query = query.bind(uuid);

        let row = self.inner_pool.fetch_optional(query).await?;
        if row.is_none() {
            return Ok(None);
        }

        let row = row.unwrap();
        let login: String = row.try_get("last_seen_user_name")?;

        Ok(Some(login))
    }

    pub async fn get_notes_list(&self, uuid: &Uuid) -> Result<Vec<AdminNoteShort>, Error> {
        let notes = sqlx::query_as::<_, AdminNoteShort>("SELECT admin_notes_id, message FROM admin_notes WHERE player_user_id = $1")
            .bind(uuid)
            .fetch_all(&self.inner_pool).await?;
        
        Ok(notes)
    }

    pub async fn get_bans_list(&self, uuid: &Uuid) -> Result<Vec<ServerBanShort>, Error> {
        let bans = sqlx::query_as::<_, ServerBanShort>(
            "SELECT server_ban_id, reason FROM server_ban WHERE player_user_id = $1"
        ).bind(uuid)
        .fetch_all(&self.inner_pool).await?;


        Ok(bans)
    }

    pub async fn get_ban_by_id(&self, ban_id: i32) -> Result<Option<ServerBan>, Error> {
        let ban = sqlx::query_as::<_, ServerBan>(
            "SELECT * FROM server_ban WHERE server_ban_id = $1"
        ).bind(ban_id)
        .fetch_optional(&self.inner_pool).await?;

        Ok(ban)
    }

    pub async fn get_note_by_id(&self, note_id: i32) -> Result<Option<AdminNote>, Error> {
        let note = sqlx::query_as::<_, AdminNote>(
            "SELECT * FROM admin_notes WHERE admin_notes_id = $1"
        ).bind(note_id)
        .fetch_optional(&self.inner_pool).await?;

        Ok(note)
    }

    pub async fn close(&self) {
        self.inner_pool.close().await;
    }
}

// data structs

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminNoteShort {
    pub admin_notes_id: i32,
    pub message: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminNote {
    pub round_id: i32,
    pub player_user_id: Uuid,
    pub message: String,
    pub created_by_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub last_edited_by_id: Uuid,
    pub last_edited_at: DateTime<Utc>,
    pub deleted: bool,
    pub deleted_by_id: Option<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub secret: bool,
    pub expiration_time: Option<DateTime<Utc>>,
    pub severity: i32
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ServerBanShort {
    pub server_ban_id: i32,
    pub reason: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ServerBan {
    pub server_ban_id: i32,
    pub player_user_id: Uuid,
    pub address: std::net::IpAddr,
    pub ban_time: DateTime<Utc>,
    pub expiration_time: Option<DateTime<Utc>>,
    pub reason: String,
    pub banning_admin: Uuid,
    pub hwid: Vec<u8>,
    pub auto_delete: bool,
    pub last_edited_at: Option<DateTime<Utc>>,
    pub last_edited_by_id: Option<Uuid>,
    pub round_id: Option<i32>,
}