#![forbid(unsafe_code)]
//! Ingest CLI — composition root. Wires concrete adapters to the application
//! ports and runs use cases. Scheduled in production by systemd timers.
//!
//! Usage:
//!   `ingest scrutin <id>`                    — ingest one scrutin (idempotent)
//!   `ingest reingest <AN|SENAT> <from> <to>` — re-ingest a day-range (US-8.4)
//!
//! Alerting: on failure the chosen [`Alerter`] fires. A webhook is used when
//! `LINDEX_ALERT_WEBHOOK` is set, otherwise a stderr log line (US-8.1).

use anyhow::{bail, Context, Result};
use chrono::NaiveDate;
use lindex_adapters_notify::{LogAlerter, WebhookAlerter};
use lindex_adapters_persistence::PgStore;
use lindex_adapters_sources::AnSource;
use lindex_application::ops::{Alerter, FailedItem, IngestionAlert, ReingestRange};
use lindex_application::usecase::IngestScrutin;
use lindex_domain::{Chamber, ScrutinId};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("scrutin") => run_scrutin(&args).await,
        Some("reingest") => run_reingest(&args).await,
        _ => {
            eprintln!(
                "usage:\n  ingest scrutin <id>\n  ingest reingest <AN|SENAT> <from> <to>  (dates YYYY-MM-DD)"
            );
            Ok(())
        }
    }
}

/// `ingest scrutin <id>` — ingest a single scrutin; records the run and, on
/// failure, alerts (best-effort).
async fn run_scrutin(args: &[String]) -> Result<()> {
    let raw_id = args.get(2).cloned().unwrap_or_default();
    if raw_id.is_empty() {
        bail!("usage: ingest scrutin <id>");
    }
    let id = ScrutinId(raw_id);

    let pool = connect().await?;
    let store = PgStore::new(pool.clone());
    let source = AnSource::new();
    let usecase = IngestScrutin {
        source: &source,
        repo: &store,
        read: &store,
    };
    let outcome = usecase.run(&id).await;
    let target = format!("scrutin:{}", id.0);
    record_run(&pool, &target, &outcome).await;
    if let Err(e) = &outcome {
        let alerter = choose_alerter();
        let alert = IngestionAlert {
            target: target.clone(),
            failed: vec![FailedItem {
                id: id.0.clone(),
                reason: e.to_string(),
            }],
            message: format!("scrutin {} failed to ingest", id.0),
        };
        let _ = alerter.alert(&alert).await;
    }
    outcome.context("ingesting scrutin")?;
    println!("ingested scrutin {}", id.0);
    Ok(())
}

/// `ingest reingest <AN|SENAT> <from> <to>` — re-ingest every persisted scrutin
/// held in `[from, to]`, idempotently (US-8.4). Records one batch audit row and,
/// on any failure, alerts once and exits non-zero so systemd's `OnFailure=` fires.
async fn run_reingest(args: &[String]) -> Result<()> {
    // Validate arguments before touching the database, so misuse fails fast.
    let chamber = parse_chamber(args.get(2).map(String::as_str).unwrap_or_default())?;
    let from = parse_date(args.get(3).map(String::as_str).unwrap_or_default(), "from")?;
    let to = parse_date(args.get(4).map(String::as_str).unwrap_or_default(), "to")?;
    if to < from {
        bail!("empty range: <to> {to} is before <from> {from}");
    }

    let pool = connect().await?;
    let store = PgStore::new(pool.clone());
    let source = AnSource::new();
    let alerter = choose_alerter();
    let reingest = ReingestRange {
        calendar: &store,
        ingest: IngestScrutin {
            source: &source,
            repo: &store,
            read: &store,
        },
        alerter: alerter.as_ref(),
    };
    let report = reingest
        .run(chamber, from, to)
        .await
        .context("re-ingesting range")?;

    // One batch audit row per scheduled invocation (US-8.1).
    let agg: Result<(), String> = if report.failures.is_empty() {
        Ok(())
    } else {
        let ids: Vec<&str> = report.failures.iter().map(|f| f.id.as_str()).collect();
        Err(format!(
            "{} of {} failed: {}",
            report.failures.len(),
            report.attempted,
            ids.join(", ")
        ))
    };
    record_run(&pool, &report.target, &agg).await;

    println!(
        "reingested {} scrutins ({} ok, {} failed) [{}]",
        report.attempted,
        report.succeeded,
        report.failures.len(),
        report.target
    );
    if !report.failures.is_empty() {
        // Alerting already happened inside ReingestRange; signal failure to the
        // scheduler after the audit row is safely written.
        std::process::exit(1);
    }
    Ok(())
}

/// Pick the alert transport from the environment (US-8.1): a webhook when
/// `LINDEX_ALERT_WEBHOOK` is set, otherwise a stderr log line.
fn choose_alerter() -> Box<dyn Alerter> {
    match std::env::var("LINDEX_ALERT_WEBHOOK") {
        Ok(url) if !url.is_empty() => Box::new(WebhookAlerter::new(url)),
        _ => Box::new(LogAlerter),
    }
}

fn parse_chamber(s: &str) -> Result<Chamber> {
    match s {
        "AN" => Ok(Chamber::AssembleeNationale),
        "SENAT" => Ok(Chamber::Senat),
        "" => bail!("usage: ingest reingest <AN|SENAT> <from> <to>"),
        other => bail!("unknown chamber {other:?}; expected AN or SENAT"),
    }
}

fn parse_date(s: &str, which: &str) -> Result<NaiveDate> {
    if s.is_empty() {
        bail!("usage: ingest reingest <AN|SENAT> <from> <to>  (missing <{which}>)");
    }
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .with_context(|| format!("bad <{which}> date {s:?}; expected YYYY-MM-DD"))
}

async fn connect() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    PgPool::connect(&database_url)
        .await
        .context("connecting to Postgres")
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
