import { describe, expect, it } from "vitest";
import { buildSeats, GROUPS_AN } from "./hemicycle";
import type { GroupTally } from "./read-model";

// Scrutin 8433 breakdown — verbatim from mockups/data.js (hémicycle order L→R).
const BD_8433: GroupTally[] = [
  { group: "LFI", pour: 0, contre: 71, abst: 0, nv: 0 },
  { group: "GDR", pour: 0, contre: 15, abst: 0, nv: 0 },
  { group: "ECOS", pour: 0, contre: 36, abst: 0, nv: 0 },
  { group: "SOC", pour: 0, contre: 56, abst: 1, nv: 0 },
  { group: "LIOT", pour: 18, contre: 1, abst: 4, nv: 0 },
  { group: "DEM", pour: 33, contre: 0, abst: 0, nv: 0 },
  { group: "EPR", pour: 84, contre: 0, abst: 0, nv: 1 },
  { group: "HOR", pour: 32, contre: 0, abst: 0, nv: 1 },
  { group: "DR", pour: 46, contre: 0, abst: 1, nv: 0 },
  { group: "UDR", pour: 17, contre: 0, abst: 0, nv: 0 },
  { group: "RN", pour: 114, contre: 0, abst: 0, nv: 0 },
  { group: "NI", pour: 7, contre: 0, abst: 1, nv: 0 },
];

describe("GROUPS_AN", () => {
  it("covers all 577 seats of the Assemblée", () => {
    const total = GROUPS_AN.reduce((sum, g) => sum + g.members, 0);
    expect(total).toBe(577);
  });
});

describe("buildSeats", () => {
  it("produces one seat per member — 577 total", () => {
    expect(buildSeats(BD_8433, "vote").seats).toHaveLength(577);
  });

  it("gives every seat a colour in both modes", () => {
    for (const mode of ["vote", "groupes"] as const) {
      const { seats } = buildSeats(BD_8433, mode);
      expect(seats.every((s) => typeof s.color === "string" && s.color.length > 0)).toBe(true);
    }
  });

  it("marks unfilled seats absent: 577 − votes cast = 38 for 8433", () => {
    const { seats } = buildSeats(BD_8433, "vote");
    const absent = seats.filter((s) => s.kind === "absent").length;
    expect(absent).toBe(38);
  });

  it("recolours by party in groupes mode without changing the seat count", () => {
    const voteSeats = buildSeats(BD_8433, "vote").seats;
    const grpSeats = buildSeats(BD_8433, "groupes").seats;
    expect(grpSeats).toHaveLength(voteSeats.length);
  });
});
