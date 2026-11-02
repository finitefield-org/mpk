# T02-W01 original function slots and declared binding annotations

The [Rust signature reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_signatures.rs)
binds ordered receiver/input/result slots to independently derived original
v2 source types. Each slot retains its original Field, optional name and type
expression node. Grouped names share an original type expression and one owned
source/type graph. Unnamed and blank slots retain their positions. Result
arity0–16 is enforced separately from struct field limits: the actual70-input
case shares one type expression without an invented64-parameter restriction.
Nominal/alias/import/local-shadow/aggregate and exact scalar constant array
length behavior comes from the existing original type reader.

Receivers require an actual local nominal base; pointers and generic/variadic
function signatures remain outside this private signature domain. A variadic
parameter variable has its actual slice type, derived internally from the
original ellipsis annotation. This variable fact does not enable variadic
calls or make ellipsis an accepted standalone public value type expression.
Type-parameter annotations retain an explicit Unsupported classification.

Declared-variable queries require an exact original scope binding and use that
binding's own annotation. An unrelated pointer or out-of-domain slot does not
change the declared type of a different variable. Inferred ValueSpecs, short
and range bindings retain InferenceRequired for the next expression-typing
stage. Initializers and bodies are not checked by an annotation query: a
constructed wrong initializer obtains no validity claim. Function bodies stay
attached by optional original locator; an absent body supplies no call proof.
Original function-type slot names are preserved in signature metadata while
remaining distinct from lexical values available in a function body.

The [native reference](../../../../../../go-tools/go2vir/practical_source_signatures_reference_test.go)
uses real pinned Go1.25.0 original package/module/export/SDK ownership and saves
whole source inputs before original discovery and type checking. All31 actual
function observations and259 native variable definitions are retained. Rust
matches27 supported signatures and250 declared variable types with exact keys
and physical field/name origins. Four valid Go signatures and three valid Go
variable types remain explicitly outside this value/signature domain; five
inferred bindings require expression typing. One native function-type name is
explicitly retained as type-only metadata. Applications/init never execute.

The matrix includes methods, literal and bare function types, grouped/blank/
unnamed slots, imported/local aliases, local nominal shadowing, folded array
lengths, zero/multiple/named results and the16/17-result boundary. Five new Rust
roots and four affected existing source-type roots pass. Ownership, duplicate
names, invalid builtin receiver base, foreign binding, absent body and annotation
versus initializer controls pass; strict Clippy, scoped Go vet and changed-file
formats pass. Existing source-type tests are selected because batched graph
construction, variadic-variable normalization and the type-parameter rejection
branch share that builder. Other parser/lexer/scalar/scope/core/ordinary tests
retain their exact earlier evidence without unrelated repeats.

The [receipt](verification.json) records11 complete source epochs/8344 preimages
and24 actual executions, including three original failures. The initial native
span-selection failure retains its complete original capture. A line-wrap
format failure and partial variable comparison also remain. The latter
revealed that a native function-type Def is not necessarily a lexical value
binding; the corrected observer keeps all native definitions and records that
distinction from original syntax. The partial run's missing later outputs are
not reconstructed. Six lossless transports preserve all33373 original files/
262995192 bytes. The [literal audit](original-byte-audit/result.json) checks all
original bytes/modes, both successful native component streams, all31 function
observations and all259 physical definition origins. Actual original native
runtimes, including the155-second final run, remain measurements of this Darwin
reference test and do not establish native Linux release/resource acceptance.

[Input review](input-review.json) scans complete current/epoch/decoded original
inputs using unchanged127 matchers/four scopes/39 seals/51 exclusions. Frozen255,
frontend189, SDK14496 and prior448 ordinary paired streams remain exact. This
checkpoint supplies declared annotation correspondence; full expression and
initializer/body/assignment/call/state typing, ordinary source evaluation and
carrier witnesses, full module/import/build/version/capture/context and resource
closure remain due. U/Plain/weights/final counts, storage/permission/lifetime/
state/program proofs, complete56-family foundation/registry and nativeLinux/
generalnamespace/selectedclosure/all169/T02–T11 remain required. Go/Rust only;
R01/R06/W01 stay open, public practical profile remains inactive and the T02
whole gate stays final W04: `./scripts/check-fast.sh`.
