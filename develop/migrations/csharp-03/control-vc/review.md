# T06-W04 direct review

Scope: the W04 control generator, original-source captures, VC/import/resource
integration, contract scope retention, affected construction tests and progress
records. No review delegation or GitHub Actions were used.

Findings resolved during the implementation/review loop:

1. A backedge target is the same header node as the previous snapshot. Separate
   post-edge identities now prevent preservation/decrease goals from comparing
   a snapshot with itself. The variable-measure fixture rejects erased identities.
2. W01's ordinary free indices reverse declaration order. Control predicates
   now bind that order, including mixed parameter types and actual result/entry
   values. Identical requires/ensures expressions are selected by their verified
   old-state scope as well as owner; loop clauses use their original shared scope.
3. Missing logical local values cannot become arbitrary initialized inputs.
   Only parameters start assigned; original stores/loads constrain slots and
   actually used contract slots have assignment goals. Preconditions are pinned
   to immutable method-entry observations.
4. Branch order, break and continue cannot be inferred from a loop header's
   source spelling. Every actual edge is retained; true/false guards partition
   the branch and continue decreases occurs at the real backedge after updates.
   An explicit function-entry edge also covers a loop starting at function entry.
5. A nonexhaustive switch is allowed to have an explicit exception path. The
   closed built-in check route now covers that path. Non-data exception and
   normal edge guards are complementary; exceptional composition remains W05.
6. Nested pattern steps and enclosing-loop returns must not duplicate ownership.
   Steps belong to their innermost decision; method postconditions occur once
   per actual return, including returns following normal loop exits. Only
   property getters referenced by the decision get total/pure obligations.
7. Source graphs are retained once per function, and generated construction
   loops retain complete region/phi/ownership evidence as pending rank goals.
   Resource counts include control terms and the shared declaration-name union.
   Innermost pattern owners are precomputed instead of repeatedly scanning all
   ranges for each node of every decision; the complete golden hashes are unchanged.
8. One hostile test originally replaced an already-true guard with true. It now
   selects an actual branch variable before changing the guard, so rejection is
   tested with a real semantic mutation.

Final direct review: no actionable findings. Targeted verification results are
recorded in `verification.json`. A total request, compiler exhaustive flag or
capture acceptance does not certify any semantic goal. Ordinary proof expansion
and checker receipts remain W09, and the full T06 gate remains W12.
