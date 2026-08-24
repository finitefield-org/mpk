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
- Strict: `true`
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
- Frontend source manifest SHA-256: `af5403a4df4c3b546c0519cfef7e8520a86180591a83545b49078e0a41e5ac80`
- Certificate source manifest SHA-256: `28e1e3c3d1b7dc5f567f4198d198bbc43459faca8fc460e796c010fe52e2a323`
- Input set SHA-256: `0dced4e13667b09d7abdd4fc941a84caa9de4ce021bc6e9b2e624d6fc3249683`
- Source map SHA-256: `02c903d2be99b4a57396122b98a838b80cdc4811e0c2fcd816f5b9e6432ab5ff`
- Source IR schema: `mpk.vir.v0`
- Source IR SHA-256: `35787bb66411c8b31bedf5d578538455a2bd52b382546aa023766e22965b150e`
- Source VC schema: `mpk.vc.v1`
- VC SHA-256: `538272ee9bfe5b67ce5e2dab6964a95373e6a82f12572a53707d46bdde9431fb`

## Verification Summary

- mpk_verified: `3`
- proof_pending: `0`
- helper_only: `0`
- unsupported: `0`

## Properties

- Property `approved_reserve_cents_callee_panic_free`: The selected function satisfies its callee\_panic\_free verification condition.
  - Status: `mpk_verified`
  - Member `payment_policy::approved_reserve_cents#callee_panic_free#000000`
    - Function: `payment_policy::approved_reserve_cents`
    - Kind: `callee_panic_free`
    - Group: `payment_policy::approved_reserve_cents.panic_free`
    - Declaration: `VC.Function.f7061796d656e745f706f6c6963793a3a617070726f7665645f726573657276655f63656e7473.panic_free`
    - Declaration SHA-256: `874cf092185dd22d438599dcba15fba924725b72ac250e888b2a48414ea88d0e`
    - Status: `mpk_verified`
    - Evidence kinds: `checked_declaration`
- Property `approved_reserve_cents_callee_precondition`: The selected function satisfies its callee\_precondition verification condition.
  - Status: `mpk_verified`
  - Member `payment_policy::approved_reserve_cents#callee_precondition#000000`
    - Function: `payment_policy::approved_reserve_cents`
    - Kind: `callee_precondition`
    - Group: `payment_policy::approved_reserve_cents.contract`
    - Declaration: `VC.Function.f7061796d656e745f706f6c6963793a3a617070726f7665645f726573657276655f63656e7473.contract`
    - Declaration SHA-256: `bf1915da589bb7f3a87a7ac90067c90825d59def71358873ed719640e6db00d8`
    - Status: `mpk_verified`
    - Evidence kinds: `checked_declaration`
- Property `approved_reserve_cents_postcondition`: The selected function satisfies its postcondition verification condition.
  - Status: `mpk_verified`
  - Member `payment_policy::approved_reserve_cents#postcondition#000000`
    - Function: `payment_policy::approved_reserve_cents`
    - Kind: `postcondition`
    - Group: `payment_policy::approved_reserve_cents.contract`
    - Declaration: `VC.Function.f7061796d656e745f706f6c6963793a3a617070726f7665645f726573657276655f63656e7473.contract`
    - Declaration SHA-256: `bf1915da589bb7f3a87a7ac90067c90825d59def71358873ed719640e6db00d8`
    - Status: `mpk_verified`
    - Evidence kinds: `checked_declaration`

## Trusted Evidence

- Certificate `program`
  - Module: `Policy.Generated`
  - Certificate SHA-256: `7170abd9820b259fadd69533ff58d3e5fe79cc331492d7dbf0e83620e3943ed1`
  - Export SHA-256: `996daea421a83b2137a35413c0af0d296cd86608ec10557f2168dc509fc37406`
  - Axiom report SHA-256: `0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5`
  - Checked declaration `VC.Function.f7061796d656e745f706f6c6963793a3a72657461696e5f617070726f76616c.contract`
    - Declaration SHA-256: `9cef78e5197ed3542ab31ebf3dbb9777bc97918be747cb03bf288adf5350cf2b`
    - Function: `payment_policy::retain_approval`
    - Group: `payment_policy::retain_approval.contract`
    - Group kind: `contract`
  - Checked declaration `VC.Function.f7061796d656e745f706f6c6963793a3a72657461696e5f617070726f76616c.panic_free`
    - Declaration SHA-256: `a90d69a01d2ad91fac87ff686ef02f04798aa33083064ee079fd2a5037285064`
    - Function: `payment_policy::retain_approval`
    - Group: `payment_policy::retain_approval.panic_free`
    - Group kind: `panic_free`
    - Dependency `VC.Function.f7061796d656e745f706f6c6963793a3a72657461696e5f617070726f76616c.contract`: `9cef78e5197ed3542ab31ebf3dbb9777bc97918be747cb03bf288adf5350cf2b`
  - Checked declaration `VC.Function.f7061796d656e745f706f6c6963793a3a617070726f7665645f726573657276655f63656e7473.contract`
    - Declaration SHA-256: `bf1915da589bb7f3a87a7ac90067c90825d59def71358873ed719640e6db00d8`
    - Function: `payment_policy::approved_reserve_cents`
    - Group: `payment_policy::approved_reserve_cents.contract`
    - Group kind: `contract`
    - Dependency `VC.Function.f7061796d656e745f706f6c6963793a3a72657461696e5f617070726f76616c.contract`: `9cef78e5197ed3542ab31ebf3dbb9777bc97918be747cb03bf288adf5350cf2b`
  - Checked declaration `VC.Function.f7061796d656e745f706f6c6963793a3a617070726f7665645f726573657276655f63656e7473.panic_free`
    - Declaration SHA-256: `874cf092185dd22d438599dcba15fba924725b72ac250e888b2a48414ea88d0e`
    - Function: `payment_policy::approved_reserve_cents`
    - Group: `payment_policy::approved_reserve_cents.panic_free`
    - Group kind: `panic_free`
    - Dependency `VC.Function.f7061796d656e745f706f6c6963793a3a617070726f7665645f726573657276655f63656e7473.contract`: `bf1915da589bb7f3a87a7ac90067c90825d59def71358873ed719640e6db00d8`
    - Dependency `VC.Function.f7061796d656e745f706f6c6963793a3a72657461696e5f617070726f76616c.contract`: `9cef78e5197ed3542ab31ebf3dbb9777bc97918be747cb03bf288adf5350cf2b`
    - Dependency `VC.Function.f7061796d656e745f706f6c6963793a3a72657461696e5f617070726f76616c.panic_free`: `a90d69a01d2ad91fac87ff686ef02f04798aa33083064ee079fd2a5037285064`
