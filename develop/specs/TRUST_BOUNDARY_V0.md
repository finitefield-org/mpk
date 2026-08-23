# MPK Trust Boundary v0

Status: approved governance baseline for the MPK MVP.

## Scope

This specification defines which artifacts may justify accepting a theorem or package in MPK v0. It applies to every checker-facing component, including the planned `mpk-core`, `mpk-cert`, `mpk-kernel`, `mpk-cli`, independent `mpk-checker-ref`, and any theory-certificate checker that participates in proof acceptance.

Frontend, API, proof-search, diagnostic, package-management, and CI components may call checker-facing components, but they do not expand the trusted boundary.

## Boundary Rule

MPK distinguishes between:

1. helper evidence that may help generate, debug, or package a proof candidate; and
2. checker-facing evidence that may justify accepting a theorem after independent validation.

Only the second category is trusted proof evidence. Any artifact not explicitly listed as trusted in this specification is untrusted for proof acceptance.

## Trusted Proof Evidence

The following artifacts may justify acceptance only after the checker recomputes or validates them:

| Artifact | Trusted use | Validation requirement |
|---|---|---|
| Canonical `.mpcert` bytes | Source-free proof artifact | Decode, canonical re-encode, table-order, dependency, type, proof-node, import, and feature checks pass. |
| Checked theory certificate | Compact proof evidence for enabled theories | The theory checker verifies the certificate against the expected type and active checker profile. |
| Rust fast-kernel verdict | Primary checker result | Produced from canonical `.mpcert` bytes without reading source, tactic, AI, or solver trace data. |
| Independent Go reference-checker verdict | Cross-checker agreement result | Produced by the clean-room source-free checker without sharing Rust kernel implementation logic. |
| `export_hash` | Public interface identity | Recomputed from the checked export block under the certificate hash-domain rules. |
| `certificate_hash` | Certificate identity | Recomputed from canonical certificate payload under the certificate hash-domain rules. |
| `axiom_report_hash` | Axiom report identity | Recomputed from the checked axiom report. |
| Recomputed axiom report | Trusted dependency disclosure | Derived from checked declarations and proof nodes, then evaluated by release policy. |
| Hash-pinned imports | Import identity | Resolved by module name and export hash; high-trust mode also verifies certificate hash in the current session. |

The table describes Certificate v0 trust capabilities, not an assertion that
every release profile implements every capability. The current
`mpk.program_certificate.alpha.v0` profile in
`PROGRAM_CERTIFICATE_ALPHA_V0.md` uses their dual-checker intersection: one
self-contained root certificate, no imports, no detached proof-node table, no
theory-certificate table or theory features, and a zero-axiom report. Selected
foundation source certificates are helper inputs until their required
declaration closure has been copied into and rechecked as part of that root.

Example: a package can claim that `Example.Identity.correct` is accepted only
if the canonical certificate checks, hashes recompute, imports resolve by
policy, the axiom report is permitted, and required checker verdicts agree.
The original source, contract, lowering artifacts, and manifest remain
traceability data, not proof evidence.

## Untrusted Helper Evidence

The following artifacts are useful engineering inputs but never justify acceptance by themselves:

| Artifact | Allowed use | Trust-boundary rule |
|---|---|---|
| Release-registry and bundle bytes | Reproducible helper selection and execution | IDs, descriptors, inventories, and hashes do not establish a theorem or authorize an import without checker validation. |
| Toolchains, compilers, and frontend binaries | Parse, type-check, and lower source | Executable identity is traceability and reproducibility data, never proof acceptance evidence. |
| Source text in any language | Source of candidate verification tasks | Affects acceptance only through theorem statements encoded in checked certificates. |
| Contract sidecars | User specification input | Untrusted text until translated into certificate-checked theorem statements. |
| Parser, type-checker, package/module loader, HIR/SSA/MIR, and other compiler output | Frontend engineering | Never read by the checker as proof evidence. |
| VIR and other frontend IR output | Verification intermediate representation | Untrusted until theorem obligations are encoded in checked certificates. |
| Source maps | Source correlation | Locations and spans are diagnostic metadata, never proof evidence. |
| Source manifests | Reproducibility and lifecycle linkage | The payload and all internal registry, toolchain, input, VIR, map, VC, and hash claims remain untrusted even when hash-pinned. |
| VC and certificate-skeleton output | Candidate theorem obligations and declaration plans | Untrusted until the resulting declarations and proof nodes are checked from canonical certificate bytes. |
| Policy scan, evidence, reports, and reproduction recipes | Release workflow and audit presentation | Cannot approve a theorem, axiom, import, or checker verdict. |
| AI proof candidates and traces | Proof search and repair hints | Affect acceptance only after expansion into certificate-checkable proof nodes or checked theory certificates. |
| Tactic scripts and replay files | Candidate generation hints | Never trusted as replay evidence in MVP. |
| Solver yes/no answers | Search hints | Never trusted; solvers must emit independently checkable certificates when used for proof. |
| Theorem indexes and theorem graphs | Retrieval or dependency hints | Never accepted as proof of availability, type, or dependency correctness. |
| Comments, pretty-printed goals, and diagnostics | Debugging and repair | Human-readable context only. |
| AI explanations and provider output | Explanatory helper output | Cannot be interpreted as proof, policy approval, or a checker verdict. |
| CI status and package-registry metadata | Operational signal | Cannot replace source-free checker verdicts and hash recomputation. |

