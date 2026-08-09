//! Domain errors — `thiserror` at the boundary (per project defaults).

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    /// Per-group tallies do not reconcile with the declared totals.
    #[error("vote totals do not reconcile with the per-group breakdown")]
    Reconciliation,
    #[error("invalid domain value: {0}")]
    Invalid(String),
}
