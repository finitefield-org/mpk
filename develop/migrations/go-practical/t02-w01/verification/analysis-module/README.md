# Go analysis module and checksum binding: in progress

The private Rust owner reads the frontend `go.mod` and `go.sum` from the same
observed snapshot as the verified vendor manifest and ZIPs. It checks the
frontend module and Go version, the exact three dependency versions and their
indirect flags, and rejects replacements, exclusions, retractions and a
toolchain suggestion. It preserves all checksum lines, including the original
unused entries and identical duplicates.

For each of the three original module archives it reads the matching cached
`.mod` and `.ziphash` from that snapshot. The `.mod` is parsed and compared to
its pinned `go.sum` checksum and, when present, the ZIP's `go.mod` member. The
`.ziphash`, archive checksum and frontend `go.sum` must agree. These facts are
relative to the supplied snapshot: the owner does not yet validate cached
`.info` metadata, authenticate the original root, prove source syntax or bind
the source generation to a registered executable or semantic context.

The [Go observations](native/result.json) retain all 213 original source
objects, 133 vendor files and three complete archives. Go and Rust agree on 20
original and adversarial cases: four accept and 16 reject. The cases cover
module/dependency changes, replacements and other directives, changed or
duplicate checksums, unread source records, changed cached manifests and ZIP
hashes. All other original source records and observed directory entries remain
unchanged in each control. The [independent inspection](inspection.json)
recomputes every snapshot and changed-source identity and checks those deltas.

The [affected Rust tests](module-tests/result.json) pass the new comparison,
both existing vendor namespace tests and both archive tests. The existing
manifest and checksum parser observations pass separately; scoped Clippy,
Go vet and changed-file formats pass. These checks target the changed owner
and its immediate consumers. `./scripts/check-fast.sh` remains deferred to
final T02-W04.

After a turn interruption the previous temporary SDK and frontend paths were
missing. The initial [prelaunch attempt](prelaunch/result.json) records that
failure. The [restored SDK](sdk-restore/result.json) matches the registered
5,577-file inventory, and the frontend matches its registered digest; the
[new prelaunch capture](prelaunch-restored/result.json) then passed. These Go
checks use local Linux emulation, not native Linux release acceptance.
T02-W01 remains open and the practical Go profile remains inactive.
