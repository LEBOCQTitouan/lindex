# Mockup conventions — L'Index (read before building any page)

This file lets several agents build different pages **in parallel** without
colliding. The homepage hybrid (`mockups/d-hybride.html`) is the reference
implementation. Read it before you start.

## Golden rule: no two sessions touch the same file
Each page-building session creates **only new files**:
- `mockups/<page>.html`  — the page itself, self-contained.
- `mockups/data-<page>.js` — that page's own real-data fixture (see below).

**Do NOT edit** `mockups/data.js`, `mockups/d-hybride.html`, or any other
existing page. If you think you need a change to a shared file, stop and flag it
instead — a shared edit breaks parallelism.

## Data
- `mockups/data.js` is the shared homepage fixture. It exposes
  `window.LINDEX = { data, fmt, ui }`. **Read it, reuse it, don't modify it.**
- Your page loads BOTH: `<script src="data.js"></script>` then
  `<script src="data-<page>.js"></script>`. Your file must expose its own
  namespace, e.g. `window.LINDEX_SCRUTIN = { ... }` (never overwrite `LINDEX`).
- **Real data.** Populate from official Assemblée nationale open records via
  WebFetch: `https://www.assemblee-nationale.fr/dyn/17/scrutins/<n>` returns the
  per-group breakdown; scrutins 8430 and 8433 (21 Jul 2026) are already in
  `data.js`. Fetch what your page needs and cite the URL in a `source`.
- **Never fabricate numbers dressed as real.** Anything you can't source live
  (CR-intégral timings, Légifrance décrets, per-member positions if not
  fetched) must be rendered but badged with `ui.illusTag()` → "illustratif".
- Every figure ships with its **baseline** (median of comparable objects). If you
  can compute a baseline from the data you fetched, do so; otherwise flag it.

## Reuse these shared components (from `window.LINDEX.ui`)
- `ui.hemicycleDots(vote, {width,dot})` — dot hémicycle; colour = vote, position
  L→R = allegiance. Pair with `ui.hemicycleLegends(vote)` + `ui.bindHemiToggle(scope)`
  for the Vote↔Groupes toggle (scope element needs class `hemi-scope`).
- `ui.voteMeter(vote)` — linear meter spanning all 577 seats (absent = ghost hatch).
- `ui.participation(vote)` — "X / 577 votant·es" with day-median baseline.
- `ui.provenance(source)`, `ui.stat(val,unit,baseline)`, `ui.outcomeChip`,
  `ui.chamberTag`, `ui.themeTag`, `ui.illusTag`, `ui.seanceBar(seance)`.
- `fmt.part(vote)`, `fmt.pct`, `fmt.theme`, `fmt.chamberLabel`.
Vote object shape: `{scrutinNo, chamber, pour, contre, abstention, nonVotants,
membersTotal, votants, exprimes, breakdown:{GID:[pour,contre,abst,nv]}, source}`.

## Design system (copy the `<style>` atoms block from `d-hybride.html`)
Because each page must be self-contained, **copy the shared atoms CSS block**
(everything between `/* shared atoms */` and `/* hybrid layout */`) from
`d-hybride.html` into your page's `<style>`, then add page-specific layout CSS.
Tokens: ink `#1B1E26`, paper `#F1F1EC`, card `#FFF`, rule `#DAD9D2`, accent
(encre bleue) `#294270`; vote colours pour `#3C6E58` / contre `#9B4B3B` /
abst `#8A7635` / nv `#B9B8B1` / absent hollow. Type: serif display (`ui-serif,
Georgia`), system sans body, **mono for every hard number / id / timestamp**.
Signature: the dot hémicycle. Mobile-first 390px. Keyboard focus + reduced-motion.

## Non-negotiables (from CLAUDE.md + project-brief)
- No composite scores, no rankings of people. No editorial verdicts.
- Symmetry: minority AND majority/executive tactics in the same visual language.
- Provenance label (tiers 1–5) + source link on every piece of content.
- **App text in French; code/comments in English.** Microcopy uses the name
  (« L'Index », « C'est tout pour hier »). Tone factual, never accusatory.
- Cross-links between pages by filename (`href="scrutin.html"`); a target that
  doesn't exist yet is fine.

## Verify before you claim done
1. `python3 -m http.server 8000` from `mockups/`, open your page.
2. Check the browser console: **zero errors**.
3. Confirm every number has a baseline and every block has a provenance label.
