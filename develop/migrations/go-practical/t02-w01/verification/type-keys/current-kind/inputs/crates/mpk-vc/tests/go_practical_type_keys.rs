use mpk_vc::go_practical_type_keys::{
    import_go_type_key_components, GoTypeKeyError, GoTypeSourceBuffer,
};
use mpk_vc::{
    canonical_json_bytes, hash_canonical_json, parse_strict_json, HashDomain, StrictJsonLimits,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn fixture(n: usize) -> (Vec<u8>, Vec<GoTypeSourceBuffer>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../develop/migrations/go-practical/t02-w01/verification/type-keys/native-components",
    );
    let case = root.join(format!("case-{n:02}"));
    let wire = std::fs::read(case.join("components.json")).unwrap();
    let mut sources = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(case.join("sources"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.extension().is_some_and(|e| e == "json") {
            sources.push(
                GoTypeSourceBuffer::import(
                    &std::fs::read(&path).unwrap(),
                    &std::fs::read(path.with_extension("raw")).unwrap(),
                )
                .unwrap(),
            );
        }
    }
    (wire, sources)
}
fn canonical(v: &Value) -> Vec<u8> {
    let bytes = serde_json::to_vec(v).unwrap();
    canonical_json_bytes(
        &parse_strict_json(
            &bytes,
            StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216),
        )
        .unwrap(),
    )
    .unwrap()
}
fn seal(v: &mut Value, domain: &'static str, field: &str) {
    v.as_object_mut().unwrap().remove(field);
    let bytes = canonical(v);
    let value = parse_strict_json(
        &bytes,
        StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216),
    )
    .unwrap();
    let id = hash_canonical_json(HashDomain::new(domain), &value)
        .unwrap()
        .to_hex();
    v[field] = json!(id);
}
fn root_seal(v: &mut Value) {
    seal(v, "MPK-GO-TYPE-COMPONENTS-2.0", "content_sha256")
}
fn sort(v: &mut Value, key: &str) {
    v.as_array_mut()
        .unwrap()
        .sort_by(|a, b| a[key].as_str().unwrap().cmp(b[key].as_str().unwrap()))
}
// Rehash every dependent source-type preimage after a semantic-key mutation.
// Underlying links are outside the identity DAG, so actual zero-array/slice
// recursion can be constructed without a cryptographic fixed-point shortcut.
fn rekey(v: &mut Value) {
    fn node(
        id: &str,
        nodes: &BTreeMap<String, Value>,
        done: &mut BTreeMap<String, Value>,
    ) -> String {
        if let Some(v) = done.get(id) {
            return v["id"].as_str().unwrap().into();
        }
        let mut v = nodes[id].clone();
        match v["kind"].as_str().unwrap() {
            "array" | "slice" => {
                v["element"] = json!(node(v["element"].as_str().unwrap(), nodes, done))
            }
            "struct" => {
                for field in v["fields"].as_array_mut().unwrap() {
                    field["type_id"] = json!(node(field["type_id"].as_str().unwrap(), nodes, done))
                }
            }
            _ => (),
        }
        seal(&mut v, "MPK-GO-SOURCE-TYPE-2.0", "id");
        let key = v["id"].as_str().unwrap().to_string();
        done.insert(id.into(), v);
        key
    }
    let nodes: BTreeMap<_, _> = v["types"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| (t["id"].as_str().unwrap().to_string(), t.clone()))
        .collect();
    let mut done = BTreeMap::new();
    for id in nodes.keys() {
        node(id, &nodes, &mut done);
    }
    let ids: BTreeMap<_, _> = done
        .iter()
        .map(|(k, v)| (k.clone(), v["id"].clone()))
        .collect();
    v["types"] = Value::Array(done.into_values().collect());
    sort(&mut v["types"], "id");
    for u in v["underlying"].as_array_mut().unwrap() {
        u["type_id"] = ids[u["type_id"].as_str().unwrap()].clone();
        u["underlying_type_id"] = ids[u["underlying_type_id"].as_str().unwrap()].clone();
    }
    sort(&mut v["underlying"], "type_id");
    for a in v["aliases"].as_array_mut().unwrap() {
        a["target_type_id"] = ids[a["target_type_id"].as_str().unwrap()].clone()
    }
    for r in v["type_roots"].as_array_mut().unwrap() {
        *r = ids[r.as_str().unwrap()].clone()
    }
    v["type_roots"]
        .as_array_mut()
        .unwrap()
        .sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    root_seal(v);
}
fn retain(name: &str, wire: &[u8], outcome: &str) {
    if let Some(root) = std::env::var_os("MPK_GO_TYPE_KEYS_RUST_OUTPUT") {
        use std::io::Write;
        let root = PathBuf::from(root);
        std::fs::create_dir_all(&root).unwrap();
        for (extension, bytes) in [("json", wire), ("outcome.txt", outcome.as_bytes())] {
            std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(root.join(format!("{name}.{extension}")))
                .unwrap()
                .write_all(bytes)
                .unwrap();
        }
    }
}
fn retain_sources(name: &str, buffers: &[GoTypeSourceBuffer]) {
    let data: Vec<_> = buffers
        .iter()
        .map(|s| json!({"object":s.object(),"whole_original_bytes":s.bytes()}))
        .collect();
    retain(
        &format!("{name}-sources"),
        &canonical(&json!(data)),
        "complete-source-set-before-import\n",
    );
}
fn reject(name: &str, v: &Value, buffers: &[GoTypeSourceBuffer], expected: GoTypeKeyError) {
    let bytes = canonical(v);
    retain(name, &bytes, "candidate-before-import\n");
    let error = import_go_type_key_components(&bytes, buffers).unwrap_err();
    println!("{name}: actual component rejection {error:?}");
    assert_eq!(error, expected, "{name}");
}
#[test]
fn imports_actual_original_go_components() {
    for n in 0..5 {
        let (wire, buffers) = fixture(n);
        retain(
            &format!("positive-{n:02}"),
            &wire,
            "original-Go-candidate-before-import\n",
        );
        let keys = import_go_type_key_components(&wire, &buffers).unwrap();
        println!(
            "original-Go-{n:02}: exact canonical components {}",
            keys.content_hash()
        );
        assert_eq!(keys.canonical_bytes(), wire);
        assert_eq!(
            keys,
            import_go_type_key_components(keys.canonical_bytes(), &buffers).unwrap()
        );
        for source in &buffers {
            assert_eq!(keys.source(&source.object().id), Some(source))
        }
        let mut caller = wire.clone();
        caller.fill(0);
        assert_eq!(keys.canonical_bytes(), wire);
    }
}
#[test]
fn strict_wire_original_bytes_and_exact_membership() {
    let (wire, buffers) = fixture(0);
    let original: Value = serde_json::from_slice(&wire).unwrap();
    for (name, mut v) in [
        ("unknown-field", original.clone()),
        ("missing-null", original.clone()),
        ("wrong-schema", original.clone()),
        ("wrong-order", original.clone()),
        ("duplicate-type", original.clone()),
        ("missing-source", original.clone()),
        ("wrong-span", original.clone()),
    ] {
        let expected = match name {
            "unknown-field" => {
                v["extra"] = json!(true);
                GoTypeKeyError::Shape
            }
            "missing-null" => {
                let node = v["types"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|n| n["name"] == "bool")
                    .unwrap();
                node.as_object_mut().unwrap().remove("width");
                GoTypeKeyError::Shape
            }
            "wrong-schema" => {
                v["schema"] = json!("mpk.go.type_components.v1");
                GoTypeKeyError::Shape
            }
            "wrong-order" => {
                v["types"].as_array_mut().unwrap().reverse();
                GoTypeKeyError::Order
            }
            "duplicate-type" => {
                let node = v["types"][0].clone();
                v["types"].as_array_mut().unwrap().insert(0, node);
                GoTypeKeyError::Order
            }
            "missing-source" => {
                v["source_objects"].as_array_mut().unwrap().pop();
                GoTypeKeyError::Source
            }
            "wrong-span" => {
                let d = &mut v["declarations"][0];
                d["end_byte"] = json!(d["start_byte"]);
                seal(d, "MPK-GO-TYPE-DECLARATION-2.0", "id");
                sort(&mut v["declarations"], "id");
                GoTypeKeyError::Reference
            }
            _ => unreachable!(),
        };
        root_seal(&mut v);
        reject(name, &v, &buffers, expected);
    }
    let duplicate = String::from_utf8(wire.clone()).unwrap().replacen(
        '{',
        "{\"schema\":\"mpk.go.type_components.v2\",",
        1,
    );
    retain(
        "duplicate-json",
        duplicate.as_bytes(),
        "candidate-before-import\n",
    );
    assert_eq!(
        import_go_type_key_components(duplicate.as_bytes(), &buffers).unwrap_err(),
        GoTypeKeyError::Json
    );
    retain("missing-owned-buffer", &wire, "candidate-before-import\n");
    retain_sources("missing-owned-buffer", &buffers[1..]);
    assert_eq!(
        import_go_type_key_components(&wire, &buffers[1..]).unwrap_err(),
        GoTypeKeyError::Source
    );
    let mut doubled = buffers.clone();
    doubled.push(buffers[0].clone());
    retain("duplicate-owned-buffer", &wire, "candidate-before-import\n");
    retain_sources("duplicate-owned-buffer", &doubled);
    assert_eq!(
        import_go_type_key_components(&wire, &doubled).unwrap_err(),
        GoTypeKeyError::Source
    );
    let source = &buffers[0];
    let mut raw = source.bytes().to_vec();
    raw.push(b'\n');
    retain(
        "modified-original-buffer",
        &canonical(&json!({"object":source.object(),"whole_original_bytes":raw})),
        "candidate-before-source-buffer-import\n",
    );
    assert_eq!(
        GoTypeSourceBuffer::import(&serde_json::to_vec(source.object()).unwrap(), &raw)
            .unwrap_err(),
        GoTypeKeyError::Identity
    );
    retain(
        "unknown-source-kind",
        &canonical(
            &json!({"capability":"root","path":"x.go","kind":"invalid_kind","whole_original_bytes":b"package x"}),
        ),
        "candidate-before-source-buffer-construction\n",
    );
    assert_eq!(
        GoTypeSourceBuffer::new("root", "x.go", "invalid_kind", b"package x").unwrap_err(),
        GoTypeKeyError::Source
    );
    retain(
        "invalid-source-path",
        &canonical(
            &json!({"capability":"../root","path":"x.go","kind":"go_file","whole_original_bytes":b""}),
        ),
        "candidate-before-source-buffer-construction\n",
    );
    assert_eq!(
        GoTypeSourceBuffer::new("../root", "x.go", "go_file", b"").unwrap_err(),
        GoTypeKeyError::Source
    );
}
#[test]
fn rejects_rehashed_invalid_type_and_package_edges() {
    let (wire, buffers) = fixture(0);
    let original: Value = serde_json::from_slice(&wire).unwrap();
    for name in [
        "old-type-schema",
        "word-kind",
        "array257",
        "fields65",
        "tag-hex",
        "unexported-package",
        "exported-package",
        "alias-as-nominal",
        "underlying-nominal",
        "alias-missing",
        "package-root-missing",
        "root-type-missing",
    ] {
        let mut v = original.clone();
        let mut type_mutation = true;
        let expected = match name {
            "old-type-schema" => {
                v["types"][0]["schema"] = json!("mpk.go.source_type.v1");
                GoTypeKeyError::Identity
            }
            "word-kind" => {
                let t = v["types"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|t| t["name"] == "int")
                    .unwrap();
                t["signed"] = json!(false);
                GoTypeKeyError::Shape
            }
            "array257" => {
                let t = v["types"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|t| t["kind"] == "array")
                    .unwrap();
                t["length"] = json!(257);
                GoTypeKeyError::Limit
            }
            "fields65" => {
                let t = v["types"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|t| t["kind"] == "struct" && !t["fields"].as_array().unwrap().is_empty())
                    .unwrap();
                let mut f = t["fields"][0].clone();
                f["name"] = json!("_");
                f["embedded"] = json!(false);
                let p = f["unexported_package_id"].clone();
                if p.is_null() {
                    f["unexported_package_id"] = original["package_roots"][0].clone()
                };
                t["fields"] = Value::Array(vec![f; 65]);
                GoTypeKeyError::Limit
            }
            "tag-hex" => {
                let t = v["types"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|t| t["kind"] == "struct" && !t["fields"].as_array().unwrap().is_empty())
                    .unwrap();
                t["fields"][0]["tag_bytes_hex"] = json!("FF");
                GoTypeKeyError::Shape
            }
            "unexported-package" | "exported-package" => {
                let field = v["types"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .filter_map(|t| t.get_mut("fields").and_then(Value::as_array_mut))
                    .flat_map(|f| f.iter_mut())
                    .find(|f| {
                        f["name"]
                            == if name == "unexported-package" {
                                "ᴬ"
                            } else {
                                "É"
                            }
                    })
                    .unwrap();
                field["unexported_package_id"] = if name == "unexported-package" {
                    Value::Null
                } else {
                    original["package_roots"][0].clone()
                };
                GoTypeKeyError::Reference
            }
            "alias-as-nominal" => {
                let alias = v["aliases"][0]["declaration_id"].clone();
                let t = v["types"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|t| t["kind"] == "nominal")
                    .unwrap();
                t["declaration_id"] = alias;
                GoTypeKeyError::Reference
            }
            "underlying-nominal" => {
                type_mutation = false;
                v["underlying"][0]["underlying_type_id"] = v["underlying"][0]["type_id"].clone();
                GoTypeKeyError::Reference
            }
            "alias-missing" => {
                type_mutation = false;
                v["aliases"] = json!([]);
                GoTypeKeyError::Reference
            }
            "package-root-missing" => {
                type_mutation = false;
                v["package_roots"] = json!([]);
                GoTypeKeyError::Reference
            }
            "root-type-missing" => {
                type_mutation = false;
                v["type_roots"] = json!(["f".repeat(64)]);
                GoTypeKeyError::Reference
            }
            _ => unreachable!(),
        };
        if type_mutation {
            rekey(&mut v)
        } else {
            root_seal(&mut v)
        };
        reject(name, &v, &buffers, expected);
    }
}
#[test]
fn rejects_expanded_zero_array_and_slice_cycles_and_depth17() {
    let (wire, buffers) = fixture(0);
    let original: Value = serde_json::from_slice(&wire).unwrap();
    for kind in ["array", "slice"] {
        let mut v = original.clone();
        let nominal = v["underlying"][0]["type_id"].clone();
        let mut node = json!({"schema":"mpk.go.source_type.v2","kind":kind,"element":nominal});
        if kind == "array" {
            node["length"] = json!(0)
        }
        seal(&mut node, "MPK-GO-SOURCE-TYPE-2.0", "id");
        v["underlying"][0]["underlying_type_id"] = node["id"].clone();
        v["type_roots"]
            .as_array_mut()
            .unwrap()
            .push(node["id"].clone());
        v["types"].as_array_mut().unwrap().push(node);
        rekey(&mut v);
        reject(
            &format!("cycle-{kind}"),
            &v,
            &buffers,
            GoTypeKeyError::Cycle,
        );
    }
    let mut v = original;
    let mut child = v["types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "bool")
        .unwrap()["id"]
        .clone();
    for _ in 0..17 {
        let mut node =
            json!({"schema":"mpk.go.source_type.v2","kind":"array","length":0,"element":child});
        seal(&mut node, "MPK-GO-SOURCE-TYPE-2.0", "id");
        child = node["id"].clone();
        v["types"].as_array_mut().unwrap().push(node)
    }
    v["type_roots"].as_array_mut().unwrap().push(child);
    rekey(&mut v);
    reject("depth17", &v, &buffers, GoTypeKeyError::Limit);
}

#[test]
fn component_boundaries_do_not_claim_source_typing() {
    let (wire, buffers) = fixture(0);
    let original: Value = serde_json::from_slice(&wire).unwrap();
    let mut v = original.clone();
    let builtin = v["types"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "bool")
        .unwrap()["id"]
        .clone();
    let field = json!({"name":"_","unexported_package_id":v["package_roots"][0],"type_id":builtin,"tag_bytes_hex":"","embedded":false});
    let mut node =
        json!({"schema":"mpk.go.source_type.v2","kind":"struct","fields":vec![field;64]});
    seal(&mut node, "MPK-GO-SOURCE-TYPE-2.0", "id");
    v["type_roots"]
        .as_array_mut()
        .unwrap()
        .push(node["id"].clone());
    v["types"].as_array_mut().unwrap().push(node);
    rekey(&mut v);
    let bytes = canonical(&v);
    retain(
        "synthetic-fields64",
        &bytes,
        "component-only-candidate-before-import\n",
    );
    import_go_type_key_components(&bytes, &buffers).unwrap();
    let mut v = original;
    let mut child = builtin;
    for _ in 0..16 {
        let mut node =
            json!({"schema":"mpk.go.source_type.v2","kind":"array","length":0,"element":child});
        seal(&mut node, "MPK-GO-SOURCE-TYPE-2.0", "id");
        child = node["id"].clone();
        v["types"].as_array_mut().unwrap().push(node);
    }
    v["type_roots"].as_array_mut().unwrap().push(child);
    rekey(&mut v);
    let bytes = canonical(&v);
    retain(
        "synthetic-depth16",
        &bytes,
        "component-only-candidate-before-import\n",
    );
    import_go_type_key_components(&bytes, &buffers).unwrap();
    // These invented component roots have no original Go declaration. Their
    // acceptance documents the boundary: hash/closure checks are not typing
    // witnesses and cannot be used as source/program or registered authority.
}
