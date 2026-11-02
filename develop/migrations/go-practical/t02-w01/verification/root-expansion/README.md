# Original Go pruned root expansion

Status: scoped checks passed. T02-W01 and the practical profile remain open/inactive.
Current product scope is Go and Rust only.

The private v2 graph importer now derives final roots from the original main
manifest instead of accepting the native graph's main outgoing edges as roots.
It applies Go 1.25's pre-graph redundant/main-path normalization, loads original
summaries with identity/incoming-pruning deduplication, computes selected versions
and upgrades existing root paths until consistent. Non-root paths are not promoted.
Every intermediate real manifest remains owned, even when absent from both endpoint
inventories. The producer captures those original bytes before the native command;
Rust independently rechecks original cache/replacement paths, go.sum and lax facts.
A missing transient input rejects rather than substituting a native graph claim.

Pinned Go sort permutations matter for equal semantic labels. The original-root
sort follows Go 1.25's unstable algorithm; the inventory keeps its separate raw-byte
tiebreak. The Rust port retains the Go BSD notice and [license](../../../../../../crates/mpk-vc/src/go_practical_type_keys/module_observations/graph_inputs/GO-SORT-LICENSE).
MVS selected versions follow the serial load order fixed by the original command's
GOMAXPROCS=1; metadata is checked separately in its breadth-first output order.

The native comparison preserves all13 initial cases and adds five cases for all
admitted main language minors, a discarded self-root and a discarded lower root
whose .mod does not exist. The18 cases also cover multi-phase upgrades, a transient
manifest, old modules unpruning modern ones, cycles, exclusions, unread pruned
requirements, duplicate paths, Go/toolchain upgrades and non-UTF8 suffix bytes.
The ordering corpus contains225 unbounded semantic-version comparisons and84
native sort permutations at0,1,2,11,12,13,49,50,51,127,128,129,513 and2048 entries
with ascending, descending, equal, saw, organ and random patterns. These are test
sizes, not new admitted-universe or kernel limits.

The initial native duplicate-consistent failure remains with its complete source
and original command outputs. Native loadModFile normalizes redundant paths before
loading any graph; the correction preserves every raw require directive while
avoiding reads of discarded lower versions. No case is removed or reclassified.

All8 affected Rust roots and8 native roots pass, with strict production/integration
Clippy, Go vet and changed-file formats. The20 complete source epochs preserve
the initial native mismatch and a later new-test format failure. The Rust checks
include both whole original HTTP owners (local181 packages/5667
objects and versioned182 packages/5697 objects), all earlier coherent graph and
checksum controls, two additional coherently sealed invented final-root claims,
and the original native expansion/ordering matrices. The whole native versioned
owner is captured again with the changed production code. Original source, package,
proof and error universes stay intact; earlier proof certificates are preserved
without claiming a new kernel replay.

General correctness proofs, complete native filesystem-read authority, transport
resource closure, compiler/root/capture/context/descriptor/registry/launcher and
source/program/foundation bindings, native Linux release gates and remaining
T02–T11 stay due. This private component does not activate a profile or complete
R01/R04/R05/R06/W01. The T02 full gate remains the final W04:
`./scripts/check-fast.sh`; it is not run in this intermediate W.

The [verification](verification.json), [inspection](inspection.json) and
[scoped review driver](review.py) retain exact inputs and complete output records.
The unchanged review policy has127 matchers/four scopes/39 seals/51 exclusions.

The first evidence review stopped on an original zero-byte go.sum represented
as JSON null by Go. Its full script preimage and traceback are preserved in
[the failed review record](review-initial-failure.json). The decoder now requires
0 bytes and the exact empty SHA-256 for that spelling; no source or case is
excluded and the127 rules remain unchanged. Runtime code/tests are unchanged.
