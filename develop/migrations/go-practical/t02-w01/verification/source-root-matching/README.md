# Go source snapshot and held-root comparison: in progress

A private Linux CLI owner opens explicitly supplied directory roots with
`openat2` and retains their file descriptors. It compares every observed native
entry name and Go file-type bit, and every captured source byte, against the
independently imported Go snapshot. It performs two complete passes and rejects
changed directory or file identity/metadata. No symlink or magic-link path is
followed for a captured directory or file. Unvisited directories remain
unobserved; their contents are not manufactured or assumed absent.

The [affected tests](expanded/result.json) use the complete existing analysis
frontend snapshot: 213 original source objects, 52 observed directories,
373 directory entries and both original capabilities. They copy the entire
original frontend tree and compare the exact captured bytes. Changed bytes,
extra or missing entries, a replaced link, altered file metadata between passes,
missing or duplicate capabilities and a noncanonical root reject. Separate
actual filesystem controls retain raw non-UTF-8 names and check regular files,
directories, links, FIFOs and Unix sockets. The [snapshot importer tests](snapshot-regression/result.json)
recheck all directly affected original lookup, native-name and malformed-input
cases. [Linux-target Clippy](clippy-linux-target-expanded/result.json) and
[changed-file format](format.json) pass.

The [initial test](initial/result.json) passed, but its archived input inventory
did not include the private codec source. The later [expanded run](expanded/inputs.json)
records the complete linked source/frozen inputs including that codec and reruns
the affected original and negative cases. The two local Rust container images
lacked Clippy; the [availability record](clippy-availability.json) and the
successful installed cross-target Clippy run are kept separately. The root test
executes Linux arm64 locally; the Clippy target is Linux x86_64. Neither is the
required native Linux release gate.

This matching result is not yet bound to an authorized launcher-selected root or
to the earlier capture transaction. It does not establish that a caller-chosen
root was the original root, prevent all later mutations of a held directory,
authenticate the frontend source against its registered binary, or prove an
application. Nested mounts are not independently authorized here. The retained
snapshot is owned data; further trusted launcher/context and complete original
capture integration remain required. T02-W01 and the practical Go profile stay
open/inactive. The cumulative T02 gate `./scripts/check-fast.sh` remains due at
T02-W04; the supported source languages remain Go and Rust only.

The later [source-root replay correction](../source-root-replay/README.md)
restores and verifies the original full frontend archive in a detached temporary
repository, so future checkout changes do not invalidate this historical test.
The [live application comparison](../source-root-live/README.md) also keeps
the three actual test roots open from before the Go capture through comparison;
launcher authorization and the separate export root remain unproved.
