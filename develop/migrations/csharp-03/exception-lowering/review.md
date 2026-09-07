# CSHARP-03-T04-W04 review

Baseline: `25c8a80b2fe59dc4400898f9a6f480dfd749be3e` (T04-W03).

## Scope and representation

The opt-in `allowExceptionControl` route extends the reserved source-exception
base-clause gate. A source exception is sealed directly over the exact pinned
`System.Exception`; its immutable payload, constructor body, assignment state,
argument order, publication and synthesized IL use the existing T03 validators.
The implicit or explicit base call is exactly parameterless. The nine built-in
exception types use only parameterless explicit construction. Every admitted
source throw is a standalone `throw new ExactException(...)` statement.

The private wire schema is `mpk.csharp_practical.t04_w04.exception_lowering.v1`.
Source payload construction uses the T03 immutable value transaction, followed
by `closed_exception`, one `explicit_throw` edge and an `exception_exit`.
The original constructor/source operations and sequence ownership handoff are
retained. Native attachment binds exception declaration member names to the
registered immutable member IDs and source hashes, then derives the existing
closed universe. Built-in tags remain 0–8; sorted source IDs follow them.
Every explicit exit records its exact type/tag, optional payload type, value
and successor IDs. No runtime exception instance, message, stack or identity
is serialized as the exception value.

The distinct exact type IDs in the ordered method-sidecar `exceptional_cases`
form the declared throws set. Multiple path-conditioned cases for the same type
remain ordered and supported. Exceptional postconditions are type-checked with
an `exception` binding and no normal result. Immutable payload projections track
their exact subject arm, including typed let aliases and known conditional
subjects. A different arm's member, Message/StackTrace, wrong result type,
pre-state exception or normal-postcondition exception binding rejects.

Exits outside that exact set remain pending catch-or-unreachable obligations;
a superclass declaration does not implicitly widen the declared set. The
operation-result consumer validates the existing T03 signature and invocation
and preserves each original check/type/successor triple. It derives no second
built-in conversion. Classification does not prove reachability, exception
freedom, postconditions or termination.

Catch/filter/finally/rethrow remain W05-owned. W06 composes source and data
edges, emits/imports whole-control VIR and revalidates construction state;
T06 discharges proofs. This route remains outside the installed frontend csproj
and emits zero frontend-success, VIR or certificate artifacts.

## Review and fixes

- The existing explicit base-operand normalizer and synthesized-constructor
  IL check assumed `System.Object`. Both now recognize the exact source
  exception base only on the opt-in route; explicit-base and empty-exception
  CLR cases cover the correction.
- Payload projection initially depended only on the surrounding exceptional
  case. It now validates the actual subject's closed arm, preserving typed
  aliases and rejecting a different closed literal's payload projection.
- The ordered exceptional-case list must preserve multiple cases for one type;
  its distinct type projection is the throws set. A regression preserves this
  behavior instead of imposing a new uniqueness restriction.
- Legacy W02/W03 wires reject the new declaration field even when null.
  Source declaration hashes and constructor operand types are checked before
  producing typed exits; source regeneration also rejects a changed type.

- Hostile source-tree child counts use checked arithmetic; duplicate throw
  ordinals and exception edges on pure closed-value wrapping reject. The
  oversized child-count regression verifies an error instead of a panic.

- A source constructor that throws still needs its normal initialization
  path. The previous loop builder rejected that field assignment. W04 now
  retains `construction_assign` only for a source-verified constructor and
  native-verified constructor identity, member and receiver. Its unique
  construction state remains T03-owned and is composed/revalidated at W06.
  The constructor-failure vector executes both caller and constructor graphs,
  comparing the original CLR exception type and normal initialized payload.

Latest task review: no findings. The retained corpus has 39 source cases,
18 private handoffs and 21 artifact-free rejections. Its 90 CLR runs cover all
built-in tags, source fields/get-only properties/empty payloads, explicit base
calls, argument order and failure, loops and patterns. The independent Rust
interpreter compares both the exact exception type and immutable Code payload.
It models the small fixture constructors; general constructor correctness is
still checked by the T03 construction owner. Native tests additionally cover
value/edge/declaration mutations, contract scope/type and inherited operation
outcomes. Conformance binds the original sources, graphs and frozen compiler
probe; final command results and file hashes are in `verification.json`.
