# Source strings checked against their original graph

The private graph binding derives a string catalogue from the complete original
contract value domain. The checker independently compares every original type,
nominal underlying edge and error case, follows the original type identities to
find every string type, and checks each generated representation with the Rust
kernel and zero axioms. It retains compact receipts for exact reconstruction;
the caller cannot supply a smaller catalogue or replace a certificate. Empty
string domains remain empty and do not imply a checked string theorem.

The existing nil-only, three-case, heterogeneous six-case, 70-case and four
builtin/nominal string-type inputs passed: all eight certificates were byte-equal
to the earlier independently Go-checked streams, and the Rust reports agree
with those Go verdicts and hashes. Crossed domains and reconstruction from a
different catalogue reject. The complete original 47-function input retains
its empty dynamic-error universe and all three source strings; all three pass.
An additional original string function has no `error` type; both checkers
accept its 644-declaration row with zero axioms and certificate hash
`48925390c845e7a773a5f1b97668203638a506e21d6f232dfa9b649b49ba5e26`.
The original no-string domain has no rows.

The first error-free test incorrectly assumed that the original 47-function
domain had no `error` type and failed. It actually has an error-typed signature
with no dynamic cases. The corrected test retains the entire original input
and separately checks the original error-free function; it passed. Linux-target
compilation and strict affected Clippy, including the test-configured live
observer, pass. Changed-file formatting passes.

The complete live 182-package error-dispatch application retains 5,517 source
objects, all 181 exports and all 5,577 registered SDK files. Its actual
`error.Error()` result yields one source-string row. The held-root and test-only
SDK owners agree; another complete graph and another original function reject.
Both checkers accept the same 655-declaration string certificate with zero
axioms and certificate hash
`f077cbc610d4fd854f4c40581d729b54633bee12023e5a62283ce758611be89a`.
The entry, call and both error-value certificates are byte-identical to the
previously Go-checked streams. The empty-string live regression is pending.

The original catalogue cases, crossed-domain controls and full capture were
selected because the change adds catalogue coverage and connects it to the
same root consumer. The empty-string live case exercises that consumer without
any string types.

These checks establish representation mathematics and exact source identities,
not source execution, string-operation semantics, postconditions, a complete
foundation, installed launcher authority or a public profile. T02-W01 remains
open. The cumulative T02 `./scripts/check-fast.sh` gate is deferred to W04.
Large verification outputs are temporary and are not committed.
