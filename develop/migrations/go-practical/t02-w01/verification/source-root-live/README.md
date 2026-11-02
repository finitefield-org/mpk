# Live original application roots: in progress

The existing complete versioned Go application test now has optional, test-only
coordination with an independent Rust observer. Before the Go capture begins,
the observer opens and retains the original application, dependency and SDK
roots. While those original roots are still live, it imports the actual Go
snapshot and checks every captured source byte and observed native directory
entry against the held handles. The original test then proceeds unchanged:
it retires the original roots and reconstructs the complete original package,
export and compiler owners.

The [live comparison and original test](live-original/result.json) pass for
5,516 source objects, 836 directories and 6,370 entries across three held roots.
The [independent inspection](live-original/independent-inspection.json)
recomputes all source identities and establishes that those exact three-root
observations are the corresponding complete part of the later 5,697-object
application/export owner. The remaining 181 objects are original export data;
their root is not authenticated by this comparison. The same inspection rehashes
all 5,577 registered SDK files (166,354,960 bytes), retains all 182 original
packages and the one selected Root function, and confirms zero contract overlays.
No selected source, unselected handler, original dependency or SDK member is
replaced by a stub.

The native test produced 16,880 original evidence files totaling 367,714,473
bytes. Their exact bytes, paths and modes are retained in 24 reversible chunks;
the [storage record](live-original/evidence-storage.json) and independent
[restoration](live-original/restore-check.json) cover every file and directory.
To restore a fresh copy outside the repository:

```sh
python3 -B develop/migrations/go-practical/t02-w01/verification/source-root-live/restore_evidence.py live-original /private/tmp/mpk-restored-live-capture
```

The initial preflight could not mount the absent temporary Go cache, so no Go
test ran in that attempt. The [Go-only retry](preflight-go-retry/result.json)
passes compilation, vet and formatting; the unchanged Rust check and format
passed in the [initial preflight](preflight/result.json). [Linux-target Clippy](clippy-linux-target/result.json)
also passes. Two exploratory commands that forced test flags into dependency
libraries without Cargo's normal test harness failed; their exact diagnostics
remain in `clippy-linux-test-cfg` and `clippy-example-test-harness`. The
[Cargo target observation](cargo-target-observation/result.json) records the
actual example configuration; those experiments are not reported as passing
project tests.

The failed first preflight archive had one incidental text-scanner match in
compressed bytes. The exact archive and finding are stored reversibly in the
[preflight storage record](preflight-archive-storage.json), and the independent
[restore check](preflight-restore-check.json) passes. Scanner rules and
exclusions remain unchanged. The archive can be restored outside the repository:

```sh
python3 -B develop/migrations/go-practical/t02-w01/verification/source-root-live/restore_preflight_archive.py /private/tmp/mpk-restored-live-preflight
```

The first full review scanned the original data but failed while aggregating
its report because two manifests shared a variable name. The
[initial failure](review-initial-error.json) and original driver are retained;
the [corrected review](review-final/result.json) passes for all 16,880 original
files and 837 ZIP members, with no active or archived findings.

These are private test roots supplied by the fixture. They do not authorize a
launcher-selected root, establish source-to-registered-frontend correspondence,
authenticate the separate export root, or prove an application. The Go test
runs under local Linux amd64 emulation and the Rust observer on local Linux
arm64; neither establishes native Linux release acceptance. T02-W01 remains
open, the practical Go profile remains inactive, and `./scripts/check-fast.sh`
is deferred to final T02-W04. Go and Rust remain the only source languages.