- Axiom report: `checked`
  - SHA-256: `0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5`
  - Total axiom count: `0`
  - Core axiom count: `0`
  - Builtin theory axiom count: `0`
  - Go semantics axiom count: `0`
  - External axiom count: `0`
- Checker `rust_fast_kernel`
  - Profile: `mvp-strict`
  - Verdict: `accepted`
  - Certificate IDs: `program`
- Checker `reference_checker`
  - Profile: `mvp-strict`
  - Verdict: `accepted`
  - Certificate IDs: `program`

## Helper Artifacts

- Untrusted helper `source:src/lib.rs`
  - Kind: `source`
  - Normalized path: `src/lib.rs`
  - SHA-256: `c9a9ed6cde367ae274fa4efbdfd639c031d7142ed032a219ccb743dabdfa29e4`
- Untrusted helper `contract:payment_policy::approved_reserve_cents`
  - Kind: `contract`
  - Normalized path: `contracts/selected.json`
  - Schema: `mpk.rust.contract.v0`
  - Raw input SHA-256: `13955417c2436e9d313d56c4947e79a38e23acca948f7e5817e6a6cd743b2b27`
  - Function: `payment_policy::approved_reserve_cents`
  - Normalized contract SHA-256: `c96fa7cadc1a6d1f6eabed95d1776649248b200adfc2b4604987e7b9a7cf6e89`
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
  - SHA-256: `35787bb66411c8b31bedf5d578538455a2bd52b382546aa023766e22965b150e`
- Untrusted helper `vc`
  - Kind: `vc`
  - Schema: `mpk.vc.v1`
  - SHA-256: `538272ee9bfe5b67ce5e2dab6964a95373e6a82f12572a53707d46bdde9431fb`

## Reproduction Recipes

- Recipe `scan`; working directory role: `source_root` (the source root)

```sh
mpk policy scan . --language rust --semantic-profile mpk.rust.checked.v0 --require-release-registry-id mpk.release.registry.v0 --require-release-registry-sha256 bdc7864663877b26345f4edc77e24c2c5a14b1582e19f15e2674ab22024ced98 --frontend-bundle frontend.rust.rust2vir.candidate.v0 --toolchain-bundle toolchain.rust.nightly-2025-06-01.candidate.v0 --target x86_64-unknown-linux-gnu --package payment-policy --function payment_policy::approved_reserve_cents --contract contracts/helper.json --contract contracts/selected.json --json-out mpk-reproduction-scan.json
```

- Recipe `verify`; working directory role: `source_root` (the source root)

```sh
mpk policy verify . --language rust --semantic-profile mpk.rust.checked.v0 --require-release-registry-id mpk.release.registry.v0 --require-release-registry-sha256 bdc7864663877b26345f4edc77e24c2c5a14b1582e19f15e2674ab22024ced98 --frontend-bundle frontend.rust.rust2vir.candidate.v0 --toolchain-bundle toolchain.rust.nightly-2025-06-01.candidate.v0 --target x86_64-unknown-linux-gnu --package payment-policy --function payment_policy::approved_reserve_cents --contract contracts/helper.json --contract contracts/selected.json --strategy-profile payment-policy-rust-alpha --checker-profile mvp-strict --axiom-profile mvp-theory --evidence-json mpk-reproduction-evidence.json --evidence-md mpk-reproduction-evidence.md --strict
```

## Trust-Boundary Notes

- Only checker-accepted canonical certificate and theory-certificate bytes are trusted evidence.
- Policy JSON, source text, contracts, VIR, VC, AI analysis, CI status, and this Markdown report are not proof evidence.
