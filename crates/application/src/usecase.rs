//! Use cases. Orchestrate ports; hold no I/O of their own.

use crate::ports::{
    ParliamentSource, RawScrutinData, ReadModelStore, RepoError, ScrutinRepository, ScrutinView,
    SourceError,
};
use chrono::NaiveDate;
use lindex_domain::{
    Baseline, Chamber, DomainError, GroupId, GroupTally, ProvenanceTier, Scrutin, ScrutinId,
    Sourced, VoteTotals,
};
use serde::Serialize;

/// Ingest one scrutin: fetch raw → map to domain → persist → project read-model.
/// Idempotent: re-running the same id reproduces the same state (US-8.4).
pub struct IngestScrutin<'a> {
    pub source: &'a dyn ParliamentSource,
    pub repo: &'a dyn ScrutinRepository,
    pub read: &'a dyn ReadModelStore,
}

impl IngestScrutin<'_> {
    pub async fn run(&self, id: &ScrutinId) -> Result<(), IngestError> {
        // 1. Fetch the raw (already normalized) source record.
        let raw = self.source.fetch_scrutin(id).await?;
        let data: RawScrutinData = serde_json::from_str(&raw.value().payload)
            .map_err(|e| IngestError::Mapping(format!("payload is not RawScrutinData: {e}")))?;

        // 2. Map the normalized record to the validated domain model.
        let scrutin = map_to_domain(&data)?;
        let scrutin = Sourced::new(scrutin, raw.provenance().clone());
        let chamber = scrutin.value().chamber;
        let held_on = scrutin.value().held_on;

        // 3. Persist: raw record first (FK target), then the unified model.
        self.repo.put_source_record(&raw).await?;
        self.repo.upsert(&scrutin).await?;

        // 4. Project the read-model with day-median baselines over the cohort.
        let cohort = self.repo.cohort_totals(&chamber, held_on).await?;
        let baselines = day_medians(&cohort)?;
        let view = build_view(&data, scrutin.provenance(), &baselines)?;
        self.read.put_scrutin_view(&view).await?;
        // The day-median just moved for every same-day scrutin, not only this
        // one — refresh the whole cohort's baselines so no peer keeps a stale,
        // smaller-sample median (ADR-0001 §3).
        self.read
            .refresh_day_baselines(&data.chamber, held_on, &view.baselines_json)
            .await?;
        Ok(())
    }
}

fn map_to_domain(data: &RawScrutinData) -> Result<Scrutin, IngestError> {
    let chamber = match data.chamber.as_str() {
        "AN" => Chamber::AssembleeNationale,
        "SENAT" => Chamber::Senat,
        other => return Err(IngestError::Mapping(format!("unknown chamber {other:?}"))),
    };
    let held_on = NaiveDate::parse_from_str(&data.held_on, "%Y-%m-%d")
        .map_err(|e| IngestError::Mapping(format!("bad held_on {:?}: {e}", data.held_on)))?;
    let totals = VoteTotals {
        pour: data.totals.pour,
        contre: data.totals.contre,
        abstention: data.totals.abstention,
        non_votants: data.totals.non_votants,
        members_total: data.totals.members_total,
    };
    let breakdown = data
        .groups
        .iter()
        .map(|g| GroupTally {
            group: GroupId(g.group.clone()),
            pour: g.pour,
            contre: g.contre,
            abstention: g.abstention,
            non_votant: g.non_votant,
        })
        .collect();
    Scrutin::try_new(
        ScrutinId(data.scrutin_id.clone()),
        chamber,
        data.title.clone(),
        held_on,
        totals,
        breakdown,
    )
    .map_err(IngestError::Domain)
}

/// Day-median baselines: the median `votants` and `abstention` over comparable
/// scrutins (same chamber, same day). Never optional — display rule #1.
struct DayBaselines {
    votants: Baseline,
    abstention: Baseline,
}

fn day_medians(cohort: &[VoteTotals]) -> Result<DayBaselines, IngestError> {
    let votants: Vec<f64> = cohort.iter().map(|t| f64::from(t.votants())).collect();
    let abstention: Vec<f64> = cohort.iter().map(|t| f64::from(t.abstention)).collect();
    match (
        Baseline::from_samples(&votants),
        Baseline::from_samples(&abstention),
    ) {
        (Some(votants), Some(abstention)) => Ok(DayBaselines {
            votants,
            abstention,
        }),
        // The cohort always contains at least the just-upserted scrutin, so this
        // is unreachable in practice; we return an error rather than panic (R3).
        _ => Err(IngestError::EmptyCohort),
    }
}

