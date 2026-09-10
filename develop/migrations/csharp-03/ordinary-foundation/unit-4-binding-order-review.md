# Binding order review checkpoint — W09 remains open

Reviewed the new ordinary canonical-order definitions against source-design
sections 8.5 and 9.3 and the exact W06 canonical-order/uniqueness obligation.

- Resolve the concrete map/set key type from closed-instance metadata and
  verify the physical element layout. Use the existing canonical comparator;
  never use source-observation equality as ordering or grant a caller comparer.
- For maps, compare only keys. Test review added equal keys with different
  values (must reject) and ascending keys with independently changed values
  (must accept); comparing full entries would violate the frozen contract.
- Compare every adjacent active key using a counted ordinary aggregate fold.
  The last pair and index 2,048 are covered by full-capacity tests. The length
  is checked as a complete unsigned 32-bit word before accessing fixed-capacity
  storage. Empty/singleton and inactive-storage cases are explicit.
- Decimal cohort normalization and signed zero must reject duplicate keys;
  integer, Boolean, ordinal-string and structural comparisons retain their
  existing canonical behavior. The test oracle is the independent frozen
  structural model and cannot supply a production definition or proof.
- The new schema/importer adds only demanded `CanonicalOrder` mappings. Exact
  regeneration, cross-context/schema mutations and base closure equivalence
  guard against retargeting or silently replacing other definitions. Existing
  unresolved reconstruction/native/default/codec/proof symbols remain pending.
- The guard assembly loop was extracted without changing its emission order.
  The generator test checks all 45 guard certificate and metadata pins, as well
  as byte-identical output in the 36 contexts that demand no order predicate.
- Lint identified an unnecessary replacement allocation in a map-value test.
  The test now replaces the boxed value in place; its values and expectations
  are unchanged. The live semantic test executable predates only this
  allocation-only edit; the pinned replay and lint compile the updated test.

Verification is recorded in `unit-4-binding-order-progress.json`. Do not infer
that source semantics, full-capacity execution or dual-checker tests have passed
until their terminal results are recorded. No whole-unit completion or commit
is claimed at this component checkpoint.

Additional review coverage: the original seeded compound keys changed both
fields in correlated directions. Eight separate string/compound edge cases
now force equal-prefix ties and opposing field orders, so a wrong field
priority cannot be hidden by those samples. These extend test coverage; no
production comparator correction was needed. Latest lint passed with the
new cases; their executable results remain separately tracked.

The eight string/compound edge observations subsequently passed in 3.01 seconds.

The 140-case source semantic test subsequently completed successfully. The
three-test generation/semantic/full-capacity process passed in 1,124.53 seconds.
Together with the separately passed eight-edge test, pinned replay, same-byte
dual-checker cases, lint, format and inventory, this closes the order component
verification. It does not complete internal unit 4 or W09.
