# Original ELF section and compressed-name observations

Status: scoped checks passed. MPK supports Go/Rust only; T02-W01 remains open
and the practical profile remains inactive.

The independent executable reader now follows pinned Go 1.25's complete ELF
section-name pass, including 32/64-bit/both-endian section and compression headers,
large/extended section counts and name indices, signed file offsets/sizes, complete
name termination and later invalid names. Every compressed section header is read
before build-info selection, with the native separate rules for opening data.

ZLIB and Zstandard section-name data retain the exact declared prefix that Go
reads. Ordered frames, skippable frames, zero/nonzero dictionary IDs, known/unknown
frame sizes, checksums, malformed/truncated data and unread suffixes remain
explicit. Native8MiB history and 128 KiB block semantics are retained; advertised
large windows are clamped as Go does. These are existing format rules. Source,
bundle, kernel and certificate budgets remain unchanged; no expanded-section or
iteration-count cap is introduced. The complete 4,320,033-byte tables exercise
streaming through their final original name and suffix.

The [native record](fixtures/native.json) and Rust comparison cover425 complete
cases, retaining all 187 earlier object-layout/text cases and9 actual SDK tools.
All 3 complete original SDK Zstandard streams are observed through the actual
public Go ELF reader, with their original compressed/expanded bytes and expected
hashes. Each complete stream is also read through all four ELF class/endian
variants. The actual nine SDK executables retain their original formats and bytes.
ELF observations provide data-reader evidence; source/program proofs remain due.

The private block decoder retains the complete original [ruzstd 0.9.0 package](dependencies/provenance.json)
and MIT license, with an ordinary std-only dependency-free manifest and a block
facade. Three upstream source files add the facade export and native block/literal/
sequence guards; all other upstream package bytes remain exact. Go frame/prefix/
checksum/history policy is owned by the independent reader, and the Go-derived
parts retain their BSD notices. The [upstream project](https://github.com/KillingSpark/zstd-rs)
and [versioned API](https://docs.rs/ruzstd/0.9.0/ruzstd/) identify the package origin.
This component changes no proof premise, mathematical rule or scanner policy.

All 8 affected Rust roots, both native roots, scoped Clippy, Go vet and changed
formats pass. The 13 source epochs retain 12 passes and the original no_std import
compilation failure; its correction changes no acceptance rule. Full prelaunch
raw hashes/archives, actual commands, failures and raw outputs remain. Later epoch
hash reuse requires identical path/device/inode/size/mtime/ctime/mode after an
initial stable full-byte read; source membership and raw preimages remain complete.

The full original 5577-file/163457770-byte SDK and 182-package/5697-object source/
Root owner still pass. All 30 compiler claims, 3 actual executable, 3 source-role,
4 strict standard-module and original unused-content controls remain selected.
Their original component shape remains1033 sources/181 packages/zero declarations/
one builtin type. No richer source/type/declaration or application theorem is
inferred. Earlier 32 proof streams and 43-root/zero-or70-error universes retain their
original evidence, with no new kernel replay claimed here.

General executable/container/codec edge-case equivalence and full installed
compiler/root/capture/context/descriptor/registry/launcher/read bindings, all 56
foundation families/all 169 source/program demands, native Linux release gates and
remaining T02–T11 stay due. R01/R03/R04/R05/R06/W01 remain open. Final T02-W04 owns
`./scripts/check-fast.sh`; this checkpoint runs affected local checks. See the
[epoch driver](run.py), [complete fixture generator](fixtures.py),
[independent inspector](inspect.py) and [scoped reviewer](review.py).

The [independent build-wiring follow-up](build-wiring/README.md) repairs seven
standalone observer/fuzz locks and includes the complete private codec in source
receipts and the owned mirror. All seven full offline/locked dependency graphs,
the actual pinned local Linux observer build and actual shared-mirror build pass.
Their original lock/collector bytes, failures, commands and outputs remain. This
supplements the425-case/eight-root reader evidence; native Linux release acceptance
and source/program proof obligations remain due.
