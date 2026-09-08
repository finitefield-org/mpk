# CSHARP-03-T06-W02 construction and type-invariant VCs

This private handoff turns the validated source/object protocol and W01 contract
expressions into concrete typed sequents. It generates obligations; it does not
execute a solver, issue a proof receipt, publish a value, or enable a profile.
T06-W03..W09 retain their primitive/container/operation definitions, control
composition, and complete ordinary proof assembly/checker responsibilities.

`ConstructionVcProgram` is reconstructed from a validated VIR's original source,
foundation and closed roots. It has no public constructor, `Default`, or
`Deserialize` implementation. Its deterministic bytes bind the VIR hash, closed
roots, source-type hashes, member order, exact type-contract clauses, program
points, actual SSA carriers, structural evidence, and generated definition names.
The unchanged VC wire commits its digest and every sequent ID. Function
construction groups depend on the global construction group; skeletons retain
these dependencies. Import compares a complete reconstruction, including when
source metadata claims an obligation is discharged (the source importer rejects
such claims). The inherited no-actual-source T02 candidate route retains its
pending obligation groups; it does not obtain a construction handoff or proof.

## Ordinary equations and observations

Every predicate uses the W01 `ContractTerm` forms. Each sequent predicate has one
free binder with exactly `subject.type_id`, the actual SSA type. Type equations
use the public source receiver as their sole binder.

| Definition recipe | Meaning and ownership |
| --- | --- |
| `PublicDomain.<source>` | The emitted `public_body`: source shape, every recursive member domain, exact enum membership, and every frozen public clause. |
| `PublicDomain.<primitive/closed>` | The shared ordinary domain equation; containers recursively apply the same source-type equations to their elements/payloads. Primitive/container bodies remain with their T06 owners. |
| `SourceShape.<source>` | The shared finite source carrier, ordered stored members, and non-null public representation. This is structural membership, not a user invariant. |
| `EnumCarrierEquals.<source>.<arm>` | Equality to that exact declared underlying value. Enum equations explicitly disjoin the complete declared arm set; undeclared zero never enters the public domain. |
| `Bool.true/false/And/Or` | Ordinary Boolean constants and connectives. Generated conjunction/disjunction trees are balanced. |
| `field.read.<member>` | The existing shared stored-member projection, with its exact source and member type. |
| `SlotAssigned.<member>` | The initialized bit of that member in the validated private slot carrier. |
| `SlotRead.<member>` | Typed private slot projection. Its definite-assignment condition is a separate goal. |
| `AssignedSnapshot.<source>` | Logical source observation with all slots assigned; whole-receiver use in a construction clause requires all member bits. |
| `FinalizedSnapshot.<source>` | Logical source observation using only the validated non-required CLR defaults for remaining slots. Required/init/uniqueness/order evidence is required at this point. |

The snapshot recipes are logical observations, not executable SSA publication.
Direct construction-clause field reads are rebound to slot projections, so an
unset required member does not need a fabricated public value merely to observe
another assigned member. Free receiver uses inside lambdas/lets retain the correct
binder depth; locally bound values do not acquire receiver slot requirements.
W01 attachment identities and exact definitions remain attached to the transformed
clause. No source evaluator or second assignment/CFG algorithm is introduced.

At function entry, only public arguments/receivers become assumptions. Private
constructor receivers are excluded. Before ordinary calls and member assignments,
public argument/member domains are goals. Produced ordinary/literal values and
all returns retain domain goals, including declared invariants on recursive zero
values. Constructors of init-bearing types establish construction clauses;
constructors without init members establish public clauses. Object finalization
has public goals before the operation. Every function return/exceptional exit
re-establishes public parameter/receiver conditions. Exceptional construction
exits retain discard evidence and have no publication result.

Structural evidence records the previously validated source initialization plan,
constructor assignment Must/May, anchored writes/finalization, and discards.
It is not a boolean proving a user clause. All semantic goals remain pending in
ordinary theorem groups. Nodes (including source equation bodies), definition
references, theorem declarations, binders, and transport are bounded before
handoff. Limit failures retain the VC limit phase.

## Retained verification

- `requests.json` / `responses.json`: nine pinned local Linux frontend requests,
  with two equal capture runs. Seven produce source facts. Missing required init
  and `default` of an enum without zero reject at source capture.
- `goldens.json`: seven complete generated handoffs and their source VIR/VC hashes.
  Good/broken constructors, zero defaults, object initializers, and enum membership
  are covered. Broken semantic predicates still generate pending VCs; a small
  test-only evaluator of generated terms witnesses their false conditions.
- Tests also replay the eight earlier object-construction captures, including
  delegation, implicit constructors, arrays, nullable members, early return and
  exceptional discard. Existing control importer mutation tests cover malformed
  protocols and construction loops.
- Hostile handoff cases remove construction/publication/preservation/assignment
  sequents or alter predicates, subjects, required/default flags and lineage.
  Mutations preserve original JSON ordering and first verify an unchanged import,
  so rejection cannot be explained by serialization order alone.
- Unit cases check member versus whole-receiver reads, private carrier rebinding,
  nested local binders, and bounded sequent expansion. Every golden predicate and
  public equation is independently checked for ordinary application/binder types.

Primary owner: `crates/mpk-vc/tests/csharp_practical_vc.rs#CSHARP-03-T06-W02`, via
`tests/support/csharp_practical_construction_vc.rs`. Run its three integration tests
with `cargo test -p mpk-vc --test csharp_practical_vc construction::`.
`review.md` and `verification.json` record final scoped checks. The T06 full
`./scripts/check-fast.sh` gate is deferred to W09 under AGENTS.md. Frozen producer
bytes, old receipts, vectors, public formats, core/checkers and activation remain
unchanged.
