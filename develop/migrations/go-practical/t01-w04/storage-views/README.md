# Canonical storage owners and typed views

Status: W04 private storage/counter design and ordinary coupling feasibility.
Entry: `9e7e896a6e9be01184c6b173b489fec2f20c8a4b`.
The [receipt](observations.json) retains 15 native/model observations, ten
ordinary candidates checked by unchanged production Rust/Go checkers, and two
legacy certificates. Five ordinary claims are accepted with zero axioms;
five altered candidates fail CORE_TYPE_MISMATCH. Rejection is not a certified
counterexample. Complete source normalization, lifetime and counter proofs
remain open. No production source, installed profile or Certificate v0 changes.

## 1. One store, many typed projections

An independent writable heap for each slice element type is insufficient.
For rows := make([][4]int,2), rows[0][:] has type []int and aliases the first
[][4]int element. Writing the child changes the parent; replacing the parent
element changes an already captured child view. A struct cell's array field
behaves similarly. These relationships remain required for full v1 Plain
aggregates; nested array views are not removed to simplify the implementation.

The private [model](../../../../probes/go-practical/storage_views.py) uses:

```text
Owner = (identity, exact layout, current complete value, provenance,
         access permission, lifetime scope)
Location = (owner identity, typed field/index path)
Slice = (exact element type, nil tag, array location, offset, length, capacity)
```

A positive-capacity descriptor's location selects an addressable array with
that exact Plain element type. Its whole capacity window lies in the selected
array. Reads and writes project the same canonical owner value. There is no
second cached writable copy for a child type. Whole array/struct replacement
retains the source location identity and updates every existing view through
that path. Field updates reconstruct from the current value and preserve
siblings. Path traversal cannot cross a slice descriptor: dereferencing it
must pivot to its captured owner/path before appending the element/field path.
Indices retain their full mathematical value until bounds are established.

Value copying creates fresh local value storage but shallow-copies descriptor
references. Equal contents do not merge objects. A borrowed owner forbids
positive writes through every descendant view. A new view grants no ownership.
Freshness and scope must come from checked allocation/value-copy/call binding,
not user annotations. Canonical names are model object identities; no native
pointer equality, byte layout, unsafe address inspection or GC claim is made.

## 2. Accessible storage, source scopes and promotion

Local value roots retain their full declared array/struct state in a separate
value arena. They are not silently truncated to the slice capacity bound and
do not consume a new heap slot for each view. The local arena still needs its
own exact structural/weighted-state limits and source/frame proof. This probe
does not freeze those values or impose a new practical array-type maximum.

When its own scope ends, an escaping local view must retain its storage;
returned array values are separate by-value copies. Promotion is a model
reference translation, not a new Go allocation. All escaping descriptors in
the same return/product/aggregate must translate together. Caller-owned roots
whose scopes remain live retain their original references; a callee cannot
snapshot/copy them at return and break later caller writes. Borrowed local
storage cannot be promoted into fresh ownership.

The prototype accepts an explicit complete escaping descriptor list. It
groups actual array paths by observable containment: an ancestor descriptor's
capacity must cover the containing cell, including positive-capacity empty
views. A child inside that window stays on the parent's canonical owner.
A child outside every such window may become its own root after the original
local scope ends; no admitted operation can regain an unexposed ancestor.
General source/lifetime and exhaustive escaping-reference closure proofs are
still required; the list is not a trusted callee summary.

For each promoted root, take the union of accessible root-array cell intervals
and map its ordered cells consecutively. Remove inaccessible gaps, retaining
each descriptor's length/capacity and every overlap. For every descriptor d,
the required window isometry is:

```text
for 0 <= k < cap(d): packed(offset(d)+k) = packed(offset(d)) + k
```

Child paths use the same translated parent-cell index. This does not compress
observable cells, invent alias separation or allow reslicing across a gap.
The native/model distant-window example returns two capped windows from a
256-element local array. Four accessible cells are represented without
pretending the source allocation itself had only four cells or adding a hidden
whole-allocation <=128 input condition. Exact general entry embedding and
padding/byte-bound/source-frame proofs remain due.

Non-nil capacity-zero descriptors use a typed NoAccess marker; nil remains
distinct. The marker observes no source owner/offset and grants no positive
read/write/ownership fact. It is a private abstraction of the existing ZS03
live-witness rule, not a new serialized null region. Production lowering must
derive the compatible live view/witness and prove validity. Positive-capacity
empty descriptors retain their complete backing and tail. Zero-count copy
has no write; positive growth beyond capacity is a profile-domain obligation,
not a fabricated Go panic.

