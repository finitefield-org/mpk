The [original assignment reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/assignments.rs)
checks actual `=`, `:=`, compound-assignment and `++`/`--` nodes from owned
original syntax. It checks every writable destination, full parallel/tuple
arity, contextual constants/shifts, nominal compatibility, blank destinations
and short-declaration reuse/shadowing. Parameters, original imported variables,
struct/array updates and addressable slice elements retain their actual types.
Constants, string elements, array/struct temporaries and other unwritable
expressions reject. Pointer/map targets remain outside this reader.

These follow the [pinned Go1.25 assignment rules](https://raw.githubusercontent.com/golang/go/go1.25.0/doc/go_spec.html).
The flat engine retains original left-hand nodes and ordered destination types;
readonly GoAssignmentTyping distinguishes actual new declarations from reused
variables and preserves all original call headers. Destination-slot facts are
nonconstant; discarded slots still require a valid default value type. Checking
one statement does not typecheck unrelated statements or callee bodies. Original
call locators identify syntax dependencies; evaluation, single execution of a
target/call, two-phase stores, aliasing and runtime permissions need their own
ordinary source/state proofs.

The [native reference](../../../../../../go-tools/go2vir/practical_assignment_typing_reference_test.go)
saves whole actual sources/candidates before original Go1.25 parser/typechecking
at gc/amd64 sizes. All1433 observations remain distinct:475 exact positive types/
target provenance,951 agreeing rejections and7 valid outside-reader forms.
Independent Go type objects agree on903 ordered slots,215 actual new short
bindings,72 blank destinations and17 original static caller/callee/body records.
There is no new small assignment/parameter cap: both parallel and short lists
have200-destination controls; function results still exercise16/17 boundaries.

All three new Rust roots pass, including owned source/header retention, foreign
locators, one tuple-call header and20000-node contextual work. The initial deep
fixture omitted spaces and produced decrement tokens; its actual Source failure
and all original buffers remain. Only that fixture changed before the passing
run. Ten affected existing scalar/aggregate/builtin/method roots pass with all
original4985/950/1659/461 outcomes unchanged. Existing builtin checker crashes
remain separate from rejections. Strict Clippy, changed formats and scoped Go
vet pass. Unchanged constant/type/scope/signature/parser/ordinary tests are not
repeated; the selection rationale accompanies every check.

The [verification](verification.json) retains five epochs/3902 complete source
preimages and14 executions, including the initial failed control. Seven lossless
transports preserve24739 original files/77332904 bytes with a [literal byte/mode
audit](original-byte-audit/result.json). [Input review](input-review.json) uses
unchanged127 matchers/four scopes/39 seals/51 exclusions and no new exceptions.
SDK14496/frontend189/frozen255/ordinary448 basis stays exact.

Return/control/whole-body checking, source evaluation and call/state linkage,
frozen direct-selector and closed-error/Plain admission, capture/context/resource/
storage/program proofs, full56 foundation/registry and nativeLinux/generalnamespace/
selectedclosure/all169/T02–T11 remain required. Go/Rust only; R01/R06/W01 stay
open and the public practical profile stays inactive. The full T02 gate remains
final W04: `./scripts/check-fast.sh`.
