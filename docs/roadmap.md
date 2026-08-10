# L'Index — implementation roadmap

Dependency-ordered plan for building the app across parallel sessions. Pair this
with `docs/handoff-protocol.md` (the SOP every session follows).

## How to use this (the operating loop)
1. Look at the **status board** → any lane with all deps ✅ is **🟡 ready**.
2. Launch a session for a ready lane (handoff = "read `docs/handoff-protocol.md`,
   then this file §`<lane>`; execute the lane").
3. The session follows the protocol and ends with a **session-end report**
   (next actions + archive-ok).
4. Its merged PR flips the board; newly-satisfied deps make more lanes 🟡.

Status legend: ⬜ blocked · 🟡 ready · 🔵 in-progress · 🟣 in-review · ✅ done.

## Parallelization model
- **Contract-first (biggest unlock):** the read-model schema (`facts.read_*`) is
  the boundary. Seed read-models from the mockup fixtures → **app lanes build
  every page on day one** while data lanes implement ingestion. They meet at the
  schema.
- **Per-object vertical slices:** each `(source → domain/projection → read-model
  → page)` is independent once the spine exists.
- **Horizontal capability lanes:** ops, search, auth/editorial, provenance.
- **Serialization point:** the **P0 spine** (reference slice: AN scrutin E2E)
  lands first; keep it on 1–2 people, then fan out.

Tracks: `T1` Sources (Rust) · `T2` Domain+Projections (Rust) · `T3` App (Next.js)
· `T4` Platform/Ops · `T5` Cross-cutting.

## Status board

| Lane | Phase | Deps | US | Status | Unblocks |
|---|---|---|---|---|---|
| **L0-DATA** spine (data) | P0 | — | 3.1, 8.4, 7.2 | ✅ | everything |
| **L0-APP** spine (app, on fixtures) | P0 | schema shape | 3.1, 7.4 | ⬜ | all app lanes |
| L1-DIGEST | P1 | L0 | 1.1,1.2,1.3,1.5 | ⬜ | — |
| L1-SCRUTIN full | P1 | L0 | 3.1, 3.6 | ⬜ | L2-JUSTIF |
| L1-METHODO | P1 | L0-APP | 7.1,7.2,7.4 | ⬜ | — |
| L1-OPS | P1 | L0-DATA | 8.1, 8.4 | 🔵 | L2-OPS+ |
| L1-SENAT source | P1 | WI-0.1 | 1.1 (both chambers) | 🟡 | Sénat slices |
| L2-DOSSIER | P2 | L0 | 2.1,2.2,2.3 | ⬜ | L3-ALERTS, L3-SUMMARY |
| L2-PROFILES | P2 | L0 | 3.2, 3.4 | ⬜ | L3-DISSIDENTS |
| L2-JUSTIF | P2 | L1-SCRUTIN, L2-CR | 3.7 | ⬜ | — |
| L2-CR transcripts | P2 | L0 | 4.1 | ⬜ | L2-JUSTIF, L3-TRANSCRIPT+ |
| L2-SEARCH | P2 | L0 (data) | 6.1 | ⬜ | L3-SAVEDSEARCH |
| L2-DIGEST+ | P2 | L1-DIGEST | 1.4, 1.6 | ⬜ | — |
| L2-OPS+ | P2 | L1-OPS | 8.2, 8.3 | ⬜ | — |
| **L3-AUTH** (hub) | P3 | L0-APP | — | ⬜ | L3-ALERTS, L3-COHERENCE, L3-SAVEDSEARCH |
| L3-SUJET | P3 | L0, Légifrance src | 2.6 | ⬜ | L4-GRAPH |
| L3-ALERTS subscribe | P3 | L2-DOSSIER, L3-AUTH | 2.4 | ⬜ | — |
| L3-SUMMARY plain-lang | P3 | L2-DOSSIER | 2.5 | ⬜ | — |
| L3-DISSIDENTS | P3 | L2-PROFILES | 3.3 | ⬜ | — |
| L3-EXPORTS | P3 | L1-SCRUTIN | 3.5 | ⬜ | — |
| L3-PROCEDURAL | P3 | L2-CR | 3.8 | ⬜ | — |
| L3-COHERENCE | P3 | L3-AUTH | 3.9 | ⬜ | — |
| L3-TRANSCRIPT+ | P3 | L2-CR | 4.2–4.5 | ⬜ | — |
| L3-SAVEDSEARCH | P3 | L2-SEARCH, L3-AUTH | 6.2, 6.3 | ⬜ | — |
| L3-OPENSOURCE | P3 | — | 7.3 | ⬜ | — |
| L4-ANNOTATE | P4 | L0 | Epic 5 base | ⬜ | L4-CLAIMS |
| L4-CLAIMS | P4 | L4-ANNOTATE | 5.1–5.5 | ⬜ | — |
| L4-GRAPH | P4 | L3-SUJET | 2.7 | ⬜ | — |

## Lane specs
Each lane: **Goal**, **Work items**, **Done when**, **Unblocks**. WIs map to the
tree in this repo's history; acceptance items are the lane's specific DoD (on top
of the generic DoD in the protocol).

### L0-DATA — spine (data plane)
- **Goal:** AN scrutin flows source → domain → Postgres → read-model, idempotently.
- **WIs:** 0.1 domain value objects + errors · 0.2 persistence base (migration
  runner, `source_record`, idempotent upsert) · 0.3 AN scrutin source adapter
  (golden fixtures 8430/8433 from the mockups) · 0.4 `IngestScrutin` use case
  (map → validate → persist → project `read_scrutin` incl. day-median baselines)
  · 0.5 ingest CLI runtime + `ingestion_runs` table · 0.7 provenance/baseline JSON
  in the read-model.
- **Done when:** `ingest scrutin 8433` populates DB; re-running is a no-op diff;
  `read_scrutin` row for 8433 matches the mockup fixture; golden-fixture tests green.
- **Unblocks:** every downstream data lane; L0-APP with real data.

### L0-APP — spine (app plane, contract-first)
- **Goal:** `/scrutin/[id]` renders from `facts.read_scrutin`, hémicycle island working.
- **WIs:** 0.6 app shell (layout, Drizzle read access, ported design-system CSS,
  the scrutin page) + a **read-model seed** from `mockups/data.js` payloads · the
  provenance/baseline UI chips (pairs with 0.7).
- **Done when:** with seeded data, `/scrutin/8433` renders (breakdown, participation,
  provenance, baseline) at 390px, focus + reduced-motion OK; swaps to real data
  once L0-DATA lands with zero app changes.
- **Unblocks:** all app lanes (they follow this page's pattern).

### L1-DIGEST — daily digest
- **Goal:** "yesterday in Parliament", both chambers, outcome-first, theme-tagged.
- **WIs:** day-events read-model + projection (1.1, 1.3) · CAP theme tagging (1.2)
  · provisional compte-rendu-analytique flag (1.5) · digest page (mockup: hybrid).
- **Done when:** digest for the fixture day renders each event with outcome + theme
  + provenance; provisional entries badged; matches the hybrid mockup.
- **Unblocks:** L2-DIGEST+.

### L1-SCRUTIN — scrutin page (full)
- **Goal:** the full vote page. **WIs:** breakdown by group L→R, abstention &
  non-votants shown distinctly, participation gap (3.1, 3.6).
- **Done when:** page matches the mockup scrutin view for 8433 & 8430; every number
  has its baseline. **Unblocks:** L2-JUSTIF, L3-EXPORTS.

### L1-METHODO — methodology & trust
- **Goal:** the trust layer surfaced. **WIs:** methodology content model + page (7.1);
  source-link-everywhere + visible last-updated (7.2, 7.4).
- **Done when:** methodology page lists sources/formulas/tiers; every data point on
  any page links to its source; pages show "last updated".

### L1-OPS — ingestion operations
- **Goal:** run ingestion unattended. **WIs:** scheduler (systemd timers) + failure
  alerting (8.1); idempotent date-range re-ingest (8.4).
- **Done when:** a scheduled run records to `ingestion_runs`, alerts on failure, and
  re-ingesting a date range is idempotent. **Unblocks:** L2-OPS+.

### L1-SENAT — Sénat scrutin source
- **Goal:** second chamber. **WIs:** Sénat scrutin source adapter → same domain model.
- **Done when:** a real Sénat scrutin ingests into `facts.scrutin` with provenance.

### L2 lanes (start when L0/their P1 deps are ✅)
- **L2-DOSSIER** (2.1–2.3): dosleg source + Dossier domain/timeline + read-model +
  page (mockup). Done: dossier timeline dépôt→CMP→promulgation, status + next step,
  each stage source-linked. Unblocks L3-ALERTS, L3-SUMMARY.
- **L2-PROFILES** (3.2, 3.4): acteurs/mandats source + parlementaire domain
  (participation aux scrutins — NOT presence; loyauté descriptive) + page + search.
  Unblocks L3-DISSIDENTS.
- **L2-CR** (4.1): speaker-tagged CR source + clean transcript page. Unblocks
  L2-JUSTIF, L3-TRANSCRIPT+.
- **L2-JUSTIF** (3.7): explications-de-vote read-model + per-group justification
  cards on the scrutin page (verbatim quote + speaker + source; fallback debate).
- **L2-SEARCH** (6.1): PG FTS (french, unaccent, pg_trgm) + `SearchIndex` port +
  search page + chamber/theme filters. Unblocks L3-SAVEDSEARCH.
- **L2-DIGEST+** (1.4, 1.6): digest filters + weekly recap.
- **L2-OPS+** (8.2, 8.3): schema-drift detection/quarantine + health dashboard.

### L3 lanes (auth-gated cluster + extensions)
- **L3-AUTH** (hub): accounts/sessions on the `app` schema. Pull to end of P2 so P3
  fans out. Unblocks L3-ALERTS, L3-COHERENCE, L3-SAVEDSEARCH.
- **L3-SUJET** (2.6): sujet anchoring (code articles + keywords) + Légifrance source
  + timeline read-model + page. Unblocks L4-GRAPH.
- **L3-ALERTS** (2.4): follow a dossier → email/RSS. **L3-SUMMARY** (2.5):
  plain-language "what this bill does". **L3-DISSIDENTS** (3.3): compute + highlight.
  **L3-EXPORTS** (3.5): CSV/JSON + stable permalinks. **L3-PROCEDURAL** (3.8):
  séance timeline bar + symmetric indicators (needs CR parse). **L3-COHERENCE**
  (3.9): coherence-flag component + human-review workflow (uses `app.review`).
  **L3-TRANSCRIPT+** (4.2–4.5): Légifrance/ref auto-linking, jump-to-amendment/vote,
  per-intervention permalinks. **L3-SAVEDSEARCH** (6.2, 6.3): saved searches +
  advanced filters. **L3-OPENSOURCE** (7.3): license, contributor docs, publish.

### L4 lanes (later)
- **L4-ANNOTATE**: `AnnotatorPort` + Anthropic adapter (provenance: model/date/source
  passage + review status). Unblocks L4-CLAIMS.
- **L4-CLAIMS** (5.1–5.5): claim detection + source pairing, quote verification
  side-by-side, distinguishable annotations, reader flagging — Python NLP sidecar.
- **L4-GRAPH** (2.7): sujet graph (nodes/edges: shared articles, co-citations).

## Dependency graph & hubs
```
L0 spine ─┬─▶ L1-{DIGEST,SCRUTIN,METHODO,OPS}  ─┬─▶ L2-{DOSSIER,PROFILES,CR,SEARCH,DIGEST+,OPS+}
          └─▶ L1-SENAT                          └─▶ L2-JUSTIF (needs SCRUTIN+CR)
L3-AUTH ──┬─▶ L3-ALERTS (needs DOSSIER)  ─▶ L3-COHERENCE / L3-SAVEDSEARCH
          └─▶ (personalization)
L3-SUJET ─▶ L4-GRAPH        L4-ANNOTATE ─▶ L4-CLAIMS
```
Two hubs to schedule early in their phase: **L0 spine** and **L3-AUTH**.

## Waves (map to concurrent sessions)
- **Wave 0** (1–2 lanes): L0-DATA, L0-APP.
- **Wave 1** (5): L1-DIGEST, L1-SCRUTIN, L1-METHODO, L1-OPS, L1-SENAT.
- **Wave 2** (6–8): L2-DOSSIER, L2-PROFILES, L2-CR, L2-SEARCH, L2-DIGEST+, L2-OPS+,
  (+ L2-JUSTIF once SCRUTIN+CR land) (+ start L3-AUTH).
- **Wave 3** (~8): the L3 cluster (auth-gated ones after L3-AUTH).
- **Wave 4** (2–3): L4-ANNOTATE, L4-CLAIMS, L4-GRAPH.
