# W09 Money integration checkpoint review

The structural program retained Money operation IDs as deferred even though the
standalone Money generator now provides their ordinary definitions. The current
change borrows the owning Relations instead of allocating a separate one, so
storage, relations, helpers and counted iteration pipelines are shared. A single
Money-local decimal cache is used across every concrete Money instance. The
standalone generator initializes the same state in the same order as before;
its exact pinned replay must establish byte preservation.

All ten expanded Money operations are emitted regardless of source invocation.
Only transition instances remain in the deferred list. Construction ownership,
source/public domains, currency-predicate proofs, native bodies and original
units 4–8 are still required; emitting the Money definitions discharges none of
those application obligations.

The integrated source test checks operation IDs against every expanded instance
and validates every metadata definition reference. Its metadata mutation set now
includes Money. A separate two-source/three-instance comparison checks all
30 Money operations and their complete ordinary definition closure against the
standalone generator, including exact predicate binders and failure definitions.
It also rejects removal of the currency predicate and reordering of create's
failure metadata. This comparison does not rely on finite host observations.

Production compilation, exact standalone replay, integrated closure and final
44-source checker verification passed. These results close this integration
component only; source/application proofs and full units remain outstanding.
The whole-repository gate stays deferred to T06-W12.

## Corpus coverage finding and correction

The original integrated corpus contained only the first Money source. The
shared string/enum currency capture was exercised by the dynamic comparison
but had no integrated pin for both-checker validation. Final generation now
adds that captured request/response pair, checks its identity and uniqueness,
and requires 44 sources. The unchanged checker harness now expects 44 integrated
certificates. The full component comparison includes both Money sources and
requires 118 source-component pairs, with exact Money metadata agreement.
The shared 43-source helper used by other components is unchanged.

Initial generation/comparison jobs started before this corpus correction, so
their compiled scope must be distinguished from the final generation/replay.
The final gate must exercise all 44 sources and 118 component pairs. Current
Git whitespace and Go-format checks passed. No production consumer uses the
deferred-instance list as proof admission; source currency/body proofs remain
separate, and the public route stays inactive.

The final 44-source revision passed lint and format checks. Its first link
attempt failed with ENOSPC before test execution, so it supplies no semantic
result. Five obsolete incremental compilation caches were removed after
confirming no rustc process was active; current binaries, logs, pins and live
tests were retained. With 21 GiB available, generation resumed as v3. The first
intermediate run generated binding-vc-money with all ten operations at 149,789
terms, 1,701 declarations and 10,441 transformers; that partial result does not
establish the complete 44-source gate.

Final generation/import/mutations passed all 44 sources in 147.95 seconds,
including the shared string/enum Money source (150,315 terms, 1,779 declarations
and 10,441 transformers). Standalone exact replay and the two-source dynamic
comparison passed all three instances, 30 operations and 2,976 transitive
declarations, plus currency-predicate and failure-order mutations (292.20s).
All five inventory tests passed. Current 44 pins are installed, preserving
42 earlier byte sequences exactly and archiving all 43 old pins/metadata under
pre-money/. Complete 118-pair comparison and current pinned replay passed in
163.58s, comparing 1,235 roots and 14,183 transitive declarations with mutations.
All 44 integrated certificates subsequently passed both same-byte checkers with
zero axioms and hash corruption rejection (5570.951s). The corpus coverage
finding is resolved; direct review found no further issue in this integration
component. Remaining source obligations and W09 completion are unchanged.
