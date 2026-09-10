# W09 boundary-literal component review

Direct review checked that only the immutable, reproduced boundary-run program
can supply values. Its source IR and recomputed static boundary VC identity must
match the current VIR. The complete run hash remains in the ordinary metadata
even when every value and certificate byte is identical across two runs. Both
directions of that provenance-substitution test reject, as do cross-source runs.

Boundary symbols retain their original spelling/type but map to core-safe names;
they are not registered as arbitrary core globals. The producer's existing frozen
symbol-name computation is now shared through BoundaryValueDefinition. Its bytes
remain unchanged: the W07 run goldens and all existing VIR literal pins pass.
Duplicate/unsorted symbolic definitions reject; the literal body encoder validates
values against the exact foundation/root/closed-instance set. The output contains
all run values, without replacing decimal cohorts or dropping inactive source
storage. Complete regeneration guards metadata, symbol/type and byte changes.

Two test setup issues were corrected without weakening the model: boxed source
fields must be mutated through their value, and frozen i64 JSON inputs use quoted
canonical strings. The numeric/unquoted form, overflow and quoted negative zero
are retained as explicit rejected-input cases. A first compile attempt also
referenced the boundary module's private digest helper; centralizing the existing
canonical name computation resolved the scope error without changing its output.

All three actual sources, nine run variants and 20,258 ordinary bit observations
passed. Tests retain both the source-returned decimal cohort and its different
reparsed storage, and observe the wide product's final field. Large inactive
storage is sampled, not proved universally. The supplied return values remain
pending native results: neither successful parsing nor constant typing proves
that an application or external adapter produced them.

Both unchanged checkers accepted all nine pins with zero axioms and rejected
hash corruptions. Exact replay, W07 and ordinary-literal consumer tests, scoped
lint, format and five inventory tests passed. No remaining actionable issue was
identified in this bounded component review. It is not a final unit-4/W09 review;
binding/codec/native propositions, application proofs and assembly remain open.
