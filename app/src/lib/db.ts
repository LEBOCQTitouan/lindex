import { drizzle } from "drizzle-orm/postgres-js";
import postgres from "postgres";
import * as schema from "./schema";

// Read-only connection against the facts schema (the read-model contract).
const client = postgres(process.env.DATABASE_URL_FACTS!, { prepare: false });

export const db = drizzle(client, { schema });
export { schema };
