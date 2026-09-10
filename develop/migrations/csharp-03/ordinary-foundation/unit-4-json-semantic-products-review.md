# Semantic JSON products review

Closed ordered_entry and Money have exact key/value and amount/currency fields
in the existing value decoder/encoder. Their carrier validation counts one
product cell plus their typed children. Money value validity does not itself
invoke the currency predicate parameter of the separate Money.create operation;
that operation and application contracts retain their own obligations.

The compiler selects source members from reconstructed source declarations,
or these two semantic schemas from the actual closed-instance template ID.
It checks carrier product shape, field IDs and order, then resolves each field
name in the source_members or semantic_field_names syntax group owned by that
exact carrier. Children retain their concrete type IDs. Metadata records the
semantic template ID; None is omitted for source products. Existing nested,
empty and eight-underlying source-product metadata and certificates remain
byte-identical after this selection change.

The parser body uses the existing sequential Let bindings, exact punctuation,
recursive depth argument, complete typed child packets, checked cumulative
cell count, original MakeStorage and final zero-invalid packet assembly.
Semantic/source schemas with equivalent carrier shape can share helper bodies,
but retain distinct type IDs and field spelling. In particular, source Amount
and Currency fields are distinct from semantic amount and currency fields.
No existing definition is omitted, and no transformer counter is reset.

One captured C# source and three original sidecars produce two Money instances
(enum currency and string currency), one ordered entry and four source products.
Replay verifies captured bindings/source, independent carrier reconstruction,
actual closed template fields, full previous primitive/syntax dependency
closures and template/member import mutations. The complete certificate has
146,723 terms and 2,263 declarations. Both unchanged checkers accepted those
same bytes with matching reports, zero axioms and hash-corruption rejection
(terminal15486, exit0,323.611s). A test-only private-API reference was corrected
to the existing public emitted closure API before source capture/replay.

Runtime cases check source/semantic spelling, order/missing/wrong child values,
normalized decimal representation, undeclared enum rejection, absolute offsets,
outer delimiters and depth boundaries. Eleven small packets are checked in full.
The five wide string-Money/envelope cases check complete headers and selected
addresses: every expected occupied leaf, surrounding/toggled selector bits,
every physical decimal leaf in string Money, string length and paired UTF-16
surrogates, last string-cell addresses, high padding and unused envelope roles.
These probes do not claim exhaustive enumeration of the wide cubes. All 16 cases passed in 669.67s. The retained complete cargo log reports one
test passed and zero failures; the consumed process handle is now missing.

Direct review checked the source/closed schema distinction, field linkage,
unchanged source-body construction, metadata omission, recursive child dispatch
and actual cumulative costs. Source/preservation/inventory/lint/format and
same-byte checker verification passed. Existing scalar/token runtime matrices
and unaffected checker vectors are not repeated. Remaining composite rules are
recorded in unit-4-json-composite-followup.md. No component-only commit is made;
units3-8 and W09 acceptance remain open, with the whole T gate at T06-W12.

A follow-up field-name review found no reachable overlong source-member path:
PracticalCapture.cs::ValidateIdentifier permits only ASCII identifiers of at
most 512 characters. Frozen original double capture rejects both 16,384- and
16,385-character member names with CSHARP_PRACTICAL_DECLARATION/source_identifier
before typed emission. The exact probe requests/responses are retained under
verification-logs/json-container-join. This does not establish a bound for
arbitrary boundary-sidecar json_name values; the future envelope parser must
apply its own UTF-16 field-name limit. Semantic Money/ordered-entry keys are
fixed short literals. No source parser change was warranted by this concern.
