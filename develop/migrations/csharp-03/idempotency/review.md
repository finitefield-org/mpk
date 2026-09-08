# CSHARP-03-T05-W05 review

Scope: the strict optional idempotency attachment, shared complete structural
snapshot DAG, frozen precedence and path recipes, actual captured source/sidecar
matrix, W04 compatibility, and task status/evidence consumers. No delegation,
proof discharge, new workflow, or T05-wide gate is included.

## Review and fixes

1. Replay also needs an output value-bound obligation. A complete stored Response
   returned alongside the unchanged State duplicates logical value cells even
   when the event sequence is empty. Added `EventAndValueBounds` to Replay and
   required it in the path-shape test. This check would fail before the fix.
2. The absent-helper-contract fixture initially supplied an empty JSON file,
   which failed in the harness before helper attachment. It now omits the
   sidecar path and bytes entirely; the test requires a transition rejection.
3. The unreachable-helper fixture initially used a constant-true branch, producing
   an unreachable-code compiler diagnostic. Its replacement uses a nonconstant
   source condition and references the helper only from Run. The final captured
   source must import successfully before the test checks Apply-only reachability.

The production reachability walk uses actual invocation, constructor, and source
property-getter edges, consistent with the existing total-closure walk. Merely
being reachable from a second selected root is insufficient. All retained
member IDs are checked against their actual source owner and exact type. The
four record roles are distinct and cannot select field projections.

The equality helper's body is never accepted as a proof. Both the correct helper
and a helper omitting Context receive complete undischarged Command/Context DAG
obligations. The latter's finite CLR mismatch is observed without advertising
T06-W08 proof/counterexample execution. Type eligibility covers hidden/inactive
storage and empty float arrays using the shared structural implementation.
The bound Presence source retains separate Missing/null tags and all storage.

Error precedence and the disabled variant retain their exact shapes. Replay
preserves State, suppresses events, returns the stored Response and checks value
bounds. New success has checked increment, invariant/event/response/value bounds,
complete record append, and preservation of all prior records in order.
Independent import reattaches original captured sidecars; rehashing an embedded
replacement does not authorize it. Final manifests and VIR remain deterministic.

## Final verification boundary

The completed commands, case counts and hashes are recorded in verification.json.
The two W04 regressions passed in the initial combined run; its unfinished W05
run was superseded after fixture/recipe fixes by a targeted W05 rerun. Inventory,
transition unit, scoped lint and formatting checks are local. No full workspace
verification was assembled from separate commands. `./scripts/check-fast.sh`
remains deferred to T05-W06 under AGENTS.md.

The final task diff review has no remaining actionable findings. Proof discharge
remains T06-W08; cumulative T05 closure and its full gate remain T05-W06.
