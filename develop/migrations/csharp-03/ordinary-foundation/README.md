# W09 internal unit 1: ordinary carriers

This directory records the first of eight user-approved internal W09 units.
It contains actual ordinary carrier/helper definitions and regression fixtures.
Internal unit 2 (scalar operations/checks) is also complete; see
`unit-2-verification.json` and `unit-2-review.md`. W09 is still in progress:
structural/collection foundations, domains, bindings/codecs, control/transition
relations and application proof assembly remain units 3-8 in
`implementation-plan.md`. Checking these carrier certificates does not
discharge an application VC.

## Representation and trust boundary

`generate_csharp_practical_ordinary_carriers` consumes only a validated original
VIR. It reconstructs the reachable type inventory and every expanded instance,
including instances whose operations are not invoked. Source products retain
stored-member order and IDs; enums use their underlying width; exceptions use
the derived closed tag/payload universe. Recursive references, unknown types,
residual template shapes and unresolved constants fail before a result is returned.

A cube C(d) is `Bool -> ... -> Bool`, with d address arguments and a Bool leaf.
Within each field/array/scalar address group, selectors are least significant
bit first. Products use a field address followed by the padded child address.
Sums use a tag/payload selector; tag is u32, payload is the largest active-arm
product. This storage reserves one active payload, rather than concatenating all
arms. Constructors, active-tag validity and inactive/unused-zero predicates
belong to unit 3. This unit defines their layout, not those predicates.

Widths are the specified integer/IEEE/UTF-16/Guid/time widths. Decimal stores
sign, scale and a 96-bit coefficient explicitly. String capacity is 16,384
UTF-16 units. Sequences/map entries/set elements have capacity 4,096. Validation
invalid payloads retain a role bound of 256 over their shared sequence carrier.
Construction state holds length, 16,384 element slots and initialization bits.
Product children and sum payloads have explicit padding to the maximum child
depth; pad prepends address bits that must be false, and unpad supplies false
for those bits. Public domains and default eligibility remain separate from
the storage-zero helper.

The builder starts from hash-pinned frozen Std.Bool bytes and emits only ordinary
Sort/Var/Const/App/Lam/Pi/Let terms. Pointwise mux calls Std.Bool.rec at Bool
leaves. Concrete `S -> S` composition uses Lam/App/Let, ordered first f then g;
fixed lists use balanced composition and count all leaf occurrences before DAG
sharing against the cumulative 16,384 transformer ceiling. Empty composition
is the identity. No Nat recursor or function-valued recursor is introduced.
Terms are interned; actual term/declaration/binder limits are enforced, including
forward/missing term references. Checkers still validate declaration typing.

`import_csharp_practical_ordinary_carriers` reconstructs both metadata and core
bytes and requires exact equality. A caller-provided digest, inventory or
certificate cannot replace reconstruction. Raw metadata SHA-256 in the golden
file is distinct from the domain-separated Certificate v0 hash. Generated names
use valid alphabetic prefixes and an injective hex encoding of concrete type IDs.
The checkers and Certificate v0 acceptance rules are unchanged.

## Local verification

`carriers/*.hex` are reproducible from the existing binding, exception, transition
and constructor request/fact captures. Cases with intentionally broken source
invariants are carrier positives only: their application VCs are not proved.
`carriers/goldens.json` pins original VIR hashes, layouts, byte hashes and actual
term/declaration counts. Source field order is also checked independently against
captured stored-member declarations. All role layouts and enum/source products
are exercised. Unit tests evaluate helper behavior with a separate finite Bool
interpreter; this interpreter supplies test observations, not trusted proofs.

The Go checker-agreement harness sends exactly the same decoded bytes to both
checkers and requires every carrier certificate to be accepted with zero axioms.
It compares module/declaration/axiom counts and all report hashes, and requires
hash-corrupted bytes to be rejected. Process/build/internal errors cannot pass
as proof rejections. The predecessor corpus remains part of the same local gate.

Targeted commands and results are recorded in `unit-1-verification.json`.
To regenerate candidate fixtures for review, run:

```sh
MPK_W09_CARRIERS_OUT=/tmp/mpk-w09-carriers cargo test -p mpk-vc --test csharp_practical_vc csharp_03_t06_w09_carriers_original_inputs_and_mutations
```

Normal tests compare checked-in bytes and layouts without updating them.
The repository-wide `./scripts/check-fast.sh` is deferred to T06-W12, per the
repository rule for intermediate W work. No GitHub Actions workflow is used.
