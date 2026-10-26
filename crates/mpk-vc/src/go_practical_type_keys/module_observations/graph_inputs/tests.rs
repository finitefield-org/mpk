//! Original native comparison of the shared source-fact/expansion machinery.
//! The adapter is test-only; production can read only the owned source snapshot.
use super::*;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification/root-expansion")
}
fn read(name: &str) -> Value {
    serde_json::from_slice(&std::fs::read(root().join(name)).unwrap()).unwrap()
}
fn source(row: &Value) -> GoTypeSourceBuffer {
    GoTypeSourceBuffer::import(
        &serde_json::to_vec(&row["object"]).unwrap(),
        &bytes(row["raw_hex"].as_str().unwrap()).unwrap(),
    )
    .unwrap()
}
fn module(row: &Value) -> GoSourceGraphModule {
    GoSourceGraphModule {
        path: row["path"].as_str().unwrap().into(),
        version: bytes(row["version_hex"].as_str().unwrap()).unwrap(),
    }
}
fn modules(rows: &Value) -> Vec<GoSourceGraphModule> {
    rows.as_array().unwrap().iter().map(module).collect()
}
fn output(name: &str, value: &Value) {
    if let Some(dir) = std::env::var_os("MPK_GO_ROOT_EXPANSION_OUTPUT") {
        use std::io::Write;
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(dir.join(name))
            .unwrap()
            .write_all(&serde_json::to_vec_pretty(value).unwrap())
            .unwrap();
    }
}
#[test]
fn original_native_root_order_and_unbounded_version_precedence() {
    let corpus =
        read("native-final/native-originals/root-ordering/complete-original-root-ordering.json");
    for row in corpus["comparisons"].as_array().unwrap() {
        let order = manifest::graph_version_compare(
            row["path"].as_str().unwrap(),
            &bytes(row["a_hex"].as_str().unwrap()).unwrap(),
            &bytes(row["b_hex"].as_str().unwrap()).unwrap(),
        );
        assert_eq!(order as i8 as i64, row["order"].as_i64().unwrap());
    }
    for row in corpus["sorts"].as_array().unwrap() {
        let mut actual = modules(&row["before"]);
        root_sort::sort(&mut actual);
        assert_eq!(actual, modules(&row["after"]), "{}", row["case"]);
    }
    output(
        "native-order-comparison.json",
        &json!({"status":"passed","comparisons":corpus["comparisons"].as_array().unwrap().len(),"sorts":corpus["sorts"].as_array().unwrap().len(),"complete_original_corpus":corpus}),
    );
}
#[test]
fn original_native_root_expansion_preserves_transient_inputs() {
    let corpus =
        read("native-final/native-originals/root-expansion/complete-original-root-expansion.json");
    let mut observations = vec![];
    for row in corpus["cases"].as_array().unwrap() {
        let main = derive_go_source_module_manifest(source(&row["main"]), true).unwrap();
        let main_path = text(main.module_path()).unwrap();
        let originals = row["sources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (module(&r["module"]), source(r)))
            .collect::<BTreeMap<_, _>>();
        let load = |roots: &[GoSourceGraphModule]| {
            expansion::phase(main_path, roots, |m| {
                if m.virtual_node() {
                    return Ok(virtual_summary(m));
                }
                ensure(m.valid(), G::Identity)?;
                source_summary(
                    &main,
                    m,
                    summary_wire(m),
                    originals.get(m).ok_or(G::Manifest)?.clone(),
                    m.path.as_bytes(),
                    &m.version,
                )
            })
        };
        let roots = original_roots(&main).unwrap();
        let initial = load(&roots).unwrap();
        let input = expand(main_path, roots, initial, load).unwrap();
        assert_eq!(
            input.roots,
            modules(&row["derived_roots"]),
            "{}",
            row["case"]
        );
        assert_eq!(
            input.roots,
            root_edges(
                &bytes(row["metadata_hex"].as_str().unwrap()).unwrap(),
                main_path
            )
            .unwrap()
        );
        for (key, native) in [
            ("initial", &input.initial),
            ("final", &input.final_phase.summaries),
        ] {
            let expected = row[key].as_array().unwrap();
            assert_eq!(native.len(), expected.len(), "{} {key}", row["case"]);
            for r in expected {
                let id = module(&r["identity"]);
                let s = &native[&id];
                assert_eq!(s.wire.pruned, r["pruned"].as_bool().unwrap());
                assert_eq!(s.wire.go_version, r["go_version"].as_str().unwrap());
                let required = r["requirements"]
                    .as_array()
                    .map_or_else(Vec::new, |rows| rows.iter().map(|m| module(m)).collect());
                assert_eq!(s.requirements, required);
            }
        }
        if row["case"] == "cascade-transient" {
            let transient = GoSourceGraphModule {
                path: "example.com/c".into(),
                version: b"v1.1.0".to_vec(),
            };
            assert!(!input.initial.contains_key(&transient));
            assert!(!input.final_phase.summaries.contains_key(&transient));
            assert!(input.manifests.contains_key(&transient));
            assert!(row["sources"]
                .as_array()
                .unwrap()
                .iter()
                .any(
                    |r| module(&r["module"]) == transient && r["captured_during_expansion"] == true
                ));
            let roots = original_roots(&main).unwrap();
            let fail = |roots: &[GoSourceGraphModule]| {
                expansion::phase(main_path, roots, |m| {
                    if m == &transient {
                        return Err(G::Manifest);
                    }
                    if m.virtual_node() {
                        return Ok(virtual_summary(m));
                    }
                    source_summary(
                        &main,
                        m,
                        summary_wire(m),
                        originals.get(m).ok_or(G::Manifest)?.clone(),
                        m.path.as_bytes(),
                        &m.version,
                    )
                })
            };
            let initial = fail(&roots).unwrap();
            assert!(matches!(
                expand(main_path, roots, initial, fail),
                Err(G::Manifest)
            ));
        }
        observations.push(json!({"case":row["case"],"roots":input.roots.iter().map(GoSourceGraphModule::wire).collect::<Vec<_>>(),"consumed_manifests":input.manifests.iter().map(|(m,f)|json!({"module":m.wire(),"source":f.original_source().object()})).collect::<Vec<_>>(),"original_metadata_hex":row["metadata_hex"],"status":"passed"}));
    }
    assert_eq!(observations.len(), 18);
    output(
        "native-expansion-comparison.json",
        &json!({"status":"passed","cases":observations,"source_program_or_installed_authority":false}),
    );
}
