# ProofOps Engine Support Design

Status: implemented on the active Go/Rust/C#/Java successor release

MPK is the verification engine for ProofOps. It owns deterministic source
capture, registered frontend execution, successor VIR and VC generation,
certificate construction, source-free checking, and machine-readable policy
evidence. ProofOps owns customer intake, UI, authentication, persistence,
billing, presentation, consent, and any external AI integration.

## Trust and ownership boundary

MPK accepts a proof claim only through canonical `.mpcert` declarations or
checked theory certificates accepted by the active checker policy. Source,
contracts, compilers, frontends, source maps, manifests, VIR, VC, policy scan
JSON, explanation requests, CI status, and report prose are helper artifacts.

MPK owns:

- the revision-3 semantic-profile registry and compiled profile contracts;
- the `mpk.release.bundle_registry.v1` Go/Rust/C#/Java release tuples;
- immutable source capture and deterministic subset diagnostics;
- successor VIR, VC, skeleton, policy, evidence, and program-certificate
  generation;
- Rust and independent reference-checker verdicts;
- stable v2 JSON transports and regression corpora.

ProofOps owns:

- customer-facing workflows, forms, dashboards, and accounts;
- source intake, privacy, retention, and incident response;
- pricing, billing, report presentation, and support;
- external model selection, credentials, network calls, generated prose, and
  model-provider governance.

## Single installed release

The active `bin/mpk` is installed beside:

```text
share/mpk/semantic-profile-registry.json
share/mpk/bundle-registry.json
libexec/mpk/bundles/<registered bundle id>/...
```

The executable validates both canonical registries and resolves exactly one
registered frontend/toolchain pair from the request's semantic profile. It
rejects predecessor registries, crossed language/profile identities, bundle
tampering, ambient plugin state, and paths outside the installed descriptors.
There is no public dual-generation or caller-selected helper path.

## Command contract

### Readiness scan

```sh
mpk policy scan <source-root> \
  --semantic-context <context.json> \
  --selection <selection.json> \
  [--contract <normalized-path> ...] \
  --json-out <scan.json>
```

The output schema is `mpk.policy.scan.v2`. It contains the validated semantic
context and selection, registered frontend/toolchain and release identities,
captured input hashes, successor VIR/source-map/manifest linkage, profile-owned
policy contract, readiness, diagnostics, and rejected features. It contains no
proof-acceptance field.

### Strict verification

```sh
mpk policy verify <source-root> \
  --semantic-context <context.json> \
  --selection <selection.json> \
  [--contract <normalized-path> ...] \
  --evidence-json <evidence.json>
```

The output schema is `mpk.policy.evidence.v2`. The command executes the same
captured frontend graph, generates successor VC and skeleton artifacts, applies
the compiled policy/evidence contracts, assembles a program certificate, and
requires the source-free checkers before committing the output. Verification
is always strict.

### Explanation projection

```sh
mpk explain <source-root> \
  --semantic-context <context.json> \
  --selection <selection.json> \
  [--contract <normalized-path> ...] \
  [--language <en|ja>] \
  --request-json-out <sanitized-request.json>
```

This route first performs the same strict local verification and then emits a
profile-owned `mpk.ai.explain.request.v2` projection. It does not authenticate,
select a provider, make a network call, ingest a provider response, write
prose, or change evidence.

Go and Rust require one or more normalized `--contract` paths. C# and Java
contracts are fixed by their validated selection envelopes, so both profiles
reject CLI contract paths.

## Closed caller surface

The commands intentionally do not accept:

- a language/profile pair separate from the semantic-context envelope;
- registry IDs, registry hashes, or registry paths;
- frontend/toolchain bundle IDs or executable/toolchain paths;
- strategy, checker, axiom, or non-strict options;
- provider, project, endpoint, model, credential, API-key, or bearer-token
  options;
- predecessor schemas or compatibility fallbacks.

This makes the semantic-context and selection envelopes the only typed request
inputs and keeps all executable identity inside the installed release.

## Evidence model

`mpk.policy.evidence.v2` separates:

- `trusted_evidence`: accepted certificate declarations, checked theory
  certificates, checker verdicts, and recomputed axiom reports;
- `helper_artifacts`: source/contract/VIR/VC hashes and traceability metadata;
- `properties`: one of `mpk_verified`, `proof_pending`, `helper_only`, or
  `unsupported`, with references that are validated against the trusted set;
- `reproduction_recipes`: deterministic release-owned recipe metadata;
- semantic, release, frontend, toolchain, policy, and evidence contract
  linkage.

An `mpk_verified` status is valid only when its evidence reference resolves to
an accepted declaration or checked theory obligation. Hashes and helper
metadata alone never elevate a property.

## Product profiles

The active registry contains exact Go, Rust, C#, and Java semantic profiles. Each
entry compiles nine contracts (`frontend`, `vir`, `source_map`, `manifest`,
`vc`, `policy`, `evidence`, `ai`, and `release`) into the release. The policy
contract fixes the strategy, checker, and axiom profiles for that language.

The first product corpus covers pure deterministic fixed-width payment-policy
functions such as reserve, refund, discount, fee, and points decisions.
Floating point, arbitrary precision, I/O, storage, time, randomness,
concurrency, reflection, unsafe behavior, and other profile-excluded features
fail closed.

## ProofOps integration rules

ProofOps may display:

- scan readiness and deterministic diagnostics as helper analysis;
- evidence property status and its validated trusted reference;
- registered semantic/release identity and helper hashes for traceability;
- sanitized explanation output only as explicitly untrusted prose.

ProofOps must not infer verification from source text, a successful command,
VIR/VC hashes, frontend readiness, CI status, model output, or operator notes.
It must preserve the distinction among `strategy_profile`, `checker_profile`,
and `axiom_profile` recorded by the compiled policy contract.

## Release and verification gate

`scripts/check-java-frontend.sh` owns the complete two-pass offline release
gate. It rebuilds active bundles, validates the semantic and bundle registries,
executes Go/Rust/C#/Java through one installed image, rejects old/crossed/tampered
identities, checks the frontend and VC corpora, runs both source-free checkers,
checks artifact paths, and requires a clean byte diff.

The gate and `release-report.json` remain helper evidence. Certificate
acceptance is still decided only by the source-free checkers.

## Non-goals

MPK does not verify an entire web service, authorize a payment, replace service
authentication, own customer data governance, accept arbitrary source
languages, or make AI output proof evidence. Later profiles must enter through
their frozen semantic profile and a new atomic release. The proposed practical
C# profile is eligible to begin at T01-W01 after the native JAVA-03-T10 receipt
was recorded on 2026-09-03.
