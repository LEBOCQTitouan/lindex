# L'Index — canonical verbs shared by humans, lefthook, and CI ("local == CI").
# Each Rust recipe carries ONLY the cargo command; the CALLER supplies the
# environment (CARGO_TARGET_DIR, CARGO_INCREMENTAL). Do NOT bake a target dir here.
set shell := ["bash", "-uc"]

default:
    @just --list

# --- Rust data plane (identical to the CI jobs) -----------------------------
fmt:
    cargo fmt --all -- --check
lint:
    cargo clippy --workspace --all-targets -- -D warnings
test *args:
    cargo nextest run --profile ci --workspace --all-targets {{args}}
deny:
    cargo deny --all-features check
doc:
    cargo test --workspace --doc

# Full local gate — everything a PR must pass.
ci: fmt lint test deny

# Auto-fix formatting + clippy (the write counterpart to fmt/lint).
fix:
    cargo fmt --all
    cargo clippy --workspace --all-targets --fix --allow-dirty --allow-staged

# --- Next.js app plane ------------------------------------------------------
app-install:
    cd app && pnpm install
app-lint:
    cd app && pnpm lint
app-build:
    cd app && pnpm build
app-dev:
    cd app && pnpm dev

# --- Data plane runtime -----------------------------------------------------
ingest *args:
    cargo run -p ingest -- {{args}}

# --- Database ---------------------------------------------------------------
db-up:
    docker compose up -d postgres
db-down:
    docker compose down
migrate:
    for f in db/migrations/*.sql; do psql "$DATABASE_URL" -f "$f"; done
# Seed the read-models from the mockup fixtures (contract-first app dev).
db-seed:
    for f in db/seed/*.sql; do psql "$DATABASE_URL" -f "$f"; done
