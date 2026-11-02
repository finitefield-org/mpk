# Complete compact signed-division prerequisites

This T02-W01 checkpoint assembles all established unsigned-division, signed
magnitude, half-bound, minimum/sign and zero prerequisites, plus the actual
SignedDiv/SignedRem source definitions, in one ordinary zero-axiom certificate.
It has 3,483 declarations, 85,883 terms and 1,255,288 bytes. Its logical
certificate hash is
`d9a8e6b5ef4d8da96c60e69b1b0999ccc4915d9c11e967892200f9139f461749`.
The frozen 4096/131072/2097152 bounds remain exact, leaving 613 declaration
slots for the signed-division laws. Those laws are still required; this is
a prerequisite certificate and W01 remains In progress.

The [structural dependency measurement](prerequisite-analysis/result.json)
finds 4,570 declarations in the original unsigned-division/signed-bound union.
Even the closed dependency union for the required final theorems has 4,173
declarations, exceeding the unchanged limit before new signed laws. Its
source-pinned partial binary reader supplies planning evidence only and is
never checker or import authority. No frozen bound or required signed theorem
is removed to accommodate this result.

The [direct zero component](../word-zero/README.md) proves the same actual
zero/nonzero/sign interfaces using a balanced all-bit predicate, avoiding
the general order/carry closure. The private untrusted
[composer](../../../../../../crates/mpk-vc/src/go_practical_ordinary/composition.rs)
freshly generates and canonically decodes four complete inputs:

- The original unsigned-division component, including all actual restoring stages.
- The original signed-bound component, including signed magnitude and minimum laws.
- The new direct-zero component with its real ordinary proofs.
- The original full word component, supplying actual signed operator definitions and Int representations.

Only 12 compiled-in theorem substitutions apply: IsZeroNatural,
DefinedDivisorPositive and ZeroSign at each of the four widths. Both old and
replacement declarations must be theorems with exactly equal normalized
named-DAG types. All common original named types, bodies, proofs, levels and
primitive-family metadata must agree. Selected closed type/value/proof/family
dependencies are rebuilt with explicit remapped names in
`Mpk.Go.Practical.Word.SignedDivision.Prerequisite`.
An iterative declaration order handles long restoring-stage chains.

Every emitted theorem includes its real proof term. Structural fingerprints
guide untrusted construction; both unchanged cores check the entire result.
There are no proof receipts, new axioms, new core rules, caller-provided truth
flags, fuel changes or external import authority. The exact source Word/Int/
Nat definitions retain their original names and bodies. Complete old component
generators retain their original raw hashes. Independent test code compares
every mapped output declaration's full type/body/proof/level/family metadata
with the rewritten original references and checks all 12 theorem interfaces.

All four widths retain unsigned conservation and remainder bounds, actual
signed-magnitude equivalence, actual magnitude divisor transfer and negation
value sum, half bound, both half/minimum directions, magnitude-sign/minimum
correspondence and unsigned-half/sign reflection. Actual SignedPositive/
SignedNegative, SignedDiv/SignedRem and Int representations/projections are
included. No signed quotient or remainder numeric theorem is included yet.

The [verification record](verification.json) binds four current affected roots,
strict [Clippy](current/clippy/result.json),
[Rust format](current/format/result.json) and
[new Go format](current/go-format/result.json).
The [same-byte matrix](paired-final/result.json) checks 33 complete streams in
both unchanged production cores: three zero-axiom acceptances with matching
hashes and 30 actual core type rejections. Controls alter generic positive/
half premises and actual unsigned conservation/guard/remainder, half-bound,
scoped-zero/divisor and magnitude-sign laws at every width.

One accepted ordinary control replaces W8.SignedDiv with a zero-valued function.
Both cores correctly accept it because no theorem in this prerequisite component
links that operator to signed arithmetic yet. The exact owning importer rejects
the changed component. This explicitly records the remaining source-linkage
obligation. Neither acceptance nor finite operator observations imply that the
signed-division feature is complete.

The [Bool-only model](../../../../../../crates/mpk-vc/tests/support/go_word_signed_division_observations.rs)
executes actual SignedDiv/SignedRem/AbsBits, actual zero/divisor guards and the
new scoped ZeroBits over all 96 original full signed-operator input rows.
There are 88 defined outputs, eight actual false divisor guards and eight
minimum/-one observations across division/remainder and all widths. Undefined
rows do not evaluate a word result; their source guards compare to actual
recovered native Go division faults. Minimum/-one division wraps to the
minimum and its remainder is zero, as normal Go behavior.

The [fresh pinned native build](native/result.json) uses an isolated exact
5577-file prepared compiler image with empty cache and complete before/after
name/byte/mode checks. Two outputs are identical across all 128 combined
zero/signed rows. The [independent comparison](native-comparison-final/result.json)
matches both decoded Bool models and an integer oracle that explicitly
computes truncation toward zero, remainder by conservation and machine wrapping.
Original query fields and full words remain exact in the
[query origins](query-origins.json). Nat/Int/theorem bodies do not execute in
the finite model; their included universal prerequisite meanings are checked
separately by the cores.

The original native run record had a copied signed-bound schema label. The
[original record](native/original-run-record.json) remains sealed; the current
record corrects only that annotation. Its source/image/commands/times/outcomes
and raw logs stay exact. The corrected runner uses the proper future label;
the unaffected native execution was not repeated.

The [fixed-source initial construction](joint-prerequisites/rust-construction/result.json)
and [initial Go inspection](joint-prerequisites/go-candidate/result.json) check
the exact final bytes. Captured production inputs stay exact through the
expanded final checks. The [255 frozen inputs](frozen-design-preservation.json)
and official scanner definitions/seals remain unchanged. Original sources and
raw diagnostics are preserved with only exact per-file libtest EOF observations.

Actual signed division/remainder with minimum/-one handling, remaining numeric/
source/carrier/state/program laws, complete bounded 56-family foundation/
descriptor/private registry/context and authoritative original application
source/export/selected-closure replay remain due. All 169 application demands
stay unelaborated. No source-program proof, immutable replay, practical-profile
activation or native Linux release acceptance is claimed. C# tests stay skipped;
the cumulative T02 gate stays deferred to W04: `check-fast.sh --skip-csharp`.
