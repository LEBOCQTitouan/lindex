"use client";

import { useMemo, useState } from "react";
import { buildSeats, GROUPS_AN, type SeatMode } from "@/lib/hemicycle";
import type { GroupTally } from "@/lib/read-model";

// Faithful port of the mockup dot-hémicycle (mockups/data.js, d-hybride.html).
// One dot = one seat: colour = vote (or party in "Groupes" mode), position
// left→right = allegiance, absent seats hollow so the participation gap stays
// visible. Geometry lives in lib/hemicycle (unit-tested); this is the renderer.

const VOTE_LEGEND: { kind: string; label: string }[] = [
  { kind: "pour", label: "pour" },
  { kind: "contre", label: "contre" },
  { kind: "abst", label: "abstention" },
  { kind: "nv", label: "non-votant·es" },
  { kind: "absent", label: "absent·es" },
];

export function Hemicycle({ breakdown }: { breakdown: GroupTally[] }) {
  const [mode, setMode] = useState<SeatMode>("vote");
  const { seats, W, H } = useMemo(() => buildSeats(breakdown, mode), [breakdown, mode]);

  return (
    <figure className={`hemi-scope${mode === "groupes" ? " show-grp" : ""}`}>
      <div className="hemi-block__head">
        <span className="hemi-block__t">Hémicycle · un siège = un·e député·e</span>
        <span className="hemi-toggle" role="group" aria-label="Colorer l'hémicycle par">
          <button
            type="button"
            aria-pressed={mode === "vote"}
            onClick={() => setMode("vote")}
          >
            Vote
          </button>
          <button
            type="button"
            aria-pressed={mode === "groupes"}
            onClick={() => setMode("groupes")}
          >
            Groupes
          </button>
        </span>
      </div>

      <div className="hemi-wrap">
        <svg
          className="hemi"
          viewBox={`0 0 ${W} ${H}`}
          role="img"
          aria-label={
            mode === "vote"
              ? "Hémicycle : chaque siège coloré selon son vote, groupes de gauche à droite"
              : "Hémicycle : chaque siège coloré selon son groupe, de gauche à droite"
          }
        >
          {seats.map((s, i) => {
            const absent = s.kind === "absent";
            return (
              <circle
                key={i}
                className="seat"
                cx={s.x}
                cy={s.y}
                r={3}
                fill={absent ? "#ffffff" : s.color}
                stroke={absent ? "#B9B8B1" : "none"}
                strokeWidth={absent ? 1 : 0}
                opacity={absent ? 0.5 : 1}
              />
            );
          })}
        </svg>
      </div>

      <div className="hemi-legend">
        <div className="lg-set lg-vote">
          {VOTE_LEGEND.map((v) => (
            <span key={v.kind} className={`lg lg--${v.kind}`}>
              <i className={`sw v-${v.kind}`} />
              {v.label}
            </span>
          ))}
        </div>
        <div className="lg-set lg-grp">
          {GROUPS_AN.map((g) => (
            <span
              key={g.id}
              className="lg"
              style={{ color: `color-mix(in srgb, ${g.color} 72%, #1B1E26)` }}
            >
              <i className="sw" style={{ background: g.color }} />
              {g.id}
            </span>
          ))}
        </div>
      </div>
    </figure>
  );
}
