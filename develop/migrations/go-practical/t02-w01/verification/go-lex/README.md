# T02-W01 independent physical Go lexing

The [Rust lexer](../../../../../../crates/mpk-vc/src/go_practical_type_keys/lex.rs)
reads a verified owned Go source buffer and returns independently owned physical
tokens. Unicode uses the existing exact pinned Go1.25.0/Unicode15 L/Nd tables.
Every identifier/keyword, numeric form, operator and delimiter is recognized;
quoted/rune/raw literals, escapes, comments, line-directive numeric checks and
Go semicolon insertion are handled. Raw strings retain their original physical
spelling separately from Go's CR-stripped token literal. EOF and inserted
semicolons have zero-width physical spans. Comments/directives never redirect
source reads or physical offsets. Malformed UTF-8, NUL and misplaced BOM reject,
including inside comments and literals. Encodings are checked for the whole
buffer before tokens; recovery tokens and error ordering need not match Go.
No caller supplies tokens, filenames to open, normalized text or a source adapter.

The [native reference](../../../../../../go-tools/go2vir/practical_lex_reference_test.go)
uses the unchanged real Go1.25 scanner. It saves full source bytes and source
objects before scanning. Its5853 cases comprise all2934 Go-file objects from the
retained original universe, every595 distinct constant string expression in the
pinned scanner test source,20 added boundary cases and2304 operator pairs. Source
applications and their init functions never execute. The complete real SDK's
14,496 files/202,722,007 bytes still match the retained original inventory;
[actual audit input and command provenance](sdk-check/result.json) remain saved.
SDK source/licensing is retained by the earlier complete pinned SDK archive and
[SDK license](../type-keys/GO-SDK-LICENSE.txt).

The [verification receipt](verification.json) records5714 accepted cases with
7,702,601 exact kind/physical-offset/literal-byte tuples, and139 cases rejected
by both implementations. The native full stream also retains485 recovery
tokens for invalid cases; those are not claimed equal to Rust's errors. Default
Rust tests run the committed2919-case edge corpus and ownership/raw-spelling/
inserted-semicolon controls. Explicit full-corpus tests separately consume every
original native input. Accepted lexical snippets include strings that are not
valid Go programs; success is not parsing or typing.

Review corrected an initial4,194,304 token cap that confused tokens with the
later independent AST-node budget. Tokens are now bounded by the actual owned
source byte count plus two: real tokens consume bytes, newline semicolons use
existing newline bytes, and an EOF semicolon plus EOF are the only additions.
This introduces no source/profile limit. A4,194,305-byte input produces4,194,307
tokens in both real Go and Rust, with the same complete stream hash. The frozen
AST budget, ordinary4096/131072/2097152/1024 caps and finite evaluator remain
unchanged. The large boundary input is lexically valid and syntactically invalid;
it intentionally tests the distinction between these stages.

Two Go roots and three Rust roots, strict Clippy, targeted Go vet and changed-file
formats pass. Eight whole-source epochs/5662 preimages retain15 actual checks,
four formatter writes and the reviewed initial cap; no actual test failure was
hidden or reconstructed. Three lossless archives retain35,114 original files/
565,549,694 original bytes. [Native records](read-native.py) retain their original
namespace/modes and bytes. [Rust records](read-rust.py) preserve four actual run
namespaces and all modes using5857 distinct original byte objects; identical
outputs share a stored byte object. The [original-byte audit](original-byte-audit/result.json)
compares every decoded file with its actual retained original bytes and modes. Readers
verify whole transport, original namespace and bytes without extraction or
execution. The [input review](input-review.json) uses the unchanged127 matchers/
four scopes/39 seals/51 exclusions. No new source, JSON, directory, seal or blanket
exception is added. All255 frozen design inputs and every189 existing Go
production frontend input remain exact. The native oracle function is byte-exact
after a separate boundary test is added; its actual original source epoch stays
preserved. Unchanged source-type component checks and448-case ordinary/core/view
evidence are retained without unnecessary repeats or another frontend rebuild.

Lexing is a dependency of independent source/declaration/type/alias/nominal
checking; it supplies none of those witnesses itself. Complete source parsing,
constant evaluation, capture/module/context binding, full U/Plain/weights/depth/
artifact counts, ordinary source/carrier and storage-route/lens correspondence,
actual shared live stores/permissions/NoAccess/allocation/lifetimes/state/program,
full56-family foundation/private descriptor/registry, native Linux/general
namespace/selected closure/all169/T02–T11 remain required. Go and Rust are the
only source languages; no installed/public profile is activated. R01/R06/W01
stay open; the T02 whole gate remains final W04: `./scripts/check-fast.sh`.
