"use client";

import { useMemo, useState } from "react";

// Faithful port of the mockup dot-hémicycle (mockups/data.js). One dot = one
// seat: colour = vote (or party in "Groupes" mode), position left→right =
// allegiance, absent seats hollow so the participation gap stays visible.

export type Tally = {
  group: string;
  pour: number;
  contre: number;
  abst: number;
  nv: number;
};

// AN 17th-legislature groups, hémicycle order left→right (id, colour, seats).
const GROUPS: { id: string; color: string; members: number }[] = [
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

const VOTE_COLOR: Record<string, string> = {
  pour: "#3C6E58",
  contre: "#9B4B3B",
  abst: "#8A7635",
  nv: "#B9B8B1",
};

type Seat = {
  x: number;
  y: number;
  kind: string;
  color: string;
};

function seatLayout(N: number, rInner: number, rOuter: number, cx: number, cy: number) {
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
      const ang = 180 - t * 180;
      const a = (ang * Math.PI) / 180;
      coords.push({ ang, x: cx + rad * Math.cos(a), y: cy - rad * Math.sin(a) });
    }
  }
  coords.sort((p, q) => q.ang - p.ang || p.y - q.y);
  return coords;
}

function buildSeats(breakdown: Tally[], mode: "vote" | "groupes") {
  const byId = new Map(breakdown.map((t) => [t.group, t]));
  const flat: { kind: string; color: string }[] = [];
  for (const g of GROUPS) {
    const t = byId.get(g.id) ?? { group: g.id, pour: 0, contre: 0, abst: 0, nv: 0 };
    const absent = Math.max(0, g.members - t.pour - t.contre - t.abst - t.nv);
    const push = (kind: string, n: number) => {
      const color = mode === "groupes" ? g.color : VOTE_COLOR[kind] ?? "#fff";
      for (let i = 0; i < n; i++) flat.push({ kind, color });
    };
    push("pour", t.pour);
    push("abst", t.abst);
    push("contre", t.contre);
    push("nv", t.nv);
    push("absent", absent);
  }
  const W = 320;
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

export function Hemicycle({ breakdown }: { breakdown: Tally[] }) {
  const [mode, setMode] = useState<"vote" | "groupes">("vote");
  const { seats, W, H } = useMemo(() => buildSeats(breakdown, mode), [breakdown, mode]);

  return (
    <figure className="hemi">
      <div className="hemi-toggle" role="group" aria-label="Colorer par">
        <button aria-pressed={mode === "vote"} onClick={() => setMode("vote")}>
          Vote
        </button>
        <button aria-pressed={mode === "groupes"} onClick={() => setMode("groupes")}>
          Groupes
        </button>
      </div>
      <svg
        viewBox={`0 0 ${W} ${H}`}
        role="img"
        aria-label="Hémicycle : un siège coloré selon son vote, groupes de gauche à droite"
      >
        {seats.map((s, i) => {
          const absent = s.kind === "absent";
          return (
            <circle
              key={i}
              cx={s.x}
              cy={s.y}
              r={3}
              fill={absent ? "#fff" : s.color}
              stroke={absent ? "#B9B8B1" : "none"}
              strokeWidth={absent ? 1 : 0}
              opacity={absent ? 0.5 : 1}
            />
          );
        })}
      </svg>
    </figure>
  );
}
