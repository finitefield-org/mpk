# Unary reconstruction candidates — partial W09

The new program composes the existing forward projection and binary rebuild
with one explicit, typed source completion per nonidentity source type. The
completion is encoded by the ordinary literal emitter. The resulting function
has exactly the frozen unary semantic-to-source signature. Identity retains
the existing definition. Missing, duplicate and foreign completion types reject.
Canonical import regenerates both metadata and certificate using the exact
completion values; their order does not affect the result.

Constant completion is a candidate construction strategy. It is not a new
profile restriction, a default-value admission or an assumption that public
invariants hold. General semantic-dependent witnesses and universal proofs
remain required where this strategy is insufficient.

The combined program includes compiled source clauses, recursive public domains,
complete source observations, semantic equality and every stored-member
comparison. It translates the original W06 projection-totality, reconstruction,
source/semantic round-trip, member-reconstruction and identity sequents directly
to ordinary Bool definitions. Each condition is the implication from the
original assumptions to all original goals. The complete original sequent is
retained for later theorem assembly. Every binding VC stays in `pending_proof_ids`,
including those whose condition has been defined. Remaining VC constants stay
explicitly unresolved. No application theorem is discharged here.

The `remapped-boundary-sequence` fixture remains a negative final proof case.
For each of its two source wrappers, the test changes `Extra` while retaining
the projected value. Both inputs satisfy the source domain. The selected
completion reconstructs the original value; the changed value must fail both
the complete source round-trip condition and the `Extra` member condition.
Changing the supplied completion also invalidates the old candidate import.

Verification scope: the new source-derived candidate corpus, the existing
projection/rebuild/relation certificate pins affected by emitter extraction,
the two unchanged checkers, inventory, targeted Clippy and formatting. These
checks address changed composition and its immediate consumers. The independent
decimal fixed-format semantic matrix interrupted by the mistaken stop judgment
must also finish. Current results are recorded separately; this review is not
a completed verification receipt. The T06 whole gate remains deferred to
T06-W12, and unit 4/W09 remain In progress.

The current production review confirms that completion validation precedes
hashing/cloning, every required nonidentity source type is supplied exactly
once, and the content-addressed literal is called inside a unary semantic
function. The original VC sequents are regenerated from the validated VIR;
their variable types and complete assumption/goal lists are preserved, and
unknown symbols fail lowering or remain explicitly unresolved. A compiled Bool
condition never removes a proof obligation from the pending list. The shared
source-clause/projection environment reuses only deterministic internal names;
standalone projection and rebuild closure/pin comparisons passed.

The first optimized full test completed 44 source contexts, then failed in the
test's final negative fixture because captured member facts do not contain an
`id` field. The test now derives the stored-member ID from exact owner, name,
type and storage facts, as the existing independent projection oracle does.
The same negative case has an isolated test before the full 45-context rerun.
This corrects test selection of the original member VC and changes no ordinary
production definition. Its focused and full outcomes are recorded separately in
`verification-logs/binding-reconstruction/member-id-fix-verification.json`.

The resumed full semantic run passed all 45 contexts: 37 candidates, 206
conditions, 12,683 bit observations and four negative conditions. The new shared
condition compiler preserves the complete metadata and bytes for all 45 cases.
All 45 candidate certificates passed both unchanged checkers with matching reports,
zero axioms and changed-hash rejection. The current-byte and terminal-log audit is
`verification-logs/binding-reconstruction/verification.json`; scoped review has no
remaining findings. This completes only the explicit constant-completion component.
