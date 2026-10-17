# Closed Go contract wire draft

This W04 draft closes the private source-overlay, source-scope and normalized
contract roots in [the catalog](catalog.json). Every root and expression arm
has exact required members; optional, unknown and inactive payload members
reject. The source and normalized languages each have 70 heads. This is a
fixture/reference-model boundary, with independent production import and all
source/definedness/value/permission/frame/loop proofs still required.
W04 stays In progress; W05 owns inactive publication and the T01 full gate.
The complete T02–T11 objective remains active.

The [scope model](../../../../probes/go-practical/contract_scope.py) adds two
candidate identities, `mpk.go.contract_scope.v1` and `mpk.go.binding.v1`, with
domains `MPK-GO-CONTRACT-SCOPE-1.0` and `MPK-GO-BINDING-1.0`. They avoid all
prior Go and reserved C# identities/domains. The source-overlay and normalized
contract identities reuse their previously proposed Go v1 names. Whole typed
contexts enter scope and contract records. Scope/binding/contract hashes cover
all required members except their own hash field, using the preceding narrow
canonical-JSON and domain-NUL rules. There is no caller proof-status field.

Raw overlay bytes are captured before capture-dependent bindings and loop
refs. Whitespace and equivalent JSON escapes retain their original raw hash
and size; duplicate JSON names reject. Normalized contract hashes never enter
the capture hash. Source loops therefore select physical source locations
before derived loop IDs exist. This removes a capture/contract/loop identity
cycle without replacing physical source selection by a caller-chosen ID.

The private capture contains one Go source and one overlay, with literal
fixture module metadata. Thirty-three retained original-Go type facts and
six synthetic boundary shapes come from the preceding carrier fixture.
Function/declaration positions use fixture locators; original native scope
facts identify binding token spans. Supplied error U, type roots and identity
roots are checked for internal consistency. None establishes independent
source/capture/U completeness, a general Go parser/importer or a complete
project snapshot. T02 must reconstruct the entire immutable selected source
and overlay capture independently.

Bindings have twelve exact members, including complete capture, source,
function, type and physical token span. Inputs follow signature order with
the value receiver first; results preserve their joint order. Current header
membership is a sorted set of lexical binding IDs, so a same-name shadow in
another loop cannot substitute for the selected binding. An unnamed result
uses a required null name. Binding IDs identify lexical declarations; runtime
invocations, joint returned values and post-state still need VC/source
bindings. A named result used at a header and at return keeps the same lexical
ID while its observation namespace changes.

The [wire encoder and decoder](../../../../probes/go-practical/contract_wire.py)
retain source nominal type IDs. Aliases resolve to the same target ID;
`Amount` and `OtherAmount` remain distinct despite equal ordinary carriers.
Source Bool and logical Bool are separate types. Field paths retain their
exact nominal owner and ordinal, with no blank-field or promotion shortcut.
Source error selectors become case type IDs in the same global U. Entry error
paths retain full slice capacity tails; an explicit empty case set means
nil-only. Entry storage remains read-only with external mutation forbidden;
contracts grant no ownership by listing an error path.

Every input, result, header binding, range pseudo-variable and heap observation
has an explicit entry/header/return namespace where applicable. `old` moves
the complete nested observation to entry. Thus `old(element(input,i))` and
`element(old(input),i)` retain different element namespaces. Results, header
locals and range pseudo-variables cannot escape into entry observations.
Range IDs bind the exact loop. Quantifiers and finite sums replace source
names by de Bruijn depths: bounds use the enclosing scope, and only the body
introduces a Nat binder. Independent shape decoding reconstructs source
expressions and rechecks types and scope against the fixture facts. Full
contract reconstruction rejects rehashed caller edits and mutated scope
authorities; this remains reference-model validation.

All70 source heads have exact arity/fields; normalized nodes have exactly
`op`, `type` and `payload`, with closed logical/source type arms. Decimal
literals retain full uint64/int64 values, and shifts retain the entire count.
The registered per-function limits count all supplied requires/ensures/loop
expressions together: 1024 nodes and depth 32, plus 64 clauses/invariants and 1024
loops. The older private 4096-digit parsing guard is an explicitly separate
probe resource limit and introduces no semantic maximum for Nat.

The catalog records13 kinds of partial-operation obligations: signed-to-Nat
nonnegativity, division/remainder nonzero, signed shift nonnegativity, byte
and element bounds, substring/reslice bounds, concatenation's256-byte domain,
matching error tag and valid same-snapshot sequence observations. These are
required proof members, not proof implementations. Go signed min/-1 remains
defined, and unsigned shifts acquire no width ceiling. Nat subtraction stays
truncated. Finite quantifier intervals are empty when upper<=lower; no bounded
search cutoff is introduced. Guarded Boolean/conditional definedness, complete
quantifier-body definedness and old-snapshot guard movement remain ordinary
proof obligations from the retained operation design.

The [24 source vectors](source-vectors.json) retain 5 normalized unproved
candidates, 1 normalized pending-variant candidate and 18 configuration
rejections. The pending candidate has three missing total-loop variants;
generated invariants/range ranks remain candidates. The
[six accepted normalizations](normalized-contracts.json) retain original raw
overlay bytes, complete scopes and normalized contracts. The
[70 typed component witnesses](expression-witnesses.json) are queries against
a captured fixture scope; they are not all literal members of one overlay
file. They cover every source and normalized head and roundtrip independently.

The [fresh replay](../../../../probes/go-practical/run_contract_wire.py) checks
97 canonical preimages: six scopes, six contracts, fifteen bindings and 70
expression components. Component hashes use the existing private pilot domain
and propose no public expression ID. Python and the unchanged Rust/Go hash
observers agree on complete canonical bytes/digests; each native observer runs
twice with identical output. The [receipt](observations.json) pins 577 inputs
and the pinned Go 1.25.0 inventory. Hash agreement proves neither source
semantics nor definedness, and produces no checker acceptance verdict.

Twenty [targeted tests](../../../../probes/go-practical/test_contract_wire.py)
cover raw-byte capture, receiver aliases, shadowing, named results, namespaces,
nested binders, nominal typing, full-width literals, capacity error paths,
numeric boundaries and rehashed/mutated authorities. The
[verification record](verification/verification.json) retains raw local logs,
selection reasons, fresh replay, metadata and compilation results. No new
Rust/Go source is changed; their earlier format/proof checks are retained by
source pins. Shared VIR/source-map/manifest/VC/policy/API/release roots and
unions, reviewed producer/consumer migration and four-profile preservation
still belong to W04. Source/body and ordinary operation/contract proof
implementation remain T02–T06; later application/release stages remain due.
