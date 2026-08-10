#![forbid(unsafe_code)]
//! Ingest CLI — composition root. Wires concrete adapters to the application
//! ports and runs use cases. Scheduled in production by systemd timers.
//!
//! Usage: `cargo run -p ingest -- scrutin 8433`  (idempotent, re-runnable)

use anyhow::{bail, Context, Result};
use lindex_adapters_persistence::PgStore;
use lindex_adapters_sources::AnSource;
use lindex_application::usecase::IngestScrutin;
use lindex_domain::ScrutinId;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("scrutin") => {
            let raw_id = args.get(2).cloned().unwrap_or_default();
            if raw_id.is_empty() {
                bail!("usage: ingest scrutin <id>");
            }
            let id = ScrutinId(raw_id);

            // --- composition: build adapters, satisfy ports ---
            let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
            let pool = PgPool::connect(&database_url)
                .await
                .context("connecting to Postgres")?;
            let store = PgStore::new(pool.clone());
            let source = AnSource::new();

            let usecase = IngestScrutin {
                source: &source,
                repo: &store,
                read: &store,
            };
            let outcome = usecase.run(&id).await;
            record_run(&pool, &format!("scrutin:{}", id.0), &outcome).await;
            outcome.context("ingesting scrutin")?;
            println!("ingested scrutin {}", id.0);
        }
        _ => eprintln!("usage: ingest scrutin <id>"),
    }
    Ok(())
}

/// Best-effort audit row (US-8.1). A failure to record must not mask the
/// ingestion result itself, so errors here are logged, not propagated.
async fn record_run<E: std::fmt::Display>(pool: &PgPool, target: &str, outcome: &Result<(), E>) {
    let (ok, detail) = match outcome {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.to_string())),
    };
    if let Err(e) =
        sqlx::query("insert into facts.ingestion_run (target, ok, detail) values ($1, $2, $3)")
            .bind(target)
            .bind(ok)
            .bind(detail)
            .execute(pool)
            .await
    {
        eprintln!("warning: could not record ingestion_run: {e}");
    }
}
