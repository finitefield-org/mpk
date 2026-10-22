# Ordinary tagged error-value observations

Entry: `ab353e38981c28949f3d09a6f4b54099c1ed06bc`.
This is a Go/Rust-only T02-W01 prerequisite checkpoint. R01, R06 and W01 remain
open; the public practical profile remains inactive.

The [ordinary definitions](../../../../../../crates/mpk-vc/src/go_practical_ordinary/error_value.rs)
pair a complete Word64 tag with an arbitrary Sort0 payload `P`. `Make`, `Nil`,
tag/payload projections, copying, nil/non-nil tests and case comparison are
ordinary terms. A payload may contain the full product of case payloads;
copying retains that entire argument. `Nil P zero` retains the supplied `zero`;
this component does not establish that the supplied argument is a source type's
canonical zero.

The pair is an **observation view**. The frozen carrier balances the complete
list `[Word64, payload_0, ..., payload_n]`, whose shape generally differs from
`Product Word64 P`. A checked adapter to that exact layout, source universe/tag
assignment, canonical inactive fields and full error-value validity remain due.
No frozen carrier encoding is changed or replaced by this view.

Nine definitions and90 laws cover full-tag reflexivity, mismatch at each of64
positions, constructor projections/comparisons, nil, nonzero tags, a concrete
tag-one value with any supplied zero payload, copying and arbitrary tag-address
observations. The nonzero laws require an actual checked nonzero witness.
Each bit-mismatch law likewise requires the actual differing-bit witness.
Conjunction helpers propagate that witness through the whole balanced64-bit
comparison. A high bit is never narrowed to a Boolean or low-bit tag.

All arbitrary payload equality is definitional preservation of the original
argument. Copy reconstruction is pointwise at the outer product. No function
extensionality or arbitrary payload equality is assumed. Closed computations
use the existing ordinary Eq congruence/transitivity proof recipes to rewrite
computed Bool majors explicitly; neither checker receives a new reduction rule.

| Component | Declarations | Terms | Bytes | Axioms | Imports |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tagged error observations |2050 |33772 |471001 |0 |0 |

The component retains every named type/body/proof/level/family identity of the
entire1892-declaration Word library and94-declaration Product.Relation library.
Fresh original generators are pinned to their complete raw hashes
`23c88fb260bf9d27c2defb64a892f275653cd1741ed030f00a911fed1b2aa11c`
and `584cbf2e2a70ce0d86f7703b754e9a22fd84dcd59ffd42e6df00e22fdb2a0d1d`.
Exact component import regenerates the complete bytes. An honestly checked
extension, trailing bytes and over-budget buffers fail exact ownership.

The [verification receipt](verification.json) binds four new Rust roots, eight
affected old roots, strict scoped Clippy and formatting. The unchanged1640-query
word-operation matrix is retained without a repeat because no word operation
changed. Current tests still pin the old complete bytes and every named identity.
No frontend or unrelated semantic suite is rerun.

[Both unchanged cores](final-paired/result.json) inspect the same169 complete
library streams: two accept with zero axioms and identical logical hashes;
167 reject through genuine core type mismatch. Controls cover all90 laws,
all69 required premises, five altered definitions and three false concrete
claims. Canonical-codec rejection or resource failure supplies no negative
evidence. These169 library checks are distinct from the still-pending169
application demands.

[Finite execution of the actual certificate DAG](final/certificates/finite-observations.json)
adds528 observations: zero, the all-ones tag and every single-bit tag, all four
two-field Boolean payloads, original/copied values, nil/non-nil, whole tag,
both payload fields and exact case matches. It takes5457888 interpreter steps.
This is finite observation evidence; the universal claims come from ordinary
checked terms, and no source Go correspondence is inferred from this matrix.

All original compiler inputs, drivers, raw outputs and failed candidates remain
retained. The first insufficient projection recipe exhausts conversion fuel in
[Rust](first/result.json) and [Go](first-go-observation/result.json). Diagnostic
epochs preserve subsequent nil-alias/computed-major failures; their individual
prefix streams were not separately persisted and are only reconstructible from
the retained full stream, source and driver. Their reports are diagnostic only.
The [first full-suite epoch](current/result.json) also retains malformed test
declaration names and their canonical-codec failures; corrected final controls
must genuinely type-reject. No failed epoch is counted as proof acceptance or a
semantic negative. The initial successful whole component and final component
bytes/reports are literally equal.

The [first scoped review](input-review-first/result.json) observed its own
not-yet-created report link. Its report and unchanged driver remain retained;
the subsequent review sees the actual saved report without an exception.

The [scoped original-input review](input-review.json) uses the unchanged127
matchers, four scopes,39 seals and51 existing exclusions. All255 frozen inputs
remain exact. No scanner exception, kernel/evaluator change, axiom/import rule,
or resource limit is added. Limits remain4096 declarations,131072 terms,
2097152 bytes,1024 conversion fuel, and50M interpreter steps/2M values.

Exact frozen layout/zero/validity adapters, actual source universe and dispatch,
receiver-once/non-nil/precondition/effect/result proofs, backing store and
frame/lifetime/state/program correspondence remain due. Complete56-family
foundation/private descriptor/registry/context, canonical capture/module roles
and scope/contract wire, native Linux and T02-T11 remain open. Earlier unchanged
historical-scope scanner findings still require resolution at the final gate.
The cumulative T02 gate stays deferred to final W04: `./scripts/check-fast.sh`.
