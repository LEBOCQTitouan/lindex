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
  totals        jsonb not null,
  breakdown     jsonb not null,      -- per-group [pour,contre,abst,nv], hémicycle order
  source_record text not null references facts.source_record(id),
  updated_at    timestamptz not null default now()
);

-- Read-model: one row = everything the /scrutin/:id page needs.
-- The Rust plane writes it; the app plane only SELECTs. This schema IS the API.
create table if not exists facts.read_scrutin (
  scrutin_id  text primary key,
  chamber     text not null,
  title       text not null,
  outcome     text not null,
  totals      jsonb not null,        -- {pour,contre,abstention,nonVotants,membersTotal,votants}
  breakdown   jsonb not null,        -- [{group,pour,contre,abst,nv}], hémicycle order
  baselines   jsonb not null,        -- medians — never null (display rule #1)
  provenance  jsonb not null,        -- {tier, source_record, url}
  updated_at  timestamptz not null default now()
);
