# Sort0 Nat induction v1

Status: frozen inactive additive core interface, production integration T02-W03.
This document owns `mpk.core.nat_induction.conformance.v1` at
`develop/specs/vectors/core-nat-induction-v1.json`. Test owners are
`crates/mpk-core/src/nat_induct.rs` and
`go-tools/mpk-checker-ref/nat_induct_test.go`; publication checks:
`scripts/check-go-practical-spec.sh`.

The generated reserved recursor has exactly this monomorphic type:

```text
Std.Nat.induct :
  (P : Nat -> Sort0) -> P zero ->
  ((n : Nat) -> P n -> P (succ n)) -> (n : Nat) -> P n
induct P z s zero     = z
induct P z s (succ n) = s n (induct P z s n)
```

Register only a Sort0 family with exactly the two canonical generated
constructors in declaration order, `zero : Nat` and `succ : Nat -> Nat`, and
the exact full eliminator type. Constructor names belong to that family.
Reject nongenerated/wrong/reserved aliases, wrong constructor order/type,
extra constructors and actual universe arguments on the family, constructors
or recursor, including neutral/partial applications. Registration scans prior
declaration types, opaque values and theorem proofs for forbidden uses. Later
constructor extension rejects before environment mutation. Unused term-table
nodes are not actual uses. Independently enforce the same guards in Rust and Go.

Iota reduction applies these two equations after normal exact spine/constructor
checks. Preserve all old Nat.rec/Bool rules, normalization budgets and
Certificate v0 opcodes/bytes. Add no eta rule, proof irrelevance, axiom category,
higher-universe elimination or implicit rule selected by arbitrary names.

Private dual-checker probes establish arbitrary-Nat/Sort0 induction feasibility
and retained certificate behavior. Their exact source/receipt pins are in the
conformance set; malformed interface/candidate failures are not certified false
theorems. Production source/library integration and complete application proofs
remain later gates. This specification alone does not enable a public profile.
