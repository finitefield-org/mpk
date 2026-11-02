# Candidate-bound original module graph

The private Go candidate compiler owner now retains its original package owner
and derives module observations and the graph from that owner and the selected
candidate roots. It compares the complete resulting export record with the
one already bound to the compiler. Callers do not supply replacement roots or
a graph to this handoff; closing the candidate releases its owner reference.

The Rust live observer independently imports those module observations from the
same held export owner. It checks the original offline module data, sums,
archives and manifests, reconstructs the graph, and requires its source
selection to equal the request/source binding. The graph, candidate and both
root sets remain held together. This is private input consistency evidence,
not an installed launcher, compiler correctness or application theorem.

The full local capture passed with the unchanged original Go test application,
182 packages, 5,516 source objects, 181 export objects and all 5,577
registered SDK files (166,354,960 bytes). Go ran with the registered Go 1.25.0
Linux/amd64 SDK under emulation; the Rust test observer ran on Linux arm64.
Rust reconstructed eight graph roots, nine expansion manifests and eight
selected versions. The Go graph retained all 13 initial and 13 final module
records, including virtual records. The same observer accepted the selected
`Root` request and rejected the distinct original `Unused` function. The
directly affected original-compiler test also passed with its separate test
bundle identity and complete original inputs.

The affected Rust graph tests passed for the complete 5,697-object owner and
for two coherently resealed root/dependency alterations. They were selected
because the observer now delegates offline data and graph validation to those
importers. The Go candidate-owner guard test and the existing fixed-Go
launcher/profile, canonical-hash and strict-JSON tests passed; the latter ran
with Go 1.23 because the new method is compiled into that shared package.
Go 1.25 `vet`, normal and test-configuration strict Clippy, changed-file Rust
and Go formatting, and `git diff --check` passed. No new validation archive is
committed.

The foundation, launcher/read authority, native Linux release acceptance and
source/program proof remain open. R04/R05/R06 and T02-W01 are not complete,
and the practical profile remains inactive. The cumulative
`./scripts/check-fast.sh` gate is deferred to final T02-W04.
