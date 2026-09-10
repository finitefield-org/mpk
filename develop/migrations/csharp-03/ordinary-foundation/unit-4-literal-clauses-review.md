# Rich literal clause component: direct review

Reviewed canonical parsing, nominal type identity, builder ownership, constant
registration and both public-domain/structural consumers. The decoder change is
internal visibility only. Rich parameters require the sole `value` field and
no arguments or ordered checks. Their original UTF-16 representation reaches the
existing type-directed decoder without a lossy serde_json string conversion.
The ordinary emitter validates each decoded value again against the same VIR.

All rich literals are emitted in one context, so repeated nested product parts
are shared across sequence and Nullable constants. Names use the same legal
encoding as existing contract definitions. No whole-program partial result can
escape on a decode or emission error. The old Bool/integer path and its pinned
consumers remain unchanged.

The actual-source test exercises eighteen literal constants and 1,414 storage
bit observations. It verifies all standalone source/public definition dependency
closures and public metadata against integrated output, as well as canonical
imports and exact candidate replay. Initial test-only mistakes (private helper
reference, mistyped Docker digest and decimal-text IEEE literal inputs) were
corrected. Existing type validation identified and rejected the invalid floats.
No parsing, checker or acceptance restriction was relaxed.

No further component finding was identified in the implementation review.
Both exact candidates passed unchanged Rust/Go checking with zero axioms and
actual hash-corruption rejection. Their current hashes, metadata and exact PASS
names agree. Final affected lint, formatting and artifact-consumer closure also
passed; no inventory fingerprint refresh was required. This is a component review; application proofs and full
internal-unit/W09 acceptance remain open.
