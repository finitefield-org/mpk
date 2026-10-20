# Go and Rust language scope v1

Status: normative; user-authorized scope amendment, 2026-10-20.
Vector schema: `mpk.semantic_profile.registry.go_rust.v1`.

MPK MUST support only Go and Rust as program source languages. Java, C#,
and other source languages MUST NOT be registered, compiled as frontend
implementations, exposed through CLI/API dispatch, or required by build,
validation, deployment, examples or documentation. Python and shell remain
repository build/verification tools; browser JavaScript remains application
code. These uses do not add MPK source-language profiles.

The only installed semantic registry is revision 4 of
`mpk.semantic_profile.registry.v1`. Its entries are `mpk.go.fixed.v0` and
`mpk.rust.checked.v0`. Their entry hashes, parameter semantics and nine
contracts are unchanged. Revision-2/3 registries and removed-language contexts
MUST reject. Revision 1 is retained only for explicit Go/Rust conformance and
predecessor artifact validation, never as an installed-registry fallback.

The active bundle registry remains `mpk.release.bundle_registry.v1` with
exactly two frontend/toolchain pairs and three target tuples: Go linux/amd64,
Rust i686-unknown-linux-gnu, and Rust x86_64-unknown-linux-gnu. Registry hashes
MUST be recomputed from canonical bytes after this scope change. Old native
release receipts apply only to their original identities; revision 4 requires
its own native Linux release gate before a new release-acceptance claim.

Practical Go implementation continues in the existing priority order, without
any dependency on another language's release. Its future common-artifact
coverage has four target contexts: retained Go fixed, practical Go, Rust32,
and Rust64. All common artifact members, all Go requirements, full ordinary
source/program semantics, compiler-derived operation/VC roots, paired
source-free checks and native isolation gates remain required. This amendment
activates no practical Go profile and does not lower proof obligations or
resource limits. The kernel caps remain 4,096 declarations, 131,072 terms,
2,097,152 certificate bytes and 1,024 fuel.

This scope overrides earlier references to retaining Java/C# contexts,
resuming their roadmaps or waiting for their releases. Historical migration
records under `develop/migrations` remain evidence of their original epochs;
they provide no executable support or current release acceptance. Historical
frozen inputs are not rewritten or relabeled. Current publication manifests
exclude the seven explicitly retired vector sets, bind revision-4 vectors,
and remove test owners whose implementations have been removed.

During intermediate W items, execute affected local tests, lint/format and
review. At a T's final W run `./scripts/check-fast.sh` over the cumulative
changes. There is no language-specific skip mode. T07/T11 additionally require
`sudo ./scripts/check-go-rust-frontend.sh` and the complete local Linux release
gate described in the root README. No GitHub Actions or workflow is used.

Conformance vector: [semantic-profile-registry-v4.json](vectors/semantic-profile-registry-v4.json).

Manifest ownership: `develop/specs/vectors/semantic-profile-registry-v4.json`
is owned by `crates/mpk-vc/tests/semantic_profile_registry_runtime.rs`.
