# Original Go package files and import headers

The [package reader](../../../../../../crates/mpk-vc/src/go_practical_type_keys/source_package.rs)
uses the complete immutable native directory view to reconstruct original Go
file membership, package names, import paths, internal/external tests and
cgo-disabled exclusions. Claimed complete-capture packages must contain the
exact derived active Go source IDs and original package name. An actual unused
channel/callback/init-panic source file remains in the complete package and
snapshot; coherently removing it from the claimed package rejects even though
the two selected call-contract candidates still derive.

Header parsing uses a physical prefix view over the complete original source
object and bytes. Source identities, locations and subsequent bodies remain
unchanged. It neither manufactures a replacement Go file nor supplies a
stubbed application. Existing full-source lexing/parsing keeps its original
worker stack and source ownership. Build-comment inspection operates on native
bytes so malformed or non-UTF8 bodies beyond the import header are retained
without being parsed as part of this inventory. Exact pinned Unicode15 graphic
ranges validate import spelling without restricting valid non-ASCII imports.

The first actual native header fixture fails the unchanged link-free-root guard
because macOS exposes its newly created temporary path through a linked parent.
Only that test's own temporary directory is resolved before binding the root.
The first failed suite remains failed: its full-original-universe and actual
owner roots pass. Only the affected header root reruns and passes. The initial
graphic generator command's standard-package lookup failure is also retained;
the exact pinned command succeeds with explicit private cache/offline settings
from the existing Go module. No compiler pin, root guard or source scope changes.

All39 actual native header cases agree, including15 native rejections, valid
non-ASCII/raw/escaped imports, cgo-disabled files, test package selection,
documentation, physical line directives and original unsupported/malformed
bodies. Every package directory containing an active Go source in all three
unchanged complete snapshots agrees:668 pure,668 wide and667 owned, totaling
2,003 directories. Every original source kind and raw buffer remains retained.

All24 affected Rust roots pass. Full original native lexer/parser archives are
restored losslessly and reused:5,853 lexical cases give5,714 acceptances,
139 rejections and7,702,601 exact positive tokens;2,996 syntax cases give2,969
acceptances,27 rejections and5,731,905 exact positive nodes. The100,000-depth
syntax boundary and the original byte-derived token bound remain checked.
The earlier182 build cases/all13,853 Go file candidates,150 language cases,
both complete43-root source units, zero/70-error domains and every byte of32
prior proof streams remain. Strict affected Clippy, Go vet and changed-file
formats pass. [Independent inspection](inspection.json) checks actual source
bytes, package-set coverage, all711 exact graphic ranges and proof retention.

All13 actual source epochs retain complete prelaunch inputs and exact outputs;
formatter mutations are distinguished from frozen verification. Unchanged old
snapshot and lexer/parser archive storage retains all original bytes/modes,
without repacking, rewriting receipts or selecting smoke subsets. New original
fixture/source buffers remain in their complete preserved epoch archives.

This is necessary Go-file/header membership. Auxiliary build inputs, cgo
directive validity and embed selection, canonical module/compiler/export and
capture/context/descriptor/registry authority, launcher roots, full source/IR/
state/storage/frame/callee/program proofs, all56 foundation families/all169
demands, native Linux gates and remaining T02-T11 stay due. R01/R06/W01 remain
open; the practical profile stays inactive. The cumulative T02 gate remains
final W04: `./scripts/check-fast.sh`. Go/Rust only.
