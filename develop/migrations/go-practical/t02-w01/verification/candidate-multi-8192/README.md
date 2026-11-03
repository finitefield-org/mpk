# 8,192-root multi-package candidate boundary

The Go capture test retains the complete original HTTP application, its
dependency cache and the registered 5,577-file Go SDK. It appends functions to
the original application and a second source package and selects 8,192 actual
functions across three packages, including one original dependency. All 183
packages, 5,517 source objects and 182 compiler exports remain in the capture.
The test independently checks every selected function's physical span in the
original Go AST and rejects omission of a requested root. It also constructs the
original module-graph inputs after retiring the replay image.

`TestPracticalCandidateMultiPackageKeeps8192OriginalRoots` passed locally with
the pinned Go 1.25 Linux/amd64 SDK under emulation. This targets the changed
multi-package handoff at the existing root limit while retaining the original
application, compiler and full dependency closure. The Rust reader has
independently bound all 8,192 requested roots to that capture; contract
candidate derivation also passed in two local implementation experiments:
sharing immutable source scopes alone, and sharing them together with an
original-syntax constant index. Both derived 8,192 functions and 8,192
expression nodes. The stronger test also matches each generated function to
its original selected root and signature and checks every default contract.
The four-root original capture and coherent replacement/omission controls
passed with the combined experiment. Targeted Rust tests for source scopes,
constants and contextual `iota`, source types and signatures, body typing,
selection, overlays, dispatch and contract modules passed, as did changed-file
formatting and Clippy. They cover the shared source owner and the readers that
derive facts from it. These implementation changes are not yet committed, and
the earlier unmodified Rust derivation is still running. No conclusion about
source or program proof is asserted by this boundary check.
Rust tests ran locally on macOS/arm64; neither environment is the native Linux
release gate.

To reproduce the cross-language check, run
`TestPracticalCandidateMultiPackageKeeps8192OriginalRoots` with an empty
absolute `MPK_GO_PRACTICAL_SYNTAX_EVIDENCE` directory and the registered
Linux/amd64 SDK. Point `MPK_GO_CANDIDATE_MULTI_INPUTS` at its
`versioned-owned` directory and explicitly run the ignored Rust test
`complete_8192_root_multi_package_capture_binds_independently_in_rust`:

```sh
cargo test --locked --offline -p mpk-vc --test go_practical_candidate_multi_inputs \
  complete_8192_root_multi_package_capture_binds_independently_in_rust -- --ignored --exact
```

This is not a test of 8,192 distinct packages or operating-system argument
limits. Exact literal `...` package paths, production batching, launcher
authority and source/program proofs remain open. W01 and R06 remain incomplete;
the T02-wide `./scripts/check-fast.sh` gate is deferred to T02-W04. No
validation archive is committed.
