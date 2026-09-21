# W03 string operation relations

Eight original C# contexts produce 52 definition occurrences and 52 SSA use
points, covering 25 operation IDs and 48 concrete argument signatures. Six
captures are reused from `../string-clauses/`; two add native multi-argument
operations. Their original-source capture evidence remains unchanged.

The result relation compares every physical storage bit, including length,
UTF-16 content, and inactive/padding addresses. Constructor emission now shares
index/range work per code unit and compares address bits directly. The output
bound and ordered failures are unchanged. See
`../unit-5-string-construction-runtime-review.md` for the binder/selector review.

The reported non-null `string.interpolation.restricted.cs` test passed in
51.54 seconds after the 600.23-second timeout. The user then completed all
25 remaining partitions: 184 additional result/guard observations, in
24.80–172.49 seconds per partition. Together with retained basic/ordinal and
fixed-interpolation evidence, all 48 scoped signatures and 320 observations pass.
The 44-case constructor oracle passed in 30.63 seconds; the large binder fixture
passed both checkers in 21.53 seconds. Future affected runs below one minute
are agent-owned using these measured timings.

There are no pending tests in this scoped component. `pending-runtime.json`
and `run-long-checks.sh` retain the completed handoff plan for reproducibility;
their name does not indicate current pending work. Do not rerun them without
an affected change. The 27 user logs and manifest are archived in
`../verification-logs/string-data/user-long-completed/`.

Four data certificate contexts changed (construction/native, nullable/non-null).
All four passed both unchanged checkers with zero axioms, matching reports and
rejection of corrupted hashes. Four basic/ordinal certificates are byte-identical
and retain earlier checker evidence. Across all affected families, all 14
changed certificates passed both checkers.
Candidate reconstruction and dependent contract/cache/comparator checks pass.
`certificates.json` records current raw-byte and certificate hashes. Replaced
pins and historical evidence are retained under
`../verification-logs/string-data/hoisted-index/previous-pins/`.

Current results and source hashes are in
`../verification-logs/string-data/user-long-completed/verification.json`.
No full-run memory measurement was taken for this correction. Other data
families, native control/ownership and application proofs remain outstanding.
This does not complete unit 5 or W09; the full gate remains at T06-W12.
