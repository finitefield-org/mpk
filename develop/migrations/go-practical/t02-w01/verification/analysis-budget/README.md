# Go analysis shared namespace budget: in progress

The private analysis snapshot owner now counts observed directory entries and
the namespace entries of its three distinct original module archives against
the same 131,072-entry bound used by the Go capture/replay counter. Previously
the two sets were bounded separately. This closes that accounting gap for the
analysis archives owned here; it does not account for unrelated archives in a
larger capture or authenticate the source snapshot.

The [native Go observations](native/result.json) retain the original 213 source
objects, 133 vendor files and all three complete ZIPs. In the new source epoch
the original snapshot has 368 observed directory entries; the ZIPs contribute
2,403 namespace entries. The exact-limit control adds directory entries without
changing or removing original objects and is accepted at a combined 131,072.
The one-over control is rejected at 131,073. The additional entries are
synthetic controls and do not replace the full original sources.

The [independent inspection](inspection.json) recomputes the original and both
control snapshot hashes and verifies that the only control changes are the
specified directory entries. The [affected Rust tests](budget-tests/result.json)
compare all three Go observations and rerun the two existing namespace and two
archive tests. Scoped Clippy, Go vet and changed-file formats pass. These checks
were selected for the changed shared budget and its immediate consumers; the
T02-wide gate remains deferred to final T02-W04.

Original-root and capture authentication, budgets for other capture archives,
Go syntax and embedded inputs, module manifest/checksum linkage, source-to-
binary build authority, private context and application proofs remain due.
T02-W01 remains open and the practical profile inactive. The Go test used local
Linux emulation; it is not native Linux release acceptance.
