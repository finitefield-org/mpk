The [original return reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/returns.rs)
checks actual ReturnStmt nodes against their nearest enclosing FuncDecl/FuncLit.
Ownership comes from original syntax edges, with per-source/function caches;
no caller signature or byte-containment guess supplies the expected results.
It checks ordered0..16 result assignments, contextual constants/shifts, one
multi-result call, named/blank empty returns and original lexical shadowing.
The [pinned Go1.25 rules](https://raw.githubusercontent.com/golang/go/go1.25.0/doc/go_spec.html)
allow blank result names and reject void calls as return values; a naked return
with another visible variable, type or constant hiding a result rejects.

Readonly GoReturnTyping retains original function/type/body locators, complete
receiver/input/result slots, actual result-variable bindings and all original
call headers. Result-slot facts are nonconstant/nonaddressable types. They do
not assert zero initialization, the value of a named result, invocation state
or runtime behavior. Nested function ownership is a type fact and grants no
closure admission. One return does not typecheck unrelated statements or the
whole package, prove control coverage or termination, or check callee bodies.

The [actual Go reference](../../../../../../go-tools/go2vir/practical_return_typing_reference_test.go)
saves full original sources/candidates before parser/typechecking at pinned
Go1.25 gc/amd64 sizes. All630 final observations remain distinct:150 exact
positive type/provenance outcomes,470 agreeing rejections and10 valid forms
outside this reader. Native object identities and physical syntax agree on
2214 complete signature slots,448 result slots,205 named result variables,
39 implicit returns,8 nested-function returns,4 method returns and17 static
caller/callee/body records, including3 bodyless callees. Seventy original
parameters and16/17-result boundaries preserve the unchanged limits.

Three new Rust roots pass, including owned source/header retention, invalid
locators, shadowed versus explicit returns, opaque joint results and20000-node
contextual work. Thirteen affected old scalar/aggregate/builtin/method/assignment
roots pass with original4985/950/1659/461/1433 outcomes unchanged. Strict Clippy,
changed-file formats and scoped Go vet pass. Unchanged constant/type/scope/
signature/parser/ordinary tests are not repeated; each execution retains its
selection reason. Earlier builtin checker crashes remain separate observations.

All original failed epochs remain. The first reference build had a local
variable-name collision and emitted no native cases. The first630-case corpus
had one nil/error helper query with an unused import: the whole Go package
rejected while the partial return-type query succeeded, with no program-proof
authority. The corrected whole original source references that import by type;
only case583's native acceptance changes. Both complete native corpora and the
failed Rust output remain, and no production type rule changed to hide it.

The [verification](verification.json) retains nine epochs/7062 source preimages
and20 executions/two initial failures. Nine lossless transports preserve24054
original files/78163074 bytes, with a [literal byte/mode audit](original-byte-audit/result.json).
[Input review](input-review.json) uses unchanged127 matchers/four scopes/39 seals/
51 exclusions and no new exceptions. SDK14496/frontend189/frozen255/ordinary448
basis remains exact.

Actual condition/control/whole-body checks, static selected closure, frozen
v1 selector and closed-error/Plain admission, source evaluation/call-state/
capture/context/resource/storage/program proofs, full56 foundation/registry,
and nativeLinux/generalnamespace/selectedclosure/all169/T02–T11 remain required.
Go/Rust only; R01/R06/W01 stay open and the public practical profile stays
inactive. The cumulative T02 gate remains finalW04: `./scripts/check-fast.sh`.
