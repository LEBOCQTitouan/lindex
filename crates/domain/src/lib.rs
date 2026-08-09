#![forbid(unsafe_code)]
//! L'Index domain — the pure model. No I/O, depends on nothing outward.
//!
//! The platform's non-negotiable rules are encoded as *types*, so violating
//! them is a compile error rather than a review comment:
//!   * every datum carries provenance ....... [`Sourced<T>`]
//!   * every displayed figure carries a baseline [`Indicator`]
//!   * no composite score / ranking of people exists in the model at all.

mod error;
mod indicator;
mod provenance;
mod scrutin;

pub use error::DomainError;
pub use indicator::{Baseline, Count, Indicator, MethodRef};
pub use provenance::{Provenance, ProvenanceTier, SourceRef, Sourced};
pub use scrutin::{Chamber, GroupId, GroupTally, Scrutin, ScrutinId, VoteTotals};
