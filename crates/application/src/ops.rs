//! Unattended-operations use cases: date-range re-ingestion (US-8.4) and failure
//! alerting (US-8.1). Built on the [`IngestScrutin`](crate::usecase::IngestScrutin)
//! spine, so a re-ingest is the single-scrutin ingest replayed over a work set.
//!
//! **Work-set discovery is from the inside.** A date-range re-ingest asks the
//! [`ScrutinCalendar`] which scrutins are already on record for the chamber over
//! the day-range, then replays the idempotent single-scrutin ingest for each. It
//! does *not* discover new scrutins upstream — that needs an AN listing source and
//! is a separate lane (ADR-0002). Deriving the work set from persisted rows is
//! what makes a re-run converge (US-8.4).

use crate::ports::RepoError;
use crate::usecase::IngestScrutin;
use async_trait::async_trait;
use chrono::NaiveDate;
use lindex_domain::{Chamber, ScrutinId};

/// Which scrutins are on record for a chamber over an inclusive day-range.
///
/// This is how a date-range re-ingest learns its work set. The range is over the
/// sitting date (`held_on`), inclusive of both bounds. Order is unspecified.
#[async_trait]
pub trait ScrutinCalendar: Send + Sync {
    /// Ids of every persisted scrutin of `chamber` whose `held_on` is in
    /// `[from, to]` (inclusive).
    async fn ids_in_range(
        &self,
        chamber: Chamber,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<ScrutinId>, RepoError>;
}

/// Where ingestion-failure notifications go (US-8.1). The adapter chooses the
/// transport (log line, webhook, …); the use case only decides *when* to alert.
#[async_trait]
pub trait Alerter: Send + Sync {
    /// Deliver one alert. A transport failure is reported, never panicked — the
    /// caller treats alerting as best-effort so it cannot mask the ingest result.
    async fn alert(&self, alert: &IngestionAlert) -> Result<(), AlertError>;
}

/// A single scrutin that failed to (re-)ingest, with the reason for the operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailedItem {
    /// The scrutin id that failed.
    pub id: String,
    /// Human-readable failure reason (the underlying error, stringified).
    pub reason: String,
}

/// The payload handed to an [`Alerter`] when a scheduled run has failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestionAlert {
    /// The run target, e.g. `reingest:AN:2026-07-20..2026-07-21`.
    pub target: String,
    /// The scrutins that failed within the run.
    pub failed: Vec<FailedItem>,
    /// A one-line human summary.
    pub message: String,
}

/// Outcome of a date-range re-ingest — the operator's audit unit (US-8.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReingestReport {
    /// The run target, e.g. `reingest:AN:2026-07-20..2026-07-21`.
    pub target: String,
    /// How many scrutins were in the work set.
    pub attempted: usize,
    /// How many re-ingested successfully.
    pub succeeded: usize,
    /// The failures (empty on a fully successful run).
    pub failures: Vec<FailedItem>,
}

/// Re-ingest every scrutin of a chamber held in `[from, to]`, idempotently.
///
/// Reuses the single-scrutin [`IngestScrutin`] use case per id, so provenance and
/// day-median baselines are re-projected exactly as on first ingest. If any id
/// fails, the whole batch is still attempted and an [`Alerter`] is fired once with
/// the failures — a partial failure never aborts the sweep (US-8.1).
pub struct ReingestRange<'a> {
    /// Discovers the work set from persisted rows.
    pub calendar: &'a dyn ScrutinCalendar,
    /// The idempotent single-scrutin ingest, replayed per id.
    pub ingest: IngestScrutin<'a>,
    /// Where a failure is reported.
    pub alerter: &'a dyn Alerter,
}

