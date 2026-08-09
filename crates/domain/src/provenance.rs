//! The trust layer: provenance labels, never scores.

use chrono::{DateTime, Utc};

/// The five-tier provenance ladder. Labels only — this is deliberately not an
/// ordering you can average into a "trust score".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvenanceTier {
    ActeAuthentique,       // scrutins, Journal officiel
    OrganismeIndependant,  // INSEE, Cour des comptes…
    CommunicationGouv,     // ministries
    TravauxParlementaires, // comptes rendus, dossiers, amendements
    PresseSocieteCivile,   // third-party badges only
}

/// A pointer back to the exact upstream record a datum was derived from.
#[derive(Debug, Clone)]
pub struct SourceRef {
    pub tier: ProvenanceTier,
    pub label: String,
    pub url: Option<String>,
    /// Id of the immutable `facts.source_record` row this came from.
    pub record_id: String,
}

#[derive(Debug, Clone)]
pub struct Provenance {
    pub source: SourceRef,
    pub retrieved_at: DateTime<Utc>,
}

/// Wraps a value so it *cannot exist* without provenance. There is no bare `T`
/// crossing a boundary in the model — construction goes through [`Sourced::new`].
#[derive(Debug, Clone)]
pub struct Sourced<T> {
    value: T,
    provenance: Provenance,
}

impl<T> Sourced<T> {
    pub fn new(value: T, provenance: Provenance) -> Self {
        Self { value, provenance }
    }
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }
    pub fn into_parts(self) -> (T, Provenance) {
        (self.value, self.provenance)
    }
}
