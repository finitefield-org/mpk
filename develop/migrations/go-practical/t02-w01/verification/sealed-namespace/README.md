# Sealed original namespaces and independent snapshot ownership

Status: scoped implementation checkpoint; T02-W01 remains In progress.
Entry: `7405cf5fd5c88f0e31ca96076618e296869ad095`.

The [snapshot implementation](../../../../../../go-tools/go2vir/practical_snapshot.go)
retains every visited directory's complete sorted native names and type bits,
all authorized capability names, and the complete original captured buffers.
[Seal](../../../../../../go-tools/go2vir/practical_capture.go) first revalidates
the original roots, visited namespaces and file bytes, then copies both the
namespace arrays and source allocations. No root handles, mutable file metadata
or collector allocations survive into the snapshot. All eight final source
roles and exact capability/path/role identities remain intact.

Read-only directory and source lookup use only those owned observations. A
missing exact name in an observed complete parent returns `fs.ErrNotExist`.
A present unread file, unvisited child directory or unvisited authorized root
returns the separate `errPracticalNotCaptured`. An unbound capability, incorrect
role/spelling, invalid logical path, link, non-directory ancestor or non-regular
file rejects. No lookup opens the original filesystem or supplies another
source. Consumers must not treat missing capture as permission for fallback.
Unrelated native asset names remain verbatim, including newline/backslash names;
only requested source/directory paths receive frozen logical-path checks.

The private derived content hash frames a domain, little-endian uint64 counts,
byte lengths, exact capability/directory/native-name bytes, native type bits
and all sorted complete source-object IDs. It excludes physical paths, inode
state and timestamps. Source IDs already bind original bytes and their roles.
Unread asset contents are not captured sources, while their complete visited
names/types remain retained. The internal hash is not a new public schema,
registered descriptor, filesystem authorization or proof authority. Raw invalid
UTF-8 names are framed as bytes without JSON replacement or normalization.

The [original regression](original/regression/result.json) reproduces the old
alias: changing a collected buffer after seal changes the snapshot's original
`package` bytes to `qackage`. Complete original source/test snapshots and raw
output remain. The repair owns a fresh buffer at seal; returned location/ID
buffers, source metadata, roots and directory arrays are also copies.

The [selected verification](verification.json) passes34 Go roots/101 subtests,
including six new snapshot roots. The cases compare complete native directory
enumerations, remove all original roots and mutate collector allocations before
reading the sealed snapshot. Thirty lookup cases distinguish proven absence,
uncaptured inputs, links/special files and exact capability/path/role rejection.
Independent original recaptures change names/types, empty directory contents,
source bytes/roles, visited coverage and authorized capability sets. Reversed
root/read order and different physical roots produce the same identity. A
separate Go1.23 compatibility build passes seven roots/eleven subtests; current
vet and changed-file gofmt pass. This does not add a practical-profile compiler.

The affected native analysis consumer rebuilds the complete frontend twice
from170 sealed original inputs, with distinct source roots and independent
empty caches. Both8,592,898-byte binaries have SHA256
`2a9d7c49b925818d372c886397c4e195d5fbf09cb53d4e31c2c39cf49ef6a67f`.
All34 frontend and133 vendor inputs plus the three module/vendor manifests
are retained; original analysis snapshots and the complete selected compiler
are checked. [Build observations](final/selected/reproduced-builds.json) retain
actual commands, environments, output and build metadata. The preceding
checkpoint's third build stays evidence of its preceding source epoch.

[Independent Python reconstruction](independent-observations.json) agrees on
all17 complete private hash preimages and99 complete source-object identities.
Eleven rows are actual native capture/recapture observations. Six raw-name rows
test framing only, including invalid UTF-8 and NUL byte strings; they cannot be
imported as captured namespaces. Their complete binary preimages remain in
[the observation directory](independent-preimages).

The [frozen-input check](frozen-design-preservation.json) preserves all255
original design/specification/vector inputs. The preceding complete4053-
declaration shift certificate is byte-identical. Unchanged arithmetic/core/
word/native matrices are not rerun for this Go capture change. The targeted
selection includes affected successful seal consumers for local/versioned
modules, archive ownership, original Go resolution and the real frontend build,
plus old snapshot/environment/public-profile boundaries. The unchanged official
strict scanner reviews every changed input; source, JSON, directory and blanket
whitespace exclusions are not introduced.

The [first precommit observer](input-review/precommit-original.py) treats the
normal no-index new-file diff exit1 as a subprocess error after all staged-byte
and review-self scanner checks pass. Its [original tool output](input-review/precommit-original-tool-output.log)
is retained. No commit runs on that failure. The corrected observer accepts
exit0/1 only when complete whitespace output is empty; review and exact staged
byte checks rerun after retaining these diagnostic artifacts.

Complete56-family ordinary source/carrier/nominal/type/operation/storage/
lifetime/state/program foundation, bounded assembly/descriptor/private registry/
context and retained-policy delegation remain due. Actual original application
source/export/selected-closure namespace materialization and authoritative
immutable loading/replay still remain required. All169 application demands and
T02-T11 remain open. No practical profile, general source equivalence, application
proof, immutable application replay or native Linux release is claimed. C# tests
stay skipped; the full T02 gate stays W04:
`./scripts/check-fast.sh --skip-csharp`.
