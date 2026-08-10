# L1-OPS — Unattended Ingestion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to
> implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Run AN scrutin ingestion unattended — a scheduled run records to
`facts.ingestion_run`, alerts on failure, and re-ingesting a date range is
idempotent (US-8.1, US-8.4).

**Architecture:** Build on the L0-DATA spine (`IngestScrutin` use case, `PgStore`,
`ingest` CLI). Add two application ports — `ScrutinCalendar` (which scrutins are on
record for a chamber over a day-range) and `Alerter` (where failure notifications
go) — and one use case `ReingestRange` that walks the calendar, re-runs the
idempotent single-scrutin ingest for each id, and alerts if any fail. Adapters:
`ScrutinCalendar for PgStore` (persistence) and a new `adapters-notify` crate
(`LogAlerter` default, `WebhookAlerter` when configured). The CLI gains a
`reingest <chamber> <from> <to>` subcommand recording one batch audit row; systemd
units schedule it. A date-range re-ingest derives its work set from persisted
`facts.scrutin.held_on`, **not** from upstream discovery — recorded in ADR-0002.

**Tech Stack:** Rust (hexagonal, tokio, async-trait, sqlx, reqwest, serde_json,
chrono, thiserror), systemd timers, Postgres.

## Global Constraints

- `#![forbid(unsafe_code)]` in every library crate lib.rs.
- No `unwrap`/`expect`/`panic` in library code (tests exempt). `thiserror` in libs,
  `anyhow` in the bin.
- Errors `#[non_exhaustive]`, compose with `#[from]`.
- `#![deny(missing_docs)]`-friendly: doc every public item.
- French UI text / English code + comments. No composite scores/rankings/verdicts;
  provenance + baseline preserved (this lane only re-runs the existing projection).
- Gate: `just ci` (fmt, clippy `-D warnings`, nextest, deny) must be green.
- CARGO: never bare cargo — `env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/agent
  cargo <cmd> -p <crate>`.
- Reuse workspace deps only (reqwest, serde_json, chrono already present) — no new
  external dependency requiring G5 justification.

---

### Task 1: `ScrutinCalendar` + `Alerter` ports and `ReingestRange` use case

**Files:**
- Create: `crates/application/src/ops.rs`
- Modify: `crates/application/src/lib.rs` (add `pub mod ops;`)

**Interfaces:**
- Consumes: `crate::usecase::{IngestScrutin, IngestError}`, `crate::ports::RepoError`,
  `lindex_domain::{Chamber, ScrutinId}`.
- Produces:
  - `trait ScrutinCalendar { async fn ids_in_range(&self, chamber: Chamber, from:
    NaiveDate, to: NaiveDate) -> Result<Vec<ScrutinId>, RepoError>; }`
  - `trait Alerter { async fn alert(&self, alert: &IngestionAlert) -> Result<(),
    AlertError>; }`
  - `struct IngestionAlert { pub target: String, pub failed: Vec<FailedItem>, pub
    message: String }`, `struct FailedItem { pub id: String, pub reason: String }`
  - `enum AlertError { Transport(String) }` (`#[non_exhaustive]`, thiserror)
  - `struct ReingestRange<'a> { pub calendar, pub ingest: IngestScrutin<'a>, pub
    alerter }`; `async fn run(&self, chamber, from, to) -> Result<ReingestReport,
    RepoError>`
  - `struct ReingestReport { pub target: String, pub attempted: usize, pub
    succeeded: usize, pub failures: Vec<FailedItem> }`

- [ ] **Step 1: Write failing tests** in `crates/application/src/ops.rs` `#[cfg(test)]`.
  Fakes: `FakeCalendar { ids: Vec<ScrutinId> }`; reuse a `FakeSource`/`FakeRepo`/
  `FakeRead` mirroring `usecase.rs` tests but where `FakeSource` can be told to fail
  for a given id (`fail_ids: HashSet<String>` → returns `SourceError::Unavailable`);
  `RecordingAlerter { alerts: Mutex<Vec<IngestionAlert-clone>> }`. Tests:
  - `reingest_runs_each_id_and_reports_success`: calendar `[8433, 8430]`, no
    failures → `report.attempted == 2 && report.succeeded == 2 &&
    report.failures.is_empty()`; alerter NOT called (`alerts.len() == 0`).
  - `reingest_alerts_on_failure`: source fails for `8430` → `report.failures.len()
    == 1`, `report.failures[0].id == "8430"`, alerter called once, its
    `IngestionAlert.failed[0].id == "8430"`, `target` == the report target.
  - `reingest_is_idempotent`: run the same range twice against shared fake stores;
    repo has one row per id and read has one view per id after both runs; the second
    report equals the first (attempted/succeeded).
  - `reingest_empty_range_does_not_alert`: calendar `[]` → attempted 0, alerter not
    called.
