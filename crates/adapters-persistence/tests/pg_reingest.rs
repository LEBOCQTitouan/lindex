//! End-to-end date-range re-ingest against a real Postgres (US-8.4).
//!
//! Gated on `DATABASE_URL`: with no database (as in CI) the test returns early
//! and passes. Run locally with `just db-up` and
//! `DATABASE_URL=postgres://lindex:password@localhost/lindex cargo nextest run \
//!   -p lindex-adapters-persistence`.

use async_trait::async_trait;
use lindex_adapters_persistence::PgStore;
use lindex_application::ops::{
    AlertError, Alerter, IngestionAlert, ReingestRange, ScrutinCalendar,
};
use lindex_application::ports::{
    ParliamentSource, RawGroup, RawScrutin, RawScrutinData, RawTotals, SourceError,
};
use lindex_application::usecase::IngestScrutin;
use lindex_domain::{Chamber, Provenance, ProvenanceTier, ScrutinId, SourceRef, Sourced};
use sqlx::{PgPool, Row};

/// Replays a fixed normalized record for whichever id it is asked for — exercises
/// the SQL, not HTTP.
struct StubSource {
    data: RawScrutinData,
}

#[async_trait]
impl ParliamentSource for StubSource {
    async fn fetch_scrutin(&self, id: &ScrutinId) -> Result<Sourced<RawScrutin>, SourceError> {
        let mut data = self.data.clone();
        data.scrutin_id = id.0.clone();
        let payload = serde_json::to_string(&data).expect("serialize");
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

/// A no-op alerter for the happy-path re-ingest.
struct NoopAlerter;
#[async_trait]
impl Alerter for NoopAlerter {
    async fn alert(&self, _alert: &IngestionAlert) -> Result<(), AlertError> {
        Ok(())
    }
}

fn data_for(id: &str) -> RawScrutinData {
    RawScrutinData {
        scrutin_id: id.into(),
        chamber: "AN".into(),
        title: format!("Scrutin public n°{id} — ordre public"),
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

async fn migrate(pool: &PgPool) {
    for sql in [
        include_str!("../../../db/migrations/0001_facts.sql"),
        include_str!("../../../db/migrations/0002_app.sql"),
    ] {
        sqlx::raw_sql(sql).execute(pool).await.expect("migrate");
    }
}

fn day(s: &str) -> chrono::NaiveDate {
    chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("date")
}

async fn ingest_one(store: &PgStore, id: &str) {
    let source = StubSource { data: data_for(id) };
    IngestScrutin {
        source: &source,
        repo: store,
        read: store,
    }
    .run(&ScrutinId(id.into()))
    .await
    .expect("ingest");
}

#[tokio::test]
async fn reingest_range_is_idempotent_over_persisted_scrutins() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!("skip pg_reingest: DATABASE_URL unset");
        return;
    };
    let pool = PgPool::connect(&url).await.expect("connect");
    migrate(&pool).await;
    for id in ["8433", "8430"] {
        for table in ["read_scrutin", "scrutin"] {
            let col = if table == "read_scrutin" {
                "scrutin_id"
            } else {
                "id"
            };
            sqlx::query(&format!("delete from facts.{table} where {col} = $1"))
                .bind(id)
                .execute(&pool)
                .await
                .expect("clean");
        }
    }

    let store = PgStore::new(pool.clone());
    ingest_one(&store, "8433").await;
    ingest_one(&store, "8430").await;

    // The calendar sees both same-day scrutins, and none the day after.
    let in_day = store
        .ids_in_range(
            Chamber::AssembleeNationale,
            day("2026-07-21"),
            day("2026-07-21"),
        )
        .await
        .expect("ids");
    let found: std::collections::HashSet<String> = in_day.iter().map(|i| i.0.clone()).collect();
    assert!(found.contains("8433"));
    assert!(found.contains("8430"));
    let next_day = store
        .ids_in_range(
            Chamber::AssembleeNationale,
            day("2026-07-22"),
            day("2026-07-22"),
        )
        .await
        .expect("ids");
    assert!(!next_day.iter().any(|i| i.0 == "8433" || i.0 == "8430"));

    // Re-ingest the whole day — twice — and assert convergence.
    let source = StubSource {
        data: data_for("placeholder"),
    };
    let alerter = NoopAlerter;
    let rr = ReingestRange {
        calendar: &store,
        ingest: IngestScrutin {
            source: &source,
            repo: &store,
            read: &store,
        },
        alerter: &alerter,
    };
    let report = rr
        .run(
            Chamber::AssembleeNationale,
            day("2026-07-21"),
            day("2026-07-21"),
        )
        .await
        .expect("reingest");
    assert_eq!(report.attempted, 2);
    assert!(report.failures.is_empty());
    rr.run(
        Chamber::AssembleeNationale,
        day("2026-07-21"),
        day("2026-07-21"),
    )
    .await
    .expect("reingest again");

    for id in ["8433", "8430"] {
        let n: i64 = sqlx::query("select count(*) from facts.scrutin where id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("count")
            .get(0);
        assert_eq!(n, 1, "one facts.scrutin row for {id} after re-ingest");
        let rn: i64 = sqlx::query("select count(*) from facts.read_scrutin where scrutin_id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .expect("count")
            .get(0);
        assert_eq!(rn, 1, "one read_scrutin row for {id} after re-ingest");
    }
}
