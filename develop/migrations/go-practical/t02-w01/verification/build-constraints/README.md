# Independent fixed-target source eligibility

The [source build reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_build.rs)
reconstructs Go1.25.0 Linux/amd64 filename and leading build constraints from
exact original bytes. It fixes gc, no cgo tag, GOAMD64=v1, release tags1.1-1.25
and the seven original target tool tags. Host defaults, ambient experiments and
caller tag adapters do not enter this constructor. Every claimed active Go
source in a complete capture must pass the production eligibility guard.
Excluded originals remain retained in the complete snapshot.

The reader handles all original known OS/architecture suffixes, initial-dot/
underscore names, test files, modern Boolean expressions, legacy AND/OR/negation,
comment placement and BOM/CRLF. Modern expressions share the existing iterative
parser and its exact native1000 operand/group bound. Legacy expressions use
Go's original100 operator bound and preserve native ignored-error behavior.
Complete-capture construction rejects binary-only and test source claims.

The [actual Go oracle](native/result.json) exercises182 boundary cases and every
Go/excluded-source object in all three unchanged complete original snapshots:
4,617 pure,4,617 wide and4,619 owned candidates. The [independent inspection](inspection.json)
checks all13,853 observations, all original source buffers across every kind,
six exact original Go source witnesses and coverage of every18 known OS and24
known architectures from the actual pinned Go source. Seven malformed modern
constraints reject; excessive legacy expressions retain native behavior.

All15 affected [Rust roots](rust/result.json) pass: six snapshot/capture roots,
two full build-oracle comparisons, four original source-input roots and three
file-language roots. Four coherent rebinding controls retain the complete5,490
object owner and both call-contract candidates, while Windows/arm64/test names
and a false Windows header reject at complete-capture construction. Both complete
original source units keep all43 roots, zero/70-error universes and every byte
of32 earlier proof streams. The shared minimum-version parser retains all150
original file-language observations. Clippy, Go vet and changed-file formats pass.

Final review found that the new comparison test depended on another test's
snapshot restoration. The [exact old consumer](old-standalone-consumer.rs)
reproduces a real standalone failure in a complete new workspace with zero raw
snapshots: one root passes and one fails. The [repair](standalone/result.json)
changes only that test reader, restores all77,162 original files/696,793,808 bytes,
and passes both standalone roots at the new absolute path. Every original path,
byte and mode is checked. Both observation files equal the earlier successful
comparisons byte-for-byte. [Affected Clippy/format checks](final-consumer-checks/result.json)
pass; the unchanged13 Rust roots and both Go oracle roots are retained. Original
failure outputs and the complete pre-fix review/document/driver epoch remain.

All nine actual source epochs retain their prelaunch inputs, exact outputs and
frozen source checks. Formatter writes are explicitly distinguished. Complete
new source archives preserve each current/pre-format workspace epoch; unchanged
prior snapshot storage preserves all original buffers without repacking or
rewriting them. Every consumed original path/hash/mode is checked before and
after each relevant worker. No source body, entry/type/error domain, proof byte,
kernel limit, review rule or exclusion is reduced.

This is a necessary filename/header eligibility check. Import/cgo classification,
complete selected package membership, canonical module/build/compiler/export
roles, capture/context/descriptor/registry and launcher authority, source/IR/
state/storage/frame/callee/program proofs, all56 foundation families/all169
demands, native Linux release gates and remaining T02-T11 stay due. R01/R06/W01
remain open; the practical profile stays inactive. The cumulative T02 gate
remains final W04: `./scripts/check-fast.sh`. Go/Rust only.
