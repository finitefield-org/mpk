# Complete original operation proofs through ordinary equality transport

This partial T06-W09 checkpoint changes the proof recipe for the same 436
original operation sequents in 45 source contexts. All complete original
metadata is identical to the `aac0ea64` checkpoint apart from the emitted
certificate hashes. All 31 pending operations and all 987 application IDs remain
pending. W09, complete application assembly and the full T06 gate remain open.

## Complete propositions and checked function agreement

Every original value binder, domain premise, normal guard and normal-result
operand is retained. Each ordered first-failure and success operand is the exact
body of its original named definition, after applying the original arguments.
This transparent beta expansion preserves the complete Boolean computation,
including negation of preceding failures; it does not substitute a literal,
raw failure or weaker guard. The source-success premise still calls the original
named success definition. All original source/condition/VC identities and the
ordered outcome metadata are unchanged.

The recipe first proves equality of each pair of original actual/concrete
wrapper functions using registered ordinary `Std.Eq.refl`. The two wrappers
call the same original component with the same arguments. This closed function
equation is checked by the kernel and cached as an ordinary theorem. It is not
a host assertion about a function or an assumption added to the certificate.

The complete operands are then related through registered `Std.Eq.rewrite`.
Its motive abstracts only the paired wrapper function, preserving the whole
normal result or ordered Boolean mask around it. Before transport, replacing
concrete wrappers with actual wrappers must reproduce the exact original left
operand. After transport, the final right operand must reproduce the exact
original right operand. A changed mask, omitted failure or incorrect function
pair cannot pass these reconstruction checks.

Motive construction shifts free values and respects nested Lam/Pi/Let binders.
The ordinary term DAG is traversed with a cache keyed by term and binder depth;
original declarations are never rewritten. Equality of the whole function
avoids repeatedly reducing the same large applied circuit when comparing the
two complete operands. The public theorem still names the original proposition,
and the intermediate theorem still exposes its identical complete Pi spine.

No original source definition, kernel, checker, primitive, recursor, axiom or
Certificate v0 rule changes. All original declaration/term prefixes and all
45 historical original-operation metadata/certificate pins are preserved.

## Actual verification and meaningful negative checks

Both targeted integration tests pass locally: all 45 actual source contexts
with complete original premises/goals and strict import negatives, and the
unchanged original operation pins. The final run repeats only the affected
proof/import integration after an argument-grouping lint correction. The pin
test is retained because the original generator, its source/test/fixture bytes
and the original operation definitions are unchanged. Clippy over library/tests
and both format checks pass. All 91 outputs are byte-identical before and after
the lint correction. The failed initial Clippy result is retained separately.

Every emitted certificate actually passes the unchanged Rust kernel with zero
axioms. Replacing a concrete normal implementation with a correctly typed zero
result still permits its definition-only certificate, but the supplied proof
certificate rejects at core checking. Complete proposition syntax is checked
against the original source-derived definitions, including every ordered mask.

The isolated verified source is promoted without modification. The promotion
audit confirms all 743 Rust/Cargo source files and all 173 relevant fixtures
match the tested bytes; those identical local tests are therefore retained.
All 45 complete metadata records match the preceding recipe after removing
only `certificate_sha256`. Evidence is preserved under
`verification-logs/concrete-operation-transport/`.

Both fixed unchanged Go/Rust binaries accept all 45 exact new certificates
with matching module/declaration/axiom counts and all three hashes. All 182
stages are terminal: 90 positive backend executions, 90 hash-corruption
rejections and two wrong-normal-value core rejections. Every axiom summary is
zero. The independent audit verifies all exact inputs, fixed binary hashes,
raw reports and stderr, and confirms current source/fixture hashes still match.
The separate earlier Money probe is retained as historical evidence; the full
45-context replay is authoritative. See
`verification-logs/concrete-operation-transport/checks/receipt-audit.json`.

The requested Linux replay passes both targeted tests afresh, Clippy and both
format checks at fixed public source `c7421af8`. The detached checkout
matches all 916 source/fixture Git blobs and all three harness blobs. The terminal
audit verifies every test log and the test binary, requires a clean checkout,
and confirms all 91 exported metadata/positive/negative files match local bytes.
The exact fetched archive, SSH execution receipt and independent audit script
are retained in `verification-logs/concrete-operation-transport/server-linux/`.
The earlier `aac0ea64` receipt remains separate historical evidence.

The original pattern-refinement source capability, internal construction states,
source ownership and the application-owned Money.create currency predicate
remain open. These operation proofs do not establish source execution, complete
application assembly or W09 acceptance. W09 remains In progress, W10-W12 remain
Blocked and `scripts/check-fast.sh` remains deferred to T06-W12.
