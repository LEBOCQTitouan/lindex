# Project Brief — L'Index

## Mission
Non-oriented civic platform to *understand* French politics: what Parliament debated and decided, who voted what and why (their own stated reasons), with best-in-class comprehension UX. Neutrality by construction: extraction with sources, never editorial verdicts.

## Retained decisions (chronological)
1. **Scope**: Parliament first (AN + Sénat). Executive layer (décrets d'application) as indicators. Local (mairies, préfets) = long-term vision only.
2. **Neutrality model**: never author arguments or verdicts — extract them from official sources with verbatim quotes and links. Pros/cons = arguments actually made in debates, attributed.
3. **Progressive disclosure**: every object exists at 4 depths — one-liner → card → structured view → primary sources.
4. **Trust layer**: provenance labels, NOT trust scores. 5 tiers: actes authentiques / independent public bodies (INSEE, Cour des comptes…) / government communication / parliamentary work / press & civil society. Press gets third-party badges only (CPPAP, JTI, IFCN/EFCSN, ownership transparency).
5. **Press module**: link-out + headline + very short extract only (droits voisins). Coverage diversity display, no quality verdicts.
6. **Sujet (topic) timeline** (US-2.6): a sujet = a curated set of code articles + keywords; its timeline = every decision touching those articles, via Légifrance. Themes auto-tagged via CAP taxonomy.
7. **Subject graph** (US-2.7): sujets as nodes, edges from shared code articles / dossiers / co-citations. Later phase.
8. **Vote justifications** (US-3.7): per-group breakdown ordered left→right (hémicycle order), each group with a "why they say they voted this way" card built from their explication de vote — verbatim quote, speaker, source link. Fallback: debate interventions.
9. **Procedural footprint, not "stalling detection"** (US-3.8): symmetric factual indicators. Minority tactics (amendments, rappels, quorum, suspensions) AND majority/executive tactics (49.3, procédure accélérée, TLP, vote bloqué, décrets manquants). Séance = timeline bar with debate segments, suspension gaps, event markers. Every count displayed vs median baseline.
10. **Coherence flags** (US-3.9): neutral "deux éléments à rapprocher" juxtapositions (said vs voted, voted vs voted on same sujet, discourse vs amendments). Context line mandatory (e.g. "le texte a été modifié entre les deux votes"). Human review before publication.
11. **Indicator catalog**: ~30 indicators across 7 scopes (séance, dossier, scrutin, parlementaire, groupe, exécutif, sujet) — see docs/indicator-catalog.md. Display rules: baseline on every number, no composite scores, no people rankings, symmetry, every number links to raw records, versioned methodology page.
12. **No presence scores**: only participation aux scrutins (recorded data) and commission attendance (official). Séance presence has no official dataset.

## Page inventory
1. Accueil/Digest · 2. Dossier · 3. Scrutin · 4. Séance · 5. Sujet · 6. Parlementaire/Groupe · 7. Compte rendu · 8. Méthodologie.
Coherence flags = a component (on scrutin, sujet, profiles), not a page.

## Homepage — current working direction (to be validated via mockups)
Three candidates explored:
- **A "Le fil"**: news-feed card stack. + zero learning curve, mobile-native. − flat hierarchy, doomscroll mechanic.
- **B "Le journal"**: finite daily édition — one lead card, secondary cards by theme, brèves strip, explicit end ("C'est tout pour hier"). + hierarchy, finishability, ritual. − lead selection must follow a published mechanical rule (scrutin solennel > adoption définitive > margin closeness > theme breadth) or it becomes an orientation vector.
- **C "Le tableau de bord"**: system-state widgets (textes en navette, séance en cours, votes attendus) + event grid. + unique, great for power users, live. − cold, mobile-hostile, selects for the convinced.
- **Working hybrid**: B skeleton + A card mechanics for secondary items + C "séance en direct" strip only when Parliament sits.

## Personas
- **Camille** (primary): curious citizen, 5 min/day, mobile, no polisci background.
- **Nadia**: engaged citizen, follows 1–2 sujets over months.
- **Marc**: journalist — permalinks, exports, primary sources.
- **Yann**: constituent checking his own député.

## Data sources
data.assemblee-nationale.fr, data.senat.fr (different schemas — unified internal model, candidate: Parla-CLARIN/ParlaMint TEI), Légifrance/DILA (JORF, codes, échéanciers décrets, ELI ids), Sénat rapport application des lois. All Licence Ouverte.

## Mockup phase — what to build
Static, no backend, realistic fake French data (real-sounding dossiers, groups, names). Mobile-first (Camille), desktop variants for Marc. Build the 3 homepage variants + hybrid for side-by-side comparison, then iterate page by page (scrutin page next: breakdown + justification cards + coherence flag component).