fn build_view(
    data: &RawScrutinData,
    provenance: &lindex_domain::Provenance,
    baselines: &DayBaselines,
) -> Result<ScrutinView, IngestError> {
    let baselines_json = BaselinesJson {
        votants: BaselineJson::from(baselines.votants),
        abstention: BaselineJson::from(baselines.abstention),
        method: MethodJson {
            id: "scrutin-day-median",
            version: 1,
        },
    };
    let provenance_json = ProvenanceJson {
        tier: tier_name(provenance.source.tier).to_string(),
        label: provenance.source.label.clone(),
        url: provenance.source.url.clone(),
        record_id: provenance.source.record_id.clone(),
        retrieved_at: provenance.retrieved_at.to_rfc3339(),
    };
    Ok(ScrutinView {
        scrutin_id: data.scrutin_id.clone(),
        chamber: data.chamber.clone(),
        title: data.title.clone(),
        held_on: data.held_on.clone(),
        outcome: data.outcome.clone(),
        totals_json: json(&data.totals, "totals")?,
        breakdown_json: json(&data.groups, "breakdown")?,
        baselines_json: json(&baselines_json, "baselines")?,
        provenance_json: json(&provenance_json, "provenance")?,
    })
}

/// Serialize a read-model fragment, tagging any error with the field name.
fn json<T: Serialize>(value: &T, what: &str) -> Result<String, IngestError> {
    serde_json::to_string(value).map_err(|e| IngestError::Serialization(format!("{what}: {e}")))
}

fn tier_name(tier: ProvenanceTier) -> &'static str {
    match tier {
        ProvenanceTier::ActeAuthentique => "ActeAuthentique",
        ProvenanceTier::OrganismeIndependant => "OrganismeIndependant",
        ProvenanceTier::CommunicationGouv => "CommunicationGouv",
        ProvenanceTier::TravauxParlementaires => "TravauxParlementaires",
        ProvenanceTier::PresseSocieteCivile => "PresseSocieteCivile",
    }
}

