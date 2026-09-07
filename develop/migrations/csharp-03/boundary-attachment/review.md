# W01 local task review

Scope: CSHARP-03-T05-W01. Review covers the complete task diff, actual source
captures, source-to-binding correspondence, typed default and codec rules,
ordinary VIR import and artifact linkage. No review agent or hosted CI is used.

## Resolved findings

1. A parsed boundary root alone was insufficient: nested field schemas, exact
   method signatures, per-direction identities, missing/null rules and frozen
   profiles now attach against captured source. Boundaries are retained in VIR,
   manifests and artifact links and revalidated by the ordinary importer.
2. Revalidating the cumulative binding closure during control attachment added
   already-present semantic roots a second time. Exact type/provenance pairs
   are now retained once. Actual presence contracts exercise this previously
   failing path through both emission and independent import.
3. Contract literals spell integers as strings, while boundary defaults use
   the frozen boundary token kinds. The shared decoder now has an explicit
   boundary-default profile, retaining legacy contract behavior. Field-selected
   fixed decimal codecs are honored even inside nullable/presence defaults.
4. A standalone outcome projection would reject a presence whose payload has
   another application business binding. Attachment now reuses the cumulative
   binding closure and ordinary source-binding importer, with W12 outcome shape
   validation. The retained Presence/Instant source checks that composition.
5. Source public-default eligibility is a proof condition, not evidence of the
   CLR zero's tag. The actual zero is obtained from the already checked source
   default graph; a missing-tag candidate retains its public-invariant obligation.
   The null-tag zero is not promoted, and missing/null defaults never collapse.
6. Type and default traversals require resource accounting even before document
   decoding exists. Depth, cell, field and name bounds are enforced; malformed
   default and field inventories fail with the boundary limit error.

## Final review

No open findings. The task adds private attachment validation, not an input
parser, invocation route or proof discharge. Source and binding invariants
remain explicit obligations for their existing owners. Frozen vectors,
profile activation and historical conformance receipts are unchanged.
Final verification results and hashes are recorded in `verification.json`.
