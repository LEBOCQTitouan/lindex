# Handoff protocol — every L'Index implementation session follows this

You have been handed **one lane** from `docs/roadmap.md`. This file is the
standard operating procedure. Read it fully, then read your lane's spec in the
roadmap. Do not skip steps.

## 0. Orient (before any code)
1. Read, in order: `CLAUDE.md`, `docs/project-brief.md`, `docs/user-stories.md`,
   `docs/indicator-catalog.md`, `docs/roadmap.md` (your lane + its deps),
   `docs/mockup-conventions.md`, and the relevant `mockups/*.html` +
   `mockups/data.js` (design source of truth), plus the scaffold you touch
   (`crates/`, `app/`, `db/migrations/`).
2. Confirm your lane's **deps are ✅ Done** on the status board. If not, stop and
   report "blocked on <lane>" — do not start.
3. Flip your lane's board row to **🔵 in-progress** (in your branch).

## 1. Superpowers workflow (mandatory — invoke the skills, don't freehand)
Run these in order; skip one only if it genuinely does not apply and say so:
- **superpowers:brainstorming** — if the lane has any design ambiguity (new UX,
  new data shape, a choice not pinned by the mockups/brief). Resolve intent first.
- **superpowers:using-git-worktrees** — ensure an isolated workspace (Conductor
  provides one; confirm you are not on a shared branch).
- **superpowers:writing-plans** — write the lane's implementation plan.
- **superpowers:test-driven-development** — all code is tests-first. (Tests are
  part of done — non-negotiable per CLAUDE.md.)
- **superpowers:executing-plans** — execute the plan with review checkpoints;
  use **superpowers:subagent-driven-development** or
  **superpowers:dispatching-parallel-agents** if the lane has independent
  sub-tasks that can fan out.
- **superpowers:verification-before-completion** — before claiming anything
  works, run the checks and show the evidence (test output, a rendered page).
- **superpowers:requesting-code-review** — before merge.
- **superpowers:finishing-a-development-branch** — integrate.

## 2. Definition of Done (the coordination guardrail)
Generic, every lane:
- Tests-first, all green; `just check` clean for Rust lanes; `pnpm build` clean
  for app lanes.
- **Contract-first**: app lanes build against seeded read-models (see the seed in
  the roadmap) and never wait on the data lane.
- No new edits to files owned by another lane. New page → new files.
Platform invariants (enforced, not aspirational):
- Every displayed figure ships with its **baseline**; provenance label + source
  link on every block; no composite scores; no rankings of people; symmetry where
  indicators appear; **French app text / English code**.

## 3. Status board rules
- The board in `docs/roadmap.md` is the shared map. Update **only your lane's
  row**. Your merged PR flips it to **✅ Done** (or **🟣 in-review** while open).
- Live in-flight status is visible in Conductor's UI; the board is the durable
  record.

## 4. Session-end report (ALWAYS produce this last — it is the point)
End every working session — done or paused — with exactly this block, so the
coordinator always knows what is next and whether the chat is safe to archive:

```
## Session-end report — <lane id / name>
- Status: ✅ done | 🟣 in-review (PR #) | 🔵 paused (blocked on <what>)
- Landed: <one-line summary> · PR: <link>
- Verified: <DoD evidence — test output, page renders, cargo/pnpm clean>
- Next actions: <downstream lanes now unblocked, by id>
- Follow-ups (paste-ready, from §5): <one self-contained handoff block per
  newly-ready & unclaimed lane, or "none — no new lanes unblocked">
- Archive-ok: YES — nothing pending/blocked/running
            | NO — still live: <what> (per CLAUDE.md, never archive while live)
```

The **Next actions**, **Follow-ups**, and **Archive-ok** lines are mandatory.
"Archive-ok: YES" is allowed only when your work is merged (or fully handed off)
and nothing is pending, blocked, or running in the background.

## 5. Emit the next tasks — the self-perpetuating chain (run by the LANE itself)
No auto-merge and no coordinator: each lane self-merges (see CLAUDE.md → MERGE
RULES), then runs this to hand the human the next tasks. It is status-aware, so
parallel lanes never emit the same task twice:
1. Read the status board on latest `main`: `git fetch origin`, then
   `git show origin/main:docs/roadmap.md`.
2. Confirm your PR merged; flip your lane's board row to ✅ (part of your merge, or
   a tiny follow-up commit to main).
3. **READY** = every lane whose deps are *all* ✅ on the board.
4. **AVAILABLE** = READY minus any lane that already has a branch
   (`git branch -a`, open PRs) or whose row is 🔵 / 🟣 / ✅ / 🟡-queued.
5. For each AVAILABLE lane, emit a **self-contained handoff** (template below) in
   your session-end report, and mark its board row `🟡 queued` in your board
   update so a near-simultaneous finisher won't re-emit it. (A lane usually
   becomes ready only when its *last* dep merges, so the finisher of that dep is
   the natural sole emitter; this guard just covers ties.)
6. If AVAILABLE is empty: state "No new lanes unblocked."

The human copies each emitted block into a fresh workspace **verbatim — no edits.**

### Self-contained handoff template (fill `<LANE>`, `<deps>`, `<task>`)
```
Implement lane <LANE> of L'Index (Rust hexagonal data plane crates/ + bin/,
Next.js app plane app/, shared Postgres facts/app schemas).
1) Sync: git fetch origin && git merge origin/main
2) Read: CLAUDE.md, CONSTITUTION.md, GUIDELINES_CHEATSHEET.md, docs/handoff-protocol.md,
   docs/roadmap.md (§<LANE> + deps), docs/project-brief.md, docs/user-stories.md, docs/indicator-catalog.md.
3) Deps (must be ✅ on the roadmap board, else stop and report blocked): <deps>
4) Task: <one-paragraph task, copied from docs/roadmap.md §<LANE>>
5) Method: follow docs/handoff-protocol.md — superpowers skills in order (brainstorming only if a
   design choice is ambiguous → writing-plans → test-driven-development → executing-plans →
   verification-before-completion → requesting-code-review → finishing-a-development-branch).
   New files only where possible; never edit another lane's files.
6) Rules: baseline on every figure; provenance + source on every datum; no composite
   scores/rankings/verdicts; symmetry; French UI / English code; thiserror in libs, anyhow
   in bins; #![forbid(unsafe_code)]; no unwrap/expect/panic in library code.
7) Gate: `just ci` (data plane) and/or `just app-lint app-build` (app plane) — local == CI.
8) Commit: Conventional Commits (scope = crate/package, body = why); update your row
   on docs/roadmap.md. Open a PR, then SELF-MERGE when green: git fetch && git merge
   origin/main, re-run the gate, then gh pr merge <#> --squash --delete-branch (retry
   if another lane merged first). No --auto, no coordinator.
9) Finish with the session-end report (§4) AND emit the follow-ups (§5) — the
   paste-ready blocks for whatever your merge unblocked.
```
