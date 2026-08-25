# MPK Program Certificate Alpha v0

Status: approved implementation profile for the current dual-checker release.

This profile resolves the certificate-assembly ambiguity between `VC_V1.md`,
`CERT_V0.md`, and `POLICY_V1.md` without changing Certificate v0 bytes or the
accepted feature set of either source-free checker. It is source-language
neutral even though the first required producer is the Rust policy route.

## 1. Scope and compatibility decision

The current Rust fast kernel and independent Go reference checker have one
common accepted certificate subset:

- the root certificate has `imports: []`;
- the root certificate has `proof_node_table: []`;
- the root certificate has `theory_certificates: []`;
- no `TheoryPrimitive` declaration or `Theory` proof node is present;
- the recomputed axiom report has total count zero; and
- every theorem is inhabited by an ordinary checked core term.

`mpk.program_certificate.alpha.v0` fixes that intersection as the only release
mode for RUST-06. Certificate v0 continues to encode import, proof-node,
theory-certificate, `TheoryPrimitive`, and `Theory` fields, but the program
assembler does not mistake a detached proof-node DAG for a theorem declaration
proof term. Enabling imports or theory features for trusted policy evidence
requires a separate specification, matching Rust and Go checker
implementations, checker-agreement vectors, and an axiom-policy review.

The `mvp-theory` axiom profile permits a zero-axiom certificate. Selecting that
profile does not require a theory certificate and does not authorize an
otherwise unsupported theory table.

## 2. Authoritative inputs and all-or-nothing result

Assembly consumes only:

1. one retained successful internal policy scan;
2. the validated canonical VC v1 bytes generated from that scan;
3. the validated grouped skeleton generated from those VC bytes;
4. the exact certificate-stage source-manifest bytes derived from the retained
   frontend-stage manifest; and
5. structurally generated member proof terms checked against the exact lowered
   member proposition.

Before proof search, the assembler builds a complete interface plan: it lowers
every required type, value, and proposition, fixes the self-contained
foundation closure, allocates all level and interface-term IDs, and recomputes
every generated declaration interface hash. A missing or mismatched registered
foundation interface, invalid lowering, or unresolved skeleton dependency is a
fail-closed assembly/configuration error and emits no policy evidence. It is not
misreported as ordinary proof search.

After a valid interface plan, the proof/checker result is exactly one of:

- `Pending`, when at least one member has no complete structural proof, with no
  candidate certificate and no checker run;
- `Candidate`, containing every generated group declaration in one canonical
  Certificate v0 byte sequence that both checkers accepted with identical
  checked reports; or
- `Unaccepted`, containing that same proof-complete canonical candidate after
  either both checkers deterministically rejected it or exactly one checker
  accepted it and the other deterministically rejected it.

If any member lacks a structural proof, the result is `Pending`. The assembler
never emits a partial certificate, never promotes a proved sibling member
independently, and never turns an unavailable proof into a source-unsupported
result. Strict policy verification writes the valid pending evidence and then
fails.

`Unaccepted` is a reportable checker verdict, not successful verification. The
producer validates the retained candidate bytes, exact certificate-stage
manifest, generated declaration interfaces, zero-axiom report, and any
accepting checker report before constructing evidence. It then validates and
commits the JSON/Markdown evidence pair and returns failure regardless of
strict mode. A checker execution/protocol/internal failure, or two accepting
checker reports that disagree, is not an `Unaccepted` result and publishes no
evidence pair.

## 3. Contextual VC-to-core lowering

VC v1 is a typed, name-based verification language, not a byte-for-byte dump
of Certificate v0 term nodes. Its validated VIR context supplies the type of
every value term. The assembler MUST independently reconstruct and type-check
the VC and skeleton before lowering them.

Define three total-on-supported-input functions:

```text
LowerType(vc_type, context)  -> core type
LowerValue(vc_term, context) -> (core value, core type)
LowerProp(vc_term, context)  -> core proposition
```

