// The app-side of the read-model contract. Mirrors db/migrations/0001_facts.sql:
// facts.read_scrutin(scrutin_id, chamber, title, outcome, totals, breakdown,
// baselines, provenance, updated_at). The jsonb columns arrive as `unknown`
// from Drizzle; the `as*` narrowing helpers validate their shape so a malformed
// row fails loudly instead of rendering NaN. No `any` — narrow from `unknown`.

export type GroupTally = {
  group: string;
  pour: number;
  contre: number;
  abst: number;
  nv: number;
};

export type ScrutinTotals = {
  pour: number;
  contre: number;
  abstention: number;
  nonVotants: number;
  membersTotal: number;
  votants: number;
};

export type ScrutinBaselines = {
  votants: number;
  abstention: number;
};

// Only what L0-DATA is contracted to emit. The display label is derived in the
// UI (never invent a field the data plane will not write).
export type ScrutinProvenance = {
  tier: string;
  source_record: string;
  url: string;
};

export type ReadScrutin = {
  scrutinId: string;
  chamber: string;
  title: string;
  outcome: string;
  totals: ScrutinTotals;
  breakdown: GroupTally[];
  baselines: ScrutinBaselines;
  provenance: ScrutinProvenance;
  updatedAt: Date;
};

// Provenance tiers — labels, never a score (project brief §4). n = tier number.
export const PROVENANCE_TIERS: Record<string, { n: number; label: string }> = {
  acte: { n: 1, label: "Acte authentique" },
  organisme: { n: 2, label: "Organisme public indépendant" },
  gouv: { n: 3, label: "Communication gouvernementale" },
  parlement: { n: 4, label: "Travaux parlementaires" },
  presse: { n: 5, label: "Presse & société civile" },
};

function isRecord(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null;
}

function num(source: Record<string, unknown>, key: string): number {
  const value = source[key];
  if (typeof value !== "number" || Number.isNaN(value)) {
    throw new Error(`read-model: expected numeric field "${key}"`);
  }
  return value;
}

function str(source: Record<string, unknown>, key: string): string {
  const value = source[key];
  if (typeof value !== "string") {
    throw new Error(`read-model: expected string field "${key}"`);
  }
  return value;
}

export function asTotals(v: unknown): ScrutinTotals {
  if (!isRecord(v)) throw new Error("read-model: totals is not an object");
  return {
    pour: num(v, "pour"),
    contre: num(v, "contre"),
    abstention: num(v, "abstention"),
    nonVotants: num(v, "nonVotants"),
    membersTotal: num(v, "membersTotal"),
    votants: num(v, "votants"),
  };
}

export function asBreakdown(v: unknown): GroupTally[] {
  if (!Array.isArray(v)) throw new Error("read-model: breakdown is not an array");
  return v.map((row) => {
    if (!isRecord(row)) throw new Error("read-model: breakdown row is not an object");
    return {
      group: str(row, "group"),
      pour: num(row, "pour"),
      contre: num(row, "contre"),
      abst: num(row, "abst"),
      nv: num(row, "nv"),
    };
  });
}

export function asBaselines(v: unknown): ScrutinBaselines {
  if (!isRecord(v)) throw new Error("read-model: baselines is not an object");
  return {
    votants: num(v, "votants"),
    abstention: num(v, "abstention"),
  };
}

export function asProvenance(v: unknown): ScrutinProvenance {
  if (!isRecord(v)) throw new Error("read-model: provenance is not an object");
  return {
    tier: str(v, "tier"),
    source_record: str(v, "source_record"),
    url: str(v, "url"),
  };
}
