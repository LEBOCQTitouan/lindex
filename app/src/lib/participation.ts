import type { ScrutinTotals } from "./read-model";

// Participation math, mirroring `fmt.part` and `ui.voteMeter` in mockups/data.js.
// The point of the whole page: show votes cast against the seats that *could*
// have voted, so the gap (absent + non-votants) is legible, not hidden.

export type Participation = {
  votants: number;
  members: number;
  absent: number;
  nonVotants: number;
  exprimes: number;
  pct: number;
};

export type VoteKind = "pour" | "contre" | "abst" | "nv" | "absent";
export type MeterSegment = { kind: VoteKind; n: number };

export function participation(t: ScrutinTotals): Participation {
  const absent = Math.max(0, t.membersTotal - t.votants - t.nonVotants);
  return {
    votants: t.votants,
    members: t.membersTotal,
    absent,
    nonVotants: t.nonVotants,
    exprimes: t.pour + t.contre,
    pct: Math.round((t.votants / t.membersTotal) * 100),
  };
}

// Segments span ALL seats (incl. the hollow "absent" gap) → they sum to
// membersTotal. Order is fixed pour → contre → abst → nv → absent.
export function meterSegments(t: ScrutinTotals): MeterSegment[] {
  const { absent } = participation(t);
  return [
    { kind: "pour", n: t.pour },
    { kind: "contre", n: t.contre },
    { kind: "abst", n: t.abstention },
    { kind: "nv", n: t.nonVotants },
    { kind: "absent", n: absent },
  ];
}
