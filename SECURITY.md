# Security Policy

MPK is an alpha-stage research and implementation repository. It is intended to
make proof evidence, helper analysis, and checker boundaries auditable. It is
not a guarantee for an entire financial system, web service, or production
deployment.

## Supported Versions

The public `main` branch is the active development line. There is no supported
stable release series yet.

## Reporting A Security Issue

Please report security-sensitive issues privately before opening a public issue.
Use GitHub Security Advisories if they are enabled for the repository. If that
is not available, contact Finite Field, K.K. through:

- <https://finitefield.org/en/>

Include:

- a concise description of the issue;
- affected commands, crates, fixtures, or examples;
- reproduction steps;
- whether the issue can make untrusted helper artifacts appear as trusted proof
  evidence;
- whether the issue affects source-free certificate checking, checked theory
  certificates, hash recomputation, axiom reports, or reference-checker
  agreement.

Please do not include private customer code, secrets, tokens, or production
logs in the initial report.

## Security Boundary

Only these artifacts can support proof acceptance:

- canonical `.mpcert` bytes;
- checked theory certificates;
- Rust kernel or verifier verdicts;
- independent source-free reference checker verdicts;
- deterministic `export_hash`, `certificate_hash`, and `axiom_report_hash`
  values.

The following artifacts are helper analysis only and must not be treated as
proof evidence:

- Go source;
- contract JSON;
- `go2gir` output;
- GIR JSON;
- VC JSON;
- policy scan JSON;
- Markdown reports;
- CI status;
- AI, Gemini, solver, or operator logs;
- web handler traces.

If a bug crosses this boundary, treat it as security-sensitive.

## Customer Data

Do not submit private customer source code or customer-specific payment-policy
artifacts to the public repository. Use reduced fixtures or redacted examples
that preserve the failing behavior without exposing secrets or proprietary
logic.
