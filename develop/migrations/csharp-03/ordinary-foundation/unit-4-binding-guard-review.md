# Binding guard review checkpoint — W09 remains open

Direct review of the new guard generator and its shared assembly extraction:

- The old relation program retains its schema and emission order. Guard
  generation uses a separate schema and corpus; an importer for one cannot
  accept metadata or bytes from the other. The existing projection, observation
  and agreement definitions cannot be replaced through the guard metadata.
- Source tag comparison covers the source enum's full width rather than the
  semantic tag's smaller range. The signed-minimum-vs-zero source fixture and
  unknown-tag tests exercise this distinction.
- Payload/member agreement reads the original mapped source field before
  conversion. Only active payloads constrain semantic payload agreement; source
  reconstruction still observes every field, including unmapped and inactive
  fields. A sidecar is not a reconstruction or native-body proof.
- Bound predicates inspect the full length word; validation's invalid payload
  is checked conditionally, with unknown arms rejected. Review/test execution
  found duplicate emission of the shared validation length getter between the
  upper-bound and nonempty predicates. The getter is now emitted once.
- Test review found that direct `MonomorphicValue` equality incorrectly treats
  decimal 1.00 and 1 as unequal observations. The test oracle now independently
  normalizes decimal cohorts recursively while preserving exact IEEE bits.
  Per-member expected comparisons select only the corresponding Money/product
  role; they do not compare unrelated fields as part of that member's result.
- Internal construction-state observation, actual defaults/use restrictions,
  reconstruction, canonical order, native outcomes and application proofs remain
  unresolved. The symbol inventory preserves those gaps. No axiom, trusted
  primitive, or unconditional success predicate was added to fill them.

The first corrected-binding checker run encountered compilation failures
because the preceding refactor referenced a guard module before that file
existed. Its final status is failure, not checker agreement. The file is now
implemented and compiles; a new complete run against the unchanged 45 binding
relation certificates is tracked separately. Passing individual subcases of
the failed run does not constitute a passing corpus.

Verification is recorded in `unit-4-binding-guard-progress.json`. The deep
member tests and full dual-checker runs may still be live at this checkpoint;
this document does not certify completion before their terminal results.

The subsequently recorded terminal results pass all guard checks: 45 same-byte
dual-checker cases with zero axioms/hash-corruption rejection; 1,019 source
tag/member/payload observations; 321 bound/unknown-tag observations; 52 deep
member/payload observations; exact pinned replay/closure preservation; lint,
format and inventory. No guard component finding remains from this review.
This does not constitute review or completion of the outstanding W09 units.
