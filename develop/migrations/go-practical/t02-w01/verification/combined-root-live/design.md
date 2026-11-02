# Binding both observed Go root sets

The earlier checkpoints matched the application/dependency/SDK roots and the
generated export-cache root in different observer processes. They independently
showed that each recorded projection belongs to the complete original export
owner, but did not retain both root-handle sets through a single binding.

The private `GoCompleteRootMatchedExportInputs` owns the complete imported
exports together with both independently matched handle sets. Its constructor
requires the export observation to contain exactly the export capability, the
source observation not to contain it, and their disjoint capability union to
equal the complete original snapshot. Every directory entry, original object
and source byte in both observations must match its complete-owner projection.
The checks do not filter object kinds or reduce the source universe. A
separate private installed-compiler wrapper retains the same matches after
the existing compiler binding succeeds.

The test observer opens the application, dependency and SDK roots before their
capture, then opens the actual Go-generated cache before export capture. It
matches each while live and retains both returned owners. The existing Go test
continues to retire its original roots and replay image; only after producing
its complete original owner does the observer import that owner and bind the
two matches. It independently imports every actual SDK file against the
registered inventory while the two match owners are still held. It does not
replace unobserved cache entries or original program bytes.

These are capture-time filesystem observations and input consistency checks.
Neither root selection by a trusted launcher nor compiler correctness,
installed-release authority, source/program semantics or any theorem follows.
The practical profile remains inactive, T02-W01 stays open, and the cumulative
T02 gate remains due at final W04. Go and Rust are the supported source
languages.
