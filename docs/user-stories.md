# User Stories — L'Index

## Personas
- **Camille — the curious citizen.** Wants 5 minutes a day to know what Parliament actually decided. No political science background. Reads on mobile, mostly.
- **Nadia — the engaged citizen.** Follows one or two topics closely (e.g., energy, housing). Wants to track a specific bill over months and know when something moves.
- **Marc — the journalist / researcher.** Needs precise, citable, verifiable data fast. Cares about primary sources, exports, and permalinks more than pretty UI.
- **Yann — the constituent.** Wants to know what *his* député/sénatrice does: votes, presence, interventions.
- **Admin (you).** Operates the ingestion pipeline and must trust it enough not to babysit it daily.

## Epic 1 — Daily digest (MVP core)
- **US-1.1** — As Camille, I want a "yesterday in Parliament" page listing what was debated and voted in both chambers, so that I can stay informed in a few minutes. *AC: one entry per event; plain-language one-paragraph summary; link to official source; both chambers; available by 9am. Event granularity: solemn votes, adoptions/rejections of texts and articles, motions, any scrutin public; amendments only when notable.*
- **US-1.2** — As Camille, I want each digest entry tagged by theme, so that I can skim to what I care about.
- **US-1.3** — As Camille, I want to see the *outcome* first (adopted/rejected/postponed, vote counts), so that I get the decision before the details.
- **US-1.4** — As Nadia, I want to filter the digest by chamber and theme.
- **US-1.5** — As Camille, I want provisional entries (compte rendu analytique) flagged as provisional and later replaced by the definitive version.
- **US-1.6** — As Camille, I want a weekly recap ("what changed this week").

## Epic 2 — Dossier (bill) tracking
- **US-2.1** — As Nadia, I want a page per dossier législatif showing its full timeline (dépôt → commissions → lectures → navette → CMP → promulgation).
- **US-2.2** — As Nadia, I want the current status and the *next scheduled step* highlighted.
- **US-2.3** — As Marc, I want each stage linked to its official documents.
- **US-2.4** — As Nadia, I want to subscribe to a dossier and get notified (email/RSS) when it moves.
- **US-2.5** — As Camille, I want a plain-language "what this bill does" summary at the top of a dossier page.
- **US-2.6** — As Nadia, I want a chronological view of every decision on a *sujet*, so I can understand its history. *A sujet = curated set of code articles + keywords; timeline = every decision touching those articles (Légifrance). Themes (broad) auto via CAP taxonomy; sujets (precise) curated, 10-20 to start.*
- **US-2.7** — As Nadia, I want a graph view of sujets (nodes) and their connections (shared code articles, dossiers, co-citations). Later phase.

## Epic 3 — Votes & participants
- **US-3.1** — As Camille, I want a page per scrutin showing the result, breakdown by group, totals.
- **US-3.2** — As Yann, I want to search my député/sénatrice and see their individual votes.
- **US-3.3** — As Marc, I want dissident votes highlighted (members voting against their group's majority).
- **US-3.4** — As Yann, I want a per-parliamentarian page (votes, interventions, commission roles).
- **US-3.5** — As Marc, I want vote data exports (CSV/JSON) and stable permalinks.
- **US-3.6** — As Camille, I want abstentions and non-votants shown distinctly.
- **US-3.7** — As Camille, I want to see how each political side voted (groups ordered left→right, hémicycle order) and *why they say* they voted that way: per-group justification cards from explications de vote — verbatim quote, speaker, source link, and sources for any claims cited. Fallback: debate interventions.
- **US-3.8** — As a reader, I want the procedural footprint of séances and dossiers made visible: séance timeline bar (debate segments, suspension gaps, event markers), indicator chips with baselines. Symmetric: minority AND majority/executive tactics.
- **US-3.9** — As a reader, I want coherence flags: neutral "deux éléments à rapprocher" juxtapositions (said vs voted / voted vs voted on same sujet / discourse vs amendments), each with mandatory context line, human-reviewed before publication. A component, not a page (appears on scrutin, sujet, profiles).

## Epic 4 — Transcripts & knowledge linking
- **US-4.1** — As Nadia, I want comptes rendus in a clean interface (speaker, group, timestamps).
- **US-4.2** — As Nadia, I want references to legal texts auto-linked to Légifrance.
- **US-4.3** — As Marc, I want mentions of reports, prior laws, institutions auto-linked.
- **US-4.4** — As Nadia, I want to jump from a transcript segment to the corresponding amendment or vote.
- **US-4.5** — As Marc, I want a permalink to any individual intervention.

## Epic 5 — Claim sourcing & quote verification
- **US-5.1** — Factual claims detected and paired with official sources. *AC: claim shown verbatim with CR link; source official and linked; no verdict — evidence only.*
- **US-5.2** — Quotes checked against the quoted source, match shown side by side.
- **US-5.3** — Sourcing annotations clearly distinguishable from official content.
- **US-5.4** — Readers can flag an incorrect annotation.
- **US-5.5** — Every LLM annotation carries provenance (model, date, source passage) and review status.

## Epic 6 — Search & alerts
- **US-6.1** — Full-text search across dossiers, transcripts, votes.
- **US-6.2** — Saved searches with alerts.
- **US-6.3** — Search filters (date, chamber, speaker, group, document type).

## Epic 7 — Trust & methodology
- **US-7.1** — Methodology page explaining data sources and how summaries are produced.
- **US-7.2** — Every displayed data point links back to its official source.
- **US-7.3** — Code and pipeline open source.
- **US-7.4** — Visible "last updated" timestamps per page.
- **Trust layer**: provenance labels, not scores — 5 tiers (actes authentiques / independent public bodies / government communication / parliamentary work / press & civil society); press gets third-party badges (CPPAP, JTI, IFCN/EFCSN, ownership).

## Epic 8 — Admin & operations
- **US-8.1** — Scheduled ingestion jobs (AN + Sénat) with failure alerting.
- **US-8.2** — Upstream schema changes detected and quarantined.
- **US-8.3** — Pipeline health dashboard.
- **US-8.4** — Idempotent re-ingestion of date ranges.

## Prioritization (MoSCoW → releases)
**Must (R1 MVP):** 1.1, 1.2, 1.3, 1.5, 3.1, 3.6, 7.1, 7.2, 7.4, 8.1, 8.4
**Should (R2):** 1.4, 1.6, 2.1, 2.2, 2.3, 3.2, 3.4, 3.7, 4.1, 6.1, 8.2, 8.3
**Could (R3):** 2.4, 2.5, 2.6, 3.3, 3.5, 3.8, 3.9, 4.2-4.5, 6.2, 6.3, 7.3
**Later (R4):** 2.7, 5.1-5.5
