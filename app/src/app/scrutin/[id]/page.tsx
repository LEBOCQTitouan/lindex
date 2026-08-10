import { eq } from "drizzle-orm";
import Link from "next/link";
import { notFound } from "next/navigation";
import { db, schema } from "@/lib/db";
import { Hemicycle } from "@/components/Hemicycle";
import {
  asBaselines,
  asBreakdown,
  asProvenance,
  asTotals,
  PROVENANCE_TIERS,
} from "@/lib/read-model";
import { meterSegments, participation } from "@/lib/participation";

// SSR: one facts.read_scrutin row → the whole page. No client data-fetching,
// no writes to facts (A2). Everything rendered here comes from the read-model
// contract, so real L0-DATA rows render identically.

const OUTCOME_LABEL: Record<string, string> = {
  adopte: "Adopté",
  rejete: "Rejeté",
  reporte: "Reporté",
  info: "Info",
};

const SEGMENT_LABEL: Record<string, string> = {
  pour: "pour",
  contre: "contre",
  abst: "abst.",
  nv: "non-votant·es",
  absent: "absent·es",
};

function chamberShort(c: string): string {
  return c === "AN" ? "AN" : c === "SENAT" ? "Sénat" : c === "EXEC" ? "JO" : c;
}

const UPDATED_FMT = new Intl.DateTimeFormat("fr-FR", {
  dateStyle: "long",
  timeStyle: "short",
});

export default async function ScrutinPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  const rows = await db
    .select()
    .from(schema.readScrutin)
    .where(eq(schema.readScrutin.scrutinId, id))
    .limit(1);

  const row = rows[0];
  if (!row) notFound();

  const totals = asTotals(row.totals);
  const breakdown = asBreakdown(row.breakdown);
  const baselines = asBaselines(row.baselines);
  const provenance = asProvenance(row.provenance);
  const part = participation(totals);
  const segments = meterSegments(totals);

  const tier = PROVENANCE_TIERS[provenance.tier];
  const outcomeLabel = OUTCOME_LABEL[row.outcome] ?? row.outcome;
  const marge = totals.pour - totals.contre;

  return (
    <main className="wrap scrutin">
      <div className="scrutin__meta">
        <span className="tag tag--chamber" data-ch={row.chamber}>
          {chamberShort(row.chamber)}
        </span>
        <span className={`chip chip--${row.outcome}`}>{outcomeLabel}</span>
        <span className="tag">scrutin n° {row.scrutinId}</span>
      </div>

      <h1>{row.title}</h1>

      <div className="hemi-block">
        <Hemicycle breakdown={breakdown} />
      </div>

      {/* Participation: votes cast against the seats that could have voted. */}
      <div className="part">
        <span className="stat">
          <span className="stat__val">
            {part.votants} / {part.members}
            <span className="stat__unit"> votant·es</span>
          </span>
          <span className="stat__base">
            médiane du jour : {baselines.votants} · {part.pct} % des sièges
          </span>
        </span>
        <span className="part__gap">
          {part.absent} absent·es · {part.nonVotants} non-votant·es
        </span>
      </div>

      {/* Linear meter: spans all 577 seats so the gap stays visible. */}
      <div className="meter">
        <div className="meter__bar">
          {segments.map((s) =>
            s.n > 0 ? (
              <span
                key={s.kind}
                className={`seg v-${s.kind}`}
                style={{ flex: s.n }}
                title={`${s.n} ${SEGMENT_LABEL[s.kind]}`}
              />
            ) : null,
          )}
        </div>
        <div className="meter__legend">
          <span className="ml ml--pour">
            <b>{totals.pour}</b> pour
          </span>
          <span className="ml ml--contre">
            <b>{totals.contre}</b> contre
          </span>
          <span className="ml ml--abst">
            <b>{totals.abstention}</b> abst.
          </span>
          {part.absent > 0 ? (
            <span className="ml is-ghost">
              <b>{part.absent}</b> absent·es
            </span>
          ) : null}
        </div>
      </div>

      <div className="foot">
        <a
          className="prov"
          href={provenance.url}
          title={`${tier ? tier.label : provenance.tier} — ouvrir la source`}
          target="_blank"
          rel="noreferrer"
        >
          <span className="prov__dot">{tier ? tier.n : "?"}</span>
          <span className="prov__label">
            Scrutin n° {row.scrutinId} — {chamberShort(row.chamber)}
          </span>
        </a>
        <span className="margeline">
          marge {marge >= 0 ? "+" : ""}
          {marge} · scrutin n° {row.scrutinId}
        </span>
      </div>

      <div className="updated">
        Mis à jour le {UPDATED_FMT.format(row.updatedAt)} · méthodologie publique
      </div>
      <Link className="backlink" href="/">
        ← L&apos;Index
      </Link>
    </main>
  );
}
