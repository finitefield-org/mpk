# W09 typed-value guarded input envelopes — review in progress

A separate generator variant composes original field parsing,canonical typed
depth and canonical typed node bounds in one owning Builder. Node definitions
are emitted after the complete existing depth-guarded program and share its
finite folds. Every parsed/defaulted argument is projected from the completed
packet and passed to its exact semantic carrier's node-bound predicate. Original
parse failure or any field failure masks the entire header/argument packet.
The original unguarded parser name remains in metadata;the final parse name
appends .TypedNodes to the retained depth-guarded definition. Empty inputs have
no field predicates but still preserve parse validity. New node metadata is
omitted from old variants,whose exact required-source bytes and metadata pass.

The first guard helper test used the wrong padding direction in its expected
packet bits. Packet layout places role then active value selectors then trailing
padding. The corrected oracle also constructs nonzero argument storage on a
synthetic failed parse,so omitting the guard's prior-validity check would fail.
All2560 packet bits now pass for exact cutoff neighbors,u32MAX,second-field
failure and original parse failure. Production guard logic did not change.

16 base original-source contexts are testing complete accepted/rejected packets,
metadata mutation/import,full field/old-parser/count closure preservation.16
compound/direct-root variants separately compare complete prior parser and
node-count closures plus metadata and limits. Their already-passed expensive
parser runtime is retained by exact definition closure equality;it is not
repeated. These closure tests do not claim a second full integrated runtime run.

Affected lint and predecessor pin tests pass. Source integration and new-byte
dual checking remain in progress. The separate node-count component's32 pinned
certificates passed both unchanged checkers and hash mutations in461.820s.
Raw JSON bounds,source/public domains,reconstruction/output and all remaining
W09 proof/assembly/acceptance obligations remain open. No component-only commit,
push or T-wide check is performed here.

All16 compound/direct-root definition closure and metadata comparisons now
pass154.17s. Their new programs and the corrected guard helper are hash-verified
and pinned. Base-source packet runs and new-byte checker acceptance remain open.

The required,nullable-default,presence-missing-default and empty-input source
variants pass8 accepted and15 rejected full-packet cases in304.88s. Their four
new candidates are hash-verified and pinned. Remaining12 base contexts continue.

All remaining12 base contexts pass22 accepted17 rejected complete packet
cases in685.02s. The total base scope is16 sources,30 accepted32 rejected
documents. All32 source variant metadata/certificate pairs are independently
hash-verified and pinned with the guard helper;new-byte checking remains open.


Follow-up:the original33-candidate run is terminal.32 cases passed and their
current file hashes/certificate metadata were reconciled. One case failed during
the Rust build because of previously incorrect source-clause signature field
names;that error is corrected. Only that case is being retried (session92592);
no certificate rejection is inferred and full completion is not yet claimed.

Final checker reconciliation:the sole retry passed both unchanged checkers and
actual hash-corruption rejection in1262.544s. Its exact PASS name and current
module hash/size/metadata match the pending row. The union with32 retained
successes covers all33 current candidates;no successful case was repeated.
There are no outstanding findings in this component. This does not complete
internal unit4 or W09,and the original remaining scope is unchanged.
