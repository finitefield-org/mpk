The [slice builtin typer](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/slice_builtins.rs)
adds original-source make, append and copy to the existing flat scalar/aggregate
reader. It resolves the real predeclared binding, so an original function named
make/append/copy keeps its own static signature instead. Original type syntax,
canonical nominal/alias keys, exact constants and expected/index contexts are
the only typing inputs. No native compiler object or callback enters production.

Slice make retains its actual named/alias result and checks its one/two size
operands in the original int index context. Constants must be nonnegative and
int-representable; two known constants must satisfy length<=capacity. Signed and
uint64 runtime sizes remain type facts with later definedness obligations. Both
implicit and explicit capacity syntax are retained; later practical admission
still applies the frozen construction rules. No extra small allocation cap is
introduced into Go type checking.

Append preserves the first slice's exact type, checks each value against its
actual element type and each spread against the matching unnamed slice type.
Its byte/string special form requires the actual byte element, rather than a
separately defined byte-width integer. Zero added values, contextual shifts,
nil spreads, named/anonymous aggregates and single tuple-producing calls retain
their own contexts. Actual200-value argument lists gain no small helper/arity
cap. Copy requires identical slice elements or the actual byte/string form and
returns nonconstant int. Go1.25's nil destination in byte/string copy is
materialized to the corresponding byte slice. The
[Go specification](https://go.dev/ref/spec#Appending_to_and_copying_slices)
and pinned source are reference rules; actual checker/compiler outcomes below
remain distinct.

Read-only GoSliceBuiltinHeader records keep the original source/call/function/
argument nodes, operation, materialized slice/element/result IDs, spread flag
and explicit-capacity flag. Both general-value and scalar API projections retain
them, including nested static calls and an opaque callee body. Original tuple
argument syntax is retained without inventing source nodes. Headers are type
and provenance facts; they prove no allocation, zero initialization, capacity
bound, non-reallocation, overlap behavior, permission, NoAccess, lifetime,
invocation state or source/program execution.

The [actual Go1.25 reference](../../../../../../go-tools/go2vir/practical_slice_builtin_reference_test.go)
saves whole original sources/candidates before gc/amd64 parser/types checking.
All1659 cases remain immutable:510 exact positive type/value/addressability
observations,1143 agreeing original rejections, three valid map/channel/float
forms outside private v2 keys and three actual checker panics. Panics preserve
message/stack and are never counted as Go rejections. The first unrecovered
process's complete partial observations and missing later outputs stay intact.

[Same-source compiler controls](compiler-controls/result.json) compile four
selected original inputs without executing applications. The actual
append(PairBytesString()...) control compiles. Two type-accepted raw-nil copy
controls produce compiler internal errors, and raw-nil string append produces
both a checker assertion and a compiler internal error. Consequently type-info
agreement is not compiled-source admission or an application proof. The pinned
SDK is preserved exactly; no compiler repair or version change is hidden here.

Two new Rust roots pass, including owned nested builtin/static headers, exact
original argument locators, scalar projection and foreign-owner rejection.
All three existing scalar roots and both aggregate roots pass: the original
4985 scalar cases retain2776 exact positives/2205 rejections/four unmodeled
float forms; the original950 aggregate cases retain474 exact positives/463
rejections/13 unmodeled forms. Original20000-node contextual work stays flat.
Strict Clippy, scoped Go vet and changed-file formats pass. Existing constants,
types, scopes, signatures, parser/lexer and ordinary semantic code remain exact,
so unrelated tests are not repeated.

The [receipt](verification.json) retains ten epochs/7723 complete preimages and
19 scoped executions, including three failures, plus four compiler observations.
Six lossless transports preserve24139 original files/80622279 bytes; the
[literal audit](original-byte-audit/result.json) checks every byte and mode.
The formatter's initial summary-map syntax failure, original checker crash and
two missing nil/byte copy-context differences remain with their full inputs and
outputs. [Input review](input-review.json) uses unchanged127 matchers/four scopes/
39 seals/51 exclusions, with no new exceptions. SDK14496/frontend189/frozen255
and the ordinary448-case paired basis stay exact.

Methods/interfaces, complete expression/body/assignment-statement and call-state
checking, ordinary source evaluation/carrier witnesses, full module/import/
build/capture/context/resource closure, U/Plain/weights/final counts and actual
storage/allocation/permission/lifetime/state/program proofs remain due. Full56
foundation/registry, nativeLinux/generalnamespace/selectedclosure/all169/T02–T11
remain open. Go/Rust only; R01/R06/W01 stay open, public practical profile remains
inactive and the T02 whole gate stays finalW04: `./scripts/check-fast.sh`.
