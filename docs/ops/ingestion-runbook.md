# Ingestion operations runbook (L1-OPS)

How L'Index ingests parliamentary data unattended: scheduling, failure alerting,
and idempotent date-range re-ingestion. Covers US-8.1 (scheduled jobs + alerting)
and US-8.4 (idempotent re-ingest). The design rationale is ADR-0002.

## What the pipeline does

- `ingest scrutin <id>` — ingest one scrutin (fetch → domain → `facts.scrutin` →
  `facts.read_scrutin`), idempotently. Records one `facts.ingestion_run` row.
- `ingest reingest <AN|SENAT> <from> <to>` — re-ingest **every scrutin already on
  record** for that chamber whose `held_on` is in `[from, to]`. Records one batch
  `facts.ingestion_run` row (`target = reingest:<CH>:<from>..<to>`). Exits non-zero
  if any scrutin failed.

Re-ingest is idempotent: each scrutin re-runs the same map/validate/project, and
the work set is derived from persisted rows, so a re-run converges to the same DB
state (proved by `crates/adapters-persistence/tests/pg_reingest.rs`).

### Known limitation — no upstream discovery

Re-ingest reprocesses scrutins **already in `facts.scrutin`**; it does not discover
*new* scrutin ids from the AN. Bringing in newly published scrutins needs an AN
listing/index source (a separate source lane). Until that lands, seed new ids with
`ingest scrutin <id>`; the scheduled `reingest` window keeps recent rows fresh
(re-parsed, baselines re-projected). See ADR-0002 §Consequences.

## Scheduling (systemd)

Units live in `ops/systemd/`:

| File | Role |
|---|---|
| `lindex-reingest@.service` | templated per-chamber oneshot job (`%i` = `AN`/`SENAT`) |
| `lindex-reingest@.timer`   | daily schedule (06:30 UTC, `Persistent`, jitter) |
| `lindex-alert@.service`    | `OnFailure=` fallback notifier |
| `lindex-reingest-window`   | wrapper computing the rolling `[today-N, today]` window |

### Install

```sh
# 1. Binary + wrapper
install -m 0755 target/release/ingest        /usr/local/bin/ingest
install -m 0755 ops/systemd/lindex-reingest-window /usr/local/bin/

# 2. Units
install -m 0644 ops/systemd/lindex-reingest@.service /etc/systemd/system/
install -m 0644 ops/systemd/lindex-reingest@.timer   /etc/systemd/system/
install -m 0644 ops/systemd/lindex-alert@.service    /etc/systemd/system/

# 3. Environment (chmod 600 — holds DATABASE_URL + webhook)
install -d -m 0750 /etc/lindex
cat > /etc/lindex/ingest.env <<'EOF'
DATABASE_URL=postgres://lindex:PASSWORD@localhost/lindex
LINDEX_ALERT_WEBHOOK=https://hooks.example.com/services/XXX   # optional
LINDEX_WINDOW_DAYS=2
EOF
chmod 600 /etc/lindex/ingest.env

# 4. Enable the AN timer (add SENAT once its source lane lands)
systemctl daemon-reload
systemctl enable --now lindex-reingest@AN.timer

# Verify
systemd-analyze verify /etc/systemd/system/lindex-reingest@.service \
                       /etc/systemd/system/lindex-reingest@.timer
systemctl list-timers 'lindex-*'
```

> The `SENAT` instance is wired but its source is not implemented yet (L1-SENAT).
> Enabling `lindex-reingest@SENAT.timer` before then produces failing runs.

## Failure alerting (US-8.1)

Two independent layers:

1. **Application-level** — the `ingest` binary fires an `Alerter` on failure. With
   `LINDEX_ALERT_WEBHOOK` set it POSTs JSON `{target, message, failedCount,
   failures[]}`; otherwise it writes a `ALERT lindex-ingest {…}` line to stderr
   (captured by journald). This carries *which* scrutins failed and why.
2. **systemd `OnFailure=`** — if the job exits non-zero, `lindex-alert@%i.service`
   posts a plain "ingest FAILED" message. Defence in depth: it fires even if the
   binary crashes before its own alert.

Inspect recent runs:

```sh
journalctl -u 'lindex-reingest@AN.service' --since '-1 day'
psql "$DATABASE_URL" -c \
  "select ran_at, target, ok, detail from facts.ingestion_run order by id desc limit 20;"
```

`ok = false` rows carry the error in `detail`. A healthy day shows one `ok = true`
batch row per enabled chamber.

## Manual re-ingest (backfill after a parser fix)

```sh
# Re-project a specific window (idempotent — safe to repeat)
DATABASE_URL=... ingest reingest AN 2026-07-01 2026-07-31
```

## Local smoke test (no Docker)

```sh
just db-up                 # or a local postgres
export DATABASE_URL=postgres://lindex:password@localhost/lindex
just migrate
ingest scrutin 8433        # seed one
ingest reingest AN 2026-07-21 2026-07-21   # re-ingest its day — idempotent
```
