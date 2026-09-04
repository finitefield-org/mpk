//! Active successor release-bundle contract.
//!
//! The implementation lives in `mpk-vc` so the build-time identity pin and
//! every runtime consumer execute the same validator.

pub use mpk_vc::release_bundle_v1::*;

// Candidate-only v2 contract. No installed resolver imports these symbols.
#[doc(hidden)]
pub use mpk_vc::csharp_practical_release::{
    build_private_successor_release_fixture, validate_private_successor_release_registry,
    PrivateReleaseBundleInput, PrivateReleaseCode, PrivateReleaseError, PrivateReleaseMemberInput,
    PrivateReleasePhase, ValidatedPrivateReleaseRegistry, PRIVATE_BUNDLE_CONTENT_HASH_DOMAIN,
    PRIVATE_BUNDLE_INVENTORY_SCHEMA, PRIVATE_FRONTEND_BUNDLE_SCHEMA,
    PRIVATE_RELEASE_REGISTRY_HASH_DOMAIN, PRIVATE_RELEASE_REGISTRY_ID,
    PRIVATE_RELEASE_REGISTRY_SCHEMA, PRIVATE_RELEASE_TUPLE_COUNT, PRIVATE_RELEASE_WORK_ITEM,
    PRIVATE_TOOLCHAIN_BUNDLE_SCHEMA,
};
