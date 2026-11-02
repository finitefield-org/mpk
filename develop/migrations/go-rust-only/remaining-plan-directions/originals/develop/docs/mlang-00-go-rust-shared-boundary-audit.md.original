# MLANG-00-T03 Go/Rust Shared-Boundary Audit

Status: complete implementation audit and remediation record for
`MLANG-00-T03`.

Prepared: 2026-08-25.

## 1. Scope and authority

This record audits the completed `VIR-01`, `GO-VIR-02`, and
`RUST-03` through `RUST-07-T05` path before successor-contract work starts.
It answers three questions:

1. Are language/profile branches closed over only the two implemented
   profiles, with no implicit semantic default?
2. Can a proof, policy, frontend, or release route select an ambient tool path
   or inject executable behavior through a plugin, callback, or registry?
3. Are the abstractions already shared by Go and Rust sufficient as
   implemented, without adding a future-language placeholder or extension
   hook?

The frozen Go/Rust specifications remain authoritative. This audit does not
activate C#, Java, Dart, TypeScript, or Python, amend a serialized schema, or
select the successor extension mechanism reserved for `MLANG-01`.

## 2. Result

The first pass found one correctness blocker, `MLANG-00-T03-R01`: both
production routes to the independent Go reference checker launched ambient
`go run` from a repository-relative source directory. The installed release
tree contains only `bin/mpk` and the registered frontend/toolchain bundles, so
installed `policy verify` could not satisfy its required dual-checker claim.
The package route also reopened each certificate by pathname after the Rust
checker had accepted it, rather than submitting the retained accepted byte
slice to both implementations.

The bounded serial remediation is complete. The release now embeds one
deterministically rebuilt, static Go reference-checker executable in
`bin/mpk`; runtime executes only those bytes from a sealed anonymous file and
sends the retained certificate bytes over standard input. No checker path,
Go tool, source checkout, registry executable, environment override, callback,
or plugin is selectable. The installed-release fixture proves the full Rust
`policy verify` route with `PATH=/nonexistent`.

The repeated audit has zero open blockers. Existing Go/Rust shared
abstractions are sufficient for their implemented boundary, and no speculative
future-language production or accepted schema value was added.

## 3. Audit method

The audit inspected production Rust and Go sources, build scripts, active
schemas and vectors, release descriptors, and the installed-release fixture.
Generated build trees, vendored library documentation, ordinary unit-test
process launchers, and roadmap prose were classified separately from active
selection surfaces.

The review covered:

- every `source_language` and `semantic_profile` parser, emitter, comparison,
  and language-specific branch;
- policy CLI argument parsing and reproduction-recipe construction;
- release tuple validation, installed-tree resolution, executable inventory,
  immutable snapshotting, and frontend sandbox launch;
- all production `Command::new` and Go `exec.Command` sites, including the
  compiler-driver paths reachable through vendored `x/tools`;
- Rust compiler `Callbacks`, source-loader callbacks, subordinate driver
  launch, and registry executable descriptors;
- future-language literals in active non-documentation code and schema
  vectors; and
- the complete installed Go/Rust policy route under a closed environment.

## 4. Closed-boundary findings

| Surface | Finding | Disposition |
| --- | --- | --- |
| VIR and semantic profiles | `SourceLanguage` and `SemanticProfile` accept only Go/Rust and their exact pairings. Unknown and crossed tuples reject. | Closed; no blocker. |
| Frontend request/response | The caller supplies language, profile, target, registry ID/hash, frontend bundle, and toolchain bundle. Frontend output, VIR, maps, manifests, VC, policy, evidence, and recipes must repeat the same tuple. | Explicit; no default. |
| Go frontend | `go2vir` requires the fixed Go profile and target. Its internal toolchain root must equal the sandbox logical root; its loader uses the registered Go executable with `GOPACKAGESDRIVER=off`, `CGO_ENABLED=0`, and a closed Go environment. | Registered toolchain only. |
| Rust frontend | `rust2vir` requires the fixed Rust profile, registered target, toolchain root, driver path, and matching hashes. The evidence runner supplies those private arguments from immutable selected snapshots. | Registered main/driver/toolchain only. |
| Installed release | The executing image must be the exact link-count-one `bin/mpk`; the registry and complete bundle directory set are validated from a retained release-root descriptor. Executables are the exact inventory paths and bytes. | No adjacent or caller path. |
| Frontend sandbox | Go and Rust executables are materialized only from retained snapshots at closed internal paths. Environment, mount, native-runtime, process, output, and time limits are profile-owned. | No executable injection. |
| Rust compiler callbacks | `PinnedCallbacks` is the one concrete implementation required by the pinned `rustc_driver` API. Its phase transition and retained loader are fixed in `rust2vir`; callers cannot register or replace it. Source-loader “callbacks” are exact compiler reads checked against the captured snapshot. | Compiler API, not a plugin surface. |
| Go `x/tools` launch helpers | The reachable package loader may invoke only the Go command selected inside the registered toolchain closure. External package drivers are disabled, cgo is disabled, and the sandbox `PATH` contains only the toolchain Go directory. Other vendored generic helpers are not MPK extension registrations. | Closed subordinate tool use. |
| Maintenance `git` launch | `policy verify --update-fixtures` uses `git` only to require that an existing output is tracked before explicit fixture replacement. It does not select a frontend, compiler, checker, proof input, profile, or accepted result. Normal verification does not invoke it. | Maintenance-only; not a boundary hook. |
| Vertex credential helper | The optional AI explanation route may launch its explicit `gcloud` credential helper. AI explanation is untrusted helper analysis and cannot affect certificate or policy acceptance. | Outside proof/frontend selection. |
| Reference checker | The initial ambient source/PATH launch was a release correctness defect. | Blocker `R01`, remediated in section 5. |
| Future-language tokens | The only active non-documentation literal found was `typescript` in three negative policy mutation vectors, each asserting unknown-language rejection. No parser, enum, profile, tuple, bundle, selection, evidence producer, or production branch accepts a future language. | Rejection control only. |

