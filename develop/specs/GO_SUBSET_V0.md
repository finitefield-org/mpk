# Go Subset v0 Specification

Status: historical; replaced by `GO_VIR_PROFILE_V0.md`.

## Rationale

The first MPK frontend targets a restricted Go subset to avoid semantic complexity while testing the proof-certificate pipeline. The goal is not full Go support. The goal is a credible, closed, auditable subset for program verification.

## Fail-closed rule

Any Go feature, type, statement, expression, build configuration, runtime behavior, or contract condition not explicitly accepted below must reject before GIR emission. The frontend must report the rejected feature rather than approximating or silently dropping semantics.

## Accepted language features

### Functions

- Top-level functions.
- Methods only after a non-pointer receiver is lowered to an explicit first argument.
- No closures in MVP.
- No package-level mutable state.
- No `init` functions.

### Types

- `bool`.
- `int8`, `int16`, `int32`, `int64`.
- `uint8`, `uint16`, `uint32`, `uint64`.
- Fixed arrays `[N]T` where `N` is small enough for the selected theory profile or represented symbolically.
- Structs whose fields are themselves accepted types.
- Strings are not accepted in v0. Equality-only string support is reserved for a future profile.

### Statements

- Variable declarations.
- Assignments to local variables.
- If/else.
- Return.
- For loops with an explicit invariant. A variant/decreases expression is required only when total correctness is claimed.

### Expressions

- Boolean operators.
- Integer arithmetic over fixed-width bitvectors.
- Signed and unsigned comparison operators.
- Struct construction and field read.
- Fixed-array construction and index read.
- Static call to a verified pure function.

## Rejected language features

- `unsafe`.
- cgo.
- Pointers and address-taking.
- Heap allocation.
- Reflection.
- Interfaces and dynamic dispatch.
- Goroutines.
- Channels.
- Defer.
- Panic/recover.
- Maps.
- Mutable slices.
- Strings.
- Floating-point and complex numbers.
- Generics.
- Package-level mutable state.
- Non-deterministic iteration.
- Build tags or build constraints that change selected files or semantics. MVP rejects conditional builds rather than accepting them through manifest pinning.

## Integer semantics

Every fixed-width integer operation must match Go semantics. Do not model `int64` as unbounded mathematical integer. Use `BV64`, with signed or unsigned interpretation for comparisons and conversions.

## Runtime errors

MVP generates runtime-safety proof obligations for supported operations whose panic condition is expressible in GIR. It rejects operations whose panic behavior is not modeled. The initial obligation set covers:

- division by zero;
- negative shift count;
- array index out of bounds;

Explicit `panic` paths remain rejected in v0.

## Loop policy

Loops are accepted only if the contract sidecar supplies:

- invariant before loop;
- invariant preservation obligation;
- variant/decreases expression, if total correctness is claimed;
- postcondition bridge from invariant and exit condition.

## Function-purity policy

A function is pure in MVP if:

- it reads only parameters and local variables or values created within the function;
- it writes only local variables;
- it makes only static calls to pure functions;
- it does not read package-level mutable state;
- it performs no I/O, no concurrency, no reflection, no panic/recover.
