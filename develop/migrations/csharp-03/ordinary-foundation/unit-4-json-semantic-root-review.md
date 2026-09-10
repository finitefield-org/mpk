# W09 direct semantic-root input coverage — review in progress

Five original C# requests bind the actual boundary root Payload to Money,
OrderedEntry, OrderedSet, OrderedMap or Transition. The map also binds its Pair
source to OrderedEntry. Each binding records the captured source content hash,
exact stored member IDs and frozen role bounds. Requests regenerate identically;
two offline compiler captures accept all five and produce identical response
bytes. The hash-checked request/response pair and capture receipt are retained.

Unlike the nested unprojected Payload fixtures, these inputs must use semantic
JSON role names and containers. Runtime tests check the imported semantic root
identity, full 128-bit parse header, captured semantic cell count, all true
argument bits and adjacent/prefix/selector probes, and exact typed JSON depth
boundaries. The corresponding source-field representation must be rejected by
both reference capture and the ordinary parser. All five runtime contexts pass in331.08s;their exact metadata and certificate
bytes are hash-verified and pinned. Same-byte checker success remains pending. The storage probes do not claim
complete wide argument coverage, source reconstruction, or application proofs.

No production generator changed for these fixtures. Remaining unit4 and W09
scope is unchanged; the T-wide check remains deferred to T06-W12.

The complete consumer closure test exposed a stale aggregate count: individual
fixtures already include the envelope and typed-depth Std namespace consumers
(total4946), while the test still expected4944. The earlier targeted fingerprint
mutation test did not inspect this aggregate. Only the aggregate assertion was
corrected;all real consumer fingerprints remain enforced. The full closure
recheck passed32.59s, including all136 unchanged consumer fingerprints. Final affected integration lint passes after pin assertions.

The five pinned candidates are now under unchanged dual checking. Scoped
formatting and final integration lint passed. W09 remains incomplete; no
component-only commit or push was performed.

All5 exact pins now pass unchanged same-byte dual checking,zero axioms and
hash mutations in1953.046s. The exact PASS set and current hashes reconcile.
