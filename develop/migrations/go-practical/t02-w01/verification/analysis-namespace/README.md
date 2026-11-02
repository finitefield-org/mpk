# Go analysis vendor namespace: in progress

The private Rust owner derives its vendor-file list from every recorded entry
under the vendor directory in one imported source snapshot. Each observed file
must have an original captured source object of the required role; unobserved
files and directories are reported as `NotCaptured`, rather than treated as
absent. The vendor manifest and all three module archives must match the exact
source objects in that same snapshot. The derived list is then checked against
the original archive members by the existing archive owner.

The [Go observations](native/result.json) retain all 213 original source
objects and original bytes, including the three archives and 133 vendor files.
Go and Rust agree on the original complete observation and five controls:
unread file, unread directory, unknown unread file and symbolic link reject;
a coherently pruned snapshot with one file removed from both its entries and
source objects is accepted with 132 vendor files. That last case shows that
this owner establishes completeness only relative to the *recorded* snapshot.
It cannot authenticate the original filesystem or prove that the snapshot has
not been replaced. The full original snapshot and original 213 source objects
remain separately retained.

The [independent inspection](inspection.json) recomputes each source identity,
both snapshot hashes for all six observations and the exact control deltas. It
compares every captured original byte with the prelaunch source tree and the
earlier archive capture. The [affected Rust tests](namespace-tests/result.json)
pass both new tests and both archive tests. The three direct snapshot lookup
tests pass [separately](snapshot-tests/result.json); scoped Clippy, Go vet and
changed-file formats also pass. The selected tests exercise the new boundary and
the two components it consumes, rather than a whole-repository gate.

Authenticated original roots and capture, shared capture limits, Go syntax
and embedded inputs, module manifest/checksum linkage, source-to-binary build
authority, private context and application proofs remain due. T02-W01 remains
open and the practical Go profile inactive. The Go checks used local Linux
emulation, not native Linux release acceptance. The cumulative T02 gate
`./scripts/check-fast.sh` remains deferred to the final T02-W04.
