# Independent original module data

The [Rust module-data reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/module_observations/data.rs)
consumes the original owned module observations and complete source/export
capture. It has no source/manifest/member adapters, resolver, host filesystem,
network or checksum-database callback. It parses every line in the original
main go.sum, including unused entries and identical duplicates, and checks
canonical module path/version pairs and canonical SHA256 h1 spelling.

Every selected versioned dependency or replacement needs its original cached
manifest, info, ziphash and entire ZIP. The manifest h1 is independently derived
from the original go.mod bytes. The ZIP reader checks every original header and namespace, applies Go’s length/CRC
rules and hashes complete member contents before deriving
the sorted Hash1 summary. Captured expanded source files must match their
corresponding original archive members. At this checkpoint only the original
info Version spelling was checked. The later [module info checkpoint](../module-info/README.md)
also checks Time, Origin and native metadata spelling. Legacy ZIPs without go.mod retain the
separate cached manifest; no manifest is inserted into an archive.

The [ZIP reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/module_observations/data/archive.rs)
retains Go1.25 central-directory, ZIP64, self-extracting prefix, descriptor,
directory and duplicate/root-header dispositions. Hash1 retains original
member names and uses the original last-name lookup for repeated headers.
Unicode simple-fold collisions use the complete pinned Go Unicode15 table,
while module filenames retain the existing independent Go Unicode15 letter
rules. ZIP filesystem attributes confer no filesystem-link authority: no
archive member is extracted into a loader filesystem.

DEFLATE content is processed through fixed output chunks; the whole expansion
is hashed without allocating a buffer for the entire expanded archive. The
existing500 MiB expanded-module limit,16 MiB manifest/license limit and131,072
namespace-entry limit remain unchanged. Implicit parents, repeated headers and
the existing captured namespace consume the applicable entry budget. Original
ZIP encoding and all cached/source buffers remain retained.

The [native Go oracle](../../../../../../go-tools/go2vir/practical_module_data_oracle_test.go)
retains1,291 complete sum inputs,28 initial whole ZIP inputs plus11 additional
compatibility inputs,1,454 nonidentity fold pairs
and the exact generated Rust table. The full scalar domain is represented by
those pairs and identity elsewhere. Cases include the actual500 MiB expansion,
131,072 namespace entries, a one-entry-short rejection, ZIP64, full opaque
assets, duplicate directories and unsafe/corrupt namespaces or bodies. All
unchanged native archive/sum tests run alongside this additional oracle.

The [failed-control analysis](failed-control-analysis.json) retains the complete
original ZIP, the exact original first-bit mutation and their complete decoded
member bytes. That mutation preserves the original content and is a valid
encoding variant. It remains a positive control; the added midpoint mutation
changes actual member contents. The original failing assertion and complete
source epoch remain preserved. Production validation is unchanged. Every
coherently changed whole snapshot/capture/export/module transport is retained
before the final assertion, including all original source selection and Root
contract identities.

Complete cached sources for miniz_oxide0.8.9,adler2 2.0.1 and base64 0.22.1 are
retained with their original distribution archives. Every source member equals
its archive bytes; each complete archive SHA256 matches Cargo.lock. Resolution
uses only the local Cargo cache. Existing dependency versions remain unchanged.
The initial lockfile, generated table and formatter preimages remain preserved
as their own source epochs.

All14 native Go roots,29 affected Rust integration roots and the whole-scalar
Rust unit pass. Every one of the1,291 sum/39 ZIP verdicts and accepted inventories
agrees with pinned Go, including full protocol boundaries and all1,112,064
Unicode scalars. Eleven coherently resealed controls preserve the same original
source selection and one Root contract: eight content/missing-data cases reject
and three duplicate/encoding/comment variants accept. Existing complete source/
export/resource consumers, both43-root units, zero/70-error universes and all32
original proof streams remain intact. Strict affected Clippy, Go vet and formats
pass. Only the changed control and previously unrun consumers repeat after its
expectation fix; earlier passed roots retain their exact evidence.

[Independent inspection](inspection.json) recomputes accepted ZIP Hash1 streams
with Python’s separate ZIP/DEFLATE implementations, verifies every original raw
module source buffer, cached dependency archive/source and Cargo.lock digest,
all transported control identities, the original failure reconstruction and
complete old proof bytes. All24 consumed source epochs preserve full original
inputs and exact outputs, including both failures and formatter/lock mutations.
The [verification record](verification.json) distinguishes passed scope from
remaining module/context/program and release obligations. No review rule, seal,
exclusion, premise, math rule, source/type/error/proof domain or kernel cap changes.
Host checks make no native Linux release resource claim.

These checks establish source/archive/checksum consistency. They do not
authenticate a checksum database, capability roots, installed compiler/context
or general MVS/version selection. Complete module directive/identity semantics,
compiler/root/capture/context/descriptor/registry/launcher authority, source/
IR/state/storage/frame/callee/program proofs, all56 foundation families/all169
demands, native Linux release gates and remaining T02–T11 stay due. R01/R04/R05/
R06/W01 remain open and the practical profile stays inactive. Go/Rust only.
The cumulative T02 gate remains final W04: `./scripts/check-fast.sh`.

Primary references are the complete pinned Go1.25 source witnesses in primary/,
the exact [Go Hash1 implementation](https://pkg.go.dev/golang.org/x/mod/sumdb/dirhash#Hash1),
and the complete retained [miniz_oxide0.8.9 source](https://docs.rs/miniz_oxide/0.8.9/miniz_oxide/inflate/stream/index.html).
