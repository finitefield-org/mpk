# T02-W01 compositional typed value views

The [ordinary implementation](../../../../../../crates/mpk-vc/src/go_practical_ordinary/storage_lens.rs)
represents a view by its actual typed read and update functions. Composition
reads the current parent before updating its child, then writes that result
through the parent view. It captures no old parent value and creates no child
heap. Identity, composition, pointwise association and disjoint observation
laws connect actual operations, recursive relations and validity predicates.
No equality between arbitrary functions or extensionality axiom is assumed.

Heterogeneous product fields preserve sibling observations. Array element
views cover every frozen depth 0 through 8, including the full 256-element
carrier. Reads, writes and disjoint-index frames retain the complete Word64
index and its Nat value. Read/update and validity laws consume their actual
shape/range witnesses; zero-length source arrays cannot supply a safe-access
witness. Restoration and overwrite use pointwise element relations, including
arrays of composite values, rather than whole-function equality. Recursive
reflexivity, symmetry and transitivity use real element-relation proofs.
Empty and nested paths inherit their seven read/update/relation/validity
laws, and disjoint observations lift through shared or different parents.

The 2,902-declaration/87,423-term/1,115,975-byte component retains all 2,688
prior array identities and the exact 1,033,685-byte producer, plus the existing
16 product-relation declarations. Its lens namespace adds 17 definitions and
181 laws, with zero axioms. No core, limit, caller-selected root, private
registry or public profile changes supply acceptance. The
[verification receipt](verification.json) records five affected Rust roots,
strict target Clippy and 11-file format on identical current source bytes.
Both [independent cores](paired/result.json) inspect 448 same-byte inputs:
two zero-axiom acceptances and 446 type mismatches, comprising 181 altered
conclusions, 249 selected actual typed premise domains and 16 type-correct wrong runtime
definitions. Each wrong definition also has an actual value countermodel;
Rust also type-checks its definition prefix before checking the damaged laws.

Pinned Go 1.25.0 [observations](native-go-current/result.json) retain 203 pairs
of complete 256-byte original/replacement buffers. They cover 22 array lengths,
nine nested-array splits and all path lengths 1 through 16. Native execution
records 1,015 complete root snapshots: child writes, replacement of the whole
parent through an existing shared view, subsequent shared writes and the
unchanged value copy. The unchanged ordinary interpreter matches 1,060 full
root outputs, including alternate association, and 406 alias reads using
67,909,080 aggregate AST steps. The per-observation ceilings stay unchanged.
No definition callback or theorem body supplies a result. These are finite
value observations; the Go pointer/slice collector is test instrumentation,
not admitted source or a live owner-store proof. The complete original SDK
and its existing reversible archive remain the toolchain authority.

Eleven complete source epochs/4,848 preimages retain 17 executions,
including four original Rust failures. Two construction failures precede
candidate emission; declaration ordering fixes the forward field reference.
A complete original type mismatch exposes variable capture in equality
transport, repaired with distinct bound variables. A complete original fuel
failure at depth four is repaired by explicit checked view equations, with
no limit increase. Both complete failed inputs are
[independently rejected](original-failures/result.json) in Go. Four formatter
runs intentionally mutate their saved preimages and are recorded separately
from checks on unchanged inputs. The first native observer omitted its JSON
replacement-tag binding; its original complete sources/cases and actual
stdout/stderr remain. Its first compiled binary was overwritten by the next
explicit build and is represented only by its original hash. That original
binary is unavailable and is not reconstructed. The
[preservation receipt](native-binary-preservation.json) records this limitation
and retains the exact current executable through reversible hex transport.

The [input review](input-review.json) checks every changed byte and complete
decoded data payload, source preimages, actual receipts, unchanged 255 frozen
inputs/186 frontend inputs and the existing 127/four/39/51 scanner counts.
No source, JSON, directory, seal or blanket exception is added. Go and Rust
remain the only source languages. Source/type/evaluation/nominal/element/
length binding, source-to-carrier preservation, actual typed path synthesis,
live shared owner stores, aliasing/permissions, NoAccess, allocation/lifetime,
entry/escape/state/joint-call/program proofs, the complete 56-family
foundation/private descriptor/registry/context, native Linux/general namespace/
selected closure/all 169/T02–T11 remain due. R01/R06/W01 stay open; the
cumulative T02 gate remains W04: `./scripts/check-fast.sh`.
