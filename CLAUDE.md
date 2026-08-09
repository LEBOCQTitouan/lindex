# L'Index — project guide (context for every change)

L'Index is a non-partisan civic platform to *understand* the French Parliament
(AN + Sénat): extraction with sources, never editorial verdicts.

**Read first:** `docs/project-brief.md` (product decisions), `CONSTITUTION.md`
(identity + non-negotiables), `GUIDELINES_CHEATSHEET.md` (one-line rules +
decision ladder), `docs/roadmap.md` + `docs/handoff-protocol.md` (how work is
coordinated across parallel workspaces). Every statement below is load-bearing.

## Architecture — two planes, one contract
- **Data plane (Rust, hexagonal):** `crates/` + `bin/`. Dependency direction
  `domain ← application ← adapters ← bin`. Ingests AN/Sénat/Légifrance → unified
  model → projects **read-models**.
- **App plane (Next.js):** `app/`. SSR-reads read-models; never writes `facts`.
- **Boundary:** one Postgres, schemas `facts` (Rust-owned, read-only to the app)
  and `app` (app-owned). `facts.read_*` IS the contract.

## Non-negotiables (full list in GUIDELINES_CHEATSHEET.md)
- Every displayed figure ships with its **baseline**; provenance on every datum;
  **no composite scores, no rankings of people, no editorial verdicts; symmetry**.
- **French app text; English code & comments.**
- Rust: `thiserror` in libraries, `anyhow` in bins; no `unwrap`/`expect`/`panic`
  in library code (tests exempt); `#![forbid(unsafe_code)]` by default (ADR to
  opt out). **Tests are part of Done.**
- **"Done" = implementation + tests + docs + CHANGELOG entry + ADR if class-A.**

## local == CI: use `just`
Every gate is a `just` verb byte-identical to CI: `just fmt`, `just lint`,
`just test`, `just deny`, and `just ci` (all of them). App plane: `just app-lint`,
`just app-build`. Run **`just ci` before opening a PR**; `lefthook` runs the fast
subset on pre-commit.

## CARGO rules (parallel-workspace build hygiene)
NEVER run bare `cargo`. Isolate the target dir so parallel workspaces don't
contend on one lock, and disable incremental (kills sccache hit-rate):
```
env CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/agent cargo <cmd> -p <crate>
```
Scope with `-p <crate>`; wrap long commands in `timeout`; prefer `cargo nextest`.

## AGENT PUSH RULES (the multi-workspace merge race)
`main` is protected: PRs must be up-to-date (strict). To merge:
1. `gh pr merge <PR#> --squash --delete-branch --auto`  (enrol once)
2. `gh pr update-branch <PR#>`  whenever GitHub reports **BEHIND** (repeat if main keeps moving — `--auto` does NOT update-branch for you)
3. GitHub auto-merges when CI is green **and** the branch is current.

Commit with an explicit identity to survive any corrupted worktree-local config:
`git -c user.name="<you>" -c user.email="<you>" commit …`
Never: merge red CI; self-merge without the review checklist.

## Class-A changes require an ADR (`docs/decisions/`)
read-model / data-schema changes, public routes or API, DB migrations,
cross-plane interface changes, guideline changes. Copy `docs/decisions/template.md`
to `NNNN-<title>.md` (`NNNN` = highest + 1); cite the ADR from the code it governs.

## Docs
`docs/` is Diátaxis-ish: `decisions/` (ADRs), `plans/` (specs), plus the product
docs. One canonical home per topic. Update `CHANGELOG.md` under `[Unreleased]`.

## Mockups (`mockups/`)
Static design reference — rules in `docs/mockup-conventions.md`. Self-contained
HTML, shared fixture `mockups/data.js`, no framework. Serve with
`python3 -m http.server 8000` from `mockups/`.
