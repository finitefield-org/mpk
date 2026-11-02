# Go analysis build inputs and reproduction: in progress

The private analysis source owner derives the complete vendor-mode frontend
build input list from the same observed snapshot. It retains 65 frontend Go
files, 133 vendor files and the original `go.mod`, `go.sum` and vendor manifest:
201 original files in path order. Go's capture, Rust's independently derived
list and the list actually supplied to the test builds agree on every object
and byte. The full observed snapshot still contains 213 source objects.

The [local Go test](native/result.json) materializes those owned inputs into
two separate roots and uses the registered Go SDK, network-disabled execution
and independent empty compiler caches. Both complete 9,004,678-byte frontend
binaries match byte-for-byte. Each original byte stream is retained as a
separate, losslessly encoded hex file; the [storage record](binary-storage.json)
verifies exact recovery. Their SHA-256 is
`e24e92e2aec2dd70e5fdcc698ee868561566a13b73dc74f07e48d399c6ce58f0`.
The [independent inspection](inspection-stored.json) recomputes the source
identities, compares the three lists and both recovered full binary byte
streams. The [initial review](review-initial.json) recorded two text-token
matches in ELF machine code; the reversible storage avoids treating those
bytes as prose without changing scanner rules or exclusions. The original Go
receipt names the raw output paths before storage, while both raw local copies
and both repository hex files preserve the original bytes.

Two historical compressed input archives are also retained losslessly as
hexadecimal data after incidental matches in compressed bytes. Their original
receipts retain the paths at the time of capture; the [storage and restore record](../source-root-replay/archive-storage.json)
preserves both complete original streams and the scanner findings.

The reproduced binary differs from the existing registered frontend
(`ccc4a984d41b5da7b3a6838cbfaeee3eb23f675a2e4d7fb5d4f9fcd7e597d538`).
The earlier registered source generation and the current source tree differ,
and the test build settings are not the registered release build procedure.
This observation does not bind the current source to that registered binary or
authenticate the original snapshot/root. It does not activate a practical
context or establish an application proof.

The [affected Rust tests](list-tests/result.json) compare the complete build
list and rerun the preceding 20-case source owner. Scoped Clippy, Go vet and
changed-file formats pass. The reproduction uses local Linux emulation, not
native Linux release acceptance. T02-W01 stays open, the Go practical profile
remains inactive, and the cumulative T02 gate `./scripts/check-fast.sh` is
deferred to the final T02-W04.
