# T02-W01 independent scalar expression and initializer typing

The [Rust scalar typing engine](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing.rs)
derives value/initializer types from owned original syntax, lexical bindings,
source annotations and exact scalar constants. Flat pending work/cache states
separate raw types, default value materialization and original expected-type
contexts. The [operator component](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/operators.rs)
preserves canonical nominal identity, alias transparency, typed integer/bool/
string behavior and implicit assignment versus explicit conversion. Constant
operations reuse the exact existing512-bit/typed-range implementation; runtime
word operations retain types without being evaluated or constant-folded.

Untyped nonconstant shifts retain the original contextual rule. A uint64
assignment or argument can determine the shifted left operand's type before
arithmetic, while a comparison materializes its own operands and yields an
untyped bool. Count constants in runtime shifts retain the full uint64 domain;
the1074 compiler bound only applies to actual constant shifts. Signed runtime
counts remain typing facts rather than proved nonnegative values. Runtime
arithmetic does not become512-bit constant arithmetic, and constant zero
divisors still reject during typing. Bool/rune/string defaults and named-bool
comparison contexts remain distinct.

Nonconstant min/max materializes and defaults its result inside the builtin.
An outer assignment cannot change `max(1<<count,2)` from int into a defined
Word type. Fully constant min/max keeps exact constant precision/rune promotion
and representability behavior. Its existing evaluator and new typer share the
same extracted exact primitive. Existing5117-case constant evidence is rerun
through both affected Rust roots and retains4832 exact positives,281 actual
rejections and four explicit unmodeled valid Go forms.

Inferred and explicit shared ValueSpecs and short declarations check all RHSs,
blank positions, target arity and reused old bindings. Variable facts remain
addressable and nonconstant even when initialized by constants. Zero-initialized
scalar declarations retain their declared types. Queries for parameter/result
bindings use original field annotations. Scalar static call headers check
argument/default/nominal compatibility and0–16 result types, including a single
multi-result argument and shared result deconstruction. Actual call/callee/body
locators are retained; an opaque body supplies no source execution or call-state
proof. A separate single-value API rejects void/multiple results while the
result-list API preserves their arity. No caller AST or native object callback
enters the production typer.

The [actual Go reference](../../../../../../go-tools/go2vir/practical_scalar_typing_reference_test.go)
saves every complete source/candidate before real pinned Go1.25 parsing and
whole-file type checking at gc/amd64 sizes. Its test-only importer returns only
helper packages checked from those same original bytes. All4985 original cases
remain observed data. Rust agrees on2775 exact positive canonical type keys,
constant values and addressability facts, and2205 actual Go rejections. Five
valid Go cases remain outside this scalar reader: float/integral-float source
forms and aggregate-value len. They remain explicit unmodeled forms rather
than Go language rejection. Applications and initializers never execute in
the reference; type/constant observations are not runtime values.

The matrix crosses all integer operators/comparisons, nominal and native widths,
untyped literals/runes, large constants, shifts/counts and expected contexts,
bool/string operations, casts, len/min/max, static imports/calls, tuple0/2/16,
and shared/reused declarations. Three new Rust roots pass, including actual
ownership/opaque-header and20000-node contextual unary work. Strict Clippy,
scoped Go vet and changed-file formats pass. Existing constant roots are
selected because their exact min/max primitive is shared; other original
parser/lexer/scope/type/signature/core/ordinary behavior is retained without
unrelated test repetition.

The [receipt](verification.json) records nine source epochs/6862 complete
preimages and18 executions. Three original failures remain: the first partial
comparison exposed min/max internal defaulting; another exposed use of a
multiple-result expression in a single-value context; strict Clippy identified
an unnecessary return after the builtin fix. Both original false acceptances
and incomplete namespaces are preserved without reconstructing missing cases.
Five lossless transports retain all49052 original files/126193706 bytes. The
[literal audit](original-byte-audit/result.json) compares every original byte and
mode, all4985 final source/candidate/outcome rows and all5117 constant rows.
[Input review](input-review.json) uses unchanged127 matchers/four scopes/39 seals/
51 exclusions over complete current/epoch/decoded data, with no new exceptions.
Frozen255, frontend189, SDK14496 and ordinary448-case paired basis remain exact.

This is the scalar typing component. Aggregate values/field/index/range/method/
interface typing, complete body/assignment-statement/call-state checks, original
source evaluation and carrier witnesses remain due. A selected declaration's
type does not prove its surrounding package/body or an opaque callee. Full
nominal/module/import/build/version/capture/context and resource closure,
U/Plain/weights/final counts, storage/permission/lifetime/state/program proofs,
complete56-family foundation/registry and nativeLinux/generalnamespace/selected
closure/all169/T02–T11 remain required. Go/Rust only; R01/R06/W01 stay open,
public practical profile remains inactive and the T02 whole gate stays final
W04: `./scripts/check-fast.sh`.
