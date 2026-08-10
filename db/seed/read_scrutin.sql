-- Read-model seed for the L0-APP contract-first spine.
--
-- Populates facts.read_scrutin with two real Assemblée nationale votes so the
-- app plane renders /scrutin/[id] with no Rust ingest running. Shapes are the
-- ADR-0001 wire contract (what L0-DATA's projection writes), so a seeded row and
-- a real ingested row render identically:
--
--   totals     {pour,contre,abstention,nonVotants,membersTotal,votants,exprimes}
--   breakdown  [{group,pour,contre,abstention,nonVotant}] hémicycle order (L→R)
--   baselines  {votants:{median,sampleSize},abstention:{median,sampleSize},method:{id,version}}
--   provenance {tier,label,url,recordId,retrievedAt}  (tier = domain enum name)
--
-- Numbers are verbatim from mockups/data.js (vote8433, vote8430). The baseline
-- is the day-median over this two-scrutin cohort (21 Jul 2026), exactly what
-- L0-DATA produces from its two golden fixtures (ADR-0001 §Consequences):
-- votants median((537,558)) = 547.5, abstention median((7,173)) = 90.
-- Idempotent: re-running upserts the same rows (US-8.4 spirit).

insert into facts.read_scrutin
  (scrutin_id, chamber, title, held_on, outcome, totals, breakdown, baselines, provenance, updated_at)
values
  (
    '8433', 'AN',
    'Projet de loi visant à offrir des réponses immédiates aux phénomènes troublant l''ordre public (texte de la CMP)',
    '2026-07-21', 'adopte',
    '{"pour":351,"contre":179,"abstention":7,"nonVotants":2,"membersTotal":577,"votants":537,"exprimes":530}'::jsonb,
    '[
      {"group":"LFI","pour":0,"contre":71,"abstention":0,"nonVotant":0},
      {"group":"GDR","pour":0,"contre":15,"abstention":0,"nonVotant":0},
      {"group":"ECOS","pour":0,"contre":36,"abstention":0,"nonVotant":0},
      {"group":"SOC","pour":0,"contre":56,"abstention":1,"nonVotant":0},
      {"group":"LIOT","pour":18,"contre":1,"abstention":4,"nonVotant":0},
      {"group":"DEM","pour":33,"contre":0,"abstention":0,"nonVotant":0},
      {"group":"EPR","pour":84,"contre":0,"abstention":0,"nonVotant":1},
      {"group":"HOR","pour":32,"contre":0,"abstention":0,"nonVotant":1},
      {"group":"DR","pour":46,"contre":0,"abstention":1,"nonVotant":0},
      {"group":"UDR","pour":17,"contre":0,"abstention":0,"nonVotant":0},
      {"group":"RN","pour":114,"contre":0,"abstention":0,"nonVotant":0},
      {"group":"NI","pour":7,"contre":0,"abstention":1,"nonVotant":0}
    ]'::jsonb,
    '{"votants":{"median":547.5,"sampleSize":2},"abstention":{"median":90,"sampleSize":2},"method":{"id":"scrutin-day-median","version":1}}'::jsonb,
    '{"tier":"ActeAuthentique","label":"Scrutin n° 8433 — AN","url":"https://www.assemblee-nationale.fr/dyn/17/scrutins/8433","recordId":"an-scrutin-8433","retrievedAt":"2026-07-22T06:57:00+00:00"}'::jsonb,
    '2026-07-22 08:57:00+02'
  ),
  (
    '8430', 'AN',
    'Projet de loi relatif à la protection de l''enfance (première lecture)',
    '2026-07-21', 'adopte',
    '{"pour":378,"contre":7,"abstention":173,"nonVotants":1,"membersTotal":577,"votants":558,"exprimes":385}'::jsonb,
    '[
      {"group":"LFI","pour":0,"contre":0,"abstention":69,"nonVotant":0},
      {"group":"GDR","pour":5,"contre":7,"abstention":5,"nonVotant":0},
      {"group":"ECOS","pour":3,"contre":0,"abstention":33,"nonVotant":0},
      {"group":"SOC","pour":0,"contre":0,"abstention":63,"nonVotant":0},
      {"group":"LIOT","pour":23,"contre":0,"abstention":0,"nonVotant":0},
      {"group":"DEM","pour":33,"contre":0,"abstention":1,"nonVotant":0},
      {"group":"EPR","pour":86,"contre":0,"abstention":2,"nonVotant":1},
      {"group":"HOR","pour":35,"contre":0,"abstention":0,"nonVotant":0},
      {"group":"DR","pour":48,"contre":0,"abstention":0,"nonVotant":0},
      {"group":"UDR","pour":17,"contre":0,"abstention":0,"nonVotant":0},
      {"group":"RN","pour":119,"contre":0,"abstention":0,"nonVotant":0},
      {"group":"NI","pour":9,"contre":0,"abstention":0,"nonVotant":0}
    ]'::jsonb,
    '{"votants":{"median":547.5,"sampleSize":2},"abstention":{"median":90,"sampleSize":2},"method":{"id":"scrutin-day-median","version":1}}'::jsonb,
    '{"tier":"ActeAuthentique","label":"Scrutin n° 8430 — AN","url":"https://www.assemblee-nationale.fr/dyn/17/scrutins/8430","recordId":"an-scrutin-8430","retrievedAt":"2026-07-22T06:57:00+00:00"}'::jsonb,
    '2026-07-22 08:57:00+02'
  )
on conflict (scrutin_id) do update set
  chamber    = excluded.chamber,
  title      = excluded.title,
  held_on    = excluded.held_on,
  outcome    = excluded.outcome,
  totals     = excluded.totals,
  breakdown  = excluded.breakdown,
  baselines  = excluded.baselines,
  provenance = excluded.provenance,
  updated_at = excluded.updated_at;
