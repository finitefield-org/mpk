# W09 structural equality data connections

Five independently captured native inputs cover seven StructuralEqual use
points over i32, i64, Boolean and a declared enum. The adapter retains existing
semantic relation definitions, exact W03 SSA/CFG identities and explicit pending
data families. Ninety-eight source result/guard cases pass. Five matching
candidate/runtime certificate pairs total 150,236 bytes (maximum 49,769 bytes).
Both unchanged checkers accept all five same-byte certificates with zero axioms
and matching reports/hashes; hash corruption and metadata mutations reject.

Native != retains the separate Boolean negation step. CanonicalCompare delegates
to the existing total-only comparison helper but has no fresh native use point
in this corpus. Input domains, complete native bodies and application proofs
remain separate required work. See `../unit-5-structural-data-progress.json`
and its direct review. Units 3-8 remain open; full gate deferred to T06-W12.
