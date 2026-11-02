# Complete original-source value relations

This is an intermediate GO-PRAC-T02-W01 checkpoint on the Go/Rust-only
implementation branch, based on `7ead1ea45e602338b52a0d9ddf6d30db92782432`.
R01/R06/W01 and T02 remain open. No public practical Go profile activates.

## Checked mathematical scope

The source-only catalogue derives a closed relation from every value type in
one immutable original-source domain. Word relations observe every bit;
array relations observe every encoded address, including padding; strings
observe the complete length and all 256 encoded bytes. Slice relations retain
nil, natural-number owner, all 16 path entries and their full kind/index,
offset, length and capacity. Structs retain every ordered field, including
zero-size carriers. Nominal types delegate only to their original underlying
source type. Error relations retain the full 64-bit tag and every same-U
payload, including inactive fields.

Each source type has a checked identity view, closed relation, observations,
reflexivity, symmetry, transitivity, equality lift and zero law. Nonempty
product shapes also have checked whole-field assembly and each projection.
These neutral-value lemmas let concrete rows discharge each complete field
separately within the unchanged core fuel limit. Each row proves whole-value
copy, repeated copy and nil relations. Each concrete case additionally proves
all box fields, box preservation and recovery of the active payload relation.
There is no function extensionality or axiom.

This full encoding relation is not Go's `==`, logical slice/string equality,
valid source input, execution semantics or source-to-IR correspondence. In
particular, observing encoded padding and ghost owners must not be advertised
as Go comparison behavior. Prior source-specific carrier/zero/boxing/nil/copy
identities are preserved. Unchanged call-site dispatch, receiver and
representation producers retain their own complete libraries; they are not
copied into an unrelated source-only row.

## Verification and retained failures

[verification.json](verification.json) records actual terminal executions and
both-checker comparisons. The affected normal matrix has nil-only, small
source, all 70 concrete large cases plus nil, and all six heterogeneous source
payload types plus nil. The heterogeneous fixture comes from the unchanged
original-Go corpus in the preceding representation checkpoint. Exact import
also rejects a different case's otherwise accepted row.

The independent observer reconstructs every closed definition and complete
statement, and checks actual decoded zero values against a change at every
encoded scalar leaf in each unit's nil row. These observations supplement the
full Rust and Go kernel checks; they do not replace proof checking.

Negative controls retain genuine false byte, bool, copy, box, active payload,
string-byte, slice-owner and last-large inactive-field obligations. A coherent
constant byte view can remain valid mathematics; exact import and the
independent original-source identity-body comparison must reject it.

The initial producer linkage failures are preserved with complete prelaunch
inputs. Product projections were missing from the dependency closure for a
nil-only source containing string. The three original heterogeneous failures hit
unchanged core fuel during whole-product copy assembly. The diagnostic prefix
search identifies the first failing declaration; its shortened prefixes are
only localization observations, never accepted source-U foundation evidence.
The full original rejected streams remain unchanged. Neutral whole-field
assembly/projection lemmas and removal of unnecessary whole-product identity
views address concrete expansion without dropping fields or raising a resource
limit. The nil-only error has a one-part product represented by its vector
itself, so it retains the function adapter. This last fix is tested in an
isolated complete source copy while the original large source epoch remains
fixed. The [adapter audit](adapter-choice-audit.json) reconstructs all 100
source-type choices: only the nil-only error changes. A fresh complete-library
and boundary-row byte replay records the unaffected large/heterogeneous
producer outputs separately. No old execution is relabeled as the new one. The first small run without the unary-product adapter and the dependent first
control root retain their actual failures. The latter lacked a positive entry
fixture; the heterogeneous and large control roots passed in that same run.
After fixing the small producer, only the first failed control root reruns.
All nine complete control streams are paired with their individual Rust
reports, preserving the failed overall suite status.

The first independent observer failed because a
local relation expression shadowed the raw byte buffer; its original driver
and actual output remain beside the corrected run.

All verification is local and affected in scope. The unchanged old source
producers do not require another complete call-site matrix. No Actions or
other-language tests run. The cumulative T02 gate remains final W04:
`./scripts/check-fast.sh`. Native Linux release/resource acceptance, full
source/input validity, callee/result/effect/frame and program/context binding,
the complete foundation/private registry and T02–T11 work remain due.
