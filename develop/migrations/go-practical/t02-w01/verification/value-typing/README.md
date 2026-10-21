The shared [original value typer](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing.rs)
now handles private v2 array, slice and struct values as well as scalars. The
original scalar APIs wrap the same flat request engine. Full canonical type
keys, normalized underlying edges, original declarations, owned source bytes
and actual static call/callee/body locators remain available; none supplies a
body, evaluation, invocation-state or registered-program proof.

[Aggregate rules](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/aggregates.rs)
retain nominal identity and transparent aliases, Go's named/unnamed assignment
rule, explicit conversions that ignore struct tags, string/byte/rune-slice
conversions and identical-element slice-to-array conversions. Array/struct
comparability follows actual fields, including blank fields. Slices compare
only with an actual nil operand. Typed nil-slice conversions remain distinct
from that operand. An untyped nil expression retains its original marker while
its value type comes from the actual expected destination; a declared variable
retains its declared type and is nonconstant/addressable.

[Fields and accesses](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/members.rs)
check original direct/promoted fields, visibility, same-depth ambiguity and
addressability. Repeated embedding paths retain multiplicity, so a diamond
cannot become unique through type deduplication. A matching method, resolved through actual receiver type denotation including
value/pointer aliases, prevents
silently selecting a deeper field and remains explicitly unsupported. Array,
slice and byte-string indexing/slicing enforce integer type, constant sign,
int representability, known static bounds, slice ordering and full-slice rules.
Untyped indices receive the original int context before a nonconstant shift is
materialized, including large shifted constants and runes. Dynamic uint64
indices keep their actual type; no dynamic bound or permission is proved.

[Composite literals](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/literals.rs)
check original keys, duplicate indices/fields, positional counts, visibility,
expected element/field types and inferred ellipsis-array lengths. Type omission
is allowed only in actual array/slice element context. Literal field values
cannot borrow that omission. Array length256, struct64 fields and aggregate
depth16 remain fixed, including synthesized slices/ellipsis arrays. A huge
slice literal can be well typed without having a modeled allocation; typing
introduces no extra small storage/allocation cap. Array/slice range variables
and aggregate static-call inputs/results use the same original type graph.
Results remain0–16; implementation work/parameter arity gains no small cap.

Array len/cap and string len retain actual int constants only under Go's
constant rules, including the absence of nonconstant calls in array operands.
Operand typing still occurs. The existing exact constant evaluator's semantic
bodies remain byte-for-byte unchanged: only an additive int-length constructor
is introduced. Consequently the affected existing scalar roots are selected,
while unrelated old constant/parser/type/signature/ordinary tests are retained
through exact source/evidence correspondence without unnecessary repeats.

The [actual Go1.25 reference](../../../../../../go-tools/go2vir/practical_value_typing_reference_test.go)
saves whole original source/candidate bytes before pinned parser/types checking
with gc/amd64 sizes. Its test-only importer checks original helper bytes; no
native object/callback enters production typing. Native untyped nil and the
actual checked destination type are observed separately. All950 original final
cases remain immutable:474 exact type/constant/addressability/nil-context
outcomes and463 actual Go rejections agree. Thirteen valid forms remain explicit
outside this reader: method values, string range, arrays beyond256, struct65
fields and aggregate depth17. The final six alias-receiver cases extend all earlier944 actual native
input/outcome rows unchanged. Intermediate848/944-case observations and failed
partial outputs remain untouched.

Two new Rust roots and all three affected scalar roots pass. The old4985 cases
now have2776 exact positives,2205 agreeing rejections and four explicit float
forms outside the reader; aggregate len moves from unmodeled to exact. Original
20000-node flat contextual work, ownership/foreign-locator and opaque aggregate
header controls pass. Strict Clippy, scoped Go vet and changed-file formats
pass. The [receipt](verification.json) retains26 source epochs/20013 complete
preimages and48 scoped executions, including four original failures. Additional
[auxiliary bookkeeping failures](auxiliary-failures.json) retain their original
drivers separately. Fifteen lossless
transports preserve50550 original files/154017214 bytes. The
[literal audit](final-alias-original-byte-audit-corrected/result.json) compares original bytes
and modes; missing failed native/comparison outputs are never reconstructed.
[Input review](input-review.json) uses unchanged127 matchers/four scopes/39 seals/
51 exclusions with no new exceptions. SDK14496/frontend189/frozen255 and the
ordinary448-case paired basis stay exact.

This is a source value-typing component. make/append/copy, method/interface and
complete expression/body/assignment-statement typing, call state, original
source evaluation/carriers, complete module/import/build/capture/context and
resource closure, U/Plain/weights/final counts and storage/permission/lifetime/
state/program proofs remain due. Full56-family foundation/registry, nativeLinux/
generalnamespace/selectedclosure/all169/T02–T11 remain open. Go/Rust only;
R01/R06/W01 stay open, the public practical profile remains inactive, and the
T02 whole gate stays finalW04: `./scripts/check-fast.sh`.
