# Exact Go PGO lookup across filesystem name behavior

Status: T02-W01 remains in progress; the cumulative T02 gate is still due.
The supported program source languages remain Go and Rust only.

On a case-insensitive host filesystem mounted into a Linux container, the actual
Go compiler resolves the existing native entry `DEFAULT.PGO` when looking up
`default.pgo`. The complete original registered source epoch, the exact
293,923-byte registered SDK profile, real Go build plan and native capture
showed that the earlier auto-mode implementation omitted this input. The
[original failed case](casefold-original/result.json) retains that entire test
and its inputs, observation, compiler profile and output.

The Go capture now resolves the actual requested path through its original root,
finds the corresponding native entry by underlying file identity, preserves
that entry's original spelling and captures its exact bytes. It ignores
unrelated entries whose metadata cannot be read and rechecks the resolved
identity before and after capturing. Explicit PGO-off mode remains unchanged.
The independent Rust owner records the untrusted native spelling separately;
the held-root consumer checks Go's actual lookup against the same captured
file and rejects a falsely claimed absence. Launcher authorization and the
build-command/mode binding remain separate requirements.

An additional control retains all 197 case-folded original objects and adds a
separate `other.pgo` file with the same profile bytes. The held-root check
rejects a claim that this different file was selected, even though its bytes
match; it accepts the file actually resolved by Go. The
[independent comparison](crossed-inspection.json) verifies that none of the
original source objects or build inputs was replaced.

Another additional control gives the same file a second hard-link name. The
initial capture chose the unrelated name, even though Go's requested path
matched `DEFAULT.PGO`; the failure and its exact inputs remain in
`hardlink-before-fix`. The repaired capture prefers the exact or case-folded
requested spelling and retains `DEFAULT.PGO`. The
[post-repair comparison](hardlink-inspection.json) independently confirms
that all nine earlier complete capture, namespace and build-input pairs remain
byte-identical, so the unchanged Rust consumers retain their original inputs.

The [independent comparison](inspection.json) checks every byte of the full
196-object registered capture and all 184 build inputs against separate Go
observations on two actual filesystems. On a case-sensitive filesystem,
`DEFAULT.PGO` is ignored and the original input lists stay unchanged. On the
case-insensitive filesystem the compiler uses it, so one exact profile object
and build input are added. The directory entry lists are identical across both
observations; the filesystem lookup is the distinguishing fact. No original
program, SDK input, path, proof obligation or limit is removed.

The complete earlier omission, its failed review and retained intermediate
observations remain available. The first scoped CLI Clippy run reported that
the new test did not read its retained snapshot; the original diagnostic is
preserved, and the test now asserts both full snapshot sizes. The subsequent
CLI Clippy run passes. The Go lookup controls, Rust target tests, CLI and
verifier Clippy, changed-file format checks and both final full-source Go
rebuilds pass. Their receipts preserve the selection reasons and
original inputs. Neither a historical build observation nor these
test-only held roots establish native Linux release acceptance, an authorized
launcher, an application theorem, W01 completion or public practical Go
activation. The full T02 `./scripts/check-fast.sh` gate remains at final W04.

The archive preflight found five incidental text matches in compressed byte
streams. Every original stream and the complete preflight report are retained
losslessly in verified chunks, with scanner rules and exclusions unchanged.
Restore them into a fresh directory with
`python3 -B develop/migrations/go-practical/t02-w01/verification/analysis-pgo-casefold/restore_preflight_archives.py /absolute/new-directory`.

The incremental review verifies 1,674 unchanged inputs from the earlier
complete review, restores its seven exact compressed streams, and scans 4,169
new or changed files, 24,749 archive members and 178,602 ZIP members. It finds
no active matches. Eight incidental matches in historical executable machine
bytes are retained with the complete binaries. The earlier full review also
covered 18,114 archive members and 119,068 ZIP members.
