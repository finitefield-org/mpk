# Collection contract adapter direct review

The map/set read adapter validates exact nominal collection/key/result types,
concrete template dependencies, entry field roles, capacity and carrier depth.
Map lookup verifies the retained lookup result dependency for the precise value
type. It reuses the existing search/lookup bodies, preserving all nine original
collection certificate pins. Shared lookup types retain the correct map/key
closures; their multi-map regression is among the tested source contexts.

Nine source contexts exercised 18 aliases and 5,598 ordinary observations,
including every lookup result bit, empty/missing/present cases, nullable values,
decimal cohort equivalence, string/product keys and floating values. The test
also compared transitive definition closures and checked import/metadata
corruptions. It passed in 12013.57 seconds. All nine resulting certificates and
metadata were regenerated identically with the current implementation in
16.39 seconds; the long semantic run is reused without repeating its work.

All nine same-byte Rust/Go checker cases passed in 97.231 seconds, asserting zero
axioms, report agreement and hash corruption rejection. Terminal PASS names,
current and runtime-tested bytes, metadata, capture receipt hashes/sizes and
retained log hashes were reconciled. Affected clippy and scoped format passed.
Final direct component review found no actionable findings.

Input public-domain obligations and application VC/native-method proofs remain
separate. Original units 3-8 and W09 stay open; no component commit/push is made,
and check-fast.sh remains deferred to T06-W12.
