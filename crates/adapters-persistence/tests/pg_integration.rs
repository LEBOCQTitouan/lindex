//! End-to-end persistence test against a real Postgres.
//!
//! Gated on `DATABASE_URL`: with no database (as in CI) the test returns early
//! and passes. Run locally with `just db-up` and
//! `DATABASE_URL=postgres://lindex:password@localhost/lindex cargo nextest run \
//!   -p lindex-adapters-persistence`.

use async_trait::async_trait;
use lindex_adapters_persistence::PgStore;
use lindex_application::ports::{
    ParliamentSource, RawGroup, RawScrutin, RawScrutinData, RawTotals, ScrutinRepository,
    SourceError,
};
use lindex_application::usecase::IngestScrutin;
use lindex_domain::{Provenance, ProvenanceTier, ScrutinId, SourceRef, Sourced};
use sqlx::{PgPool, Row};

/// A source that replays a fixed normalized record — exercises the SQL, not HTTP.
struct StubSource {
    data: RawScrutinData,
}

#[async_trait]
impl ParliamentSource for StubSource {
    async fn fetch_scrutin(&self, id: &ScrutinId) -> Result<Sourced<RawScrutin>, SourceError> {
        let payload = serde_json::to_string(&self.data).expect("serialize");
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

fn data_8433() -> RawScrutinData {
    RawScrutinData {
        scrutin_id: "8433".into(),
        chamber: "AN".into(),
        title: "Scrutin public n°8433 — ordre public (CMP)".into(),
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
    // `raw_sql` uses the simple query protocol, so each file's multiple
    // statements (and SQL comments) run in one call. Statements are
    // `if not exists`, so replaying every run is safe.
    for sql in [
        include_str!("../../../db/migrations/0001_facts.sql"),
        include_str!("../../../db/migrations/0002_app.sql"),
    ] {
        sqlx::raw_sql(sql).execute(pool).await.expect("migrate");
    }
}

#[tokio::test]
async fn ingest_persists_and_is_idempotent() {
    let Ok(url) = std::env::var("DATABASE_URL") else {
        eprintln!("skip pg_integration: DATABASE_URL unset");
        return;
    };
    let pool = PgPool::connect(&url).await.expect("connect");
    migrate(&pool).await;
    // Clean slate for a deterministic assertion.
    sqlx::query("delete from facts.read_scrutin where scrutin_id = '8433'")
        .execute(&pool)
        .await
        .expect("clean read");
    sqlx::query("delete from facts.scrutin where id = '8433'")
        .execute(&pool)
        .await
        .expect("clean scrutin");

    let source = StubSource { data: data_8433() };
    let store = PgStore::new(pool.clone());
    let id = ScrutinId("8433".into());

    for _ in 0..2 {
        IngestScrutin {
            source: &source,
            repo: &store,
            read: &store,
        }
        .run(&id)
        .await
        .expect("ingest");
    }

    // Idempotent: exactly one row in each table.
    let scrutin_rows: i64 = sqlx::query("select count(*) from facts.scrutin where id = '8433'")
        .fetch_one(&pool)
        .await
        .expect("count")
        .get(0);
    assert_eq!(scrutin_rows, 1);

    let read_rows: i64 =
        sqlx::query("select count(*) from facts.read_scrutin where scrutin_id = '8433'")
            .fetch_one(&pool)
            .await
            .expect("count")
            .get(0);
    assert_eq!(read_rows, 1);

    // Read-model carries provenance + a non-null baseline (rules #1, P4).
    let row = sqlx::query(
        "select outcome, baselines->'method'->>'id' as method, \
                provenance->>'tier' as tier \
         from facts.read_scrutin where scrutin_id = '8433'",
    )
    .fetch_one(&pool)
    .await
    .expect("read row");
    assert_eq!(row.get::<String, _>("outcome"), "adopte");
    assert_eq!(row.get::<String, _>("method"), "scrutin-day-median");
    assert_eq!(row.get::<String, _>("tier"), "ActeAuthentique");

    // Unified model round-trips through the domain.
    let back = store.by_id(&id).await.expect("by_id").expect("present");
    assert_eq!(back.totals.pour, 351);
    assert_eq!(back.breakdown.len(), 2);
}
