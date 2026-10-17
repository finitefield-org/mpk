# Derived lifetime roots and reusable storage slots

Status: W04 private lifetime/accounting design, not a source proof or freeze.
Entry: `cc6874975060743dde67fd399865caf6288d2b8b`.
The [receipt](observations.json) retains 12 native/model observations, five
ordinary candidates in unchanged production Rust/Go checkers, and two legacy
certificates. Two distinct ordinary statements are accepted with zero axioms;
three alterations fail CORE_TYPE_MISMATCH. A rejection is not a certified
counterexample. Installed profiles, core rules and Certificate v0 are unchanged.

## 1. Derive roots from complete modeled state

The retained [owner/view probe](../storage-views/README.md) supplied an explicit
escaping descriptor list and retained heap records conservatively. This new
[private model](../../../../probes/go-practical/storage_lifetimes.py) layers a
typed state machine on that algorithm without changing its pinned source.
Its collection/exit API accepts neither a caller live mask nor an escape list.
It enumerates:

- All declared variables of every active activation and lexical scope.
- Constant entry argument values while their invocation remains active.
- All evaluated operands and pending call/return products, including every
  positional projection of the same joint invocation result.
- Captured assignment target locations before source assignment commits.
- Descriptors recursively in root arrays, structs and active closed-error
  payloads; a value-array copy retains the shallow-copied descriptors.

A descriptor with length zero and positive capacity still roots its backing
owner. Nil and typed NoAccess capacity-zero descriptors grant no positive
storage access. Backing cells must be Plain under the complete closed error
universe, so a nonplain error payload cannot hide a descriptor inside []error.
The extension also checks Boolean, bounded raw-byte string, full-width
unsigned-word and plain-error values. It is not a complete source type checker.

Allocating expressions enter an evaluation packet before another allocation
can collect them. A result can be assigned only once, with its exact signature,
and becomes ready after the full scope-exit translation succeeds. Projecting
an operand for continued evaluation materializes its own packet before the
joint result is consumed. A captured destination retains the original owner
even if an earlier left-hand-side assignment rebinds the source descriptor.
Root completeness still depends on future independent validation of the
actual source/capture/evaluation protocol; these model APIs are not evidence
that arbitrary Go source has been normalized correctly.

## 2. Retire, promote and reuse

Collection retires only owners unreachable from the modeled roots. Slots are
indexed by exact root element type; the candidate remains two positive slots
per type and 128 accessible root cells per owner. Reusing a slot gives a fresh
semantic owner identity. A stale descriptor is invalid rather than silently
referring to the new contents. Identity/history counters in this prototype
are ghost bookkeeping, not a claim of a bounded concrete semantic carrier.
Production needs a proved finite slot representation and namespace relation.

Scope exit is an atomic candidate transition. It rejects unconsumed evaluation
tokens, derives all surviving descriptors, retires dying nonescaping owners,
and promotes/rewrites escaping local windows together. The canonical packing
still preserves lengths, capacities, overlaps and hierarchical typed paths.
Inherited caller storage retains its identity; borrowed storage cannot become
fresh through promotion. A quota failure leaves the original state intact.

The 1,000-allocation loop observes the native scalar sum 499500. Its model
reuses one positive slot throughout. This demonstrates the proposed live-owner
accounting, not a cumulative-allocation bound, a native address/GC observation,
an OOM guarantee or proof that every source application fits the candidate.
Local value arenas, aggregate weights, activation and temporary counts, exact
slot canonicalization and implementation resource limits remain to be frozen.
Past-store snapshot counts and full layout weights need their own accounting;
separating namespaces does not make saved observation state free.

## 3. Entry and return observations use separate stores

Active entry argument values remain current roots even after parameter
reassignment: contracts can still observe their original positive-capacity
descriptors against the current store. An old expression instead reads its
captured past store. Immutable past snapshots are separate namespaces; they
do not keep the current slot occupied indefinitely and cannot resolve through
the current slot table after reuse.

For each invocation the prototype captures its entry and return-post stores,
original entry values and ordered outputs. The post store is captured before
retirement or escape translation, preserving the callee's postcondition and
input frame at that return endpoint. A later caller allocation therefore
cannot change the evidence's old or post read. Escape translation accompanies
the same invocation's evidence. This candidate evidence is not an accepted
serialized summary or a substitute for source/post/frame correctness proofs.

## 4. Two ordinary facts, three altered candidates

The [untrusted producer](../../../../probes/go-practical/lifetime-certificates/src/lifetime.rs)
quantifies over arbitrary A:Sort0 cells, a store of two 128-cell slots, a new
128-cell row, a live mask, a default value and a selected slot/address.

The masked-frame statement assumes mask(false)=false. Replacing slot false
preserves every masked read: the dead slot observes default and the other
slot retains its old read. The proof splits the slot Boolean only; it does
not enumerate address flags or stored data, use function extensionality/eta,
or add a semantic axiom. Weakening the dead-slot premise or claiming an
unmasked preserved read makes the altered proof fail type checking.

The past-namespace statement uses an independently quantified old store.
Updating a current-store argument cannot change an old-store read. Ordinary
beta/equality rules prove this; an altered claim identifying that old read
with the new current read rejects. The statement does not prove that the
source importer actually captures a correct independent past store.

Both production checkers agree on accepted export/certificate/axiom hashes
with zero axioms. Legacy Bool/Nat certificates remain accepted. The largest
candidate has 3,234 canonical bytes, 352 stored terms and 23 declarations.
The private 60-second sequential-child ceiling is a measurement setting,
not a frozen practical-profile budget or whole-application cost guarantee.

## 5. Selected validation and remaining work

Twenty-four targeted tests cover slot/identity separation, live roots and
quota failures, pending operands/results/targets, error payloads, value copies,
entry/post snapshots, recursive scope escape and repeated allocation.
The [native observer](../../../../probes/go-practical/lifetime_reference.go)
matches 12 independently specified model facts; its maps, snapshot copies,
type assertion and observation aggregation are instrumentation outside
verified roots. Native values do not observe physical reuse or GC behavior.

Planned practical_storage.go and go_practical_storage.rs retain the assigned
source lowering and independent storage/path validation responsibilities and
add root/retirement/translation validation; their dedicated planned tests
cover forged missing roots, wrong invocation/projection and stale slot IDs.
Ordinary foundation must prove complete root enumeration, safe retirement,
packing/translation and past/current observations. T06 must bind the complete
source/call/alias/frame/effect transitions. None of those executors is claimed
implemented by this private probe.

The [verification record](verification/verification.json) records selected
checks and why unaffected production suites are not repeated. Exact source,
limits, structural/capture/frame counters, schemas, hashes, independent import
and migration freeze remain open. W04 stays In progress; W05 owns the
cumulative T01 gate. T02–T11 and the full implementation/release goal remain.
