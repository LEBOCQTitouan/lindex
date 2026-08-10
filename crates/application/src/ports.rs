//! Driven ports. Implemented by adapters; nothing here knows about Postgres,
//! HTTP, or any concrete technology.

use async_trait::async_trait;
use chrono::NaiveDate;
use lindex_domain::{Chamber, Scrutin, ScrutinId, Sourced, VoteTotals};
use serde::{Deserialize, Serialize};

/// Raw payload exactly as fetched, before mapping to the domain model.
/// Stored immutably so re-ingestion is deterministic (US-8.4). The payload is a
/// serialized [`RawScrutinData`] — the source-agnostic normalized record.
#[derive(Debug, Clone)]
pub struct RawScrutin {
    pub payload: String,
}

/// The normalized, source-agnostic extraction a `ParliamentSource` produces from
/// its upstream format. This — not the source's HTML/XML — is what
/// `facts.source_record` stores, and what the ingest use case maps to the domain.
/// AN-specific parsing stays inside the source adapter (hexagon boundary).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawScrutinData {
    pub scrutin_id: String,
    /// `"AN"` | `"SENAT"`.
    pub chamber: String,
    pub title: String,
    /// ISO calendar date of the sitting, `YYYY-MM-DD`.
    pub held_on: String,
    /// Source-stated result, verbatim: `"adopte"` | `"rejete"`. Never computed.
    pub outcome: String,
    pub totals: RawTotals,
    /// Per-group tallies in hémicycle (left→right) order.
    pub groups: Vec<RawGroup>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawTotals {
    pub pour: u32,
    pub contre: u32,
    pub abstention: u32,
    #[serde(rename = "nonVotants")]
    pub non_votants: u32,
    #[serde(rename = "membersTotal")]
    pub members_total: u32,
    pub votants: u32,
    pub exprimes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawGroup {
    pub group: String,
    pub pour: u32,
    pub contre: u32,
    pub abstention: u32,
    #[serde(rename = "nonVotant")]
    pub non_votant: u32,
}

/// The denormalized read-model row — the *contract* with the app plane.
/// One row = everything the `/scrutin/:id` page needs. JSONB shapes are pinned
/// in ADR-0001.
#[derive(Debug, Clone)]
pub struct ScrutinView {
    pub scrutin_id: String,
    pub chamber: String,
    pub title: String,
    pub held_on: String,
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

/// Persistence of the unified model. Every write is idempotent.
#[async_trait]
pub trait ScrutinRepository: Send + Sync {
    /// Persist the immutable raw source record (the provenance backbone).
    /// Must run before [`ScrutinRepository::upsert`] so the scrutin's foreign
    /// key to its source record resolves.
    async fn put_source_record(&self, raw: &Sourced<RawScrutin>) -> Result<(), RepoError>;
    async fn upsert(&self, scrutin: &Sourced<Scrutin>) -> Result<(), RepoError>;
    async fn by_id(&self, id: &ScrutinId) -> Result<Option<Scrutin>, RepoError>;
    /// Totals of every scrutin of the same chamber held on the same day —
    /// the comparables for the day-median baseline (includes the just-upserted
    /// row). Order is unspecified; the caller takes a median.
    async fn cohort_totals(
        &self,
        chamber: &Chamber,
        held_on: NaiveDate,
    ) -> Result<Vec<VoteTotals>, RepoError>;
}

/// Projection of read-models consumed by the app plane.
#[async_trait]
pub trait ReadModelStore: Send + Sync {
    async fn put_scrutin_view(&self, view: &ScrutinView) -> Result<(), RepoError>;
    /// Overwrite the baseline of every already-projected scrutin of the same
    /// chamber and day. The day-median is a day-level figure shared by the whole
    /// cohort, so a newly ingested same-day scrutin must refresh its peers'
    /// baselines — otherwise earlier rows keep a stale, smaller-sample median
    /// (ADR-0001 §3; display rule #1).
    async fn refresh_day_baselines(
        &self,
        chamber: &str,
        held_on: NaiveDate,
        baselines_json: &str,
    ) -> Result<(), RepoError>;
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SourceError {
    #[error("source unavailable: {0}")]
    Unavailable(String),
    #[error("source payload could not be parsed: {0}")]
    Parse(String),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RepoError {
    #[error("repository backend error: {0}")]
    Backend(String),
}
