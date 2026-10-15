# GO-PRAC T01-W02 Offline Snapshot and Closure Contract

Status: candidate implementation contract, 2026-10-16. This is a private
feasibility handoff to T01-W04 and T02, not a frozen profile or public input
schema. The [probe matrix](../../../probes/go-practical/module-cases.json) and
[measured report](module-probes.json) give the compiler observations behind it.

## 1. Exact context and evidence scope

Analysis is compiled and run with Go 1.25.0, using the already vendored
`golang.org/x/tools` v0.36.0, `x/mod` v0.27.0, and their closed vendor inventory.
The measured compiler host is darwin/arm64; the analyzed application target is
linux/amd64, GOAMD64 v1, int/pointer width 64. The runner verifies the complete
host distribution against its official archive SHA-256, membership, file
digests and executable modes. The bundled specification matches the witness
in the registered Linux Go 1.25.0 toolchain.

This is evidence about public Go loader/type APIs and closure construction.
It is not a native Linux runner receipt, VIR/VC acceptance, source/model
equivalence proof, or application theorem. `closed` means the probe found a
direct body closure within its explicitly documented early exclusions; it
does not mean the complete practical profile admits or proves the function.

## 2. Snapshot inputs

Before a production load, the input inventory must contain all of:

| Input family | Required identity and capture rule |
| --- | --- |
| Selection and overlay | Exact package/function IDs; typed contracts; target, compiler, language-version and build-tag selection; raw canonical configuration bytes |
| Main module | Original `go.mod`, optional nonempty `go.sum`, module root capability, and original source tree; no rewritten/stub package |
| Resolved graph | Go's selected module versions and replacements, all consulted module manifests, relevant lock/metadata bytes, and the exact module instance behind each imported package |
| Dependency sources | Versioned module ZIP/content identities and explicit local-replacement roots; retain original module/package paths and source content |
| Build selection | Compiled and excluded source file identities, filename/GOOS/GOARCH constraints, build expressions, effective per-file versions, assets/auxiliary files consulted by the compiler |
| Compiler and libraries | Registered target compiler/tool inventory, standard library source/export provenance and required native closure; no ambient installation fallback |
| Analysis dependencies | Closed frontend/analysis-library inventory; compiler-produced caches are derived helpers, not independent semantic authority |

Every inventory member has its kind, normalized relative identity, byte
length and digest. Referenced local replacement roots must be inside an
explicit capture capability; paths and symlinks must not escape it. A nested
module is a separate resolved module instance, not implicitly the main module.
Unknown inputs or a changed namespace/content during capture reject before
downstream artifact publication. T02 owns immutable-handle/no-follow capture
and TOCTOU protection; ordinary read-only permissions in these probes are
not proof of that production isolation.

The probe records actual compiled/excluded/auxiliary input digests and module
manifests, plus the entire pinned compiler and vendor inventories. This
establishes which input classes the production capture must account for.
Go assembly may exist outside proof and still be a build input. A reachable
body supplied only by assembly or an opaque object rejects without a checked
semantic adapter; hashing its file does not verify its behavior. Embedded
assets and directives likewise require an explicit frozen admission rule;
their discovery alone does not authorize them.

Do not trust `go.sum` or an existing host module cache as a substitute for a
snapshot inventory. Production preparation checks the selected archive and
Go module checksum identities, records their provenance, and binds every
consulted byte. The verification process receives sealed prepopulated inputs.
The synthetic probe dependency has a fixed ZIP and recorded Go `h1` sums;
it is provisioned from a local file proxy and is never an external service.

## 3. Resolution and invocation

Separate provisioning from measurement/verification. The production path
must have no module, VCS, checksum-server or toolchain download capability.
Use the pinned compiler, `GOTOOLCHAIN=local`, `GOENV=off`, `GOWORK=off`,
`GOPROXY=off`, explicit caches and target settings, and a closed process
environment. No caller Go flags, executable overrides or package patterns
can replace that configuration.

The measured loader uses `-mod=readonly`, explicit tags and `-trimpath`,
`Tests=false`, and `packages.Load` name/files/imports/dependencies/modules/
syntax/types/type-info/type-sizes modes. `go list -m -json all` supplies the
actual resolved graph; MPK must not implement a subtly different version
selection algorithm. `require`, `exclude`, `retract`, versioned replacements,
captured local replacements and a satisfiable pinned `toolchain` directive
retain their original bytes and resolution effects. Missing offline inputs
fail; verification never repairs them or rewrites `go.mod`/`go.sum`.

