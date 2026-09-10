//! Integration must preserve component definitions, including their dependencies.
use super::*;
use mpk_cert::encode::{Certificate, DeclarationKind, LevelNode, TermNode};

fn roots(value: &Value, names: &mut BTreeSet<String>) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                if key == "definition" || key.ends_with("_definition") {
                    if let Some(name) = value.as_str() {
                        names.insert(name.to_owned());
                    }
                } else {
                    roots(value, names);
                }
            }
        }
        Value::Array(values) => values.iter().for_each(|v| roots(v, names)),
        _ => {}
    }
}

enum Pair {
    Global(String, String),
    Term(u32, u32),
    Level(u32, u32),
}

// Compare syntax, not an evaluator result or a host oracle. IDs and DAG sharing
// may differ, but names, binders, arguments, levels, declaration kinds and the
// complete transitive definitions must agree. Worklists avoid deep recursion.
pub(super) fn same_definition_closure(
    left: &Certificate,
    right: &Certificate,
    roots: &BTreeSet<String>,
) -> Result<BTreeSet<String>, String> {
    compare_definitions(left, right, roots, true, true, false)
}

/// Compare each named definition directly; callers must separately enumerate
/// and compare every global dependency before drawing a closure conclusion.
pub(super) fn same_definition_bodies(
    left: &Certificate,
    right: &Certificate,
    roots: &BTreeSet<String>,
) -> Result<BTreeSet<String>, String> {
    compare_definitions(left, right, roots, false, true, false)
}

pub(super) fn same_definition_types(
    left: &Certificate,
    right: &Certificate,
    roots: &BTreeSet<String>,
) -> Result<BTreeSet<String>, String> {
    compare_definitions(left, right, roots, false, false, false)
}

// Shared decimal circuits are interned by their complete typed circuit. The
// first operation to request one supplies its internal name. Permit only those
// internal names to differ, then compare their entire declarations recursively.
// Public operation names, all other globals, bodies, types and levels stay exact.
pub(super) fn same_decimal_definition_closure(
    left: &Certificate,
    right: &Certificate,
    roots: &BTreeSet<String>,
) -> Result<BTreeSet<String>, String> {
    compare_definitions(left, right, roots, true, true, true)
}
fn shared_decimal_names(left: &str, right: &str) -> bool {
    [
        "Mpk.CSharp.Ordinary.DecimalArithmetic.",
        "Mpk.CSharp.Ordinary.DecimalSteps.",
    ]
    .iter()
    .any(|prefix| left.starts_with(prefix) && right.starts_with(prefix))
}

