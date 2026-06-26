# Executive Summary

## Goal

Build **MPK: Machine Proof Kernel**, an AI-oriented theorem proving and program-verification system optimized for:

- machine-generated proof certificates;
- high-speed kernel checking;
- deterministic rejection diagnostics for AI repair loops;
- Go-to-verification-condition transpilation through a restricted and explicit intermediate representation;
- small trusted base and independent source-free reference checking.

## Non-goals

MPK is **not** intended to compete with Lean, Rocq, Isabelle, or Agda as a human-first interactive theorem prover. Human-friendly syntax, tactic ergonomics, notation, IDE features, and mathematical library breadth are secondary. The first objective is to make proof candidates cheap to generate, cheap to reject, and safe to accept only through canonical certificate checking.

## Strategic direction

The project should adopt a certificate-first architecture:

```text
Untrusted side:
  source parser
  Go frontend
  SSA/GIR conversion
  VC generator
  AI proof search
  tactic search
  SMT/SAT solver calls
  theorem index
  comments and diagnostics

Trusted/checker-facing side:
  canonical .mpcert bytes
  Rust fast verifier verdict
  independent source-free reference checker verdict
  export_hash
  certificate_hash
  axiom_report_hash
  recomputed axiom report
```

## MVP scope

The first version should support:

- dependent core with `Sort`, `Var`, `Const`, spine `App`, `Lam`, `Pi`, `Let`;
- opaque theorem declarations and reducible/opaque definitions;
- equality, Bool, Nat, Int, BitVec8/16/32/64, and fixed arrays;
- deterministic, fuel-limited definitional equality;
- canonical binary certificates;
- Rust fast kernel;
- independent Go reference checker;
- restricted Go subset: pure functions, fixed-width integers, bool, if, return, struct field read, fixed-array read, explicit loop invariants, and optional variant/decreases metadata only when total correctness is claimed;
- VC generator for straight-line code, branches, and simple loops;
- AI-oriented proof-node API and structured kernel diagnostics.

## First success benchmark

The first credible alpha should verify a corpus similar to:

```text
100 small Go functions
1,000 generated verification conditions
10,000 failed AI proof candidates
full Rust fast-kernel check
full independent Go reference-checker check
stable deterministic hashes across machines
zero trusted use of source text, tactics, AI traces, or solver yes/no answers
```

## Main implementation risk

The main risk is not proof-search intelligence. The main risk is allowing trusted scope creep. The kernel must remain small, deterministic, fuel-limited, and boring. Solvers may be powerful, but they must emit checkable certificates. Frontends may be useful, but their output must be hash-pinned and treated as untrusted unless a later verified frontend is built.
