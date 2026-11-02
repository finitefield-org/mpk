# Complete live capture with a cross-package call

A second local capture keeps the complete existing test application, its
unselected HTTP/concurrency handler, all dependencies and the registered Go
SDK, and appends one selected function that calls the existing `Identity`
function in `example.com/c`. Its original module declares Go 1.22; the frontend
and SDK are Go 1.25.0. The original call-free `Root` case is retained and was
rerun separately.

The two complete captures each contain 182 packages, 5,516 non-export source
objects, 181 exports and all 5,577 registered SDK files. An independent
byte-for-byte comparison checked all non-export sources: 5,515 are identical;
`policy.go` is exactly the original bytes followed by the new function. Their
non-export directory observations are identical. No original declaration,
source, dependency or error universe was replaced or removed.

With both original root sets retained, the private same-owner constructor
validated the selected root and its one transitive callee, one real call site,
the complete offline module graph and the test-only installed-SDK owner. It
also rejected a different complete graph and an unselected original function.
Both Rust and Go checkers accepted the exact entry-equation certificate (2,073
declarations, 475,066 bytes) and call-interface certificate (2,085 declarations,
479,491 bytes) with zero axioms and matching hashes. Their certificate hashes
are respectively
`ef81e49a18585b250cbe7e3751a78a80a9119292fbe4b24f9ddef262d0253967` and
`3e425a9c636c84858ad60ac7506371c09a45dad123ec55f2e5be699dfc0bc99b0d`.
The call-free baseline also passed, and its entry certificate is byte-identical
to the earlier independently Go-checked baseline.

These are entry and call-interface equations. The fixture has no user overlay;
both functions receive default `true` contracts. No actual callee execution,
application property, source/IR correspondence, launcher authority or complete
foundation is proved. The SDK owner is test-only. Go ran as Linux/amd64 under
local emulation and Rust on Linux arm64, so this is not native Linux release
acceptance.

The new and original full-capture tests, both certificate checkers, and the
complete source comparison were selected to exercise the new nonempty call
closure without weakening the existing capture. Go 1.25 `vet`, changed-file Go
and Rust formatting, and strict affected Linux observer Clippy (including its
test configuration) passed. No unrelated suites or new validation archives
were added. The separate 70-error invocation regression remains running.
R01 and T02-W01 remain open, the practical profile stays inactive, and the
cumulative `./scripts/check-fast.sh` gate is deferred to final T02-W04.
