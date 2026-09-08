# CSHARP-03-T06-W03 data verification conditions

`DataVcProgram` is a private, independently reconstructed handoff from validated
monomorphic VIR to the ordinary assembler. It records pending definition and
verification obligations, not semantic proof receipts. W09 owns ordinary-core
translation, proof construction and same-byte dual checking. No host evaluator,
BCL call, theorem primitive, axiom or source-supplied discharge bit is used by
this generator. The certificate and public artifact schemas are unchanged.

The program binds the source VIR, foundation and closed-set hashes. Only invoked
data operations are specialized. Each definition retains the exact closed
signature and check precedence, the instantiated foundation equation when one
exists, concrete carrier parameters, and the shared structural recipe DAG.
Scalar carriers fix integer widths/sign/modulo/shift conventions, IEEE fraction/
exponent/bias and rounding, 96-bit decimal coefficients and scales, ordinal
UTF-16 bounds, Gregorian day numbers, ticks, Unix milliseconds and GUID fields.
Closed definitions retain collection bounds, sum payloads, currency arguments
and ownership carrier shapes. Source products retain stored member IDs/order
and declared enum arms. These are finite definition inputs, not claims that a
particular execution succeeded. Operation relation/failure references are
pending monomorphic definition requirements; W09 must expand and check them
before proof acceptance. The handoff itself cannot certify such a reference.

For invocation `op(args) -> result`, free variable index `i` binds exactly
`subjects[i]`. Operands precede the result. If check `i` fails under predicate
`F_i(args)`, its prefix is `P_i = AND(j<i, NOT F_j)` and its failure guard is
`P_i AND F_i`. The success relation is required under `AND(i, NOT F_i)`.
This retains the distinction between the semantic success arm and a normal CFG
edge that also carries tagged error results.

- Static obligations require `P_i -> NOT F_i`. Even output-bound checks depend
  on the computed mathematical size from operands, before result construction.
- Exceptions retain the exact check, exception type, target and captured payload;
  W05 composes handler/search/unwind obligations. They are not forbidden by a
  spurious static non-failure goal.
- Parse and application error outcomes require the correct tagged result relation
  under their failure guard, using the actual result SSA carrier. Their complete
  failure identity is part of the definition signature and reference.
- Ownership evidence retains the independently replayed before/actions/after
  protocol, including publication bounds, initialized cells, borrows, transfers,
  freezes and discards. Private construction carriers are excluded from public
  structural comparison; this evidence does not prove a user invariant.

Contract attachments additionally retain typed definedness bodies. Checked
arithmetic and exception-producing expressions must be defined; tagged errors
remain values. Indexing, payload extraction and formatting retain their partial
operation conditions. Conditional branches guard their own definedness; lets
and bounded quantifiers retain lexical scopes. Definedness bodies are attached
conditions to instantiate at the original contract use point, not unqualified
assumptions about all values of a type. Binding, control and boundary claims
retain their existing owners and original attachments.

All generated ordinary references, sequents, term nodes, subject binders and
transport bytes are reserved under the existing limits. Shared W01/W02/W03
definition names are counted once in the cumulative declaration reservation. Definitions and
per-invocation/check/ownership/contract identities enter the data obligation
groups. Function data groups depend on the global data group. The VC digest and
skeleton therefore bind the complete program. Standalone import compares exact
canonical bytes against regeneration; it never deserializes a supplied program
into a validated capability.

## Verification

The primary owner remains
`crates/mpk-vc/tests/csharp_practical_vc.rs#CSHARP-03-T06-W03`, with tests in
`tests/support/csharp_practical_data_vc.rs` and two private generator unit tests.

`goldens.json` pins 208 original frontend captures covering 218 distinct data
step rows from T03's retained corpus. Each row is independently imported and
emitted, and its invocation subjects, results, checks and handoff are compared.
Every generated sequent has a success witness and a failing-relation witness;
every check is also tested as the first of several simultaneous failures.
These are Boolean proofs/counterexamples for generated guard composition, with
semantic predicates treated as atoms. They are not arithmetic/library proofs
or kernel acceptance receipts. Concrete semantic definitions and their proofs
must still be translated and checked by the W09 assembler.

Mutation cases preserve original field ordering and first assert an unchanged
import. They then alter check order, prefix guards, exceptional targets, SSA
operands, normal goals, carrier definitions, definition references or lineage,
or remove an invocation. Unit cases cover lazy conditional definedness, lexical
binders and the binder limit. Existing W01/W02 consumers and resource accounting
remain covered; W02's current VC-hash goldens change because the new data digest
and obligations now participate, while construction handoffs remain identical.
Historical verification receipts are not rewritten.

The scoped commands and results are recorded in `verification.json`; review
iterations are recorded in `review.md`. Under AGENTS.md the full
`./scripts/check-fast.sh` gate is deferred to T06's final W12. This corrects the
W09 scheduling label in the earlier W01/W02 records; W09 remains the ordinary
proof-assembly owner. No full gate or equivalent split workspace check ran here.
