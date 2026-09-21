# Symbolic ownership proof candidates

These candidates attach ordinary equality theorems to the exact ownership flow
and receiver-state equations reconstructed from validated source VIR. The 14
captured contexts contain 14 closed flow conclusions and 40 closed receiver
failure conclusions. They are not application-VC completion receipts.

The generator copies the frozen equality foundation and constructs proofs with
reflexivity, congruence and transitivity. No axiom, proof-node entry, theory
certificate, trusted evaluator verdict or checker modification is introduced.
Each word comparison still covers all 32 bits. Named comparison prefixes keep
individual conversions small. Cases with more than two allocation origins use
balanced comparison ranges over all 32 bits and a checked universal
WordTransport theorem. The final flow conjunction is balanced and
retains every original equation. Pure literal selector words may share checked
C5 definitions, and small constructor computations may share proof terms.
Computed Bool recursor majors retain their neutral syntax during conversion.
Multiple-slot cases also share two universally quantified BoolStep theorems,
with explicit equality premises, instead of repeating concrete proof lambdas.

Both checker acceptance and semantic mutation checks are required. Successful
candidate generation alone is not acceptance. Resource exhaustion and test
framework timeouts are diagnostics with no semantic verdict. Current results
and historical failed attempts are retained under
`../verification-logs/ownership-proofs/`. All fourteen current candidates pass
both unchanged checkers, zero-axiom/report agreement and hash-corruption checks.
All three valid-hash semantic mutants are rejected for type mismatch.
The old 941,806-byte eight-live pin and timeout diagnostics are archived under
`before-word-mux-promotion/`. Its 470,792-byte replacement is now promoted:
Go positive/hash checks passed in 3847.845/3867.047 seconds, and Rust
positive/hash checks in 8.208/8.755 seconds. Other thirteen candidates and all
three semantic mutants retain their exact bytes. Current receipts are
`current-proof-verification.json`, `word-mux-progress.json` and
`word-mux-eight-complete/` in that evidence directory.
The local `run-checks.py` script records each checker stage, exact input hashes,
and timeouts and cleans up child processes. All tests are agent-owned.

`application_scope_pending` remains true. Eighteen W03 ownership uses are now
specialized at their exact source points and linked to the associated flow
proofs; that six-certificate checkpoint passed both checkers. See
`../verification-logs/construction-data/source-ownership/verification.json`.
Seventeen symbolic ownership records across nine data/control contexts now
have generated predicates and checked proof terms; all nine current certificates
pass both unchanged checkers and hash-corruption rejection. See
`../verification-logs/construction-data/ownership-records/verification.json`. A point-specific closed result does not justify globally defining
the generic ownership failure predicate as false. Concrete ownership, borrow,
transfer, remaining native control relations and application proof assembly
remain outstanding. Unit 5 and W09 are incomplete; W10 remains blocked.
