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

The [whole-snapshot binding](whole-snapshot-default/result.json) passes using
a test-only sealer over all registered Linux SDK bytes and the complete original
source/export owner. A coherently rehashed wrong-host claim and the complete
original Darwin owner crossed with the Linux SDK both reject. The full check is
also part of the default test target. This check does not exercise an installed
root. An independently [validated alternate registry](alternate-registry-validation/result.json)
retains all Go/Rust entries while changing one executable and rehashing its
complete identity; it will be used as an installed-registry pin control. The scoped host compilation, uninstalled-root
rejection, strict Clippy and changed-source format checks have separate receipts.
The first emulated Go owner run hit the existing 120-second child observation
limit; the subsequent serial run passed without changing that limit. The first
Clippy run identified an MSRV issue and was corrected; both failures are retained.

The initial complete Rust input provisioning returned publication unavailable.
An independent byte-reproduction attempt then failed because the default macOS
filesystem conflated filenames that differ only in case. A new attempt is using
a separate case-sensitive local volume and the same fixed original inputs. The
actual four-bundle installed-root test is pending that reconstruction. Emulated
Linux observations do not replace the required native Linux release gate.

T02-W01 remains open; the practical Go profile is inactive. Remaining private
context, foundation, capture/read authority, source/program proofs and later
T02–T11 work retain their existing requirements. No whole gate runs in this
intermediate W; `./scripts/check-fast.sh` is due in the final T02-W04.
