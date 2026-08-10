import Link from "next/link";

export default function Home() {
  return (
    <main className="wrap">
      <h1>L&apos;Index</h1>
      <p className="muted">
        Squelette de l&apos;app. La page digest lira les read-models produits par
        le plan Rust (schéma <code>facts</code>).
      </p>
      <p>
        <Link href="/scrutin/8433">→ Exemple : scrutin 8433 (ordre public)</Link>
      </p>
    </main>
  );
}