Root workspaces and vendor mode are rejected in the initial scope. Ancestor
workspaces are disabled by the environment. Arbitrary build hooks, cgo and
unmodeled source/compiler directives remain outside the initial production
profile. `go:generate` is never executed. Existing generated Go files can be
captured as ordinary source. Any additional go.mod directive needs a frozen
disposition in W04; the probe's ability to parse it is insufficient.

The macOS runner disables Go proxy/checksum/toolchain/telemetry transports.
It does not claim an OS-level egress-isolation receipt. The production Linux
registered runner still has to supply the real process/filesystem/network
isolation required by the active release contracts.

## 4. Language and compiler versions

The proposed effective language table is go1.22, go1.23, go1.24 and go1.25.
Retain the raw module/file version and independently derive its language
minor with `go/version.Lang`. Go 1.25.0's `types.Info.FileVersions` can contain
`go1.25.0`; comparing it literally with `go1.25` would reject a supported
language version. Conversely, normalizing away the patch requirement before
checking the toolchain would incorrectly accept `go 1.25.1` under Go 1.25.0.

Check the full module/toolchain requirement against the exact compiler first.
Then apply the effective per-file language table to every body entering proof.
A `go1.21` build expression may lower a selected file's language version even
when its module says `go 1.25.0`; the probe observes and rejects that case.
Type-check-only dependencies can use versions accepted by the pinned compiler,
but entering their bodies into proof also requires an admitted effective
language version. No host-default or nearest-version substitution is allowed.

## 5. Two closures and declaration accounting

The capture/type-check closure is the compiler's original package and import
graph. It includes required metadata and all consulted build inputs even if
the selected function never calls their code. Ordinary syntax/type errors in
an unused function still fail type checking. Unsupported but valid code may
remain type-check-only; removing that function to manufacture a successful
replacement package is forbidden.

The verification closure is the fixed point of the selected root's direct
concrete calls, nominal type/constant dependencies, required signatures and
contract/effect dependencies. Keep the call graph acyclic initially. A direct
captured external Go helper can enter this closure; an unmodeled standard
library call, assembly body, callback or interface dispatch cannot. A claimed
unreachable branch is still traversed until a checked proof authorizes its
exclusion. Retain a deterministic root-to-finding path.

Record every selected/dependent/type-check-only declaration in the production
manifest. Multiple `init` declarations need distinct source identities; they
must not overwrite each other in a map keyed only by `package.init`. Nominal
dependencies are bound to the resolved module instance as well as package and
declaration. Aliases preserve their actual Go identity.

An application-owned value error may require `Error() string` as a type
signature dependency without calling the method. The probe demonstrates that
its body can stay outside the selected closure. Closure construction must not
silently turn interface conversion into execution or trusted error-message
evaluation.

## 6. Entry state and ambient effects

The theorem starts at function entry, after any application initialization.
MPK never runs application `init` or generation while loading source. Unused
initializers, handlers, generics or mutable declarations do not themselves
justify including their bodies in proof. Reaching a package variable from
the selected function rejects; an initializer is not an excuse to model a
mutable global as a constant.

Future slice/map/pointer contracts explicitly state borrowed/exclusive
regions and the absence of concurrent external interference. That entry
assumption does not prove the application's startup, database, HTTP handler,
race freedom, or eventual invocation of the function. Any reachable ambient
effect or unknown call must be rejected or modeled and proved under the
frozen profile; it cannot be dropped from the closure.

## 7. Replay and handoff

Each matrix case is evaluated twice from fresh original-source roots.
Preflight rejections stop before loading; the other cases load the original
package. Normalize
only disposable absolute prefixes in observations; preserve raw input bytes,
module IDs, source identities, effective versions, dispositions and findings.
Both normalized observations must match. Source and dependency bytes are
inventoried before/after, and all measurement loads use the sealed cache with
downloads disabled. The probe never truncates or patches an application file.

The retained report, source/case/runner hashes and toolchain/vendor inventory
hashes are the W02 evidence. W04 turns the candidate rules into exact closed
schemas, limits, diagnostics and vectors. T02 implements the real snapshot,
original-package loading, closure manifest and independent import consumers.
W03, rather than this compiler probe, owns the next proof-encoding and
both-checker feasibility work.
