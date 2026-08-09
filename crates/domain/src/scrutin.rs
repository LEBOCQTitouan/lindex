//! Scrutin (recorded vote) — the flagship domain object.

use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chamber {
    AssembleeNationale,
    Senat,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScrutinId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupId(pub String);

/// One political group's tally. Groups are stored in hémicycle (left→right)
/// order by the position of the `GroupTally` in [`Scrutin::breakdown`].
#[derive(Debug, Clone)]
pub struct GroupTally {
    pub group: GroupId,
    pub pour: u32,
    pub contre: u32,
    pub abstention: u32,
    pub non_votant: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct VoteTotals {
    pub pour: u32,
    pub contre: u32,
    pub abstention: u32,
    pub non_votants: u32,
    pub members_total: u32,
}

impl VoteTotals {
    /// Seats that took part (pour + contre + abstention).
    pub fn votants(&self) -> u32 {
        self.pour + self.contre + self.abstention
    }

    /// Decisive votes (pour + contre) — excludes abstentions.
    pub fn exprimes(&self) -> u32 {
        self.pour + self.contre
    }

    /// Seats that neither voted nor were recorded present: the participation gap.
    pub fn absents(&self) -> u32 {
        self.members_total
            .saturating_sub(self.votants() + self.non_votants)
    }
}

#[derive(Debug, Clone)]
pub struct Scrutin {
    pub id: ScrutinId,
    pub chamber: Chamber,
    pub title: String,
    pub totals: VoteTotals,
    /// Left→right hémicycle order.
    pub breakdown: Vec<GroupTally>,
}

impl Scrutin {
    /// Construct a scrutin, checking that the per-group breakdown reconciles
    /// with the declared totals. Mapping/validation lives here, not in adapters.
    pub fn try_new(
        id: ScrutinId,
        chamber: Chamber,
        title: String,
        totals: VoteTotals,
        breakdown: Vec<GroupTally>,
    ) -> Result<Self, DomainError> {
        let sum = |f: fn(&GroupTally) -> u32| breakdown.iter().map(f).sum::<u32>();
        let reconciles = sum(|g| g.pour) == totals.pour
            && sum(|g| g.contre) == totals.contre
            && sum(|g| g.abstention) == totals.abstention;
        if !reconciles {
            return Err(DomainError::Reconciliation);
        }
        Ok(Self {
            id,
            chamber,
            title,
            totals,
            breakdown,
        })
    }
}
