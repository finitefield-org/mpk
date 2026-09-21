# Source array identity and current native memory bindings

Four exact source transfers in index_update now retain native memory observations:
one store uses its native exit normal state, and three loads use their native
entry pre-invocation state. The original transfer value identifies the allocation;
all three loads resolve to newer SSA storage IDs. The generated metadata retains
the original source slot type, transfer, allocation origin, exact state ID,
checked ownership flow theorem, and both native carrier arguments. It does not
change the original transfer identity to pretend it was a mutable SSA value.

The ordinary predicate compares the full private carrier: length, cells,
initialized bitmap and physical padding. Tests include the private 16384 bound
and 4096/4097 boundary, stale first-cell contents and high physical-bit changes
on both sides. These are open snapshot-copy relations; arbitrary raw test
storage does not constitute a source-execution, initialization or domain proof.
The flow theorem is a separately checked dependency, not a claim that evaluating
a dictionary proves reachability or assignedness.

Review of allocation resolution: only construction carrier transfers are
selected. An exact single origin must own the source anchor's identity/current
name; missing/ambiguous origin, missing native state/value/type or absent checked
flow theorem fails generation. Current storage must be a typed native value in
the same function. Import reconstructs the full source/ownership program and
rejects erasing any binding field or substituting the stale source anchor ID.
All snapshot equations are appended after the previous guard/slot/phi definitions.
Their entire prior term/declaration prefixes and metadata remain unchanged.

A public bounded sequence has capacity 4096 while private construction storage
has capacity 16384. No truncation, freeze, default or unconditional public-slot
projection is introduced. public_slot_projection_pending remains true. The
existing strict source-slot emitter still rejects incompatible source/native
types. General source-slot/heap projections, non-transfer memory effects,
nullable slot presence, node-entry merges and native/application composition
remain required. This component does not complete unit 5 or W09.

Verification scope: re-execute the changed index_update source and its new
snapshot/corruption cases, regenerate all other sixteen edge contexts exactly,
and check only the changed certificate with both unchanged checkers and hash
corruption. Four forwarding modules add only one type export; removing that line
reconstructs each prior source byte-for-byte. No new Std consumer path or checker
source is introduced, so inventory and unrelated data tests are not repeated.
The full gate stays deferred to T06-W12.

Final checks passed: index_update runtime 4,559 observations (52.24 seconds),
exact seventeen-context regeneration (2.13 seconds test execution), two
preservation checks, clippy, formatting and same-byte Go/Rust acceptance plus
hash rejection. Go positive/hash stages took 150.356/143.353 seconds; Rust
stages took 0.363/0.386 seconds. Sixteen prior certificate/metadata pairs are
unchanged. Final pins match the exact checked and runtime-tested candidate.
No remaining defect was found within this native-snapshot component. Public
slot conversion and all listed native/application obligations remain open.
