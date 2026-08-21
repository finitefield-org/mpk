# MPK Design with Implementation Roadmap and Task Plan

Migration note: this document records the current pre-cutover Go/GIR baseline.
It remains current, not historical, until the atomic cutover owned by
`05_rust_frontend_design-todo.md`. Forward work follows that file's VIR-00,
VIR-01, GO-VIR-02, then RUST dependency order and must not expose a mixed
helper interface. Certificate v0 and checker acceptance remain unchanged.

## 1. Purpose

MPK is an AI-oriented theorem prover and program-verification toolchain. Its first target is a restricted Go subset. The system is designed for machine-generated proof certificates, fast kernel checking, deterministic rejection diagnostics, and source-free verification.

## 2. Core architecture

```text
Go source
  -> untrusted go2gir frontend
  -> untrusted GIR
  -> untrusted VC generator
  -> theorem obligations
  -> untrusted AI / solver / tactic candidate generation
  -> canonical .mpcert
  -> Rust fast kernel
  -> independent Go reference checker
```

## 3. Trust boundary

Trusted evidence:

```text
canonical .mpcert bytes
Rust fast-kernel verdict
independent source-free reference-checker verdict
export_hash
certificate_hash
axiom_report_hash
recomputed axiom report
hash-pinned imports
```

Untrusted evidence:

```text
Go source
Go frontend
GIR
VC generator
AI output
solver yes/no output
tactic traces
theorem indexes
source maps
pretty-printed goals
CI status
```

## 4. Core calculus

MVP term grammar:

```text
Level ::= Zero | Succ Level | Max Level Level | Param Name

Term ::=
    Sort Level
  | Var u32
  | Const GlobalId [Level]
  | App Term [Term]
  | Lam Type Body
  | Pi Type Body
  | Let Type Value Body
```

MVP declaration grammar:

```text
Decl ::= Axiom | Def | Theorem | Inductive | Constructor | Recursor | TheoryPrimitive
```

Definitional equality includes alpha via de Bruijn representation, beta, delta for reducible definitions, iota for generated recursors, and zeta for lets/local definitions. It excludes eta conversion, proof irrelevance conversion, theorem proof unfolding, opaque definition unfolding, axiom unfolding, equality saturation, typeclass search, and SMT-backed conversion.

## 5. Certificate model

The certificate is canonical binary `.mpcert`. Logical layout:

```text
Certificate:
  header
  imports
  name_table
  level_table
  term_table
  proof_node_table
  declarations
  theory_certificates
  export_block
  axiom_report
  source_manifest?     # untrusted metadata
  hashes
```

The certificate checker decodes, validates canonicality, recomputes hashes, resolves imports, checks declarations and proof nodes, recomputes axiom reports, and rejects unsupported features.

## 6. Go subset

MVP accepts pure functions over bool, fixed-width signed/unsigned integers, fixed arrays, simple structs, local variables, if/else, return, static pure calls, and loops only with explicit invariants. Variant/decreases metadata is required only when a contract claims total correctness; otherwise loop VCs are partial-correctness obligations.

MVP rejects unsafe, cgo, reflection, dynamic interfaces, goroutines, channels, defer, panic/recover, maps, mutable slices, pointer aliasing, floating point, complex numbers, generics, package-level mutable state, and unsupported build conditions.

## 7. Integer semantics

Fixed-width Go integers must be modeled as bitvectors. Signed operations and comparisons use explicit signed interpretations. Mathematical integers may appear only through explicit conversion or specifications.

## 8. Implementation roadmap

The implementation proceeds through these phases:

1. Governance and spec freeze.
2. Core data model.
3. Type checking and definitional equality.
4. Canonical certificate format.
5. Fast Rust kernel.
6. Independent Go reference checker.
7. Foundational standard library.
8. Go frontend and GIR.
9. VC generator.
10. AI proof API.
11. Theory certificate checkers.
12. CI and package verification.
13. Alpha corpus and performance hardening.

For detailed tasks, see `tasks/TASK_BACKLOG.md` and `tasks/task_backlog.csv`.

## 9. Success criteria

The alpha is successful when it can verify a small Go corpus through both checkers, reject many failed AI candidates deterministically, and produce stable hashes and axiom reports across clean machines.

```text
100 small Go functions
1,000+ generated VCs
10,000 invalid AI candidates rejected
Rust fast-kernel verdict
Go reference-checker verdict
stable certificate/export/axiom hashes
explicit axiom report
```

## 10. Design discipline

Never speed up the system by trusting AI, trusting tactics, trusting solver yes/no answers, trusting theorem indexes, or expanding definitional equality. Speed should come from canonical data structures, hash-consing, caching, proof-node locality, and small independently checkable theory certificates.