## 3. Counter interpretation and required enforcement

The measured candidates remain two slots and 128 root-storage elements. A slot
is a positive canonical heap owner of an exact root storage element type;
typed descendant projections do not each consume another slot. One root with
128 array-valued cells can have 128 differently indexed child views. Each root
cell retains its full captured Plain layout; this is not 128 scalar bits or a
claim that aggregate payload costs are free. Slot/cardinality and cell/layout
costs are separate counters. Zero-access witnesses consume no positive owner.

The final source-domain accounting must inspect the complete observable state:
all live roots, aliases, argument/result temporaries, pending return products,
callee footprints and entry/old snapshots. Descriptor length zero does not
remove positive capacity. Root fields, value-array copies and closed error
payloads retain their referenced owners. Permissions follow owners across
paths, calls and scope changes. Caller scopes and source per-iteration
bindings need exact activation/lifetime identities.

Retirement/reuse is allowed only after proof that no required positive-capacity
reference or pinned entry observation can reach that storage. The current
prototype conservatively retains every heap record and implements no general
liveness/retirement analysis. Its allocation counter is therefore not a
normative cumulative-allocation limit. Complete live-owner accounting, local
arena weights, canonical role/slot assignment and bound enforcement are still
freeze obligations. No unsupported workload is declared supported by these
candidate numbers. A source may allocate/grow successfully in Go while its
profile-domain or proof-resource obligation fails.

This proposal needs a successor structured storage-reference union and typed
path validation; it cannot widen an existing integer region field in place.
Exact wire/hash identities, alias-preserving old/new migration dispositions
and immutable profile boundaries remain W04 work. The four installed profiles
and inactive C# identities retain their previous meanings.

## 4. Checked ordinary coupling laws

The [untrusted producer](../../../../probes/go-practical/view-certificates/src/views.rs)
quantifies over an arbitrary A:Sort0, a store of two rows of 64 A cells, selected
row/index flags, arbitrary replacement values and rows. It proves:

- Child write is observed by the parent at that exact cell.
- Replacing a complete row is observed by an already selected child read.
- Replacing a row preserves every cell of the other row.
- Replacing a complete owner is observed through its existing child path.
- Updating one of two distinct owners preserves reads of the other owner.

These are pointwise typed equalities using only existing Bool cases and Eq
congruence/transitivity. They add no function extensionality, eta, semantic
axiom or new trusted computation. The largest construction splits seven
address flags (128 addresses), not stored values or all data-bit patterns;
other families split one/no address flag. Full frame/source relations must
compose these observations with exact layout/path validity, permissions and
control, rather than assuming an unproved equality of store functions.

Both production checkers accept all five positive claims with matching
export/certificate/axiom hashes and zero axioms. The five altered claims try
stale-parent/child or wrong-owner observations and fail type checking. Legacy
Bool/Nat certificates remain accepted. The receipt pins 123 inputs; observed
maxima are 51,647 canonical bytes, 5,823 stored terms and 23 declarations. The
private 60-second child ceiling is a measurement setting, not a new normative
limit or whole-application cost guarantee.

## 5. Validation, ownership and remaining work

The native [observer](../../../../probes/go-practical/view_reference.go) and
independently specified expected data cover hierarchical updates, sibling
fields, value-copy/descriptor-copy distinctions, overlap snapshots, returned
array copies versus aliases, zero tags, distant windows and inherited caller
scopes. Snapshot aggregation is observer instrumentation outside verified
roots. Fifteen observations match the private model. Twenty targeted tests
also check 128 child views on one owner, permissions, full indices, no-access
behavior, packing overlap and failure before promotion state changes.

Primary slice/construction owners retain their responsibilities. Planned
practical_storage.go and go_practical_storage.rs add shared Go lowering and
independent Rust type/path/storage validation, with dedicated tests. Ordinary
foundation owns complete lens/packing/lifetime/normalization laws; T06 owns
source/alias/frame and complete application binding. The new files are planned
executors, not completed production code.

The [verification record](verification/verification.json) separates native,
model and ordinary certificate evidence from unproved source normalization.
General compositional paths, entry/escape closure, live-state/retirement,
all limits, exact schemas and migration review remain due. W04 stays In
progress, W05 owns inactive publication and the cumulative T01 gate, and the
full T01–T11 implementation/release objective remains active.
