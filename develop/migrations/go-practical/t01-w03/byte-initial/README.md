# True one-cell claims with incorrect proof witnesses

This historical finding is not false-claim or completion evidence. The
[two original attempts](observations.json) preserve exact canonical bytes,
raw independent checker observations, source/checker identities and private
source snapshots before correction. Unchanged production sources resolve
through the recorded root hashes. Both attempts fail at core type checking,
not resource exhaustion.

At capacities 0 and 1 the finite model lift has one backing byte. Its total
model read ignores the index and returns that byte. Removing the relation
between indices leaves a true full-byte equality. This does not permit an
out-of-range Go read: source-definedness is a separate guard. The original
producer applied the general theorem with an index self-equality of the
wrong type. Both checkers rejected that witness, not the truth of the claim.

The correction uses reflexivity of the actual single byte and retains both
as accepted `single-cell-control` cases. Both checkers accept them with zero
axioms. The other 44 candidates retain identical bytes and case metadata.
The final matrix has 25 acceptances and 21 altered-claim type failures, with
no fuel failures. No axiom, checker rule, entry predicate, input domain or
profile bound is changed to obtain the correction.
