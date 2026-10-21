The [original method reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/methods.rs)
adds value-method calls, immediately called method expressions and actual
predeclared-error compatibility to the flat scalar/aggregate engine. Receiver
denotation comes from original type syntax and canonical identities, including
cross-file aliases and pointer aliases used as method barriers. Per-package
method/name and receiver caches avoid repeatedly scanning an imported package.
No native Go object, callback or caller-provided method table enters production.

Original method lookup respects direct fields, visibility, embedding depth,
repeated-path ambiguity and pointer-method shadowing. Named types do not inherit
their underlying type's methods; aliases keep actual identity. Value-receiver
calls check original parameter/result types, expected constant/shift contexts
and single multi-result arguments. Method expressions include the actual outer
receiver parameter. Parameters gain no small arity cap; results remain0–16.
Method values, pointer calls and interface dispatch remain unsupported. These
are [Go method rules](https://go.dev/ref/spec#Method_sets) and pinned-source type
facts, not compiled-body or invocation-state proofs.

Read-only GoSourceMethod and GoMethodCallHeader keep original call/function/
receiver nodes, selected and declared receiver types, original field-index
paths, source declaration/body locators and ordered argument/result IDs. Both
general-value and scalar projections preserve them. Promotion is represented
as type information; it does not relax frozen v1's direct/nonpromoted selector
admission. Interface/pointer/method-value restrictions remain separate.

GoErrorImplementation derives actual Error() string compatibility, including
field/method shadowing and ambiguity. Pointer methods do not make a value type
implement error. A distinct string result type or extra input/result slots do
not satisfy the signature. Original signature dependencies are kept separately
from actual calls; boxing, assignment or passing an error does not invoke its
Error body. The exact predeclared interface signature can appear through an
embedded error with no original source declaration. That type fact supplies no
static interface target, closed error universe or Plain-payload admission.
Explicit error conversion produces a nonconstant interface value; typed scalar
constants retain their source type/value in assignment context. Dynamic payload,
nil-interface, comparability/definedness and state proofs remain separate.

The [actual Go1.25 reference](../../../../../../go-tools/go2vir/practical_method_typing_reference_test.go)
saves every whole original source/candidate before parser/types checking at
gc/amd64 sizes. All461 cases remain observed data:143 exact positive type/value/
addressability and method-provenance outcomes,300 agreeing original rejections
and18 valid forms outside this reader. Real Go selections independently identify
69 supported method calls, including23 promoted receiver paths,18 method
expressions and8 bodyless declarations. Rust agrees on physical caller/function/
receiver and callee/body spans, selected/declared receiver keys, field paths and
complete parameter/result types. The initial classification hint that treated
all methods on a pointer-method-bearing wrapper as unsupported was corrected;
other promoted value methods are typed. Both original epochs remain intact.

Three new Rust roots pass. They distinguish unused Error signature dependencies
from actual method calls even when its unvisited body contains an unsupported
operation, retain opaque joint results and owned source/header bytes, reject
foreign owners and handle a200-call receiver chain without recursive evaluator
work. A70-parameter method and16/17-result boundary preserve the fixed limits.
All seven affected scalar/aggregate/builtin roots pass with unchanged original
4985/950/1659 cases, including the20000-node contextual basis. Previous builtin
checker crashes remain distinct from rejections. Strict Clippy, scoped Go vet
and changed-file formats pass; unrelated old semantic tests are not repeated.

The [receipt](verification.json) records nine epochs/6990 complete preimages and
22 scoped executions, all passed. Ten lossless transports retain31709 original
files/98995757 bytes; the [literal audit](original-byte-audit/result.json) compares
every original byte and mode. [Input review](input-review.json) applies unchanged
127 matchers/four scopes/39 seals/51 exclusions with no new exceptions. SDK14496,
frontend189, frozen255 and the ordinary448-case paired basis remain exact.

Complete source body/statement/call-state checking, static selected closure,
closed-error/Plain proof, source evaluation/carrier/module/import/build/capture/
context/resource closure, U/weights/final counts and actual storage/allocation/
permission/lifetime/state/program proofs remain due. Full56 foundation/registry,
nativeLinux/generalnamespace/selectedclosure/all169/T02–T11 remain required.
Go/Rust only; R01/R06/W01 remain open, public practical profile stays inactive,
and the T02 whole gate stays finalW04: `./scripts/check-fast.sh`.
