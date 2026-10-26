use mpk_vc::go_practical_type_keys::*;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|b| format!("{b:02x}")).collect()
}
fn facts(info: &GoExecutableBuildInfo) -> Value {
    fn module(m: &GoExecutableModule) -> Value {
        json!({"path_hex":hex(&m.path),"version_hex":hex(&m.version),"sum_hex":hex(&m.sum),"replacement":m.replacement.as_deref().map(module)})
    }
    json!({"version_hex":hex(&info.go_version),"path_hex":hex(&info.path),"main":module(&info.main),"dependencies":info.dependencies.iter().map(module).collect::<Vec<_>>(),"settings":info.settings.iter().map(|(k,v)|[hex(k),hex(v)]).collect::<Vec<_>>()})
}
#[test]
fn exact_original_binary_layout_and_arbitrary_text_facts_match_pinned_go() {
    let dir = root().join("compiler-build-info/fixtures");
    let cases: Value =
        serde_json::from_slice(&std::fs::read(dir.join("native.json")).unwrap()).unwrap();
    let original_dir = root()
        .join("compiler-inputs/native/native-originals/versioned-owned/original-compiler-files");
    let mut observations = vec![];
    let mut originals = 0;
    for row in cases.as_array().unwrap() {
        let name = row["name"].as_str().unwrap();
        let is_text = row["form"] == "text";
        let original = row["form"] == "original";
        let path = if original {
            originals += 1;
            original_dir.join(format!("{}.raw", row["raw_sha256"].as_str().unwrap()))
        } else {
            dir.join(row["file"].as_str().unwrap())
        };
        let raw = std::fs::read(path).unwrap();
        assert_eq!(hex(&Sha256::digest(&raw)), row["raw_sha256"], "{name}");
        assert_eq!(raw.len(), row["bytes"].as_u64().unwrap() as usize, "{name}");
        let parsed = if is_text {
            parse_go_executable_build_info_text(&raw)
        } else {
            read_go_executable_build_info(&raw)
        };
        assert_eq!(
            parsed.is_ok(),
            row["accepted"].as_bool().unwrap(),
            "{name}: {parsed:?}"
        );
        if row["form"] != "binary-control" && !is_text {
            assert!(
                parsed.is_ok(),
                "layout probe rejected by both readers: {name}: {}",
                row["error"]
            );
        }
        if let Ok(mut info) = parsed {
            assert_eq!(facts(&info), row["facts"], "{name}");
            if is_text {
                info.go_version = b"go1.25.0".to_vec();
            }
            let (tool, host) = if original {
                (row["tool"].as_str().unwrap(), row["host"].as_str().unwrap())
            } else {
                ("compile", "linux_amd64")
            };
            assert_eq!(
                info.validate_compiler_tool(tool, host).is_ok(),
                row["binding_accepted"].as_bool().unwrap(),
                "{name}"
            );
            observations.push(json!({"name":name,"facts":facts(&info),"binding_accepted":row["binding_accepted"]}));
        } else {
            observations.push(json!({"name":name,"rejected":true}));
        }
    }
    assert_eq!(originals, 9);
    if let Some(out) = std::env::var_os("MPK_GO_BUILD_INFO_OUTPUT") {
        let out = PathBuf::from(out);
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(
            out.join("native-comparison.json"),
            serde_json::to_vec_pretty(&json!({"status":"passed","cases":observations})).unwrap(),
        )
        .unwrap();
    }
}
