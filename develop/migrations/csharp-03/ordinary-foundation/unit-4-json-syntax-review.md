# Closed JSON syntax component review

Scope: W09 unit 4 field-name and punctuation matching. This component is not a
typed compound JSON parser, boundary relation, application proof, or completed
internal work unit. Units 3–8 and W09 acceptance remain open.

The generated ordinary functions accept a C24 document and a C5 absolute cursor.
The C6 result contains a match flag, an at-document-end flag, a 32-bit absolute
end, and zero padding. Invalid results are entirely zero. The end flag alone
does not establish validity of a JSON value. Matching performs no whitespace
skipping or alternate spelling normalization.

Review checked all 24 document selector positions in Capture32, the 32-byte
chunk boundaries, active byte comparisons, balanced conjunction, and the final
original-document bounds. The final bounds require length at most 1,048,576,
start no greater than length, and literal length no greater than the remaining
length. Thus wrapped intermediate chunk offsets cannot yield acceptance.
Inactive document bytes cannot satisfy a truncated match. The complete packet
runtime test observes all 64 bits, including invalid and padding bits.

Immutable sidecar UTF-16 names are encoded canonically and followed by a colon.
Source members retain stored-member order within each source type. Semantic
names describe a vocabulary; payload presence and active sum arms must be
enforced by the subsequent typed grammar. Equal full byte strings share a
matcher; chunk reuse also compares full bytes. Definition hashes do not replace
byte comparisons. Import regenerates the program and compares exact metadata
and certificate bytes against the reconstructed VIR.

The source test indexed the first field of an
empty contract; it now mutates an always-present literal and additionally
mutates field metadata when fields exist. The initial source test only checked
names returned by the generator; it now independently compares sidecar names,
owners and ordering, and source members from the original captured facts, so
omissions or reordered members fail. Public packet documentation now explicitly
distinguishes the end flag from complete JSON validity.

Consumer verification found a missing inventory update for the new standard
logic reference. An independent reconstruction produced the exact prior
119-path fingerprint after removing only the new syntax module. The inventory
now records all 120 paths, with its linked inventory hash refreshed; no path
was excluded or search assertion weakened. The addition/deletion mutation test
passed after this correction. The closure test then exposed the separately
pinned family/path total; updating 4,942 to 4,943 and rerunning only that failed
test passed. Both affected consumer tests are now verified.

The focused runtime test passed 111 complete packets, including lone surrogates,
multibyte Unicode, lengths around 32 and 64 bytes, a 257-byte name, last-position
matches, poisoned inactive bytes, declared length overflow, and changed bytes.
The source test covers 68 original contexts and three nonempty programs,
including a contract with no fields. Complete bytes are pinned with readback.
All three distinct pinned certificates passed both unchanged checkers, matching
reports, zero-axiom checks and hash-corruption rejection. Current source linkage,
checker, lint and inventory results are recorded in
`unit-4-json-syntax-progress.json`; unfinished checks must not be inferred from
this review. All selected component checks have now passed, and direct review
found no further issue in this component. This is not a clean review of the
unfinished W09 task. The runtime test's ordinary definitions are unchanged by the later
comment and equivalent punctuation-array lint fix.

Next integration must consume these exact match definitions in the typed
object/array grammar, enforce required/optional/null fields and cumulative
limits, and connect the resulting predicates to the actual boundary VCs. No
field vocabulary or successful checker invocation discharges those obligations.
The repository-wide gate remains deferred to T06-W12. No component-only commit
or push is made.
