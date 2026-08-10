# Contributing to L'Index

Read `CONSTITUTION.md` and `GUIDELINES_CHEATSHEET.md` first — they are the rules.
This file is the mechanics.

## Setup
- Rust: `rustup` picks up `rust-toolchain.toml`. Install dev tools:
  `cargo install cargo-nextest cargo-deny --locked` and `gitleaks`, plus
  [`just`](https://github.com/casey/just) and [`lefthook`](https://lefthook.dev).
- Install hooks: `lefthook install`.
- App: `just app-install` (pnpm).

## The loop
1. Pick a **ready** lane from `docs/roadmap.md` (deps ✅). Follow `docs/handoff-protocol.md`.
2. Write tests first (TDD). Keep the domain pure; put I/O in adapters.
3. `just ci` (data plane) and `just app-lint app-build` (app plane) must pass —
   they are byte-identical to CI. `lefthook` runs the fast subset on commit.
4. Open a PR; end your session with the session-end report.

## Commits & PRs
- **Conventional Commits**: `<type>(<scope>): <subject>`. Scope = crate or package.
  Types: `feat` `fix` `docs` `refactor` `perf` `test` `build` `ci` `chore`.
  **The body explains *why***, not just what — the log is the durable decision
  trail below the ADR threshold.
- Every new dependency is justified in the PR description.
- Review is required even solo (reviewer, overnight delay, or the self-checklist).
  Never merge red CI. Never same-day self-merge without the checklist.
- Merging (see CLAUDE.md → MERGE RULES): lane sessions open a PR and stop; a
  coordinator merges manually — `gh pr update-branch <PR#>` if BEHIND, then
  `gh pr merge <PR#> --squash --delete-branch` (no auto-merge on this private repo).

## Class-A changes need an ADR
read-model/schema, public API/routes, DB migrations, cross-plane interfaces,
guideline changes. Copy `docs/decisions/template.md` → `NNNN-<title>.md`.

## Definition of Done
implementation + tests + docs + `CHANGELOG.md` entry + ADR if class-A.
