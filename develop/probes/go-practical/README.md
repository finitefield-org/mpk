# Go Practical Feasibility Probes

These are private feasibility/audit tools. They register no profile, add no
public command, and cannot establish a program proof by themselves.

## T01-W01 baseline audit

From the repository root:

```sh
python3 develop/probes/go-practical/entry_audit.py --check
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_entry_audit.py' -v
```

The audit checks the last installed-release receipt against the raw registered
bytes, pins its Go toolchain metadata and specification witnesses, checks the
fixed Go corpus's indexed artifact hashes, and inventories initial consumers
and regression files. It does not execute the Linux compiler or verify all
application theorems. Its historical receipt does not certify later source
changes. The ledger records the separate clean Git baseline check; the audit
does not invoke Git.

`--write` deliberately regenerates the report only while establishing W01.
After production changes, retain the original hashes and compare against a
checkout of the recorded baseline; do not rewrite them to make drift pass.
Tests corrupt receipt state, registry identity, target width, language
versions, artifact digests, negative-case coverage, and duplicate JSON keys.

The proposed effective-language-version table is go1.22–go1.25. T01-W02 must
probe it using the exact Go 1.25.0 analysis release before T01-W04 can freeze
admission. Host Go 1.23.2 regression results are explicitly separate evidence.

## T01-W02 module, version and closure probes

Preparation is explicit and separate from measurement. The pinned host
archive identity comes from the [official Go download metadata](https://go.dev/dl/?mode=json&include=all).
The private host pin is not an installed Linux release tuple.

```sh
python3 -B develop/probes/go-practical/provision_host_toolchain.py
python3 -B develop/probes/go-practical/run_module_probes.py --check
python3 -B develop/probes/go-practical/check_module_report.py
python3 -B -m unittest discover -s develop/probes/go-practical -p 'test_module_report.py' -v
```

The preparation command may download the exact pinned Go 1.25.0 archive,
checks its SHA-256 and every extracted file, and keeps it under `/private/tmp`.
The measurement command performs no downloads. It builds the private analyzer
from the pinned vendored analysis libraries, prepares a synthetic local-file
dependency, seals application/dependency inputs, and evaluates every case
twice from fresh roots with Go downloads disabled. Cases that pass preflight
load the original package; metadata rejections stop before loading. It records 39
cases: 20 closed early-exclusion closures, 17 explicit rejections, and two
original source/module errors.

`--write` updates retained evidence only for an intentional reviewed change.
`--probe --case CASE_ID` runs selected debug cases without overwriting the
complete report. The report checker verifies retained source pins, original
fixture identity, case coverage and evidence scope without repeating Go loads.
Its adversarial tests reject missing cases, changed source, lost selected
bodies, host/target confusion and invented theorem claims.

See the [measured matrix](../../migrations/go-practical/t01-w02/module-matrix.md)
and [snapshot contract](../../migrations/go-practical/t01-w02/offline-snapshot-contract.md).
These are compiler/closure observations, not VIR/VC generation, an application
proof, or an OS-level Linux isolation receipt. W03 owns the next actual
ordinary-encoding and both-checker feasibility work.