`LowerType` and `LowerValue` resolve only declarations copied into the checked
self-contained foundation closure or earlier generated declarations. A name
without an exact checked interface has no lowering in this profile.

`LowerProp` is contextual:

1. `forall(T, body)` lowers to `Pi(LowerType(T), LowerProp(body))` with the
   specified de Bruijn shift.
2. A two-argument VC application named `Std.Logic.Imp` lowers to
   the checked zero-axiom `Std.Logic.Imp` declaration applied to
   `LowerProp(antecedent)` and `LowerProp(consequent)`.
3. A two-argument VC application named `Std.Eq` lowers to the checked
   zero-axiom dependent equality family with the independently inferred common
   operand type inserted as its explicit first argument. Both operands lower
   with `LowerValue`.
4. Any other VC term independently inferred to have a supported Boolean
   carrier lowers through `Holds`:

   ```text
   Holds_T(b) = Std.Eq(T, b, True_T)
   ```

   where `True_T` is the checked true constructor for that exact carrier. The
   assembler does not coerce between distinct Boolean carriers.
5. A term that is neither one of the exact proposition markers above nor a
   value of a checked Boolean carrier has no proposition lowering.

This rule keeps program Boolean operations value-level while making their use
as an assumption or conclusion an actual core proposition. It also prevents a
Boolean constructor such as `Std.Bool.true` from being accepted directly as a
theorem type.

### 3.1 Generated grouping markers

The three names frozen in VC v1 grouping are serialization markers at the
VC/skeleton boundary. When the assembler reconstructs the generated grouping
tree, it lowers them as follows:

```text
generated True     -> Std.Eq(Std.Bool, Std.Bool.true, Std.Bool.true)
generated Imp(P,Q) -> Std.Logic.Imp(LowerGenerated(P), LowerGenerated(Q))
generated And(P,Q) -> Std.Logic.And(LowerGenerated(P), LowerGenerated(Q))
```

`LowerGenerated` recursively applies this table to reconstructed grouping
nodes and applies `LowerProp` to stored VC leaves.

The proof of generated `True` is checked `Std.Eq.refl`; generated conjunctions
use checked `Std.Logic.And.intro`. All three lowered propositions inhabit
`Sort 0`. The assembler knows which nodes are generated because it reconstructs
them from the validated VC member/group arrays; it MUST NOT infer marker
provenance from an arbitrary untrusted term tree.

Stored member assumptions, conclusions, and inline `forall` bodies use
`LowerProp`; nested program Boolean `and`, `or`, `not`, or `if` operations
inside those stored terms remain value operations and are wrapped by `Holds`
when they form a proposition. Thus the same serialized `Std.Bool.and` spelling
cannot accidentally combine core proposition types.

When exact-hypothesis reuse needs one leaf of a multi-element conjunction, the
planner may follow only the frozen balanced tree and use the checked
`Std.Logic.And.rec` eliminator. It cannot flatten, commute, or search for a
logically equivalent leaf.

### 3.2 Exact theorem interface

For every group, apply the section 3 lowering to the exact balanced tree,
member order, local binders, outer requires, and parameter binders defined by
`VC_V1.md`. The resulting core type—not the skeleton JSON bytes—is the type in
the Certificate v0 theorem declaration and the input to the recomputed
`MPK-DECL-0.1` interface hash.

Two assemblers agree only if their complete lowered core terms and canonical
certificate bytes agree. Hashing a `GroupedTheoremDeclaration` JSON object
under the declaration domain is not a certificate declaration hash.

The interface plan orders terms so that the complete foundation/type/value and
generated theorem-type closure is allocated before any proof-only term. A
candidate appends proof-only terms after that fixed interface closure and reuses
an earlier term only when structurally identical. Because Certificate v0
theorem interface hashing excludes the proof field, a pending plan and its
eventual candidate have the same generated declaration hashes. The candidate
must recompute and match every planned hash.

## 4. Self-contained checked foundation closure

