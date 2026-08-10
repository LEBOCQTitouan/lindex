//! Scrutin (recorded vote) — the flagship domain object.

use crate::error::DomainError;
use chrono::NaiveDate;

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
    /// Calendar date of the sitting — the key that groups a scrutin's
    /// comparables for the day-median baseline.
    pub held_on: NaiveDate,
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
        held_on: NaiveDate,
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
            held_on,
            totals,
            breakdown,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 7, 21).expect("valid date")
    }

    fn totals() -> VoteTotals {
        VoteTotals {
            pour: 18,
            contre: 72,
            abstention: 5,
            non_votants: 0,
            members_total: 577,
        }
    }

    fn breakdown() -> Vec<GroupTally> {
        vec![
            GroupTally {
                group: GroupId("LFI".into()),
                pour: 0,
                contre: 71,
                abstention: 0,
                non_votant: 0,
            },
            GroupTally {
                group: GroupId("LIOT".into()),
                pour: 18,
                contre: 1,
                abstention: 4,
                non_votant: 0,
            },
            GroupTally {
                group: GroupId("NI".into()),
                pour: 0,
                contre: 0,
                abstention: 1,
                non_votant: 0,
            },
        ]
    }

    #[test]
    fn reconciling_breakdown_builds_and_preserves_held_on() {
        let s = Scrutin::try_new(
            ScrutinId("8433".into()),
            Chamber::AssembleeNationale,
            "Ordre public".into(),
            date(),
            totals(),
            breakdown(),
        )
        .expect("reconciles");
        assert_eq!(s.held_on, date());
        assert_eq!(s.totals.votants(), 95);
    }

    #[test]
    fn non_reconciling_breakdown_is_rejected() {
        let mut bad = breakdown();
        bad[0].contre = 70; // one vote short of the declared total
        let err = Scrutin::try_new(
            ScrutinId("8433".into()),
            Chamber::AssembleeNationale,
            "Ordre public".into(),
            date(),
            totals(),
            bad,
        )
        .expect_err("must not reconcile");
        assert!(matches!(err, DomainError::Reconciliation));
    }
}
