# Structural verification-expression clauses

The frozen `structural_equal` and `structural_compare` recipes reuse existing
ordinary semantic relations. Comparison is available only for total types;
nominal argument IDs and Bool/i32 results are checked. Partial recipes still
require definedness work and are rejected.

`requests.json` and `responses.json` retain the exact original C# source/sidecar
capture. Its Value invariant tests source equality, i32 equality with 1 and i32
comparison with 0. The resulting predicate is true exactly when Amount is
negative or 1. Seven signed boundary observations pass. The standalone source
and integrated structural/public candidates are pinned in `certificates.json`;
both unchanged checkers accept identical bytes with zero axioms and reject
actual hash corruption.

All standalone source and public-default definition dependency closures, as
well as every original structural definition, match the integrated program for
this same VIR. Relation caches are carried with builder/storage and bound to
that VIR to prevent duplicate or cross-context definitions.

See `../unit-4-structural-clauses-progress.json` for selected verification and
its rationale. These are ordinary definitions, not application-VC proofs.
Internal units 3–8 and W09 remain open; the full gate is deferred to T06-W12.
