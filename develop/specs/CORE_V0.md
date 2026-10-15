# MPK Core v0 Specification

Status: frozen for implementation. This file is the stable MPK Core v0 target for SPEC-001. Changes require a new spec revision or a governance-approved amendment.

## Design principles

- Keep the core small.
- Prefer explicit machine representation over human syntax.
- Make type checking deterministic.
- Make definitional equality fuel-limited.
- Keep theorem bodies opaque after checking.
- Never rely on parser, frontend, tactic, AI, theorem index, or solver yes/no answers.

## Names

Global names use dotted ASCII component paths:

```text
Name      ::= Component ("." Component)*
Component ::= [A-Za-z_][A-Za-z0-9_']*
```

Only ASCII apostrophe is accepted. Empty components, operator notation, Unicode lookalikes, and source-language display names are rejected at certificate decode or declaration registration time. Binder display names are debug metadata only and are not stored in trusted term nodes.

## Universe levels

```text
Level ::=
    Zero
  | Succ Level
  | Max Level Level
  | Param Name
```

MVP intentionally omits `imax`. If an inductive encoding later requires `imax`, it must be added in a new spec revision with deterministic normalization rules before implementation.

## Terms

```text
Term ::=
    Sort Level
  | Var u32
  | Const GlobalId [Level]
  | App Term [Term]
  | Lam Type Body
  | Pi Type Body
  | Let Type Value Body
```

Notes:

- Variables use de Bruijn indices. `Var 0` refers to the innermost binder; binary certificates must not mix index and level representations.
- Binder display names are not stored in trusted term nodes.
- `App` uses spine form to avoid deep binary application trees.
- There are no holes, metavariables, implicit arguments, notation, macros, tactic blocks, or source-level pattern matching in core certificates.

## Declaration kinds

```text
Decl ::=
    Axiom
  | Def
  | Theorem
  | Inductive
  | Constructor      # generated artifact
  | Recursor         # generated artifact
  | TheoryPrimitive  # tightly limited MVP primitives
```

### Axiom

```text
Axiom:
  type : Sort u
```

The axiom report must include direct and transitive axiom dependencies.

### Definition

```text
Def:
  type : Sort u
  value : type
  reducibility : Reducible | Opaque
```

Only reducible definitions unfold during definitional equality.

### Theorem

```text
Theorem:
  type : Sort u
  proof : type
```

The theorem is exported as an opaque constant. Its proof body is checked inside the defining certificate but is not exported for downstream delta reduction.

### Inductive

MVP inductives are intentionally minimal:

- Bool;
- Nat;
- equality if encoded inductively;
- product/sigma only if needed;
- list only after fixed-array verification works.

The positivity checker is conservative. Unknown nested functors fail closed.

Implementation requirements:

- Accepted inductive declarations generate canonical constructor and recursor declarations.
- Generated names, dependency order, and hashes must be deterministic.
- Non-generated recursor equations do not participate in definitional equality.
- Until a positivity or recursor case is specified, the checker rejects it rather than approximating.

### Sort0-dependent Boolean cases

The 2026-10-07 approved CSHARP-03 design amendment adds the canonical generated
`<family>.cases` interface for a monomorphic Sort0 Boolean family, using existing
Certificate v0 recursor declarations and ordinary terms:

```text
cases : (P : Bool -> Sort0) -> P false -> P true -> (b : Bool) -> P b
cases P no yes false = no
cases P no yes true = yes
```

Here `Bool`, `false` and `true` refer to that declared family and its canonical
`<family>.false` and `<family>.true` nullary generated constructors. The complete
family, exactly two constructors, their order/types/generated flags, and the
entire dependent eliminator type must be checked before registering `cases` or
using its equations. Registering `cases` closes that family's constructor set:
any later constructor for the family rejects, even if the eliminator is only
used with a neutral major and never reduced. The interface has no universe
parameters: nonempty universe arguments on `cases`, its family or its
constructors reject during type inference as well as reduction, including
neutral and partial applications.
Registration also rejects these universe arguments in the types, definition
values (including opaque values), or proofs of preceding checked declarations.
Only terms reachable from those declarations are checked; unused term-table
entries and families without this `cases` interface retain their prior rules.
Nongenerated or malformed reserved `cases` declarations reject; other recursor names gain no
new equations. Neutral majors remain neutral, partial applications remain
partial, and surplus arguments apply to the selected branch. This introduces
neither eta nor proof irrelevance, and changes no existing `<family>.rec` type,
equation, artifact hash or wire-format field.

## Definitional equality

Generated by:

```text
alpha equivalence through de Bruijn representation
beta reduction
delta reduction for reducible definitions only
iota reduction for generated recursors only
zeta reduction for let/local definitions
```

Not included:

```text
eta conversion
proof irrelevance conversion
theorem proof unfolding
opaque definition unfolding
axiom unfolding
equality-saturation normalization
SMT-backed conversion
typeclass search
```

## Type checking

Core checking uses inference plus conversion:

```text
infer(ctx, env, term) -> type
check(ctx, env, term, expected) succeeds iff infer(term) is definitionally equal to expected
```

All checker errors must be deterministic structured errors. Human-readable strings are diagnostic output only, never the acceptance boundary.

## Builtin theories in MVP

The core may expose tightly limited theory-certificate checkers for:

- Bool normalization;
- BitVec ground normalization;
- small linear integer arithmetic certificates;
- fixed-array read/write rules.

External solver answers are not trusted. Only solver certificates checked by MPK are accepted.

## Resource limits

Every reduction and definitional-equality path must be bounded by deterministic fuel. Fuel exhaustion is rejection, not timeout-based acceptance.

## Unsafe-code policy

Normative MVP unsafe-code requirements are defined in `specs/UNSAFE_POLICY_V0.md`. The fast Rust kernel and certificate verifier must forbid unsafe code at crate level during MVP. If unsafe code is ever considered for performance, it must be isolated, audited, fuzzed, and kept outside the initial trusted path.
