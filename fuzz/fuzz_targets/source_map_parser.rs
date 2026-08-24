#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use mpk_vc::{
    import_source_map_json, import_vir_json, CapturedInput, InputKind, SourceMapValidationContext,
    VirModule,
};

const MAX_FUZZ_INPUT: usize = 1_048_576;
const CAPTURED_INPUTS: &[CapturedInput<'static>] = &[CapturedInput {
    kind: InputKind::Source,
    normalized_path: "identity.go",
    bytes: b"package vector\n",
}];

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];
    let context = SourceMapValidationContext {
        vir: source_vir(),
        captured_inputs: CAPTURED_INPUTS,
        synthetic_permissions: &[],
    };
    let first = import_source_map_json(data, context);
    let second = import_source_map_json(data, context);
    assert_eq!(signature(&first), signature(&second));
});

fn source_vir() -> &'static VirModule {
    static VIR: OnceLock<VirModule> = OnceLock::new();
    VIR.get_or_init(|| {
        let vector: serde_json::Value =
            serde_json::from_slice(include_bytes!("../../develop/specs/vectors/vir-v0.json"))
                .expect("tracked VIR vector parses");
        let input = &vector["module_cases"][0]["input"];
        import_vir_json(&serde_json::to_vec(input).expect("VIR fixture serializes"))
            .expect("tracked Go identity VIR imports")
    })
}

fn signature(
    result: &Result<mpk_vc::ValidatedSourceMap, mpk_vc::SourceMapError>,
) -> (bool, String, String) {
    match result {
        Ok(validated) => (
            true,
            validated.hash().as_str().to_owned(),
            validated.canonical_bytes().len().to_string(),
        ),
        Err(error) => (
            false,
            error.phase.as_str().to_owned(),
            error.code.as_str().to_owned(),
        ),
    }
}