fn compare_definitions(
    left: &Certificate,
    right: &Certificate,
    roots: &BTreeSet<String>,
    follow_globals: bool,
    compare_values: bool,
    allow_decimal_helper_names: bool,
) -> Result<BTreeSet<String>, String> {
    if left.imports != right.imports {
        return Err("different imports".into());
    }
    let globals = |c: &Certificate| {
        c.declarations
            .iter()
            .enumerate()
            .map(|(i, d)| (c.name_table[d.name as usize].clone(), i))
            .collect::<BTreeMap<_, _>>()
    };
    let (left_globals, right_globals) = (globals(left), globals(right));
    let name =
        |c: &Certificate, id: u32| c.name_table[c.declarations[id as usize].name as usize].clone();
    let mut pending = roots
        .iter()
        .map(|s| Pair::Global(s.clone(), s.clone()))
        .collect::<Vec<_>>();
    let mut seen_globals = BTreeSet::new();
    let mut seen_terms = BTreeSet::new();
    let mut seen_levels = BTreeSet::new();
    while let Some(pair) = pending.pop() {
        match pair {
            Pair::Global(global, right_global) => {
                if !seen_globals.insert((global.clone(), right_global.clone())) {
                    continue;
                }
                let l = left_globals
                    .get(&global)
                    .ok_or_else(|| format!("missing component global {global}"))?;
                let r = right_globals
                    .get(&right_global)
                    .ok_or_else(|| format!("missing integrated global {right_global}"))?;
                let (l, r) = (&left.declarations[*l].kind, &right.declarations[*r].kind);
                if std::mem::discriminant(l) != std::mem::discriminant(r) {
                    return Err(format!("declaration kind changed: {global}"));
                }
                match (l, r) {
                    (
                        DeclarationKind::Def {
                            ty: lt,
                            value: lv,
                            reducibility: lr,
                        },
                        DeclarationKind::Def {
                            ty: rt,
                            value: rv,
                            reducibility: rr,
                        },
                    ) => {
                        if lr != rr {
                            return Err(format!("reducibility changed: {global}"));
                        }
                        pending.push(Pair::Term(*lt, *rt));
                        if compare_values {
                            pending.push(Pair::Term(*lv, *rv));
                        }
                    }
                    (
                        DeclarationKind::Theorem { ty: lt, proof: lp },
                        DeclarationKind::Theorem { ty: rt, proof: rp },
                    ) => {
                        pending.push(Pair::Term(*lt, *rt));
                        if compare_values {
                            pending.push(Pair::Term(*lp, *rp));
                        }
                    }
                    (
                        DeclarationKind::Inductive { ty: l },
                        DeclarationKind::Inductive { ty: r },
                    ) => pending.push(Pair::Term(*l, *r)),
                    (
                        DeclarationKind::Constructor {
                            ty: lt,
                            inductive: li,
                            generated: lg,
                        }
                        | DeclarationKind::Recursor {
                            ty: lt,
                            inductive: li,
                            generated: lg,
                        },
                        DeclarationKind::Constructor {
                            ty: rt,
                            inductive: ri,
                            generated: rg,
                        }
                        | DeclarationKind::Recursor {
                            ty: rt,
                            inductive: ri,
                            generated: rg,
                        },
                    ) => {
                        if lg != rg || name(left, *li) != name(right, *ri) {
                            return Err(format!("inductive linkage changed: {global}"));
                        }
                        pending.extend([
                            Pair::Term(*lt, *rt),
                            Pair::Global(name(left, *li), name(right, *ri)),
                        ]);
                    }
                    _ => return Err(format!("unsupported or axiomatic declaration: {global}")),
                }
            }
            Pair::Term(l, r) => {
                if !seen_terms.insert((l, r)) {
                    continue;
                }
                let (l, r) = (&left.term_table[l as usize], &right.term_table[r as usize]);
                if std::mem::discriminant(l) != std::mem::discriminant(r) {
                    return Err(format!("term shape changed: {l:?} / {r:?}"));
                }
                match (l, r) {
                    (TermNode::Sort(l), TermNode::Sort(r)) => pending.push(Pair::Level(*l, *r)),
                    (TermNode::Var(l), TermNode::Var(r)) if l == r => {}
                    (
                        TermNode::Const {
                            global: lg,
                            levels: ll,
                        },
                        TermNode::Const {
                            global: rg,
                            levels: rl,
                        },
                    ) if (name(left, *lg) == name(right, *rg)
                        || (allow_decimal_helper_names
                            && shared_decimal_names(&name(left, *lg), &name(right, *rg))))
                        && ll.len() == rl.len() =>
                    {
                        if follow_globals {
                            pending.push(Pair::Global(name(left, *lg), name(right, *rg)));
                        }
                        pending.extend(ll.iter().zip(rl).map(|(l, r)| Pair::Level(*l, *r)));
                    }
                    (
                        TermNode::App {
                            function: lf,
                            arguments: la,
                        },
                        TermNode::App {
                            function: rf,
                            arguments: ra,
                        },
                    ) if la.len() == ra.len() => {
                        pending.push(Pair::Term(*lf, *rf));
                        pending.extend(la.iter().zip(ra).map(|(l, r)| Pair::Term(*l, *r)));
                    }
                    (
                        TermNode::Lam { ty: lt, body: lb } | TermNode::Pi { ty: lt, body: lb },
                        TermNode::Lam { ty: rt, body: rb } | TermNode::Pi { ty: rt, body: rb },
                    ) => {
                        pending.extend([Pair::Term(*lt, *rt), Pair::Term(*lb, *rb)]);
                    }
                    (
                        TermNode::Let {
                            ty: lt,
                            value: lv,
                            body: lb,
                        },
                        TermNode::Let {
                            ty: rt,
                            value: rv,
                            body: rb,
                        },
                    ) => {
                        pending.extend([
                            Pair::Term(*lt, *rt),
                            Pair::Term(*lv, *rv),
                            Pair::Term(*lb, *rb),
                        ]);
                    }
                    (TermNode::Const { global: l, .. }, TermNode::Const { global: r, .. }) => {
                        return Err(format!(
                            "constant target changed: {} / {}",
                            name(left, *l),
                            name(right, *r)
                        ));
                    }
                    _ => return Err(format!("term contents changed: {l:?} / {r:?}")),
                }
            }
            Pair::Level(l, r) => {
                if !seen_levels.insert((l, r)) {
                    continue;
                }
                match (
                    &left.level_table[l as usize],
                    &right.level_table[r as usize],
                ) {
                    (LevelNode::Zero, LevelNode::Zero) => {}
                    (LevelNode::Succ(l), LevelNode::Succ(r)) => pending.push(Pair::Level(*l, *r)),
                    (LevelNode::Max(la, lb), LevelNode::Max(ra, rb)) => {
                        pending.extend([Pair::Level(*la, *ra), Pair::Level(*lb, *rb)])
                    }
                    (LevelNode::Param(l), LevelNode::Param(r))
                        if left.name_table[*l as usize] == right.name_table[*r as usize] => {}
                    _ => return Err("universe level changed".into()),
                }
            }
        }
    }
    Ok(seen_globals.into_iter().map(|(left, _)| left).collect())
}

