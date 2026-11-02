# Exact original observation bytes in private JSON records

Status: scoped checks passed. T02-W01 stays open and the public
practical Go profile remains inactive. The product supports Go and Rust only.

The native observer already bounds stdout at16MiB and stderr at1MiB. Hexadecimal
transport needs32MiB and2MiB respectively, while the common Go JSON codec's1MiB
string limit previously rejected records once a raw stream exceeded512KiB. The
private export/module/graph record path now derives only its root `metadata_hex`
and `diagnostics_hex` limits from those unchanged native byte budgets. It keeps
the existing three schemas, hash domains and exact canonical bytes. Large nested
fields or similarly named ordinary fields cannot use the root exception.

The general Go codec stays unchanged. The general Rust source-family codec also
stays unchanged, including its existing268MiB record/node,64-depth and16MiB
ordinary-string limits. Each implementation retains its predecessor's ordinary
string policy; this change does not lower Rust's ordinary16MiB bound to Go's1MiB.
The Rust observation parser gives only the two named root hex fields their native
byte-derived limits, then uses the same strict shape and domain/hash checks.
Missing/null/non-string, odd/uppercase/non-hex, over-budget, duplicate, malformed
Unicode, fractional/unsafe numeric and trailing-input records reject. Cycle and
invalid-UTF8 programmatic values retain the common Go rejection.

The Go producer's30 exact transport witnesses span all three schemas and both
fields at0,1,512KiB,512KiB+1 and the native byte boundary. Every original binary
stream and full canonical JSON record is retained; Rust compares complete bytes,
canonical encodings and hashes. Boundary probes test transport only and do not
supply source/compiler/context authority. Existing small-payload hashes must agree
with the unchanged common Go encoder. Invalid values cannot widen ordinary limits
or cross schema/domain boundaries.

A separate real native owner retains the full original182-package/5697-object
HTTP application and all original cache/archive/source objects, unused unsupported
bodies and unexecuted init. Its original SDK is relocated to a longer physical
path before validation, without changing its contents; an original dependency
manifest contains15000 duplicate requirement edges. Both original native package
and graph streams must exceed the former JSON ceiling before the source/image
roots are retired and the complete owner is independently imported in Rust. This
is an additional fixture: all earlier original owners/cases remain intact.

Only affected private producer/import consumers, the unchanged shared Go JSON
checks and the new exact boundaries/full owner are selected, with explicit changed
formats, Go vet and strict affected Clippy. The complete original source/package/
proof/error universes and whole pinned SDK inputs are preserved per consuming
source epoch. Earlier proof byte streams retain their prior evidence; this
checkpoint does not claim a new kernel proof replay or installed authority.

General correctness proofs, other record/aggregate/native-read resource and
compiler/root/capture/context/descriptor/registry/launcher/source/program/foundation
obligations, native Linux release gates and T02–T11 remain due. No source/type/
error/proof universe, premise, math rule, scanner policy or kernel/certificate cap
is changed. The cumulative T02 gate stays final W04: `./scripts/check-fast.sh`.
It is not run in this intermediate W.

The initial full-owner attempt stopped before observation because Darwin refused
to move its read-only disposable SDK root. The complete source/output epoch is
retained in `owner/`. The fixture now temporarily gives that root write permission
for the move and reseals it before the normal full compiler validation. No SDK
file contents or final inventory modes are changed, and no authority check is
bypassed. The30 byte-boundary witnesses/common tests and initial production/
integration Clippy had already passed and retain their exact input epochs.

The first independent Rust control comparison rejects the assumption that stderr
has an8MiB budget: the authoritative native runner uses the existing
`maximumContractBytes = 1_048_576`. Go's witnesses already use that exact constant.
The initial Rust30 accepted byte/hash rows agree, but the over-budget diagnostics
control exposes the wrong Rust8MiB assumption. Its complete unit source epoch,
original record and failure remain; the correction must use the actual1MiB raw
stderr budget, preserving the original producer/observer limits.

The second owner attempt preserves original graph630780-byte and package510544-
byte outputs. Its graph crosses the old ceiling, but the package stream remains
below524289 bytes, so the coverage assertion fails. The new fixture retains that
attempt and adds another valid original SDK path component; neither threshold
nor the15000 duplicate edges is reduced. Complete output records are preserved
before the coverage assertion in subsequent attempts.

All15 affected Rust roots and7 native roots pass, with current affected Clippy,
Go vet and explicit changed-file formats. The full actual owner retains182
packages/5697 objects,586034 original package bytes and630780 original graph
bytes. All16 consuming source epochs preserve complete inputs/outputs, including
three failed epochs. Earlier local/versioned original owners and all22 coherent
graph claims plus the coherent source-content checksum control remain covered.
The [verification](verification.json), [inspection](inspection.json) and
[review driver](review.py) record affected-test selection and the deferred T gate.
