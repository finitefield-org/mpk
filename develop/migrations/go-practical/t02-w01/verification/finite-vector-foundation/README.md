# Ordinary finite-vector sequence bridge

Status: scoped component checks passed; R01/W01 remain in progress. Entry is
`b8fdd9973575288d5b974d9cd755d52b8002b101`. Source languages stay Go/Rust only.

The [producer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/finite_vector.rs)
adds91 ordinary definitions and139 universal laws to the actual unchanged183-member
sequence prerequisite. The complete413-declaration certificate has6834 terms,
80953 bytes, no imports and zero axioms. Both unchanged independent checkers
accept the same full bytes with matching export, axiom-report and certificate
hashes; see the [complete paired record](paired/result.json).

For depth0 through8 the data type has exactly d Boolean address arguments and
an arbitrary Sort0 element:1,2,4,8,16,32,64,128 or256 complete slots. The first
argument is the most significant address bit. Capacity starts at1 and doubles.
Index maps those address bits to a Nat; Lookup compares with the actual lower
half capacity and subtracts it for the upper half. LookupIndex proves that
reading every Boolean address returns exactly that original slot. There is no
enumeration of element values, word values or source programs.

All quantifies the actual predicate over every slot. LookupValid proves the
predicate for every Nat index below capacity, using both child predicates and
the actual unbounded right-index bound proved in the sequence prerequisite.
Embed retains an explicit Nat length and that lookup function. Its length,
arbitrary read and slot read equations, guarded IndexValid and predicate
preservation are checked ordinary terms. EmbedValid consumes both All and the
quantified Fits witness: every index below the supplied length must be below
the actual capacity. FitsFull and FitsEmpty are separately proved.

This preserves the finite data shape needed for positive arrays and the full
256-slot string data field; the arbitrary element type includes all eight bits
of each byte. It does not bind the full64-bit string length or source index.
No high word bits are truncated. The frozen empty-array Bool dummy is separate
from the one-slot depth0 vector. Actual nominal lengths, padding validity,
canonical empty values, source evaluation/domain/permissions/invocation/alias/
storage/state/program correspondence still require their original ordinary
proofs. No profile, private context or source/program claim activates here.

Direct definition expansion originally exhausted the fixed1024 fuel cap at
depth5 LookupIndex. A fully retained symbolic-spacing attempt also failed, at
depth4. The final component keeps the original fixed capacities, proves one-level
branch/bound laws with arbitrary Nat sizes, and composes checked definition
equations with congruence/rewrite. Function/product definition equalities are
reflexivity, not function extensionality. No core rule, axiom, source carrier or
resource cap changes. The initial formatting, two compiler ownership failures,
original resource failures, full candidate bytes and actual outputs remain in
[the epoch records](verification.json). Earlier Rust-only outcomes are not
relabeled as final independent Go results. Driver versions reconstructed from
the exact retained predecessor/session edits match each original execution
receipt hash; [provenance is explicit](driver-preservation.json).

[Three affected Rust tests](epochs/complete/selected/result.json),
[strict Clippy](epochs/complete/lint/result.json) and
[changed-file format](epochs/complete/format/result.json) pass. The independent
Go run checks all202 original complete inputs: two zero-axiom acceptances and200
genuine type mismatches. Those include all139 altered conclusions,50 erased
validity/observation/range premises, and11 type-correct changed operations.
Boolean range controls reverse the actual truth value and keep the whole
computed left side. Wrong operations drop an address bit, shift the one-slot
index, ignore an upper half, erase the embedded length or halve capacity. Each
modified definition prefix typechecks before the full original law stream
rejects. No partial certificate acceptance is claimed. An honest extra valid
definition passes both cores and fails exact owning import; trailing and
oversized inputs fail the separate owning boundary.

The [source/proof review](manual-review.json) records the chosen verification
scope. All prior sequence declaration types/bodies/proofs/metadata remain exact.
Unchanged frontend, source capture, both core implementations and unrelated
suites are retained without repeat. All255 frozen design inputs and185 current
Go frontend build inputs remain required. Strict127-rule/four-scope/39-seal
review gains no new exception. All original whole buffers remain required.
R01/R06/W01/full56-all6-19 ordinary foundation/private context/native kernel/
namespace/selected closure/all169 demands/T02-T11 remain open. The cumulative
T02 gate stays W04: `./scripts/check-fast.sh`.