// Only reassociation is permitted: expand the concrete group and require all
// 8,192 StepTwo calls, with unchanged mode/predicate/length arguments. Guarded
// Compose is associative: both bracketings stop at the first inactive result.
// Normalize only the validated tree, then compare every old declaration. This
// is a regression audit, not a proof of an application VC.
pub(super) fn same_aggregate_scan(old: &Certificate, new: &Certificate) -> Result<(), String> {
    const PREFIX: &str = "Mpk.CSharp.Ordinary.AggregateFold.";
    fn lambdas(c: &Certificate, mut body: u32) -> Result<([u32; 3], u32), String> {
        let mut types = [0; 3];
        for ty in &mut types {
            let TermNode::Lam { ty: t, body: b } = c.term_table[body as usize] else {
                return Err("aggregate lambda shape changed".into());
            };
            *ty = t;
            body = b;
        }
        Ok((types, body))
    }
    fn normalize(c: &Certificate) -> Result<Certificate, String> {
        let pipeline = c
            .declarations
            .iter()
            .position(|d| c.name_table[d.name as usize] == format!("{PREFIX}Pipeline"))
            .ok_or("missing aggregate Pipeline")?;
        let DeclarationKind::Def { ty, value, .. } = c.declarations[pipeline].kind else {
            return Err("aggregate Pipeline kind".into());
        };
        let (types, body) = lambdas(c, value)?;
        let vars = |args: &[u32], expected: &[u32]| {
            args.len() == expected.len()
                && args
                    .iter()
                    .zip(expected)
                    .all(|(a, e)| c.term_table[*a as usize] == TermNode::Var(*e))
        };
        let mut pending = vec![body];
        let mut steps = 0;
        let mut visits = 0;
        while let Some(term) = pending.pop() {
            visits += 1;
            if visits > 40_000 {
                return Err("excessive aggregate expansion".into());
            }
            let TermNode::App {
                function,
                arguments,
            } = &c.term_table[term as usize]
            else {
                return Err("aggregate tree node is not an application".into());
            };
            let (global, args, children) = match &c.term_table[*function as usize] {
                TermNode::Const { global, levels } if levels.is_empty() => {
                    (*global, arguments.as_slice(), None)
                }
                TermNode::App {
                    function,
                    arguments: params,
                } => {
                    let TermNode::Const { global, levels } = &c.term_table[*function as usize]
                    else {
                        return Err("aggregate composition function".into());
                    };
                    if !levels.is_empty() {
                        return Err("aggregate composition levels".into());
                    }
                    (*global, params.as_slice(), Some(arguments))
                }
                _ => return Err("aggregate function shape".into()),
            };
            let d = &c.declarations[global as usize];
            match c.name_table[d.name as usize].strip_prefix(PREFIX) {
                Some("Compose") if vars(args, &[2, 0]) => {
                    let children = children.ok_or("missing aggregate composition children")?;
                    if children.len() != 2 {
                        return Err("aggregate composition arity".into());
                    }
                    pending.extend([children[1], children[0]]);
                }
                Some("StepTwo" | "StepEight") if children.is_none() && vars(args, &[2, 1, 0]) => {
                    if c.name_table[d.name as usize].ends_with("StepTwo") {
                        steps += 1;
                    } else {
                        let DeclarationKind::Def {
                            ty: group_ty,
                            value,
                            ..
                        } = d.kind
                        else {
                            return Err("aggregate group kind".into());
                        };
                        let (group_types, body) = lambdas(c, value)?;
                        // Builder interns these exact types; reject any changed binder.
                        if group_ty != ty || group_types != types {
                            return Err("aggregate group types changed".into());
                        }
                        pending.push(body);
                    }
                }
                _ => return Err("aggregate operation or arguments changed".into()),
            }
        }
        if steps != 8192 {
            return Err(format!("aggregate step count changed: {steps}"));
        }
        let mut result = ty;
        for _ in 0..3 {
            let TermNode::Pi { body, .. } = c.term_table[result as usize] else {
                return Err("aggregate Pipeline type".into());
            };
            result = body;
        }
        let TermNode::Pi { ty: state_ty, .. } = c.term_table[result as usize] else {
            return Err("aggregate transformer type".into());
        };
        let mut normalized = c.clone();
        let mut body = normalized.term_table.len() as u32;
        normalized.term_table.push(TermNode::Var(0));
        for ty in std::iter::once(state_ty).chain(types.into_iter().rev()) {
            normalized.term_table.push(TermNode::Lam { ty, body });
            body = normalized.term_table.len() as u32 - 1;
        }
        let DeclarationKind::Def { value, .. } = &mut normalized.declarations[pipeline].kind else {
            unreachable!()
        };
        *value = body;
        Ok(normalized)
    }
    let roots = old
        .declarations
        .iter()
        .map(|d| old.name_table[d.name as usize].clone())
        .collect();
    same_definition_closure(&normalize(old)?, &normalize(new)?, &roots)?;
    Ok(())
}

