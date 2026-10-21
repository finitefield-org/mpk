# T02-W01 original source type denotation

The [Rust source type reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_types.rs)
derives private v2 type keys from complete owned syntax, lexical bindings and
exact scalar source constants. Builtins retain native int/uint distinctions;
byte/rune and source aliases preserve their canonical targets. Real TypeSpec
origins supply nominal identity. A definition's underlying type erases any
intermediate nominal spelling while the original declaration/alias trace stays
available separately. Arrays, slices and ordered structs retain grouped field
expansion, embedded alias identity, unexported package identity and exact raw
tag bytes, including byte escapes and raw CR removal.

The work list and caches are flat. Aliases and expanded recursive value graphs
reject cycles, including zero arrays and slices. Existing array256/fields64/
depth16 limits and private construction limits remain unchanged. Pointers,
selected generic types and interfaces other than the actual predeclared error
or its aliases remain outside the private v2 value domain. Scalar constant
coverage is inherited: valid floating or aggregate-valued constant lengths
remain unmodeled. This does not claim full Go type checking.

The declaration checker compares actual derived children, underlying edges and
alias targets to a candidate whose hashes/shape/closure were already checked.
Anonymous candidate roots lacking an original declaration acquire no source
authority. An observed type stream can omit imported packages unused by that
type even though its full source file still imports them. The explicit scope
variant uses a larger immutable original source universe; every candidate Go
source must match its complete original object/bytes and package identity.
Unknown imports are never guessed. This extra source universe remains candidate
metadata rather than a registered build/capture/module context.

The [actual Go reference](../../../../../../go-tools/go2vir/practical_source_types_reference_test.go)
uses the existing original-package owner, real module/export discovery and
original compiler/SDK captures. Original inputs are saved before discovery or
typing. Seven actual observed streams cover imported types/constants, local
nominal chains, embedded struct/array/slice aliases, Unicode field visibility,
blank fields, grouped tags, exact byte/raw tags and bounded constant lengths.
Together with the five unchanged earlier streams, all12 candidate declaration
graphs match their original source derivation. Applications/init never execute.

Four new Rust roots pass. Seven fully rehashed body mutations pass the existing
wire importer and then reject for actual array length/child, field order/tag/
embedded marker, alias target or nominal underlying mismatch. Separate source
and package context substitutions reject. Geometry/cycle/non-type controls and
depth16/17 and fields64/65 boundaries pass. The eight affected existing importer
roots, strict Clippy, scoped Go vet and changed-file formats pass. Tests target
the new derivation and the importer changed to admit real embedded aliases;
parser/lexer/scalar/scope logic and ordinary/core behavior retain their prior
evidence without unrelated reruns. Whole SDK14496, frontend189 and frozen255
inputs remain exact.

The [receipt](verification.json) retains seven complete source epochs/5288
preimages and17 executions. Two initial Rust compile failures and the partial
comparison failure remain original observations. The partial run's missing
outputs are not reconstructed. Five lossless transports preserve all11339
original files/88718325 bytes, including the empty failed-build namespace.
The [literal audit](original-byte-audit/result.json) checks every original byte
and mode, and all observed source/component streams. [Input review](input-review.json)
uses unchanged127 matchers/four scopes/39 seals/51 exclusions, scanning complete
current/epoch/decoded source data without new exceptions.

Complete source typing, ordinary evaluation/carrier witnesses, full module/
import/build/version/capture/context correctness, aggregate resource budgets,
U/Plain/weights/final counts and source/storage/permission/lifetime/state/program
proofs remain due. Full56-family foundation/registry and nativeLinux/general
namespace/selected closure/all169/T02–T11 remain required. Go/Rust only;
R01/R06/W01 stay open, the public practical profile stays inactive and the T02
whole gate stays final W04: `./scripts/check-fast.sh`.
