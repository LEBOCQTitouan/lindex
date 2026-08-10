import type { GroupTally } from "./read-model";

// Pure hémicycle geometry, ported verbatim from mockups/data.js (ui.hemicycleDots
// + seatLayout). Kept React-free so the seat-conservation invariants are unit
// testable and the island stays a thin renderer. One dot = one seat; colour =
// vote (or party in "groupes" mode); position left→right = allegiance; unfilled
// seats are marked "absent" so the participation gap stays visible.

// AN 17th-legislature groups, hémicycle order left→right (id, colour, seats).
// Reference data (577 seats), not per-scrutin — identical for every AN vote.
// AN-ONLY: seat count and per-group `absent` derive from this table, so this
// renders a 577-seat Assemblée for any row. A Sénat read_scrutin (348 seats)
// needs its own group reference before it can be drawn (future L1-SENAT lane).
export const GROUPS_AN: { id: string; color: string; members: number }[] = [
  { id: "LFI", color: "#C4002E", members: 71 },
  { id: "GDR", color: "#B5121B", members: 17 },
  { id: "ECOS", color: "#4CA85B", members: 38 },
  { id: "SOC", color: "#E63C8C", members: 68 },
  { id: "LIOT", color: "#E0A02A", members: 23 },
  { id: "DEM", color: "#F08C00", members: 37 },
  { id: "EPR", color: "#D4B106", members: 91 },
  { id: "HOR", color: "#12A5A0", members: 35 },
  { id: "DR", color: "#2E6FB5", members: 48 },
  { id: "UDR", color: "#1D3D8F", members: 17 },
  { id: "RN", color: "#22326B", members: 122 },
  { id: "NI", color: "#8A8D94", members: 10 },
];

// Muted vote palette (matches the mockup CSS custom properties).
export const VOTE_COLOR: Record<string, string> = {
  pour: "#3C6E58",
  contre: "#9B4B3B",
  abst: "#8A7635",
  nv: "#B9B8B1",
};

export type SeatMode = "vote" | "groupes";
export type Seat = { x: number; y: number; kind: string; color: string };

function seatLayout(
  N: number,
  rInner: number,
  rOuter: number,
  cx: number,
  cy: number,
): { x: number; y: number; ang: number }[] {
  const rows = Math.max(5, Math.round(0.55 * Math.sqrt(N)));
  const radii: number[] = [];
  for (let i = 0; i < rows; i++) {
    radii.push(rInner + (rOuter - rInner) * (rows === 1 ? 0 : i / (rows - 1)));
  }
  const sumR = radii.reduce((a, b) => a + b, 0);
  const counts = radii.map((r) => Math.max(1, Math.round((N * r) / sumR)));
  let diff = N - counts.reduce((a, b) => a + b, 0);
  let idx = rows - 1;
  while (diff !== 0) {
    counts[idx] += diff > 0 ? 1 : -1;
    diff += diff > 0 ? -1 : 1;
    idx = (idx - 1 + rows) % rows;
  }
  const coords: { x: number; y: number; ang: number }[] = [];
  for (let r = 0; r < rows; r++) {
    const c = counts[r];
    const rad = radii[r];
    for (let j = 0; j < c; j++) {
      const t = c === 1 ? 0.5 : j / (c - 1);
      const ang = 180 - t * 180; // 180° = left, 0° = right
      const a = (ang * Math.PI) / 180;
      coords.push({ ang, x: cx + rad * Math.cos(a), y: cy - rad * Math.sin(a) });
    }
  }
  coords.sort((p, q) => q.ang - p.ang || p.y - q.y); // left → right
  return coords;
}

export function buildSeats(
  breakdown: GroupTally[],
  mode: SeatMode,
  opts: { width?: number } = {},
): { seats: Seat[]; W: number; H: number } {
  const byId = new Map(breakdown.map((t) => [t.group, t]));
  const flat: { kind: string; color: string }[] = [];
  for (const g of GROUPS_AN) {
    const t = byId.get(g.id) ?? { group: g.id, pour: 0, contre: 0, abstention: 0, nonVotant: 0 };
    const absent = Math.max(0, g.members - t.pour - t.contre - t.abstention - t.nonVotant);
    const push = (kind: string, n: number) => {
      const color = mode === "groupes" ? g.color : (VOTE_COLOR[kind] ?? "#ffffff");
      for (let i = 0; i < n; i++) flat.push({ kind, color });
    };
    // Within a group: pour, abstention, contre, non-votant, absent.
    push("pour", t.pour);
    push("abst", t.abstention);
    push("contre", t.contre);
    push("nv", t.nonVotant);
    push("absent", absent);
  }
  const W = opts.width ?? 320;
  const rOuter = W * 0.46;
  const rInner = W * 0.2;
  const cx = W / 2;
  const pad = 6;
  const H = rOuter + pad * 2;
  const cy = H - pad;
  const coords = seatLayout(flat.length, rInner, rOuter, cx, cy);
  const seats: Seat[] = coords.map((c, i) => ({ x: c.x, y: c.y, ...flat[i] }));
  return { seats, W, H };
}
