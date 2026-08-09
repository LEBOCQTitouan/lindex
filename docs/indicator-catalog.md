# Indicator Catalog — L'Index

Every indicator must be: countable from official data, displayed with its baseline (median of comparable objects), symmetric (minority AND majority/executive tactics), and never aggregated into a single "score" per person or group.

## 1. Per séance (the timeline bar)
| Indicator | Definition | Source | Caveat |
|---|---|---|---|
| Durée effective vs pauses | Debate time vs suspensions (count, duration, requester) | CR intégral — suspension/reprise timestamps verbatim in XML | Some suspensions are logistical (dinner); show time of day |
| Rappels au règlement | Count, per group | CR — labeled interventions | Some are legitimate procedure |
| Demandes de scrutin public | Count, requester | CR + scrutins data | Each costs ~5 min; also forces positions on record |
| Vérifications de quorum | Count (mostly Sénat) | CR | Classic clock-eater |
| Motions de procédure | Rejet préalable, renvoi en commission, censure | CR + dossier data | — |
| Faits personnels | Count | CR — labeled | — |
| Rythme | Articles/amendements examined per hour | CR structure + agenda | Complex texts legitimately slow — compare within text type |
| Séance de nuit | Flag + end time | Agenda / CR | — |

## 2. Per dossier
| Indicator | Definition | Source | Caveat |
|---|---|---|---|
| Temps calendaire | Dépôt → adoption vs median of comparable texts | Dossiers législatifs XML | Compare PJL with PJL, PPL with PPL |
| Procédure accélérée | Flag (government decision) | Dossier data | Majority-side indicator |
| Temps législatif programmé | Flag + allocated hours | Conférence des présidents | Majority-side: caps debate |
| 49.3 / vote bloqué (44.3) / seconde délibération | Count per dossier | CR + dossier data | Majority-side |
| Amendements | Filed/adopted/rejected/irrecevable (art. 40, art. 45) per group | Amendements datasets (AN, Sénat Ameli) | Volume alone ≠ obstruction; pair with duplicate share |
| Taux de quasi-doublons | Share of near-identical amendments per group | Computed (textual similarity) | Threshold published in methodology |
| Sort en CMP | Accord / échec | Dossier data | — |
| Censure constitutionnelle | Articles censored by CC | CC décisions | — |

## 3. Per scrutin
| Indicator | Definition | Source | Caveat |
|---|---|---|---|
| Participation | Votants / group size | Scrutins XML (per-member) | Full nominal detail on solemn votes only |
| Marge | Pour − contre | Scrutins | — |
| Cohésion de groupe | Rice index per group | Computed | Publish formula; align with Datan's methodology |
| Dissidents | Members voting against group majority | Computed | Always show their explication if given |
| Abstention / non-votants | Shown distinctly | Scrutins | Non-votant ≠ absent |

## 4. Per parliamentarian (most care)
| Indicator | Definition | Source | Caveat |
|---|---|---|---|
| Participation aux scrutins | Share of solemn scrutins with recorded position | Scrutins | NOT presence — no official séance attendance dataset. Never a "presence score" |
| Assiduité en commission | Recorded attendance | AN réunions data; Sénat equivalent | Excused absences not always distinguishable |
| Loyauté | Share of votes aligned with group majority | Computed | Descriptive, not a virtue score |
| Production | Amendements, rapports, PPL, questions | Respective datasets | Much work invisible — say so on every profile |
| Interventions | Count + speaking time | CR (speaker-tagged) | Speaking time allocated by group size — normalize |
| Mandats & fonctions | History, cumul, roles | Acteurs/mandats datasets | Factual, safe |

## 5. Per group
| Indicator | Definition | Source |
|---|---|---|
| Cohésion moyenne | Mean Rice index over period | Computed |
| Taux de succès des amendements | Adopted / filed | Amendements data |
| Couverture des explications de vote | Share of solemn votes with stated reasons | CR |
| Temps de parole | Share vs group size | CR |

## 6. Executive-side (the symmetry)
| Indicator | Definition | Source | Caveat |
|---|---|---|---|
| Décrets d'application | Share published; median delay per law | Légifrance échéanciers + Sénat annual report | The executive's stalling indicator: law voted but unapplied |
| Réponses aux questions écrites | Response rate + median delay per ministry | Questions datasets | Officially timestamped — fully mechanical |
| Procédure accélérée / 49.3 / ordonnances | Counts per legislature vs previous | Dossiers + JORF | Historical baseline essential |

## 7. Per sujet
| Indicator | Definition | Source |
|---|---|---|
| Activité | Decisions/débats per period | Sujet anchoring (code articles + keywords) |
| Acteurs principaux | Most active members/groups | CR + amendements |
| État d'application | Décrets pending on the sujet's laws | Légifrance échéancier |

## Data sources
- **data.assemblee-nationale.fr** — acteurs/mandats/organes, scrutins, amendements, dossiers, questions, agendas/réunions, comptes rendus. XML/JSON, daily fil de l'eau, Licence Ouverte.
- **data.senat.fr** — dosleg, Ameli, scrutins, séances; different schemas → unified model.
- **CR intégral/analytique** — speaker-tagged XML: suspensions, rappels, explications de vote, motions.
- **Légifrance / DILA** — JORF, consolidated codes, échéanciers décrets, CC decisions, ELI ids.
- **Sénat rapport annuel application des lois** — décrets tracking.
- **Computed layer** — cohesion, loyalty, duplicates, baselines: formulas on methodology page.

## Display rules (non-negotiable)
1. Every indicator ships with its **baseline** — a raw count is not information.
2. **No composite scores**, no rankings of people.
3. **Symmetry**: minority and majority/executive tactics in the same visual language.
4. Every number **links to its raw records**.
5. Methodology page documents every formula and threshold, versioned.
