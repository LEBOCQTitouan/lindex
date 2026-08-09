/* ============================================================================
   L'Index — shared fixture (mockups)
   Single source of truth for the homepage variants.
   No external dependencies. Exposes window.LINDEX = { data, fmt, ui }.

   REAL DATA. This fixture is populated from official Assemblée nationale
   open records (data / analyse des scrutins, 17th legislature) for a real
   sitting day: Tuesday 21 July 2026 (session extraordinaire).
     · Every scrutin number, per-group breakdown and total is verbatim from
       https://www.assemblee-nationale.fr/dyn/17/scrutins/<n>
     · Baselines (median votants / median abstentions) are COMPUTED from that
       day's five scrutins — not invented.
     · Two components rely on data we could not source live and are therefore
       flagged `illustrative:true` and badged "illustratif" in the UI:
         - séance procedural timings (needs CR intégral parsing)
         - décret d'application delays (needs Légifrance échéanciers)
     · Sénat is AN-only here: the Sénat scrutin URL scheme was not resolvable
       blind and we do not fabricate a vote. The model is chamber-agnostic.

   App-facing strings stay in French (product language); code is English.
   ========================================================================== */
(function () {
  "use strict";

  /* --- Provenance: 5 tiers (labels, never a score) ------------------------- */
  var provenanceTiers = {
    acte:      { n: 1, label: "Acte authentique",        hint: "Scrutins & JO officiels" },
    organisme: { n: 2, label: "Organisme public indép.", hint: "INSEE, Cour des comptes…" },
    gouv:      { n: 3, label: "Communication gouv.",      hint: "Ministères" },
    parlement: { n: 4, label: "Travaux parlementaires",   hint: "CR, dossiers, amendements" },
    presse:    { n: 5, label: "Presse & société civile",  hint: "Badges tiers uniquement" }
  };

  /* --- Themes (broad, CAP-style) ------------------------------------------- */
  var themes = {
    justice:     "Justice & sécurité",
    enfance:     "Famille & enfance",
    numerique:   "Numérique",
    institutions:"Institutions & État",
    agriculture: "Agriculture"
  };

  /* --- Political groups, hémicycle order left → right ----------------------
     Real groups & effectifs of the 17th legislature (as of the 21 Jul 2026
     scrutins). `color` = the group's identity colour (descriptive, never a
     judgement); rendered muted so no single one dominates.                    */
  var groupsAN = [
    { id: "LFI",  name: "La France insoumise – NFP",          members: 71,  color: "#C4002E" },
    { id: "GDR",  name: "Gauche démocrate et républicaine",   members: 17,  color: "#B5121B" },
    { id: "ECOS", name: "Écologiste et social",               members: 38,  color: "#4CA85B" },
    { id: "SOC",  name: "Socialistes et apparentés",          members: 68,  color: "#E63C8C" },
    { id: "LIOT", name: "Libertés, Indép., Outre-mer, Terr.", members: 23,  color: "#E0A02A" },
    { id: "DEM",  name: "Les Démocrates",                     members: 37,  color: "#F08C00" },
    { id: "EPR",  name: "Ensemble pour la République",        members: 91,  color: "#D4B106" },
    { id: "HOR",  name: "Horizons & Indépendants",            members: 35,  color: "#12A5A0" },
    { id: "DR",   name: "Droite Républicaine",                members: 48,  color: "#2E6FB5" },
    { id: "UDR",  name: "Union des droites pour la Rép.",     members: 17,  color: "#1D3D8F" },
    { id: "RN",   name: "Rassemblement national",             members: 122, color: "#22326B" },
    { id: "NI",   name: "Députés non inscrits",               members: 10,  color: "#8A8D94" }
  ];
  // Total: 577 seats.

  function groupMap(list) {
    var m = {}; list.forEach(function (g) { m[g.id] = g; }); return m;
  }
  var ANm = groupMap(groupsAN);

  /* --- Real scrutins ------------------------------------------------------
     breakdown: per-group [pour, contre, abstention, nonVotant] (verbatim).
     Absent seats per group are derived = members − (pour+contre+abst+nv).
     ---------------------------------------------------------------------- */

  // Scrutin 8433 — LEAD. Ordre public (texte CMP). Adopté 351–179–7.
  var vote8433 = {
    scrutinNo: 8433, chamber: "AN", kind: "Scrutin public (ensemble du texte)",
    result: "adopte", pour: 351, contre: 179, abstention: 7, nonVotants: 2,
    membersTotal: 577, votants: 537, exprimes: 530, majoriteAbsolue: 266,
    breakdown: {
      LFI:[0,71,0,0], GDR:[0,15,0,0], ECOS:[0,36,0,0], SOC:[0,56,1,0],
      LIOT:[18,1,4,0], DEM:[33,0,0,0], EPR:[84,0,0,1], HOR:[32,0,0,1],
      DR:[46,0,1,0], UDR:[17,0,0,0], RN:[114,0,0,0], NI:[7,0,1,0]
    },
    source: { tier: "acte", label: "Scrutin n° 8433 — AN", url: "https://www.assemblee-nationale.fr/dyn/17/scrutins/8433" }
  };

  // Scrutin 8430 — Protection de l'enfance (1re lecture). Adopté 378–7, abst 173.
  var vote8430 = {
    scrutinNo: 8430, chamber: "AN", kind: "Scrutin public (ensemble du texte)",
    result: "adopte", pour: 378, contre: 7, abstention: 173, nonVotants: 1,
    membersTotal: 577, votants: 558, exprimes: 385, majoriteAbsolue: 193,
    abstentionHigh: true,
    breakdown: {
      LFI:[0,0,69,0], GDR:[5,7,5,0], ECOS:[3,0,33,0], SOC:[0,0,63,0],
      LIOT:[23,0,0,0], DEM:[33,0,1,0], EPR:[86,0,2,1], HOR:[35,0,0,0],
      DR:[48,0,0,0], UDR:[17,0,0,0], RN:[119,0,0,0], NI:[9,0,0,0]
    },
    source: { tier: "acte", label: "Scrutin n° 8430 — AN", url: "https://www.assemblee-nationale.fr/dyn/17/scrutins/8430" }
  };

  // Scrutin 8431 — Mineurs & réseaux sociaux (CMP). Adopté 279–81, abst 66.
  var vote8431 = {
    scrutinNo: 8431, chamber: "AN", kind: "Scrutin public (ensemble du texte)",
    result: "adopte", pour: 279, contre: 81, abstention: 66, nonVotants: 2,
    membersTotal: 577, votants: 426, exprimes: 360, abstentionHigh: true,
    source: { tier: "acte", label: "Scrutin n° 8431 — AN", url: "https://www.assemblee-nationale.fr/dyn/17/scrutins/8431" }
  };

  // Scrutin 8432 — Motion de rejet préalable (ordre public, Chatelain). Rejeté 85–207.
  var vote8432 = {
    scrutinNo: 8432, chamber: "AN", kind: "Motion de rejet préalable",
    result: "rejete", pour: 85, contre: 207, abstention: 1, nonVotants: 2,
    membersTotal: 577, votants: 293, exprimes: 292,
    source: { tier: "acte", label: "Scrutin n° 8432 — AN", url: "https://www.assemblee-nationale.fr/dyn/17/scrutins/8432" }
  };

  // Scrutin 8434 — Patrimoine immobilier de l'État (CMP). Adopté 276–86–2.
  var vote8434 = {
    scrutinNo: 8434, chamber: "AN", kind: "Scrutin public (ensemble du texte)",
    result: "adopte", pour: 276, contre: 86, abstention: 2, nonVotants: 2,
    membersTotal: 577, votants: 364, exprimes: 362,
    source: { tier: "acte", label: "Scrutin n° 8434 — AN", url: "https://www.assemblee-nationale.fr/dyn/17/scrutins/8434" }
  };

  // Scrutin 8427 — Souveraineté agricole (CMP), 20 Jul (context brève). Adopté 296–224.
  var vote8427 = {
    scrutinNo: 8427, chamber: "AN", result: "adopte",
    pour: 296, contre: 224, abstention: 41, nonVotants: 1, membersTotal: 577, votants: 561,
    source: { tier: "acte", label: "Scrutin n° 8427 — AN", url: "https://www.assemblee-nationale.fr/dyn/17/scrutins/8427" }
  };

  /* --- Day-level baselines, COMPUTED from the 5 scrutins of 21 Jul --------- */
  function median(arr) {
    var a = arr.slice().sort(function (x, y) { return x - y; });
    var m = Math.floor(a.length / 2);
    return a.length % 2 ? a[m] : (a[m - 1] + a[m]) / 2;
  }
  var dayVotants = [vote8430, vote8431, vote8432, vote8433, vote8434].map(function (v) { return v.votants; });
  var dayAbst    = [vote8430, vote8431, vote8432, vote8433, vote8434].map(function (v) { return v.abstention; });
  var dayMedians = { votants: median(dayVotants), abstention: median(dayAbst) }; // {426, 7}

  /* --- Séance procedural footprint — ILLUSTRATIVE (CR not parsed) ---------- */
  var seanceOrdrePublic = {
    id: "an-ordre-public",
    chamber: "AN",
    label: "Séance du soir — Assemblée nationale",
    dossier: "PJL réponses immédiates aux phénomènes troublant l'ordre public",
    illustrative: true,
    nightFlag: true,
    startedLabel: "21 h 30", endedLabel: "00 h 45",
    wallMinutes: 195, suspensionMinutes: 47, suspensionMedianPct: 12,
    suspensions: [
      { atLabel: "21 h 55", minutes: 12, by: "LFI-NFP" },
      { atLabel: "22 h 40", minutes: 10, by: "présidence" },
      { atLabel: "23 h 20", minutes: 15, by: "RN" },
      { atLabel: "00 h 10", minutes: 10, by: "SOC" }
    ],
    rappels: 9, rappelsMedian: 2,
    scrutinsDemandes: 4, scrutinsDemandesMedian: 3,
    motions: 1,
    source: { tier: "parlement", label: "CR intégral AN (à analyser)", url: "https://www.assemblee-nationale.fr/dyn/17/comptes-rendus" }
  };

  /* --- Live-session strip (feature demo; timings ILLUSTRATIVE) ------------- */
  var liveSession = {
    chamber: "AN",
    title: "PJL réponses immédiates aux phénomènes troublant l'ordre public",
    dossierId: "an-ordre-public",
    currentItem: "Explications de vote sur l'ensemble du texte (CMP)",
    illustrative: true,
    startedLabel: "21 h 30", elapsedMinutes: 132, suspensionMinutes: 37,
    rappels: 9, rappelsMedian: 2,
    source: { tier: "parlement", label: "Séance — AN", url: "https://www.assemblee-nationale.fr/dyn/17/comptes-rendus" }
  };

  /* --- Events of the day (21 Jul 2026) ------------------------------------ */
  var events = [
    {
      id: "s8433", chamber: "AN", timeLabel: "2e séance", sortKey: 22,
      theme: "justice", type: "Scrutin — texte CMP",
      outcome: "adopte", outcomeLabel: "Adopté (texte CMP)",
      title: "Ordre public : l'Assemblée adopte le texte de la commission mixte paritaire",
      dossierTitle: "Projet de loi visant à offrir des réponses immédiates aux phénomènes troublant l'ordre public (texte de la CMP)",
      summary: "L'Assemblée nationale a adopté le texte élaboré par la commission mixte paritaire. Députés et sénateurs s'étaient accordés sur une version commune ; le vote de l'Assemblée vaut adoption de ce texte.",
      isLead: true, vote: vote8433, seance: "an-ordre-public",
      justificationsNote: "Explications de vote par groupe : à extraire du compte rendu (US-3.7).",
      source: vote8433.source
    },
    {
      id: "s8430", chamber: "AN", timeLabel: "1re séance", sortKey: 11,
      theme: "enfance", type: "Scrutin — 1re lecture",
      outcome: "adopte", outcomeLabel: "Adopté (1re lecture)",
      title: "Protection de l'enfance : adoption en première lecture, avec une très forte abstention",
      dossierTitle: "Projet de loi relatif à la protection de l'enfance (première lecture)",
      summary: "Le texte a été adopté en première lecture. L'abstention a été inhabituellement élevée : les groupes LFI-NFP, Socialistes et Écologiste se sont abstenus quasi intégralement.",
      isSecondaryLead: true, vote: vote8430, source: vote8430.source
    },
    {
      id: "s8431", chamber: "AN", timeLabel: "1re séance", sortKey: 12,
      theme: "numerique", type: "Scrutin — texte CMP",
      outcome: "adopte", outcomeLabel: "Adopté (texte CMP)",
      title: "Réseaux sociaux et mineurs : la proposition de loi issue de la CMP est adoptée",
      dossierTitle: "Proposition de loi visant à protéger les mineurs des risques auxquels les expose l'utilisation des réseaux sociaux (texte de la CMP)",
      summary: "L'Assemblée a adopté le texte commun élaboré en commission mixte paritaire. Soixante-six abstentions ont été enregistrées.",
      vote: vote8431, source: vote8431.source
    },
    {
      id: "s8432", chamber: "AN", timeLabel: "2e séance", sortKey: 21,
      theme: "justice", type: "Motion de rejet préalable",
      outcome: "rejete", outcomeLabel: "Motion rejetée",
      title: "Ordre public : la motion de rejet préalable déposée par Cyrielle Chatelain est repoussée",
      dossierTitle: "Motion de rejet préalable du projet de loi relatif à l'ordre public (Mme Cyrielle Chatelain)",
      summary: "Déposée avant l'examen du texte, la motion de rejet préalable a été repoussée. L'examen s'est donc poursuivi.",
      vote: vote8432, source: vote8432.source
    },
    {
      id: "s8434", chamber: "AN", timeLabel: "2e séance", sortKey: 23,
      theme: "institutions", type: "Scrutin — texte CMP",
      outcome: "adopte", outcomeLabel: "Adopté (texte CMP)",
      title: "Patrimoine immobilier de l'État : le texte de la commission mixte paritaire est adopté",
      dossierTitle: "Proposition de loi visant à moderniser la gestion du patrimoine immobilier de l'État (texte de la CMP)",
      summary: "L'Assemblée a adopté le texte commun de la commission mixte paritaire sur la gestion du patrimoine immobilier de l'État.",
      vote: vote8434, source: vote8434.source
    },
    {
      id: "s8427", chamber: "AN", timeLabel: "la veille", sortKey: 5,
      theme: "agriculture", type: "Scrutin — texte CMP",
      outcome: "adopte", outcomeLabel: "Adopté (texte CMP)",
      title: "Souveraineté agricole : la loi d'urgence a été adoptée la veille",
      dossierTitle: "Projet de loi d'urgence pour la protection et la souveraineté agricoles (texte de la CMP)",
      summary: "Adopté le 20 juillet (296 pour, 224 contre) : le texte de la commission mixte paritaire sur la protection et la souveraineté agricoles a été approuvé.",
      brefOnly: true, vote: vote8427, source: vote8427.source
    },
    {
      id: "decret-illus", chamber: "EXEC", timeLabel: "Suivi", sortKey: 1,
      theme: "institutions", type: "Application des lois",
      outcome: "info", outcomeLabel: "Suivi d'application",
      title: "Application des lois : indicateur de décrets en attente (illustratif)",
      dossierTitle: "Suivi des décrets d'application",
      summary: "Exemple de l'indicateur côté exécutif. Données à connecter aux échéanciers Légifrance — chiffres illustratifs.",
      brefOnly: true, illustrative: true,
      decrets: { published: 2, expected: 6, monthsElapsed: 7, monthsMedian: 5 },
      source: { tier: "acte", label: "Échéancier — Légifrance (à connecter)", url: "https://www.legifrance.gouv.fr/" }
    }
  ];

  /* --- System state (dashboard widgets) — navette/votes are plausible ------ */
  var systemState = {
    navette: [
      { id: "n1", title: "PJL réponses immédiates à l'ordre public", step: "Texte CMP adopté AN", next: "Sénat — lecture du texte CMP", chamberNext: "SENAT" },
      { id: "n2", title: "PPL mineurs & réseaux sociaux", step: "Texte CMP adopté AN", next: "Sénat — lecture du texte CMP", chamberNext: "SENAT" },
      { id: "n3", title: "PJL protection de l'enfance", step: "Adopté AN · 1re lecture", next: "Sénat — 1re lecture", chamberNext: "SENAT" }
    ],
    votesAttendus: [
      { id: "v1", chamber: "SENAT", when: "Cette semaine", what: "Lecture du texte CMP — ordre public", kind: "solennel" },
      { id: "v2", chamber: "AN", when: "À programmer", what: "Explications de vote — protection de l'enfance", kind: "solennel" }
    ],
    decretsRetard: { published: 2, expected: 6, monthsElapsed: 7, monthsMedian: 5, law: "Indicateur illustratif (Légifrance)", illustrative: true }
  };

  /* --- Edition ------------------------------------------------------------- */
  var edition = {
    number: 21072026,
    coversISO: "2026-07-21",
    coversLabel: "mardi 21 juillet 2026",
    publishedLabel: "mercredi 22 juillet 2026, 9 h 00",
    updatedLabel: "22 juil. 2026, 08 h 57",
    dayMedians: dayMedians,
    leadRule: "scrutin sur l'ensemble d'un texte > adoption définitive > marge la plus serrée > diversité des thèmes"
  };

  var data = {
    edition: edition,
    parliamentSitting: true,
    liveSession: liveSession,
    provenanceTiers: provenanceTiers,
    themes: themes,
    groups: { AN: groupsAN },
    groupMap: { AN: ANm },
    events: events,
    seances: { "an-ordre-public": seanceOrdrePublic },
    systemState: systemState
  };

  /* =========================================================================
     fmt — formatting helpers (never a figure without its baseline)
     ========================================================================= */
  var fmt = {
    chamberLabel: function (c) {
      return c === "AN" ? "Assemblée nationale" : c === "SENAT" ? "Sénat" : c === "EXEC" ? "Exécutif" : c;
    },
    chamberShort: function (c) {
      return c === "AN" ? "AN" : c === "SENAT" ? "Sénat" : c === "EXEC" ? "JO" : c;
    },
    outcomeLabel: function (o) {
      return { adopte: "Adopté", rejete: "Rejeté", reporte: "Reporté", info: "Info" }[o] || o;
    },
    theme: function (id) { return themes[id] || id; },
    pct: function (n, d) { return Math.round((n / d) * 100); },
    // Derived participation numbers for a scrutin
    part: function (v) {
      var votants = v.votants != null ? v.votants : (v.pour + v.contre + v.abstention);
      var absent = v.membersTotal - votants - (v.nonVotants || 0);
      return {
        votants: votants,
        exprimes: v.exprimes != null ? v.exprimes : (v.pour + v.contre),
        nonVotants: v.nonVotants || 0,
        absent: Math.max(0, absent),
        members: v.membersTotal,
        pct: fmt.pct(votants, v.membersTotal)
      };
    }
  };

  /* =========================================================================
     ui — shared components (identical HTML/SVG across variants → fair compare)
     ========================================================================= */
  var ui = {};

  ui.escape = function (s) {
    return String(s).replace(/[&<>"]/g, function (c) {
      return { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c];
    });
  };

  ui.chamberTag = function (c) {
    return '<span class="tag tag--chamber" data-ch="' + c + '">' + fmt.chamberShort(c) + "</span>";
  };
  ui.themeTag = function (id) {
    return '<span class="tag tag--theme">' + ui.escape(fmt.theme(id)) + "</span>";
  };
  ui.outcomeChip = function (outcome, label) {
    return '<span class="chip chip--' + outcome + '">' + ui.escape(label || fmt.outcomeLabel(outcome)) + "</span>";
  };
  ui.illusTag = function () {
    return '<span class="tag tag--illus" title="Données non encore connectées à la source — valeurs illustratives">illustratif</span>';
  };
  ui.provenance = function (src) {
    if (!src) return "";
    var t = provenanceTiers[src.tier];
    var n = t ? t.n : "?", name = t ? t.label : src.tier;
    var inner = '<span class="prov__dot">' + n + "</span><span class=\"prov__label\">" + ui.escape(src.label || name) + "</span>";
    return src.url
      ? '<a class="prov" href="' + src.url + '" title="' + ui.escape(name) + ' — ouvrir la source">' + inner + "</a>"
      : '<span class="prov">' + inner + "</span>";
  };
  ui.stat = function (value, unit, baseline) {
    var b = baseline ? '<span class="stat__base">' + ui.escape(baseline) + "</span>" : "";
    return '<span class="stat"><span class="stat__val">' + ui.escape(value) +
           (unit ? '<span class="stat__unit">' + ui.escape(unit) + "</span>" : "") + "</span>" + b + "</span>";
  };

  /* --- Vote colours (semantic; text label always present alongside) -------- */
  var VOTE = {
    pour:  { cls: "v-pour",  label: "pour" },
    contre:{ cls: "v-contre",label: "contre" },
    abst:  { cls: "v-abst",  label: "abst." },
    nv:    { cls: "v-nv",    label: "non-votant·es" },
    absent:{ cls: "v-absent",label: "absent·es" }
  };

  /* --- Participation line: votes cast vs seats that could have voted ------- */
  ui.participation = function (v) {
    var p = fmt.part(v);
    var med = data.edition.dayMedians.votants;
    return '<div class="part">' +
      '<span class="part__main">' + ui.stat(p.votants + " / " + p.members, " votant·es",
        "médiane du jour : " + med + " · " + p.pct + " % des sièges") + "</span>" +
      '<span class="part__gap">' + p.absent + " absent·es · " + p.nonVotants + " non-votant·es</span>" +
      "</div>";
  };

  /* --- Linear vote meter: spans ALL seats so the gap (absent) is visible --- */
  ui.voteMeter = function (v) {
    var p = fmt.part(v);
    var seg = [
      { k: "pour",   n: v.pour },
      { k: "contre", n: v.contre },
      { k: "abst",   n: v.abstention },
      { k: "nv",     n: p.nonVotants },
      { k: "absent", n: p.absent }
    ];
    var bars = seg.map(function (x) {
      if (x.n <= 0) return "";
      return '<span class="seg ' + VOTE[x.k].cls + '" style="flex:' + x.n + '" title="' + x.n + ' ' + VOTE[x.k].label + '"></span>';
    }).join("");
    return '<div class="meter"><div class="meter__bar">' + bars + "</div>" +
      '<div class="meter__legend">' +
        '<span class="ml ml--pour"><b>' + v.pour + "</b> pour</span>" +
        '<span class="ml ml--contre"><b>' + v.contre + "</b> contre</span>" +
        '<span class="ml ml--abst"><b>' + v.abstention + "</b> abst.</span>" +
        (p.absent ? '<span class="ml is-ghost"><b>' + p.absent + "</b> absent·es</span>" : "") +
      "</div></div>";
  };

  /* --- Seat layout for the dot hémicycle ----------------------------------
     Produces N seat coordinates in a semicircle, ordered left → right.       */
  function seatLayout(N, rInner, rOuter, cx, cy) {
    var rows = Math.max(5, Math.round(0.55 * Math.sqrt(N)));
    var radii = [];
    for (var i = 0; i < rows; i++) radii.push(rInner + (rOuter - rInner) * (rows === 1 ? 0 : i / (rows - 1)));
    var sumR = radii.reduce(function (a, b) { return a + b; }, 0);
    var counts = radii.map(function (r) { return Math.max(1, Math.round(N * r / sumR)); });
    var diff = N - counts.reduce(function (a, b) { return a + b; }, 0);
    var idx = rows - 1;
    while (diff !== 0) { counts[idx] += diff > 0 ? 1 : -1; diff += diff > 0 ? -1 : 1; idx = (idx - 1 + rows) % rows; }
    var seats = [];
    for (var r = 0; r < rows; r++) {
      var c = counts[r], rad = radii[r];
      for (var j = 0; j < c; j++) {
        var t = c === 1 ? 0.5 : j / (c - 1);
        var ang = 180 - t * 180;                       // 180° = left, 0° = right
        var a = ang * Math.PI / 180;
        seats.push({ ang: ang, x: cx + rad * Math.cos(a), y: cy - rad * Math.sin(a) });
      }
    }
    seats.sort(function (p, q) { return q.ang - p.ang || p.y - q.y; }); // left → right
    return seats;
  }

  /* --- Dot hémicycle: one dot per seat.
     colour = vote (pour/contre/abst/nv/absent) with absent hollow (the gap);
     position left→right = allegiance; a toggle recolours by group.
     Returns SVG; the container carries class hemi--vote (default) or
     hemi--groupes and per-seat CSS vars/attrs so the toggle is pure CSS.      */
  ui.hemicycleDots = function (v, opts) {
    opts = opts || {};
    var chamber = v.chamber || "AN";
    var groups = data.groups[chamber];
    var W = opts.width || 320;
    var rOuter = opts.outerR || W * 0.46;
    var rInner = opts.innerR || W * 0.20;
    var cx = W / 2, pad = 6, H = rOuter + pad * 2, cy = H - pad;
    var dot = opts.dot || 3;

    // Build the ordered vote list, group by group (contiguous L→R blocks).
    // Within a group: pour, abstention, contre, non-votant, absent.
    var flat = [];
    groups.forEach(function (g) {
      var b = (v.breakdown && v.breakdown[g.id]) || [0, 0, 0, 0];
      var pour = b[0], contre = b[1], abst = b[2], nv = b[3];
      var absent = Math.max(0, g.members - pour - contre - abst - nv);
      function push(k, n) { for (var i = 0; i < n; i++) flat.push({ g: g, k: k }); }
      push("pour", pour); push("abst", abst); push("contre", contre);
      push("nv", nv); push("absent", absent);
    });

    var seats = seatLayout(flat.length, rInner, rOuter, cx, cy);
    var body = seats.map(function (s, i) {
      var f = flat[i]; if (!f) return "";
      return '<circle class="seat" data-vote="' + f.k + '" data-group="' + f.g.id +
        '" style="--gc:' + f.g.color + '" cx="' + s.x.toFixed(1) + '" cy="' + s.y.toFixed(1) +
        '" r="' + dot + '"><title>' + ui.escape(f.g.id + " — " + VOTE[f.k].label) + "</title></circle>";
    }).join("");

    var uid = "h" + (v.scrutinNo || Math.round(cx));
    return '<div class="hemi-wrap hemi--vote" id="' + uid + '">' +
      '<svg class="hemi" viewBox="0 0 ' + W + " " + H.toFixed(0) +
        '" role="img" aria-label="Hémicycle : chaque siège coloré selon son vote, groupes de gauche à droite">' + body + "</svg>" +
      '</div>';
  };

  /* --- Legends for the dot hémicycle (vote + group), and toggle wiring ----- */
  ui.hemicycleLegends = function (v) {
    var chamber = v.chamber || "AN";
    var groups = data.groups[chamber];
    var voteL = ['<span class="lg lg--pour"><i class="sw v-pour"></i>pour</span>',
                 '<span class="lg lg--contre"><i class="sw v-contre"></i>contre</span>',
                 '<span class="lg lg--abst"><i class="sw v-abst"></i>abstention</span>',
                 '<span class="lg lg--nv"><i class="sw v-nv"></i>non-votant·es</span>',
                 '<span class="lg lg--absent"><i class="sw v-absent"></i>absent·es</span>'].join("");
    // Group labels take their own party colour, darkened toward ink for legibility.
    var grpL = groups.map(function (g) {
      return '<span class="lg" style="color:color-mix(in srgb, ' + g.color + ' 72%, #1B1E26)">' +
             '<i class="sw" style="background:' + g.color + '"></i>' + g.id + "</span>";
    }).join("");
    return '<div class="hemi-legend"><div class="lg-set lg-vote">' + voteL + "</div>" +
           '<div class="lg-set lg-grp">' + grpL + "</div></div>";
  };
  // Attach toggle behaviour to a container holding hemicycle + [data-hemi-toggle] buttons.
  ui.bindHemiToggle = function (scope) {
    scope.querySelectorAll("[data-hemi-toggle]").forEach(function (btn) {
      btn.addEventListener("click", function () {
        var mode = btn.getAttribute("data-hemi-toggle"); // "vote" | "groupes"
        var wrap = scope.querySelector(".hemi-wrap");
        if (wrap) { wrap.classList.toggle("hemi--vote", mode === "vote"); wrap.classList.toggle("hemi--groupes", mode === "groupes"); }
        scope.classList.toggle("show-grp", mode === "groupes");
        scope.querySelectorAll("[data-hemi-toggle]").forEach(function (b) { b.setAttribute("aria-pressed", b === btn ? "true" : "false"); });
      });
    });
  };

  /* --- Séance timeline bar: debate segments & suspension gaps -------------- */
  ui.seanceBar = function (s) {
    var parts = [];
    var debateChunk = (s.wallMinutes - s.suspensionMinutes) / (s.suspensions.length + 1);
    s.suspensions.forEach(function (sp) {
      parts.push({ cls: "tl--debate", min: debateChunk });
      parts.push({ cls: "tl--susp", min: sp.minutes, by: sp.by });
    });
    parts.push({ cls: "tl--debate", min: debateChunk });
    var bar = parts.map(function (p) {
      return '<span class="' + p.cls + '" style="flex:' + p.min.toFixed(1) + '"' +
        (p.by ? ' title="Suspension ' + ui.escape(p.by) + ' — ' + p.min + ' min"' : "") + "></span>";
    }).join("");
    var pct = fmt.pct(s.suspensionMinutes, s.wallMinutes);
    return '<div class="tl"><div class="tl__bar">' + bar + "</div>" +
      '<div class="tl__legend"><span class="tl__k tl--debate"></span>Débat ' +
      '<span class="tl__k tl--susp"></span>Suspension — ' +
      ui.stat(pct + " %", "", "médiane : " + s.suspensionMedianPct + " %") + " du temps</div></div>";
  };

  window.LINDEX = { data: data, fmt: fmt, ui: ui };
})();