- [ ] **Step 2: Run** `env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/agent cargo
  nextest run -p lindex-application ops` → FAIL (module/types absent).
- [ ] **Step 3: Implement `ops.rs`.** Ports as above (`async_trait`, doc comments).
  `ReingestRange::run`: `let ids = self.calendar.ids_in_range(chamber, from, to)
  .await?;` build `target = format!("reingest:{}:{from}..{to}", chamber_code)` where
  `chamber_code` matches persistence (`AN`/`SENAT`); loop ids, `match
  self.ingest.run(&id).await { Ok(()) => succeeded += 1, Err(e) =>
  failures.push(FailedItem { id: id.0.clone(), reason: e.to_string() }) }`. If
  `!failures.is_empty()`, best-effort `let _ = self.alerter.alert(&IngestionAlert {
  target: target.clone(), failed: failures.clone(), message: format!("{} of {}
  scrutins failed to re-ingest", failures.len(), attempted) }).await;` (a failed
  alert must not mask the report — log via the alerter adapter, not here). Return
  `ReingestReport`. Add a `chamber_code(Chamber) -> &'static str` local helper
  (`AN`/`SENAT`).
- [ ] **Step 4: Modify `lib.rs`** add `pub mod ops;` after `pub mod usecase;`.
- [ ] **Step 5: Run** the same nextest command → PASS. Then `cargo clippy -p
  lindex-application --all-targets -- -D warnings` clean.
- [ ] **Step 6: Commit** `feat(application): add ReingestRange use case + calendar/alerter ports`.

---

### Task 2: `ScrutinCalendar for PgStore` (persistence adapter)

**Files:**
- Create: `crates/adapters-persistence/src/calendar.rs`
- Modify: `crates/adapters-persistence/src/lib.rs` (add `mod calendar;`)
- Test: `crates/adapters-persistence/tests/pg_reingest.rs` (gated on `DATABASE_URL`)

**Interfaces:**
- Consumes: `lindex_application::ops::ScrutinCalendar`, `PgStore` (this crate),
  `lindex_domain::{Chamber, ScrutinId}`.
- Produces: `impl ScrutinCalendar for PgStore`.

- [ ] **Step 1: Write failing integration test** `tests/pg_reingest.rs`. Gate: early
  return if `DATABASE_URL` unset (mirror `pg_integration.rs`). `migrate(pool)` via the
  same `include_str!` of both migration files. Ingest 8433 & 8430 (same day
  `2026-07-21`, AN) using a `StubSource` (copy the pattern). Then:
  - `ids_in_range(AN, 2026-07-21, 2026-07-21)` returns a set containing both ids.
  - `ids_in_range(AN, 2026-07-22, 2026-07-22)` returns empty.
  - Build `ReingestRange { calendar: &store, ingest: IngestScrutin{...}, alerter:
    &NoopAlerter }`, `run(AN, 2026-07-21, 2026-07-21)` → `report.attempted == 2`,
    `report.failures.is_empty()`; assert exactly one `facts.scrutin` row per id after
    the reingest (idempotent). `NoopAlerter` is a local test double returning `Ok`.
- [ ] **Step 2: Run** `DATABASE_URL` unset → PASS (early-return) to confirm the test
  compiles and is CI-safe: `env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/agent cargo
  nextest run -p lindex-adapters-persistence pg_reingest`. (Compiles → the trait must
  exist, so this drives Step 3.) Expected first: FAIL to compile (`ScrutinCalendar`
  not impl for `PgStore`).
