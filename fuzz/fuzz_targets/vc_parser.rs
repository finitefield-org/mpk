#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use mpk_vc::{
    generate_vc_v1_from_context, import_vc_skeleton_v1_json, import_vc_v1_json,
    ValidatedVcCertificateSkeleton, ValidatedVcDocument, VcSkeletonValidationError,
    VcSourceContext, VcValidationError,
};

const MAX_FUZZ_INPUT: usize = 1_048_576;
const VECTOR: &[u8] = include_bytes!("../../develop/specs/vectors/vc-v1.json");

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];
    let context = source_context();

    let first_vc = import_vc_v1_json(data, context);
    let second_vc = import_vc_v1_json(data, context);
    assert_eq!(vc_signature(&first_vc), vc_signature(&second_vc));

    let first_skeleton = import_vc_skeleton_v1_json(data, source_vc(), context);
    let second_skeleton = import_vc_skeleton_v1_json(data, source_vc(), context);
    assert_eq!(
        skeleton_signature(&first_skeleton),
        skeleton_signature(&second_skeleton)
    );
});

fn source_context() -> &'static VcSourceContext {
    static CONTEXT: OnceLock<VcSourceContext> = OnceLock::new();
    CONTEXT.get_or_init(|| {
        let vector: serde_json::Value =
            serde_json::from_slice(VECTOR).expect("tracked VC vector parses");
        serde_json::from_value(vector["source_contexts"][0].clone())
            .expect("tracked VC context parses")
    })
}

fn source_vc() -> &'static [u8] {
    static BYTES: OnceLock<Vec<u8>> = OnceLock::new();
    BYTES
        .get_or_init(|| {
            generate_vc_v1_from_context(source_context())
                .expect("tracked VC context generates")
                .canonical_bytes()
                .to_vec()
        })
        .as_slice()
}

fn vc_signature(
    result: &Result<ValidatedVcDocument, VcValidationError>,
) -> (bool, String, String, usize) {
    match result {
        Ok(value) => (
            true,
            value.hash().as_str().to_owned(),
            String::new(),
            value.canonical_bytes().len(),
        ),
        Err(error) => (
            false,
            error.phase().as_str().to_owned(),
            error.code().to_owned(),
            0,
        ),
    }
}

fn skeleton_signature(
    result: &Result<ValidatedVcCertificateSkeleton, VcSkeletonValidationError>,
) -> (bool, String, String, usize) {
    match result {
        Ok(value) => (
            true,
            "accepted".to_owned(),
            String::new(),
            value.canonical_bytes().len(),
        ),
        Err(error) => (
            false,
            error.phase().as_str().to_owned(),
            error.code().to_owned(),
            0,
        ),
    }
}
