# W09 internal unit 2: Boolean/integer component

Unit 2 is **in progress**, not complete. This increment implements the Boolean
and fixed-width integer portion of the approved scalar work unit. Float,
decimal, string operations, Guid, date/time/duration/instant and their remaining
numeric conversions still belong to unit 2. Units 3-8 and W09's exit condition
are unchanged. This record is not a W09 completion receipt.

## Implemented component

The generator derives exactly the Boolean/integer signatures retained in the
original validated VIR. It uses the existing closed scalar-signature registry
for operand/result types and ordered checks. It emits ordinary definitions for
the normal result, success predicate and mutually exclusive ordered failure
predicates. The normal result is zero on failure and must be consumed with its
success predicate. These definitions are not proofs of source invariants.

Covered operations are Boolean not/and/or/xor/equality, promoted i32/u32/i64/u64
arithmetic, division/remainder, comparisons, bitwise operations, the three shift
forms and checked/unchecked conversions among the nine integer/char carriers.
Arithmetic uses little-endian two's-complement words, widened mathematical
intermediates for overflow, restoring unsigned division with signed correction,
truncation toward zero and the low five/six shift-count bits. Division by zero
precedes signed minimum divided/reduced by minus one overflow, matching the
frozen signature registry. The C# specification also describes this division/
remainder edge in [integer remainder](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/expressions).

Signed multiplication uses an unsigned n-by-n product with the two sign
corrections modulo 2^(2n), rather than multiplying sign-extended 2n-bit operands.
Overflow compares the full product to the sign/zero extension of its low n bits.
No host arithmetic result is inserted as a definition or a trusted proof.

## Ordinary-core lowering

A private acyclic Boolean circuit records only inputs, constants, not, and, xor
and mux. Sharing is deterministic and dead gates are removed from the result
and failure roots. Gate evaluation in the tests is an independent observation
path; production emits actual Certificate v0 terms.

Each concrete state transformer binds at most 32 Bool gates with ordinary Let
and updates one aligned register block of a Boolean cube. Address selection and
updates use Bool-result recursors exclusively. Balanced static composition
preserves gate order. This bounds binder depth independently of arithmetic DAG
length; actual term/declaration/binder/transformer counters are still enforced.
The generator cannot replace a limit failure with an axiom, runtime observation
or partial accepted certificate.

The component retains the original VIR/foundation identities. Its importer
reconstructs the definitions and certificate bytes exactly; missing/reordered/
changed definitions and source/foundation/certificate substitutions reject.
It is a partial scalar capability with an explicit integer-only API name, not
a generic whole-foundation or application proof capability.

## Verification and review

Tests compare widened i128/u128 arithmetic with the bit circuits at signed and
unsigned endpoints, zero, minus one and deterministic samples. They cover all
162 conversion forms, both modes of promoted integer operations, unary edges,
and ordered failures. Separate tests evaluate emitted core definitions against
circuit observations, including updates across register blocks and canonical
zero results on failure. Original captured constructor programs exercise exact
VIR/operation-table reconstruction and substitution rejection. Unit 1's 24
carrier goldens are retained byte for byte.

`integer-circuits/*.hex` and `metrics.json` pin seven actual certificates,
including signed 64-bit multiplication and unsigned 64-bit division. Normal tests
regenerate and compare them. Both unchanged checkers must accept the exact same
bytes with zero axioms and reject hash corruption. Build/process/internal errors
cannot count as proof rejection. These tests verify definition typing and
translation; they do not discharge the later application VCs.

Direct review addressed core selector scope, Let de Bruijn indices, signed
multiplication correction, pruning/reindexing, shift masks, overflow/error
ordering and exact original-source linkage. The additional Std.Bool consumer
is reflected in the pinned consumer inventory and its historical-cache link.
The remaining scalar categories are deliberately recorded as unfinished.

Targeted commands/results are in `unit-2-integer-verification.json`.
The full T06 gate remains deferred to T06-W12; no workflow is introduced.
Candidate fixtures can be regenerated for review with:

```sh
MPK_W09_INTEGER_OUT=/tmp/mpk-w09-integers cargo test -p mpk-vc --lib integer_circuits_
```
