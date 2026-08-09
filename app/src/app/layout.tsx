import "./globals.css";
import type { ReactNode } from "react";

export const metadata = {
  title: "L'Index",
  description: "Comprendre l'activité du Parlement français, sans orientation.",
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="fr">
      <body>{children}</body>
    </html>
  );
}
