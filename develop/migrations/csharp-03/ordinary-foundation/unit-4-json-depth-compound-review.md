# W09 compound input depth integration — review in progress

The new test imports original captured sources from six existing JSON source
families. It validates input with the independent input capture, checks the
entire C7 header, and checks the parsed argument's typed-depth predicate against
the captured canonical JSON tree at its exact depth boundary. This deliberately
does not claim complete wide argument-storage coverage or application proofs.

The first source-array run generated the combined program successfully but
stopped because the small field-test cell oracle lacked Sequence. The oracle
now covers arrays/sequences, map/set/entry, Money, tagged sums and transition
values. It counts semantic cells: implicit map entry objects and transition
event-array containers do not add cells. The corrected array run passed with
142,518terms,2,174declarations and8,913transformers. An earlier estimate adding
a separate8,192-transformer fold was incorrect: the owning Builder shares
existing helpers. No production fold optimization or limit change was made.

Nested and empty source products passed before a Money fixture was rejected
by the authoritative capture. These boundaries expose a source Payload whose
nested members retain their source types; they therefore require `Amount`,
`Currency`, `Items`, `State`, etc., rather than semantic role names. The corrected
eight-context run preserves that distinction. Direct semantic-root boundary
coverage remains a separate required extension; source wrappers are not evidence
that those projected roots have been exercised.

The two successful product candidates and array candidate are retained. Their
source/generator inputs are unchanged and successful runtime cases need not be
repeated merely because a later fixture failed. All eight corrected cases passed in1226.90s. Together with those three retained
cases, all11 candidate metadata/byte pairs were reconciled, hash-verified and
published. Same-byte checking and additional deep/direct-semantic sources
remain pending. The full W09 objective, including node limits, source relations,
output, native/control/replay and proofs/assembly/acceptance, remains open.

All eleven current source candidates subsequently passed both unchanged checkers
with zero axioms and hash-corruption rejection (7,598.946 seconds). The exact
PASS set and current module hashes were reconciled. This closes that pending
checker run; remaining W09 work and separate semantic coverage are unchanged.
