#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use mpk_vc::{
    canonical_json_bytes, import_certificate_source_manifest_json,
    import_frontend_source_manifest_json, import_source_map_json, import_vir_json,
    parse_strict_json, validate_release_registry, CapturedInput, InputKind, SemanticParameters,
    SemanticProfile, SourceManifestError, SourceManifestValidationContext,
    SourceMapValidationContext, StrictJsonLimits, ValidatedReleaseRegistry,
    ValidatedSourceManifest, ValidatedSourceMap, ValidatedVcIdentity, VirModule,
};

const MAX_FUZZ_INPUT: usize = 1_048_576;
const ALL_JSON_LIMITS: StrictJsonLimits =
    StrictJsonLimits::new(268_435_456, 268_435_456, 256, 1_048_576);
const CAPTURED_INPUTS: &[CapturedInput<'static>] = &[
    CapturedInput {
        kind: InputKind::Contract,
        normalized_path: "contracts/identity.json",
        bytes: b"{\"schema\":\"mpk.go.contract.v0\",\"function\":\"example.com/mpk/vector.Identity\",\"requires\":[],\"ensures\":[{\"op\":\"eq\",\"lhs\":{\"result\":0},\"rhs\":{\"var\":\"value\"}}],\"modifies\":[],\"loops\":[]}\n",
    },
    CapturedInput {
        kind: InputKind::BuildManifest,
        normalized_path: "go.mod",
        bytes: b"module example.com/mpk/vector\n\ngo 1.25\n",
    },
    CapturedInput {
        kind: InputKind::Lockfile,
        normalized_path: "go.sum",
        bytes: b"",
    },
    CapturedInput {
        kind: InputKind::Source,
        normalized_path: "identity.go",
        bytes: b"package vector\n\nfunc Identity(value int8) int8 { return value }\n",
    },
];

const VIR_VECTOR: &[u8] = include_bytes!("../../develop/specs/vectors/vir-v0.json");
const SOURCE_MAP_VECTOR: &[u8] = include_bytes!("../../develop/specs/vectors/source-map-v0.json");
const RELEASE_VECTOR: &[u8] = include_bytes!("../../develop/specs/vectors/release-bundles-v0.json");

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];
    let context = SourceManifestValidationContext {
        vir: vir(),
        source_map: source_map(),
        captured_inputs: CAPTURED_INPUTS,
        release_registry: release_registry(),
        expected_language_configuration: None,
    };

    let first_frontend = import_frontend_source_manifest_json(data, context);
    let second_frontend = import_frontend_source_manifest_json(data, context);
    assert_eq!(signature(&first_frontend), signature(&second_frontend));

    let first_certificate = import_certificate_source_manifest_json(data, context, vc_identity());
    let second_certificate = import_certificate_source_manifest_json(data, context, vc_identity());
    assert_eq!(
        signature(&first_certificate),
        signature(&second_certificate)
    );
});

fn vir() -> &'static VirModule {
    static VIR: OnceLock<VirModule> = OnceLock::new();
    VIR.get_or_init(|| {
        let vector: serde_json::Value =
            serde_json::from_slice(VIR_VECTOR).expect("tracked VIR vector parses");
        import_vir_json(&canonical_fixture(&vector["module_cases"][0]["input"]))
            .expect("tracked Go identity VIR imports")
    })
}

fn source_map() -> &'static ValidatedSourceMap {
    static SOURCE_MAP: OnceLock<ValidatedSourceMap> = OnceLock::new();
    SOURCE_MAP.get_or_init(|| {
        let vector: serde_json::Value =
            serde_json::from_slice(SOURCE_MAP_VECTOR).expect("tracked source-map vector parses");
        import_source_map_json(
            &canonical_fixture(&vector["map_cases"][0]["input"]),
            SourceMapValidationContext {
                vir: vir(),
                captured_inputs: CAPTURED_INPUTS,
                synthetic_permissions: &[],
            },
        )
        .expect("tracked Go identity source map imports")
    })
}

fn release_registry() -> &'static ValidatedReleaseRegistry {
    static REGISTRY: OnceLock<ValidatedReleaseRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let vector: serde_json::Value =
            serde_json::from_slice(RELEASE_VECTOR).expect("tracked release vector parses");
        let mut bytes = canonical_fixture(&vector["fixtures"]["valid_registry"]);
        bytes.push(b'\n');
        validate_release_registry(&bytes).expect("tracked release registry validates")
    })
}

fn vc_identity() -> &'static ValidatedVcIdentity {
    static VC: OnceLock<ValidatedVcIdentity> = OnceLock::new();
    VC.get_or_init(|| {
        ValidatedVcIdentity::new(
            "e05e9fa46ee44a198470ada4935f756b12e9d6779601a7f88e4cb468d151ab31".to_owned(),
            "mpk.vir.v0".to_owned(),
            "374dbbcc0c9454bf29c0117c02f1bbdc0424df970297af9fe4560512d40d0690".to_owned(),
            serde_json::from_str::<SemanticProfile>("\"mpk.go.fixed.v0\"")
                .expect("fixed semantic profile parses"),
            serde_json::from_str::<SemanticParameters>(
                "{\"target_id\":\"linux/amd64\",\"pointer_width\":64}",
            )
            .expect("fixed semantic parameters parse"),
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        )
        .expect("fixed VC identity validates")
    })
}

fn canonical_fixture(value: &serde_json::Value) -> Vec<u8> {
    let encoded = serde_json::to_vec(value).expect("tracked fixture serializes");
    let strict = parse_strict_json(&encoded, ALL_JSON_LIMITS).expect("tracked fixture is strict");
    canonical_json_bytes(&strict).expect("tracked fixture canonicalizes")
}

fn signature(
    result: &Result<ValidatedSourceManifest, SourceManifestError>,
) -> (bool, String, String, usize) {
    match result {
        Ok(value) => (
            true,
            value.hash().as_str().to_owned(),
            format!("{:?}", value.stage()),
            value.canonical_bytes().len(),
        ),
        Err(error) => (
            false,
            error.phase.as_str().to_owned(),
            error.code.as_str().to_owned(),
            0,
        ),
    }
}
