use chrono::{DateTime, Local};
use sqlx::{Executor, PgPool};
use uuid::Uuid;
use sqlx::Row;

use crate::error::Error;

pub mod utilities;

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
        if let None = row {
            return Ok(None)
        }
        let row = row.unwrap();

        let uuid: Uuid = row.try_get("user_id")?;

        Ok(Some(uuid))
    }

    pub async fn whitelistadd(&self, user_id: Uuid) -> Result<u64, Error> {
        let query = sqlx::query("INSERT INTO whitelist (user_id) VALUES ($1)");
        let query = query.bind(user_id);
        
        let affected_rows = self.inner_pool.execute(query).await?.rows_affected();
        Ok(affected_rows)
    }

    pub async fn whitelistrm(&self, user_id: Uuid) -> Result<u64, Error> {
        let query = sqlx::query("DELETE FROM whitelist WHERE user_id = $1");
        let query = query.bind(user_id);

        let affected_rows = self.inner_pool.execute(query).await?.rows_affected();
        Ok(affected_rows)
    }

    pub async fn get_login_by_uuid(&self, uuid: Uuid) -> Result<Option<String>, Error> {
        let query = sqlx::query("SELECT last_seen_user_name FROM player WHERE user_id = $1");
        let query = query.bind(uuid);

        let row = self.inner_pool.fetch_optional(query).await?;
        if let None = row {
            return Ok(None);
        }

        let row = row.unwrap();
        let login: String = row.try_get("last_seen_user_name")?;

        Ok(Some(login))
    }

    pub async fn get_notes(&self, uuid: Uuid) -> Result<Vec<AdminNote>, Error> {
        let notes = sqlx::query_as::<_, AdminNote>("SELECT * FROM admin_notes WHERE player_user_id = $1")
            .bind(uuid)
            .fetch_all(&self.inner_pool).await?;
        
        Ok(notes)
    }

    pub async fn close(&self) {
        self.inner_pool.close().await;
    }
}

// data structs

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminNote {
    pub round_id: i32,
    pub player_user_id: Uuid,
    pub message: String,
    pub created_by_id: Uuid,
    pub created_at: DateTime<Local>,
    pub last_edited_by_id: Uuid,
    pub last_edited_at: DateTime<Local>,
    pub deleted: bool,
    pub deleted_by_id: Option<Uuid>,
    pub deleted_at: Option<DateTime<Local>>,
    pub expiration_time: Option<DateTime<Local>>,
    pub severity: i32,
}