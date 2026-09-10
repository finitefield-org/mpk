# W09 decimal data connection review

The adapter reconstructs original W03 semantic definitions and use points from
validated VIR. It accepts the decimal subset of FloatingDecimal, checks exact
scalar signatures and failure-table lengths, and rejects non-exception or
tagged-result checks. Other data families remain explicit pending definitions.

Result relations compare all 512 product bits, including sign, scale,
coefficient and padding. The shared integer/floating comparator only adds 512
to its accepted widths; its prior paths and all six regenerated certificate and
metadata pins remain byte-identical. A new generic raw-cube test rejects each
of 512 one-bit mutations, including padding, and accepts identical storage.
That test checks the comparison machinery, not decimal input-domain validity.

Decimal arithmetic reuses the existing independent failure definitions. Zero
and overflow can coexist, so neither may be replaced by a predicate already
masked by an earlier exception. Original W03 failure predicates, prefix guards
and failure guards keep that separation and priority. Point lowering reuses the
corrected source-subject to de Bruijn mapping; function/node, operand/result SSA
and normal/exceptional successor metadata are preserved.

The three native inputs produce nine definition occurrences and eleven use
points with exact existing scalar-body dependency closures. Source successor
and certificate mutations reject. Candidate tests, comparator regression,
previous-pin replay, lint, inventory and formatting pass. Native value/exception
observations remain pending. All three earlier unshared certificates passed both
unchanged checkers with zero axioms and matching reports/hashes; hash mutations
rejected. Final shared bytes must pass the checks below.

No actionable static finding was identified in this adapter scope. It covers
six fresh native operations, not all decimal signatures or complete native
bodies. Original units 3-8 and full W09 acceptance remain open.


A subsequent execution review found that repeated references to the computed
term ID did not establish call-by-need sharing across result bits. The relation
now binds that result with an ordinary core Let; comparison variables are
computed Var(0) and actual Var(1) beneath it. This preserves lazy demand and
unchanged scalar definitions. Final candidate/runtime/checker validation is
required again for these changed relation bytes; previous checker results are
historical evidence, not acceptance of the revised candidate.


Final shared candidate reconstruction, both decimal comparator regressions,
lint, inventory and formatting now pass. The generic sharing regression
observes identical true results with 162,221 versus 676,539 core transitions.
This measures the bounded regression only. The revised native runtime and
same-byte dual-checker runs remain pending in the progress receipt; the old
runtime was explicitly superseded with no verdict.
