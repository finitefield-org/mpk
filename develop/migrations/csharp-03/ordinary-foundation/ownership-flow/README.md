# Native symbolic construction ownership witnesses

`ValidatedPracticalVir::symbolic_construction_ownership()` replays the existing
source ownership checker and returns the complete per-function witness. It
records allocation identities, incoming states before/after exceptional cleanup
and phi rewriting, local-action input, invocation input, normal output and
delayed loop backedges. The exceptional output is the invocation input: failed
allocate/write/freeze does not commit a new token version.

The native source protocol is symbolic linear ownership. The older W03 ownership
rows can contain only discard actions and empty before/after lists; those rows
alone cannot supply native SSA ownership equations. This API makes the already
reconstructed live maps available for ordinary equation generation.

The ordinary importer runs the same algorithm with compile-time capture disabled;
it does not retain full witness copies. Only explicit API calls capture maps.
Concrete ownership/borrow/transfer records and functions without native array
operations are outside this API. Empty results never establish those obligations.

The pinned corpus covers 12 genuine captured source contexts, 1085 blocks,
157 exceptional edges and 14 delayed backedges. Tests match every source CFG
edge, unique origin/version map, entry, invocation transition and phi against
the returned witness, and check deterministic reconstruction. Existing hostile
loop fixed-point mutations reject. Six construction-data fixtures regenerate
without any byte or metadata change. Lint, formatting and inventory pass.

This is witness data, not a proof, a trusted Boolean, or an accepted application
certificate. Ordinary transfer/edge/phi/cleanup propositions are now generated and verified
in `../ownership-equations/`. Checked proofs must still be generated and linked
to original SSA ownership checks. The 183
pending construction formula roles have not been marked discharged. Unit 5
and W09 remain incomplete. See ../verification-logs/ownership-flow/verification.json.
