# T02-W01 matched analysis-root owner

Status: private component; W01 remains in progress. The Linux-only owner
retains the independent Rust analysis-source input together with the already
opened and compared source-root handles. It accepts only when the entire
source snapshot is the same: capabilities, native directory-entry bytes/types,
source identities, original byte buffers and canonical snapshot all agree.
The owner does not authorize launcher-selected roots, authenticate a capture
epoch, prove an analysis binary's source or activate a practical profile.

The test restores all 269 original registered source-epoch files and uses the
complete original 196-object snapshot with 184 build inputs. It also imports
the later complete 213-object analysis generation with 201 build inputs. The
historical owner and its independently opened roots bind; a crossed later
owner rejects. A changed original source byte with recomputed object and
snapshot identities matches its modified test root but rejects against the
unchanged original owner. The same holds for an added unread root entry and
its coherently recomputed snapshot. No original source universe is replaced or
pruned; all changes occur only in a separately restored test copy.

The affected Linux integration test passes on the full inputs, including the
cross-generation and two coherent-change controls. Linux-target Clippy and
changed-file Rust formatting pass. The first Clippy attempt found an unused
`PathBuf` import in the new test; its failure and the successful test/lint/
format reruns after removing only that import are retained. The unchanged Go
producer and existing source/root consumers are covered by the preceding
registered-source and held-root checks; no unrelated suites were repeated.
The first scoped review found six incidental text matches in compressed input
archive bytes. All six full streams and the exact failed review are stored
losslessly and independently restored; original receipts retain their
capture-time paths. Scanner rules and exclusions are unchanged.
The corrected review checks 8,820 archived files and 42,024 ZIP members with
no active or archived findings, using the unchanged scanner and all six
exactly recovered original archives.
Local Linux arm64 execution and x86-64 cross-target lint are not native Linux
release acceptance. Trusted launcher authorization, registered frontend/source
binding, the separate application export root, context and program proofs
remain due. The whole T02 `./scripts/check-fast.sh` gate remains deferred to
final W04.
