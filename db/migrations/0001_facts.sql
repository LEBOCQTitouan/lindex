-- facts schema: parliamentary data, owned by the Rust data plane.
-- The app plane has SELECT-only access here.
create schema if not exists facts;

-- Immutable raw source records — the provenance backbone. Idempotent
-- re-ingestion replays these, so derivations are deterministic (US-8.4).
create table if not exists facts.source_record (
  id         text primary key,
  tier       text not null,          -- ProvenanceTier
  url        text,
  fetched_at timestamptz not null default now(),
  payload    jsonb not null
);

-- Unified model (normalized). Written by the ingest use cases.
create table if not exists facts.scrutin (
  id            text primary key,
  chamber       text not null,
  title         text not null,
  held_on       date not null,       -- sitting date; groups a scrutin's day-cohort baseline (ADR-0001)
  totals        jsonb not null,
  breakdown     jsonb not null,      -- per-group [pour,contre,abst,nv], hémicycle order
  source_record text not null references facts.source_record(id),
  updated_at    timestamptz not null default now()
);

-- Index the day-cohort lookup that computes baselines.
create index if not exists scrutin_chamber_day on facts.scrutin (chamber, held_on);

-- Ingestion audit trail — one row per `ingest` invocation (US-8.1 / 8.4).
create table if not exists facts.ingestion_run (
  id       bigserial primary key,
  target   text not null,            -- e.g. "scrutin:8433"
  ran_at   timestamptz not null default now(),
  ok       boolean not null,
  detail   text                      -- error message on failure
);

-- Read-model: one row = everything the /scrutin/:id page needs.
-- The Rust plane writes it; the app plane only SELECTs. This schema IS the API.
create table if not exists facts.read_scrutin (
  scrutin_id  text primary key,
  chamber     text not null,
  title       text not null,
  held_on     date not null,         -- sitting date (drives the "last updated"/context surfaces)
  outcome     text not null,         -- source-stated result, verbatim (adopte|rejete)
  totals      jsonb not null,        -- {pour,contre,abstention,nonVotants,membersTotal,votants,exprimes}
  breakdown   jsonb not null,        -- [{group,pour,contre,abstention,nonVotant}], hémicycle order
  baselines   jsonb not null,        -- {votants:{median,sampleSize},abstention:{…},method:{id,version}} — never null (rule #1)
  provenance  jsonb not null,        -- {tier,label,url,recordId,retrievedAt}
  updated_at  timestamptz not null default now()
);
