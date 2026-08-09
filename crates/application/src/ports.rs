//! Driven ports. Implemented by adapters; nothing here knows about Postgres,
//! HTTP, or any concrete technology.

use async_trait::async_trait;
use lindex_domain::{Scrutin, ScrutinId, Sourced};

/// Raw payload exactly as fetched, before mapping to the domain model.
/// Stored immutably so re-ingestion is deterministic (US-8.4).
#[derive(Debug, Clone)]
pub struct RawScrutin {
    pub payload: String,
}

/// The denormalized read-model row — the *contract* with the app plane.
/// One row = everything the `/scrutin/:id` page needs.
#[derive(Debug, Clone)]
pub struct ScrutinView {
    pub scrutin_id: String,
    pub chamber: String,
    pub title: String,
    pub outcome: String,
    pub totals_json: String,
    pub breakdown_json: String,
    pub baselines_json: String,
    pub provenance_json: String,
}

/// An upstream parliamentary data source (AN, Sénat, Légifrance).
#[async_trait]
pub trait ParliamentSource: Send + Sync {
    async fn fetch_scrutin(&self, id: &ScrutinId) -> Result<Sourced<RawScrutin>, SourceError>;
}

/// Persistence of the unified model. `upsert` must be idempotent.
#[async_trait]
pub trait ScrutinRepository: Send + Sync {
    async fn upsert(&self, scrutin: &Sourced<Scrutin>) -> Result<(), RepoError>;
    async fn by_id(&self, id: &ScrutinId) -> Result<Option<Scrutin>, RepoError>;
}

/// Projection of read-models consumed by the app plane.
#[async_trait]
pub trait ReadModelStore: Send + Sync {
    async fn put_scrutin_view(&self, view: &ScrutinView) -> Result<(), RepoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("source unavailable: {0}")]
    Unavailable(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error("repository backend error: {0}")]
    Backend(String),
}
