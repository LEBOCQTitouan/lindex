-- app schema: user & editorial state, owned by the Next.js app plane.
-- The Rust plane never touches it.
create schema if not exists app;

create table if not exists app.account (
  id         uuid primary key default gen_random_uuid(),
  email      text unique not null,
  created_at timestamptz not null default now()
);

-- Follow a dossier for alerts (US-2.4 / 6.2).
create table if not exists app.follow (
  account_id uuid not null references app.account(id) on delete cascade,
  dossier_id text not null,
  created_at timestamptz not null default now(),
  primary key (account_id, dossier_id)
);

-- Human review of coherence flags before publication (US-3.9 / 5.4 / 5.5).
create table if not exists app.review (
  id          uuid primary key default gen_random_uuid(),
  subject_ref text not null,          -- e.g. "coherence:scrutin:8433"
  status      text not null default 'pending',   -- pending | approved | rejected
  reviewer    text,
  reviewed_at timestamptz
);
