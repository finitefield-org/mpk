# W09 ordered input objects — review in progress

The new ordinary generator composes existing field decoders into a complete
object parser yielding a semantic argument tuple and the full C7 header.
Contract order is reconstructed from immutable emitted boundaries. Each field
step tentatively matches a separator and its exact name; an absent optional
field retains the original cursor and adds its frozen default's semantic cells.
A matched field uses its selected codec/null/payload decoder and updates exactly
its argument slot. Missing required fields make the complete state invalid.
Unknown, duplicate and out-of-order fields cannot be consumed by a later step;
the final exact closing brace and EOF check reject remaining input.

The state carries header, whether an earlier field was supplied, and all semantic
arguments. Argument writes preserve other slots and clear the written slot's
padding. Fixed field indices include the high bits needed for256fields. A
separate step definition per field keeps binder depth independent of field
count. The completed state is bound once before extracting the output.

Direct review caught initially uncharged, linearly nested state calls. Field
steps now use `Builder::compose`: each concrete S->S occurrence is counted
against16,384, and balanced composition preserves left-to-right state flow.
The earlier targeted source job was explicitly stopped because its implementation
was superseded, not because of a timeout. Its partial output is not acceptance
evidence. The current source job uses the corrected counted composition.

Targeted helper evidence:144complete omitted-cell headers pass, including zero,
65,536 and overflowing counts and invalid reserved bits;7,680complete argument
storage bits pass, including256-field address endpoints and padding. The first
writer test passed a test-only Cube value where a Bool was required; correcting
the test argument representation fixed that evaluator panic. The initial source
test's empty-contract lookup used serde_json on surrogate-bearing sidecars; it
now uses the authoritative UTF-16-aware practical JSON parser. Integration lint
also required naming the generation return tuple, which is now `JsonGeneration`.

Completed source validation includes required fields, a leading omitted default
and an empty contract. It compares complete output packets against original
boundary-input captures, then checks whitespace, trailing data/comma, unknown
and duplicate fields, reordered fields and missing required fields where
applicable. All16source contexts passed:30original documents and32rejected
mutations, with every packet bit checked. Every source candidate preserves all
predecessor field decoder definitions/dependencies. Publication independently
verified all candidate hashes and original source identities. Same-byte dual
checking and hash mutations are running on those16pins.

Affected inventory review found exactly one added namespace consumer, the new
envelope implementation. Removing that path reproduces the previous121-path
fingerprint. Updating the consumer fingerprint and inventory correction receipt
restored the targeted addition/deletion test. Final implementation lint and the
existing required-field pinned-byte regression passed. The envelope test now
also checks pinned metadata and exact bytes when no output directory is supplied.
The new assertion path passed on the empty-input source, including its original
document and six invalid documents; affected integration lint also passed.

Inspection of the authoritative input capture identifies two distinct tree
checks that the admission layer must preserve. `preflight` and `value_limits`
check the supplied JSON at root depth0 with262,144raw nodes. After decoding,
`typed_json` inserts semantic sum tags/payloads and materializes defaults;
`value_limits` checks each such field at depth1. Thus an input parser's raw
nesting check cannot establish the canonical typed tree's depth bound: nullable
and presence wrappers can add a level even though no object wrapper was present
in the input. Semantic cell accounting at65,536 is a third, different measure.
The current header carries that semantic count, not a canonical typed-depth
witness. The next admission implementation must measure the active typed
representation (including defaults) and retain these distinctions; certificate
typing and the current scalar source corpus do not discharge that obligation.

This decoder does not yet define W07 AcceptInput. Canonical typed-value limits,
the independent raw-node-bound obligation, source public domains and actual
source reconstruction/invocation links must still be incorporated or proved
from the exact ordinary definitions. Compound/string/configured-codec field
integration coverage, output, native/control/replay, universal proofs, assembly,
mutation and predecessor acceptance remain outstanding. Units3-8 and W09 are
incomplete; no component-only commit/push or `check-fast.sh` is performed.

All16 original unguarded envelope pins now pass both unchanged checkers,zero
axioms and hash mutations in5344.339s. Their exact PASS filename set and current
pinned hashes reconcile. Separate typed-guard integration remains tracked in
unit-4-json-typed-guard-progress.json;this does not complete W09.
