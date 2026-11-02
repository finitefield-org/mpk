# Go/Rust-only scope closure

The executable pipeline and installed revision-4 registries already support
Go and Rust only. This checkpoint removes remaining current documentation
that directed future Java/C#/other-language implementation or release work.
The project charter, executive summary, Rust handoff, milestones, release
gates and semantic-registry admission rules now follow the
[language scope](../../../specs/LANGUAGE_SUPPORT_SCOPE_V1.md) and existing Go
priority. Dedicated candidate-language research pages are retirement notices.
Python/shell build tools and browser application code remain ordinary tools
and application code, not MPK source-language profiles.

[Complete previous bytes](originals.json) retain their original modes and
identities. Historical records provide no current implementation, resumption
task or release acceptance. No production compiler/checker, profile entry, registry hash or fixture is
changed. The reference index and Rust upgrade commands now cover the supported
Go/Rust tools only.

[Documentation review](documentation-review.json) confirms all25 current
Markdown links resolve. The [publication check](publication-check/result.json)
reconstructs the Go/Rust-only 25-set manifest and inactive Go publications,
preserving all255 frozen W04 pins. The existing installed-registry test also
passes: it checks the exact Go/Rust entries and rejects removed/unknown
profiles and a predecessor revision. Its full actual command and output are
in the [targeted registry run](../../go-practical/t02-w01/verification/source-string-lengths/registry/result.json).
No removed-language test is run.

The [retired replay inventory](retired-checker-replay.json) removes127 tagged
C# ordinary-foundation-only checker replay roots and their private helper.
The generic certificate corpus and report/process protocol tests remain.
[Two targeted protocol roots](../../go-practical/t02-w01/verification/source-string-lengths/checker-protocol-fixed/result.json)
and [Go formatting](../../go-practical/t02-w01/verification/source-string-lengths/checker-format-fixed/result.json)
pass after removing an unused import. The initial compile failure remains
with complete prelaunch source and actual output; no retired replay executes.

This is an intermediate T02-W01 checkpoint. Its affected validation is
separate from `./scripts/check-fast.sh`, deferred to final T02-W04. It claims
neither practical-Go profile activation nor native Linux release acceptance.
