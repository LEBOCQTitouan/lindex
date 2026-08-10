# Changelog

All notable changes to L'Index are documented here. Format:
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project follows
[Semantic Versioning](https://semver.org). Pre-1.0, breaking changes bump the minor.

## [Unreleased]

### Added
- L0-APP spine: `/scrutin/[id]` renders entirely from the `facts.read_scrutin`
  read-model — totals, the participation gap (votants vs 577 seats) with its
  day-median baseline, provenance chip + source link, and the ported
  `<Hemicycle>` island (Vote ↔ Groupes toggle). Read-model seed from the mockup
  fixtures (`db/seed/read_scrutin.sql`, scrutins 8433 & 8430); pure mapping in
  `app/src/lib` unit-tested with Vitest; swaps to L0-DATA's real rows with zero
  app changes. App toolchain wired for the CI gate (ESLint, lockfile, Vitest).
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
