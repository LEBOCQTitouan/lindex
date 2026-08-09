# Guidelines cheatsheet — one line each

## Decision ladder
1. Covered by a guideline below? Apply it.
2. Violates a non-goal (CONSTITUTION §2)? Don't build it.
3. Class-A change (read-model/schema, public API/route, DB migration, cross-plane
   interface, guideline)? File an ADR.
4. Ambiguous? Surface to the maintainer — don't guess.

## Product
- **P1** Every figure ships with its baseline (median of comparables). A raw count is not information.
- **P2** No composite scores. No rankings of people or groups.
- **P3** No editorial verdicts. Extract and attribute; never author arguments.
- **P4** Provenance label (tier 1–5) + source link on every datum.
- **P5** Symmetry: minority AND majority/executive tactics in one visual language.
- **P6** French app text; English code and comments. Microcopy uses the name (« L'Index »).

## Rust (data plane)
- **R1** `just fmt` + `just lint` (clippy `-D warnings`) block merge.
- **R2** `thiserror` in libraries, `anyhow` in bins.
- **R3** No `unwrap`/`expect`/`panic` in library code (tests exempt).
- **R4** `#![forbid(unsafe_code)]` by default; opting out needs an ADR.
- **R5** Test layers: unit + integration + doc; proptest for parsers/invariants; golden tests for the read-model wire format.
- **R6** `#![deny(missing_docs)]` on library public items.
- **R7** Errors are `#[non_exhaustive]`; compose upward with `#[from]`; match with `matches!`, not string compares.

## App (Next.js plane)
- **A1** ESLint + Prettier + `tsc --noEmit` block merge; TypeScript `strict`; no `any` in shared types.
- **A2** SSR-read read-models only; never write `facts`. Writes go to the `app` schema.
- **A3** Component + integration tests; e2e for critical flows.

## Process
- **G1** `local == CI`: run `just ci` before a PR.
- **G2** Conventional Commits; scope = crate/package; **body explains why**, not just what.
- **G3** Code review required (even solo): reviewer, or overnight delay, or the self-checklist.
- **G4** Every bug becomes a regression test. No silent tech debt (fix in PR or file a tagged issue).
- **G5** Every new dependency justified in the PR description.
- **G6** "Done" = implementation + tests + docs + CHANGELOG entry + ADR if class-A.
- **G7** All governance files present: README, LICENSE, CONTRIBUTING, SECURITY, GOVERNANCE, CODE_OF_CONDUCT.
