#![forbid(unsafe_code)]
//! Postgres persistence adapter. Implements the application storage ports via
//! sqlx. Writes the `facts` schema only — never `app` (owned by the app plane).

use async_trait::async_trait;
use lindex_application::ports::{ReadModelStore, RepoError, ScrutinRepository, ScrutinView};
use lindex_domain::{Scrutin, ScrutinId, Sourced};
use sqlx::PgPool;

pub struct PgStore {
    pub pool: PgPool,
}

impl PgStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ScrutinRepository for PgStore {
    async fn upsert(&self, _scrutin: &Sourced<Scrutin>) -> Result<(), RepoError> {
        todo!("INSERT ... ON CONFLICT (scrutin_id) DO UPDATE into facts.scrutin")
    }

    async fn by_id(&self, _id: &ScrutinId) -> Result<Option<Scrutin>, RepoError> {
        todo!("SELECT from facts.scrutin, map row -> domain::Scrutin")
    }
}

#[async_trait]
impl ReadModelStore for PgStore {
    async fn put_scrutin_view(&self, _view: &ScrutinView) -> Result<(), RepoError> {
        todo!("UPSERT into facts.read_scrutin")
    }
}
