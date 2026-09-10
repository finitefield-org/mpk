# Integrated ordinary foundations (W09, in progress)

These 44 actual-source programs combine structural/collection definitions,
complete source observation and Money operations in one ordinary program per
validated VIR. One Builder, concrete iteration pipeline and storage cache are
shared. Semantic equality and source observation use distinct caches and names:
IEEE NaN/sign/zero bit observations must not replace application value equality.

The corpus reaches all twelve template families and emits every expanded
operation of eleven families, including uninvoked Money operations. It contains
462 operation occurrences and 285 source-observation carrier occurrences.
Maximum costs are 150,315 terms, 1,779 declarations and 10,441 static
transformers, within the frozen limits. Transition operations retain exact
deferred IDs and owner 6. Construction ownership, source/public-domain conditions
and Money's explicit source currency predicate still require ordinary proofs.

Shared composition matters: the actual nested-box source has 16 operations and
uses 8,250 transformers, while the sum of its separate domain, sequence and
construction counts exceeds the 16,384-transformer limit. Money uses the same
relation/storage state and a single decimal-operation cache across concrete
instances. No core language, axiom or checker acceptance rule changes.

Generation/import reconstruct exact metadata and bytes from validated VIR.
The current 44-source generation/import/mutation test passed in 147.95 seconds.
Forty-two existing certificate byte sequences are unchanged; binding-vc-money
now contains its ten Money operations, and shared-string-enum-money adds the
two additional concrete Money instances and their twenty operations.

The Money-only integration comparison passed all two sources, three instances,
30 operations and 2,976 transitive declarations against standalone generation.
Its predicate-removal and failure-order metadata mutations reject. Exact replay
of the standalone Money pins also passed; these two tests completed in 292.20
seconds. The complete current pinned replay and all 118 source-component pairs passed
1,235 roots and 14,183 transitive declarations with root/dependency mutation
rejection (163.58 seconds). The 44 same-byte Rust/Go checker cases remain
running and must pass before the current integration review can close.

The previous 43-source checkpoint, all 43 certificates and its manifest are
preserved under pre-money/. money-integration.json records every old/new file
hash. That previous corpus passed both checkers with zero axioms and corrupted
hash rejection, and its 116-pair component comparison passed 1,160 roots and
11,207 transitive declarations. Those are historical results. The still older
42-source pre-observations/ archive is unchanged.

See ../unit-4-integrated-money-progress.json and its review for current commands,
versions and live jobs. The earlier unit-3 and observation receipts retain their
chronological coverage. Units 3–8 and W09 remain incomplete, including recursive
semantic checks, remaining bindings/codecs, native/source/control relations,
transitions, ordinary propositions/proofs and final certificate assembly.
The full T06 gate remains deferred to T06-W12.
