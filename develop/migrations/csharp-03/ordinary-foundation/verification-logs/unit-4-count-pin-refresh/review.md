# W09 unit-4 count-helper consumer pin correction

Status: the binding declaration/linkage audit, all 48 distinct same-byte checker
cases and pinned replay passed (`verification.json`). The additional consumers
below have separate checker runs. This is not unit-4 completion.

The existing binding-relation preservation test exposed consumer pins left
behind by `fe9dc4fc3ebc2554127591e708b2a2d224357751`. That commit replaces the
saturated logical-cell addition helper's generic register pipeline with its
guarded 17-bit ripple addition and shared Boolean lets. The reconstruction
emitter extraction does not introduce this change.

The original 45 relation contexts, 45 guard contexts and 9 ordering contexts
were regenerated with their existing strict import and mutation checks. The
three declaration audits compare types and bodies by syntax and global names,
independently of term-table numbering. In every changed certificate:

- No declaration was added and no retained declaration's type changed.
- Only the saturated cell addition `DomainCount` result body changed.
- Removed declarations are its old `Block.B3` through `Block.B27`, `State`
  and `Success` helpers and, where no other operation needs them, the four
  depth-10 cube helpers.
- Manifest changes are limited to certificate hashes and measured term,
  declaration and static-transformer counts. Source, foundation, binding VC,
  projection, predicate, agreement and unresolved-symbol metadata are identical.

The detailed before/after hashes and declaration names are in the three
`*-audit.json` files. There are 23 changed relation contexts, 23 changed guard
contexts and 9 changed ordering contexts. Their 55 occurrences contain 48
distinct certificate byte sequences. Checker selection covers one occurrence
of every changed byte sequence; the seven reused occurrences must match the
representative's complete raw bytes and domain-separated certificate hash. The
receipt records both hashes explicitly. The other 44 relation/guard
contexts are byte-identical and retain their existing checking evidence.

The affected tests are selected because these programs embed the shared count
helper. Unaffected projection and binary rebuild pins already passed unchanged.
The boundary-rule semantic regression passed all 15 contexts and 250 predicate
observations in 1,034.07 seconds. Its complete metadata and all 15 certificate
byte sequences are unchanged (`boundary-rules-audit.json`); those bytes retain
their prior dual-checker evidence. The repository-wide gate remains deferred
to T06-W12.

A subsequent scan found the same old helper in 274 further current codec, JSON
and clause fixtures across 38 directories. This marker scan is an affected-
consumer inventory, not proof that all their differences are identical. Of
these, 268 have source-bound program metadata covering 137 original VIR contexts
and 15 producer schemas; six are deliberately source-free helper certificates.
All 268 source programs now regenerate and import against their exact original
VIR hashes. Seven contexts came from `data-stage-replay.json`, closing the
initial request/response inventory's missing-context finding. Of the 268,
262 differ only in saturated addition and its obsolete helpers. Six also
retained the earlier fixed product-count, zero-region and inactive-tail domain
definitions from the same cause commit. Those six have separate declaration
review, original-source test replay and recursive-domain semantic evidence.
All six source-free helper tests passed as well.

The 274 additional occurrences contain 224 distinct byte sequences. Their
reviewed promotion updates canonical program copies and measured hash/count
fields while preserving source identities and semantic metadata. Their two
checker batches remain separate from the completed binding checks. See
`additional-consumers/verification.json` and `additional-consumers/review.md`;
the 55 binding occurrences above must not be presented as this larger closure.