- [ ] **Step 3: Implement `calendar.rs`.**
  ```rust
  use async_trait::async_trait;
  use chrono::NaiveDate;
  use lindex_application::ops::ScrutinCalendar;
  use lindex_application::ports::RepoError;
  use lindex_domain::{Chamber, ScrutinId};
  use sqlx::Row;
  use crate::{chamber_code, PgStore, backend};

  #[async_trait]
  impl ScrutinCalendar for PgStore {
      async fn ids_in_range(
          &self, chamber: Chamber, from: NaiveDate, to: NaiveDate,
      ) -> Result<Vec<ScrutinId>, RepoError> {
          let rows = sqlx::query(
              "select id from facts.scrutin \
               where chamber = $1 and held_on between $2 and $3 order by id")
              .bind(chamber_code(chamber)).bind(from).bind(to)
              .fetch_all(&self.pool).await.map_err(backend)?;
          rows.iter().map(|r| Ok(ScrutinId(r.try_get::<String,_>("id").map_err(backend)?)))
              .collect()
      }
  }
  ```
  Make `chamber_code` and `backend` visible to the module: they are private fns in
  `lib.rs`; change to `pub(crate)` (in-crate visibility only — additive, no external
  surface). Add `mod calendar;` to `lib.rs`.
- [ ] **Step 4: Run** the nextest command again → PASS (compiles + early-returns with
  no DB). If Postgres is available locally (`just db-up`), run with `DATABASE_URL` set
  and confirm real assertions pass.
- [ ] **Step 5:** `cargo clippy -p lindex-adapters-persistence --all-targets -- -D
  warnings` clean.
- [ ] **Step 6: Commit** `feat(adapters-persistence): ScrutinCalendar day-range id lookup`.

---

### Task 3: `adapters-notify` crate — `LogAlerter` + `WebhookAlerter`

**Files:**
- Create: `crates/adapters-notify/Cargo.toml`
- Create: `crates/adapters-notify/src/lib.rs`
- Modify: `Cargo.toml` (workspace `members` add `crates/adapters-notify`)

**Interfaces:**
- Consumes: `lindex_application::ops::{Alerter, IngestionAlert, AlertError}`.
- Produces: `struct LogAlerter;` (writes a structured line to stderr, always Ok);
  `struct WebhookAlerter { client: reqwest::Client, url: String }` with `fn
  new(url: impl Into<String>) -> Self`; a pure `fn alert_payload(alert:
  &IngestionAlert) -> serde_json::Value` (unit-testable without HTTP).

- [ ] **Step 1: Write failing tests** in `src/lib.rs` `#[cfg(test)]`:
  - `payload_carries_target_and_failures`: `alert_payload(&IngestionAlert{ target:
    "reingest:AN:..".into(), failed: vec![FailedItem{id:"8430".into(),
    reason:"boom".into()}], message:"1 of 2 ...".into() })` → JSON has
    `["target"] == "reingest:AN:.."`, `["failedCount"] == 1`,
    `["failures"][0]["id"] == "8430"`, and a `["message"]`.
  - `log_alerter_reports_success`: `LogAlerter.alert(&a).await` is `Ok(())`.
- [ ] **Step 2: Run** `env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/agent cargo
  nextest run -p lindex-adapters-notify` → FAIL (crate absent).
- [ ] **Step 3: Implement** `Cargo.toml` (deps: `lindex-application` path,
  `async-trait`, `reqwest`, `serde_json`, `thiserror` workspace; `[lints] workspace =
  true`; `[dev-dependencies] tokio`); `lib.rs` with `#![forbid(unsafe_code)]`, module
  doc, both alerters, `alert_payload`. `LogAlerter::alert` writes `eprintln!` of the
  payload and returns `Ok`. `WebhookAlerter::alert` POSTs `alert_payload` JSON, maps
  transport errors to `AlertError::Transport`. Add crate to workspace `members`.
- [ ] **Step 4: Run** nextest → PASS. Clippy clean for the crate.
- [ ] **Step 5: Commit** `feat(adapters-notify): log + webhook failure alerters`.

---

### Task 4: `ingest reingest` CLI subcommand + alert wiring

**Files:**
- Modify: `bin/ingest/src/main.rs`
- Modify: `bin/ingest/Cargo.toml` (add `lindex-adapters-notify` path dep)

