# W14 review finding: constructor transaction handoff

Status: prerequisite amendment approved by the user on 2026-09-07;
the implemented protocol and final verification are recorded in
`contract-attachment.md` and `integration-review.md`.
Owner: CSHARP-03-T03-W14 integration; affected frozen handoff: T02-W03/W05.
The earlier approved T02 binding/error/commutation amendment does not resolve
this separate constructor-result representation issue.

## Reproducer and original mismatch

W05's production source harness admits:

```csharp
public sealed class Data {
    public required string Name { get; init; }
    public int Count { get; init; }
}
// Selected body:
return new Data { Name = "ok" }.Count;
```

`PracticalConstruction.FinalizeInitializer` requires constructor execution,
construction-invariant observation, ordered initializer RHS evaluation/member
assignment, the public invariant, and finalization. An initializer exception
must discard the transaction and must not publish a value. Required members
cannot be assigned by the constructor in this profile.

Before the amendment, native `PracticalVirFunction` validation required zero or one result,
with that result equal to the SourceCall signature's normal result type. Source
constructor identity uses its owner type as that result. The source emitter's
`finish_constructor` therefore built a complete ordinary product. It could not
represent the unassigned required non-null string in the example. Returning an
empty string, publishing an incomplete product, or dropping the constructor
call/invariant would change the frozen semantics.

A provisional scalar-required initializer path was experimentally captured and
imported, but it constructed an owner value before initializer evaluation. That
path was removed during review; it is not accepted W14 implementation or evidence
of W05 finalization semantics. The replacement private transaction now passes
actual source emission and independent import, including required reference
members and exceptional cleanup.

## Approved concrete amendment

1. Retain source callable identity, original signature, raw body, source spans,
   method contracts and declaration roots unchanged. Add an explicit private
   constructor-execution signature linked to that original declaration.
2. Represent the constructor result as an owned construction transaction with
   per-member assigned bits and typed slots. An unassigned non-null member has no
   payload value; it is not a null or fabricated public value. Derive its concrete
   representation through the existing T02 product/option expansion machinery.
3. Add exact private begin/constructor-return/member-read/member-write/finalize/
   discard handoff rules, including delegation and early-return merges. A member
   read requires assignment on every incoming path. Only directly permitted
   receiver operations can use this state; it cannot be a source call argument,
   field value, comparison operand, selected return value, or codec input.
4. Carry the existing W05 ordered initialization plan into the source handoff,
   bound to its callable/body ordinal and source bytes. Preserve argument and
   initializer evaluation once and in source order. Initializer/constructor
   exception edges discard the same unique transaction.
5. Finalization alone creates the source owner value and retains its public
   invariant VC. Constructor normal exits retain construction-invariant VCs
   when init members exist, and public-invariant VCs otherwise. No proof is
   marked discharged by this amendment.
6. Independently validate the original/private signature relationship, assigned
   member coverage, exact member types, unique ownership, merges, all escape
   boundaries and finalization/discard in the native importer. Source maps and
   method-contract VC subjects remain linked to the original declaration.
7. Regenerate the affected T02 handoff schemas, hashed fixtures and current
   private evidence; preserve historical artifacts and installed release inputs.

## Required review and regression

Run the real W04/W05 source matrix through the W14 emitter/importer, including
required strings/arrays/source types, unassigned optional eligible defaults,
constructor delegation, early returns, nested creation, source-order helper
calls, throwing constructor/initializer RHSs, and missing/duplicate/init mutation
rejections. Rehashed VIR mutations must reject early publication, read-before-
assignment, duplicate writes, type swaps, dropped cleanup, forged private
signatures, stale source-plan ordinals and transaction escapes. Repeat the full
T03 source/fuzz replay and standard local gates after the final stable snapshot.

This amendment is a prerequisite to W14 completion, not a reassignment of W05
acceptance or an implementation of T04 source handlers/T05 boundary calls.
