# Sealed package selection and original source type checking

Entry: `ae1e11cb85931d708b35f32b1dd84be0fa77e96d`. This is private T02-W01
compiler preparation. R06 and W01 remain pending.

The [package selector](../../../../../../go-tools/go2vir/practical_package.go)
replays Go1.25.0 package file selection over the complete sealed original
namespace. Every filesystem callback is closed; the Go library cannot invoke
an external module command or fall back to host paths. Target is gc/Linux/
amd64, GOAMD64 v1, cgo off, no user build tags and the exact default target
experiments. Dwarf5 is enabled for this Linux target even on the Darwin
development host. Release/file-name/build/cgo/test selection uses the actual
pinned Go build library. Complete selected, excluded and auxiliary source
identities must agree with their original captured roles. Unread/crossed roles
reject. All raw inputs remain intact.

The [source type checker](../../../../../../go-tools/go2vir/practical_typecheck.go)
uses only captured selected source modules and the exact original go.mod
requirements. It derives relative/absolute local replacement locations from
the existing actual module-discovery facts without rewriting any manifest.
Versioned source locations use the original cache layout; full ZIP/sum/
metadata and every loaded source member are checked again against the owned
original inputs. Full source trees go through actual Go1.25.0 go/types with
gc/amd64 sizes and function-body/unused-import checks enabled. Initializers
are retained and type checked without execution. The same package instance
supplies source imports; no caller-supplied typed-package/signature adapter
can supply dependency facts. Nominal and alias distinctions survive across
packages. Exact module requirements and per-source file versions remain
available as untrusted compiler metadata.

Closed import checking rejects cycles, crossed internal packages, unselected
nested modules, explicit vendor paths and imported program packages. Missing
standard-library source/export input remains `errPracticalNotCaptured`; no
guessed signature or filesystem fallback fills it. The actual Go compiler's
unsafe intrinsic is used only during source type checking. Selected unsafe
operations remain outside the v1 profile. Complete authoritative native module/
package/export replay and standard-library source/export capture remain due.

[The receipt](verification.json) binds24 affected roots/201 subtests in75.81s,
current vet/format and Go1.23 compile/link with zero tests. All81 frozen original
sources reproduce their native outcomes:77 type checked, three original type
errors and one original syntax error. The original unused dependency type error
also fails. A separate actual pinned `go list` result agrees on the full active/
excluded Linux target/cgo/test file sets, then sealed replay succeeds after
removing the original physical inputs. Relative and absolute replacements
type check after original-root removal. A well-typed changed extracted module
source fails its original ZIP content binding. All of these are compiler/input
facts; no profile admission, source/program theorem or application is claimed.

Ninety-one actual runtime fixture records retain declared source/module bytes
and physical roots. Independent reconstruction checks260 complete final source
preimages:81 original cases, six build-selection source objects and173 current
frontend/vendor inputs. Two actual independent empty-cache builds from37
frontend/133 vendor files plus metadata agree on8592898-byte binaries, SHA256
`257d3603a0fa20493b0ba80e2ad5cab731558baac9f9b1c52ba85eadc3dc1d6a`.
All255 frozen members remain exact; original syntax/scalar/checker evidence is
retained without unrelated full-suite reruns.

Three failed source epochs remain complete at the test-command level. The
[initial fixture](initial/regression/result.json) required a Go1.23 dependency
under mainGo1.22, so actual Go refused the read-only module load. The old driver
did not expose nested child stdout/stderr; those initial child bytes are not
claimed retained. A [fresh diagnostic reproduction](fixture-diagnostics/diagnostic/result.json)
retains its own complete generated inputs and actual bounded child outputs,
without relabeling them as the initial physical-root observation. The private
discovery error now owns those bounded outputs and preserves the existing
Error/Unwrap surface. Raising the test main module's requirement makes the
original intended cross-package input valid. The
[third epoch](original-module-language/regression/result.json) correctly rejects
changed ZIP content, while a test expected the wrong diagnostic wording. Its
expectation is corrected without relaxing the product. None is a semantic
source counterexample or a proof-resource verdict.

All actual handles are terminal before staging. Full bounded56-family source/
carrier/nominal/type/operation/storage/lifetime/state/program foundation,
all six common members/19 theorem families, complete compiler-derived operation/
VC roots/private descriptor/registry/context/retained policies, authoritative
immutable original module/package/export/selected-closure replay and all169
application demands/T02-T11 remain required. C# tests stay skipped. The cumulative
T02 gate remains W04: `./scripts/check-fast.sh --skip-csharp`. No public profile,
source/program/application proof or native Linux release claim is made.