Thus registry bytes, toolchains, compilers, frontends, source, contracts, VIR,
source maps, manifests and their internal claims, VCs, policy output, and AI
output are all untrusted helper artifacts. Validating their structure, hashes,
or provenance does not promote them to trusted proof evidence.

Example: an AI API response may say that a proof node is likely repairable by `rewrite`, but the checker must still verify the resulting proof node against the expected type. The diagnostic text cannot be a success condition.

## Acceptance Rule

A declaration is accepted only if all relevant conditions hold:

1. certificate bytes decode under the canonical schema;
2. re-encoding produces byte-identical canonical bytes;
3. imports resolve by module name and required hashes when enabled by the
   active profile; the current program-certificate alpha profile instead
   requires an empty import table;
4. every referenced level, term, proof node, declaration, theory certificate, and import is well-formed and in dependency order;
5. the declared theorem type infers to a sort;
6. the proof checks against the theorem type;
7. all definitional-equality checks terminate within deterministic fuel;
8. export block, certificate hash, axiom report hash, and axiom report recompute exactly;
9. the active checker profile permits every used proof-node kind and theory-certificate format;
10. the active release policy permits every used axiom and core feature;
11. required checker verdicts agree.

If any condition is unsupported, unknown, ambiguous, or not implemented, the checker must reject.

## Fail-Closed Requirements

MVP components must reject rather than approximate when encountering:

- unknown certificate tags;
- non-canonical binary encoding;
- duplicate, unreachable, or out-of-order table entries;
- unresolved metavariables or holes;
- unsupported core features;
- unsupported proof-node tags under the active checker profile;
- unsupported source-language, semantic-profile, or lowering features;
- non-deterministic ordering;
- checker fuel exhaustion;
- unrecognized theory-certificate formats;
- malformed checked-theory evidence;
- solver evidence that does not independently verify;
- import hash mismatch;
- unapproved axiom use.

## Taboo List

The following must not enter the trusted MVP checker path:

- parser trust;
- source-language frontend, toolchain, or compiler trust;
- contract sidecar trust;
- VIR or other frontend IR trust;
- source-map or source-manifest-claim trust;
- VC generator trust;
- policy-output trust;
- tactic replay trust;
- SMT/SAT solver yes/no trust;
- theorem graph trust;
- theorem-index trust;
- AI search trust;
- CI status trust;
- package or release-registry metadata trust;
- general equality saturation;
- typeclass search;
- proof irrelevance as conversion;
- eta conversion;
- quotient primitives;
- theorem proof unfolding;
- opaque definition unfolding;
- axiom unfolding;
- general recursion.

## Checker-Facing Component Requirements

Unsafe-code restrictions for checker-facing components are defined in `specs/UNSAFE_POLICY_V0.md`.

Every checker-facing crate or tool must satisfy these requirements when created:

- its crate-level documentation, README, or module overview must reference `specs/TRUST_BOUNDARY_V0.md`;
- it must document which inputs it reads for acceptance;
- it must not read source files, source maps, tactic traces, AI traces, solver yes/no output, or CI metadata as proof evidence;
- it must return deterministic structured errors for trust-boundary rejection paths;
- it must preserve source-free checking for `.mpcert` inputs;
- it must include negative tests for at least one trust-boundary rejection path relevant to the component.

The planned checker-facing components are:

| Component | Boundary role |
|---|---|
| `mpk-core` | Type checking, definitional equality, declaration validation, and core rejection behavior. |
| `mpk-cert` | Canonical decoding, encoding, hash recomputation, import policy, export block generation, and axiom report generation. |
| `mpk-kernel` | Fast source-free verification orchestration. |
| `mpk-cli` | User-facing checker commands that must not bypass kernel acceptance rules. |
| `mpk-checker-ref` | Independent source-free reference checking and verdict agreement. |
| Theory-certificate checkers | Independently check compact theory evidence before a `Theory` proof node can be accepted. |

Non-checker components such as source-language frontends, compilers,
`mpk-vc`, policy/reporting tools, and `mpk-api` may generate artifacts and call
checkers, but they must not create alternate acceptance paths.

## Release Gate

Axiom category semantics and axiom allowlist profiles are defined in `specs/AXIOM_POLICY_V0.md`.

Before any release:

- no source parser is called by the checker;
- no tactic script is called by the checker;
- no AI trace is read by the checker;
- no solver yes/no answer is trusted;
- all accepted proof evidence is in `.mpcert` bytes or checked theory certificates;
- required fast-kernel and reference-checker verdicts agree;
- axiom reports are recomputed and evaluated by release policy.

Frontend, VIR, source-map, source-manifest, VC, policy, and AI schema revisions
do not modify these Certificate v0 acceptance inputs.
