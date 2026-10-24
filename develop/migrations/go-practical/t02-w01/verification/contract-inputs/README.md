# Owned original source and contract inputs

This private handoff carries the source-only package component and every raw
overlay in the same owned captured inventory. Its envelope binds the original
image/compiler/main/requested-package provenance, exact source-component hash
and the complete ordered overlay object list. It does not accept a caller's
replacement annotation list or infer missing annotations from active Go files.
The existing source-only component and its type seed remain unchanged.

The Go producer holds the original-package owner lock for the entire handoff.
It checks the saved inventory digest and complete list/buffer/path counts,
reads each original overlay through the owned source reader and returns
independent buffers. Closed, incomplete or damaged owners reject; failures in
the owned input reader remain sticky. Unselected annotations remain captured
raw inputs. No application function or initializer executes.

The independent Rust reader requires exact closed JSON fields and hashes, the
complete source-only component, every raw overlay listed in the envelope and
exact object identities. Missing, extra, duplicate, changed or reordered objects and
case-colliding paths reject. Contract bytes, captured bytes and object counts
retain the existing bounds. Contract derivation uses only that owned source/
annotation set and the requested original function selectors. No sidecar means
the existing explicit default contract, with no manufactured overlay object.

The provenance digests provide input linkage; a self-hashed packet is not
launcher authentication. A coherently changed candidate envelope is not an
authenticated claim about the original capture. Complete canonical capture/context/descriptor/registry
integration and source/IR/state/storage/frame/callee/program proofs remain due.
This handoff does not activate a practical profile, accept an application,
change a frozen schema/member, introduce an axiom or change a checker cap.

The first native attempt retains its complete source inputs, raw originals and
actual outputs. Its complete-sidecar ownership root passed; two later roots
stopped because their evidence paths reused an existing no-overwrite filename.
The corrected tests use separate evidence directories. All original failed
outputs remain, without relabeling the failed epoch. The later inventory
integrity check also covers omission from the saved list/buffer/path tables.

Verification is selected for the changed producer, strict importer and direct
source-input/contract-module consumers, with Go vet, Rust Clippy, changed-file
format checks and a Go1.23 zero-test compilation. The unchanged mathematical
libraries, proof kernels and unrelated frontends are not rerun. This is an
intermediate T02-W01 checkpoint: R01/R06/W01 and remaining T02-T11 work stay
open. All56 foundation families, all169 program demands and native Linux release
gates remain required. The cumulative T02 gate is deferred to final W04:
`./scripts/check-fast.sh`. Go and Rust remain the only source languages.

[Verification](verification.json) records14 affected Rust roots and6 actual Go
roots, all passing. [Independent inspection](inspection/result.json) validates
both complete original source/contract packets,10971 original objects, all raw
identities and exact native package/overlay membership. The initial and final
complete source-component/overlay bytes match. The existing32 call-proof
certificates replay literally through the source-input regression consumer.
All four saved-inventory mutation cases reject with sticky errors. The exact
8MiB contract-byte boundary is accepted at the input stage; one extra byte
rejects. This boundary observation does not admit duplicate function annotations
at the later contract-module stage.

[Lossless snapshot storage](snapshot-storage/manifest.json) retains all54856
original failed/final files,435872068 bytes, with their exact paths and modes.
The original local copies remain physically unchanged; only duplicate copies
are ignored by Git. All member bytes are scanned under their original virtual
paths. Restore them for independent inspection with
`python3 -B develop/migrations/go-practical/t02-w01/verification/contract-inputs/preserve-originals.py --restore`.
Normal Rust tests read the small tracked original packets directly.
[Input review](input-review.json) keeps the existing127 rules/four scopes/
39 seals/51 exclusions. Original producer stdout/stderr retain their exact
bytes, including final blank lines; whitespace checks apply to all other files.

All15 original compressed prelaunch containers,321970696 bytes, are also
[retained exactly](archive-storage/manifest.json) in reversible tracked chunks;
the duplicate local `inputs.tar.gz` copies remain physically unchanged.
`preserve-transports.py --restore` restores each exact local compressed copy.
The first review's40 matches were solely in compressed bytes; its
[complete original report](review-compressed-binary/original-report.json)
remains failed. Every decoded source/snapshot member remains scanned at its
original virtual path. No matcher, seal, exclusion or input scope changes.
The first review-import failure also retains its
[actual driver and diagnosis](review-first-attempt/observed-result.json).
