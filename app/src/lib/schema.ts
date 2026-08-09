import { pgSchema, text, jsonb, timestamp } from "drizzle-orm/pg-core";

// The `facts` schema is owned by the Rust plane. The app plane only reads it,
// and only the read-models. This declaration mirrors db/migrations/0001_facts.sql.
export const facts = pgSchema("facts");

export const readScrutin = facts.table("read_scrutin", {
  scrutinId: text("scrutin_id").primaryKey(),
  chamber: text("chamber").notNull(),
  title: text("title").notNull(),
  outcome: text("outcome").notNull(),
  totals: jsonb("totals").notNull(),
  breakdown: jsonb("breakdown").notNull(),
  baselines: jsonb("baselines").notNull(),
  provenance: jsonb("provenance").notNull(),
  updatedAt: timestamp("updated_at").notNull(),
});
