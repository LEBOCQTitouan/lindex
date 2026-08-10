import { describe, expect, it } from "vitest";
import {
  asBaselines,
  asBreakdown,
  asProvenance,
  asTotals,
  PROVENANCE_TIERS,
} from "./read-model";

// The jsonb columns of facts.read_scrutin arrive as `unknown` from Drizzle.
// These narrowing helpers are the app-side of the read-model contract: they
// accept exactly what db/migrations/0001_facts.sql documents and reject
// malformed rows loudly (rather than rendering NaN).

describe("asTotals", () => {
  it("accepts a well-formed totals object", () => {
    const t = asTotals({
      pour: 351,
      contre: 179,
      abstention: 7,
      nonVotants: 2,
      membersTotal: 577,
      votants: 537,
    });
    expect(t.pour).toBe(351);
    expect(t.membersTotal).toBe(577);
    expect(t.votants).toBe(537);
  });

  it("throws when a required numeric field is missing", () => {
    expect(() => asTotals({ pour: 1, contre: 2 })).toThrow();
  });
});

describe("asBreakdown", () => {
  it("maps an array of per-group tallies", () => {
    const b = asBreakdown([{ group: "LFI", pour: 0, contre: 71, abst: 0, nv: 0 }]);
    expect(b).toHaveLength(1);
    expect(b[0]).toEqual({ group: "LFI", pour: 0, contre: 71, abst: 0, nv: 0 });
  });

  it("throws when not an array", () => {
    expect(() => asBreakdown({ group: "LFI" })).toThrow();
  });
});

describe("asBaselines", () => {
  it("reads day medians", () => {
    expect(asBaselines({ votants: 426, abstention: 7 })).toEqual({
      votants: 426,
      abstention: 7,
    });
  });
});

describe("asProvenance", () => {
  it("reads {tier, source_record, url}", () => {
    const p = asProvenance({
      tier: "acte",
      source_record: "an-scrutin-8433",
      url: "https://example.test/8433",
    });
    expect(p.tier).toBe("acte");
    expect(p.url).toBe("https://example.test/8433");
  });
});

describe("PROVENANCE_TIERS", () => {
  it("labels the acte-authentique tier as tier 1", () => {
    expect(PROVENANCE_TIERS.acte.n).toBe(1);
    expect(PROVENANCE_TIERS.acte.label).toBe("Acte authentique");
  });
});
