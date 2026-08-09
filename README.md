# L'Index

Non-partisan civic platform to *understand* the French Parliament (AN + Sénat):
extraction with sources, never editorial verdicts.

## Architecture — two planes, one contract

```
     DATA PLANE (Rust · hexagonal)                    APP PLANE (TypeScript · Next.js)
 sources → domain → persistence → Postgres  ◀── read ── Next.js (SSR) → /scrutin/:id …
   ingest CLI (systemd timers)              read-models        islands: hémicycle, filters
```

- **Data plane (`crates/`, `bin/`)** — Rust hexagon. Ingests AN/Sénat/Légifrance
  into a unified model and projects **read-models**. Dependencies point inward:
  `domain ← application ← adapters ← bin`. The domain encodes the platform's
  invariants as types (`Sourced<T>`, `Indicator` with a required baseline).
- **App plane (`app/`)** — Next.js (App Router). SSR-reads the read-models; the
  mockup hémicycle is ported to a React island.
- **Boundary** — one Postgres, two schemas: `facts` (Rust-owned, read-only to the
  app) and `app` (app-owned: accounts, follows, editorial review). The
  `facts.read_*` tables are the contract.

`mockups/` holds the approved design; `docs/` the product decisions.

## Run

```sh
just db-up                       # Postgres 16 via docker compose
DATABASE_URL=postgres://lindex:password@localhost:5432/lindex just migrate
just check                       # cargo check --workspace
just app-install && just app-dev # Next.js at http://localhost:3000
```

Skeleton status: structure, invariant types, and ports are in place; the ingest
use case and adapters are `todo!()` — the first real slice fills them in.
