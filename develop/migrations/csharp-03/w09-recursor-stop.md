# CSHARP-03-T01-W09 ordinary-term feasibility finding and resolution

Status: `Complete`, 2026-09-04. Finding `CSHARP-03-T01-W09-F01` (P1) is
`Resolved`. This record retains the original counterexample and accepted
replacement. It is not itself a checker-capacity measurement or complete W09
freeze; those are linked below and recorded in ledger section 12.

## Retained finding

The original `develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md` section 4 used
`C(0) = Nat -> Bool`, products/sums/collections as higher Boolean trees,
conditionals through `Bool.rec` with an arbitrary concrete result, and bounded
folds through `Nat.rec` with a concrete accumulator. It required the checked
zero-axiom standard interfaces and prohibited a new kernel rule or axiom.

The actual checked interfaces are:

```text
Std.Bool.rec : Bool -> Bool -> Bool -> Bool
Std.Nat.rec  : Nat -> (Nat -> Nat -> Nat) -> Nat -> Nat
```

These types are decoded from `proofs/std/bool/std-bool.hex` and
`proofs/std/nat/std-nat.hex` and agree with
`crates/mpk-core/src/inductive_gen.rs:143`. Neither has a motive/result-type
argument. The retained direct `Bool.rec`-to-tree and `Nat.rec`-to-Bool
certificates therefore still reject in both checkers. `Nat.rec` rejects the
Boolean seed even when its major is zero, so reducing a numerical bound cannot
repair the old recipe.

The smallest retained Nat case remains `nat_to_bool.0`: 1,107 bytes, 84 term-
table entries, raw SHA-256
`f2dd60dbcbcf13bc48716e7aa855e5172c55825f066a11b7ece0a33c045435ed`.
Rust reports `CORE_TYPE_MISMATCH` / `check_type_mismatch`; the independent Go
checker reports `core_check`, `term 15 inferred const but expected const`.
Both are normal exit-code-1 semantic rejections, not a timeout, signal, build,
allocation, or scheduling failure.

## Authorized replacement

The amended W08 calculus uses a finite binary-addressed Boolean cube:

```text
C(0)   = Bool
C(d+1) = Bool -> C(d)
Z(0)   = false
Z(d+1) = lambda b. Z(d)
```

Selector binders are little-endian address bits. Product/sum/padding/sequence
selection introduces all selector lambdas before applying `Std.Bool.rec` at the
Bool leaf. Consequently every recursor false case, true case, major, and result
has the checked Bool type. A concrete state `S` is advanced without an
inductive recursor: the generator builds guarded `S -> S` transformers for the
finite role bound and combines contiguous ranges with a balanced, source-order-
preserving lambda/application/let composition tree. This lets any output
coordinate depend on any prior state coordinate without asking `Bool.rec` or
`Nat.rec` to return `S`.

Fixed-width index/range, scalar, decimal, and codec logic is emitted as finite
Boolean circuits. Wide domain scalars are never converted to unary Nat. The
replacement applies no `Std.Nat.rec`; it changes no core/checker code or
interface and adds no inductive shape, axiom, theory primitive, proof node, or
theory certificate. The exact address, padding, product, sequence, circuit, and
ordered-composition rules are normative in the amended foundation specification
section 4.

## Reproduction and controls

`develop/probes/csharp-03/recursor_feasibility.rs` is included by the assigned
owner `crates/mpk-vc/tests/csharp_practical_spec.rs`. It copies the checked Bool
and Nat certificates into a self-contained root, remapping only table IDs, and
adds one typed ordinary definition per case. Every case has 13 declarations,
empty proof/theory tables, and a recomputed total axiom count of zero. Canonical
decoding and export/axiom hash recomputation succeed before either checker runs.

| Case family | Cases | Rust | Reference |
| --- | ---: | --- | --- |
| Bool-valued `Bool.rec` | 2 | accepted | accepted |
| Nat-valued `Nat.rec` | 3 | accepted | accepted |
| direct `Bool.rec` returning `Nat -> Bool` | 2 | type mismatch | type mismatch |
| `Nat.rec` returning Bool | 3 | type mismatch | type mismatch |
| pointwise Bool selection under a Nat binder | 2 | accepted | accepted |
| two-address binary Bool cube | 1 | accepted | accepted |
| concrete function-valued state with static transformer composition | 2 | accepted | accepted |

Each checker receives the **same bytes**, twice, in independent processes:
15 cases x 2 checkers x 2 runs = 60 invocations. Per run, ten controls and
replacement cases accept; the five old cross-result applications retain their
expected type mismatch. The replacement cases establish the required ordinary
type path for binary addressing, pointwise selection, cross-coordinate whole-
state transformer composition, and `Let` sharing. They do not establish
semantic correctness of every foundation operation, full-program proof, sustainable maximum network
size, or release fitness.

Every case also records the number of term-table applications whose function is
the checked `Std.Nat.rec` declaration. That count is zero for every pointwise
Bool-tree, binary Bool-cube, and static-transformer replacement case; the owner
test recomputes it from the certificate instead of trusting the replacement
metadata flag.

The canonical record is
`develop/migrations/csharp-03/probes/recursor-feasibility.json`, raw SHA-256
`c1a9024df81555ab3af21926885c62a1da88ded918842ca9f657794a079a8785`.
It retains every certificate byte, standard-input and checker-source hashes,
decoded signatures, and raw output from both repetitions. Its
`capacity_measurement` binds
`develop/migrations/csharp-03/probes/checker-capacity.json` at raw SHA-256
`de040d4342e90a23e4bbe6464aeaccbfa9f2630c1423b77b716b40c805ac8a99`;
`release_gate` remains false. The observed host is Darwin arm64; this is not a
native Linux or release receipt.

Reproduce from the repository root:

```sh
python3 develop/probes/csharp-03/run-recursor-probe.py --check
cargo test -p mpk-vc --test csharp_practical_spec --test csharp_practical_inventory
```

The runner rebuilds the Rust CLI and independent Go checker, uses a fresh
temporary Go cache, and compares retained bytes and outputs. Each subprocess
has a 60-second deadline; an unexpected result aborts recording and is never
converted into a semantic rejection. `--update` explicitly regenerates
evidence and cannot relabel a mismatching checker result as accepted.

## Candidate amendment and completed follow-on gate

The W08 specification, generated definitions, descriptor, and conformance
vectors were regenerated as one deterministic set for the replacement:

| Artifact | Raw SHA-256 / identity |
| --- | --- |
| foundation specification | `29c5986e3c7ce2ab018e36eea61caaf9d9e53d6b8e47f0229ef4681db8c3fc8b` |
| foundation definitions | `25738447bf793e37dc2125e7a07da55a03fb15f2fa4dfb87b25646a16cc9d1b4` |
| foundation descriptor content hash | `d8c2a023f1c445470123519f5024a17aaca1766553331a2fed4733fecf7deec1` |
| foundation conformance vectors | `5889d91e2365dfb8bce4260a4eae0fb3dd63b2e5fa430f7ed5a6dc8a0220bdc1` |

No source-visible API, source generic, active profile, installed
registry, public route, production source, release candidate, or core/checker
behavior changed. F01 is closed because both checkers accept the exact
replacement type path. The separate capacity record subsequently accepts all
four ordinary-certificate counter families at limit minus one, limit, and
limit plus one through both checkers twice. The private freeze at
`develop/migrations/csharp-03/freeze/profile-freeze.json` fixes the remaining
schemas, names, transition rules, and limits. W09 is complete and W10 is ready.

Feasibility-amendment review result: zero findings. This record still claims no
release acceptance; W09 completion and review belong to ledger section 12.