**Interfaces:**
- Consumes: `ReingestRange`, `ScrutinCalendar` (via `PgStore`), `LogAlerter`/
  `WebhookAlerter`, `IngestScrutin`, `AnSource`, `PgStore`.
- Produces: CLI `ingest reingest <AN|SENAT> <from> <to>` (dates `YYYY-MM-DD`).

- [ ] **Step 1: Extend `main.rs`.** Add a `reingest` arm to the `match`. Parse
  `chamber` (`"AN"`|`"SENAT"` → `Chamber`, else `bail!`), `from`/`to` via
  `NaiveDate::parse_from_str(..,"%Y-%m-%d").context(..)?` (`bail!` usage string when
  missing). Build `store`, `source = AnSource::new()`, `ingest = IngestScrutin {
  source: &source, repo: &store, read: &store }`. Select alerter: `let webhook =
  std::env::var("LINDEX_ALERT_WEBHOOK").ok();` → `WebhookAlerter::new(url)` if
  `Some`, else `LogAlerter`; bind both behind `&dyn Alerter` (build one or the other,
  reference it). `let rr = ReingestRange { calendar: &store, ingest, alerter };`
  `let report = rr.run(chamber, from, to).await.context("reingesting range")?;`
  Build an aggregate `Result<(), String>` for the audit: `Ok(())` if
  `report.failures.is_empty()` else `Err(format!("{}/{} failed: {}", ...ids...))`;
  call the existing `record_run(&pool, &report.target, &agg).await;`. Print a summary
  line (`println!("reingested {} scrutins ({} ok, {} failed)", attempted, succeeded,
  failures.len())`). If any failure, `std::process::exit(1)` **after** recording, so
  systemd's `OnFailure=` fires. (Alerting already happened inside `ReingestRange`.)
- [ ] **Step 2:** In the existing `scrutin` arm, after `record_run`, on `Err` also
  fire a best-effort alert so a single scheduled scrutin ingest alerts too: reuse the
  chosen alerter — factor alerter selection into a small local `fn choose_alerter()`
  returning an owned enum or box, OR inline in both arms. Keep it best-effort (`let _
  = ...`). Update the usage string (`_ =>`) to list `reingest`.
- [ ] **Step 3: Add dep** to `bin/ingest/Cargo.toml`:
  `lindex-adapters-notify = { path = "../../crates/adapters-notify" }`.
- [ ] **Step 4: Build** `env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/agent cargo
  build -p ingest` → OK. `cargo run -p ingest -- reingest` (no args) prints the usage
  and exits non-zero without needing a DB (validate arg parsing happens before the DB
  connect, or accept the DB-connect error — arg validation must precede connect).
  Clippy clean for `-p ingest`.
- [ ] **Step 5: Commit** `feat(ingest): reingest date-range subcommand with failure alerting`.

---

### Task 5: systemd scheduler units + operations runbook

**Files:**
- Create: `ops/systemd/lindex-reingest@.service`
- Create: `ops/systemd/lindex-reingest@.timer`
- Create: `ops/systemd/lindex-alert@.service`
- Create: `ops/systemd/lindex-reingest-window` (executable wrapper: computes the
  rolling window and calls the binary)
- Create: `docs/ops/ingestion-runbook.md`

**Interfaces:**
- Consumes: the `ingest reingest` CLI, `LINDEX_ALERT_WEBHOOK` env.
- Produces: a templated timer per chamber (`%i` = `AN`/`SENAT`).

- [ ] **Step 1: Write the units.** `lindex-reingest@.service`: `Type=oneshot`,
  `EnvironmentFile=/etc/lindex/ingest.env` (holds `DATABASE_URL`,
  `LINDEX_ALERT_WEBHOOK`, `LINDEX_WINDOW_DAYS`), `ExecStart=/usr/local/bin/
  lindex-reingest-window %i`, `OnFailure=lindex-alert@%i.service`, hardening
  (`DynamicUser=` off since it needs env file; `NoNewPrivileges=true`,
  `ProtectSystem=strict`, `ProtectHome=true`). `lindex-reingest@.timer`:
  `OnCalendar=*-*-* 06:30:00`, `Persistent=true`, `RandomizedDelaySec=300`,
  `[Install] WantedBy=timers.target`. `lindex-alert@.service`: `Type=oneshot`,
  `ExecStart=/usr/local/bin/ingest reingest %i "$FROM" "$TO"`… no — the alert unit is
  a fallback notifier: `ExecStart=/bin/sh -c 'echo "lindex ingest FAILED for %i" |
  ...'` posting to `$LINDEX_ALERT_WEBHOOK` via `curl` (belt-and-braces alongside the
  app-level alert). Keep it a documented example.
