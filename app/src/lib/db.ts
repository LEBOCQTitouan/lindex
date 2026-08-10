import { drizzle } from "drizzle-orm/postgres-js";
import postgres from "postgres";
import * as schema from "./schema";

// Connection against the facts schema (the read-model contract). Read-only is
// enforced by DB privileges — DATABASE_URL_FACTS uses a SELECT-only role (A2),
// not by this client. The app only ever issues SELECTs here.
const client = postgres(process.env.DATABASE_URL_FACTS!, { prepare: false });

export const db = drizzle(client, { schema });
export { schema };
