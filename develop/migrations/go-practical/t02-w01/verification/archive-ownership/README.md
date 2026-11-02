# Original module archive ownership

Status: scoped checks passed. Final source/result bindings and all related
actual-handle completion are recorded in the current receipt. Entry is `7154ec0b2ac2ab56ac21e87e018d2f2c02545366`.
W01 remains In progress. The [receipt](verification.json) binds the original
failures, current sources, affected checks and complete current build inputs.

The private capture session originally returned its cached archive's mutable
member map and source-byte allocation on both the initial parse and cache hits.
Changing the returned map let a fresh cached handle authorize changed dependency
content despite the unchanged original ZIP and Go checksum. Deleting a member
changed the cached namespace; changing returned raw bytes reached the original
captured input. This is an actual private returned-handle ownership failure.

The [production change](../../../../../../go-tools/go2vir/practical_archive.go)
copies the complete raw source buffer and complete member map separately for
the cache and every returned value. Identity, raw bytes, h1, CRC, parser,
namespace rules and original budget charges are unchanged. The
[regression](../../../../../../go-tools/go2vir/practical_archive_ownership_test.go)
checks initial/cache-hit handles crossed with member-map/raw-byte mutations,
then recaptures, rejects changed member content, compares the complete original
inventory and source object, checks unchanged counters and seals the original ZIP.

Both [first](original/regression/result.json) and
[fully logged](reviewed/regression/result.json) pre-fix runs fail all four
subcases. Their original production/test sources, runner and raw output remain.
The logged epoch preserves every complete actual ZIP/main metadata/changed-member
payload before the fix. The [independent input observation](observed-inputs/result.json)
compares all four pre-fix and four passing payloads, reconstructs the complete
source-object identity and computes the original Go h1 from both full original
members. Exact original inputs and altered member bytes remain owned verbatim.

The [selected Go checks](final/selected/result.json) pass14 affected roots and45
subtests in54.17s: all three rooted archive roots, four versioned module snapshot
roots, five original module-resolution roots and both analysis capture/rebuild
roots. These exercise the changed archive boundary and its actual consumers.
Unchanged pure ZIP/hash/native-oracle and scalar/checker suites retain their
prior evidence without repetition. Current [vet](final/vet/result.json) and
[format](final/format/result.json) pass. The
[Go1.23 check](final/legacy/result.json) compiles and links with `-run ^$`;
it executes zero tests and supplies no legacy rooted-runtime evidence.

Because the compiled production input changed, the actual frontend is rebuilt
twice from distinct roots with independent empty caches. Both complete
8,592,898-byte outputs have SHA256
`7a8a86f713dbc1e0098db6af12f5b24134f5603466e59dfc9cf25edabea2142a`.
The [build receipt](final/selected/reproduced-builds.json) and
[independent source observation](build-input-observation.json) bind all170
sealed inputs:34 production frontend files,133 vendor files and the original
manifests. Every complete source-object identity is reconstructed independently;
all original bytes are copied and retained. The unchanged auxiliary vendor TODO
length difference remains recorded with both original and actual bytes.
No registered practical context or Linux release acceptance is claimed.

The first independent observer incorrectly assumed each Go JSON Output fragment
contained a complete long log line. Its [failed source and output](observer-failures/result.json)
remain. The repaired observer concatenates every original fragment in order
before inspecting complete lines; no source, input or JSON is normalized or
excluded. All255 [frozen inputs](frozen-design-preservation.json) remain exact.

Every archive and related product validation handle is terminal. The
[initial scoped receipt](verification-before-final-product-consolidation.json)
retains its original Go-pending observation epoch; current finalization preserves
those exact initial bytes and binds the completed product checkpoint. Final
changed-input review and staged/committed-blob audits cover both changes together. No source/JSON/directory exclusion or checker/axiom/fuel/cap change
is introduced. C# tests stay skipped. The cumulative T02 gate remains W04:
`./scripts/check-fast.sh --skip-csharp`.

Archive ownership protects captured-input integrity. Full56-family foundation,
source/carrier/nominal/type/operation/storage/lifetime/state/program bindings,
descriptor/private registry/context and retained-policy delegation remain due.
Original package/export/selected-closure loading and authoritative immutable
application replay remain due. All169 application demands and T02-T11 stay open;
R06 remains pending and the public practical profile stays inactive.
