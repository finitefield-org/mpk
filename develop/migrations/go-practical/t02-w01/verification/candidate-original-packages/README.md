# Candidate original package owner

The private replay image can reconstruct all candidate-selected packages and
their full source-typed dependency closure from its captured original inputs.
The owner retains its own copy of the complete candidate request, package set,
native observations, source objects and compiler export objects. It rejects a
different request, a crossed export snapshot and use of a candidate observation
through the legacy single-package constructor. The subsequent multi-package
handoff uses distinct candidate records rather than the legacy single-package
record.

The targeted test uses the full registered Go SDK and the original HTTP
application and dependencies, with one added function and independent source
package. It selects four functions across three packages including an original
dependency. After materializing the image it removes only the test's temporary
original copies and clears their in-memory buffers. The candidate owner then
reconstructs 183 packages, including 182 source-typed packages, and verifies
all 5,517 original source objects and 182 compiler export objects by their
complete source identities and content hashes. It retains unselected original
function bodies and survives retirement of the replay image. A different or
missing request, changed export snapshot, caller mutation and closed owner are
also checked.

The existing full registered-SDK single-package owner, original candidate
request and original-package ownership tests passed. The ownership test covers
15 crossed or missing module, compiler, path, package and source cases. These
tests were selected because the new constructor shares their reconstruction
and request-selection paths. Go vet and changed-file formatting passed. All
checks ran locally under Linux amd64 emulation, not a native Linux release gate.

This is a private preparation checkpoint. The later multi-package handoff has
its own verification record. Literal `...` import paths, native argument-size
boundaries, launcher authority and source/program proofs remain
open. W01 remains incomplete; the T02-wide
`./scripts/check-fast.sh` gate is deferred to final T02-W04. No validation
archive is committed.
