# L0-APP — scrutin page from `facts.read_scrutin` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to
> implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Make `app/` `/scrutin/[id]` render entirely from the `facts.read_scrutin`
read-model — totals, participation gap (votants vs 577 seats + baseline),
provenance chip, and the ported `<Hemicycle>` island with the Vote↔Groupes
toggle — seeded from `mockups/data.js` (scrutins 8433 & 8430), swapping to
L0-DATA's real data with zero app changes.

**Architecture:** Contract-first. A SQL seed populates `facts.read_scrutin` in the
shape L0-DATA will later write. The app SSR-reads one row → page. Pure mapping
logic (participation math, hémicycle seat allocation, read-model types) lives in
`app/src/lib/` (unit-tested); React islands render it; global CSS is ported from
the `d-hybride` mockup.

**Tech Stack:** Next.js 15 (App Router, RSC) · React 19 · Drizzle + postgres-js ·
TypeScript strict · Vitest (unit) · Postgres 16.

## Global Constraints
- Every displayed figure ships with its **baseline** (P1). Provenance label +
  source link on every datum (P4). No composite scores / rankings / verdicts.
- **French app text; English code & comments** (P6).
- TypeScript `strict`; **no `any` in shared types** (A1). App **never writes**
  `facts` — SELECT only (A2).
- The app must render byte-for-byte the same once L0-DATA writes real rows:
  read only the columns defined in `db/migrations/0001_facts.sql`
  (`scrutin_id, chamber, title, outcome, totals, breakdown, baselines,
  provenance, updated_at`); never depend on the seed's provenance beyond
  `{tier, source_record, url}`.
- Gate: `just app-lint app-build` (== CI `app.yml`: `pnpm install
  --frozen-lockfile && pnpm lint && pnpm build`). Requires a committed
  `app/pnpm-lock.yaml` and an ESLint config.
- Seed numbers are **verbatim** from `mockups/data.js`; breakdown per-category
  sums MUST equal totals; per-group seat sum MUST equal 577.

---

### Task 1: Toolchain — lockfile + ESLint so the CI gate can run

**Files:**
- Modify: `app/package.json` (add eslint, eslint-config-next, vitest devDeps +
  `test` script)
- Create: `app/.eslintrc.json`
- Create: `app/pnpm-lock.yaml` (generated)

**Steps:**
- [ ] Add `.eslintrc.json`: `{ "extends": "next/core-web-vitals" }`.
- [ ] Add devDeps `eslint`, `eslint-config-next` (matched to next 15),
  `vitest`; add `"test": "vitest run"` and `"test:watch": "vitest"` scripts.
- [ ] `cd app && pnpm install` → generates `pnpm-lock.yaml`.
- [ ] Verify `pnpm lint` runs (no interactive prompt) and `pnpm build` compiles.
- [ ] Commit.

### Task 2: Read-model contract types (`lib/read-model.ts`)

**Files:** Create `app/src/lib/read-model.ts`; Test `app/src/lib/read-model.test.ts`.

**Interfaces — Produces:**
```ts
export type GroupTally = { group: string; pour: number; contre: number; abst: number; nv: number };
export type ScrutinTotals = { pour: number; contre: number; abstention: number; nonVotants: number; membersTotal: number; votants: number };
export type ScrutinBaselines = { votants: number; abstention: number };
export type ScrutinProvenance = { tier: string; source_record: string; url: string };
export type ReadScrutin = { scrutinId: string; chamber: string; title: string; outcome: string; totals: ScrutinTotals; breakdown: GroupTally[]; baselines: ScrutinBaselines; provenance: ScrutinProvenance; updatedAt: Date };
// Narrowing helpers (jsonb columns arrive as unknown):
export function asTotals(v: unknown): ScrutinTotals
export function asBreakdown(v: unknown): GroupTally[]
export function asBaselines(v: unknown): ScrutinBaselines
export function asProvenance(v: unknown): ScrutinProvenance
```

- [ ] RED: test `asTotals`/`asBreakdown` round-trip a plain object; throw on a
  missing required numeric field.
- [ ] GREEN: implement with runtime shape checks (no `any`; use `unknown` +
  narrowing). Export the `PROVENANCE_TIERS` map (n + label) for the chip.
- [ ] Commit.

### Task 3: Participation math (`lib/participation.ts`)

**Files:** Create `app/src/lib/participation.ts`; Test `app/src/lib/participation.test.ts`.

**Interfaces — Consumes:** `ScrutinTotals`. **Produces:**
```ts
export type Participation = { votants: number; members: number; absent: number; nonVotants: number; exprimes: number; pct: number };
export function participation(t: ScrutinTotals): Participation; // absent = members - votants - nonVotants (>=0); pct = round(votants/members*100)
export type MeterSegment = { kind: "pour" | "contre" | "abst" | "nv" | "absent"; n: number };
export function meterSegments(t: ScrutinTotals): MeterSegment[]; // spans ALL 577 seats incl. absent
```

