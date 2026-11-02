# Go analysis source syntax and embed checks: in progress

The private analysis source owner enumerates all observed non-test Go files in
the frontend root and checks all retained vendored Go files. It uses the
independent Rust Go parser on each original file, rejects the same unmodeled
native file suffixes as the Go capture and rejects line comments whose Go
scanner text begins with `//go:embed`. Comment detection uses token gaps, so
string and rune literals and block comments are not mistaken for directives;
it also follows Go's removal of carriage returns from line-comment text.

The [native observations](native/result.json) retain the full 213-object
analysis snapshot, including all 65 frontend and 126 vendor Go files. Go and
Rust agree on 20 original/adversarial cases: nine accept and eleven reject.
Controls cover embed prefixes, strings and comments, malformed syntax, unread
production files, ignored tests and unrelated assets, forbidden assembly,
symlinks and raw non-UTF-8 directory names. The [independent inspection](inspection.json)
recomputes every snapshot and changed source identity, compares all 191
original Go files with the retained capture, and verifies exact control deltas.

The [affected Rust test](source-tests/result.json), existing direct parser and
token-span tests, scoped Clippy, Go vet and changed-file formats pass. The test
selection covers the new owner and the parser/lexer it consumes. The cumulative
T02 gate `./scripts/check-fast.sh` remains deferred to the final T02-W04.

These checks establish observed syntax facts, not exhaustive Go parser
equivalence, original-root or source-to-binary authority, a registered context
or an application proof. The Go observations used local Linux emulation, not
native Linux release acceptance. T02-W01 stays open and the practical Go profile
remains inactive.
The later [build-list checkpoint](../analysis-build-list/README.md) derives the
complete available build input list and compares two offline reproductions; it
does not establish registered binary or original-root authority.
