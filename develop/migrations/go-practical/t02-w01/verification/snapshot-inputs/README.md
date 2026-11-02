# Complete original snapshot and capture inputs

The Go original-package owner now carries the complete captured source object
set and observed native namespace together with its exact function/source/
annotation selection. One owner lock covers both handoffs. Every original
captured object is read and checked; no caller inventory, AST/type adapter,
filesystem fallback or replacement selection is accepted. Returned byte buffers
remain independently owned through caller mutation and owner retirement.

The snapshot input carries all capability names, visited directory enumerations,
full source-object records and the original private snapshot digest. Entry names
use canonical lowercase hex so arbitrary native bytes are preserved rather than
normalized through UTF8 JSON strings. Type bits retain the exact Go FileMode
values; permission bits do not masquerade as namespace type bits. Independent
Rust reconstruction reproduces the original framed SHA256: domain+NUL, little
endian64-bit counts/lengths, exact capability/path/name bytes, type bits and
complete source-ID order.

The independent reader validates closed fields/hashes, canonical ordering,
logical paths/capabilities, connected visited directories, source membership,
all raw source identities, complete supplied buffers and the existing capture/
contract/directory bounds. All eight frozen source kinds remain distinct.
An exact missing name in a visited directory establishes absence. A present
unread file, unvisited child or unvisited authorized root remains uncaptured.
Wrong spelling, role, linked/special entries, non-directory ancestors and escapes
reject. No disk/process callback supplies a missing observation.

The capture wrapper binds that entire snapshot to the preceding exact physical
function selection. Every selected-source buffer, captured raw annotation and
main go.mod must belong to the same full inventory; all captured overlays remain
present. Contract derivation accepts no replacement roots/source/annotations.
These are private owned inputs and independently checked data, not authorization
of capability roots or authentication of a launcher/installed context. Coherent
new self-hashed data does not authenticate the original capture.

The first actual owned-package fixture exposed an existing source-capture gap:
only active GoFiles were captured before sealing, while immutable go/build
replay also needed the excluded original Windows source. The complete failed
original input and output remain. The export collector now captures the actual
Go command's target/build/test-excluded Go files and auxiliary build inputs
before sealing. The same Windows source remains; original internal/external
test files and excluded assembly bytes add coverage. These auxiliary records
are captured build data, not additional MPK program source languages.

The initial Rust build also retained a test-helper lifetime error, and the first
library Clippy check retained the manual parity-style diagnostic. The fixes use
a static domain argument and is_multiple_of, without changing the snapshot
algorithm. All original failures and commands remain under their original epochs.

[Verification](verification.json) records9 passing Rust roots. They compare all30
actual native namespace queries, exact original bytes and all eight kinds;
replacement UTF8 bytes and raw FE/FF names remain distinct framing units. Three
complete owned captures include the SDK/export namespace and all original source
buffers. The two unchanged complete prior units retain all43 physical roots each,
the complete zero/70-error universes and all32 original proof certificate bytes.
The raw-name units do not claim that every host filesystem can create those names.

The initial native suite passed11 roots/41 subtests and failed only the original
excluded-file ownership root. The repair suite passes that root, both export
metadata controls and full original standard-library source typechecking:
4 roots/30 subtests. Current strict Clippy, Go vet, changed-file Rust/Go formats
and Go1.23 zero-test compilation pass. The original failed suite remains failed;
its successful roots are not reclassified as a full-suite pass. Unchanged kernels,
full foundation matrices and unrelated frontend/retired-language tests are not
repeated. The cumulative T02 gate stays final W04: `./scripts/check-fast.sh`.

[Independent inspection](inspection-final/result.json) checks five full
snapshots, exact native framing/digests, every raw object, all30 lookup outcomes,
full native package membership and source/selection/capture hash linkage.
[Snapshot storage](snapshot-storage/manifest.json) retains all77162 original
files/696793808 bytes with exact paths and modes, including failed and successful
native captures. [Transport storage](archive-storage/manifest.json) retains all19
complete original prelaunch containers/353598517 bytes. Local duplicate originals
remain physically unchanged. Exact restore:

```sh
python3 -B develop/migrations/go-practical/t02-w01/verification/snapshot-inputs/preserve-originals.py --restore
python3 -B develop/migrations/go-practical/t02-w01/verification/snapshot-inputs/preserve-transports.py --restore
```

Normal Rust tests automatically restore missing original raw files losslessly
from tracked storage before reading them. No source is regenerated or rewritten.
[Input review](input-review.json) scans all decoded originals under the existing
127 rules/four scopes/39 seals/51 exclusions; no rule or source/proof exclusion
is added. All producer stdout/stderr retain exact bytes.

Full canonical capture/context/descriptor/registry authority, independent module/
build/compiler role validation, source/IR/state/storage/frame/callee/program
proofs, all56 foundation families/all169 demands, native Linux release gates and
remaining T02-T11 stay due. R01/R06/W01 stay open; the practical profile stays
inactive. Go and Rust remain the only MPK source languages.
