# Security policy

## Reporting a vulnerability
Please report security issues **privately** via GitHub's "Report a vulnerability"
(Security → Advisories) on this repository, not as a public issue. We aim to
acknowledge within a few days.

## Scope & posture
- L'Index handles only **public parliamentary records** — no personal data beyond
  the public record. Do not introduce datasets that add PII.
- The **data plane** ingests untrusted upstream XML/JSON; parsers must be
  defensive (fuzz-worthy) and never `panic` on malformed input.
- Secrets never enter the repo: `.env` is gitignored, `.env.example` holds
  placeholders only, and `gitleaks` scans every push.
- Supply chain is gated by `cargo-deny` (advisories + licenses) and a weekly
  `cargo-audit` scan.

## Trust boundaries
The **app plane** has read-only access to the `facts` schema and owns writes to
the `app` schema; it must never mutate parliamentary facts.
