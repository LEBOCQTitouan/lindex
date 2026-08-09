#![forbid(unsafe_code)]
//! Assemblée nationale open-data source adapter.
//! Fetches `https://www.assemblee-nationale.fr/dyn/17/scrutins/<n>`.
//!
//! Golden fixtures for its tests already exist: the real scrutin payloads
//! (8430, 8433, …) captured during the mockup phase.

use async_trait::async_trait;
use lindex_application::ports::{ParliamentSource, RawScrutin, SourceError};
use lindex_domain::{ScrutinId, Sourced};

pub struct AnSource {
    base_url: String,
}

impl AnSource {
    pub fn new() -> Self {
        Self {
            base_url: "https://www.assemblee-nationale.fr/dyn/17".to_string(),
        }
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

impl Default for AnSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ParliamentSource for AnSource {
    async fn fetch_scrutin(&self, _id: &ScrutinId) -> Result<Sourced<RawScrutin>, SourceError> {
        // Skeleton: GET {base_url}/scrutins/{id}, then wrap the raw body in a
        // `Sourced<RawScrutin>` with an ActeAuthentique provenance pointing at
        // the stored source_record.
        let _ = &self.base_url;
        todo!("HTTP GET + wrap raw payload in Sourced with provenance")
    }
}
