# L0-DATA — AN scrutin ingest spine — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans / test-driven-development. Steps use checkbox syntax.

**Goal:** `just ingest scrutin 8433` fetches the AN `dyn` page → validated
`Sourced<Scrutin>` → `facts.scrutin` → `facts.read_scrutin` (provenance +
day-median baselines); re-running is idempotent.

**Architecture:** Hexagon. `adapters-sources` parses AN HTML → normalized
`RawScrutinData` (JSON). `application` maps normalized→domain, validates in the
domain, projects the read-model with day-median baselines. `adapters-persistence`
does the SQL. Composition root = `bin/ingest`. See ADR-0001.

**Tech Stack:** Rust (tokio, sqlx/pg, reqwest, regex, serde/serde_json,
thiserror, chrono), cargo-nextest.

## Global Constraints
- `#![forbid(unsafe_code)]`; no `unwrap`/`expect`/`panic` in library code (tests exempt).
- `thiserror` in libs, `anyhow` in bins. Errors `#[non_exhaustive]`, `#[from]` compose.
- French app text / English code. Baseline on every figure; provenance on every datum.
- New deps justified: `regex` (robust AN HTML parse), `serde` derive (normalized payload + read-model JSON).
- CI has **no Postgres** → DB-backed tests skip when `DATABASE_URL` is unset.
- Golden fixtures: real AN HTML for 8430 & 8433 under `crates/adapters-sources/tests/fixtures/`.

---

### Task 1: Domain — `held_on` + median baseline
**Files:** Modify `crates/domain/src/scrutin.rs`, `crates/domain/src/indicator.rs`, `crates/domain/src/lib.rs`.
**Produces:** `Scrutin.held_on: chrono::NaiveDate` (new `try_new` param, after `title`); `Baseline::from_samples(&[f64]) -> Option<Baseline>` (None on empty; median of sorted; sample_size = len).
- [ ] Test: `try_new` with reconciling breakdown + a date → Ok, `held_on` preserved.
- [ ] Test: `Baseline::from_samples(&[537.0,558.0])` → median 547.5, sample_size 2; `&[7.0]` → 7.0/1; `&[]` → None.
- [ ] Implement; run domain tests green; commit.

### Task 2: application ports — normalized payload + cohort port + serde
**Files:** Modify `crates/application/src/ports.rs`, `crates/application/Cargo.toml` (add serde, serde_json).
**Produces:**
- `RawGroup { group:String, pour:u32, contre:u32, abstention:u32, non_votant:u32 }` (Serialize/Deserialize, camelCase `nonVotant`).
- `RawTotals { pour,contre,abstention,non_votants,members_total,votants,exprimes:u32 }` (camelCase).
- `RawScrutinData { scrutin_id, chamber, title, held_on:String(ISO), outcome:String, totals:RawTotals, groups:Vec<RawGroup> }` — the JSON in `RawScrutin.payload` and `source_record`.
- `ScrutinRepository::cohort_totals(&self, chamber:&Chamber, held_on:NaiveDate) -> Result<Vec<VoteTotals>, RepoError>` (same-chamber same-day totals, incl. the just-upserted row).
- `ReadModelStore` unchanged (`put_scrutin_view`).
- [ ] Add serde structs + trait method (adapters will implement). Compiles; commit.

### Task 3: application usecase — `IngestScrutin::run` + projection
**Files:** Modify `crates/application/src/usecase.rs`; tests in same file (`#[cfg(test)]`) with in-memory fakes.
**Consumes:** Task 1 & 2. **Produces:** full `run`:
1. `raw = source.fetch_scrutin(id)?` → `Sourced<RawScrutin>`.
2. parse `raw.value().payload` as `RawScrutinData` (map serde err → `IngestError::Mapping`).
3. build `VoteTotals` + `Vec<GroupTally>` + parse `held_on`; `Scrutin::try_new(...)` (map `DomainError` → `IngestError::Domain`).
4. `scrutin = Sourced::new(scrutin, raw.provenance().clone())`; `repo.upsert(&scrutin)?`.
5. `cohort = repo.cohort_totals(chamber, held_on)?`; `votants_baseline = Baseline::from_samples(&cohort.map(votants))`, `abstention_baseline` similarly (Option → `IngestError::EmptyCohort` guard, but cohort always ≥1 since it includes the upserted row).
6. assemble `ScrutinView` (serde_json for `totals/breakdown/baselines/provenance` strings, outcome verbatim); `read.put_scrutin_view(&view)?`.
- [ ] Test: fakes (`FakeSource` returns fixture-derived payload; `FakeRepo` HashMap; `FakeRead` HashMap). Run twice → one scrutin row, identical `ScrutinView` (idempotent). Assert baselines = day-median over ingested cohort.
- [ ] Test: golden `ScrutinView` JSON for 8433 (breakdown hémicycle order, totals, provenance tier `ActeAuthentique`, baseline present).
- [ ] Add `IngestError` variants (`Mapping`, `Domain`) `#[non_exhaustive]`. Implement; green; commit.

