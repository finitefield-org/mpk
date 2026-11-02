# Original embedded resource capture

The [private original resource reader](../../../../../../go-tools/go2vir/practical_embed.go)
captures the original non-test resources selected by the pinned Go1.25.0
Linux/amd64/cgo-disabled package command. Original Go headers supply the patterns;
the independently reconstructed file list must equal the actual compiler
metadata's `EmbedFiles`. Selected resource identities belong to the complete
original package owner. Replay uses only its sealed directory observations and
original buffers, including after the physical roots have been removed.

Patterns retain native path syntax, glob components, hidden-name and `all:`
behavior, module boundaries, invalid module filenames, directory traversal and
default symlink rejection. No file is fabricated or rewritten. New resources
use the existing auxiliary role; an already captured Go source or module file
keeps its original role when embedded. Unrelated names remain in the original
namespace without being parsed as Go or resources.

All54 complete filesystem cases agree with actual `go list` output:27 accepted
and27 rejected. Both successful and unsuccessful cases retain every original
file byte, link target and exact native stdout/stderr. Accepted cases preserve
the complete sealed packet and per-pattern file lists and replay after removing
their original roots. Cases include binary resources, direct and recursive
patterns, duplicate and overlapping patterns, hidden files/directories, self
source/module embedding, bad native names, nested modules and symlinks.

The actual original package owner captures all5,535 objects, including four
assets with hidden names, nested directories and arbitrary binary bytes.
Returning and changing an inventory cannot change the owner's resource list.
Missing resource buffers, altered bytes and missing observed directories reject.
The Rust consumer imports the complete original capture and compares every
resource object and raw byte. Its one original selected Root contract remains
intact. This byte handoff does not independently derive resource pattern/file
selection in Rust.

All seven actual [Go roots](native/result.json) and28 subtests pass, including
whole-package/export ownership and both original source-invocation units.
All13 affected [Rust roots](rust/result.json) pass, with strict Clippy, Go vet,
explicit changed-file formats and legacy Go compilation with zero executed
tests. Both43-root source units, zero/70-error universes and every byte of32
prior proof streams remain intact. Earlier complete header/auxiliary/build/cgo
and lexer/parser/language/kernel/foundation matrices retain their unchanged
scopes and evidence without repetition.

[Independent inspection](inspection.json) verifies all complete original case
bytes, native output, resource identities, the entire107,062,681-byte owner,
three full pinned Go source witnesses and the old proof streams. Ten source
epochs preserve every consumed input and exact output; formatter mutations
remain distinguished from frozen verification. Prior original snapshot storage
and source buffers retain exact paths/bytes/modes. No source/root/type/error/
proof universe, review rule/seal/exclusion, premise, math rule or kernel budget
is reduced. Host checks do not establish native Linux release resource bounds.

Independent Rust resource selection, canonical module/build/compiler/export/
capture/context/descriptor/registry and launcher authority, source/IR/state/
storage/frame/callee/program proofs, all56 foundation families/all169 demands,
native Linux gates and remaining T02-T11 stay due. R01/R06/W01 remain open;
the practical profile stays inactive. The cumulative T02 gate remains final W04:
`./scripts/check-fast.sh`. Go/Rust only.
