# Native private-array storage relations

This component connects frozen allocate/read/fill/rewrite/freeze bodies and the
private construction.complete observation to original W03 SSA subjects.
Six captured source contexts contain 21 invoked definitions and 26 use points.
Uninvoked frozen construction bodies remain in the helper closure.

Eighteen ownership checks now bind to the exact source function, node, receiver
and symbolic state. Their ordinary equality proofs retain a checked dependency
on the corresponding flow proof. All 183 previously pending formula roles now
have source-scoped definitions; the generic operation failure remains unresolved
outside those use points. Nine original ownership records, native execution and
application proofs remain pending. No trusted host verdict or ownership flag is
introduced. Other data families, unit 5 and W09 remain incomplete.

Update-result equality compares the changed branch recursively, compares every
untouched subtree, and checks all result role/padding bits. Upper index bits
prevent wraparound. Exact zero predicates share empty regions without reducing
the physical coverage. Other results are compared across their full carriers.

See ../verification-logs/construction-data/ for exact commands and results.

Freeze equality preserves precisely the first 4096 physical cells, fixing the
two high private index bits only after the twelve low selectors. Published
length padding remains checked. The final affected freeze run passes 20
observations in 1.05 seconds; the prior scoped run passed 178 in 199.62 seconds.
Unchanged definitions are compared independently to reuse those earlier checks.
The source-scoped ownership extension passes 220 current runtime observations
and all six same-byte dual-checker/hash-corruption cases, with zero axioms. See
`../verification-logs/construction-data/source-ownership/verification.json`.

## Symbolic ownership record extension

Nine original symbolic discard records now bind to their exact source local,
invocation, incoming-edge cleanup/phi/join, cleanup-coverage and terminal
equations. Each has an ordinary equality proof with the complete source flow
theorem retained as a checked dependency. Concrete borrow/transfer records
without a matching symbolic trace remain pending. Native execution and
application scopes remain open.

All previous terms, declarations and operation metadata are unchanged prefixes.
The existing 220 runtime observations therefore remain applicable; nine new
record predicates and 36 repeated source-use checks pass in 8.59 seconds.
All five changed data certificates and all three added loop/finally certificates
pass both unchanged checkers and hash-corruption rejection. The return-array
certificate retains its previous acceptance of identical bytes. Seventeen
original ownership records are covered across the nine contexts; see
`../verification-logs/construction-data/ownership-records/verification.json`. The earlier
source-ownership receipt describes the archived pre-record bytes.
