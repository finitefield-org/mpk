# Original pattern primitive conditions — partial W09 review

Historical 81-condition checkpoint. The later 102-condition source-route mode
is recorded in `unit-5-pattern-routes-review.md`.

This extends the historical 61-condition checkpoint in
`unit-5-pattern-source-conditions-review.md` to 81 of the 102 original
PatternStep conditions. All 215 sequents and the 18 original source contexts
remain. The 20 additions cover three unary updates, five binary operations,
three pattern equalities, two relational patterns, two type/presence tests,
one nullable payload binding, one stored-field read, one array read, one
string-length property and one null conversion.

Primitive conditions apply the existing ordinary success guard AND result
relation to the original free source operands and result. Matching requires
the exact operation ID, source node/ordinal, entry/exit, anchor result,
artifact node and complete typed operand/result list. This does not assume
successful native execution or use an executed native result as a source fact.
Updates also require the independently represented integer-one literal.
Type tests and payload bindings require the exact original target-type key.

Nullable extraction requires Some and stores the extracted payload in the
assigned source slot. A string-length read constructs the canonical Some
receiver and composes the ordinary constructor/read relations. Null conversion
requires the physical Unit value and the complete canonical None result.
Existing condition and native declaration bodies are preserved. No axiom,
theory certificate, proof node or application proof is introduced.

The generated bodies pass 102 positive and 183 changed-value/state
observations in the recorded candidate run. Tests reject checked update
overflow, division by zero and MIN/-1, missing or changed payloads, invalid
array indices, wrong results and changed slot state. A changed successful
update and the second array element are also checked. Strict regeneration
and the historical capture pins remain covered.

Verification selects the source-condition test, historical capture
compatibility, typed observations and three scope units, plus Clippy and
format. These cover the changed definitions and their consumers. Unaffected
parser, scalar and codec tests are not repeated. The runner verifies 384
source and 294 fixture hashes before and after each stage; there are 674
distinct inputs. Logs and execution receipts are kept under
`verification-logs/control-predicates/with-pattern-primitives/`.
The six targeted tests, Clippy and format pass locally and on the specified
Linux server at the exact published `0aed36d4` commit. The clean server
checkout, all 674 Git blobs, execution logs and both test binaries are
independently verified. All 72 Go/Rust stages and the independent report audit
pass: 28 stages were executed for the seven changed certificates, and 44
predecessor exits were retained after exact input/binary comparison. Both
checkers report zero axioms and reject hash corruption.

The 21 remaining source conditions are seven governing-expression markers,
11 ordered branches, two break/handler completions and one no-match builtin
throw. The Unit conversion operand remains explicitly unconnected in the
native/capture scopes. Source execution, native/source equivalence, loop
induction and all 987 original application proofs remain open. Unit 5 and
W09 remain In progress; W10-W12 remain Blocked. The full T06 gate is deferred
to T06-W12.