#[derive(Serialize)]
struct BaselineJson {
    median: f64,
    #[serde(rename = "sampleSize")]
    sample_size: u32,
}
impl From<Baseline> for BaselineJson {
    fn from(b: Baseline) -> Self {
        Self {
            median: b.median,
            sample_size: b.sample_size,
        }
    }
}
#[derive(Serialize)]
struct MethodJson {
    id: &'static str,
    version: u32,
}
#[derive(Serialize)]
struct BaselinesJson {
    votants: BaselineJson,
    abstention: BaselineJson,
    method: MethodJson,
}
#[derive(Serialize)]
struct ProvenanceJson {
    tier: String,
    label: String,
    url: Option<String>,
    #[serde(rename = "recordId")]
    record_id: String,
    #[serde(rename = "retrievedAt")]
    retrieved_at: String,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum IngestError {
    #[error(transparent)]
    Source(#[from] SourceError),
    #[error(transparent)]
    Repo(#[from] RepoError),
    #[error("mapping source record to domain: {0}")]
    Mapping(String),
    #[error("domain validation: {0}")]
    Domain(#[from] DomainError),
    #[error("serializing read-model: {0}")]
    Serialization(String),
    #[error("baseline cohort was empty")]
    EmptyCohort,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{RawGroup, RawScrutin, RawTotals};
    use async_trait::async_trait;
    use lindex_domain::{Provenance, SourceRef};
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// A reconciling two-group payload driven by its top-line numbers, so tests
    /// can vary `votants`/`abstention` to exercise the day-median.
    fn raw_data(id: &str, pour: u32, contre: u32, abstention: u32) -> RawScrutinData {
        RawScrutinData {
            scrutin_id: id.into(),
            chamber: "AN".into(),
            title: "Ordre public".into(),
            held_on: "2026-07-21".into(),
            outcome: "adopte".into(),
            totals: RawTotals {
                pour,
                contre,
                abstention,
                non_votants: 0,
                members_total: 577,
                votants: pour + contre + abstention,
                exprimes: pour + contre,
            },
            groups: vec![
                RawGroup {
                    group: "LFI".into(),
                    pour: 0,
                    contre,
                    abstention: 0,
                    non_votant: 0,
                },
                RawGroup {
                    group: "EPR".into(),
                    pour,
                    contre: 0,
                    abstention,
                    non_votant: 0,
                },
            ],
        }
    }

    struct FakeSource {
        id: String,
        payload: String,
    }
    impl FakeSource {
        fn new(data: &RawScrutinData) -> Self {
            Self {
                id: data.scrutin_id.clone(),
                payload: serde_json::to_string(data).expect("serialize fixture"),
            }
        }
    }
    #[async_trait]
    impl ParliamentSource for FakeSource {
        async fn fetch_scrutin(&self, _id: &ScrutinId) -> Result<Sourced<RawScrutin>, SourceError> {
            let provenance = Provenance {
                source: SourceRef {
                    tier: ProvenanceTier::ActeAuthentique,
                    label: format!("Scrutin n° {} — AN", self.id),
                    url: Some(format!(
                        "https://www.assemblee-nationale.fr/dyn/17/scrutins/{}",
                        self.id
                    )),
                    record_id: format!("an-scrutin-{}", self.id),
                },
                retrieved_at: chrono::Utc::now(),
            };
            Ok(Sourced::new(
                RawScrutin {
                    payload: self.payload.clone(),
                },
                provenance,
            ))
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

    async fn ingest(source: &FakeSource, repo: &FakeRepo, read: &FakeRead, id: &str) {
        IngestScrutin { source, repo, read }
            .run(&ScrutinId(id.into()))
            .await
            .expect("ingest ok");
    }

    #[tokio::test]
    async fn ingesting_the_same_scrutin_twice_is_idempotent() {
        let data = raw_data("8433", 351, 179, 7);
        let source = FakeSource::new(&data);
        let repo = FakeRepo::default();
        let read = FakeRead::default();

        ingest(&source, &repo, &read, "8433").await;
        let first = read.views.lock().expect("lock").get("8433").cloned();
        ingest(&source, &repo, &read, "8433").await;
        let second = read.views.lock().expect("lock").get("8433").cloned();

        // Re-ingest converges to a single row per store — no duplication.
        assert_eq!(repo.scrutins.lock().expect("lock").len(), 1);
        assert_eq!(repo.source_records.lock().expect("lock").len(), 1);
        assert_eq!(read.views.lock().expect("lock").len(), 1);
        // The facts are identical; only the retrieval timestamp legitimately
        // advances (it drives the "last updated" surface, US-7.4).
        let (first, second) = (first.expect("view"), second.expect("view"));
        assert_eq!(first.totals_json, second.totals_json);
        assert_eq!(first.breakdown_json, second.breakdown_json);
        assert_eq!(first.baselines_json, second.baselines_json);
        assert_eq!(first.outcome, second.outcome);
    }

    #[tokio::test]
    async fn projects_the_read_model_contract() {
        let data = raw_data("8433", 351, 179, 7);
        let source = FakeSource::new(&data);
        let repo = FakeRepo::default();
        let read = FakeRead::default();
        ingest(&source, &repo, &read, "8433").await;

        let view = read
            .views
            .lock()
            .expect("lock")
            .get("8433")
            .cloned()
            .expect("view");
        assert_eq!(view.chamber, "AN");
        assert_eq!(view.outcome, "adopte");
        assert_eq!(view.held_on, "2026-07-21");

        let totals: serde_json::Value = serde_json::from_str(&view.totals_json).expect("json");
        assert_eq!(totals["membersTotal"], 577);
        assert_eq!(totals["votants"], 537);
        assert_eq!(totals["nonVotants"], 0);

        // Breakdown keeps hémicycle order (LFI before EPR) — symmetry (P5).
        let breakdown: serde_json::Value =
            serde_json::from_str(&view.breakdown_json).expect("json");
        assert_eq!(breakdown[0]["group"], "LFI");
        assert_eq!(breakdown[1]["group"], "EPR");
        assert_eq!(breakdown[1]["nonVotant"], 0);

        // Provenance tier + source link on every datum (P4).
        let prov: serde_json::Value = serde_json::from_str(&view.provenance_json).expect("json");
        assert_eq!(prov["tier"], "ActeAuthentique");
        assert_eq!(prov["recordId"], "an-scrutin-8433");
        assert!(prov["url"]
            .as_str()
            .expect("url")
            .ends_with("/scrutins/8433"));

        // Baseline present, with its method (P1).
        let base: serde_json::Value = serde_json::from_str(&view.baselines_json).expect("json");
        assert_eq!(base["method"]["id"], "scrutin-day-median");
        assert_eq!(base["votants"]["median"], 537.0);
        assert_eq!(base["votants"]["sampleSize"], 1);
    }

    #[tokio::test]
    async fn baseline_is_the_day_median_over_same_day_scrutins() {
        let repo = FakeRepo::default();
        let read = FakeRead::default();

        let a = raw_data("8433", 351, 179, 7); // votants 537
        let b = raw_data("8430", 378, 7, 173); // votants 558, same day
        let src_a = FakeSource::new(&a);
        let src_b = FakeSource::new(&b);
        ingest(&src_a, &repo, &read, "8433").await;
        ingest(&src_b, &repo, &read, "8430").await;

        let base_of = |id: &str| -> serde_json::Value {
            let v = read
                .views
                .lock()
                .expect("lock")
                .get(id)
                .cloned()
                .expect("view");
            serde_json::from_str(&v.baselines_json).expect("json")
        };
        // median votants over [537, 558] = 547.5; abstention over [7, 173] = 90.
        let base_b = base_of("8430");
        assert_eq!(base_b["votants"]["median"], 547.5);
        assert_eq!(base_b["votants"]["sampleSize"], 2);
        assert_eq!(base_b["abstention"]["median"], 90.0);
        // The earlier scrutin's baseline was refreshed too — not left stale at
        // sampleSize 1 (ADR-0001 §3; the day-median is shared across the cohort).
        let base_a = base_of("8433");
        assert_eq!(base_a["votants"]["median"], 547.5);
        assert_eq!(base_a["votants"]["sampleSize"], 2);
        assert_eq!(base_a["abstention"]["median"], 90.0);
    }
}
