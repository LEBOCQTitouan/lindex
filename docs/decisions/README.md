# Architecture Decision Records

Class-A changes (see `CLAUDE.md` / `GOVERNANCE.md`) are recorded here before merge.

## How
1. Copy `template.md` to `NNNN-<short-title>.md`, where `NNNN` = the highest
   existing number + 1. **ADR numbers must be unique** — many parallel workspaces
   will collide otherwise; check `git fetch && ls docs/decisions/` first, and CI
   guards uniqueness.
2. Status starts `Proposed`; becomes `Accepted` when the PR merges.
3. Guideline-changing ADRs wait 24h before merge.
4. Cite the ADR from the code it governs (`// See docs/decisions/0003-….md`).

## Index
_(none yet — the first ADR will likely be `0001-license.md`.)_
