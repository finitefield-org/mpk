# Complete live capture with a source contract

The complete cross-package test application is also captured with an added
`contracts/root.json` sidecar for `RootWithCall`. It states that the result
equals the input. The original Go 1.22 dependency and its `Identity` call,
all 182 packages and the registered Go 1.25 SDK remain present. The source
and export roots are opened before capture and remain held through the
independent Rust binding.

The capture has 5,517 non-export objects, 181 export objects and all 5,577 SDK
files. Independent byte comparisons against the preceding no-overlay capture
check all 5,516 existing non-export objects and their hashes; only the new
sidecar and its directory are added. The selected contract retains the actual
sidecar identity, parsed result/input equality and original selected function.
The callee retains its default contract. A different complete graph and an
unselected original function reject; the SDK-bound owner is test-only.

Both Rust and Go checkers accept the complete entry and call-interface
certificates with zero axioms and matching hashes. These certificate bytes are
identical to the no-overlay case: they establish entry and call mathematics,
not the stated postcondition. No callee execution, application theorem,
source/IR correspondence, launcher authority or complete foundation is proved.
Go ran as Linux/amd64 under local emulation and Rust on Linux arm64; this is
not native Linux release acceptance.

The new complete-capture test and the source comparison passed. Go 1.25 vet,
changed-file formatting and strict affected Linux observer Clippy, including
its test configuration, passed. The two original shared-path cases were
selected because the new fixture extends their shared capture helper and
observer. Their first retries both stopped during offline module discovery with
`signal: killed` at the unchanged 120-second command limit under high host
load; neither reached the new binding check. Their full-input retries remain
in progress.
No new validation archive is committed.

R01 and T02-W01 remain open, the practical profile stays inactive, and the
cumulative `./scripts/check-fast.sh` gate is deferred to final T02-W04.
