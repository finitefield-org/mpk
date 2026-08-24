# MPK Policy Evidence Report

## Target and Profiles

- Source language: `rust`
- Semantic profile: `mpk.rust.checked.v0`
- Target: `x86_64-unknown-linux-gnu`
- Pointer width: `64`
- Overflow mode: `checked`
- Panic mode: `abort`
- Package: `payment-policy`
- Crate: `payment_policy`
- Unit kind: `lib`
- Function: `payment_policy::approved_reserve_cents`
- VIR limit profile: `mpk.vir.limits.v0`
- Verification limit profile: `mpk.verify.limits.v0`
- Strategy profile: `payment-policy-rust-alpha`
- Checker profile: `mvp-strict`
- Axiom profile: `mvp-theory`
- Strict: `false`
- Update fixtures: `false`

## Source and Release Identities

- Registry schema: `mpk.release.bundle_registry.v0`
- Registry ID: `mpk.release.registry.v0`
- Registry SHA-256: `bdc7864663877b26345f4edc77e24c2c5a14b1582e19f15e2674ab22024ced98`
- Frontend bundle: `frontend.rust.rust2vir.candidate.v0`
- Frontend name: `rust2vir`
- Frontend version: `0.1.0`
- Frontend binary SHA-256: `60b148614f2a22734b45c8ba0366c94505ea735e82e05f3df7c2b03b3ba2b2c4`
- Subordinate frontend `rust2vir-driver`
  - Version: `0.1.0`
  - Binary SHA-256: `e18ada1ff29d0a9dce87230698cd89d77274633de716559ada1dc34f40e0f3ee`
- Toolchain bundle: `toolchain.rust.nightly-2025-06-01.candidate.v0`
- Toolchain distribution SHA-256: `cdaa0ae4d4f56da86f403d58799fd2298f078b043d8392311487315cbcc2c63f`
- Toolchain executable `cargo`
  - Release: `1.89.0-nightly`
  - Binary SHA-256: `4ab49080934031ce3b87b1a8792e685f99819e8a3f537f110a339d7331f1dcea`
- Toolchain content `native-runtime`
  - Release: `nightly-2025-06-01`
  - Content SHA-256: `0f448df12a3bb58ca6ab51fcee4c470b117ce7072a02b489ab214454f302a479`
- Toolchain content `rust-compiler-runtime`
  - Release: `nightly-2025-06-01`
  - Content SHA-256: `3f61be824744b3ad52281dbebaba6718c10ed6af9a82b936a02419b7f43f5693`
- Toolchain content `rust-target-i686`
  - Release: `nightly-2025-06-01`
  - Content SHA-256: `a1c72b8bdb5dd4d589f386fc0142adee3274ebcb104d69203ad1f4ce5600c5c9`
- Toolchain content `rust-target-x86_64`
  - Release: `nightly-2025-06-01`
  - Content SHA-256: `73019eb46832161dad2e55a17cc044ff4523441643e5bc1b1ab1c68408961956`
- Toolchain executable `rustc`
  - Release: `1.89.0-nightly`
  - Commit hash: `4d08223c054cf5a56d9761ca925fd46ffebe7115`
  - Binary SHA-256: `a7c2179d845e8f40305bace1657b903f10d149cc6d72b0c08ecef75487418922`
- Frontend source manifest SHA-256: `fedfe395014dca9f469e0ac72d2efedf32303f1f6d8bb0bda17a7be12253ebd3`
- Certificate source manifest SHA-256: `3d8631bb654da17e10d6f931cbd2fdac0b4f7af1a9c73583ba48fe5af0a91ee2`
- Input set SHA-256: `f7ebda2f084dc81c781bb3e15cf896ad48c02b9df98877292997cf7be2240db6`
- Source map SHA-256: `3eced9e3b9e453dcdc360ca3d2120f492b75f4a27f81f6a9f40f49a4cf04b226`
- Source IR schema: `mpk.vir.v0`
- Source IR SHA-256: `d39ecdb5f08712e41f5671ee74c1c533c93c49b902041b6984e981e436847a18`
- Source VC schema: `mpk.vc.v1`
- VC SHA-256: `92789f2806ba9d01703214ddfb46ed9049470a65370dd74375a950ee307f6c59`

## Verification Summary

- mpk_verified: `0`
- proof_pending: `3`
- helper_only: `0`
- unsupported: `0`

## Properties

- Property `approved_reserve_cents_callee_panic_free`: The selected function satisfies its callee\_panic\_free verification condition.
  - Status: `proof_pending`
  - Member `payment_policy::approved_reserve_cents#callee_panic_free#000000`
    - Function: `payment_policy::approved_reserve_cents`
    - Kind: `callee_panic_free`
    - Group: `payment_policy::approved_reserve_cents.panic_free`
    - Declaration: `VC.Function.f7061796d656e745f706f6c6963793a3a617070726f7665645f726573657276655f63656e7473.panic_free`
    - Declaration SHA-256: `874cf092185dd22d438599dcba15fba924725b72ac250e888b2a48414ea88d0e`
    - Status: `proof_pending`
    - Evidence kinds: `helper_artifact`
