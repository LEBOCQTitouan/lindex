# Governance

L'Index is currently a **solo-maintainer** project, built across parallel
Conductor workspaces (human + agents). This is provisional; bus-factor is 1.

## Decision rights
| Change class | Who decides | Recorded as |
|---|---|---|
| Routine code (below thresholds) | Author | Commit + PR |
| Class-A (read-model/schema, public API/routes, DB migration, cross-plane interface, guideline) | Maintainer | **ADR** in `docs/decisions/` |
| Release | Maintainer | `CHANGELOG.md` + tag |
| Security fix | Maintainer | GitHub Security Advisory |
| Licensing | Maintainer | **ADR (open — not yet decided)** |

## Principles
- `CONSTITUTION.md` is the identity; it changes only via ADR.
- No silent scope creep: novel scope is checked against the non-goals.
- Every displayed number is sourced and baselined; no scores, no rankings.
