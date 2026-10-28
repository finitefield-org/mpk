# Mode-aware Go analysis PGO inputs

Status: T02-W01 remains in progress. This checkpoint examines the exact Go
analysis frontend build input; it does not activate a practical Go profile.
The supported program source languages remain Go and Rust only.

Go 1.25's automatic PGO mode consults `default.pgo` at the main package root.
The original Go capture recorded the root entry but omitted that file from the
input bytes; the independent Rust importer also accepted the incomplete
observation. The original failing Go and Rust tests, source files, SDK input
inventory, profile, logs and output remain in `native-before-fix` and
`rust-before-fix`.

The private Go capture and independent Rust importer now retain the chosen PGO
mode. Their existing entry points remain PGO-off: ignored `default.pgo` entries
are never read and do not enter the build list. The separate auto-mode entry
points require an observed profile's exact original bytes as an auxiliary
source; an absent profile adds no input. An observed but uncaptured profile,
a nonregular entry or a profile above the existing source-file cap is rejected
by this bounded capture. This does not relax any prior cap or imply that Go
itself has the same capture restrictions. A future trusted build observation
must still bind the selected mode to the actual command and source roots.
The [case-folded lookup follow-up](../analysis-pgo-casefold/README.md) also
checks the actual filesystem resolution against held roots; directory spelling
alone does not establish whether Go uses a differently spelled profile.

The positive case restores all 269 original source files and modes, including
242 frontend files, then adds the exact registered Go SDK compiler profile as
an extra file. All 196 original captured objects and 184 build inputs remain
byte-identical; auto adds one object and one build input. The profile is
293,923 bytes, SHA-256
`d7355ec9e3c8bf05e214b202174ede4205092f852124e6f7f7287e22f6b5d980`.
Independent comparisons in `inspection.json` check every retained object,
raw buffer, build input and the exact profile against the SDK.

Five separate restored copies test an ignored regular file, directory,
symlink, oversized supplemental file and absent entry with PGO off. Each
retains all original 196 source objects and 184 build inputs; the extra
oversized file's complete original bytes are retained. Auto captures the
regular profile, accepts its absence, and rejects the three unbounded or
nonregular controls. These are additional controls, not replacements for
original program files. The Rust checker independently imports their complete
native observations and exact original buffers.

With the same restored historical sources, SDK and release flags, two actual
offline builds were made with the real profile: auto records
`-pgo=default.pgo` and produces a 9,181,672-byte binary, while off has no PGO
build setting and reproduces the registered 9,004,378-byte binary exactly.
Their full original bytes are stored losslessly as separate hex files. Go's
and Rust's existing analysis-build-info validators reject the automatic-PGO
binary and accept the off binary. These build observations do not themselves
establish production launcher authority or a source/program theorem.

The intermediate unconditional profile capture was also tested and retained.
It was then replaced with explicit mode-aware capture because reading an
ignored PGO-off entry would narrow existing valid inputs. All original failure
and intermediate epochs remain distinct from the final checks. An initial
requirements-record insertion omitted a closing brace; its complete invalid
bytes and JSON diagnostic are retained, and the corrected record validates.
The first scoped review found incidental text matches in seven compressed
archive streams. Their complete original bytes and review are retained in
lossless hex storage; two large encoded streams are split into verified chunks.
To restore the original archive paths and modes into a fresh directory, run
`python3 -B develop/migrations/go-practical/t02-w01/verification/analysis-pgo/restore_archives.py /absolute/new-directory`.
The scanner rules and exclusions are unchanged.

Validation is scoped to the changed producer, importer, direct source/build
list and root consumers, actual frontend rebuild and the corresponding
lint/format checks. Nine selected Rust tests, seven Go PGO root/subcases, five
additional Go source/build/rebuild tests and both Linux root consumers pass.
Go test's vet checks, Rust Clippy for the library and affected CLI, and the
changed-file Go/Rust format checks also pass. Test receipts name the selected
cases and retain their full input archives. The cumulative T02 gate
`./scripts/check-fast.sh` is deferred
to final W04 under the repository instructions. Local Linux emulation is not
native Linux release acceptance; registered launcher/export-root authority,
source/program proof, the rest of W01 and T02–T11 remain open.
