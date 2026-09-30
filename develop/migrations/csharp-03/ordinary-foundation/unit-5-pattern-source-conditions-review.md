# Original pattern source conditions — partial W09 review

The ordinary predicate consumer now accepts the exact W04 source operands and
before/after slot observations. It retains all 215 sequents across the original
18 contexts and defines 61 of their 102 original PatternStep conditions:
17 source constants, nine selected-input joins, 19 assigned-slot loads,
12 stores and four bindings whose input and result types agree. Constants come
from the original source operation, not a recipe hash or an executed result.
All source symbols preserve their complete positional argument signatures.

Loads require the incoming assigned flag, equal outgoing assigned flag, an
unchanged represented storage value, and an exact loaded result. Stores and
bindings require the exact source input/result, outgoing assigned flag and
stored value; an overwritten incoming value is allowed. Nullable storage uses
the exact closed Option layout. Conditions are ordinary core definitions,
with no axiom, theory certificate or application proof added.

The richer capture scopes retain 113 normal/exceptional native paths, 423
observations, 108 compact definitions and five complete component fallbacks at
the combined binder bound. Matching uses full source/native binding identities
and exact source entry/exit kinds, even when both phases share a node ID. One
Unit operand on the original `type` conversion remains unconnected and is
reported explicitly. Source-condition availability and scope establishment
remain separate. The older APIs retain their JSON/hex pins and shared native
declaration bodies; richer scopes have distinct names and argument types.

The actual generated source bodies pass 77 positive and 150 changed-result/state
observations. Incorrect results, unassigned reads, outgoing unassigned flags and
changed stored/frame values reject. Successful overwrites accept different old
values. Strict regeneration rejects a changed proof-pending flag and a different
source context. Two recorded candidate failures were test defects: the first
incorrectly required the replaced scope names to survive in the richer mode;
the second omitted the original `total_variable` response fixture. Their logs
and exits remain separate from the successful generation run.

The selected verification covers the new conditions, historical capture/native
compatibility, typed observations, three premise/transport/binder units,
Clippy and format. Source and fixture hashes are checked before and after each
stage. The changed seven certificates are checked with the unchanged Go/Rust
binaries; unchanged predecessor stages require exact input and binary hashes.
All 72 stages and report/hash comparisons pass, with 28 newly executed stages
and 44 retained stages. Both checkers report zero axioms and reject hash
corruption. The six selected tests, Clippy and format pass both locally and on
the specified Linux server at the exact published `039bbc10` commit. The runner
verifies 381 source and 258 fixture hashes before and after execution. Its
635 distinct inputs are also matched to Git blobs. The server checkout remains
clean and both test binary hashes are independently checked after completion.
Completed execution receipts are under
`verification-logs/control-predicates/with-pattern-observations/`.

This checkpoint leaves 41 original source conditions undefined, including
payload extraction, arithmetic/comparisons, members, type/branch decisions,
conversion and source exceptions. Defining these relations does not establish
an application's execution, native/source equivalence, loop induction or any
of the 987 original application proofs. Internal unit 5 and W09 remain
In progress; W10-W12 remain Blocked. The full T06 gate is deferred to T06-W12.
