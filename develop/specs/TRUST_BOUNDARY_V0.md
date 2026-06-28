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

Example: a package can claim that `Example.Max64.correct` is accepted only if the canonical certificate checks, hashes recompute, imports resolve by policy, the axiom report is permitted, and required checker verdicts agree. The original Go source and contract sidecar remain traceability data, not proof evidence.

## Untrusted Helper Evidence

The following artifacts are useful engineering inputs but never justify acceptance by themselves:

| Artifact | Allowed use | Trust-boundary rule |
|---|---|---|
| Go source text | Source of candidate verification tasks | Affects acceptance only through theorem statements encoded in checked certificates. |
| Contract JSON sidecar | User specification input | Untrusted text until translated into certificate-checked theorem statements. |
| Parser, type-checker, package loader, and SSA output | Frontend engineering | Never read by the checker as proof evidence. |
| GIR output | Verification IR | Untrusted until theorem obligations are encoded in checked certificates. |
| VC generator output | Candidate theorem obligations | Untrusted until emitted obligations are checked as certificate declarations. |
| AI proof candidates and traces | Proof search and repair hints | Affect acceptance only after expansion into certificate-checkable proof nodes or checked theory certificates. |
| Tactic scripts and replay files | Candidate generation hints | Never trusted as replay evidence in MVP. |
| Solver yes/no answers | Search hints | Never trusted; solvers must emit independently checkable certificates when used for proof. |
| Theorem indexes and theorem graphs | Retrieval or dependency hints | Never accepted as proof of availability, type, or dependency correctness. |
| Source maps, comments, pretty-printed goals, and diagnostics | Debugging and repair | Human-readable context only. |
| CI status and package registry metadata | Operational signal | Cannot replace source-free checker verdicts and hash recomputation. |

Example: an AI API response may say that a proof node is likely repairable by `rewrite`, but the checker must still verify the resulting proof node against the expected type. The diagnostic text cannot be a success condition.

## Acceptance Rule

A declaration is accepted only if all relevant conditions hold:

1. certificate bytes decode under the canonical schema;
2. re-encoding produces byte-identical canonical bytes;
3. imports resolve by module name and required hashes;
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
- unsupported Go language features;
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
- Go frontend trust;
- contract sidecar trust;
- GIR trust;
- VC generator trust;
- tactic replay trust;
- SMT/SAT solver yes/no trust;
- theorem graph trust;
- theorem-index trust;
- AI search trust;
- CI status trust;
- package registry metadata trust;
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

Non-checker components such as `go2gir`, `mpk-vc`, and `mpk-api` may generate artifacts and call checkers, but they must not create alternate acceptance paths.

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
