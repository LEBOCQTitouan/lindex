// The app-side of the read-model contract. Mirrors db/migrations/0001_facts.sql
// and the wire shapes pinned in docs/decisions/0001-scrutin-read-model-contract.md
// (ADR-0001), which L0-DATA's projection writes. The jsonb columns arrive as
// `unknown` from Drizzle; the `as*` narrowing helpers validate their shape so a
// malformed row fails loudly instead of rendering NaN. No `any` — narrow from
// `unknown`.

export type GroupTally = {
  group: string;
  pour: number;
  contre: number;
  abstention: number;
  nonVotant: number;
};

export type ScrutinTotals = {
  pour: number;
  contre: number;
  abstention: number;
  nonVotants: number;
  membersTotal: number;
  votants: number;
  exprimes: number;
};

// A day-median baseline and the size of the cohort it was taken over (ADR-0001).
export type Baseline = { median: number; sampleSize: number };
export type ScrutinBaselines = {
  votants: Baseline;
  abstention: Baseline;
  method: { id: string; version: number };
};

// `url` is nullable in the contract; `tier` is the domain enum name (PascalCase).
export type ScrutinProvenance = {
  tier: string;
  label: string;
  url: string | null;
  recordId: string;
  retrievedAt: string;
};

export type ReadScrutin = {
  scrutinId: string;
  chamber: string;
  title: string;
  heldOn: string;
  outcome: string;
  totals: ScrutinTotals;
  breakdown: GroupTally[];
  baselines: ScrutinBaselines;
  provenance: ScrutinProvenance;
  updatedAt: Date;
};

// Provenance tiers — labels, never a score (project brief §4). Keyed by the
// domain enum name serialized in `provenance.tier` (usecase.rs `tier_name`).
export const PROVENANCE_TIERS: Record<string, { n: number; label: string }> = {
  ActeAuthentique: { n: 1, label: "Acte authentique" },
  OrganismeIndependant: { n: 2, label: "Organisme public indépendant" },
  CommunicationGouv: { n: 3, label: "Communication gouvernementale" },
  TravauxParlementaires: { n: 4, label: "Travaux parlementaires" },
  PresseSocieteCivile: { n: 5, label: "Presse & société civile" },
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

function strOrNull(source: Record<string, unknown>, key: string): string | null {
  const value = source[key];
  if (value === null || value === undefined) return null;
  if (typeof value !== "string") {
    throw new Error(`read-model: expected string or null field "${key}"`);
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
    exprimes: num(v, "exprimes"),
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
      abstention: num(row, "abstention"),
      nonVotant: num(row, "nonVotant"),
    };
  });
}

function asBaseline(v: unknown, what: string): Baseline {
  if (!isRecord(v)) throw new Error(`read-model: ${what} baseline is not an object`);
  return { median: num(v, "median"), sampleSize: num(v, "sampleSize") };
}

export function asBaselines(v: unknown): ScrutinBaselines {
  if (!isRecord(v)) throw new Error("read-model: baselines is not an object");
  const method = v.method;
  if (!isRecord(method)) throw new Error("read-model: baselines.method is not an object");
  return {
    votants: asBaseline(v.votants, "votants"),
    abstention: asBaseline(v.abstention, "abstention"),
    method: { id: str(method, "id"), version: num(method, "version") },
  };
}

export function asProvenance(v: unknown): ScrutinProvenance {
  if (!isRecord(v)) throw new Error("read-model: provenance is not an object");
  return {
    tier: str(v, "tier"),
    label: str(v, "label"),
    url: strOrNull(v, "url"),
    recordId: str(v, "recordId"),
    retrievedAt: str(v, "retrievedAt"),
  };
}
