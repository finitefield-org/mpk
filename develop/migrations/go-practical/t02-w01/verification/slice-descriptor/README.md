# T02-W01 complete slice descriptor and full-width header guards

The [ordinary component](../../../../../../crates/mpk-vc/src/go_practical_ordinary/slice_descriptor.rs)
implements the exact frozen balanced product of six fields: Bool nil tag,
unbounded Nat owner, 16 path cells of full Word8 kind/full Word64 index, and
full Word64 offset, length and capacity. No field or high bit is dropped.
Its 2,290 declarations/69,289 terms/896,669 bytes preserve all 2,142 parent
identities and the entire 865,248-byte string-slice producer. There are 35
new definitions and 113 ordinary laws, with zero axioms and no core rule,
function extensionality, caller root, import or resource-limit change.

Construction and projections cover all six fields and all 16 arbitrary path
cells. Raw bit observations retain all eight kind bits and all 64 index bits.
Copy identity and reconstruction preserve the actual owner/path association;
field equality is reflexive, symmetric and transitive. Updating metadata with
WithBounds preserves nil, owner and the complete path, while retaining all bits
of the new bounds. These persistent value laws supply no Go ownership or store
proof. No independent writable heap is introduced for a child view.

Unsigned and signed indexing, two-index reslicing and three-index reslicing
use actual full-width circuits. Their ordinary proofs connect the results to
the full Nat values and existing signed Int interpretation. Reslice limits use
capacity even when the old length is zero. Sign, ordering and upper-bound
premises remain separate actual conjuncts. Pure AbsoluteIndex adds the complete
offset/index values in unbounded Nat; no machine wrap or physical-address
conversion is assumed. General step equations bind path projections, numeric
fields and address evaluation for arbitrary inputs, including nonzero values.

HeaderValid proves real length/capacity bounds and canonical complete metadata
when nil is true. It does not establish a live typed owner/view, path validity,
permissions, freshness, lifetime or source correspondence. Nil has no safe
index. EmptyNonNilRaw preserves the non-nil tag and zero length/capacity as a raw
header; deriving the actual NoAccess live witness remains required. Positive
capacity with zero length keeps the complete backing window available to a
permitted reslice. No new entry fit, separation or source-access premise is
silently introduced by these header facts.

The [verification receipt](verification.json) records five affected Rust roots,
strict target Clippy and six-file format on identical current source bytes.
Both [independent checkers](paired/result.json) inspect 186 full same-byte
inputs: two zero-axiom acceptances and 184 genuine type mismatches, comprising
113 altered conclusions, 43 real typed witness domains and 28 type-correct
wrong runtime definitions. Every wrong definition has an actual result
countermodel. Exact ownership also rejects a core-accepted extra definition
and the unchanged byte cap. No unrelated suite or intermediate full gate runs.

Pinned original Go 1.25.0 [observations](native-go/result.json) cover 400 whole
256-byte source inputs, 2,400 actual slice bounds results and 16 nil/non-nil
descriptor cases. They include high bits, negative signed values, zero-length
positive-capacity windows, capacity boundaries and endpoint order. Ten shared
view observations retain child-to-parent writes, parent replacement through an
existing child, shared capacity tails and distinct nil/non-nil zero reslices.
The observer's recovery/collection instrumentation is not admitted application
code or an ordinary source proof. The unchanged full SDK authority and its
previously committed complete reversible archive retain all 14,496 files.

The unchanged finite interpreter evaluates actual ordinary definitions for
1,264 complete descriptor outputs and all 2,400 native guards, totaling
60,486,975 AST steps across fresh cases. Raw/copy/reconstruct and changed-bound
outputs preserve every field and all 16 path cells. Sixteen additional raw
models cover all 256 kind bytes and full-width path/offset values; these
unvalidated typed paths are not admitted-source counterexamples. Complete
original buffers and all relevant 64-bit arrays precede evaluation. No
definition callback, theorem execution or 50M-step/2M-value/2M-environment
ceiling change supplies the observations.

Five complete source epochs/2,073 preimages retain 11 executions, including
three original conversion-fuel failures. Explicit field equations repair nil
numeric projection; capacity congruence repairs reslice preservation; checked
equality rewriting repairs header-validity copying. All three original full
certificates are [independently rejected](original-failures/result.json) by Go.
The earlier [capture failure](epochs/frozen-layout/capture-failure.json) retains
its original driver and five complete changed Rust preimages: a wrong parent
fixture path stopped before Cargo or candidate emission. Missing original
error files are not reconstructed. Resource failures are not source semantic
counterexamples or current type-mismatch controls.

The [input review](input-review.json) checks exact source preimages, all actual
outputs, complete decoded byte payloads, unchanged 255 frozen/186 frontend
inputs, local links and the existing 127/four/39/51 matcher/scope/seal/exclusion
counts. No source, JSON, directory, seal or blanket exception is added.
Go and Rust remain the only source languages. Original source/type/evaluation/
nominal binding, source-to-carrier preservation, live typed owners/views,
aliasing, permissions, allocation/lifetime/entry/escape/state/program proofs,
the full 56-family foundation/private descriptor/registry/context, native
Linux/general namespace/selected closure/all 169/T02–T11 remain due.
R01/R06/W01 stay open; the cumulative T02 gate remains W04:
`./scripts/check-fast.sh`.
