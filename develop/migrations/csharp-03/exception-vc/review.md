# T06-W05 direct review

Scope: exceptional-control generator, W01 contract-scope retention, private W04
helpers, VC wire/import/resource integration, original captures, affected tests,
goldens and task records. Review was performed directly without delegation.

Findings resolved:

1. Source uncaught throws are normally lowered through search/unwind to native
   exit edges. Checking only terminal abrupt-throw nodes omitted their clauses.
   Actual exception-to-exit edges now receive uncaught-result goals. Originating
   abrupt records cannot bypass an outgoing handler path.
2. A Boolean stand-in cannot represent a returned value. Pending normal summaries
   now bind the actual typed SSA result (unit needs no value argument). Exception
   outcomes retain actual abrupt SSA or named closed operation/check outputs.
   Finally relation names are unique per transfer, avoiding overloaded signatures.
3. Exceptional postconditions require exact exception attachment scope, and
   rethrow can carry a subtype of the active catch. Clauses are selected using
   the dynamic closed exact tag and first applicable path, with definedness,
   assignment, conditional ensures and explicit coverage goals. Real false/empty
   sidecars fail the Boolean oracle; they never become assumptions or receipts.
4. Edge ownership after a throwing operation cannot be copied from that block's
   normal output. Edge snapshots now name the destination entry and preserve all
   native phis and cleanup actions. Postcondition slots carry exact edge IDs;
   method-entry old slots and frozen exception observations remain distinct.
5. A filter may catch its own exception. Only an escaping exceptional edge is
   classified as filter failure; locally handled exceptions retain normal
   exception transfer. Search predicates separately preserve the original value
   and require earlier candidates to fail before selection. Mixed filter
   outcomes and closed ancestry are checked by a Boolean oracle.
6. Finally preservation and override require a finite completion rule table,
   not an untyped completion token. All five incoming kinds are checked against
   the existing completion algebra for normal and throwing cleanup; return,
   outward break and continue from cleanup have no admitted rule.
7. Functions containing source/constructor calls must retain call obligations
   even when their declared exceptional check set is empty. Inclusion now checks
   invocation tags as well as local exception edges; source replay tests require
   every such invocation to occur in the generated call handoff.
8. W05 terms and sequents must be reserved in the shared resource calculation;
   its groups depend on applicable W03/W04 groups and exact program digests.
   Tests recompute totals and exact import rejects removed edges/goals, changed
   regions, selection types, completion rules and injected runtime identity.

Final direct review: no actionable findings. Verification is recorded separately
in `verification.json`. Generated obligations and test-oracle results are not
semantic/kernel proofs; W09 owns ordinary expansion and proof assembly. The
full local T06 gate remains deferred to W12.
