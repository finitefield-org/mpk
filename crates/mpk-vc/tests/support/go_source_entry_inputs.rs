use mpk_vc::go_practical_type_keys::*;
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits,
};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

pub(super) fn evidence() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/go-practical/t02-w01/verification")
}
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn entries(
    scopes: &GoSourceScopes,
    roots: &[GoSourceSelectionRef],
    cases: &[GoSourceSelectionRef],
) -> Vec<GoErrorEntryConstraint> {
    let mut entries = vec![];
    for root in roots {
        let signature =
            derive_go_source_signature(scopes, root.source_object_id(), root.node()).unwrap();
        let graph = signature.types();
        for (input, slot) in signature
            .slots()
            .iter()
            .filter(|s| s.role() != GoSourceSignatureRole::Result)
            .enumerate()
        {
            let mut todo = vec![(slot.type_id().to_string(), vec![])];
            while let Some((id, path)) = todo.pop() {
                match graph.types().find(|t| t.id() == id).unwrap() {
                    GoSourceTypeKey::Nominal { .. } => {
                        todo.push((graph.underlying(&id).unwrap().into(), path))
                    }
                    GoSourceTypeKey::Builtin { name, .. } if name == "error" => entries.push(
                        GoErrorEntryConstraint::new(root.clone(), input, path, cases.to_vec()),
                    ),
                    GoSourceTypeKey::Array { element, .. }
                    | GoSourceTypeKey::Slice { element, .. } => {
                        let mut p = path;
                        p.push(GoErrorEntryStep::Elements);
                        todo.push((element.clone(), p));
                    }
                    GoSourceTypeKey::Struct { fields, .. } => {
                        for (i, f) in fields.iter().enumerate() {
                            let mut p = path.clone();
                            p.push(GoErrorEntryStep::Field(i));
                            todo.push((f.type_id.clone(), p));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    entries
}
pub(super) fn original(name: &str) -> GoSourceErrorSelection {
    let packets: Value = serde_json::from_slice(
        &std::fs::read(evidence().join("source-inputs/corpus.json")).unwrap(),
    )
    .unwrap();
    let prior: Value = serde_json::from_slice(
        &std::fs::read(evidence().join("source-invocations/corpus.json")).unwrap(),
    )
    .unwrap();
    let packet = packets["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == name)
        .unwrap();
    let prior = prior["units"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == name)
        .unwrap();
    assert_eq!(packet["input"], prior["input"]);
    let p = &packet["input"];
    let sources = p["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            GoTypeSourceBuffer::import(
                &serde_json::to_vec(&s["object"]).unwrap(),
                &unhex(s["raw_hex"].as_str().unwrap()),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let input =
        import_go_source_inputs(&unhex(p["components_hex"].as_str().unwrap()), &sources).unwrap();
    let scopes = input.source_scopes().unwrap();
    let locate = |r: &Value, kind: &str| {
        let source = r["source"].as_str().unwrap();
        let n = scopes
            .source(source)
            .unwrap()
            .nodes()
            .iter()
            .position(|n| {
                n.kind() == kind
                    && n.start() == r["start"].as_u64().unwrap() as usize
                    && n.end() == r["end"].as_u64().unwrap() as usize
            })
            .unwrap();
        GoSourceSelectionRef::new(source, n)
    };
    let roots = prior["roots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| locate(r, "FuncDecl"))
        .collect::<Vec<_>>();
    let cases = prior["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| locate(r, "TypeSpec"))
        .collect::<Vec<_>>();
    derive_go_source_error_selection(
        input.components(),
        &roots,
        &entries(&scopes, &roots, &cases),
    )
    .unwrap()
}
fn seal(value: &mut Value, domain: &'static str, field: &str) {
    let parsed = parse_strict_json(
        &serde_json::to_vec(value).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    value[field] = json!(hash_canonical_json(HashDomain::new(domain), &parsed)
        .unwrap()
        .to_hex());
}
pub(super) fn supplement() -> GoSourceErrorSelection {
    let project = evidence().join("source-entries/project");
    let sources = [("main.go", "go_file"), ("go.mod", "go_mod")]
        .into_iter()
        .map(|(name, kind)| {
            GoTypeSourceBuffer::new(
                "project",
                name,
                kind,
                &std::fs::read(project.join(name)).unwrap(),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let id = |kind: &str| {
        sources
            .iter()
            .find(|s| s.object().kind == kind)
            .unwrap()
            .object()
            .id
            .clone()
    };
    let mut package = json!({"schema":"mpk.go.type_package.v2","import_path":"example.com/entry","package_name":"main","module_source_object_id":id("go_mod"),"source_objects":[id("go_file")]});
    seal(&mut package, "MPK-GO-TYPE-PACKAGE-2.0", "id");
    let mut seed = json!({"schema":"mpk.go.source_type.v2","kind":"builtin","name":"bool","width":null,"signed":null});
    seal(&mut seed, "MPK-GO-SOURCE-TYPE-2.0", "id");
    let mut objects = sources
        .iter()
        .map(|s| s.object().clone())
        .collect::<Vec<_>>();
    objects.sort_by(|a, b| a.id.cmp(&b.id));
    let mut record = json!({"schema":"mpk.go.type_components.v2","source_objects":objects,"package_roots":[package["id"]],"packages":[package],"declarations":[],"type_roots":[seed["id"]],"types":[seed],"underlying":[],"aliases":[]});
    seal(&mut record, "MPK-GO-TYPE-COMPONENTS-2.0", "content_sha256");
    let parsed = parse_strict_json(
        &serde_json::to_vec(&record).unwrap(),
        StrictJsonLimits::new(268435456, 268435456, 64, 16777216),
    )
    .unwrap();
    let input = import_go_source_inputs(&canonical_json_bytes(&parsed).unwrap(), &sources).unwrap();
    let scopes = input.source_scopes().unwrap();
    let source = id("go_file");
    let parsed = scopes.source(&source).unwrap();
    let roots = parsed
        .nodes()
        .iter()
        .enumerate()
        .filter_map(|(i, n)| {
            if n.kind() != "FuncDecl" {
                return None;
            }
            let name = n
                .edges()
                .iter()
                .find(|e| e.role() == "Name")
                .unwrap()
                .targets()[0];
            let name = parsed.nodes()[name]
                .attributes()
                .find_map(|(k, v)| (k == "Name").then_some(v))
                .unwrap();
            ["Named", "ReadZero", "Hidden", "Shadow"]
                .contains(&name)
                .then(|| GoSourceSelectionRef::new(&source, i))
        })
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 4);
    derive_go_source_error_selection(input.components(), &roots, &[]).unwrap()
}
pub(super) fn write(name: &str, raw: &[u8]) {
    if let Some(out) = std::env::var_os("MPK_GO_SOURCE_ENTRY_OUTPUT") {
        use std::io::Write;
        let path = PathBuf::from(out).join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap()
            .write_all(raw)
            .unwrap();
    }
}
pub(super) fn function_name(s: &GoSourceErrorSelection, r: &GoSourceSelectionRef) -> String {
    let parsed = s.selection().source(r.source_object_id()).unwrap();
    let n = &parsed.nodes()[r.node()];
    let name = n
        .edges()
        .iter()
        .find(|e| e.role() == "Name")
        .unwrap()
        .targets()[0];
    parsed.nodes()[name]
        .attributes()
        .find_map(|(k, v)| (k == "Name").then_some(v.to_string()))
        .unwrap()
}
