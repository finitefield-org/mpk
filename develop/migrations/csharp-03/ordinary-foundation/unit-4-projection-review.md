# W09 unit 4 forward-projection component review

Scope: the new ordinary binding projection generator, public exports, original
source/request/semantic tests, fixed certificate corpus and checker registration.
This is a component review, not completion of unit 4 or the full W09 review.

Resolved findings:

- Inline ordered-map element products could not be lowered through the original
  reference-only element path. The generator now requires exact closed ordered-
  entry argument and shape matching. Original ordered-map capture generation
  failed before the fix; all source cases now generate and replay exactly.
- Mapping every physical sequence slot would corrupt inactive zero storage when
  a zero source tag projects to a nonzero semantic tag. Conversion is guarded by
  index < full u32 length and inactive storage is zero. The supplemental original
  source exercises this with value=0/missing=i64::MIN/null=1 tags. It checks all
  child bits at inactive indices, including 4095; removing the guard exposes the
  semantic value-tag bit in those zero-expected observations.
- Truncating a source enum comparison to 32 bits would alias zero and i64::MIN.
  Comparisons span the complete source enum width. Supplemental source samples
  distinguish the missing and value arms with identical low 32 bits.
- Forward projection cannot supply an inverse when stored Extra fields are lost.
  No nonidentity inverse is emitted. Full source member IDs remain attached;
  changing Extra changes source storage and preserves every observed projected
  bit, while reconstruction remains a separate proof obligation.
- Inventory drift from the new ordinary namespace consumer is recorded as one
  additional file, with the independently checked path hash, total and receipt
  linkage updated together. Search/mutation validation must pass on those bytes.

Test-harness corrections: captured member IDs are derived from source type/name/
type/storage facts, rather than reading a nonexistent id field; boundary_field
uses its frozen ineligible default policy for the remapped-tag fixture. The
initial option request was rejected by the binding schema and was not retained.
The checker command without its build tag ran no tests and is not verification;
the tagged default-cache attempt failed permissions. The actual checker run uses
the existing writable temporary Go cache and the checkeragreement build tag.

Direct review checked exact regeneration/context links, role coverage, source
member addressing, enum widths, active payloads, unused padding, bounded indexing,
inline map dependencies, ordinary limits and unchanged checker acceptance rules.
No remaining actionable finding in the reviewed forward-projection component;
its exact verification state is recorded in unit-4-projection-progress.json.

Missing universal source-domain/target-domain proofs, reconstruction relations,
operation/outcome commutation, canonical codecs, native/control semantics,
transition/replay propositions and complete proof assembly remain in the original
W09 plan. Observations and helper certificates cannot discharge these obligations.
