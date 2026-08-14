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

## Credentials And Repository Hygiene

Never commit API keys, access or refresh tokens, local Application Default
Credentials (ADC) files, service-account key JSON, private keys,
credential-bearing `.env` files, copied `gcloud` configuration, or real
customer AI request and response artifacts.

- Keep local Google Cloud ADC in the platform's normal user configuration
  directory outside this checkout.
- Prefer user ADC for local development and attached workload identities for
  deployed environments. Do not copy a downloaded service-account key into the
  repository.
- Put local generated reports and request previews under the ignored `target/`
  tree unless another approved, non-repository location is required.
- Use placeholders and deterministic fake authentication in documentation,
  fixtures, snapshots, and tests.
- Inspect staged changes before pushing and run every configured repository
  secret scanner. `.gitignore` is defense in depth, not a security boundary.
- Do not paste credentials or credential-bearing logs into commits, issues,
  pull requests, CI output, or release artifacts.

If a credential may have been exposed, revoke or rotate it first. Then report
the incident privately, inspect provider audit logs, remove retained copies and
Git history as appropriate, and verify the replacement credential. Deleting
the value in a later commit or rewriting history does not make the exposed
credential safe to reuse.

The optional `mpk explain` command is ADC-only. It invokes the fixed
`gcloud auth application-default print-access-token --quiet` argument vector
and never accepts an API key, raw bearer token, or credential-file path. The
dry run does not authenticate or access the network. Normal mode sends only
the reviewed minimal redacted evidence payload to the selected Vertex AI
project/location; it does not send source, contracts, certificates, paths,
commands, or original property identifiers. The provider still processes the
request remotely, so operators must obtain appropriate customer consent and
review Google's current retention, abuse-monitoring, caching, and data-use
terms. MPK does not promise zero retention.

The final JSON and Markdown reports are customer artifacts even though they
are untrusted helper analysis: they contain the project ID, exact source hash,
and locally restored property identifiers. Store them under an approved
location, restrict access, and delete local ADC with
`gcloud auth application-default revoke` when it is no longer needed.

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
