# T01-W03 proposed Nat induction and abrupt control

This is private feasibility evidence for an additive core/foundation amendment.
The production Rust and Go checkers, old Nat interfaces, normative specifications
and public registry are unchanged. W03 remains In progress pending its resource
review. Predecessor: `cfb9e961a1e96a3ce29654a2e41767d04db61cb9`.

## Why an additional interface is needed

Legacy `Std.Nat.rec` has type `Nat -> (Nat -> Nat -> Nat) -> Nat -> Nat`.
It cannot return a complete state frame or a dependent invariant proof. The
[first diagnostic](../control-initial/README.md) retains the actual rejected
attempt. The corrected new-interface candidate is also rejected by both
unchanged production checkers. Those two limitations are distinct from a
false-claim counterexample or a production induction feature.

The proposed monomorphic Sort0 interface is separate:

```text
Nat.induct : (P : Nat -> Sort0) -> P zero
           -> ((n : Nat) -> P n -> P (succ n)) -> (n : Nat) -> P n
Nat.induct P z step zero = z
Nat.induct P z step (succ n) = step n (Nat.induct P z step n)
```

A canonical Sort0 family must have exactly its generated zero and successor,
with exact names, order and types. The complete eliminator type and generated
flag are checked before registration or reduction. Registration closes later
constructors. Nonempty universe arguments on the new eliminator, family and
constructors reject even in neutral/partial uses. Actual preceding types,
opaque values and theorem proofs are traversed so earlier universe arguments
cannot evade the restriction. Unused term-table nodes remain permitted.
Legacy `Nat.rec` and Boolean interfaces keep their types, equations and hashes.
No eta, proof irrelevance, new wire-format field, fuel increase or axiom is used.

The [prototype recipe](../../../../probes/go-practical/control_foundation_build.py)
mirrors four original Rust crates and the original Go checker into a private
temporary workspace. Two independent implementations add the proposed rule;
75 mirrored input hashes are checked. Production files are never rewritten.
The separate [candidate crate](../../../../probes/go-practical/control-certificates/Cargo.toml)
has 45 independently reproduced generated Rust inputs. Exact source/checker
pins surround measurement; the final replay also matches both binary hashes.
This extension must be frozen with explicit production ownership in W04 and
integrated/reviewed during T02; this prototype is not that integration.

## Ordinary conditional proof schemas

A frame contains all 128 arbitrary bits. `Iterate next n initial` uses genuine
Nat induction, with arbitrary `n`; it is not 128 finite unrollings. An arbitrary
invariant, its actual initialization proof and its state-dependent body
preservation proof establish the invariant after every iteration. These are
proof obligations to discharge for a generated source body, not additional
caller entry assumptions. The theorem does not assert that every body has an
invariant or terminates. Source-specific variant/exit linkage remains T06 work.

A halted-step wrapper preserves an actually stopped whole frame for arbitrary
Nat counts. Single-edge equalities establish continue through post while
skipping the rest, break without post, and return precedence over break.
All full frame fields remain quantified. The six correct claims pass both
private checkers with zero axioms and matching hashes; their six deliberately
false variants reject at core type checking. Explicit original-Go witnesses
refute erased initialization, preservation and stopped premises and incorrect
continue/post, break and return results.

Fifteen separate interface cases admit two legacy/unused-node controls and
reject thirteen malformed declarations/uses. Wrong successor typing rejects
at ordinary function typing; the other forgeries reject as invalid declarations.
These are interface-validation negatives, not semantic counterexamples.

## Native observations, replay and checks

The [original-Go observer](../../../../probes/go-practical/control_reference.go)
executes actual `for`, `continue`, `break` and `return` with complete two-word
frames. All 576 observations match independent bit arithmetic: 144 preserved
invariants, 32 bad-entry and 16 bad-step refutations, 288 stopped frames,
72 continue/post observations, 144 break exits and 144 return-precedence
observations. Runtime counts span 0/1/2/31/32/33/127/128/129; the ordinary
proofs separately quantify arbitrary genuine Nat. Observer allocation and I/O
are not admitted Go functionality or a proof of the production source binding.

```sh
python3 -B develop/probes/go-practical/run_control_probes.py --write
python3 -B develop/probes/go-practical/run_control_probes.py --check
python3 -B develop/probes/go-practical/check_control_report.py
python3 -B develop/probes/go-practical/check_control_initial.py
PYTHONPATH=develop/probes/go-practical python3 -B -m unittest test_control_report -v
```

The complete 27-candidate measurement and separate final replay pass, including
two fresh generations, exact certificate/native bytes and both checker binary
identities. No production checker acceptance is inferred. The arbitrary-Nat
invariant certificate has 26 declarations, 287 terms and 2,869 bytes. Its
observed Rust/Go wall time is 0.006/0.582 seconds and peak resident size is
2,965,504/5,652,480 bytes. Other claims and original CPU/resource observations
are retained individually; cold process startup can dominate tiny proofs.
The adjacent [hardware receipt](local-resource-host.json) identifies the
Darwin/arm64 M1 Max host and 64 GiB RAM separately from the original report.

[Retained verification logs](verification/checks.json) record seven new Rust
interface tests, five new Go interface tests, seven existing Rust Boolean
regressions and four Go Boolean regressions. Thirty-five report/native/build/
diagnostic adversarial tests pass in 1.953 seconds; two receipt tests also pass.
The test sources and independently rebuilt checker inputs match. Offline
locked release Clippy passes with warnings denied. Current Rust and pinned Go
formatting exclude archived snapshots. Unchanged production suites and earlier
large kernel matrices are checked by preservation rather than rerun.

The private 60-second ceiling is not a frozen profile budget. W03's remaining
resource-envelope and cumulative exit review is in the
[exit audit](../feasibility-exit-audit.md). The full T01 gate stays deferred to
W05, and the full eleven-stage objective remains active.
