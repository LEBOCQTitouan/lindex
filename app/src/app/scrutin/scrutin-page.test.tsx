import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";

// Integration test: drive the REAL page server component with a mocked
// facts.read_scrutin row (8433, ADR-0001 wire shapes) and assert the rendered
// HTML. Proves the page renders totals, the participation gap + baseline, the
// provenance chip and the hémicycle island entirely from the read-model — the
// live-DB render check is otherwise gated on Docker, unavailable in CI.

vi.mock("drizzle-orm", () => ({ eq: () => ({}) }));

vi.mock("@/lib/db", () => {
  const row = {
    scrutinId: "8433",
    chamber: "AN",
    title:
      "Projet de loi visant à offrir des réponses immédiates aux phénomènes troublant l'ordre public (texte de la CMP)",
    heldOn: "2026-07-21",
    outcome: "adopte",
    totals: {
      pour: 351,
      contre: 179,
      abstention: 7,
      nonVotants: 2,
      membersTotal: 577,
      votants: 537,
      exprimes: 530,
    },
    breakdown: [
      { group: "LFI", pour: 0, contre: 71, abstention: 0, nonVotant: 0 },
      { group: "RN", pour: 114, contre: 0, abstention: 0, nonVotant: 0 },
    ],
    baselines: {
      votants: { median: 547.5, sampleSize: 2 },
      abstention: { median: 90, sampleSize: 2 },
      method: { id: "scrutin-day-median", version: 1 },
    },
    provenance: {
      tier: "ActeAuthentique",
      label: "Scrutin n° 8433 — AN",
      url: "https://www.assemblee-nationale.fr/dyn/17/scrutins/8433",
      recordId: "an-scrutin-8433",
      retrievedAt: "2026-07-22T06:57:00+00:00",
    },
    updatedAt: new Date("2026-07-22T06:57:00Z"),
  };
  const q = {
    select: () => q,
    from: () => q,
    where: () => q,
    limit: () => Promise.resolve([row]),
  };
  return { db: q, schema: { readScrutin: { scrutinId: "scrutin_id" } } };
});

// eslint-disable-next-line import/first
import ScrutinPage from "./[id]/page";

describe("ScrutinPage", () => {
  it("renders totals, participation gap, baseline, provenance and the hémicycle from the read-model", async () => {
    const element = await ScrutinPage({ params: Promise.resolve({ id: "8433" }) });
    const html = renderToStaticMarkup(element);

    // Totals + outcome
    expect(html).toContain("Adopté");
    expect(html).toContain("ordre public");
    // Participation gap: votes cast vs 577 seats, with its baseline (P1)
    expect(html).toContain("537 / 577");
    expect(html).toContain("médiane du jour : 547.5");
    expect(html).toContain("93 % des sièges");
    expect(html).toContain("38 absent·es · 2 non-votant·es");
    // Abstention is a displayed figure → it must ship with its baseline (P1).
    expect(html).toContain("abstentions");
    expect(html).toContain("médiane du jour : 90");
    // Provenance chip → source link, using the contract's label (P4)
    expect(html).toContain("https://www.assemblee-nationale.fr/dyn/17/scrutins/8433");
    expect(html).toContain("Scrutin n° 8433 — AN");
    // Hémicycle island + toggle
    expect(html).toContain("<svg");
    expect(html).toContain(">Vote<");
    expect(html).toContain(">Groupes<");
  });
});
