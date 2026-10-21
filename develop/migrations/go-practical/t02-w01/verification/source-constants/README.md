# T02-W01 independent exact scalar constant evaluation

The [Rust scalar reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_constants.rs)
evaluates actual owned syntax and lexical bindings. A sign/magnitude multiword
integer implementation supplies exact addition/subtraction/multiplication,
truncating division/remainder, infinite signed bit operations and arithmetic
right shifts. Runtime word wrapping is not applied to constants. Each operation
separately checks the pinned compiler's512-bit untyped precision or its typed
integer range. Typed operands preserve canonical type identity; incompatible
named types reject, aliases preserve their target, and implicit assignment is
distinct from explicit conversion. The frozen target's int/uint remain64 bits.
These are the actual pinned Go1.25 [precision](https://github.com/golang/go/blob/go1.25.0/src/go/types/const.go)
and [shift](https://github.com/golang/go/blob/go1.25.0/src/go/types/expr.go) rules,
including the1074 shift-count and10000-byte numeric-literal checks; core,
ordinary evaluator and profile limits are not raised.

Original integer/rune, bool and byte-string literals, raw CR handling, exact
byte/string operations, scalar conversions, comparisons/logical operators and
constant string len/min/max are handled. Signed and untyped complement uses an
infinite sign mask; unsigned typed complement uses its real width. Integer-to-
string conversion uses Unicode scalars/replacement, keeping it distinct from
string byte escapes. Constant logical operands are checked as constant operands
on both sides: a runtime short-circuit cannot hide a division error or turn a
variable/function call into a constant. This changes no runtime evaluation rule.

The [iterative evaluator](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_constants/evaluate.rs)
retains each declaration member, expression and current repeated-ConstSpec
context. Inferred/grouped/multiple constants, iota including blank specs, typed
annotations, forward/imported constants and aliases resolve from original
TypeSpec/ValueSpec nodes. An omitted expression is reused with its actual new
lexical point: local bindings that become visible between repeated specs are
not resolved at the old expression position. Inherited type annotations are
also re-resolved. Cycles reject and pending work/caches use flat heap structures
instead of recursive expression/declaration evaluation. Results own their data.

The [actual Go reference](../../../../../../go-tools/go2vir/practical_constant_reference_test.go)
stores every complete source/candidate before invoking real Go1.25 parser/type
checking at gc/amd64 sizes. Its reference-only import map returns packages
checked from those same complete original bytes; production has no callback or
Go object-map dependency. All5117 cases remain actual retained observations.
Rust agrees on4832 accepted scalar values, complete canonical scalar/nominal
keys and underlying facts, and281 actual Go rejections. Four valid Go cases
are explicitly outside this reader: floating/complex constants (including an
integral float conversion) and aggregate-value constant builtins. Those are
reported as unmodeled, never as Go language rejection. The scalar reader does
not activate profile support for generic, aggregate or runtime conversions.

The matrix crosses large signed/unsigned values through511 bits, every integer
operator/comparison, positive/negative/boundary shifts, all frozen widths/native
int/uint and byte/rune aliases, typed overflow, negative division/remainder,
nominal confusion, aliases, byte/Unicode strings, const groups and imported
values. Two new Rust roots, the three affected existing scope roots, strict
Clippy, targeted Go vet and changed-file formats pass. The original Go root is
repeated after adding an actual runtime-version assertion; all10235 emitted
files remain literally byte/mode equal, so the already passing Rust comparison
is retained. All14,496 SDK files/202,722,007 bytes are checked against the complete
original inventory before the runtime-pinned run. Applications/init never run.

The [receipt](verification.json) records seven source epochs/5063 complete
preimages and18 executions. The initial Go oracle/vet compile failures (missing
type assertion) and Rust large-enum lint failure remain complete; conversion
metadata is boxed rather than suppressing lint. Five lossless transports preserve
all30,978 original files/66,409,565 bytes, including the actual empty failed-build
output namespace. The [literal byte/mode audit](original-byte-audit/result.json)
compares every decoded file to its execution original. [Input review](input-review.json)
scans complete current/epoch/decoded source and candidate data using unchanged
127 matchers/four scopes/39 seals/51 exclusions, with no new source/JSON/directory/
seal/blanket exception. Source/lexer/parser, existing importer behavior, all189
frontend inputs, frozen255 and prior448 ordinary paired streams remain exact.

Candidate scalar type keys still have the earlier private module/package
identity boundary. Complete type-expression/array-length/struct/alias graph
correspondence, ordinary original-source evaluation/carrier witnesses, full
nominal/module/import/build/version/capture/context and aggregate resource
budget/behavior remain due. This private scalar evaluator is not whole-source
type checking or source/program authority. Full U/Plain/weights/final counts,
source/storage/permission/lifetime/state/program,56-family assembly/private
descriptor/registry and nativeLinux/generalnamespace/selectedclosure/all169/
T02–T11 remain required. Go/Rust only; R01/R06/W01 stay open. Public practical
profile remains inactive and the T02 whole gate stays W04: `./scripts/check-fast.sh`.
