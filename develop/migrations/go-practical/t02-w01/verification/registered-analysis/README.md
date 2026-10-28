# Registered Go analysis source epoch

Status: W01 remains in progress; the registered practical profile remains
inactive. The original frontend is the Go/Rust-only source epoch
`ed67a4c388defe8babdbc55f29851552562121d9`, not the later evolving checkout.

The retained archive and receipt reconstruct all 269 original files, including
all 242 original frontend files, with each path, source byte and receipt mode
checked. The archive's own file modes differ from the later extracted receipt
modes for all 269 files; both the exact archive and the recorded extracted
modes are preserved. Its PAX commit identifier is also validated. The first
Go capture attempt rejected that global PAX header; its unchanged input archive
and failed transcript remain recorded before the corrected run.

After the successful empty-cache run, the restored temporary path is resolved
through platform symlinks so that macOS temporary directories remain usable.
The affected Go test reruns with `-count=1` and a warm local compiler cache;
its complete capture packet and all 192 unique raw files are byte-identical to
the earlier run. Unchanged Rust and held-root tests are therefore not repeated.

Go captures 196 original source objects and 52 directories containing 287
native entries. Its complete build-input view has 48 frontend Go files,
133 vendor files, and three module/manifest files: 184 objects total. Rust
independently imports the original captured buffers, checks the vendor and
module ownership, and derives the same 184 ordered objects and bytes. An
independent Linux root comparison checks the complete capture against the
restored frontend and original archive roots, rejecting changed file bytes and
an extra unread directory entry. These are restored test roots, not authorized
production launcher roots.

Two separate empty-cache, network-disabled local builds use the complete
269-file source epoch, the checked 5,577-file registered SDK and the original
release build flags. Both reproduce all 9,004,378 bytes of the registered
frontend, SHA-256
`ccc4a984d41b5da7b3a6838cbfaeee3eb23f675a2e4d7fb5d4f9fcd7e597d538`.
The whole output streams are retained separately as reversible hexadecimal
files. An independent inspection rehashes all original archive files, compares
every build object to its original source bytes and checks both output streams
against the registered inventory. This historical reproduction is not a claim
that the newer source generation produced that registered executable, nor does
it make a launcher, semantic context or application theorem authoritative.

The first scoped review identified two incidental text matches in the raw
compressed bytes of two input archives. Both complete streams, their original
modes and the entire failed review are stored losslessly and independently
restored. Scanner rules and exclusions are unchanged; the original receipts
refer to their capture-time paths, while the storage record identifies their
recoverable current locations.

The corrected scoped review finds no active or archived matches across 5,492
retained archive members, 26,265 ZIP members and both decoded complete binary
streams. It independently checks the encoded original archives before reading
them; scanner rules and exclusions remain unchanged.

The selected Go capture test, Rust owner test and held-root test cover the new
historical input. Go vet, changed-file formats and targeted Linux/Rust lint are
recorded separately. The exploratory macOS Go 1.23 tar probe could not compile
because that host installation's standard library is unavailable; its source
and diagnostic are retained, and it supplies no archive observation. Local
Linux arm64 execution and x86-64 emulation are not native Linux release
acceptance. W01, trusted launcher and separate export-root authority, source
proofs and T02 completion remain open; the full `./scripts/check-fast.sh`
gate is deferred to the last T02-W04.
