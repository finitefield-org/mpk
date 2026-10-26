# Independent original embedded resources

The [Rust resource reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_package/embedded.rs)
derives production patterns from complete original Go files and selects their
resource objects from the immutable captured namespace. The caller cannot
supply a replacement source or pattern list. Complete capture now requires
every resource buffer and every directory observation needed for this selection.
Unknown facts reject without consulting a host filesystem.

The independent byte comment walk follows the pinned Go build reader across
the whole original source. Comment placement, quoted/rune/raw literals, BOM,
physical newlines, carriage returns, Unicode whitespace and malformed argument
lines retain their native build-inventory dispositions. Pattern values retain
arbitrary Go string bytes. Literal invalid UTF-8 follows native Unquote's
RuneError decoding; escaped arbitrary bytes remain unchanged. Original files,
objects and locations are never rewritten. Compiler enforcement of pragma
placement, variable restrictions and malformed lines remains separate.

The matcher follows the native Go1.25.0 chunk search and byte-wise star advance,
including its UTF-8 boundary behavior. Filename validation uses the already
pinned Go Unicode15 letter table and native module filename rules. Original
source/module roles survive embedding; new resource roles remain auxiliary.
Direct/recursive patterns, hidden and all-prefix selection, module boundaries,
regular files and default symlink rejection operate only on captured facts.
A go.mod link requires target/stat evidence; a name/type observation alone
cannot establish its native disposition, so missing facts reject.

Production, internal-test and external-test pattern inventories remain distinct.
Only production resources are requested for normal package capture. An invalid
or absent resource referenced solely by test source cannot introduce a normal
build resource dependency. Original test source bytes and patterns remain owned.

All54 unchanged original native cases agree:27 accepted and27 rejected. The
complete original files, link targets, directory entries and buffers are captured
before resource-selection failure. Every native child stdout/stderr is retained
before assertions or decoding. All54 pragma cases agree, including six native
header rejections and body cases that distinguish escaped arbitrary bytes,
literal invalid UTF-8 and incomplete UTF-8 sequences. Every56 original pinned
path.Match case plus17 actual byte/UTF-8 cases agrees. The original complete
pinned test source is saved before consumption and remains byte-exact.

All668 pure/668 wide/667 owned original package directories agree:2,003 total
with75 original production patterns. Three additional complete native group
cases preserve nonempty internal/external test patterns. Invalid or missing
test-only patterns do not change the one actual production resource selected
by normal go list. The tests include an external test file sorted before the
production file, and retain all original test source and pattern bytes.

The original owner retains all5,535 objects/107,062,681 bytes and four assets.
The Rust reader reconstructs their full per-pattern identities and exact bytes
from original source. Removing an actual resource buffer, or its required
observed directory plus the now unread source, and coherently resealing snapshot/
capture rejects with the identical original source selection and one Root
contract. Every original buffer remains preserved outside these separate controls.

All five actual [Go roots](verification.json),26 affected Rust integration roots
and the native-glob comparison unit pass, with strict affected Clippy, Go vet
and explicit formats. The first25 integration roots and glob unit retain exact
production sources and complete original test-source prefixes when the new group
test is added. [Source correspondence](source-correspondence.json) checks every
240 Rust/187 Go production source. Both43-root units, zero/70-error universes
and all32 old proof streams remain byte-exact. Unchanged full lexer/parser/
build/language/kernel/foundation matrices and original Go owner/export/invocation
production retain their earlier complete evidence without repetition.

[Independent inspection](inspection.json) checks all complete original case
bytes, before-assertion native outputs, resource objects, original package/pattern
sets, entire owner, coherent control transports, primary Go source witnesses and
original proof streams. All22 consumed source epochs and exact outputs remain
complete, distinguishing formatter writes from frozen checks. Prior native buffers
and snapshot storage retain all original paths/bytes/modes. No source, root/type/
error/proof domain, review rule/seal/exclusion, premise, math rule or kernel
limit is reduced. Host checks do not establish native Linux resource bounds.

Canonical module/compiler/export/capture/context/descriptor/registry and launcher/
root authority, required module-link target/stat observations, compiler pragma
validation, source/IR/state/storage/frame/callee/program proofs, all56 foundation
families/all169 demands, native Linux gates and remaining T02-T11 stay due.
R01/R06/W01 remain open; the practical profile stays inactive. The cumulative
T02 gate remains final W04: `./scripts/check-fast.sh`. Go/Rust only.