impl ReingestRange<'_> {
    /// Run the re-ingest. Returns the [`ReingestReport`]; the caller records it to
    /// `facts.ingestion_run` and sets the process exit code.
    pub async fn run(
        &self,
        chamber: Chamber,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<ReingestReport, RepoError> {
        let target = format!("reingest:{}:{}..{}", chamber_code(chamber), from, to);
        let ids = self.calendar.ids_in_range(chamber, from, to).await?;
        let attempted = ids.len();
        let mut succeeded = 0usize;
        let mut failures = Vec::new();
        for id in &ids {
            match self.ingest.run(id).await {
                Ok(()) => succeeded += 1,
                Err(e) => failures.push(FailedItem {
                    id: id.0.clone(),
                    reason: e.to_string(),
                }),
            }
        }
        if !failures.is_empty() {
            let alert = IngestionAlert {
                target: target.clone(),
                failed: failures.clone(),
                message: format!(
                    "{} of {} scrutins failed to re-ingest ({target})",
                    failures.len(),
                    attempted
                ),
            };
            // Best-effort: a broken alert transport must not mask the report.
            let _ = self.alerter.alert(&alert).await;
        }
        Ok(ReingestReport {
            target,
            attempted,
            succeeded,
            failures,
        })
    }
}

/// Stable chamber code used in run targets (matches the persistence encoding).
fn chamber_code(chamber: Chamber) -> &'static str {
    match chamber {
        Chamber::AssembleeNationale => "AN",
        Chamber::Senat => "SENAT",
    }
}