The current checkers do not resolve Certificate v0 imports. The alpha assembler
therefore constructs one self-contained root certificate:

1. Decode and canonically re-encode each registered foundation source
   certificate selected by the lowering.
2. Require both source-free checkers to accept each selected source
   certificate, require its own import and theory tables to be empty, and
   require its recomputed axiom count to be zero.
3. Order selected source certificates by `(module, export_hash,
   certificate_hash)` and preserve declaration order within each source.
4. Copy only the transitive declaration/level/term closure needed by the
   lowered program, remapping every ID deterministically.
5. Reject every duplicate global name, conflicting interface, forward
   reference, unsupported declaration kind, or non-zero axiom dependency.
6. Append generated declarations in VC callee-first,
   contract-before-panic-free order.
7. Recompute the complete root export block and axiom report and require
   `imports: []`, `proof_node_table: []`, `theory_certificates: []`, and total
   axiom count zero.

The copied declarations are not trusted because they came from a foundation
file. They are trusted only because the final self-contained root certificate
checks from scratch. Foundation source certificates are not extra
`trusted_evidence.certificates` rows; the only candidate row remains `program`.

This is a temporary compatibility profile, not import emulation. The assembler
must not strip an import from an already assembled certificate, resolve an
ambient file by module name, or substitute a declaration having only the same
name.

The initial closed foundation registry contains only the canonical
module/export/certificate hash tuples recorded for:

- `proofs/program/base/std-program-base.hex`;
- `proofs/std/bool/std-bool.hex`;
- `proofs/std/eq/std-eq.hex`; and
- `proofs/std/logic/std-logic.hex`.

All four are self-contained, dual-checker accepted, and zero-axiom. A local
path is not identity; the assembler verifies the pinned tuple and canonical
bytes before selecting a closure. In particular,
`proofs/std/bitvec/std-bitvec.hex` is not an alpha foundation source because
its current report contains `CoreAxiom` entries. Consequently a VC requiring
its literals, operations, or relations has no valid alpha interface plan. This
is a fail-closed profile limitation, not permission to omit the operation from
the theorem type.

## 5. Structural member-proof subset

The alpha proof planner may construct only ordinary core terms from:

- the canonical inhabitant of generated `True`;
- checked `Std.Eq.refl` when both lowered operands are
  definitionally equal under the existing deterministic checker rules;
- the exact current outer-requirement or member-assumption implication binder,
  with `Std.Logic.And.rec` projection along only the frozen balanced path when
  the required proposition is one exact conjunction leaf;
- an exact earlier generated declaration listed by the skeleton dependency
  set, instantiated with checked arguments;
- lambda introduction for parameter, local, and implication binders; and
- checked `Std.Logic.And.intro` in the
  specification-frozen balanced shape.

Candidate matching uses the complete lowered core proposition. A member ID,
classifier label, VC hash, source expression, theory solver verdict, or helper
proof-node status is not a substitute for that equality.

The planner has no fallback axiom. It emits theorem proof `TermId`s directly
and leaves the certificate proof-node table empty; an API or helper proof-node
DAG must first be compiled into one of these complete ordinary terms. It does
not use `ProofNode::Theory`, a theory-certificate table, `TheoryPrimitive`, or
an unbound solver result. A VC outside this subset remains proof-pending.

## 6. Canonical certificate and checker agreement

The assembler attaches the exact retained certificate-stage source-manifest
bytes and then derives, in order:

1. the export block;
2. the complete zero-axiom report;
3. export and axiom-report hashes;
4. canonical Certificate v0 bytes with the embedded certificate-hash
   placeholder required by v0; and
5. the external certificate hash over those bytes.

The Rust fast kernel and Go reference checker receive the same byte slice. Both
must agree on acceptance, module, declaration count, certificate hash, export
hash, axiom-report hash, total axiom count, and complete axiom report. A
rejection or disagreement cannot emit `mpk_verified`; an execution/internal
failure publishes no evidence pair.

