# Trust Boundary

Normative MVP trust-boundary requirements are defined in `specs/TRUST_BOUNDARY_V0.md`. This document is a design summary of the same boundary.

## Core policy

MPK must maintain a narrow trusted base. The project should explicitly distinguish between:

1. evidence that may help produce a proof; and
2. evidence that may justify accepting a theorem.

Only the second category belongs in the trusted boundary.

## Trusted proof evidence

The trusted/checker-facing evidence is:

```text
canonical .mpcert bytes
Rust fast-kernel verdict
independent source-free reference-checker verdict
deterministic export_hash
deterministic certificate_hash
deterministic axiom_report_hash
recomputed axiom report
hash-pinned imports
```

## Untrusted helper evidence

The following are useful but untrusted:

```text
Go source text
contract sidecar text
parser output
elaborator output
Go SSA output
GIR output
VC generator output
AI proof candidates before checking
AI traces
solver yes/no answers
tactic scripts
tactic replay files
theorem indexes
source maps
comments
pretty-printed goals
CI status
package registry metadata
```

## Acceptance rule

A declaration is accepted only if:

1. the certificate bytes decode under the canonical schema;
2. re-encoding produces byte-identical canonical bytes;
3. imports resolve by module name and hash;
4. all referenced terms, levels, and declarations are well-formed;
5. the theorem type infers to a sort;
6. the proof checks against the theorem type;
7. all definitional equality checks terminate within deterministic fuel;
8. the export block and axiom report recompute exactly;
9. the active policy permits all used axioms and core features.

## Fail-closed rules

MPK must reject rather than approximate when encountering:

- unknown certificate tags;
- non-canonical binary encoding;
- unresolved metavariables;
- unsupported core features;
- unsupported Go language features;
- non-deterministic orderings;
- checker fuel exhaustion;
- unrecognized theory-certificate formats;
- solver certificates that do not independently verify.

## Kernel design taboo list

Do not place these in the trusted kernel during MVP:

- parser trust;
- Go frontend trust;
- tactic replay trust;
- SMT solver yes/no trust;
- theorem graph trust;
- theorem-index trust;
- AI search trust;
- general equality saturation;
- typeclass search;
- proof irrelevance as conversion;
- eta conversion;
- quotient primitives;
- theorem proof unfolding;
- opaque definition unfolding;
- axiom unfolding;
- general recursion.

## Axiom policy

Normative axiom categories and release-profile behavior are defined in `specs/AXIOM_POLICY_V0.md`.

Every axiom must be visible in the axiom report. MVP should distinguish:

- `CoreAxiom`: required logical primitives, ideally zero or near-zero;
- `BuiltinTheoryAxiom`: primitive theory interface assumptions, if any;
- `GoSemanticsAxiom`: Go semantic modeling assumptions, temporary and targeted;
- `ExternalAxiom`: rejected by default for release-ready packages.

The long-term goal is to replace semantic axioms with checked definitions and certified lemmas.
