# Sort0 Boolean type cases v1

Status: frozen inactive additive core interface, production integration T02-W03.
This document owns `mpk.core.bool_type_cases.conformance.v1` at
`develop/specs/vectors/core-bool-type-cases-v1.json`. Test owners are
`crates/mpk-core/src/bool_rec_type.rs` and
`go-tools/mpk-checker-ref/bool_rec_type_test.go`; publication checks:
`scripts/check-go-practical-spec.sh`.

The generated reserved recursor has exactly this monomorphic type and equations:

```text
Std.Bool.recType : Sort0 -> Sort0 -> Bool -> Sort0
recType A B false = A
recType A B true  = B
```

This is an explicitly owned additive core computation rule selecting a type.
Register only a Sort0 Boolean family with exactly the two canonical generated
constructors, false then true with their exact family type, and the exact full
eliminator signature. Reject wrong/nongenerated names or declarations, extra
constructors, higher family/result universes and actual universe arguments even
in neutral/partial/prior uses. Scan prior declaration types, opaque definitions
and theorem proofs. Later constructors reject before changing the environment.
Unused term-table nodes do not create an actual application. Rust and Go enforce
the registration and reduction guards independently.

The two equations do not construct Eq Bool false true. Ordinary contradiction
elimination may use this type selector and the existing Eq rewrite rule but
must prove its premise. Preserve old Bool.rec/Bool.cases/Nat rules, all byte
formats and normalization budgets. No higher-universe extension, new axiom,
caller adapter, eta or proof-irrelevance rule is introduced.

The twenty isolated interface cases and ordinary false-elimination/strong-
induction receipts are private feasibility evidence with exact pinned inputs.
They distinguish malformed registration and wrong proof candidates from false
source claims. Production integration and source/application proofs remain
mandatory T02–T07 work. Publication grants no installed checker/profile route.
