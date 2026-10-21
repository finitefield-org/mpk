The [private original-source closure observer](../../../../../../go-tools/go2vir/practical_closure.go)
reads the intact owned package universe after original source/export/compiler
reconstruction and full Go typechecking. Callers select exact physical original
function/method spans. They supply no AST, typed package, adapter, callback or
unreachability oracle. Original application files, SDK/cache preparation paths
and the image are retired before this consumer runs. Initializers never run.

The fixed point records top-level original function/type/value specifications
and local declarations encountered in selected bodies, with selected-root/body/type/constant/signature/type-check-only
or rejected-dependency roles. It retains deterministic directed dependency edges,
actual call spans, declaration/module/source identities and root-to-issue paths.
All original package Go files remain owned, including unused channel/map/callback
handlers and panic initializers. Explicit queues/stacks traverse dependencies
and call cycles; no helper/root/parameter cap is added. Every branch remains in
the graph. Unknown indirect/interface calls, call cycles, mutable package-state
access, missing bodies and observed unsupported forms produce diagnostics.
Concrete Error method compatibility adds a signature dependency; its body enters
only when selected or called. No arbitrary library-name adapter is introduced.

Unevaluated constant array len/cap arguments retain type/constant dependencies,
without becoming mutable runtime reads. Runtime reads/writes also classify the
actual package variable as a rejected dependency. Grouped and blank constants
retain their original inherited expressions/types. Because native Info.Uses has
one slot for a reused Ident, dependency lookup uses the current real ConstSpec
scope and visibility position; selector fields retain native member identity.
Two actual outer/local-shadow cases verify both constant dependencies.

The [actual integration tests](../../../../../../go-tools/go2vir/practical_closure_test.go)
capture and rebuild a complete two-package application plus unsafe, keep its
original exports and SDK, remove original locations, then inspect31 selected
cases:13 without these metadata diagnostics and18 with required diagnostics.
Native objects and independent physical token positions agree on528 call edges;
1101 dependency edges and23 issues pass inventory/path/cycle checks. A512-function
call chain,70 parameters and70 simultaneous roots preserve the unchanged limits.
Caller root order normalizes deterministically; duplicate/crossed spans reject.
Returned source/graph mutation and closing the owner preserve independent data.
The existing original-owner regression passes all15 crossed-owner controls
(195.6seconds). Existing81 complete source results remain77 typed/3 type errors/
1 syntax error; a separate unused-dependency type error still rejects. Current
Go format and vet pass. Rust, ordinary/kernel and old Go production semantics
are unchanged, so their unrelated semantic tests are not repeated.

The [verification](verification.json) retains15 complete source epochs/12735
preimages and25 actual checks. Two failures remain with original inputs/results:
a plain bodyless fixture was rejected by the real export build; its corrected
original compiler-recognized linkname declaration is explicitly diagnosed. Later,
all29 initial selected cases passed but a test comparison map aliased a result
role slice; copying test evidence fixed that assertion. Available original output
is retained without reconstructing absent output. Postreview changes add exact
inherited-scope dependencies, mutable-variable rejection roles and explicit edges.
Nine lossless transports retain55282 original files/494736364 bytes with a
[literal byte/mode audit](original-byte-audit/result.json). [Review](input-review.json)
uses unchanged127 matchers/four scopes/39 seals/51 exclusions. SDK14496 files,
255 frozen inputs and the old189 frontend inputs remain exact. The current Go
build additionally contains this one private production source file; the old
frontend artifact is preservation evidence, not authority for the new consumer.

Unselected nested declarations remain captured under their enclosing type-check-only
function; separate rows for all such locals remain part of full W02 inventory.

This is R06 preparatory compiler/source dependency metadata. It is not the full
W02 verification-closure algorithm or independent source admission. Rust-side
closure reconstruction/validation, contract dependencies, Root/Plain/closed-error
admission, complete original namespace/capture/nominal/build/context authority,
actual source execution/store/call/loop/state/program proofs, full56 ordinary
foundation/registry and nativeLinux/generalnamespace/all169/T02–T11 remain due.
Go/Rust only; R01/R06/W01 remain open, the public practical profile stays inactive,
and the T02 full gate stays finalW04: `./scripts/check-fast.sh`.

The current190 captured frontend inputs additionally rebuild twice from independent
empty caches with identical8592898-byte outputs (SHA256
`589e82c3c73b95df654501cc71776d9a1a3095371a0ee11170ac2dff4a816eea`).
This is a local Darwin/arm64 build observation; native Linux acceptance remains due.