The Go checker executable is the deterministic static asset embedded in the
executing `bin/mpk` and byte-checked by the release build gate. It is not
resolved from `PATH`, a source checkout, an installed sibling path, a release
registry executable, or a callback. `mpk` copies only those embedded bytes to
a sealed anonymous executable, clears the child environment, supplies the
candidate on standard input, and bounds wall time plus stdout/stderr. The Go
process remains an implementation independent from the Rust fast kernel; its
embedding changes executable provenance, not checker semantics or proof
inputs.

Package verification also retains the byte slice accepted by the Rust checker
and submits that exact slice to the Go checker. It MUST NOT reopen the
certificate pathname between checker invocations.

The checker-result lifecycle is exact:

1. Two accepted, byte-bound, equal reports produce `Candidate`.
2. Two deterministic rejections produce `Unaccepted` with
   `POLICY_CHECKER_REJECTED`.
3. Exactly one deterministic rejection and one byte-bound acceptance produce
   `Unaccepted` with `POLICY_CHECKER_DISAGREEMENT`.
4. Two accepted reports that differ cannot be represented faithfully by policy
   evidence v1's single shared report-hash triple. That disagreement fails with
   `POLICY_CHECKER_DISAGREEMENT` before publication.
5. A launch failure, malformed or contradictory checker transport, internal
   checker invariant, or accepted report not bound to the submitted bytes fails
   with `POLICY_CHECKER_EXECUTION` before publication.

## 7. Policy evidence projection

After dual acceptance, evidence contains:

- the singleton certificate ID `program`;
- every and only `VC.Function.*` group declaration with its actual certificate
  interface hash;
- exact direct generated dependencies from the skeleton;
- each verified member's containing checked declaration reference plus the
  required transitive generated-declaration closure;
- a checked zero-axiom report;
- two accepted checker verdicts over `program`; and
- `theory_certificates: []`.

For `Unaccepted`, evidence instead contains the same singleton `program`
candidate, complete generated declaration projection, checked zero-axiom
report, and empty theory-certificate array. Its two ordered checker verdicts
are either both `rejected`, or one `accepted` and one `rejected`; every verdict
names `program`. Every selected member and property remains `proof_pending`,
has no trusted declaration reference, and carries at least the `vc` helper
reference. After contextual validation, policy verification commits both
evidence outputs and then returns `POLICY_CHECKER_REJECTED` for dual rejection
or `POLICY_CHECKER_DISAGREEMENT` for acceptance disagreement.

For `Pending`, no candidate exists, both checker verdicts are
`not_run`, and every member remains `proof_pending`. Pending member rows carry
the exact planned declaration-interface hashes from section 3.2, not hashes of
the skeleton JSON. This all-or-nothing rule is the alpha release rule even if
some groups could be checked independently.

Accepted-report disagreement and checker execution/protocol/internal failure
produce neither the `Unaccepted` projection nor any policy evidence output.

## 8. Required agreement cases

RUST-06-T03 must add dual-checker cases for generated `True`, checked equality
reflexivity, exact-hypothesis reuse (including one balanced conjunction
projection), balanced multi-member conjunction introduction, and an exact
earlier generated dependency. It must
also prove fail-closed behavior for a nonempty import table, nonempty proof-node
table, nonempty theory table, `TheoryPrimitive`, `Theory`, a raw Boolean theorem
type, an unregistered value/type interface, a skeleton/core declaration-hash
substitution, and one pending member among otherwise provable siblings.

## 9. Deferred features

The following are explicitly outside this profile:

- hash-resolved Certificate v0 import environments;
- imported-global numbering and imported axiom propagation;
- proposition-bound theory payload formats;
- Go reference checking of theory certificates;
- `TheoryPrimitive` registration; and
- theory-backed theorem-term construction.

Adding those features may retain the Certificate v0 binary encoding because
the tags are reserved, but it changes checker semantics and therefore requires
its own governed task and cross-checker vectors. Until then, documentation and
release evidence must not describe them as active.
