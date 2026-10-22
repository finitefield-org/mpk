# Original contract-expression types and namespaces

This Go/Rust-only T02-W01 checkpoint starts at
`4b6ded3931d37aa5b9fec44244d097ab63e29f2b`. R01/R06/W01 remain open and the
public practical profile remains inactive.

The [expression constructor](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_overlays/expressions.rs)
binds original entry sidecars and uses the existing finite source selection and
global error universe before deriving clause types. It covers all70 closed
source heads. Logical Bool/Nat/Int remain distinct from Go value types. Exact
source type IDs retain nominal distinctions; aliases erase only through the
original type declarations. Direct fields retain the actual owner and ordered
ordinal. Clauses/invariants require logical Bool and explicit variants require
Nat. Word literals check the full signed/unsigned endpoints through64 bits;
logical integer strings have no private digit ceiling. Original source/overlay
limits and all kernel/evaluator limits are unchanged.

Contract-only qualified type references resolve to actual original package
TypeSpecs. They add compilation dependencies, never error tags or selected
method bodies. Their expanded Root/Plain/depth/weight graph is checked against
the same existing global universe. The tests include a contract-only type with
an unsupported Error body: successful type reference does not select that body.
Mentioning a compatible error case in a clause cannot extend the global U.

Every loop header starts in its own implicit lexical scope at the body boundary.
Body-only declarations, later declarations and sibling loops stay outside it;
variables hidden by nearer constants or types cannot be recovered as outer
variables. Original checked body metadata supplies inferred local types without
checking the same body again per expression. Signature slots supply receiver,
parameter and named-result bindings. The native comparison uses actual
`go/types` scopes and physical declaration offsets, including shadowing, range
variables, methods, CRLF, Unicode and a forged line directive. The underlying
language rules are described by the [Go specification](https://go.dev/ref/spec#Declarations_and_scope).

Input observations retain entry-argument positions, results are available only
at the same invocation's return, and locals/range observations reference their
own original header. Quantifiers use fresh lexical Nat binders; lower/upper
bounds are checked outside the new binder and references retain de Bruijn
depths. Binder names disappear from the typed derivation, so alpha renaming
preserves the expression. `old` moves the whole nested observation into the
entry snapshot, rejects nested snapshots and makes current locals/results/range
observations unavailable. Heap observations on element/sequence operations
retain their complete entry/return/header namespace. In particular,
`old(element(input, index))` and `element(old(input), index)` remain different.

This is a read-only internal typed tree with owned original bytes and source
node references. It creates no capture-dependent binding, loop, scope, contract
or context IDs, and exposes no deserializer for caller-provided typing facts.
Missing loop annotations remain visibly absent. Default/callee contracts and
default variants are not generated here. A typed division, index, reslice,
payload projection, signed-to-Nat bridge or concatenation still needs its
definedness proof. Tests explicitly preserve those pending obligations instead
of treating type correctness as runtime safety.

## Verification boundary

[The local record](verification.json) retains complete prelaunch source inputs,
actual drivers, original failures and raw outputs, with target-selection reasons.
The Go observer captures intact original source/sidecar buffers and removes the
original sidecar namespace after sealing. Rust receives those bytes plus a Bool
seed, not compiler ASTs, expected bindings or native type tables. All original
header names, type IDs and declaration offsets are compared after construction.
The source-only witness extraction retains all70 historical source expressions
with one explicit package-prefix renaming; the historical scope and native type
IDs are not constructor inputs. Its original frozen file remains unchanged.

The new six-root Rust suite checks the complete vocabulary, exact original
headers, wrong types/phases/scopes, nominal and alias cases, integer endpoints,
contract-only domains, snapshot differences and partial operations. Affected
existing overlay/scope/domain/finite/static-selection suites and scoped
Clippy/vet/formats accompany it. The Go production frontend is unchanged, so
rebuilding it or rerunning unrelated semantics would not add change coverage.
The unchanged original SDK, frozen and ordinary bases remain retained.

The [scoped input review](input-review.json) uses the unchanged127 matchers,
four scopes,39 seals and51 configured exclusions. Every original preimage is
scanned. New-input findings and missing links are zero; four repeated basis
findings refer to the same two unchanged historical-scope locations. The first
review's premature skip of configured originals is retained, then corrected;
no scanner exception is added.15 source epochs/11799 preimages retain15 actual
executions, including three original failures.10987 native files/87255542 bytes
and their modes are preserved in the verified lossless archive. The14496-file,
202722007-byte original SDK still matches its earlier complete inventory.

Full application/stdlib and original capture/compiler/namespace/context
authority, whole-module optional sidecars/default contracts, canonical blank
field zero obligations and all entry/callee/store/frame/loop/state/runtime/
program proofs remain due. Full56 foundation/private registry/native Linux/
all169/T02–T11 remain due. The earlier frozen historical-scope scanner findings
are not resolved by this checkpoint: this is scoped change review, not strict
whole-repository acceptance. No matcher/seal/exclusion changes are made.

The T02 full gate remains deferred to final W04: `./scripts/check-fast.sh`.
No workflow, subagent, other-language test or intermediate equivalent full gate
is used.
