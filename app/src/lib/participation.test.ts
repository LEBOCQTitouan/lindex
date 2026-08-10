import { describe, expect, it } from "vitest";
import { meterSegments, participation } from "./participation";
import type { ScrutinTotals } from "./read-model";

// Scrutin 8433 (ordre public, texte CMP) — verbatim from mockups/data.js.
const T_8433: ScrutinTotals = {
  pour: 351,
  contre: 179,
  abstention: 7,
  nonVotants: 2,
  membersTotal: 577,
  votants: 537,
  exprimes: 530,
};

describe("participation", () => {
  it("derives the participation gap for 8433", () => {
    const p = participation(T_8433);
    expect(p).toEqual({
      votants: 537,
      members: 577,
      absent: 38, // 577 − 537 votants − 2 non-votants
      nonVotants: 2,
      exprimes: 530, // pour + contre
      pct: 93, // round(537 / 577 * 100)
    });
  });

  it("never returns a negative absent count", () => {
    const full: ScrutinTotals = {
      pour: 300,
      contre: 277,
      abstention: 0,
      nonVotants: 0,
      membersTotal: 577,
      votants: 577,
      exprimes: 577,
    };
    expect(participation(full).absent).toBe(0);
  });
});

describe("meterSegments", () => {
  it("spans all seats so the gap is visible", () => {
    const segs = meterSegments(T_8433);
    const total = segs.reduce((sum, s) => sum + s.n, 0);
    expect(total).toBe(577);
    expect(segs.map((s) => s.kind)).toEqual([
      "pour",
      "contre",
      "abst",
      "nv",
      "absent",
    ]);
    expect(segs.find((s) => s.kind === "absent")?.n).toBe(38);
  });
});
