-- Read-model seed for the L0-APP contract-first spine.
--
-- Populates facts.read_scrutin with two real Assemblée nationale votes so the
-- app plane renders /scrutin/[id] before L0-DATA's ingestion exists. Every
-- number is VERBATIM from mockups/data.js (vote8433, vote8430) — the same
-- fixture the design mockups use — so the app meets L0-DATA at this schema and
-- swaps to real rows with zero app changes.
--
--   totals     {pour,contre,abstention,nonVotants,membersTotal,votants}
--   breakdown  [{group,pour,contre,abst,nv}] in hémicycle order (left→right)
--   baselines  day medians computed in data.js: {votants:426, abstention:7}
--   provenance {tier, source_record, url} — display label is derived in-app
--
-- Idempotent: re-running upserts the same rows (US-8.4 spirit).

insert into facts.read_scrutin
  (scrutin_id, chamber, title, outcome, totals, breakdown, baselines, provenance, updated_at)
values
  (
    '8433', 'AN',
    'Projet de loi visant à offrir des réponses immédiates aux phénomènes troublant l''ordre public (texte de la CMP)',
    'adopte',
    '{"pour":351,"contre":179,"abstention":7,"nonVotants":2,"membersTotal":577,"votants":537}'::jsonb,
    '[
      {"group":"LFI","pour":0,"contre":71,"abst":0,"nv":0},
      {"group":"GDR","pour":0,"contre":15,"abst":0,"nv":0},
      {"group":"ECOS","pour":0,"contre":36,"abst":0,"nv":0},
      {"group":"SOC","pour":0,"contre":56,"abst":1,"nv":0},
      {"group":"LIOT","pour":18,"contre":1,"abst":4,"nv":0},
      {"group":"DEM","pour":33,"contre":0,"abst":0,"nv":0},
      {"group":"EPR","pour":84,"contre":0,"abst":0,"nv":1},
      {"group":"HOR","pour":32,"contre":0,"abst":0,"nv":1},
      {"group":"DR","pour":46,"contre":0,"abst":1,"nv":0},
      {"group":"UDR","pour":17,"contre":0,"abst":0,"nv":0},
      {"group":"RN","pour":114,"contre":0,"abst":0,"nv":0},
      {"group":"NI","pour":7,"contre":0,"abst":1,"nv":0}
    ]'::jsonb,
    '{"votants":426,"abstention":7}'::jsonb,
    '{"tier":"acte","source_record":"an-scrutin-8433","url":"https://www.assemblee-nationale.fr/dyn/17/scrutins/8433"}'::jsonb,
    '2026-07-22 08:57:00+02'
  ),
  (
    '8430', 'AN',
    'Projet de loi relatif à la protection de l''enfance (première lecture)',
    'adopte',
    '{"pour":378,"contre":7,"abstention":173,"nonVotants":1,"membersTotal":577,"votants":558}'::jsonb,
    '[
      {"group":"LFI","pour":0,"contre":0,"abst":69,"nv":0},
      {"group":"GDR","pour":5,"contre":7,"abst":5,"nv":0},
      {"group":"ECOS","pour":3,"contre":0,"abst":33,"nv":0},
      {"group":"SOC","pour":0,"contre":0,"abst":63,"nv":0},
      {"group":"LIOT","pour":23,"contre":0,"abst":0,"nv":0},
      {"group":"DEM","pour":33,"contre":0,"abst":1,"nv":0},
      {"group":"EPR","pour":86,"contre":0,"abst":2,"nv":1},
      {"group":"HOR","pour":35,"contre":0,"abst":0,"nv":0},
      {"group":"DR","pour":48,"contre":0,"abst":0,"nv":0},
      {"group":"UDR","pour":17,"contre":0,"abst":0,"nv":0},
      {"group":"RN","pour":119,"contre":0,"abst":0,"nv":0},
      {"group":"NI","pour":9,"contre":0,"abst":0,"nv":0}
    ]'::jsonb,
    '{"votants":426,"abstention":7}'::jsonb,
    '{"tier":"acte","source_record":"an-scrutin-8430","url":"https://www.assemblee-nationale.fr/dyn/17/scrutins/8430"}'::jsonb,
    '2026-07-22 08:57:00+02'
  )
on conflict (scrutin_id) do update set
  chamber    = excluded.chamber,
  title      = excluded.title,
  outcome    = excluded.outcome,
  totals     = excluded.totals,
  breakdown  = excluded.breakdown,
  baselines  = excluded.baselines,
  provenance = excluded.provenance,
  updated_at = excluded.updated_at;
