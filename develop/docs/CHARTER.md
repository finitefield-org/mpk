# MPK Project Charter

Status: project charter for the MPK MVP.

Migration note: this charter's Go/GIR scope describes the current pre-cutover
release baseline. It remains current, not historical, until the atomic cutover
owned by `05_rust_frontend_design-todo.md`. The staged language-neutral VIR and
Rust work must follow that dependency graph and must not partially reinterpret
this baseline. Certificate v0 and the trust boundary do not change.

## Mission

MPK exists to make machine-generated proof candidates cheap to reject and safe to accept. The project builds a certificate-first theorem proving and program-verification toolchain for a restricted Go subset, with acceptance based on canonical proof certificates and independent checker verdicts.

The project optimizes for:

- small trusted base;
- deterministic source-free checking;
- canonical `.mpcert` artifacts with stable hashes;
- explicit axiom reports;
- structured rejection diagnostics for AI repair loops;
- reproducible package and release verification.

## Non-goals

MPK is not a human-first interactive proof assistant. Human proof syntax, tactic ergonomics, notation, IDE workflows, broad mathematical library coverage, and full Go language support are not MVP objectives.

The MVP also does not aim to trust:

- Go source text;
- parser, frontend, SSA, GIR, or VC generator output;
- AI traces or proof-search logs;
- tactic scripts or tactic replay;
- solver yes/no answers;
- theorem indexes or CI status;
- source maps, comments, or diagnostics.

These artifacts may help generate candidate proofs, but they do not justify accepting a theorem.

## MVP Scope

The MVP is limited to the smallest chain needed to demonstrate certificate-first Go verification:

- dependent core terms using `Sort`, `Var`, `Const`, spine `App`, `Lam`, `Pi`, and `Let`;
- opaque theorem declarations and reducible or opaque definitions;
- deterministic, fuel-limited definitional equality;
- minimal inductives and theory interfaces for equality, Bool, Nat, Int, BitVec8/16/32/64, and fixed arrays;
- canonical binary `.mpcert` encoding, decoding, hashing, import resolution, and axiom reporting;
- Rust fast kernel and independent Go source-free reference checker;
- untrusted `go2gir` frontend for pure functions in the supported Go subset;
- untrusted GIR-to-VC generation for straight-line code, branches, runtime-safety checks, and annotated loops;
- AI-oriented proof-node API and repair diagnostics that cannot bypass certificate checking.

Unsupported or ambiguous behavior must fail closed. The MVP must reject rather than approximate unsupported core features, unsupported Go features, unknown certificate tags, non-canonical encodings, unresolved metavariables, checker fuel exhaustion, unrecognized theory-certificate formats, and solver evidence that is not independently checkable.

## Acceptance Philosophy

MPK accepts a declaration only when the checker-facing evidence is sufficient:

- certificate bytes decode under the canonical schema;
- re-encoding produces byte-identical canonical bytes;
- imports resolve under the active hash policy;
- all referenced terms, levels, declarations, proof nodes, and theory certificates check;
- definitional equality terminates within deterministic fuel;
- export hash, certificate hash, axiom report hash, and recomputed axiom report match;
- the active release policy permits every used axiom and core feature;
- the Rust fast kernel and independent Go reference checker agree when both are required.

Implementation speed, proof-search convenience, frontend usability, or solver power must not expand the trusted base. Performance work should come from canonical data structures, locality, caching, and independently checkable theory certificates.

## Governance Commitments

Before implementation proceeds past governance and spec freeze, contributors must be able to explain:

- which artifacts are trusted proof evidence;
- which artifacts are untrusted helper evidence;
- which Go subset features are accepted or rejected;
- which axioms are allowed by release policy;
- which checker profiles permit each proof-node family;
- which release gates block publication.

Any proposal that changes the trusted boundary, accepted Go subset, axiom policy, unsafe-code policy, certificate format, or release gates must update the corresponding specification and be reviewed before implementation relies on it.