pub(super) fn certificate(directory: &str, file: &str) -> Certificate {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation")
        .join(directory)
        .join(file);
    let hex = fs::read_to_string(path).unwrap();
    let hex = hex.trim();
    let bytes = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    mpk_cert::decode_canonical_certificate(&bytes).unwrap()
}

#[test]
fn csharp_03_t06_w09_structural_boundary_previous_definition_preservation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation/structural-boundary");
    for path in fs::read_dir(root.join("previous-quoted-decimal"))
        .unwrap()
        .map(|r| r.unwrap().path())
    {
        if path.extension().and_then(|v| v.to_str()) != Some("hex") {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap();
        let old = certificate("structural-boundary/previous-quoted-decimal", name);
        let new = certificate("structural-boundary", name);
        if old
            .name_table
            .iter()
            .any(|v| v == "Mpk.CSharp.Ordinary.AggregateFold.Pipeline")
        {
            same_aggregate_scan(&old, &new).unwrap();
        } else {
            let roots = old
                .declarations
                .iter()
                .map(|d| old.name_table[d.name as usize].clone())
                .collect();
            same_definition_closure(&old, &new, &roots).unwrap();
        }
    }
}

#[test]
fn csharp_03_t06_w09_aggregate_aliased_consumer_preservation() {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../develop/migrations/csharp-03/ordinary-foundation");
    let output = std::env::var_os("MPK_W09_AGGREGATE_CONSUMER_OUT").map(std::path::PathBuf::from);
    let mut changed = 0;
    let mut unchanged = 0;
    for family in [
        "binding-guards",
        "binding-orders",
        "binding-relations",
        "collection-operations",
        "construction-operations",
        "domains",
        "entry-operations",
        "money-operations",
        "outcome-operations",
        "sequence-operations",
        "source-observations",
        "structural-foundations",
    ] {
        let root = base.join(family);
        let archive = root.join("previous-step-eight");
        let target = output.as_ref().map_or(root.clone(), |out| out.join(family));
        for path in fs::read_dir(&archive).unwrap().map(|v| v.unwrap().path()) {
            if path.extension().and_then(|s| s.to_str()) != Some("hex") {
                continue;
            }
            let file = path.file_name().unwrap().to_str().unwrap();
            let old_hex = fs::read_to_string(&path).unwrap();
            let new_hex = fs::read_to_string(target.join(file)).unwrap();
            let old = certificate(archive.to_str().unwrap(), file);
            let uses_aggregate = old
                .name_table
                .iter()
                .any(|name| name == "Mpk.CSharp.Ordinary.AggregateFold.Pipeline");
            if old_hex == new_hex {
                assert!(
                    !uses_aggregate,
                    "{family}/{file}: aggregate pin was not regenerated"
                );
                unchanged += 1;
                continue;
            }
            let new = certificate(target.to_str().unwrap(), file);
            assert!(new
                .name_table
                .iter()
                .any(|name| name == "Mpk.CSharp.Ordinary.AggregateFold.StepEight"));
            same_aggregate_scan(&old, &new).unwrap_or_else(|e| panic!("{family}/{file}: {e}"));
            changed += 1;
        }
        fn remove_counts(value: &mut Value) {
            match value {
                Value::Object(fields) => {
                    for key in [
                        "certificate_sha256",
                        "static_transformers",
                        "terms",
                        "declarations",
                    ] {
                        fields.remove(key);
                    }
                    for value in fields.values_mut() {
                        remove_counts(value);
                    }
                }
                Value::Array(values) => values.iter_mut().for_each(remove_counts),
                _ => {}
            }
        }
        let mut old: Value =
            serde_json::from_slice(&fs::read(archive.join("certificates.json")).unwrap()).unwrap();
        let mut new: Value =
            serde_json::from_slice(&fs::read(target.join("certificates.json")).unwrap()).unwrap();
        remove_counts(&mut old);
        remove_counts(&mut new);
        assert_eq!(old, new, "{family}: non-cost metadata changed");
    }
    assert!(changed > 0);
    eprintln!("Aggregate aliased consumers: {changed} reassociated vectors, {unchanged} exactly unchanged vectors");
}

