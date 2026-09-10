# Source-enum JSON component review

The existing boundary encoder represents a source enum as a JSON string
containing its canonical underlying integer, not its declared member name.
The importer constructs that enum value and validates membership against the
captured source's declared values. This ordinary implementation preserves that
representation and domain; it does not widen enum values to arbitrary integers.

Each source enum selects the existing quoted i64 or u64 token parser from its
validated underlying signedness. The token parser owns canonical spelling,
quoted framing, overflow, absolute start/end and the outer delimiter. The new
converter compares all 64 payload bits against the actual declared integers.
Signed constants are compared in their 64-bit two's-complement form, including
sign extension for smaller signed enums. Only after membership succeeds is
the value narrowed to the original carrier width. A value whose high bits
would alias a declared small value therefore cannot become that enum value.

Token validity and declared membership jointly gate the entire result. The
C8 role packet retains EOF and every cursor bit, records one logical cell,
preserves the full 8/16/32/64-bit enum carrier, and zeros all other storage.
Invalid results zero every bit. Product composition treats enums as scalar
children, with depth eligibility enforced by the parent as for other scalars.
The packet projection definitions reuse the existing ordinary projections.

The first converter test incorrectly placed a mathematical value greater than
u64::MAX into a 64-bit test input, discarding the overflow bit before execution.
Those remaining bits equal a declared value, so that assertion could not prove
overflow rejection. The corrected 52 converter cases use representable token
inputs; actual-core parser cases separately supply out-of-range JSON text,
including both signed64 limits and u64::MAX plus one. This was a test-domain
error, not evidence that the quoted parser accepts textual overflow.

One original C# fixture contains all eight admitted enum underlying types and
a product holding each. Its frozen offline double capture accepted all nine
source types. Replay checks declared values against captured facts, carrier
identity against independent reconstruction, all underlying routes, metadata
mutations, and complete previous primitive/syntax declaration closures.
The complete new certificate has 146,028 terms and 2,102 declarations. Actual
costs and its exact hash are retained in the progress receipt.

Existing 74 source contexts retain all nine complete product certificates and
metadata byte for byte. Empty enum metadata is omitted, and the emitter creates
no definitions when no source enum exists. Existing checker/semantic evidence
for those unchanged bytes is therefore retained without repeating the old
runtime matrices or dispatching duplicate checker work. The extracted shared
request producer also preserves the previous exact request bytes.

Direct review checked signedness, comparison before narrowing, complete output
padding, token/child argument order, generated-name uniqueness, source linkage,
explicit deferred carrier accounting, import regeneration and cumulative builder
limits. Converter, original-source, preservation, inventory, lint and format
checks passed. All 115 actual-core enum/product packet cases passed (terminal10883, exit0,
669.90s). The new vector passed both unchanged checkers, matching reports,
zero-axiom checks and hash-corruption rejection (terminal57943, exit0,
279.843s). Retained terminal logs are linked in the progress receipt.

Builtin enum/error vocabulary, collections/sums, absence/null rules and complete
boundary/application propositions and proofs remain outstanding. This is one
component of unit 4, not a completion review for unit 4 or W09. No component-only
commit is made. The full T gate remains at T06-W12.