- [ ] **Step 2: Write the wrapper** `lindex-reingest-window` (POSIX sh):
  ```sh
  #!/bin/sh
  set -eu
  CHAMBER="${1:?usage: lindex-reingest-window <AN|SENAT>}"
  DAYS="${LINDEX_WINDOW_DAYS:-2}"
  TO="$(date -u +%Y-%m-%d)"
  FROM="$(date -u -d "-${DAYS} days" +%Y-%m-%d 2>/dev/null || date -u -v-"${DAYS}"d +%Y-%m-%d)"
  exec /usr/local/bin/ingest reingest "$CHAMBER" "$FROM" "$TO"
  ```
  `chmod +x`.
- [ ] **Step 3: Write `docs/ops/ingestion-runbook.md`** — install steps (copy binary,
  `/etc/lindex/ingest.env`, `systemctl enable --now lindex-reingest@AN.timer`), what
  a run records (`facts.ingestion_run` batch row), how alerting works (app-level
  `Alerter` + systemd `OnFailure`), the idempotence guarantee, and the **known
  limitation**: re-ingest derives its work set from already-persisted scrutins
  (`held_on`); discovering *new* scrutin ids needs an AN listing source (follow-up
  lane) — cite ADR-0002. Verify: `systemd-analyze verify ops/systemd/*.service
  ops/systemd/*.timer` if systemd present (note as manual check; CI has no systemd).
- [ ] **Step 4: Commit** `feat(ops): systemd reingest timers + ingestion runbook`.

---

### Task 6: ADR-0002, CHANGELOG, roadmap board, full gate

**Files:**
- Create: `docs/decisions/0002-unattended-ingestion-ops.md`
- Modify: `CHANGELOG.md` (`[Unreleased] Added`)
- Modify: `docs/roadmap.md` (L1-OPS row → status)

- [ ] **Step 1: Write ADR-0002** (template). Decision: (1) date-range re-ingest
  discovers its work set from persisted `facts.scrutin.held_on`, not upstream, so a
  re-run converges (US-8.4); (2) failure alerting is an application `Alerter` port
  with log/webhook adapters, fired inside `ReingestRange`, reinforced by systemd
  `OnFailure`; (3) scheduling via templated systemd timers running a rolling-window
  reingest. Consequences: new-object discovery is out of scope (needs a listing
  source) — a named follow-up. Alternatives: upstream date→id discovery (rejected:
  belongs to a source lane, would edit `adapters-sources`); per-scrutin audit rows
  (rejected: one batch row per scheduled invocation is the operator's unit).
- [ ] **Step 2: CHANGELOG** add under `### Added`: the reingest command, calendar +
  alerter ports, `adapters-notify`, systemd units + runbook (US-8.1, US-8.4);
  reference ADR-0002.
- [ ] **Step 3: roadmap** flip L1-OPS row `🟡` → `🟣 in-review` (then `✅` at merge).
- [ ] **Step 4: Full gate** `just ci` (via the CARGO env wrapper for each cargo step,
  or run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D
  warnings`, `cargo nextest run --profile ci --workspace --all-targets`, `cargo deny
  --all-features check`, each with `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/agent`).
  All green.
- [ ] **Step 5: Commit** `docs: ADR-0002 unattended-ingestion ops + changelog + board`.
```
```

## Self-Review notes
- Spec coverage: US-8.1 scheduler (Task 5) + alerting (Tasks 1,3,4); US-8.4 idempotent
  date-range reingest (Tasks 1,2,4). "records to ingestion_run" (Task 4 audit row).
- Type consistency: `IngestionAlert{target,failed,message}`, `FailedItem{id,reason}`,
  `ReingestReport{target,attempted,succeeded,failures}` used identically across tasks.
- No new external deps (reqwest/serde_json/chrono/thiserror already in workspace).
