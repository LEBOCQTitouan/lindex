# ADR-0002: Unattended-ingestion operations (scheduling, alerting, date-range re-ingest)

- **Status:** Accepted
- **Date:** 2026-08-10
- **Deciders:** L1-OPS lane

## Context

L1-OPS makes ingestion run unattended (US-8.1 scheduled jobs + failure alerting;
US-8.4 idempotent date-range re-ingest). It builds on the L0-DATA spine: the
idempotent `IngestScrutin` use case, `PgStore`, `facts.ingestion_run`, and the
`ingest` CLI. The AN source (ADR-0001) fetches **by scrutin id** from the `dyn`
analysis page; there is no per-day listing endpoint in scope.

Bearing guidelines: **R2/R3** (thiserror in libs, anyhow in bins, no panics),
hexagonal dependency direction (ports in `application`, adapters outward), and the
admin persona's need to "trust it enough not to babysit it daily."

The open question: a "date range" has no upstream index, so what is the work set
of `reingest <from> <to>`?

## Decision

1. **Re-ingest derives its work set from persisted rows.** `ReingestRange` asks a
   new `application::ops::ScrutinCalendar` port for the ids of scrutins already in
   `facts.scrutin` whose `held_on` is in `[from, to]`, then replays the idempotent
   single-scrutin `IngestScrutin` for each. Because the set comes from persisted
   state and each step is idempotent, a re-run converges (US-8.4). No new DB
   schema — the existing tables suffice.

2. **Failure alerting is an application port.** `application::ops::Alerter` is
   fired by `ReingestRange` (and the single-scrutin CLI path) when a run has
   failures. Adapters live in a new `adapters-notify` crate: `LogAlerter` (stderr,
   journald-visible, zero-config default) and `WebhookAlerter` (POSTs JSON). The
   CLI selects the transport from `LINDEX_ALERT_WEBHOOK`. systemd `OnFailure=`
   adds a second, independent notifier (defence in depth).

3. **Scheduling is systemd timers.** A templated `lindex-reingest@<chamber>` timer
   runs a rolling-window re-ingest daily; a wrapper computes `[today-N, today]`.
   One batch `facts.ingestion_run` row is recorded per invocation, and a failed
   run exits non-zero so `OnFailure=` fires.

## Consequences

- Re-ingest keeps existing scrutins fresh (re-parsed, baselines re-projected) and
  is safe to repeat for backfills after a parser fix.
- **New-scrutin discovery is out of scope.** Pulling in newly published scrutins
  needs an AN listing/index source; that is a separate source lane. Until it
  lands, new ids are seeded with `ingest scrutin <id>`. The runbook states this.
- Two new `application` ports (`ScrutinCalendar`, `Alerter`) and one new adapter
  crate; the read-model contract and `facts` schema are unchanged (not class-A on
  those axes — this ADR records the operational-interface decision).
- The CLI gains a `reingest` verb — a stable operator interface documented in the
  runbook.

## Alternatives considered

- **Discover ids upstream by date (scrape a listing page).** Rejected here: it
  belongs to a source adapter (`adapters-sources`), would broaden L1-OPS beyond
  operations, and couples scheduling to an unproven listing endpoint. Deferred to
  a source lane; the port boundary (`ScrutinCalendar`) leaves room to add a
  source-backed calendar later without changing `ReingestRange`.
- **One `ingestion_run` row per scrutin in a batch.** Rejected: the operator's
  unit is the scheduled invocation, not each scrutin. A batch row plus the
  per-scrutin detail in the alert keeps the audit trail readable; per-id failures
  are still named in `detail` and the alert payload.
- **systemd-only alerting (`OnFailure=` alone).** Rejected as the primary channel:
  it cannot say *which* scrutins failed or why. Kept as a fallback layer.
- **Alerting inside the bin only (no port).** Rejected: it would make the
  "alert on failure" rule untestable at the use-case level. The port lets
  `ReingestRange` unit-test the alert decision with a recording fake.
