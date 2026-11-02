# Original source-root test replay: in progress

The held-root integration test now restores the exact historical frontend tree
from the original analysis build's retained input archive instead of reading
the evolving `go-tools/go2vir` checkout. It verifies the complete archive hash,
then checks every original file's byte length, SHA-256 and mode, together with
the exact set of 328 original files. The test copies the archive and receipt to
a detached temporary repository containing no Go checkout before restoring it.

The [affected Linux tests](detached/result.json) again compare all 213 captured
objects, two capabilities and 373 native entries across 52 directories, including
the same source, namespace, link and inter-pass mutation controls. The earlier
[original replay attempt](original/result.json) also passes; the final detached
attempt additionally proves that restoration does not need the current checkout.
[Linux-target Clippy](clippy-linux-target/result.json) and the changed test's
[format check](format.json) pass. The earlier snapshot importer regression was
unaffected by this test-only correction and its [three original cases](../source-root-matching/snapshot-regression/result.json)
remain recorded.

The existing text scanner reported three incidental matches in compressed
bytes of two earlier analysis-build archives. Both complete compressed byte
streams and the full initial findings are retained as reversible hexadecimal
data in the [storage record](archive-storage.json). The [restoration check](restore-check.json)
recovers both original archives byte-for-byte. Scanner rules and exclusions
remain unchanged. To restore them into a new directory outside the repository:

```sh
python3 -B develop/migrations/go-practical/t02-w01/verification/source-root-replay/restore_archives.py /private/tmp/mpk-recovered-analysis-archives
```

The restored tree and its complete original archive are test fixtures. This
does not authenticate a caller's source root, bind the original capture epoch,
establish registered frontend/source correspondence or prove an application.
The Linux arm64 test and Linux x86-64 cross-target check are not the native
Linux release gate. T02-W01 stays open, the practical Go profile is inactive,
and the cumulative T02 gate remains deferred to T02-W04. Go and Rust remain
the only supported source languages.
