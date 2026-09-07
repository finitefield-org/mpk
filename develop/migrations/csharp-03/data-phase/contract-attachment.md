# W14 private data handoff and contract attachment

This describes the private CSHARP-03-T03-W14 integration. Verification and
completion evidence are recorded in `integration-review.md`.

The Roslyn handoff contains the exact compilation ID and a path-ordered
`input_files` inventory, including source and sidecar kind, raw SHA-256 and
byte count. Native import checks this inventory against the selected captured
inputs before accepting any declaration, body, default or operation facts.
Logical declaration IDs remain independent of source paths and content;
provenance is separately checked against those captured bytes.

`CaptureSelected` resolves stored-member IDs from actual source declarations
and uses the actual selected semantic-binding sidecars to call the existing
W12/W13 source validators. W14's construction call defers sidecar attachment
to native strict parsing; it does not create discharged invariant claims.
Native code validates every selected type/method contract against source
identities, stored members, signatures, the sole derived closure and the full
frozen expression union before artifacts are emitted.

The private `mpk.vir.v2` module has a required `data_contracts` array immediately
after `binding_commutations`. Entries are the complete canonical JSON texts of
validated type/method contracts, ordered by schema then contract hash. Encoding
them as JSON strings preserves the original canonical field order and lone
UTF-16 surrogates. The array participates in the VIR hash. The importer requires
the actual captured source handoff whenever data contracts are attached,
rederives the binding closure, reruns contract attachment, and compares the
complete array to the captured sidecars. Empty, substituted or stale contract
arrays cannot be rescued by recomputing the VIR hash.

VC generation retains the VIR linkage and creates an unresolved obligation
subject for every retained contract. Type contracts use the construction/type
invariant owner; method contracts use the data owner, with exceptional clauses
also assigned to the exceptional-control owner. No contract or binding proof
is discharged at this stage. Full proof construction remains T06-owned.

The type-contract `default_eligible` field is checked against structural
availability of a well-typed CLR zero. Public and construction invariants
remain separate obligations. Source metadata `actual_default` contains the
complete recursive CLR zero, including null references and zero enum carriers
that may be ineligible for public default construction. It never substitutes a
placeholder source-type reference for a nested product. Source public-default
flags do not prove a declared invariant.

The selected-source test matrix and the selected-sidecar request/response
fixtures are produced by the pinned offline compiler. The sidecar fixtures
include both compiler rejections and successful fact capture followed by
native contract rejection. A compiler fact capture alone is not a successful
frontend artifact. The Linux W14 test replays both fixture sets twice in fresh
isolated compiler builds and checks the exact bytes before native import.

Each callable also retains `data_steps` from the owning string/numeric/domain/business source
validators. A step records its body node ordinal, source-ordered operand node
ordinals, semantic argument positions, family, operation and explicit rounding
mode. The normalized logical body hash is unchanged by these provenance links.
The source operation matching checks exact span, kind, implicitness and fully
qualified type; ambiguity rejects. Native import checks preorder containment
and intrinsic constants, and emission uses the existing relation factories.
`decimal.round.<mode>.<arity>` selects the original numeric relation with those
explicit parameters; `string.equals.instance.ordinal` retains the receiver null
check already present in the string relation. Both participate in operation
and VIR hash preimages and cannot silently select another mode or overload.

The required `source_obligations` array follows `data_contracts`. Each row
retains family, source site, kind, concrete type ID when applicable, member/
array subjects, predicate text, `discharged: false`, owning declaration ID,
and original source hash/byte range. Its complete contents participate in the
VIR hash and must equal the independently imported captured facts. Proof-only
nullable carriers are added to the exact source roots through the existing
closed-type parser and sole specialization engine. Binding obligations refer
to the original source projection, whose final semantic target is derived by
that engine. Intermediate W12 targets do not bypass nested business bindings.

Source obligation and canonical contract text is data rather than executable
VIR vocabulary. Generic-token rejection scans executable fields; these two
required data fields instead undergo exact original-fact/contract attachment,
strict field/type validation and concrete closure checks. Predicates such as
`index < length` are therefore retained without admitting generic operations.
VC generation creates one pending subject for every source obligation.

The T02 concrete construction-action validator remains available for its model
vectors. W14 source arrays all use the ordinary SSA construction path below.
Both paths enforce unique ownership; inactive bookkeeping may be retired, while
live state cannot disappear or reuse an allocation identity.

`nullable.coalesce` emits a presence branch and evaluates its right operand
only on the absent edge. `GetValueOrDefault(fallback)` evaluates its argument
before the shared `value_or` operation. Lifted operations evaluate both source
operands before applying the existing W12 relation; checkedness is explicit in
`lifted.<primitive>.<operation>.<checkedness>` and independently validated.

