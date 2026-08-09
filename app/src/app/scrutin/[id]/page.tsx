import { eq } from "drizzle-orm";
import { notFound } from "next/navigation";
import { db, schema } from "@/lib/db";
import { Hemicycle, type Tally } from "@/components/Hemicycle";

// SSR: one read-model row → the page. No client data-fetching.
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

  const s = rows[0];
  if (!s) notFound();

  const totals = s.totals as {
    pour: number;
    contre: number;
    membersTotal: number;
  };
  const breakdown = s.breakdown as Tally[];

  return (
    <main className="wrap">
      <h1>{s.title}</h1>
      <p className="mono">
        {totals.pour} pour · {totals.contre} contre · sur {totals.membersTotal}{" "}
        sièges
      </p>
      <Hemicycle breakdown={breakdown} />
    </main>
  );
}
