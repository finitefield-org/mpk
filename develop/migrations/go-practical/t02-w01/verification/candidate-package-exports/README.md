# Candidate package export capture

The private Go capture can derive every distinct package argument from the
complete owned candidate request. It runs the pinned Go compiler offline with
the existing output and time limits, requires exact root metadata for each
package and retains the complete observed source and export closure. The
observation is bound to the complete request, package set, actual command
arguments, compiler, cache and native output. A multi-package observation is
not accepted as the earlier single-package observation.

The native test preserves the existing 182-package HTTP application and the
full 5,577-file registered SDK. It appends another original function and adds
one independent source package, then selects four functions across three
packages, including an original dependency. The resulting capture retains 183
packages, 5,517 source objects and 182 export objects. It checks the complete
original application file, the new package file and the sealed snapshots. It
also compares both captures directly: all 5,516 source objects and all 181
compiler export objects from the original single-package closure retain their
exact bytes in the multi-package closure. A different request, narrowed package
set, changed native output or command arguments, and a selected package
relabeled as a dependency reject. The 8,192-root boundary, including 8,192
distinct derived package paths, and existing single-package metadata and
command controls pass.

The complete original single-package capture also passes after the shared
observation refactor. One initial full-input multi-package attempt hit the
unchanged 120-second offline module-discovery limit before the new capture;
the same case passed on retry and again after the final source/argument checks.
Affected original-observation tests, including the no-`unsafe` package and
whole-output ownership controls, pass; Go vet and changed-file formatting pass.
These tests were selected because the implementation shares the native command
and export-validation paths with the prior single-package consumer.

This is a private preparation checkpoint. Subsequent checks bind the captured
closure to source types and a private image owner; the independent Rust wire
consumer is not yet connected. None of these proves source execution or an
application property.
Go's `all`, `std`, `cmd` and `...` command-pattern spellings fail closed here;
an exact source-root-based route for any real package with such a name remains
required. The 8,192-package derivation is covered, but a full native command
at that size and any operating-system argument-size limit are not yet closed.
W01 and the inactive practical profile remain open. The cumulative
`./scripts/check-fast.sh` gate is deferred to final T02-W04. No validation
archive is committed.
