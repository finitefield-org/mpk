# T06-W09 internal unit 5: source normal-step composition (scoped checks passed)

The emitter composes exact source frames, successful slot transfers, native
normal invocation predicates and native literal-result predicates. Every
component is applied to indices in one shared binder list. SSA values are
resolved to their actual parameter, phi, literal, invocation-result or caught-
exception definition point from validated VIR; types must match. The input and
output source-slot snapshots stay distinct even at a single native node.

The anchor's native nodes must form one complete normal path from its entry to
its exit. Every literal and invocation along that path is accounted for. Missing
call/data semantics, internal control/phi selection and exceptional nodes leave
the step pending. A pending step never receives an execution predicate. For a
complete step the predicate is the conjunction of every component, not a guard
implication. The composed Lam/Pi depth includes each formal's cube type as well as the
number of shared arguments. Depths above 256 retain all indexed components
instead of an oversized binder. Current examples use at most 80 arguments.
Direct review found and fixed the original argument-count-only check; the exact
256/257 boundary test passes and all 18 candidate metadata/certificate pairs
remain byte-identical. See `binder-depth-verification.json`.

Normal entry phi values, source results, branch conditions and returned values
retain explicit shared argument indices for subsequent control composition.
The step assumes its incoming state/SSA environment; it does not establish
reachability, incoming-edge selection, inter-step state transport, exceptional
execution, alias effects, or application VCs. Existing ownership-scoped guards
and their retained proof dependencies are reused at their exact invocation.
Function-entry, handler, unreachable and alias-update frame reasons remain
explicit, as does the unsupported constructor call.

The original 18 source contexts contain 703 source nodes. Independent coverage
checks require all 579 nodes with complete existing normal semantics to produce
normal steps. Exactly 124 records remain pending: 18 source entries, 93 exception
nodes, 10 unreachable nodes, two alias updates and one non-data invocation.
The largest complete metadata file is 13,441,834 bytes, below the frozen bound.

Runtime checks independently compare each SSA origin and each component's
original arguments, evaluate the composed relation and its constituents, and
mutate each argument. A specific cross-component test changes an increment's
literal operand and its result together: the arithmetic relation still holds,
but the source step must reject the changed literal. Metadata mutations reject.
These tests execute ordinary terms; runtime results are not emitted as axioms.

The first runtime attempt reached its explicitly configured 600-second evaluator
budget without a semantic verdict. The exact same corpus then ran without
evaluator time/step limits and passed in 8,193.415 seconds. No cases or mutations
were removed. Its 18 contexts execute 20,584 observations: 9,837 true and 10,727
false component outcomes across 7,160 shared arguments. The completed runtime's
36 candidate files are byte-identical to the independently generated metadata
candidate and the promoted fixture. Fourteen preservation/slot checks retain all
previous terms, declarations and metadata. Inventory and targeted lint/format
pass. All 15 distinct candidates pass same-byte Go/Rust checking with matching
reports, zero axioms and hash-corruption rejection; see
`checker-verification.json` and `runtime-unbounded.json`.

This scoped normal-step candidate is now promoted. Unit 5 and W09 remain
incomplete because entry selection, inter-step transport, exceptional/alias
execution and complete application proof assembly remain open. W10 is blocked,
and the whole T06 gate remains with its final W12.
