# Original pattern control conditions — partial W09 review

The new private `with_pattern_routes` producer/importer extends the historical
81-condition API with 32 positional successor-choice observations. It retains
all 102 original PatternStep obligation IDs, their existing argument prefixes
and all 215 sequents. All 102 source conditions now have ordinary definitions
in this mode. Older observation, capture and native APIs remain reconstructible.

Seven governing-expression markers compare the once-evaluated value with its
exact source operand and require the normal successor. Eleven branches require
the true-first flag to equal the source condition and the false-second flag to
equal its complement. Both-selected and neither-selected states reject.
Two break completions require their exact successor; the no-match switch
expression requires its closed SwitchExpressionException route.

Each native scope keeps its explicit source-execution premise and records the
observed successor choices separately from SSA/slot observations. Matching
checks the exact successor frame entry, native target and Boolean guard
polarity. The builtin throw uses its exact source-node identity and closed
exception check ID. Route observations become conditional equality components
under that execution premise; they do not assert that a path ran. The original
native/capture definitions and component fallback at the binder limit remain.

The one erased Unit operand is now constrained by the original pure null
literal, with exact producer/value linkage and no exceptional successor.
It is represented by the canonical false Unit value. No native argument is
invented. The richer capture scopes have no unconnected observations.

Candidate generation passes all 88 states of the 11 branches, rejects stale
governing captures and absent completion routes, and checks 54 exact capture
route transports and the Unit literal transport. All old condition/native
declaration bodies are preserved apart from the replaced richer scopes.
Strict imports reject a changed source successor. The first candidate failed
compilation because an intermediate re-export was missing; its failure log is
kept separately from the passing candidate.

Selected verification covers this regression, the historical 81-condition
test, capture compatibility, typed observations, three scope units, Clippy
and format. These cover the changed conditions and their consumers; unrelated
parser, scalar and codec tests are not repeated. The manifest pins 388 source
and 348 fixture hashes, comprising 732 distinct inputs. All seven targeted
tests, Clippy and format pass locally and on the specified Linux server at
the exact published `a9d2e6fe` commit. The clean checkout, all 732 Git blobs,
execution logs and both test binaries pass an independent audit. All 72
Go/Rust stages and their independent report audit pass: 28 were executed for
the seven changed certificates, and 44 predecessor exits were retained after
exact input/binary comparison. Both checkers report zero axioms and reject
hash corruption. Receipts are under
`verification-logs/control-predicates/with-pattern-routes/`.

Condition availability does not establish source execution, native/source
equivalence, loop induction or an application theorem. All 987 original
application proofs remain open. Unit 5 and W09 remain In progress; W10-W12
remain Blocked. The full T06 gate is deferred to T06-W12.
