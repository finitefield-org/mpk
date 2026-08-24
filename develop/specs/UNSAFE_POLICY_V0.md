# MPK Unsafe-Code Policy v0

Status: approved governance baseline for the MPK MVP.

## Scope

This specification defines the MVP unsafe-code policy for checker-facing MPK components. It applies to the planned Rust crates `mpk-core`, `mpk-cert`, `mpk-kernel`, `mpk-cli`, Rust theory-certificate checkers, and the independent Go `mpk-checker-ref`.

Non-checker tools such as `go2gir`, `mpk-vc`, and `mpk-api` must not create alternate proof-acceptance paths, but this policy focuses on the kernel and checker code that participates in accepting certificates.

## Core Rule

MVP checker-facing code must be safe by default:

- Rust checker-facing crates must forbid unsafe Rust at crate level.
- The Go reference checker must not import Go's `unsafe` package.
- The Go reference checker must not use cgo.
- Build scripts must not generate unsafe checker code.
- Dependencies that contain unsafe code must not enter the trusted checker path unless explicitly reviewed and isolated outside MVP acceptance.

If a component cannot satisfy this rule, it is not eligible for the MVP trusted checker path.

## Rust Requirements

Each Rust checker-facing crate must include:

```rust
#![forbid(unsafe_code)]
```

at its crate root. The ban covers:

- `unsafe` blocks;
- `unsafe fn`;
- `unsafe impl`;
- `extern` blocks requiring unsafe interaction;
- unsafe generated Rust included into the crate;
- unsafe code hidden behind feature flags.

The crate must not weaken this rule with local `allow(unsafe_code)` or `expect(unsafe_code)` attributes.

## Go Reference-Checker Requirements

The independent Go checker must satisfy:

- no `import "unsafe"`;
- no cgo usage;
- no generated Go file that imports `unsafe`;
- no build tag that selects an unsafe checker implementation.

CI must run with cgo disabled for the reference-checker gate:

```sh
CGO_ENABLED=0 go test ./...
```

## Dependency Requirements

Dependencies used by checker-facing code must be treated as part of the security review surface. During MVP:

- checker-facing crates should prefer dependencies that do not use unsafe code;
- any dependency with unsafe code must be documented in a dependency exception list before it can be used;
- the exception must explain why the dependency is outside the proof-acceptance path or why it is unavoidable;
- the exception must name an owner, audit notes, and a removal or containment plan;
- dependency unsafe use must not bypass canonical decoding, source-free checking, deterministic fuel, or axiom reporting.

No dependency exception is approved except those listed below.

### Approved dependency exceptions

- Dependency: `mpk-linux-sandbox`, on the Linux-only path from `mpk-cli`.
  Owner: MPK Rust sandbox boundary maintainers (RUST-07). Scope: only the
  reviewed `clone3(CLONE_INTO_CGROUP | CLONE_PIDFD | CLONE_CLEAR_SIGHAND)`,
  pidfd signal/wait,
  signal-state, pipe/descriptor, working-directory/process-group, exact-rlimit,
  `close_range`, and `execveat` adapter. Stable safe Rust process APIs cannot
  create a task atomically in a delegated cgroup; attaching after `fork`
  permits pre-attachment kernel and memory charges. This adapter is outside
  certificate decoding, canonicalization, kernel checking, deterministic
  fuel, axiom reporting, and proof acceptance. It may only launch or terminate
  the release frontend and return owned process streams to caller-enforced
  bounded capture or a closed failure.
  `mpk-cli` continues to forbid unsafe code; the helper exposes only typed
  owned descriptors and process handles, denies `unsafe_op_in_unsafe_fn`,
  documents every unsafe precondition, and has no checker/proof-crate
  dependency. Syscall-result, malformed-input, descriptor-closure, rlimit,
  full clone-ABI/flag, invalid-cgroup rejection, pidfd kill/reap, and cleanup
  tests cover the helper boundary. The crate is Linux-only and `publish=false`;
  only `mpk-cli` may depend on its typed executable/cgroup/cwd/argv/environment
  launcher surface, solely for the frozen release-frontend contract.
  Maintainers must enforce that containment in workspace dependency review,
  minimize the crate to the required syscalls, and replace it when a safe
  standard or library API provides atomic cgroup placement and pidfd process
  control. Any API, dependency, or scope expansion, or any failed
  sandbox/unsafe audit, blocks release.

## Build Script And Code Generation Requirements

Build scripts and code generators are untrusted helper tools. They must not be able to smuggle unsafe code into checker-facing crates.

Checker-facing crates must reject or fail CI when:

- generated Rust contains `unsafe`;
- generated Go imports `unsafe`;
- a `build.rs` emits checker code that is not checked into reviewable source form;
- feature flags switch from safe checker code to unsafe checker code;
- generated bindings or FFI are required for proof acceptance.

## CI Enforcement

The first CI gate that creates checker-facing crates must enforce the unsafe ban mechanically.

For Rust checker-facing crates, CI must include an equivalent of:

```sh
cargo check --all-targets --all-features
```

and each checker-facing crate root must contain `#![forbid(unsafe_code)]`. CI must also search checker-facing Rust sources for local attempts to weaken the ban. The gate fails if this search prints any match:

```sh
rg -n 'allow\\(unsafe_code\\)|expect\\(unsafe_code\\)' crates
```

For the Go reference checker, CI must include:

```sh
CGO_ENABLED=0 go test ./...
CGO_ENABLED=0 go list -deps ./... | rg '^unsafe$'
rg -n '\"unsafe\"' go-tools/mpk-checker-ref --glob '*.go'
```

The gate fails if `unsafe` appears in the dependency list or Go source search. The exact script names are defined later by CI tasks, but the enforcement must be automated before any checker-facing code is release-gated.

## Security Review Requirements

Any proposal to introduce unsafe code into a checker-facing component is outside MVP scope unless a governance review updates this policy. The review must state:

- why safe Rust or safe Go is insufficient;
- which component would contain unsafe code;
- whether the unsafe code is inside or outside the proof-acceptance path;
- how the unsafe surface is isolated;
- how fuzzing, negative fixtures, and checker-agreement tests cover it;
- which release gate blocks it until approval;
- how it will be removed or minimized.

Until the policy is revised, the default decision is reject.

## Release Gate

Before release:

1. every Rust checker-facing crate root contains `#![forbid(unsafe_code)]`;
2. no checker-facing Rust crate weakens the ban with local attributes;
3. reference-checker tests run with `CGO_ENABLED=0`;
4. the Go reference checker does not import `unsafe`;
5. no generated checker code contains unsafe Rust or Go `unsafe`;
6. dependency exceptions, if any, are documented and reviewed;
7. malformed input and fuzz tests cannot rely on unsafe-code behavior for rejection.

Failure of any item blocks release.
