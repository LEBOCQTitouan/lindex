//! Use cases. Orchestrate ports; hold no I/O of their own.

use crate::ports::{ParliamentSource, ReadModelStore, RepoError, ScrutinRepository, SourceError};
use lindex_domain::ScrutinId;

/// Ingest one scrutin: fetch raw → map to domain → persist → project read-model.
/// Idempotent: re-running the same id reproduces the same state (US-8.4).
pub struct IngestScrutin<'a> {
    pub source: &'a dyn ParliamentSource,
    pub repo: &'a dyn ScrutinRepository,
    pub read: &'a dyn ReadModelStore,
}

impl IngestScrutin<'_> {
    pub async fn run(&self, _id: &ScrutinId) -> Result<(), IngestError> {
        // Skeleton: wiring only. Filling this in is the first real slice —
        //   1. self.source.fetch_scrutin(id)  -> Sourced<RawScrutin>
        //   2. map raw payload -> Sourced<Scrutin> (validated in the domain)
        //   3. self.repo.upsert(&scrutin)
        //   4. build ScrutinView (with baselines) and self.read.put_scrutin_view(...)
        todo!("first real slice: map RawScrutin -> Sourced<Scrutin>, persist, project")
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IngestError {
    #[error(transparent)]
    Source(#[from] SourceError),
    #[error(transparent)]
    Repo(#[from] RepoError),
}
