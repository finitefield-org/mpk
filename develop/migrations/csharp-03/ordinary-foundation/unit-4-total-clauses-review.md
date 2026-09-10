# Total observation clause component: direct review

Reviewed the three new dispatch arms,nominal signatures,template/arm validation,
relation-cache ownership and both downstream consumers. `sequence_length` only
accepts a validated bounded-sequence instance and preserves the existing full
Length definition. Its i32 interpretation is valid for domain-admitted lengths;
the ordinary function does not silently truncate invalid high-bit headers.

`tagged_is` accepts the five supported semantic sum families,then resolves an
exact frozen arm name in the existing structural operations. The existing
predicate compares all32 tag bits and ignores payload storage. ParseError kind
checks its nominal source/result and exact32-bit carrier before using identity.
No new definedness assumption or inactive-payload normal value is introduced.

The direct tests exercise unknown/sparse high-bit tags,payload independence,
all single-bit length/kind words and source expression hashes/truth values.
Same-VIR dependency closure comparison covers source clauses,standalone public
predicates and every original structural definition;metadata and exact import
checks agree. Earlier structural/rich-literal pins are preserved. The initial
test-only hash API mismatch was corrected without a production hash change.
Both new candidates passed same-byte checking and actual hash-corruption
rejection;current hashes and PASS names were reconciled. No further component
finding was identified. Final affected lint,formatting and artifact-consumer closure passed;no inventory
fingerprint refresh was needed. Full internal-unit/W09 acceptance and application
proofs remain open.