- [ ] RED: for 8433 totals → `{votants:537, absent:38, nonVotants:2, exprimes:530, pct:93}`;
  meter segment n's sum to `membersTotal` (577).
- [ ] GREEN: implement (mirror `fmt.part` / `ui.voteMeter` from data.js).
- [ ] Commit.

### Task 4: Hémicycle geometry to pure lib (`lib/hemicycle.ts`)

**Files:** Create `app/src/lib/hemicycle.ts`; Modify `app/src/components/Hemicycle.tsx`
(import from lib); Test `app/src/lib/hemicycle.test.ts`.

**Interfaces — Produces:**
```ts
export const GROUPS_AN: { id: string; color: string; members: number }[]; // 577 total, L→R
export type Seat = { x: number; y: number; kind: string; color: string };
export function buildSeats(breakdown: GroupTally[], mode: "vote" | "groupes"): { seats: Seat[]; W: number; H: number };
```

- [ ] RED: `GROUPS_AN` members sum to 577; `buildSeats(bd8433,"vote").seats.length === 577`;
  every seat has a colour; absent-seat count for a group = members − voted.
- [ ] GREEN: move `GROUPS`, `seatLayout`, `buildSeats`, `VOTE_COLOR` into lib;
  `Hemicycle.tsx` imports them (keeps `"use client"` + toggle + legends).
- [ ] Commit.

### Task 5: Enrich the `<Hemicycle>` island (legends, a11y)

**Files:** Modify `app/src/components/Hemicycle.tsx`; Modify `app/src/app/globals.css`.

- [ ] Add vote + group legends (hidden/shown per mode, mirroring mockup
  `.hemi-legend .lg-set`), `aria-pressed` toggle already present; ensure focus
  ring + `prefers-reduced-motion`.
- [ ] Port hémicycle/legend CSS atoms into `globals.css`.
- [ ] Commit (verified visually in Task 8).

### Task 6: Seed `facts.read_scrutin` (`db/seed/read_scrutin.sql`)

**Files:** Create `db/seed/read_scrutin.sql`; Modify `justfile` (add `db-seed`).

- [ ] Two `INSERT ... ON CONFLICT (scrutin_id) DO UPDATE` rows for 8433 & 8430,
  values verbatim from `mockups/data.js` (cite the `vote8433`/`vote8430` +
  `dayMedians` lines). `totals`/`breakdown`/`baselines`/`provenance` as jsonb.
- [ ] Add `db-seed` recipe: `psql "$DATABASE_URL" -f db/seed/read_scrutin.sql`.
- [ ] Commit.

### Task 7: Full scrutin page render (`app/src/app/scrutin/[id]/page.tsx`)

**Files:** Modify `app/src/app/scrutin/[id]/page.tsx`; Modify `app/src/app/globals.css`.

**Consumes:** `asTotals/asBreakdown/asBaselines/asProvenance`, `participation`,
`meterSegments`, `PROVENANCE_TIERS`, `<Hemicycle>`.

- [ ] Render (French): outcome chip; `<h1>` title + chamber/kind tags; the
  hémicycle island; participation line — `votants / membersTotal votant·es`
  with baseline `médiane du jour : {baselines.votants} · {pct} % des sièges`
  and gap `{absent} absent·es · {nonVotants} non-votant·es`; linear vote meter
  (all 577 seats, absent hatched); provenance chip → source link (tier dot +
  `Scrutin n° {id} — {chamberShort}`, title=tier label); marge line
  `marge +{pour−contre} · scrutin n° {id}`.
- [ ] Every figure has a baseline; provenance links out; no verdicts.
- [ ] Commit.

### Task 8: Verification (DB up → migrate → seed → render 390px)

- [ ] `docker compose up -d postgres`; `DATABASE_URL=postgres://lindex:password@localhost:5432/lindex just migrate db-seed`.
- [ ] `DATABASE_URL_FACTS=postgres://lindex:password@localhost:5432/lindex pnpm build && pnpm start`.
- [ ] Screenshot `/scrutin/8433` and `/scrutin/8430` at 390px; check focus ring
  (Tab to toggle) and reduced-motion.
- [ ] `just app-lint app-build` clean; `pnpm test` green.
- [ ] Update `CHANGELOG.md` [Unreleased]; flip roadmap row to ✅ (on merge).

## Self-Review
- Spec coverage: totals ✓(T7) · participation gap ✓(T3,T7) · provenance chip
  ✓(T2,T7) · baseline ✓(T3,T7) · hémicycle island + toggle ✓(T4,T5) · 390px /
  focus / reduced-motion ✓(T5,T8) · seed 8433+8430 ✓(T6) · zero-change swap
  ✓(contract via T2 + read-only columns) · gate ✓(T1,T8).
- No placeholders; types consistent across tasks (`GroupTally`, `ScrutinTotals`
  used verbatim in T2→T3→T4→T7).
