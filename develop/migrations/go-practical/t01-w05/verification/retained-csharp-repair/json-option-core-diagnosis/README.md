# Original Option core fixture-selection failures

The unchanged source320 integration binary reproduces both reported failures:
[Option i32](result.json) fails in 0.124 seconds at the original row lookup,
and [Option string](string-result.json) fails in 0.126 seconds at the same old
document ID. Both exit 101 before reading a definition certificate or evaluating
a JSON packet. The original source copies and complete raw stdout/stderr remain
unchanged. These observations identify test-input selection failures, not parser
semantic or checker rejections.

[Current fixture inventory](current-fixture-inventory.json) records all seven
current metadata rows. The old `document-7a157feb...` row is absent; its exact
current source partner is `document-41dc96...`. The existing retained source
guard verifies the complete original/current source, contracts, roots, native
facts and raw input digests for all three document pairs, retaining its narrowly
defined context-field differences and UTF16 preservation. The repaired cases
must pass that existing guard before reading the current row. The i32 case
selects exactly one Option with an i32 payload rather than relying on the order
of the i32 and string Option definitions.

[Git input verification](input-git-verification.json) checks ten original source,
guard, metadata, certificate and request/response members against source320.
The i32 case matrix and full packet assertion suffix remain byte-identical:
17 cases and all 256 bits per packet. The string case retains all four cases,
typed UTF16 cell counts and every original bounded wide-packet probe; its check
helper and other runtime tests are unchanged. This does not broaden the original
wide-packet observations into a complete or universal proof.

Three [diagnostic preparation failures](initial-preparation-failures.json) are
preserved separately. Initial inspection repeated the obsolete-ID assumption
and assumed a `sums` field in empty programs; the valid format omits empty
collections. Those attempts did not execute the original test or modify inputs.
The actual original test failures above were then obtained without preselecting
any row in the wrapper.

The [repair verification](../json-option-core-repair/verification.json) records
actual repaired execution, the existing source-correspondence controls, and
the affected current source producer separately. The original source320 whole
diagnostic continues unchanged. No current fixture, production implementation,
checker rule, resource limit, application-proof status or installed profile is
changed by the fixture-selection repair. A complete repaired gate remains
required before T01 completion or T02.
