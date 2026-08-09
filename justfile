# L'Index — task runner across the two planes.
set shell := ["bash", "-uc"]

default:
    @just --list

# --- Rust data plane ---------------------------------------------------------
check:
    cargo check --workspace
test:
    cargo test --workspace
fmt:
    cargo fmt --all
ingest *ARGS:
    cargo run -p ingest -- {{ARGS}}

# --- Next.js app plane -------------------------------------------------------
app-install:
    cd app && pnpm install
app-dev:
    cd app && pnpm dev
app-build:
    cd app && pnpm build

# --- Database ----------------------------------------------------------------
db-up:
    docker compose up -d postgres
db-down:
    docker compose down
migrate:
    for f in db/migrations/*.sql; do psql "$DATABASE_URL" -f "$f"; done