### Task 4: adapters-sources — AN HTML parse + fetch
**Files:** Modify `crates/adapters-sources/src/lib.rs`; add `crates/adapters-sources/src/an_parse.rs` (private `parse_scrutin_html(html:&str, id:&ScrutinId) -> Result<RawScrutinData, ParseError>`); `crates/adapters-sources/Cargo.toml` (add regex, serde_json, chrono; dev: none); fixtures `tests/fixtures/scrutin_8433.html`, `scrutin_8430.html`; test `tests/golden_parse.rs`.
**Parsing rules (verified against real pages):**
- totals: `Nombre de votants : <b>N</b>`, `Nombre de suffrages exprimés : <b>N</b>`, `Pour l'adoption : <b>N</b>`, `Contre : <b>N</b>`, `Abstention : <b>N</b>`.
- outcome: bold sentence `L'Assemblée nationale a adopté` → `adopte`; contains `n'a pas adopté`/`a rejeté` → `rejete`.
- date: `séance du … <D> <month-fr> <YYYY>` → `NaiveDate`.
- groups: for each `data-organe-id="PO…"` block whose `FocusableList` has tally spans (`_small">Label : N`), read Pour/Contre/Abstention/Non votant; map organe id → short id via `ORGANE_MAP`; drop the empty legend blocks; order by `HEMICYCLE_RANK`.
- `members_total = 577` (AN); `non_votants = Σ group non_votant`; `votants = pour+contre+abstention`.
- `ORGANE_MAP`/`HEMICYCLE`: PO845413=LFI(0) PO845514=GDR(1) PO845439=ECOS(2) PO845419=SOC(3) PO845485=LIOT(4) PO845454=DEM(5) PO845407=EPR(6) PO845470=HOR(7) PO845425=DR(8) PO872880=UDR(9) PO845401=RN(10) PO840056=NI(11).
- [ ] Test (`golden_parse.rs`): parse `scrutin_8433.html` → totals {pour351,contre179,abst7,nv2,votants537,exprimes530,members577}, outcome `adopte`, held_on 2026-07-21, 12 groups hémicycle order, `LFI [0,71,0,0]`, `EPR [84,0,0,1]`, `NI [7,0,1,0]`. Same for 8430 (`LFI [0,0,69,0]`, `GDR [5,7,5,0]`, outcome `adopte`).
- [ ] `fetch_scrutin`: `reqwest::get({base_url}/scrutins/{id})` → text → `parse_scrutin_html` → serialize `RawScrutinData` to JSON → `Sourced::new(RawScrutin{payload}, Provenance{ ActeAuthentique, label "Scrutin n° {id} — AN", url, record_id "an-scrutin-{id}", retrieved_at now })`. Map errors → `SourceError`.
- [ ] `#[ignore]` live-network test hitting the real 8433 URL (documented; not in CI). Green offline; commit.

### Task 5: adapters-persistence — PgStore SQL
**Files:** Modify `crates/adapters-persistence/src/lib.rs`, `Cargo.toml` (add serde_json, chrono); test `tests/pg_integration.rs` (gated on `DATABASE_URL`).
**SQL:**
- `upsert`: `INSERT INTO facts.source_record ... ON CONFLICT (id) DO UPDATE`; then `INSERT INTO facts.scrutin (id,chamber,title,held_on,totals,breakdown,source_record) VALUES (...,$::jsonb,...) ON CONFLICT (id) DO UPDATE SET ...`. `totals`/`breakdown` serialized from domain via serde_json. (Requires `source_record` payload — pass it through `Sourced<Scrutin>`? No: adapter has domain only. Store a minimal source_record built from provenance + a re-serialized domain snapshot.) See note.
- `by_id`: `SELECT ... FROM facts.scrutin WHERE id=$1`; rebuild `Scrutin` via `try_new`.
- `cohort_totals`: `SELECT totals FROM facts.scrutin WHERE chamber=$1 AND held_on=$2`; deserialize each to `VoteTotals`.
- `put_scrutin_view`: `INSERT INTO facts.read_scrutin (...) ON CONFLICT (scrutin_id) DO UPDATE SET ...` with the 4 JSONB strings cast `::jsonb`.
- [ ] Gated integration test: migrate, ingest 8433 twice → 1 row; `read_scrutin` JSON well-formed; `by_id` round-trips. Skips cleanly with no DB. Compiles; commit.

### Task 6: migration + CLI wiring + docs
**Files:** Modify `db/migrations/0001_facts.sql` (add `held_on date not null` to `facts.scrutin`; add `held_on date not null` to `facts.read_scrutin`); `bin/ingest/src/main.rs` (already wired — verify); `CHANGELOG.md`; `docs/roadmap.md` board row.
- [ ] Add `held_on` columns. `ingestion_runs` table (WI 0.5): `create table facts.ingestion_run (id bigserial pk, target text, ran_at timestamptz default now(), ok boolean, detail text)` + CLI records a row per run (best-effort).
- [ ] `just ci` green (fmt, clippy -D, nextest, doc, deny). Update CHANGELOG `[Unreleased]`; flip roadmap row to 🟣/✅. Commit.

## Self-review notes
- **source_record payload (port addition):** add `ScrutinRepository::put_source_record(&self, raw:&Sourced<RawScrutin>) -> Result<(),RepoError>`. The use case persists the raw record **first** (id = provenance `record_id`, payload = `raw.payload` = the normalized `RawScrutinData` JSON, tier/url from provenance, fetched_at = `retrieved_at`), then `upsert(&scrutin)` whose `facts.scrutin.source_record` = the same `record_id` (FK satisfied). This stores the actual normalized source record per ADR-0001, no domain re-serialization hack.
- **Baseline never optional in the view:** cohort always contains ≥1 (the upserted row), so `from_samples` is `Some`; still guard with `IngestError::EmptyCohort` for totality (no panic).
- **outcome verbatim** — never computed (P3). **breakdown hémicycle order** — from HEMICYCLE ranks (P5 symmetry, same visual language).
