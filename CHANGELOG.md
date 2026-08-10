# Changelog

All notable changes to L'Index are documented here. Format:
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project follows
[Semantic Versioning](https://semver.org). Pre-1.0, breaking changes bump the minor.

## [Unreleased]

### Added
- **L0-DATA spine:** AN scrutin ingest end-to-end — `ingest scrutin <id>` fetches
  the AN `dyn` analysis page, parses it to a normalized source record, validates
  it in the domain, persists `facts.scrutin`, and projects `facts.read_scrutin`
  with provenance and **day-median baselines**; re-running is idempotent (US-8.4).
  Real 8430/8433 pages are golden parser fixtures. Contract pinned in ADR-0001.
- `Scrutin.held_on`, `Baseline::from_samples` (median of comparables), the
  normalized `RawScrutinData` port payload, and `facts.ingestion_run` audit rows.
- Homepage mockups: four variants + hybrid on real Assemblée nationale data
  (21 Jul 2026), with a shared fixture and a dot-hémicycle (Vote ↔ Groupes).
- Rust data-plane scaffold (hexagonal): `domain` (invariant types), `application`
  (ports + ingest use case), `adapters-{persistence,sources}`, `ingest` CLI.
- Next.js app-plane scaffold: `/scrutin/[id]` reading the `facts.read_scrutin`
  read-model, with the hémicycle ported to a React island.
- Postgres schema contract: `facts` (Rust-owned) + `app` (app-owned).
- Coordination: `docs/roadmap.md` (lanes + status board), `docs/handoff-protocol.md`.
- Engineering conventions imported from the `tau` project: `CONSTITUTION.md`,
  `GUIDELINES_CHEATSHEET.md`, governance files, ADR system, `justfile`
  (local == CI), `lefthook`, quality config (`rustfmt`, `deny`, `nextest`,
  `gitleaks`, `.gitattributes`, workspace lints), CI/security/claude-review
  workflows, and `.worktreeinclude`.

### Notes
- Ingest use case and adapters are `todo!()` scaffolding — the first real slice
  (L0-DATA) fills them in.
- Follow-ups: SHA-pin GitHub Actions; choose a license (ADR); add the read-model
  schema-from-types drift test when L0 lands.
