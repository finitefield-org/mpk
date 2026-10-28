# Original live Go export-cache observation

Status: a scoped T02-W01 checkpoint. The practical Go profile remains inactive;
Go and Rust are the supported source languages. The original capture and its
limits remain unchanged when the private test observer is not requested.

The earlier source-root observation covers the original application,
dependency and SDK capabilities but not the separate generated export cache.
Recreating only its selected files would invent the unread cache namespace.
This checkpoint instead opens Go's actual private build-cache directory after
the command has produced its exports and before their capture begins. Rust
then checks the full observed directory entries and original export buffers
twice against that held directory while it is live. Go hands the observer
independent copies; changing those copies does not change the original owner.

The original live capture contains 181 export objects, 128 observed
directories and 959 directory entries. The existing original application and
compiler owner retains all 182 packages, 5,697 source objects and 5,577 SDK
files totaling 166,354,960 bytes. Independent inspection recomputes all
source identities and SDK hashes and checks that the live export snapshot is
the exact export capability in the full owner. A separate Rust integration
test imports the complete original compiler owner and rejects both an export
byte change and an added unread namespace entry after their inputs have been
coherently resealed. The changed packets are retained in `projection-original`.

The full 16,880 original evidence files, their bytes, paths and modes are
retained as lossless archive chunks in `live-original/evidence-storage`.
`restore_evidence.py live-original /absolute/new-directory` reconstructs and
checks all of them; both a separate restore and the integration test restore
passed. The exact capture and compiler inputs are never replaced with stubs.

The scoped validation includes the changed Go capture and affected root/owner
tests with vet, the Linux Rust observer and complete-owner integration test,
Clippy and formatting checks. The complete live test used Linux containers on
the local host; this is not native Linux release acceptance. The scoped review
also reconstructs and scans every stored original archive member under the
unchanged review rules. `./scripts/check-fast.sh` is deferred to final T02-W04
as required for an intermediate W.

The Rust observer exits after its live match. Matching that recorded snapshot
to the complete owner does not retain both source-root and export-root handles
in one bound owner, authorize a production launcher, prove compiler behavior,
or establish an application theorem. Those bindings and the remaining W01 and
T02–T11 work are still required; this checkpoint does not declare W01 complete.
