# T02-W01 exact original declaration syntax membership

The [Rust importer](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_declarations.rs)
now parses every candidate package's complete original Go files and checks its
package clause. Each candidate declaration must identify one actual TypeSpec
with the exact physical byte range, identifier and alias flag. Immutable results
retain the complete parsed sources and actual TypeSpec/name/type syntax indices.
Indices refer only to that retained syntax arena; they are not canonical type
identities. The existing private v2 wire identities and all frozen limits remain
unchanged. A generic TypeSpec cannot serve as a v2 declaration: this candidate
format has no type-argument semantics. Unselected generic declarations are still
parsed in full. Original byte positions include Unicode and CRLF; line directives
never redirect reads or replace physical locations.

The [affected tests](../../../../../../crates/mpk-vc/tests/go_practical_type_keys.rs)
accept the five previously retained actual Go-produced component streams with
identical canonical bytes/hashes and check every bound declaration. Six further
constructed candidates exercise grouped declarations, shadowed local types,
Unicode/CRLF/line directives, unselected generics and the syntax/typing boundary.
Sixteen new controls reject renamed or inexact declarations, package-name
changes, reversed aliases, selected generics, a value declaration presented as a
type, malformed complete bodies, invalid original bytes and invalid unselected
files. All dependent package/declaration/type IDs and references are recomputed
before import; each rejection comes from original-source membership. Existing
wire/ownership/geometry/cycle controls also pass. All eight affected Rust roots,
strict library/test Clippy and changed-file format pass locally.

The [verification receipt](verification.json) records the exact input epochs,
actual prelaunch drivers and unmodified stdout/stderr. The complete emitted
236 files/842726 bytes are [retained](rust-originals/manifest.json) and literally
compared to original bytes and modes. The [input review](input-review.json) uses
the unchanged127 matchers/four scopes/39 seals/51 exclusions, decodes original
source/TypeSpec byte arrays, and checks every recorded source preimage. No new
source, JSON, directory, seal or blanket exception is added. Historical original
type-key negative-control findings remain in their earlier evidence unchanged.
The first review checked the report's link before creating the report. Its
actual driver, output and prospective report are [preserved](initial-review-failure/result.json);
creating the report before link validation corrects that review ordering.

Parser/lexer implementations, all Go production/test sources, the189-input
frontend, ordinary proofs/core/evaluator and registry are unchanged. Their
previous passed evidence, including the full2996-case independent syntax
comparison and448 ordinary paired streams, is retained without unrelated runs.
This checkpoint binds declaration syntax only. In particular, an unresolved type
expression can still accompany a self-consistent uint16 candidate graph; no
denotation correspondence or Go type correctness is inferred. Independent
scopes/name/import/type resolution, constant evaluation, full nominal/alias and
module/build/version/capture/context correctness remain due, as do complete
source/carrier/storage/lifetime/state/program and56-family foundation/private
descriptor/registry work, U/Plain/weight/final counts, full capture budgets,
nativeLinux/generalnamespace/selectedclosure/all169/T02–T11. Go and Rust remain
the only source languages. No public profile is activated. R01/R06/W01 remain
open; the T02 whole gate remains final W04: `./scripts/check-fast.sh`.