#[test]
fn csharp_03_t06_w09_decimal_json_collection_definition_closure() {
    let dir = std::env::var("MPK_W09_DECIMAL_SHARED_OUT")
        .unwrap_or("decimal-json-collection-shared".into());
    let combined = certificate(&dir, "collection-json-decimal.hex");
    let shared = certificate("json-collection-shared", "collection-json.hex");
    let roots = shared
        .declarations
        .iter()
        .map(|d| shared.name_table[d.name as usize].clone())
        .collect();
    same_definition_closure(&shared, &combined, &roots).unwrap();
    let old = certificate(
        "decimal-parsers/previous-step-eight",
        "binding-vc-money.hex",
    );
    same_aggregate_scan(&old, &combined).unwrap();
}

#[test]
fn csharp_03_t06_w09_aggregate_regroup_audit_mutations() {
    let old = certificate("aggregate-folds/previous-step-eight", "core.hex");
    let dir = std::env::var("MPK_W09_AGGREGATE_OUT").unwrap_or("aggregate-folds".into());
    let new = certificate(&dir, "core.hex");
    same_aggregate_scan(&old, &new).unwrap();
    let value = |c: &Certificate, suffix: &str| {
        let d = c
            .declarations
            .iter()
            .find(|d| {
                c.name_table[d.name as usize]
                    == format!("Mpk.CSharp.Ordinary.AggregateFold.{suffix}")
            })
            .unwrap();
        let DeclarationKind::Def { value, .. } = d.kind else {
            panic!()
        };
        value
    };
    let mut body = value(&new, "StepEight");
    for _ in 0..3 {
        let TermNode::Lam { body: next, .. } = new.term_table[body as usize] else {
            panic!()
        };
        body = next;
    }
    let TermNode::App { ref arguments, .. } = new.term_table[body as usize] else {
        panic!()
    };
    let mut shorter = new.clone();
    shorter.term_table[body as usize] = new.term_table[arguments[0] as usize].clone();
    assert!(same_aggregate_scan(&old, &shorter).is_err(), "short group");
    for helper in ["StepTwo", "Active", "Compose"] {
        let mut changed = new.clone();
        changed.term_table[value(&new, helper) as usize] = TermNode::Var(0);
        assert!(
            same_aggregate_scan(&old, &changed).is_err(),
            "changed {helper}"
        );
    }
    let step = new
        .term_table
        .iter()
        .position(|t| {
            let TermNode::App {
                function,
                arguments,
            } = t
            else {
                return false;
            };
            let TermNode::Const { global, .. } = new.term_table[*function as usize] else {
                return false;
            };
            new.name_table[new.declarations[global as usize].name as usize]
                == "Mpk.CSharp.Ordinary.AggregateFold.StepTwo"
                && arguments.len() == 3
        })
        .unwrap();
    for arg in 0..3 {
        let mut changed = new.clone();
        let wrong = changed.term_table.len() as u32;
        changed.term_table.push(TermNode::Var(9));
        let TermNode::App { arguments, .. } = &mut changed.term_table[step] else {
            panic!()
        };
        arguments[arg] = wrong;
        assert!(
            same_aggregate_scan(&old, &changed).is_err(),
            "changed argument {arg}"
        );
    }
}

