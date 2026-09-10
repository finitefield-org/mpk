# Collection contract reads (partial W09)

This component connects `map_contains`, `map_lookup`, and `set_contains` to the
ordinary collection search/lookup definitions. The standalone collection
generator and the shared contract compiler use the same helper emitters.
Contracts retain exact nominal collection, key, value and lookup-result types;
the helpers do not establish input public domains or application VC proofs.

The nine request contexts preserve their original C# source and semantic binding
bytes and binding paths. New method contracts cover empty and populated inputs,
present and missing queries, decimal numeric equivalence, string/source-product
keys, nullable values, floating values, and two maps sharing one lookup type.
The contract decimal literals use the required normalized spelling. Direct
ordinary-call tests retain non-normalized decimal storage to check cohort
equivalence. Lookup results are compared at every output bit against the
independent existing collection model, including found nullable values.

The source test checks each actual contract alias and its complete transitive
definition closure against the standalone collection program, then checks exact
import and metadata corruption. These observations supplement ordinary
certificate checking; they are not theorem proofs. The targeted preservation
test confirmed all nine predecessor collection certificates remain byte-identical
after extracting the common search/lookup helpers.

`capture-receipt.json` records the fresh frontend inputs and isolation; all nine
contexts were accepted. `../unit-4-collection-clauses-progress.json` records the
current verification status. All nine source contexts, 18 recipes and 5,598
ordinary observations passed (12013.57s). Regeneration with the current code
preserved every certificate and metadata byte (16.39s), so the long runtime
result is reused. All nine same-byte Rust/Go checker cases also passed (97.231s), with zero
axioms, report agreement and hash corruption rejection. The final component
review and reconciled evidence are recorded in the linked progress file. Original units 3-8 and W09 remain open, and the full gate remains
deferred to T06-W12.
