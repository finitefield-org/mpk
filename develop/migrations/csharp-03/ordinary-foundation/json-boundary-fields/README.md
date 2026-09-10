# Ordinary boundary field decoder candidates

Fifteen original boundary-attachment sources and the 29 accepted documents in
`boundary-input/conformance.json` supply the source and runtime evidence. Each
`.json` retains the generated field/program metadata and source snapshot; the
matching `.hex` is the complete ordinary Certificate v0 candidate. The combined
`certificates.json` is sorted by case name. Publication verified every module
certificate hash against both metadata locations and all 15 accepted source IDs.
The largest candidate has 142,747 terms and 2,174 declarations.

The initial source test completed six contexts (ten original documents) before
failing closed on a missing raw-instant codec adapter. After the adapter fix,
those six current certificates matched the previously executed bytes exactly;
their full core observations were retained without rerunning them. The focused
raw-instant test passed one document and ten packet cases. The remaining eight
contexts passed 18 documents and 54 complete packet/carrier cases. Source
generation compares every predecessor JSON declaration/dependency, and the
nullable-default context also exercises exact import and metadata/certificate
mutations. Header arithmetic has a separate 432-case complete-packet regression.

The unchanged Rust/Go checkers are now checking these 15 candidates, with
matching zero-axiom reports and hash-corrupted rejection required. Runtime or
typing of a field decoder does not establish whole-document admission or a
source/application theorem. See `../unit-4-json-boundary-fields-progress.json`
and `../unit-4-json-boundary-fields-review.md` for live/terminal evidence and
remaining envelope, canonical typed-value and source reconstruction work.

The source test normally checks the exact pinned bytes and per-case metadata.
Its explicit source selection supports affected-only validation. Its optional
prior-core reuse mode requires an explicit subset and exact candidate-byte
equality; output separately counts reused contexts, never reports them as new
runtime cases. This is test evidence reuse, not an application acceptance path.
No component-only commit/push or T-wide gate is performed here.
