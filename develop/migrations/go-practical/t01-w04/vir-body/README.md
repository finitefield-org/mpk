# Typed Go function-body and structured-region draft

This W04 proposal closes one Go function-body root, seven node arms, region
interfaces and three terminators. The [schema](schema.json) owns exact fields,
counter mapping and planned producer/importer/VC/test paths. The private
[constructor](../../../../probes/go-practical/vir_body_build.py) and separate
[reference validator](../../../../probes/go-practical/vir_body.py) use the
preceding concrete operation signatures. This is a typed graph proposal;
complete shared VIR, source reconstruction and ordinary semantics remain due.

The new candidate `mpk.go.function_body.v1` and
`MPK-GO-FUNCTION-BODY-1.0` avoid every earlier Go and reserved C# name/domain.
Every root member except `body_hash` enters its canonical preimage. Full typed
context, capture/source-scope/function/signature, complete normalized-contract hash,
all concrete definitions, entry bindings, the entire graph and all derived
obligation sites are bound. The consumer reconstructs the source environment
from original buffers, not producer-mutable maps, and recomputes the complete
root. No caller `verified` or proof-status member is admitted.

Each graph has globally unique canonical value, node and region preorders.
Parent outputs reserve IDs before child traversal but remain unavailable to
children. Every operand must dominate its use. Regions can read only their
own parameters/earlier definitions and an exact sorted capture set; sibling
values, future parent outputs, control captures and unused captures reject.
The complete referenced definition set is sorted and independently derived;
missing, extra, altered or crossed-context definitions reject.

The seven node arms are concrete operation application, internal location
address, full location load, ordered product projection and frozen assignment
values. Types retain source nominal IDs, full word widths and fixed ordered
function results. Internal locations grant no Go pointer feature, separation,
fresh owner or permission. The separate assignment-value sequence supports
more than 16 values; complete ordinary expansion/accounting of these internal
sequences remains required alongside the other operation/source proofs.

State inputs must use the current state. An update produces its successor;
older states cannot be replayed after a write/call or substituted for a child
region's current parameter. The single exception is immutable invocation-post
observation for product projection. Static calls and finite error dispatch
create their ordered product and post-state together. Projection checks the
same producing node and exact post-state, even when another invocation has
identical types. An ordinary pack cannot impersonate a call product.
Historical projection does not roll back the current state.

This is static provenance. Repeated node execution inside a loop still needs
the actual activation/invocation/source relation; no runtime same-invocation
theorem is inferred from a numeric SSA ID. Similarly, body dependency sites
pin exact source declaration/signature/object/span and require the callee's
ordinary body relation. An existing overlay is additionally bound where
present. A `source_body` requirement supplies no trusted callee contract or
permission to assume its behavior. Source closure/call cycles and complete
dependency programs remain independent import/VC requirements.

Control results form a barrier: instructions cannot continue past them.
`yield` returns the exact interface, `normal` returns the current normal state,
and `resume` runs a continuation only for a normal result while propagating
return/break/continue. Branch and switch outcomes combine their region exits;
a switch consumes only its own break. For/range consume only their own break
and continue, leaving return/outer targets intact. Init/test/body/post roles
and parameters are exact. Missing-test loops with no break have no modeled
normal exit; this permits a partial divergent body without asserting totality.
Actual post scheduling, source evaluation and termination still require
ordinary proofs, not merely these static control sets.

Internal addresses distinguish a live local/parameter/result root from a
separately frozen slice descriptor root. Paths retain exact nominal field
owners, nonblank ordinals and full integer array indices. A slice crossing
cannot be disguised as an array step. Rebinding a local aggregate retains its
location handle; a slice target retains the captured descriptor SSA value.
Creating an address does not eagerly perform the later read/write check.
Derived access sites attach full-index bounds, validity, initialization,
permission and frame requirements at the actual load/store/view point, using
that point's current state and the location's retained recipe. Ordered assignment
uses a prefix-state reference for each actual target ordinal, including blank
target positions, so later stores observe earlier updates without rollback. NoAccess and
borrowed inputs acquire no write grant through a typed handle.

Declarations initialize one captured local and dominate its later addresses.
Lexical scopes retire that availability on exit; for-init declarations remain
available in test/body/post and end at the loop boundary. Actual source-block
correspondence, activation/retirement, array/view association, alias and owner identity,
finite-id normalization and all source lifetime proofs remain due. Source
spans are bounded component locators, not independent AST correspondence.
Supplied switch context remains a private syntactic witness. All source facts
retain the earlier single-source/overlay fixture boundary.

The shared limits now have explicit body counting: every SSA definition,
including state/control/handle/region parameters, counts toward 65536 locals;
every region/continuation counts toward 8192 function blocks; parameters plus
captures count toward 4096 block parameters. All nodes count toward the 100000
block/function instruction bounds. Canonical JSON uses 192MiB, input JSON 256MiB,
decoded strings 1MiB and actual container depth 256, root at depth 1. Module-wide
counting belongs to the remaining shared module wrapper.

The [new Python canonicalizer](../../../../probes/go-practical/body_canonical.py)
and new untrusted Rust/Go hash observers use those shared JSON limits. They
preserve safe integral metadata, full words as strings, Unicode scalar values,
UTF16 key order, duplicate rejection and domain-NUL hashing. Rust imports the
unchanged production canonicalizer. The earlier 64-depth/128KiB private hash
observer and its receipts stay unchanged. Boundary vectors include 64, 65, 256
and 257 containers, empty arrays, duplicate names, unsafe integers and Unicode.
These helpers are not kernel changes or ordinary proof acceptance.

[Fifty-six component queries](components.json) exercise every operation family,
including guarded/control regions and static call projection. Their explicit
test interfaces may contain external handles/control context; they are not
function inputs or production VIR. The [three function candidates](functions.json)
exercise straight return, loop/init/test/post with break/continue and a normal
exit for the original discard-only InputErrors function. The source still
contains blank assignments; their erasure/source equivalence is unproved.
The first two bodies are typed candidates and do not claim to match the
original function's semantics. No candidate establishes a Go application claim.

The [20 graph cases](vectors.json) retain 3 typed-unproved candidates and 17
configuration rejections. Rehashed controls cover forward/sibling scope,
duplicate IDs, stale state, region/result order, target/header/context/source
crossing and omitted/altered obligation sites. The
[21 targeted tests](../../../../probes/go-practical/test_vir_body.py) also cover
call/pack/error projection, explicit captures, frozen slice targets, deferred
access, 32-value assignment, divergence and full JSON depth boundaries.

The [replay](../../../../probes/go-practical/run_vir_body.py) checks 69 complete
preimages in Python, Rust and Go: three body hashes, 56 private component hashes
and ten canonical boundaries, with 64 accepted and five rejected hash inputs.
Component hashes use the existing private pilot domain and introduce no public
component ID. Both native observers run twice identically; the
[receipt](observations.json) pins 604 source/data inputs and the same pinned
Go 1.25.0 inventory. The [verification record](verification/verification.json)
retains local logs and selection reasons. No production checker or Certificate
v0 byte meaning changes, and hash agreement supplies no source/ordinary proof.

W04 stays In progress. The shared five-profile VIR wrapper, complete capture/
module/callee/ordinary binding, maps/manifests/VC/policy/API/release roots and
reviewed producer/consumer migration remain required. W05 owns inactive
publication and the T01 full gate. All T02–T11 work remains active.
