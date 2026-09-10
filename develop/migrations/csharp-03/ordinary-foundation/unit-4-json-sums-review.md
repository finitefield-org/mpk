# W09 unit 4 tagged JSON values: implementation review in progress

The emitter resolves only the frozen option, lookup, result, validation and
boundary_field closed templates. It checks the exact arm order, tags and payload
cardinality against the validated carrier. Payload parsers resolve recursively
through the same product/sequence compiler. Unsupported payloads defer their
parent instead of introducing opaque definitions.

Each branch parses the exact tag field, canonical quoted arm name, mandatory or
absent payload, and closing delimiter. The existing syntax and typed child
headers retain absolute cursor, ending, cumulative cells and invalid masking.
A tag contributes no semantic cell, but is a JSON child: even a no-payload arm
requires a parent nesting depth below 32. Payload counts include the entire
child. Validation.invalid therefore includes the sequence container cell and
separately requires a full 32-bit length in 1..=256. It does not subtract the
sequence cell or use its generic 4096-slot bound as the validation role bound.

Constructors retain the original active arm and zero inactive/padded storage.
Alternative branches have disjoint exact tag strings. Their parser packets are
bound before selection. Shared tag literals are deduplicated; primitive and
standalone syntax definitions are retained with complete dependency closures.
No checker, Certificate v0 rule, axiom or proof discharge is added.

The first Option core test exposed an error in its expected packet addresses:
the packet assembler appends unused high address selectors; it does not insert
leading padding before a C6 value. The test now uses the original packet mapping
`1 | (value_bit << 1)`. Production code and candidate bytes were unchanged by
this correction. The first mutation replay also incorrectly expected only one
Option instance; the original source reaches both Option<i32> and Option<string>.
The corrected assertion preserves both required instances.

Only the affected original document vector was replaced. The full earlier
manifest and that vector are retained in `json-products/previous-sum-parser`.
The other six metadata rows are exact. Previous batch acceptance is bound to
the archived bytes; the changed vector's checker result is tracked separately.

The source/coverage follow-up is in `unit-4-json-sum-roles-review.md` and its
progress receipt. It adds all five frozen template roles, a supported nested
Result<Option<int>,bool>, full narrow packet cases, original Validation capacity
documents and Option<string> observations. Direct Option<Option<int>> remains
forbidden by the unchanged original-input validator; the rejected draft is
retained. Pending runtime/checker jobs are not acceptance receipts.

The initial Option<i32> test passed all 17 complete C8 packets (110.61s).
The changed original document certificate passed both unchanged checkers,
matching reports, zero axioms and hash mutation rejection (354.616s).
Metadata mutations, exact Money/entry and sequence preservation, consumer
fingerprints and final affected lint/format also pass. W09, unit4 and the
remaining units are incomplete. There is no component-only commit or push;
the whole T gate remains at T06-W12.