#[test]
fn csharp_03_t06_w09_structural_foundation_component_definition_closure() {
    let combined = read("ordinary-foundation/structural-foundations/certificates.json");
    assert_eq!(combined.as_array().unwrap().len(), 44);
    let mut contexts = 0;
    let mut root_occurrences = 0;
    let mut dependency_occurrences = 0;
    let mut mutations = false;
    for directory in [
        "domains",
        "defaults",
        "finite-operations",
        "sequence-operations",
        "construction-operations",
        "outcome-operations",
        "entry-operations",
        "collection-operations",
        "source-observations",
        "money-operations",
    ] {
        let rows = read(&format!(
            "ordinary-foundation/{directory}/certificates.json"
        ));
        for row in rows.as_array().unwrap() {
            let integrated = combined
                .as_array()
                .unwrap()
                .iter()
                .find(|r| {
                    r["metadata"]["source_ir_sha256"] == row["metadata"]["source_ir_sha256"]
                        && r["metadata"]["foundation_sha256"]
                            == row["metadata"]["foundation_sha256"]
                })
                .unwrap_or_else(|| panic!("uncovered component source: {directory}/{}", row["id"]));
            if directory == "source-observations" {
                assert_eq!(
                    row["metadata"]["definitions"], integrated["metadata"]["source_observations"],
                    "observation metadata must not select warmed semantic-equality cache entries"
                );
            }
            if directory == "money-operations" {
                assert_eq!(
                    row["metadata"]["definitions"],
                    integrated["metadata"]["money"]
                );
            }
            let l = certificate(directory, row["file"].as_str().unwrap());
            let r = certificate(
                "structural-foundations",
                integrated["file"].as_str().unwrap(),
            );
            let mut names = BTreeSet::new();
            roots(&row["metadata"], &mut names);
            let closure = same_definition_closure(&l, &r, &names)
                .unwrap_or_else(|e| panic!("{directory}/{}: {e}", row["id"]));
            contexts += 1;
            root_occurrences += names.len();
            dependency_occurrences += closure.len();
            if !mutations && !names.is_empty() {
                let root = names
                    .iter()
                    .find(|name| {
                        r.declarations.iter().any(|d| {
                            &r.name_table[d.name as usize] == *name
                                && matches!(d.kind, DeclarationKind::Def { .. })
                        })
                    })
                    .unwrap();
                let dependency = closure
                    .iter()
                    .find(|name| {
                        !names.contains(*name)
                            && r.declarations.iter().any(|d| {
                                &r.name_table[d.name as usize] == *name
                                    && matches!(d.kind, DeclarationKind::Def { .. })
                            })
                    })
                    .unwrap();
                for name in [root, dependency] {
                    let mut changed = r.clone();
                    let d = changed
                        .declarations
                        .iter_mut()
                        .find(|d| &r.name_table[d.name as usize] == name)
                        .unwrap();
                    let DeclarationKind::Def { ty, value, .. } = &mut d.kind else {
                        unreachable!()
                    };
                    *value = *ty;
                    assert!(
                        same_definition_closure(&l, &changed, &names).is_err(),
                        "ignored body mutation: {name}"
                    );
                }
                mutations = true;
            }
        }
    }
    assert_eq!(contexts, 118);
    assert!(mutations && root_occurrences > 0 && dependency_occurrences > root_occurrences);
    eprintln!("component/integrated definitions: {contexts} source-component pairs, {root_occurrences} roots, {dependency_occurrences} transitive declarations; root/dependency mutations rejected");
}