/// Failure delivering an alert. Reported, never fatal.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum AlertError {
    /// The alert transport (webhook, mailer, …) failed.
    #[error("alert transport failed: {0}")]
    Transport(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{
        ParliamentSource, RawGroup, RawScrutin, RawScrutinData, RawTotals, ReadModelStore,
        ScrutinRepository, ScrutinView, SourceError,
    };
    use lindex_domain::{Provenance, ProvenanceTier, Scrutin, SourceRef, Sourced, VoteTotals};
    use std::collections::{HashMap, HashSet};
    use std::sync::Mutex;

    fn raw_data(id: &str) -> RawScrutinData {
        RawScrutinData {
            scrutin_id: id.into(),
            chamber: "AN".into(),
            title: format!("Scrutin {id}"),
            held_on: "2026-07-21".into(),
            outcome: "adopte".into(),
            totals: RawTotals {
                pour: 351,
                contre: 179,
                abstention: 7,
                non_votants: 0,
                members_total: 577,
                votants: 537,
                exprimes: 530,
            },
            groups: vec![
                RawGroup {
                    group: "LFI".into(),
                    pour: 0,
                    contre: 179,
                    abstention: 0,
                    non_votant: 0,
                },
                RawGroup {
                    group: "EPR".into(),
                    pour: 351,
                    contre: 0,
                    abstention: 7,
                    non_votant: 0,
                },
            ],
        }
    }

    /// Serves a fixed set of normalized records; fails for any id in `fail_ids`.
    struct FakeSource {
        payloads: HashMap<String, String>,
        fail_ids: HashSet<String>,
    }
    impl FakeSource {
        fn new(ids: &[&str], fail_ids: &[&str]) -> Self {
            Self {
                payloads: ids
                    .iter()
                    .map(|id| {
                        (
                            (*id).to_string(),
                            serde_json::to_string(&raw_data(id)).expect("serialize fixture"),
                        )
                    })
                    .collect(),
                fail_ids: fail_ids.iter().map(|s| (*s).to_string()).collect(),
            }
        }
    }
    #[async_trait]
    impl ParliamentSource for FakeSource {
        async fn fetch_scrutin(&self, id: &ScrutinId) -> Result<Sourced<RawScrutin>, SourceError> {
            if self.fail_ids.contains(&id.0) {
                return Err(SourceError::Unavailable(format!("boom {}", id.0)));
            }
            let payload = self
                .payloads
                .get(&id.0)
                .cloned()
                .ok_or_else(|| SourceError::Unavailable(format!("no fixture {}", id.0)))?;
            let provenance = Provenance {
                source: SourceRef {
                    tier: ProvenanceTier::ActeAuthentique,
                    label: format!("Scrutin n° {} — AN", id.0),
                    url: Some(format!("https://example.test/scrutins/{}", id.0)),
                    record_id: format!("an-scrutin-{}", id.0),
                },
                retrieved_at: chrono::Utc::now(),
            };
            Ok(Sourced::new(RawScrutin { payload }, provenance))
        }
    }

    #[derive(Default)]
    struct FakeRepo {
        source_records: Mutex<HashMap<String, String>>,
        scrutins: Mutex<HashMap<String, Scrutin>>,
    }
    #[async_trait]
    impl ScrutinRepository for FakeRepo {
        async fn put_source_record(&self, raw: &Sourced<RawScrutin>) -> Result<(), RepoError> {
            let id = raw.provenance().source.record_id.clone();
            self.source_records
                .lock()
                .expect("lock")
                .insert(id, raw.value().payload.clone());
            Ok(())
        }
        async fn upsert(&self, scrutin: &Sourced<Scrutin>) -> Result<(), RepoError> {
            let s = scrutin.value().clone();
            self.scrutins
                .lock()
                .expect("lock")
                .insert(s.id.0.clone(), s);
            Ok(())
        }
        async fn by_id(&self, id: &ScrutinId) -> Result<Option<Scrutin>, RepoError> {
            Ok(self.scrutins.lock().expect("lock").get(&id.0).cloned())
        }
        async fn cohort_totals(
            &self,
            chamber: &Chamber,
            held_on: NaiveDate,
        ) -> Result<Vec<VoteTotals>, RepoError> {
            Ok(self
                .scrutins
                .lock()
                .expect("lock")
                .values()
                .filter(|s| s.chamber == *chamber && s.held_on == held_on)
                .map(|s| s.totals)
                .collect())
        }
    }

    #[derive(Default)]
    struct FakeRead {
        views: Mutex<HashMap<String, ScrutinView>>,
    }
    #[async_trait]
    impl ReadModelStore for FakeRead {
        async fn put_scrutin_view(&self, view: &ScrutinView) -> Result<(), RepoError> {
            self.views
                .lock()
                .expect("lock")
                .insert(view.scrutin_id.clone(), view.clone());
            Ok(())
        }
        async fn refresh_day_baselines(
            &self,
            chamber: &str,
            held_on: NaiveDate,
            baselines_json: &str,
        ) -> Result<(), RepoError> {
            let held_on = held_on.to_string();
            for view in self.views.lock().expect("lock").values_mut() {
                if view.chamber == chamber && view.held_on == held_on {
                    view.baselines_json = baselines_json.to_string();
                }
            }
            Ok(())
        }
    }

    struct FakeCalendar {
        ids: Vec<ScrutinId>,
    }
    #[async_trait]
    impl ScrutinCalendar for FakeCalendar {
        async fn ids_in_range(
            &self,
            _chamber: Chamber,
            _from: NaiveDate,
            _to: NaiveDate,
        ) -> Result<Vec<ScrutinId>, RepoError> {
            Ok(self.ids.clone())
        }
    }

    #[derive(Default)]
    struct RecordingAlerter {
        alerts: Mutex<Vec<IngestionAlert>>,
    }
    #[async_trait]
    impl Alerter for RecordingAlerter {
        async fn alert(&self, alert: &IngestionAlert) -> Result<(), AlertError> {
            self.alerts.lock().expect("lock").push(alert.clone());
            Ok(())
        }
    }

    fn ids(list: &[&str]) -> Vec<ScrutinId> {
        list.iter().map(|s| ScrutinId((*s).into())).collect()
    }

    const DAY: &str = "2026-07-21";
    fn day() -> NaiveDate {
        NaiveDate::parse_from_str(DAY, "%Y-%m-%d").expect("date")
    }

    #[tokio::test]
    async fn reingest_runs_each_id_and_reports_success() {
        let source = FakeSource::new(&["8433", "8430"], &[]);
        let repo = FakeRepo::default();
        let read = FakeRead::default();
        let calendar = FakeCalendar {
            ids: ids(&["8433", "8430"]),
        };
        let alerter = RecordingAlerter::default();
        let rr = ReingestRange {
            calendar: &calendar,
            ingest: IngestScrutin {
                source: &source,
                repo: &repo,
                read: &read,
            },
            alerter: &alerter,
        };

        let report = rr
            .run(Chamber::AssembleeNationale, day(), day())
            .await
            .expect("run");

        assert_eq!(report.attempted, 2);
        assert_eq!(report.succeeded, 2);
        assert!(report.failures.is_empty());
        assert_eq!(report.target, "reingest:AN:2026-07-21..2026-07-21");
        // No failure → no alert.
        assert_eq!(alerter.alerts.lock().expect("lock").len(), 0);
    }

    #[tokio::test]
    async fn reingest_alerts_on_failure() {
        let source = FakeSource::new(&["8433", "8430"], &["8430"]);
        let repo = FakeRepo::default();
        let read = FakeRead::default();
        let calendar = FakeCalendar {
            ids: ids(&["8433", "8430"]),
        };
        let alerter = RecordingAlerter::default();
        let rr = ReingestRange {
            calendar: &calendar,
            ingest: IngestScrutin {
                source: &source,
                repo: &repo,
                read: &read,
            },
            alerter: &alerter,
        };

        let report = rr
            .run(Chamber::AssembleeNationale, day(), day())
            .await
            .expect("run");

        assert_eq!(report.attempted, 2);
        assert_eq!(report.succeeded, 1);
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].id, "8430");

        // Exactly one alert, naming the failed id and the run target.
        let alerts = alerter.alerts.lock().expect("lock");
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].target, report.target);
        assert_eq!(alerts[0].failed.len(), 1);
        assert_eq!(alerts[0].failed[0].id, "8430");
    }

    #[tokio::test]
    async fn reingest_is_idempotent() {
        let source = FakeSource::new(&["8433", "8430"], &[]);
        let repo = FakeRepo::default();
        let read = FakeRead::default();
        let calendar = FakeCalendar {
            ids: ids(&["8433", "8430"]),
        };
        let alerter = RecordingAlerter::default();
        let rr = ReingestRange {
            calendar: &calendar,
            ingest: IngestScrutin {
                source: &source,
                repo: &repo,
                read: &read,
            },
            alerter: &alerter,
        };

        let first = rr
            .run(Chamber::AssembleeNationale, day(), day())
            .await
            .expect("run");
        let second = rr
            .run(Chamber::AssembleeNationale, day(), day())
            .await
            .expect("run");

        // Same work set, replayed: converges to one row per id in every store.
        assert_eq!(repo.scrutins.lock().expect("lock").len(), 2);
        assert_eq!(repo.source_records.lock().expect("lock").len(), 2);
        assert_eq!(read.views.lock().expect("lock").len(), 2);
        assert_eq!(first.attempted, second.attempted);
        assert_eq!(first.succeeded, second.succeeded);
        assert_eq!(first.target, second.target);
    }

    #[tokio::test]
    async fn reingest_empty_range_does_not_alert() {
        let source = FakeSource::new(&[], &[]);
        let repo = FakeRepo::default();
        let read = FakeRead::default();
        let calendar = FakeCalendar { ids: Vec::new() };
        let alerter = RecordingAlerter::default();
        let rr = ReingestRange {
            calendar: &calendar,
            ingest: IngestScrutin {
                source: &source,
                repo: &repo,
                read: &read,
            },
            alerter: &alerter,
        };

        let report = rr
            .run(Chamber::AssembleeNationale, day(), day())
            .await
            .expect("run");

        assert_eq!(report.attempted, 0);
        assert_eq!(report.succeeded, 0);
        assert!(report.failures.is_empty());
        assert_eq!(alerter.alerts.lock().expect("lock").len(), 0);
    }
}
