# Ordered private capture roots

The private root session owns the source handles first, then opens the single
reserved export-cache capability, matches its snapshot, matches the source
snapshot and finally binds both matches to the same complete export owner.
Each state is consumed on transition. A source capability cannot use the
reserved export name or collide under ASCII case folding. The complete-owner
constructor is private to the session, so its existing full namespace and byte
checks cannot be bypassed by another caller in this module's consumers.

The unchanged local Go capture and the changed Rust observer passed together:
5,516 original source objects, 181 generated export objects, 182 original
packages and all 5,577 registered Go SDK files (166,354,960 bytes). Both root
sets stayed held through binding, and the generated cache path was absent
before the final binding. Go ran as Linux amd64 under local emulation; the Rust
observer ran on Linux arm64. The source and SDK inputs were mounted read-only.
The test used the existing full Go test and its original 40-minute timeout,
not a reduced package, source or cache universe.

Scoped validation selected the root-binding integration test because it covers
the changed state transitions and the existing complete compiler/root consumer:
`cargo test --offline --locked -p mpk-cli --test go_practical_capture_binding --
--nocapture --test-threads=1` passed all five tests in a local Linux container.
It restored the full existing fixture, including all 16,880 evidence files and
5,577 SDK files. The first run passed the binding checks but failed the existing
native-name test: the shared container volume did not enumerate a FIFO and
socket created after their directory was opened. A separate filesystem probe
reproduced that behavior; the unchanged full test passed on Linux tmpfs. This
failure and its environment cause are recorded here rather than changing or
skipping the test.

Strict Clippy for the two affected observers and the root-binding test passed
for `x86_64-unknown-linux-gnu`; Rust format and `git diff --check` passed for
the changed files. The installed-release observer was compiled but was not
rerun with a staged installed release in this checkpoint. No new validation
archive is committed. This session enforces ordering within its API; it does
not authorize the selected paths or prove what an external process read. It
does not establish a registered context, native Linux release acceptance,
compiler correctness or an application theorem. T02-W01 and R06 remain open,
the practical profile remains inactive, and the full `./scripts/check-fast.sh`
gate is deferred to T02-W04.
