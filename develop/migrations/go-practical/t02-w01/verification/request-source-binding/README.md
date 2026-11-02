# Private candidate request and source binding

The inactive practical-Go request is now resolved against the same independently
parsed original source scopes as the existing overlay selectors. The resulting
declarations must be exactly the already imported physical roots. Receiver
aliases that name the same declaration cannot introduce duplicate roots. A
single per-package name index retains the full 8,192-root limit without scanning
the source universe once per root. The binding borrows the validated request
and original source owner, rather than copying the captured source set.

The owner of the matched source and export roots can perform this binding while
keeping both root sets live. The installed-compiler owner has the corresponding
API; it was compiled in this checkpoint but was not run against a staged
installed release. Neither API authorizes launcher paths or proves which bytes
an external process read.

The new five-test request/source suite passed, including the full original
8,192-root case, an independently resealed but different physical declaration,
missing names, an ordinary `init` method and rejection of a retained Go-fixed
request. All 8,192 roots were resolved in about 133 ms on the local host. The
existing complete contract-module suite (six tests), selection-input suite
(five tests, including its original 8,192-root and error universes), and
overlay suite (six tests, including a receiver-alias duplicate control) passed.
These tests were selected because source selection and overlays share the
refactored resolver. After the binding was changed to borrow the same owners,
the new suite was rerun; the resolver itself was unchanged.

The complete live Go capture and Rust test observer also passed together. The
Go test supplied its actual candidate request for `example.com/policy.Root`;
the observer imported it and bound it to the held original source/export
owner. The original capture included 182 packages, 5,516 source objects, 181
export objects and all 5,577 registered SDK files (166,354,960 bytes). Go ran
with the registered Go 1.25.0 Linux/amd64 SDK under local emulation and Rust
ran on Linux arm64. The request digest was
`24000c87ae0e4b16194b84233c06cebfd2a2a8da044bba38740ca58d99f00c3b`.
The observer is a test executable, not the installed launcher.

Strict Clippy passed for the affected Rust tests and for both Linux observers
and the capture-binding integration test. Changed-file Rust and Go formatting,
Go `vet`, and `git diff --check` passed. The full original test data and limits
were retained. No new validation archive is committed.

The foundation is still candidate-only; launcher authority, native Linux release
acceptance, source/program proof and T02-W01 R01/R06 remain open. The practical
profile remains inactive. The cumulative `./scripts/check-fast.sh` gate is
deferred to the final T02-W04 work item.
