# Installed Go compiler boundary: in progress

This checkpoint stages a private owner for the Go SDK selected from the executing
installed successor release. It validates the build-pinned revision-4 semantic
and release registries, snapshots the registered Go frontend and toolchain via
the existing descriptor-relative loader, and binds the independently imported
original source/export/compiler record to that SDK's actual files, modes and
identities. The owner carries no caller-supplied SDK path, digest or file list.
It is not yet connected to a practical semantic context, launcher or proof result.

The registered Go frontend was reproduced from the retained Go/Rust-only
source epoch and matches the registered 9,004,378-byte executable and full
Linux SDK inventory. A fresh original Go capture retains all 5,577 registered
SDK files (166,354,960 bytes), 182 packages and 5,697 source objects, including
the original HTTP application, unused bodies and init. The capture has one Root
selection and **zero contract overlays**. Earlier receipts that describe a Root
contract for this particular owner should be read as a Root selection; they do
not establish that a contract overlay was present.

The [whole-snapshot binding](whole-snapshot-diagnostic/result.json) passes using
a test-only sealer over all registered Linux SDK bytes and the complete original
source/export owner. A coherently rehashed wrong-host claim and the complete
original Darwin owner crossed with the Linux SDK both reject. Both owners retain
the same original Root selection and source, and zero overlays. The full check is
also part of the default test target. This check does not exercise an installed
root. The scoped host compilation, strict Clippy and changed-source format checks
have separate receipts.
The first emulated Go owner run hit the existing 120-second child observation
limit; the subsequent serial run passed without changing that limit. The first
Clippy run identified an MSRV issue and was corrected; both failures are retained.

The initial complete Rust input provisioning returned publication unavailable.
An independent byte-reproduction attempt then failed because the default macOS
filesystem conflated filenames that differ only in case. On a separate
case-sensitive volume the original Rust build completed, with the fixed LLVM
archive recovered by verified byte ranges after a network timeout. The first
finalization used an incomplete runtime-library search; the [corrected finalization](rust-finalized-bundles/result.json)
uses the original ordered `/lib` and `/usr/lib` search and validates all 360 Rust
toolchain files (1,467,509,632 bytes) and both frontend binaries against the
registered inventories. Earlier failures and their inputs remain recorded.

The [installed-root checks](installed-qemu-binding/result.json) place the actual
test executable at `bin/mpk` beside both registry transports and all four complete
registered Go/Rust bundles. The existing descriptor-relative loader enumerates
the full four-bundle namespace and snapshots the selected Go frontend and SDK.
The original SDK and complete source/export owner bind. An uninstalled executable,
rehashed wrong-host claim, complete crossed Darwin owner, changed installed SDK
byte and missing Rust namespace member reject. A separately valid alternate
registry with a matching changed Go executable and recomputed identities also
rejects the build pin. The [independent inspection](installed-qemu-inspection.json)
checks all nine observations, identities and restored inputs.

The first actual installed-root attempt [failed](installed-root-binding/result.json)
because the local x86-64 translation returned `ENOSYS` for `openat2`, even with
Docker seccomp unconfined. The retained [diagnostic](openat2-environment-diagnostic/result.json)
and the older [QEMU 7 probe](qemu-bookworm-openat2/result.json) show the failure;
[QEMU 10](qemu-backports-openat2/result.json) supports the required syscall and
executable path. The passing installed-root checks use QEMU 10 user-mode
emulation on Linux arm64, with the x86-64 runtime extracted from the pinned local
Linux image and networking disabled. This is scoped integration evidence, not
native Linux release acceptance; the production boundary was not weakened.

T02-W01 remains open; the practical Go profile is inactive. Remaining private
context, foundation, capture/read authority, source/program proofs and later
T02–T11 work retain their existing requirements. No whole gate runs in this
intermediate W; `./scripts/check-fast.sh` is due in the final T02-W04.
