# Candidate source-type correspondence

The private candidate export observation can now be checked against a
source-typed package closure. The binder derives every selected package from
the complete owned request, requires all selected packages to have been
type-checked, and applies the existing whole-closure source, import, compiler
archive and native metadata correspondence checks. Candidate and legacy
single-package observations remain separate. This correspondence check alone
does not construct an owned image or pass the closure to the Rust wire reader;
the subsequent candidate image checkpoint is documented separately.

The targeted test uses the original HTTP application, all original
dependencies and the full 5,577-file registered Go SDK. It adds a function and
an independent package, selects four functions across three packages including
an original dependency, and checks all 183 native packages against 182 typed
packages plus the `unsafe` intrinsic. The capture retains 5,517 source objects
and 182 compiler export objects. An untyped selected package, incomplete source
loader, altered request and missing original package sources are rejected;
the legacy binder rejects the candidate observation. The original unselected
function remains present in the typed source.

The existing complete registered-SDK owner test, the original no-`unsafe`
package test, and the standard-library source/export correspondence test
passed. The latter includes missing, duplicate, crossed compiler/module,
selected-root, source/import and modified archive controls. They were selected
because this change shares their correspondence code. Go vet and changed-file
formatting also passed. These checks ran locally under Linux amd64 emulation;
they are not a native Linux release gate.

This is an internal preparation checkpoint. The later Rust multi-package
handoff has a separate verification record. Exact handling for literal `...`
import paths, launcher authority and source/program proofs remain open.
W01 remains incomplete; the T02-wide `./scripts/check-fast.sh` gate is deferred
to final T02-W04. No validation archive is committed.
