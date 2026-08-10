import { describe, expect, it } from "vitest";
import {
  asBaselines,
  asBreakdown,
  asProvenance,
  asTotals,
  PROVENANCE_TIERS,
} from "./read-model";

// The jsonb columns of facts.read_scrutin arrive as `unknown` from Drizzle.
// These narrowing helpers are the app-side of the ADR-0001 wire contract: they
// accept exactly what L0-DATA's projection writes and reject malformed rows
// loudly (rather than rendering NaN).

describe("asTotals", () => {
  it("accepts a well-formed totals object", () => {
    const t = asTotals({
      pour: 351,
      contre: 179,
      abstention: 7,
      nonVotants: 2,
      membersTotal: 577,
      votants: 537,
      exprimes: 530,
    });
    expect(t.membersTotal).toBe(577);
    expect(t.votants).toBe(537);
    expect(t.exprimes).toBe(530);
  });

  it("throws when a required numeric field is missing", () => {
    expect(() => asTotals({ pour: 1, contre: 2 })).toThrow();
  });
});

describe("asBreakdown", () => {
  it("maps an array of per-group tallies", () => {
    const b = asBreakdown([
      { group: "LFI", pour: 0, contre: 71, abstention: 0, nonVotant: 0 },
    ]);
    expect(b).toHaveLength(1);
    expect(b[0]).toEqual({ group: "LFI", pour: 0, contre: 71, abstention: 0, nonVotant: 0 });
  });

  it("throws when not an array", () => {
    expect(() => asBreakdown({ group: "LFI" })).toThrow();
  });
});

describe("asBaselines", () => {
  it("reads nested day medians with method", () => {
    expect(
      asBaselines({
        votants: { median: 547.5, sampleSize: 2 },
        abstention: { median: 90, sampleSize: 2 },
        method: { id: "scrutin-day-median", version: 1 },
      }),
    ).toEqual({
      votants: { median: 547.5, sampleSize: 2 },
      abstention: { median: 90, sampleSize: 2 },
      method: { id: "scrutin-day-median", version: 1 },
    });
  });

  it("throws when a baseline is a bare number (old shape)", () => {
    expect(() => asBaselines({ votants: 426, abstention: 7 })).toThrow();
  });
});

describe("asProvenance", () => {
  it("reads {tier, label, url, recordId, retrievedAt}", () => {
    const p = asProvenance({
      tier: "ActeAuthentique",
      label: "Scrutin n° 8433 — AN",
      url: "https://example.test/8433",
      recordId: "an-scrutin-8433",
      retrievedAt: "2026-07-22T06:57:00+00:00",
    });
    expect(p.tier).toBe("ActeAuthentique");
    expect(p.label).toBe("Scrutin n° 8433 — AN");
    expect(p.url).toBe("https://example.test/8433");
    expect(p.recordId).toBe("an-scrutin-8433");
  });

  it("accepts a null url (the contract allows it)", () => {
    const p = asProvenance({
      tier: "ActeAuthentique",
      label: "x",
      url: null,
      recordId: "r",
      retrievedAt: "t",
    });
    expect(p.url).toBeNull();
  });
});

describe("PROVENANCE_TIERS", () => {
  it("labels the acte-authentique tier (domain enum name) as tier 1", () => {
    expect(PROVENANCE_TIERS.ActeAuthentique.n).toBe(1);
    expect(PROVENANCE_TIERS.ActeAuthentique.label).toBe("Acte authentique");
  });
});
