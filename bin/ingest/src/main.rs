#![forbid(unsafe_code)]
//! Ingest CLI — composition root. Wires concrete adapters to the application
//! ports and runs use cases. Scheduled in production by systemd timers.
//!
//! Usage: `cargo run -p ingest -- scrutin 8433`  (idempotent, re-runnable)

use anyhow::Result;
use lindex_adapters_persistence::PgStore;
use lindex_adapters_sources::AnSource;
use lindex_application::usecase::IngestScrutin;
use lindex_domain::ScrutinId;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("scrutin") => {
            let id = ScrutinId(args.get(2).cloned().unwrap_or_default());

            // --- composition: build adapters, satisfy ports ---
            let database_url = std::env::var("DATABASE_URL")?;
            let pool = sqlx::PgPool::connect(&database_url).await?;
            let store = PgStore::new(pool);
            let source = AnSource::new();

            let usecase = IngestScrutin {
                source: &source,
                repo: &store,
                read: &store,
            };
            usecase.run(&id).await?;
            println!("ingested scrutin {}", id.0);
        }
        _ => eprintln!("usage: ingest scrutin <id>"),
    }
    Ok(())
}
