# ADR-0001: Scrutin read-model contract and day-median baselines

- **Status:** Accepted
- **Date:** 2026-08-10
- **Deciders:** L0-DATA lane

## Context

L0-DATA implements the reference vertical slice: AN scrutin → domain → Postgres
→ `facts.read_scrutin`. `facts.read_scrutin` **is the contract** with the app
plane (CLAUDE.md, roadmap §Parallelization). Its JSONB column shapes, plus the
provenance and baseline payloads, are a class-A change (read-model / data-schema)
and must be pinned before app lanes build against them.

Bearing guidelines: **P1** (every figure ships its baseline — median of
comparables), **P4** (provenance tier + source link on every datum), **P2/P3**
(no composite scores, no verdicts), **R5** (golden tests for the read-model wire
format). The AN source is the `dyn` HTML page
(`https://www.assemblee-nationale.fr/dyn/17/scrutins/<n>`); there is no public
per-scrutin JSON endpoint (verified 2026-08-10: `.json` and
`data.assemblee-nationale.fr/api/scrutin/<n>` both 404 / return HTML).

## Decision

1. **Source adapter normalizes at the boundary.** `AnSource` fetches the `dyn`
   HTML, parses it, and emits a **normalized JSON** payload
   (`application::ports::RawScrutinData`) — the source-agnostic record stored
   immutably in `facts.source_record.payload` (jsonb). AN-specific HTML parsing
   never leaves `adapters-sources`. The application maps normalized-JSON → domain
   (source-agnostic), preserving the hexagon's inward dependency direction.

2. **`held_on` date is part of the model.** `facts.scrutin` and `Scrutin` gain a
   `held_on` (calendar date, from the page's "séance du …"). It is required to
   group a scrutin's **comparables** for the baseline.

3. **Baseline = day-median.** For each scrutin the read-model carries a baseline
   for `votants` and `abstention`, each the **median over all scrutins of the
   same chamber on the same `held_on`** currently in `facts.scrutin`, with the
   sample size. Method id `scrutin-day-median` v1. Projection recomputes the
   whole day cohort on every ingest, so a later same-day scrutin updates its
   peers' baselines. No composite score is ever formed.

4. **`facts.read_scrutin` JSONB shapes (the wire contract):**
   - `totals`: `{pour,contre,abstention,nonVotants,membersTotal,votants,exprimes}`
   - `breakdown`: `[{group,pour,contre,abstention,nonVotant}]`, hémicycle L→R order
   - `baselines`: `{votants:{median,sampleSize},abstention:{median,sampleSize},method:{id,version}}`
   - `provenance`: `{tier,label,url,recordId,retrievedAt}`
   - `outcome`: the source-stated result verbatim (`adopte` | `rejete`), never computed.

## Consequences

- App lanes (L0-APP) build every scrutin view against these exact shapes and swap
  to real data with zero app changes.
- Re-ingesting a scrutin is idempotent (upsert on primary key); re-projecting the
  day cohort is deterministic given the same stored records (US-8.4).
- Matching the mockup baseline (`{votants:426, abstention:7}`) requires all five
  21 Jul 2026 scrutins ingested; with the two golden fixtures (8430, 8433) the
  day-median is `{votants:547.5, abstention:90}` — same mechanism, smaller cohort.
- New obligation: any future source adapter must emit `RawScrutinData`, and any
  read-model field change reopens this ADR.
- **Maintenance coupling:** the AN adapter's `GROUPS` table (organe id → short id
  → hémicycle rank) is reference data pinned to the 17th-legislature roster. A new
  or renamed group makes the parser fail loud (`ParseError::UnknownGroup`) rather
  than silently mis-attributing votes — an intentional trade: ingestion of the
  affected scrutins stops until the table is updated, surfaced via
  `facts.ingestion_run`.

## Alternatives considered

- **Store the raw 300 KB HTML in `source_record`.** Rejected: `payload` is jsonb
  (schema intent is a structured record), and it would force AN HTML parsing into
  the application use case — an outward dependency that breaks the hexagon. Trade
  accepted: we persist the extracted record, not the literal page bytes; the
  provenance URL still points at the live page for audit.
- **Compute the outcome from the vote math** (`pour ≥ majorité absolue`).
  Rejected: motions and special votes have different adoption rules; the page
  states the outcome authoritatively. Trade accepted: we depend on one more
  parsed string, in exchange for never authoring a verdict (P3).
- **Baseline = median over the whole corpus (no date).** Rejected: it mixes
  non-comparable sittings and needs no less machinery than day grouping. Trade
  accepted: a `held_on` column and a cohort re-projection per ingest.
