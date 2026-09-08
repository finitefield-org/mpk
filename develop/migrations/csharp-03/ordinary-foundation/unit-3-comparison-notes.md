# W09 unit 3 comparison implementation notes

These notes record inspected requirements, not implementation or completion.
The ordered-fold helper is being verified before relation assembly.

- Reconstruct the reachable carrier/closed-instance set from validated VIR.
  Source enums compare underlying numeric carriers, including signed widths.
- Use existing scalar equality semantics. IEEE NaN is unequal to itself and
  both zero signs are equal. Such types and every enclosing type are not eligible
  for canonical total ordering, even for an empty collection or absent option.
  The same type-level rejection applies to closed exceptions.
- Compare source products in stored declaration order; sums by tag and the
  active payload; sequences and map/set elements lexicographically, then length.
  Equality uses semantic fields/elements, not physical padding or function equality.
- Money is the special product ordering: frozen storage is amount then currency,
  but comparison is currency then numeric decimal. Never derive its order solely
  from storage order. Decimal equality/order must ignore representation-only scale
  and zero-sign differences, using the reviewed numeric definitions.
- Concrete ordered-entry and transition products retain their specified field
  ordering. Map/set domain ordering constraints remain separate from comparison.
- Internal sequence-construction state is not a storable source value and does
  not gain a public comparison from having a product storage shape.
- A fold's count clamping does not discharge bounds. Sequence equality/order must
  receive valid lengths, use the minimum length for the element prefix, then
  compare full lengths. Recursive domains and public source clauses remain
  separate obligations.

Inspected sources: foundation/foundation-definitions.json ordinary_core and
money/collection templates; csharp_practical_structural.rs; the monomorphic
relation implementation in csharp_practical_vir_model.rs; ordinary scalar,
decimal, IEEE and ordinal helper implementations. The eight-unit W09 scope and
its exit gate remain unchanged.
