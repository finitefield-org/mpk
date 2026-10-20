# Original Go constant strings and complete native carrier data

Status: affected checks passed; R01/R06/W01 stay open. Entry is
`8b54c50c3c917baff244fd0f8f6102d6e83d8230`; current source-language scope is Go/Rust.

The private original-package owner now observes a string constant using only
the original package path and physical source-ID/byte span. It uses its own
complete original Go type information, checks actual source membership and
retains the whole original source, expression and every native value byte.
Callers supply no AST, typed package, import callback, replacement source or
value. All returned byte views and carrier arrays are independent of their
owner and remain intact after preparation roots, donor buffers, image and owner
retire. Existing sticky input/type/syntax failures stop further observations.

The data conversion retains a full64-bit length and256 full8-bit slots with
zero padding. It preserves NUL, invalid UTF-8 and all256 byte values; a UTF-8
string is counted in bytes. The empty string has zero length and all zero
slots. A257-byte native value remains complete and returns a distinct modeled
domain error, independently of invalid-query or damaged-source errors. These
are native data observations. The exact curried ordinary carrier constructor,
source-expression evaluation and nominal/source identity proofs remain due.
An alias's actual display name is preserved; display strings grant no identity.

Ten affected roots and77 subtests pass. The42 new string subtests cover literal,
folded, referenced, selector, parenthesized, named/alias and rune-conversion
constants; CRLF raw strings; all bytes and255/256/257 bounds; damaged observations,
foreign/partial spans, import/tag/non-string/runtime expressions and failed
owners. Full unused channel/map/callback bodies and a panic initializer remain
original and are never executed. The existing four-module/181-package/1,472,495-
node HTTP/replacement image also observes two actual strings after original
roots and donor buffers retire. Its final5771-file/7421-namespace-node observation
is retained, without claiming a general authoritative closure or kernel context.

The first run failed one test expectation: Go displays `AliasText`, whereas the
test assumed its target name `Text`. The original native observation, complete
source epoch and test output remain. The corrected display-only expectation
passes. A later test-only record update captures actual successful child module
diagnostics and owned module/package outputs. Missing separately archived earlier
module diagnostics are explicit in [verification.json](verification.json);
later runs do not replace them. All original drivers are saved before launch.
All five original source-object frames and native observations remain under
their exact external paths in that record and the generated `input-review.json`.

The complete actual build closure is now186 inputs:50 frontend files,133 vendor
files and three metadata files. Every prior185 input remains exact; the only
addition is `practical_string_constant.go`. Two fresh pinned offline Linux/amd64
development builds agree at9,004,162 bytes/SHA256
`8eb9e81d1be647a0814ffbb81ce94050b73c161cb726b10c60758e2d48cee2ec`.
Two empty-cache host builds from captured originals agree at8,592,898 bytes/SHA256
`4c9c0cfd4f42e26a995b69e41591c6fc0857b1fffc6762782fad191d930f30f8`.
The new development bytes remain separate from the original published registry.
Closed public admission, Go vet/changed-file format and zero-test Go1.23
compilation also pass. Later edits affect Go1.25 tests/records only, so unchanged
production builds, owner and public/legacy consumers are retained without repeat.

All255 frozen inputs, original127 matchers/four scopes/39 seals and original byte
epochs remain; no scan exception or limit changes. Unchanged Rust arithmetic,
core/ref, protocol and unrelated suites are not repeated. No other-language
test, agent or Actions runs. The complete ordinary foundation/private context,
native kernel/general namespace/selected closure/all169/T02-T11 remain due.
The cumulative T02 gate stays W04: `./scripts/check-fast.sh`.
