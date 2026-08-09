# L'Index — mockup phase

## Context
L'Index is a non-partisan civic platform for understanding the activity of the French Parliament (AN + Sénat): daily digest, dossiers, votes (scrutins) with group justifications, sittings (séances), topics (sujets), procedural indicators.

**Read `docs/project-brief.md` first** — all product decisions live there. Supplement with `docs/user-stories.md` and `docs/indicator-catalog.md` as needed.

> Status note: the project has moved past mockups into implementation. The app
> lives in `crates/` (Rust data plane, hexagonal) + `app/` (Next.js app plane);
> see `docs/roadmap.md` and `docs/handoff-protocol.md`. The mockup rules below
> still govern anything under `mockups/`.

## Goal of the mockup phase
Static mockups only. No backend, no framework, no build step.

## Rules
- One self-contained `.html` file per variant (inline CSS or `<style>`, no external dependencies except system fonts).
- `mockups/index.html` = landing page listing every variant with links.
- Mobile-first 390px (Camille persona); desktop variant only when requested.
- Fake but realistic data: real political-group names, plausible dossiers, coherent figures. (Now populated from real AN open records — see `mockups/data.js`.)
- Deliberately include edge cases: a 12-word dossier title, a 3-member group, a high-abstention vote, a sitting with 40% suspensions.
- Shared fixture: `mockups/data.js` (same data in every variant so they compare on equal terms).
- Microcopy using the name: « L'Index du jour », « C'est tout pour hier ».
- Tone: factual, never accusatory. Every figure shown with its baseline (« 14 rappels — médiane : 2 »). Provenance labels on every piece of content.
- Serve with `python3 -m http.server 8000` from `mockups/`.

## Forbidden (mockup phase)
- No composite scores, no rankings of people.
- No editorial verdicts (no « vrai/faux », no « obstruction »).
- No JS framework, no `npm install` for the mockups.