#[test]
fn csharp_03_t06_w09_money_composes_with_structural_foundations() {
    let bundle = b();
    let mut contexts = 0;
    let mut instances = 0;
    let mut operations = 0;
    let mut dependencies = 0;
    for (id, row, facts) in money_tests::sources() {
        let (context, captures) = support::replay_context(&bundle, &row);
        let source = ValidatedDataSource::import_captured_facts(
            &bundle,
            &context,
            &captures,
            &serde_json::to_vec(&facts).unwrap(),
        )
        .unwrap();
        let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
        if !emitted
            .closure()
            .closed()
            .entries()
            .iter()
            .any(|e| e["template_id"] == "mpk.csharp.semantic.money.v1")
        {
            continue;
        }
        let standalone = generate_csharp_practical_ordinary_money(emitted.vir()).unwrap();
        let combined = generate_csharp_practical_ordinary_structural_foundations(emitted.vir())
            .unwrap_or_else(|e| panic!("Money integration {id}: {e:?}"));
        assert_eq!(combined.money(), standalone.definitions());
        assert!(combined
            .deferred_instances()
            .iter()
            .all(|d| d.template_id != "mpk.csharp.semantic.money.v1"));
        let l = mpk_cert::decode_canonical_certificate(standalone.certificate_bytes()).unwrap();
        let r = mpk_cert::decode_canonical_certificate(combined.certificate_bytes()).unwrap();
        validate_csharp_practical_certificate_structure(&r).unwrap();
        assert!(combined.static_transformers() <= 16_384);
        let mut names = BTreeSet::new();
        roots(
            &serde_json::to_value(standalone.definitions()).unwrap(),
            &mut names,
        );
        let closure = same_definition_closure(&l, &r, &names)
            .unwrap_or_else(|e| panic!("Money integration {id}: {e}"));
        assert!(!names.is_empty());
        dependencies += closure.len();
        // A warmed structural relation/storage cache must not alter any Money
        // body, predicate binder, failure order or transitive definition.
        let metadata: Value = serde_json::from_slice(&combined.canonical_bytes()).unwrap();
        for mutation in ["currency_predicate", "failure_order"] {
            let mut forged = metadata.clone();
            let create = forged["money"][0]["operations"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|o| o["operation_id"].as_str().unwrap().ends_with(".create"))
                .unwrap();
            match mutation {
                "currency_predicate" => {
                    assert!(!create["currency_predicate_argument_type_id"].is_null());
                    create["currency_predicate_argument_type_id"] = Value::Null;
                }
                "failure_order" => create["failures"].as_array_mut().unwrap().swap(0, 1),
                _ => unreachable!(),
            }
            assert!(
                import_csharp_practical_ordinary_structural_foundations(
                    &serde_json::to_vec(&forged).unwrap(),
                    combined.certificate_bytes(),
                    emitted.vir()
                )
                .is_err(),
                "accepted {mutation} mutation: {id}"
            );
        }
        for d in combined.money() {
            instances += 1;
            operations += d.operations.len();
        }
        contexts += 1;
        eprintln!(
            "integrated Money {id}: {} instances, {} terms, {} declarations, {} transformers",
            combined.money().len(),
            r.term_table.len(),
            r.declarations.len(),
            combined.static_transformers()
        );
    }
    assert_eq!((contexts, instances, operations), (2, 3, 30));
    eprintln!("Money integration: {contexts} sources, {instances} instances, {operations} operations, {dependencies} transitive declarations; currency-predicate and failure-order mutations rejected");
}