Reference dereference selects `reference.value.<option-instance-id>`, restricted
to reference payloads and the existing presence/payload relation. Its inactive
arm throws `System.NullReferenceException`; it cannot substitute the
`System.InvalidOperationException` of Nullable<T>.Value. Flow-narrowed local/
parameter references retain explicit extraction. Conditional access evaluates
its receiver once, reads the member only on the present edge, then constructs
the appropriate Some/None result through the existing option definitions.

The symbolic array route invokes the same engine-expanded
`sequence_construction` operations with ordinary SSA length/index operands.
It is enabled only with original captured data facts and cannot share a function
with concrete construction-action states. Its importer checks linear SSA
ownership/lifetime separately from the pending bounds/bitmap/completion
conditions; no array evaluator or specialization algorithm is duplicated.
Construction types cannot be function parameters/results or operands/results
of ordinary operations. Phi versions must preserve allocation identity, and
all predecessor states must agree after applying those phis.

The sole Exit may contain Discard actions identifying allocation origins. In
the symbolic route these are conditional incoming-exception-edge cleanups:
each live origin is discarded before states merge, and the declared set must
be exactly the union of live incoming origins. Unknown, extra, missing or
duplicate origins reject. Normal-path Discard actions consume the current SSA
version. The concrete route continues to validate its concrete state machine.
Normal and exceptional successors cannot coincide in the symbolic route.

Current source emission handles fixed/dynamic allocations, complete initializers,
partial and symbolic initialization, reads, Length, scalar updates, alias/call/
stored-member/return publication and compatible active-state branches. Object
initializer transactions use the private owned protocol described below.
The complete existing-stage source replay is recorded in `integration-review.md`.

## Additional source ownership handoff

Array simple assignments retain W07's `fill`, `rewrite`, or `fill_or_rewrite`
recipe in required typed data steps. The allocation default flag is derived by
the shared default generator; ineligible elements start uninitialized, including
zero-length arrays whose completion predicate is already true. Array allocations
and initializer fills share one ordinary SSA path, so initializer exceptions have
explicit cleanup. The older concrete-action importer remains a T02 model path.

`construction.complete.<closed-instance-id>` is a private Data observation of
`SequenceConstructionState`'s existing completion predicate. Its sole argument
is the exact engine-derived construction instance, its result is bool, and it
has no exceptional checks. The independent ownership walk admits this read-only
use only for the current live version. The source emitter branches to the frozen
`rewrite` operation when complete and `fill` otherwise; both advance the origin's
version once. The observation never freezes, publishes or alters initialization.

Source-level scalar update traits include operator, checked/lifted flags and
postfix status; compound assignments also retain the compiler's input/output
identity-conversion flags. Integer updates compose existing scalar arithmetic
and conversion signatures, with postfix returning the saved old value. No new
arithmetic evaluator is introduced. Property assignment targets do not add
getter-call reachability edges. Nullable receiver extraction for ordinary calls
occurs after argument evaluation, and for indexing after index evaluation.

W03's type closure retains every property getter as a declaration root even when
no selected method reads that property. Source capture excludes write-only
assignment targets from getter call edges, while native source and VIR closure
retain those property roots from the original captured facts. Emission includes
these getter bodies without inventing a runtime invocation at an assignment.
Required `is_property_getter` source facts come from the compiler MethodKind;
an ordinary method's `get_` name alone cannot introduce a property root.


## Constructor transaction and source closure

Each captured callable retains exact W05 `initialization_plans`, including
original body ordinals, constructor identity, ordered member assignments and
normal-exit Must/May masks. Native import reconstructs these from the actual
constructor bodies, including delegation and early returns. Synthesized
parameterless constructors have an explicit `is_synthesized_constructor`
marker, zero parameters, the hashed empty body and owner-declaration source
provenance. W04 retains these constructors as declaration proof roots, alongside
W03 getters, without inventing caller invocation edges.

The private `constructor_execute` signature carries an owned transaction as its
first parameter and normal result. Source contracts still use its original
logical signature. Its representation uses existing boundary-field slots and a
balanced ordered-entry product: Missing has no payload, while nullable None is
an assigned Value. Defaults use the shared generator. Only consuming
`object.finalize.<owner>` produces an ordinary source value.

`object_protocol` binds begin, constructor, ordered writes and finalization to
the exact source plan. The importer separately tracks origin identity, the
current SSA version, assigned-member masks and allowed uses. Same-owner
transactions cannot exchange states. Exceptional discards equal the union of
live incoming origins; a callee discards its transferred receiver on failure,
and the caller cleans up its remaining origins. Normal paths cannot leak live
state. Private operations require original source facts.

Nullable annotation widening composes the existing option constructor at
returns, calls and member writes. Restricted interpolation evaluates all parts
in source order and invokes the existing interpolation relation once, with an
exact string/char shape signature. It does not introduce intermediate output
bounds or change exception order. Source-call dependencies are scheduled in
DAG order; bounded source expression traversal uses a 64 MiB worker stack so
accepted source limits do not depend on a caller's stack size.