- Property `approved_reserve_cents_callee_precondition`: The selected function satisfies its callee\_precondition verification condition.
  - Status: `proof_pending`
  - Member `payment_policy::approved_reserve_cents#callee_precondition#000000`
    - Function: `payment_policy::approved_reserve_cents`
    - Kind: `callee_precondition`
    - Group: `payment_policy::approved_reserve_cents.contract`
    - Declaration: `VC.Function.f7061796d656e745f706f6c6963793a3a617070726f7665645f726573657276655f63656e7473.contract`
    - Declaration SHA-256: `bf1915da589bb7f3a87a7ac90067c90825d59def71358873ed719640e6db00d8`
    - Status: `proof_pending`
    - Evidence kinds: `helper_artifact`
- Property `approved_reserve_cents_postcondition`: The selected function satisfies its postcondition verification condition.
  - Status: `proof_pending`
  - Member `payment_policy::approved_reserve_cents#postcondition#000000`
    - Function: `payment_policy::approved_reserve_cents`
    - Kind: `postcondition`
    - Group: `payment_policy::approved_reserve_cents.contract`
    - Declaration: `VC.Function.f7061796d656e745f706f6c6963793a3a617070726f7665645f726573657276655f63656e7473.contract`
    - Declaration SHA-256: `bf1915da589bb7f3a87a7ac90067c90825d59def71358873ed719640e6db00d8`
    - Status: `proof_pending`
    - Evidence kinds: `helper_artifact`

## Trusted Evidence

- Candidate certificate: `not_generated`
- Axiom report: `not_generated`
- Checker `rust_fast_kernel`
  - Profile: `mvp-strict`
  - Verdict: `not_run`
  - Certificate IDs: `none`
- Checker `reference_checker`
  - Profile: `mvp-strict`
  - Verdict: `not_run`
  - Certificate IDs: `none`

## Helper Artifacts

- Untrusted helper `source:src/lib.rs`
  - Kind: `source`
  - Normalized path: `src/lib.rs`
  - SHA-256: `c9a9ed6cde367ae274fa4efbdfd639c031d7142ed032a219ccb743dabdfa29e4`
- Untrusted helper `contract:payment_policy::approved_reserve_cents`
  - Kind: `contract`
  - Normalized path: `contracts/insufficient-precondition.json`
  - Schema: `mpk.rust.contract.v0`
  - Raw input SHA-256: `2486b55f838b86ce4701f2d6e9c01ec4f24d917e8e71504b7bd5aa85b50348e2`
  - Function: `payment_policy::approved_reserve_cents`
  - Normalized contract SHA-256: `d03e2ec3a4e802fef1544f7953b1dfdf7f4043ea6bcfae73f9eb78460682394c`
- Untrusted helper `contract:payment_policy::retain_approval`
  - Kind: `contract`
  - Normalized path: `contracts/helper.json`
  - Schema: `mpk.rust.contract.v0`
  - Raw input SHA-256: `f76fb050d91e0669795922f15210abceb8fac19ef0aba14ac53e2f93bab5c6ed`
  - Function: `payment_policy::retain_approval`
  - Normalized contract SHA-256: `c3ff6a4dbbe3c2e91fefe7b6831fc2fcab93836b1b5a516552dceab57ff174ba`
- Untrusted helper `verification_ir`
  - Kind: `verification_ir`
  - Schema: `mpk.vir.v0`
  - SHA-256: `d39ecdb5f08712e41f5671ee74c1c533c93c49b902041b6984e981e436847a18`
- Untrusted helper `vc`
  - Kind: `vc`
  - Schema: `mpk.vc.v1`
  - SHA-256: `92789f2806ba9d01703214ddfb46ed9049470a65370dd74375a950ee307f6c59`

## Reproduction Recipes

- Recipe `scan`; working directory role: `source_root` (the source root)

```sh
mpk policy scan . --language rust --semantic-profile mpk.rust.checked.v0 --require-release-registry-id mpk.release.registry.v0 --require-release-registry-sha256 bdc7864663877b26345f4edc77e24c2c5a14b1582e19f15e2674ab22024ced98 --frontend-bundle frontend.rust.rust2vir.candidate.v0 --toolchain-bundle toolchain.rust.nightly-2025-06-01.candidate.v0 --target x86_64-unknown-linux-gnu --package payment-policy --function payment_policy::approved_reserve_cents --contract contracts/helper.json --contract contracts/insufficient-precondition.json --json-out mpk-reproduction-scan.json
```

- Recipe `verify`; working directory role: `source_root` (the source root)

```sh
mpk policy verify . --language rust --semantic-profile mpk.rust.checked.v0 --require-release-registry-id mpk.release.registry.v0 --require-release-registry-sha256 bdc7864663877b26345f4edc77e24c2c5a14b1582e19f15e2674ab22024ced98 --frontend-bundle frontend.rust.rust2vir.candidate.v0 --toolchain-bundle toolchain.rust.nightly-2025-06-01.candidate.v0 --target x86_64-unknown-linux-gnu --package payment-policy --function payment_policy::approved_reserve_cents --contract contracts/helper.json --contract contracts/insufficient-precondition.json --strategy-profile payment-policy-rust-alpha --checker-profile mvp-strict --axiom-profile mvp-theory --evidence-json mpk-reproduction-evidence.json --evidence-md mpk-reproduction-evidence.md
```

## Trust-Boundary Notes

- Only checker-accepted canonical certificate and theory-certificate bytes are trusted evidence.
- Policy JSON, source text, contracts, VIR, VC, AI analysis, CI status, and this Markdown report are not proof evidence.
