The [original-source dependency scheduler](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_dependencies.rs)
lets the exact constant reader use actual value/type inference for array len/cap.
Declared and inferred variables, original aliases/named types, literal ellipses
and keys, fields/indexes, function/method return declarations and original import
identities participate. Runtime calls inside arguments prevent constant results;
constant builtins/conversions retain their own Go constant rules. Direct and
indirect type/value/constant cycles reject. No native AST/type callback is used
by production code.

The scheduler uses an explicit work queue and memoized original-source queries.
A missing dependency suspends the current attempt through an internal pending
request; it is resolved before retrying the parent. Active dependencies reject
cycles. Thread-local sessions are scoped to one immutable source universe and
restored by a guard. Returned graphs share owned immutable sources. Repeated
ConstSpec expressions preserve the current declaration's iota and lexical scope
only for that original reused syntax, while referenced declarations retain their
own contexts. Compiler variable reads propagate through constant/type results,
so a local used only by const len(a) is checked as used. These reads do not assert
runtime evaluation of the builtin argument.

The [Go1.25 reference](../../../../../../go-tools/go2vir/practical_aggregate_constants_reference_test.go)
saves whole original sources and candidates before parser/typechecking. All406
cases are retained:345 exact positive constant values/types,56 agreeing
rejections and5 valid controls outside the frozen value domain (array pointers,
float/complex element types and array length257). No new exclusion is added.
The original four new Rust roots also check inherited iota, compiler reads,
512 cross-reader dependency links, direct cycles and four concurrent thread
sessions with error/success cleanup. Thirty-eight current Rust roots pass,
including affected old value, body, constant, type, scope and signature readers.
Old495 body cases now have176 exact positives/299 rejections/20 valid outside
forms; the required len/cap gaps are closed. Old5117 constant cases now have4833
exact positives/281 rejections/3 outside forms. All original corpus bytes stay
unchanged. Other old4985/950/1659/461/1433/630 outcomes remain exact; three original
native builtin checker crashes remain distinct. Clippy, affected formats and Go
vet pass. Unrelated ordinary/kernel/parser semantic tests are not repeated.

The [verification](verification.json) retains8 epochs/6656 complete preimages and
24 actual checks. One initial Rust helper-visibility compilation failure remains
with its original diagnostics. The first scope/signature inventory omitted their
tracked fixture inputs; those inputs were added before rerunning only those two
targets. Both complete original output maps equal the initial maps. All15 lossless
transports retain34406 original files/93144308 bytes with a [literal byte/mode
audit](original-byte-audit/result.json). [Review](input-review.json) uses unchanged
127 matchers/four scopes/39 seals/51 exclusions without new exceptions.
SDK14496/frontend189/frozen255/ordinary448 basis remains exact.

This closes array len/cap constant dependencies inside the supported source
reader. Whole-v1 admission, other body/package/import validity and selected
callee closure remain due. The independent reader still supplies no runtime
source execution, call/target/store order, result zero/payload, allocation/alias/
permission/NoAccess/lifetime/state/program proof, complete nominal/module/build/
version/capture/context/resources/U/Plain/weights/final counts, full56 foundation/
registry or nativeLinux/generalnamespace/selectedclosure/all169/T02–T11 acceptance.
Go/Rust only; R01/R06/W01 remain open, the public practical profile stays inactive,
and the T02 full gate stays finalW04: `./scripts/check-fast.sh`.