## 5. Bounded serial remediation `MLANG-00-T03-R01`

### 5.1 Defect

`program_certificate.rs` and `mpk package verify-certs` used
`Command::new("go")` with `go run ./cmd/mpk-checker-ref` from
`go-tools/mpk-checker-ref`. This had three invalid consequences:

1. required verification depended on ambient `PATH`, a Go installation, and a
   source checkout that are absent from the exact installed release;
2. the executed checker bytes were not bound by the installed release image;
   and
3. package verification reopened a certificate pathname after retaining the
   report from the Rust checker, permitting the two implementations to observe
   different reads.

This defect paused T03. It was not deferred to `MLANG-01` and did not broaden
the task into a generic checker registry.

### 5.2 Remediation contract

The fixed route has these properties:

- `scripts/release_bundles.py` builds `mpk-checker-ref` with the existing
  digest-pinned Go 1.25 image, no network, a closed build environment,
  `CGO_ENABLED=0`, `GOAMD64=v1`, trimmed paths, and no Go build ID;
- the tracked executable asset is a static Linux AMD64 ELF, and release
  `check-go`, `check-all`, and both installed fixtures rebuild it and require
  byte equality before continuing;
- `mpk-cli` embeds those exact bytes. They are not an installed sibling,
  bundle inventory member, registry executable, proof input, or selectable
  path;
- runtime copies them to an anonymous `memfd`, applies immutable size/write
  seals (and the executable seal where the host ABI supports it), and executes
  the retained descriptor through `/proc/self/fd`;
- the child receives only `LANG=C`, `LC_ALL=C`, `TZ=UTC`, and
  `GOMAXPROCS=1`; certificate bytes arrive on standard input; stdout and
  stderr are each bounded to 1 MiB and wall time to 30 seconds; and
- package validation retains the exact certificate bytes accepted by the Rust
  checker and submits that same slice to the reference checker.

The Go checker CLI's `verify -` mode is covered by a direct byte-preservation
test. The package CLI and installed Rust policy fixture both run with
`PATH=/nonexistent`, proving that neither route resolves Go or a checker from
the host.

## 6. Repeated audit after remediation

The remediation introduces one new process boundary, but no new extension
surface:

- its public callable shape accepts only certificate bytes and has no path,
  executable, registry, environment, language, profile, callback, or feature
  parameter;
- its payload is part of the `mpk` build identity and is independently rebuilt
  by the release gate;
- the separate Go process preserves implementation independence from the Rust
  fast kernel while remaining source-free; and
- the active release registry remains unchanged and contains only the existing
  Go/Rust frontend and toolchain bundles.

The future-language literal search is unchanged after remediation: the three
negative TypeScript mutations still reject, and there is no C#, Java, Dart,
TypeScript, or Python accepted value or production path.

## 7. Existing shared abstractions are sufficient

| Shared boundary | Implemented evidence | T03 decision |
| --- | --- | --- |
| VIR | One closed module language/profile/parameter tuple, language-neutral typed operations and CFG, and fail-closed Go/Rust validation. | Sufficient as implemented. |
| Contracts and VC | Language-owned source contract parsing lowers into shared VIR contracts and VC v1; source semantics remain profile-owned. | Sufficient as implemented. |
| Source map and manifests | Shared schemas carry normalized paths, hashes, selection, semantic parameters, target, frontend, toolchain, and release identity for both languages. | Sufficient as implemented. |
| Release runner | One strict registry and immutable-snapshot runner dispatches two closed language branches without a raw public tool path. | Sufficient as implemented. |
| Policy and evidence | One explicit policy CLI and shared v1 schemas preserve the complete Go/Rust tuple and structured recipe. | Sufficient as implemented. |
| Certificate/checkers | Both languages converge on the same source-free Certificate v0 and exact dual-checker byte boundary. | Sufficient after `R01`. |
| AI explanation | Evidence v1 is validated and redacted through one language-neutral untrusted-helper model with closed Go/Rust profile registrations. | Sufficient as implemented. |

No current abstraction was widened for a candidate future language. Whether a
successor uses a revised closed tagged union or a closed hash-pinned semantic
profile registry remains a decision for `MLANG-01-T02` after the C# gap audit.

## 8. Exit evidence

T03 exits with:

- zero open findings in the blocker ledger;
- deterministic checker-asset rebuild equality;
- Go checker, package dual-checker, program-certificate, and installed-release
  policy verification tests passing;
- the installed Rust verification route passing with no host `PATH`;
- the active release registry byte-for-byte unchanged;
- only rejection-vector occurrences of a future-language literal in active
  non-documentation sources; and
- no C#, Java, Dart, TypeScript, or Python production code, profile, tuple,
  bundle, schema branch, or accepted value.

`MLANG-00` is therefore complete. The next task is `MLANG-01-T01`; no C#
production implementation is authorized yet.
