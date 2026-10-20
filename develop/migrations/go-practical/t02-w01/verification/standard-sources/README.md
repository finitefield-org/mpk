# Original standard sources and compiler export correspondence

Entry: 3029c13a2eb6b1f8969521fcd023e73dddc39b77. This is private
T02-W01 compiler preparation; R06 and W01 remain pending.

The [SDK source collector](../../../../../../go-tools/go2vir/practical_stdlib.go)
captures every original source member of the prepared Go1.25.0 image. It
compares complete bytes, lengths and source roles to the original compiler
inventory, checks the original std module manifest and rejects missing, extra,
altered or relabeled members. The shared rooted reader classifies each final
source role from the same full original read. It does not relabel a previously
captured object or race a second physical read. SDK vendor resolution and
internal-package ownership follow the original compiler source namespace.

The [source checker](../../../../../../go-tools/go2vir/practical_typecheck.go)
types complete original application and SDK bodies from sealed snapshots using
the actual Go1.25 library. A fixture imports strings, fmt and net/http and keeps
an unselected HTTP handler. All178 packages, including the application, type
check after the application/cache roots are removed and both compiler-root
paths are changed to a nonexistent path. Their names, original selected files
and vendor import maps agree with actual pinned Go. The native closure has179
packages: unsafe is the actual compiler intrinsic, with no source AST or export
adapter substituted for it. Selected unsafe remains outside the v1 profile.
The complete syntax image contains1472407 nodes within its unchanged bound.

The [export collector](../../../../../../go-tools/go2vir/practical_exports.go)
runs the pinned compiler with offline/no-VCS/no-toolchain-fallback settings and
retains its full list/deps/export/compiled observation. Its fresh private cache
is physically resolved before generation; caller roots retain their strict
link-free path requirements. All178 original compiler archives,79390348 bytes,
are captured before that cache is removed. Correspondence checks selected-root,
compiler/module ownership, full original source lists, compiled filenames,
original import order/maps, complete package closure and each archive's own
target/header/build ID. SDK vendor rewriting preserves original import order.
Archive bytes provide compiler preparation; export signatures do not replace
whole original source typing or establish source semantics.

[The independent receipt](verification.json) checks every retained SourceObject
preimage, full original native metadata, package/import/source correspondence,
all archive members and own build IDs. All51 affected roots/325 subtests pass
in142.19s, including14 export alterations and five SDK controls. Current vet/
format pass. Go1.23 compiles and links with zero tests. The changed common
source read warrants capture/snapshot/module/archive consumers; source typing
and new production files warrant package/analysis consumers. Unrelated scalar,
Rust, kernel and full-Go suites are retained without repetition.

Two independent empty-cache frontend builds from175 captured inputs,39 frontend/
133 vendor files plus three metadata files, agree on8592898 bytes and SHA256
23b866a6f56459c2461993a97f7d37d4899e69dee68af3ff1ccc320671e15ce9.
All255 frozen members remain exact.

All original producer byte buffers, including arbitrary binary SDK assets and
compiler archives, are retained as lossless JSON byte strings alongside their
original SourceObjects. Transfer receipts retain original producer paths,
raw hashes/lengths and byte-exact round trips. Original SDK trailing whitespace
and blank endings remain in decoded bytes; they are not normalized. Independent
strict review scans every stored file and every decoded payload using all127
unchanged rules/four scopes/39 seals. No source, JSON, directory or blanket
whitespace exclusion is introduced.

Three failed export epochs remain explicit. In the
[first](original-exports/regression/result.json), native generation succeeded
but strict capture rejected the private cache's /var symlink spelling. Full
native output/error and original test/application inputs remain; the original
archive payloads were not exposed before cleanup and are unavailable. The
[second](physical-cache/regression/result.json) incorrectly required newline
padding rather than Go's zero padding. All5482 input objects and native output
remain, but the initially read archive payload was not exposed before cleanup
and remains unavailable. Later fresh archive bytes are not substituted for
either original observation. The [third](original-archive-padding/regression/result.json)
retains all5660 original objects, including all178 archives; it exposed the
incorrect import sorting after vendor resolution. The correction preserves
actual source and native order. These are implementation failures, not semantic
source counterexamples or proof-resource verdicts. Earlier simple strings/fmt
and HTTP source-only epochs retain their own original input/source/output sets.

All actual handles are terminal before staging. Complete original export/
compiler ownership and authoritative immutable native module/package/export/
selected-closure replay remain due. Full56-family source/carrier/nominal/type/
operation/storage/lifetime/state/program foundation, all six common members/
19 theorem families, compiler-derived complete operation/VC roots/private
descriptor/registry/context/retained policies and all169 demands/T02-T11 remain
required. C# tests stay skipped; the T02 gate remains W04:
./scripts/check-fast.sh --skip-csharp. No public profile, source/program/
application proof or native Linux release claim is made.

Strict review initially detected an obsolete identifier spelling used only as
a diagnostic metadata key. The current test uses original_application_bytes,
and its complete affected SDK/export fixture plus metadata roots pass again:
three roots/30 subtests in54.78s. Production bytes and every original Go body
are unchanged; the unaffected51-root results remain exact. Earlier full
producer source/JSON files and the initial strict failure remain recoverable
through [their byte manifest](historical-producer-bytes/manifest.json), with
original paths/hashes and no fresh observation substitution. The official
strict guard scans every actual stored file unchanged. Additional decoded
historical-data observations remain explicit in the review receipt; these
inert original byte records are not current source or activated helper kinds.
No matcher, scanner fixture, historical scanner seal or file/directory exclusion
is added or changed.
