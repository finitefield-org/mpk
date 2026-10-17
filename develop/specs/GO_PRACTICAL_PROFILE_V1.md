# Go practical profile v1

Status: frozen inactive design, GO-PRAC-T01-W05. Public selection, the installed
four-language registry and executable release remain unchanged. T02–T07 implement
and prove this profile; T08–T11 add maps/pointers and the second release. This
specification owns `mpk.go.practical.profile.conformance.v1` at
`develop/specs/vectors/go-practical-profile-v1.json`. Primary test owner:
`crates/mpk-vc/tests/go_practical_spec.rs`; publication checks:
`scripts/check-go-practical-spec.sh`. A declared future test owner is not evidence
that its implementation exists.

## Frozen source and semantic contract

The byte-pinned specification members in the conformance set are normative
data for admission, evaluation, contracts, operation signatures, representation,
limits and migration. The W04 completion audit resolves the earlier members'
historical draft/open-status wording. Only their semantic/schema/rule data is
incorporated; prototype execution claims, pending implementation notes and
historical observation statuses retain their original scope. No record claiming
unimplemented definitions becomes a proof or registered runtime descriptor.
This document and the final program-binding rules take precedence over earlier
checkpoint status descriptions. Unknown tags/fields, omitted required fields,
crossed scopes, duplicate identities and mixed schema generations reject.

Target is Linux amd64, GOAMD64 v1, full64-bit int/uint and Go1.25.0. Effective
source language minors are go1.22–go1.25; check complete module/toolchain patch
requirements before normalizing a file's language minor. Original packages
must type-check unchanged. Unselected unsupported declarations may remain
type-check-only; a selected/transitively reached unsupported construct rejects
with its exact closure path. Entry theorems exclude package initialization.
Reachable globals, I/O, clocks, randomness and concurrency remain excluded.

Immutable offline capture binds original files, manifests, selected module
versions/replacements, Go sums and archive provenance, build selection/tags,
effective file versions, compiler/stdlib/analysis inventories and exact source
identities. The full capture closure and selected verification closure are
separate. Missing snapshots, path/symlink escape, mutable namespace races,
network/VCS/generation/toolchain fallback and opaque reached bodies reject.
No rewritten package or source stub can replace original type checking.

V1 includes named/alias scalar/array/string/slice/struct values, ordinary local
syntax, constants/iota, fixed products of0–16 results, named/forwarded returns,
two-phase assignment and single-call result/state linkage. Closed non-pointer
application errors preserve NilInterface versus every concrete zero payload.
Error boxing checks its signature; calling Error includes all finite target
bodies and their effects. Arbitrary interfaces/generics, dynamic unknown calls,
recursive call closure and out-of-profile source forms remain rejected.

Strings preserve arbitrary eight-bit bytes, including NUL and invalid UTF8;
len/order/index/slicing/concat use complete source words. HasPrefix/HasSuffix
require exact registered Go1.25.0 symbol/body/target bindings or verified source.
String length is at most256 bytes. An exceeded modeled-domain bound is distinct
from a Go runtime panic. Slices retain typed owner, nil tag, offset, length,
capacity, initialized full backing and aliasing. Storage has128 elements and
two simultaneously live positive owners per exact root element type, subject
to the complete weighted-current/entry/post accounting rules. Positive-capacity
empty slices retain tails; zero-capacity non-nil values use checked NoAccess
normalization. Root value arrays/structs may contain descriptors; backing cells
cannot contain nested mutable references. Finite-ID, root completeness, lifetime
and allocator correspondence proofs remain implementation obligations.

Fresh local construction, make, value updates, overlap-correct copy and append
within existing capacity have explicit permission/frame obligations. Borrowed
input mutation and reallocating append remain excluded. Range is limited to
admitted arrays/slices; ordinary loops, nested loops and unlabeled break/continue/
return retain exact source control and evaluation order. Continue executes the
correct post step; break does not imply a false loop condition.

## Contracts, proof and bounds

The closed70-head typed overlay has exact nominal/logical types, positional
input/results, entry-old/current/header namespaces and source loop identities.
Generated invariants and variants are untrusted candidates. Every loop needs
initialization/preservation/all exits/safety; totality additionally needs a
proved well-founded Nat rank. Partial correctness permits safe divergence.
Requires cannot assume their own truth to prove definedness. Source binding,
full-word arithmetic, operations, calls, frames and totality are ordinary
zero-axiom obligations under the two separately frozen additive core rules.
See [foundation](GO_PRACTICAL_FOUNDATION_V1.md) and
[shared artifacts](GO_PRACTICAL_SHARED_ARTIFACTS_V1.md).

All64 numeric counters and complete carrier weights are frozen by the hashed
limits members. In particular practical syntax has4,194,304 nodes while old Go
retains1,000,000. Capture files remain16MiB each/256MiB total; contract files
1MiB each/8MiB total. Counts, UTF8 transport bytes, source byte values and full
machine-word semantics are distinct. All API/frontend/release transport
limits and LF/hash framing are exactly their owned schema members.

A separate native practical-Go host requires Linux x86_64 GNU, kernel ABI6.4.0,
cgroup v2, zero swap, namespaced immutable inputs and complete descendant
cleanup. Frontend tree:120 seconds/2GiB/128 tasks,512MiB scratch/131,072 inodes,
16GiB address-space limit,1,024 descriptors, no core dumps. Proof construction
and each checker run sequentially at60 seconds/1GiB,64MiB scratch/16,384 inodes.
Certificate caps are2MiB/131,072 terms/4,096 declarations. Native enforcement
and complete application fit must be measured at T07. Current Darwin samples
justify design headroom only. Missing host primitives are environment errors;
resource exhaustion stays pending and never establishes source/theorem falsity.

## Acceptance, migration and release

An admitted model, a generated proposition, an accepted source-free certificate
and a registered source-bound application claim are distinct results. Every
required source/definedness/operation/path/call/loop/frame/result member must
bind the exact independently regenerated ordinary statement and be discharged
by identical certificate bytes in both checkers. Go requires zero axioms.
Retained Rust/C#/Java contexts preserve their exact mvp-theory policy where
applicable. AI prose, valid metadata, definition-only checks and missing-member
lists cannot establish a source claim. Existing169 private Go demands remain
unelaborated at this publication; no complete Go program proof is published.

The183-file migration and24-requirement handoff remain mandatory implementation
coverage. Historical observations replay at their original source epoch; new
production bytes require fresh evidence. T07 activates one complete shared
artifact family only after all native local gates, both-checker application
proofs, four-profile preservation, two-pass installed replay and atomic rollback
checks. Inactive C# practical identities remain reserved and are not activated
as a Go prerequisite. Every stage's final W must pass its cumulative local
`check-fast`; release gates follow the root README. No workflow is required.
