# Private construction storage type equivalence proofs

This partial T06-W09 checkpoint adds an opt-in type route for the six original
sequence-construction instances. It supplies all 87 original type equivalence
proof candidates across 45 source contexts; the old route remains at 81 with
its six internal instances pending. The new route has no pending type instances,
but every private storage domain explicitly retains ownership_pending=true.
All 987 application proof IDs remain pending. W09 stays In progress and W10-W12
stay Blocked; the full T06 gate remains deferred to T06-W12.

## Domain and proof construction

The private three-field layout retains the complete 32-bit length, 16,384
element cells and initialization bitmap. Length must be at most 16,384.
Initialized cells use the existing recursive public child domain and complete
source clauses. Uninitialized cells must have all-zero physical storage; their
element need not have a default public value. Both contribute their proper
logical-cell counts, including the enclosing cell, with saturation at 65,537.
Field padding, the unused fourth role, inactive cells and bitmap tail must be
zero. The count/validity definitions enforce the complete 65,536-cell bound.

Each instance must match the exact original W06 construction type descriptor
and element dependency. Builder.resume retains every old term/declaration and
all public domains, source clauses, definitions, conditions and historical pins.
The added wrappers reconstruct every complete original concrete_type_equivalence sequent
and both original domain/definition operands. Ordinary Std.Eq reflexivity proves
these definitional equations without additional assumptions or axioms. Strict
imports regenerate the exact source-specific metadata and certificate bytes.

The domains describe private storage only. Origin/current SSA version, lifetime,
exclusive ownership, initialization/publication capabilities and default
eligibility remain separate obligations. No ownership bit, public capability,
kernel rule, foundation schema, Certificate v0 rule or acceptance behavior is
introduced. All 25 remaining operations, seven original is_binding refinements,
native execution establishment and application assembly remain pending.

## Verification and direct review

Four targeted local tests pass: all 45 source contexts and 87 supplied proofs,
six private-domain contexts with 247 complete observations, the affected legacy
type proof regression and all 45 historical type pins. The shared proof helper
and serialized type metadata changed, so both legacy consumers are included.
Other generators and operation algorithms are unchanged. Crate Clippy with
warnings denied, crate format and both changed support-file format checks pass.

The tests independently reconstruct original W06 sequents and inspect the full
theorem operands, premise shape and old certificate prefix. They exercise strict
imports, changed metadata/context, size bounds and actual unchanged Rust kernel
acceptance with zero axioms. A well-typed always-true private type definition
remains accepted without proof candidates and is rejected at core checking
when the supplied proofs are present. The earlier wrong-proof negative remains.

The semantic matrix covers full length 16,384, complete high length bits,
every padding selector branch, both ends of unused storage and inactive cells,
inactive bitmap indices and nonzero uninitialized cells. Four exact source
products containing nullable strings reach independently counted totals
65,535/65,536/65,537, checked against separately generated child public domains.

The initial semantic failure was a nominal test fixture error: the source
element is a product containing a nullable-string member, but the fixture used
the nullable value directly. The repaired test preserves the exact source
product. No production or proof-generator bytes changed after the first proof
export. The failed attempt and the later prelaunch missing-catalog failure are
retained with their actual results. The final 747-source/173-fixture audit passes,
all 92 proof exports exactly match the first passing proof stage and all 91
legacy exports match the preceding published type corpus.

Both unchanged checkers pass the 26 affected stages. The independent terminal
audit verifies all 184 stages, including 158 retained only after exact current
input/binary/raw-report/stderr comparison. All positive hashes and module/
declaration/axiom counts agree with zero axioms; both checkers reject all hash
mutations and both typed wrong proofs at core checking. Their actual inputs
exactly match the final proof exports after the semantic test fixture repair.
Requested Linux verification remains pending an exact published-source terminal receipt.
See verification-logs/construction-storage-types/.
