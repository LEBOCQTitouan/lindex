# L'Index — Constitution

Audience: humans reading to understand what L'Index is; LLMs reading as context
for every agentic decision about this codebase. Every statement is load-bearing.

This document changes only via an ADR (`docs/decisions/`). No silent revisions.

## 1. Identity
L'Index makes the activity of the French Parliament **understandable** without
orienting the reader. It **extracts** facts from official sources — votes,
debates, dossiers, procedure — and presents them with provenance and baselines.
It never authors arguments or verdicts. Neutrality is a construction, not a tone.

## 2. Non-goals (writing these down reduces scope-creep pressure)
- Not an opinion or analytics product; no editorial "good/bad", no "obstruction".
- No composite scores and no rankings of people or groups — ever.
- Not a general CMS, forum, or social network.
- No PII beyond the public parliamentary record.
- Not a press aggregator: press is link-out + headline + short extract only.
- The app plane never mutates parliamentary facts; the data plane never owns
  user/editorial state.
- No scope beyond Parliament (executive as indicators only; local politics is a
  long-term vision, not now).

## 3. Quality posture
Quality is a first-class constraint, not a trade-off. The rule is **no cheap
shortcuts** — not "every possible practice." See `GUIDELINES_CHEATSHEET.md` for
the concrete bars. Tests, docs, and provenance are part of shipping, not extras.

## 4. Process
- Work is coordinated across parallel Conductor workspaces via `docs/roadmap.md`
  (lanes + status board) and `docs/handoff-protocol.md` (the per-session SOP).
- Class-A changes (§ CLAUDE.md) require an ADR before merge.
- `local == CI`: the `just` verbs are the single source of truth for gates.
- Every session ends with a session-end report (next actions + archive-ok).
