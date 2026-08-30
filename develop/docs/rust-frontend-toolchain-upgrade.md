# Rust Frontend Toolchain Upgrade Procedure

Status: reviewed release procedure after `RUST-07-T05`.

The Rust compiler, frontend, build closure, and every artifact they produce are
untrusted helper inputs. Upgrading them does not change Certificate v0, either
source-free checker, the four axiom categories, or the proof trust boundary.
An upgrade is one reviewed transaction; automatic dependency, compiler,
container, or release-bundle upgrades are prohibited.

## 1. Open the reviewed identity change

Start from a clean checkout of the last accepted release. Record the old
descriptor `build_inputs_sha256`, registry digest, frontend/toolchain bundle
digests, and both target-library digests. Assign one reviewer for the compiler
and MIR adapter change and one for the build/runtime/license closure.

Before fetching or generating bytes, update every applicable normative identity
and conformance vector. The review must cover:

- registry origin and lock-source identity;
- nightly date, rustc and Cargo releases and commits, host, components,
  distribution archives, LLVM release, and both registered Rust targets;
- linker, archiver, ranlib, strip tool, linker arguments and configuration,
  development sysroot, native-runtime image and platform digests, loader,
  library closure, minimum kernel, host profile, and runtime-layout profile;
- frontend, fuzz, and cargo-fuzz manifests and locks, every dependency package,
  checksum, feature set, and root, plus the cargo-fuzz tag, commit, tree,
  archive, executable, build arguments, and environment profile;
- frontend/Cargo/rustc argument profiles and the complete allowlisted MIR form
  and checked-assertion adapter inventory; and
- every Rust, LLVM, Ubuntu, cargo-fuzz, libFuzzer, sanitizer, and vendored-crate
  license or notice identity and file.

The owning files include `develop/specs/RUST_SUBSET_V0.md`,
`develop/specs/RELEASE_BUNDLES_V0.md`, their vector sets and vector manifest,
the isolated frontend manifests/locks/toolchain pin, and any MIR adapter golden
owned by the changed compiler API. Do not update only a lockfile or only a
nightly string.

## 2. Replace the tracked build-input transaction

Run the only command authorized to replace the tracked descriptor:

```sh
./scripts/build-release-bundles.sh --update-build-inputs rust
```

Review the complete diff of
`release/build-inputs/rust/build-inputs.json` and its owning specification and
vectors. Check every provenance value, component inventory, dependency graph,
license/notice reference, build argument, and the new content-addressed cache
path key. Unrelated bytes must be unchanged. The ignored cache is never a
review authority.

From the unchanged reviewed descriptor bytes, require:

```sh
./scripts/build-release-bundles.sh --check-build-inputs rust
python3 -m json.tool develop/specs/vectors/rust-build-inputs-v0.json >/dev/null
```

A restored cache is untrusted and receives the same check. A missing cache may
be recreated explicitly with `--provision-build-inputs rust`; verification
never provisions or invokes rustup implicitly.

## 3. Rebuild every registered release root

Follow Execution Rule 11 as one atomic registered update:

```sh
./scripts/build-release-bundles.sh --update all
./scripts/build-release-bundles.sh --check all
cargo build -p mpk-cli
./scripts/check-release-bundles.sh --fixture all
cargo test -p mpk-cli --test frontend_runner
cargo test -p mpk-cli --test rust_frontend_runner
```

Review the complete frontend, subordinate-driver, compiler, toolchain,
native-runtime, target-library, and installed inventory diff and every changed
bundle/registry root. Both Rust target tuples must select the new registered
pair. The removed candidate publication path and candidate commands must remain
unusable; build-only tools, vendor bytes, linker, sysroot, and dependency cache
must not become release bundles.

## 4. Regenerate and review compiler-owned goldens

Use only each fixture owner's explicit update mode. Regenerate every affected
private MIR lowering, public VIR, source map, frontend-stage manifest,
certificate-stage manifest, VC, grouped skeleton, certificate, checker report,
axiom report, policy artifact, corpus index, and release report. Regenerate the
active Rust corpus only through the pinned Linux toolchain:

```sh
MPK_UPDATE_RUST_POSITIVE_CORPUS=1 \
  ./scripts/run-rust2vir-toolchain.sh cargo test --locked --test positive_corpus
```

Review semantic and byte changes independently. Compiler-local IDs, paths,
timestamps, hostnames, and raw diagnostics must not enter public artifacts.
Build-input identity remains release-report provenance only; it must not be
added to `mpk.policy.evidence.v2`, either source-manifest stage, a certificate,
or a checker input.

## 5. Close the transaction

Run the differential, two-clean-build, path, limit, fuzz, obsolete-interface,
checker, and complete release gates:

```sh
sudo ./scripts/check-csharp-frontend.sh
./scripts/check-no-active-gir.sh --strict
sudo ./scripts/check-all.sh
cargo test --workspace
(cd go-tools/go2vir && go test -count=1 ./...)
./scripts/run-rust2vir-toolchain.sh cargo test --locked
python3 scripts/generate-release-report.py --check
git diff --check
```

The aggregate successor gate requires root in the initial cgroup namespace only to
create its fresh delegated cgroup and fixed `noswap` tmpfs backing. It executes
the frontend, compiler, and generated program after entering the registered
user/execution namespaces and setting `no_new_privileges`.

Commit and release only if the complete transaction passes with an empty review
ledger. On any failure, do not publish a partial descriptor, bundle, registry,
or golden set. The last committed descriptor and registered release remain the
sole authority; discard the failed transaction through the normal reviewed
version-control workflow and diagnose before starting a new one.
