# Nullable source slot storage review

The original `type` source declares `Box? b`, while W04 nominal slot metadata
contains Box and the validated native transfers contain the exact Option<Box>
instance. Slot state now records a storage override instead of dropping the
presence bit or changing either original record. Native SSA IDs/types and W04
source/protocol linkage remain unchanged. Unassigned locals still have no
invented default, and assigned None remains distinct from unassigned storage.

The storage override is selected from the validated closed option instance and
its exact payload type. Unrelated mismatches still fail closed. If a raw payload
enters an option storage slot, an explicit Some constructor preserves all payload
bits and produces the exact u32 tag and zero padding. Loads of raw payloads from
option storage require that same Some relation; None cannot be silently unwrapped.

The type source has 406 positive/negative state observations, including None,
Some with zero payload, Some(Box(17)), source load/store and frame mutations.
The prior eight source contexts regenerate exactly. Three Some storage variants
(Bool, C5 and C9 payload) have 3465 observations, including changing every tag,
payload and padding bit individually. Both new certificate artifacts pass
same-byte Go/Rust checking with zero axioms, matching reports and hash rejection.
See verification.json and array-baseline-preservation.json.

Nullable **edge transport remains pending**: current source-slot relations carry
the option state, but the old edge joins still use nominal source slot carriers.
The original W04 current-slot observations will also need an explicit mapping
at application assembly. These scoped definitions do not establish native
execution, source domains, reachability, loop proofs or application VCs. W09
units 3–8 remain incomplete, and the whole gate remains deferred to T06-W12.


Nullable edge transport now retains the complete Option storage representation
for the `type` source's 53 joins. Its 2,758 observations replace 2,016 previous
observations; the current edge total is 38,901. Sixteen other edge programs are
byte-identical, and the changed certificate passes both checkers and hash
rejection. See `../../control-edges/nullable-slots/verification.json`
and its review. Nullable slot-to-edge transport is implemented; mapping W04
logical observations, node-entry merging, non-transfer memory effects and
complete application proofs remain pending. The changed fixture contains zero
W04 goal bindings. Unit 5 and W09 remain incomplete.
