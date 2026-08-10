#![forbid(unsafe_code)]
//! Assemblée nationale open-data source adapter.
//! Fetches `https://www.assemblee-nationale.fr/dyn/17/scrutins/<n>`, parses the
//! analysis page, and emits the source-agnostic [`RawScrutinData`] (ADR-0001).
//!
//! Golden fixtures for the parser live in `tests/fixtures/` — the real scrutin
//! pages (8430, 8433) captured during the mockup phase.

mod an_parse;

use async_trait::async_trait;
use chrono::Utc;
use lindex_application::ports::{ParliamentSource, RawScrutin, RawScrutinData, SourceError};
use lindex_domain::{Provenance, ProvenanceTier, ScrutinId, SourceRef, Sourced};

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

    fn scrutin_url(&self, id: &ScrutinId) -> String {
        format!("{}/scrutins/{}", self.base_url, id.0)
    }
}

impl Default for AnSource {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ParliamentSource for AnSource {
    async fn fetch_scrutin(&self, id: &ScrutinId) -> Result<Sourced<RawScrutin>, SourceError> {
        let url = self.scrutin_url(id);
        let response = reqwest::get(&url)
            .await
            .map_err(|e| SourceError::Unavailable(format!("GET {url}: {e}")))?
            .error_for_status()
            .map_err(|e| SourceError::Unavailable(format!("GET {url}: {e}")))?;
        let html = response
            .text()
            .await
            .map_err(|e| SourceError::Unavailable(format!("body {url}: {e}")))?;

        let data: RawScrutinData = an_parse::parse_scrutin_html(&html, id)
            .map_err(|e| SourceError::Parse(format!("{url}: {e}")))?;
        let payload = serde_json::to_string(&data)
            .map_err(|e| SourceError::Parse(format!("serialize {url}: {e}")))?;

        let provenance = Provenance {
            source: SourceRef {
                tier: ProvenanceTier::ActeAuthentique,
                label: format!("Scrutin n° {} — AN", id.0),
                url: Some(url),
                record_id: format!("an-scrutin-{}", id.0),
            },
            retrieved_at: Utc::now(),
        };
        Ok(Sourced::new(RawScrutin { payload }, provenance))
    }
}
