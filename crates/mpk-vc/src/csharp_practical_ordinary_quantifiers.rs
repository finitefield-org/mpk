//! Half-open finite integer ranges and their W03 definedness quantification.
use super::*;
const INDEX_BITS: u32 = 14;
const CAPACITY: i128 = 1 << INDEX_BITS;

#[derive(Clone)]
pub(super) struct QuantifierFunctions {
    bound: String,
    forall: String,
    exists: String,
}

// Reject a statically known oversized expansion before emitting any network.
// Symbolic ranges carry the same full-width bound predicate in W03; no range
// is silently truncated to the physical network's capacity.
pub(super) fn validate_static_ranges(expression: &VerifiedContractExpression) -> R<()> {
    fn literal(term: &ContractTerm, definitions: &[ContractDefinition]) -> Option<i128> {
        let ContractTerm::Const { name, .. } = term else {
            return None;
        };
        let d = definitions
            .iter()
            .find(|d| &d.name == name && d.tag == "literal")?;
        let p: Value = serde_json::from_str(&d.parameters).ok()?;
        p["value"].as_str()?.parse().ok()
    }
    fn visit(term: &ContractTerm, definitions: &[ContractDefinition]) -> R<()> {
        let mut head = term;
        let mut args = vec![];
        while let ContractTerm::App {
            function, argument, ..
        } = head
        {
            args.push(argument.as_ref());
            head = function;
        }
        args.reverse();
        if let ContractTerm::Const { name, .. } = head {
            if definitions.iter().any(|d| {
                &d.name == name && matches!(d.tag.as_str(), "bounded_forall" | "bounded_exists")
            }) && args.len() == 3
            {
                if let (Some(lower), Some(upper)) =
                    (literal(args[0], definitions), literal(args[1], definitions))
                {
                    if upper - lower > CAPACITY {
                        return Err(OrdinaryCarrierError::Limit);
                    }
                }
            }
        }
        match term {
            ContractTerm::App {
                function, argument, ..
            } => {
                visit(function, definitions)?;
                visit(argument, definitions)?;
            }
            ContractTerm::Lam { body, .. } => visit(body, definitions)?,
            ContractTerm::Let { value, body, .. } => {
                visit(value, definitions)?;
                visit(body, definitions)?;
            }
            _ => {}
        }
        Ok(())
    }
    visit(expression.term(), expression.definitions())
}

fn scalar(c: &mut Clauses<'_>, id: &str) -> R<String> {
    if !c.scalars.contains_key(id) {
        let d = super::super::super::scalar_bits::emit_integer(&mut c.b, id)?;
        let checks = d.ordered_failure_definitions.clone();
        c.scalars.insert(id.into(), (d, checks));
    }
    let (d, _) = &c.scalars[id];
    if !d.operation.ordered_checks.is_empty() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(d.result_definition.clone())
}

fn emit(c: &mut Clauses<'_>, type_id: &str) -> R<QuantifierFunctions> {
    if let Some(functions) = c.quantifiers.get(type_id) {
        return Ok(functions.clone());
    }
    let (token, unsigned, width, depth) = match type_id {
        "mpk.csharp.value.i32.v1" => ("i32", "u32", 32, 5),
        "mpk.csharp.value.u32.v1" => ("u32", "u32", 32, 5),
        "mpk.csharp.value.i64.v1" => ("i64", "u64", 64, 6),
        "mpk.csharp.value.u64.v1" => ("u64", "u64", 64, 6),
        _ => return Err(OrdinaryCarrierError::Linkage),
    };
    let carrier = c
        .carriers
        .get(type_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    if carrier.depth != depth || carrier.shape != (OrdinaryShape::Bits { width }) {
        return Err(OrdinaryCarrierError::Shape);
    }
    let ordered = scalar(c, &format!("integer.{token}.less_equal.checked"))?;
    let subtract = scalar(c, &format!("integer.{unsigned}.subtract.unchecked"))?;
    let within = scalar(c, &format!("integer.{unsigned}.less_equal.checked"))?;
    let add = scalar(c, &format!("integer.{unsigned}.add.unchecked"))?;
    let fold = ordered_fold::emit_fold(&mut c.b, INDEX_BITS)?;
    let base = format!("{PREFIX}.ContractQuantifier.{token}");
    let functions = QuantifierFunctions {
        bound: format!("{base}.Bound"),
        forall: format!("{base}.Forall"),
        exists: format!("{base}.Exists"),
    };
    let count_name = format!("{base}.Count");
    let lower = c.b.var(1)?;
    let upper = c.b.var(0)?;
    let in_order = call(&mut c.b, &ordered, vec![lower, upper])?;
    // Modular subtraction is the exact unsigned mathematical distance when
    // signed/unsigned lower <= upper, including signed ranges crossing zero.
    let distance = call(&mut c.b, &subtract, vec![upper, lower])?;
    let cap = equal_address(&mut c.b, depth, 0, depth, INDEX_BITS)?;
    let cap = c.b.wrap_selectors(depth, cap)?;
    let fits = call(&mut c.b, &within, vec![distance, cap])?;
    let bound = call(&mut c.b, "Std.Bool.and", vec![in_order, fits])?;
    define(&mut c.b, &functions.bound, &[depth, depth], 0, bound)?;
    // The count is zero on invalid bounds, before predicate traversal. Check
    // all 64 distance bits before projecting the low count word.
    let lower = c.b.var(1)?;
    let upper = c.b.var(0)?;
    let distance_value = call(&mut c.b, &subtract, vec![upper, lower])?;
    let lower = c.b.var(2)?;
    let upper = c.b.var(1)?;
    let bound_value = call(&mut c.b, &functions.bound, vec![lower, upper])?;
    // Share distance and the range check outside the count's selector lambdas.
    let distance = c.b.var(6)?;
    let mut selectors = c.b.selectors(5)?;
    if depth == 6 {
        selectors.push(bit(&mut c.b, false)?);
    }
    let value = c.b.app(distance, selectors)?;
    let bound = c.b.var(5)?;
    let leaf = call(&mut c.b, "Std.Bool.and", vec![bound, value])?;
    let count = c.b.wrap_selectors(5, leaf)?;
    let bool_ty = c.b.cube(0)?;
    let count = c.b.term(TermNode::Let {
        ty: bool_ty,
        value: bound_value,
        body: count,
    })?;
    let word_ty = c.b.cube(depth)?;
    let count = c.b.term(TermNode::Let {
        ty: word_ty,
        value: distance_value,
        body: count,
    })?;
    define(&mut c.b, &count_name, &[depth, depth], 5, count)?;
    let args = vec![
        type_id.into(),
        type_id.into(),
        format!("({type_id}->{SOURCE_BOOL})"),
    ];
    for (name, reducer) in [
        (&functions.forall, &fold.all_definition),
        (&functions.exists, &fold.any_definition),
    ] {
        // Predicate index j is an offset. Translate it to lower+j in the
        // original nominal integer carrier without narrowing signed/64-bit data.
        let mut leaf = bit(&mut c.b, false)?;
        for i in 0..INDEX_BITS {
            let at = equal_address(&mut c.b, depth, 0, depth, i)?;
            let value = c.b.var(depth + INDEX_BITS - 1 - i)?;
            leaf = mux(&mut c.b, at, value, leaf)?;
        }
        let offset = c.b.wrap_selectors(depth, leaf)?;
        let lower = c.b.var(INDEX_BITS + 2)?;
        let index = call(&mut c.b, &add, vec![lower, offset])?;
        let predicate = c.b.var(INDEX_BITS)?;
        let value = c.b.app(predicate, vec![index])?;
        let predicate = c.b.wrap_selectors(INDEX_BITS, value)?;
        let lower = c.b.var(2)?;
        let upper = c.b.var(1)?;
        let count = call(&mut c.b, &count_name, vec![lower, upper])?;
        let value = call(&mut c.b, reducer, vec![predicate, count])?;
        let bound = call(&mut c.b, &functions.bound, vec![lower, upper])?;
        let mut body = call(&mut c.b, "Std.Bool.and", vec![bound, value])?;
        for arg in args.iter().rev() {
            let ty = c.ty(arg, 0)?;
            body = c.b.lam(ty, body)?;
        }
        let ty = c.ty(&signature(&args, SOURCE_BOOL), 0)?;
        c.b.define(name, ty, body)?;
    }
    c.quantifiers.insert(type_id.into(), functions.clone());
    Ok(functions)
}

pub(super) fn recipe(c: &mut Clauses<'_>, d: &ContractDefinition, params: &Value) -> R<u32> {
    if params != &serde_json::json!({})
        || d.argument_types.len() != 3
        || d.result_type != SOURCE_BOOL
        || d.argument_types[0] != d.argument_types[1]
        || d.argument_types[2] != format!("({}->{SOURCE_BOOL})", d.argument_types[0])
    {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let functions = emit(c, &d.argument_types[0])?;
    c.constants.insert(
        format!("Mpk.CSharp.Data.ContractBound.{}", d.name),
        (
            signature(&d.argument_types[..2], SOURCE_BOOL),
            functions.bound,
        ),
    );
    c.constants.insert(
        format!("Mpk.CSharp.Data.ContractForall.{}", d.name),
        (
            signature(&d.argument_types, SOURCE_BOOL),
            functions.forall.clone(),
        ),
    );
    c.b.constant(if d.tag == "bounded_forall" {
        &functions.forall
    } else {
        &functions.exists
    })
}

#[cfg(test)]
mod tests {
    use super::super::super::super::test_eval::{apply, bit as observed, eval, run, V};
    use super::*;
    use crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure;
    type QuantifierRow = (String, QuantifierFunctions, String, String);
    fn fixture() -> (mpk_cert::Certificate, Vec<QuantifierRow>, usize) {
        let types = [
            ("bool", 1, 0),
            ("i32", 32, 5),
            ("u32", 32, 5),
            ("i64", 64, 6),
            ("u64", 64, 6),
        ]
        .map(|(token, width, depth)| OrdinaryCarrier {
            type_id: format!("mpk.csharp.value.{token}.v1"),
            depth,
            shape: OrdinaryShape::Bits { width },
        });
        let mut c = Clauses {
            vir: None,
            relations: relations::ContractRelationCache::default(),
            b: Builder::new().unwrap(),
            carriers: types.iter().map(|c| (c.type_id.as_str(), c)).collect(),
            storage: StorageCache::default(),
            constants: BTreeMap::new(),
            scalars: BTreeMap::new(),
            strings: Default::default(),
            finite: None,
            codecs: Default::default(),
            binding_projections: None,
            quantifiers: BTreeMap::new(),
        };
        let mut rows = vec![];
        for carrier in &types[1..] {
            let token = carrier
                .type_id
                .strip_prefix("mpk.csharp.value.")
                .unwrap()
                .strip_suffix(".v1")
                .unwrap();
            let functions = emit(&mut c, &carrier.type_id).unwrap();
            let eq = scalar(&mut c, &format!("integer.{token}.equal.checked")).unwrap();
            let truth = format!("Test.Quantifier.{token}.True");
            let body = bit(&mut c.b, true).unwrap();
            define(&mut c.b, &truth, &[carrier.depth], 0, body).unwrap();
            rows.push((token.into(), functions, eq, truth));
        }
        let transformers = c.b.static_transformers;
        let bytes = c.b.finish().unwrap();
        let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        validate_csharp_practical_certificate_structure(&cert).unwrap();
        let metadata = serde_json::to_vec_pretty(&json!({
            "certificate_sha256": mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes)),
            "static_transformers": transformers,
            "terms": cert.term_table.len(),
            "declarations": cert.declarations.len()
        }))
        .unwrap();
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        for (name, content) in [
            ("core.hex", hex.as_bytes()),
            ("core.json", metadata.as_slice()),
        ] {
            if let Some(out) = std::env::var_os("MPK_W09_QUANTIFIER_CORE_OUT") {
                let out = std::path::PathBuf::from(out);
                std::fs::create_dir_all(&out).unwrap();
                std::fs::write(out.join(name), content).unwrap();
            } else {
                let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../develop/migrations/csharp-03/ordinary-foundation/quantifiers");
                assert_eq!(std::fs::read(root.join(name)).unwrap(), content, "{name}");
            }
        }
        (cert, rows, transformers)
    }
    fn word(value: i128, width: usize) -> V {
        V::Cube(
            (0..width)
                .map(|i| (value as u128) & (1 << i) != 0)
                .collect(),
        )
    }
    fn number(c: &mpk_cert::Certificate, value: V) -> u32 {
        (0..32).fold(0, |out, i| {
            let mut leaf = value.clone();
            for k in 0..5 {
                leaf = apply(c, leaf, V::Bit(i & (1 << k) != 0));
            }
            out | ((observed(leaf) as u32) << i)
        })
    }
    #[test]
    fn ordinary_quantifier_ranges_and_values() {
        let (c, rows, transformers) = fixture();
        assert!(transformers <= 16384);
        let mut cases = 0;
        for (token, f, eq, truth) in rows {
            let width = if token.ends_with("32") { 32 } else { 64 };
            let signed = token.starts_with('i');
            let min = if signed { -(1i128 << (width - 1)) } else { 0 };
            let max = if signed {
                (1i128 << (width - 1)) - 1
            } else {
                (1i128 << width) - 1
            };
            for (lower, upper) in [
                (0, 0),
                (0, 1),
                (3, 7),
                (7, 3),
                (min, min),
                (min, min + 3),
                (max - 3, max),
                (max, max),
                (0, 16384),
                (0, 16385),
                (min, max),
            ] {
                let args = vec![word(lower, width), word(upper, width)];
                let valid = lower <= upper && upper - lower <= CAPACITY;
                assert_eq!(
                    observed(run(&c, &f.bound, args.clone())),
                    valid,
                    "{token} {lower}..{upper}"
                );
                assert_eq!(
                    number(
                        &c,
                        run(
                            &c,
                            &format!("{PREFIX}.ContractQuantifier.{token}.Count"),
                            args
                        )
                    ),
                    if valid { (upper - lower) as u32 } else { 0 }
                );
                cases += 1;
            }
            for (lower, upper) in [
                (0, 0),
                (0, 1),
                (3, 7),
                (7, 3),
                (min, min + 3),
                (max - 3, max),
            ] {
                let valid = lower <= upper;
                let always = run(&c, &truth, vec![]);
                assert_eq!(
                    observed(run(
                        &c,
                        &f.forall,
                        vec![word(lower, width), word(upper, width), always.clone()]
                    )),
                    valid
                );
                assert_eq!(
                    observed(run(
                        &c,
                        &f.exists,
                        vec![word(lower, width), word(upper, width), always]
                    )),
                    valid && lower < upper
                );
                for target in [lower, upper, lower + (upper - lower).max(0) / 2] {
                    let predicate = run(&c, &eq, vec![word(target, width)]);
                    assert_eq!(
                        observed(run(
                            &c,
                            &f.exists,
                            vec![word(lower, width), word(upper, width), predicate.clone()]
                        )),
                        valid && lower <= target && target < upper,
                        "{token} exists {target} in {lower}..{upper}"
                    );
                    assert_eq!(
                        observed(run(
                            &c,
                            &f.forall,
                            vec![word(lower, width), word(upper, width), predicate]
                        )),
                        valid && (lower == upper || upper - lower == 1 && target == lower)
                    );
                    cases += 1;
                }
            }
        }
        eprintln!("quantifier core range/value cases:{cases};transformers:{transformers}");
    }
    #[test]
    fn ordinary_quantifier_full_capacity() {
        let (c, rows, _) = fixture();
        let (_, f, _, truth) = rows.iter().find(|r| r.0 == "u64").unwrap();
        let low = (1i128 << 63) + 17;
        assert!(observed(run(
            &c,
            &f.forall,
            vec![
                word(low, 64),
                word(low + CAPACITY, 64),
                run(&c, truth, vec![])
            ]
        )));
    }
    #[test]
    fn ordinary_quantifier_skips_body_for_empty_or_invalid_ranges() {
        let (c, rows, _) = fixture();
        for (token, f, _, _) in rows {
            let width = if token.ends_with("32") { 32 } else { 64 };
            for (lower, upper) in [(0, 0), (7, 3), (0, 16385)] {
                // A non-function deliberately panics if the core ever applies
                // this body. Empty/invalid bounds must be decided first.
                let args = vec![word(lower, width), word(upper, width), V::Bit(true)];
                assert_eq!(observed(run(&c, &f.forall, args.clone())), lower == upper);
                assert!(!observed(run(&c, &f.exists, args)));
            }
        }
    }
    #[test]
    fn ordinary_quantifier_offset_selector_bits() {
        let (c, rows, _) = fixture();
        let predicate_cube = |name: &str| {
            let declaration = c
                .declarations
                .iter()
                .find(|d| c.name_table[d.name as usize] == name)
                .unwrap();
            let mpk_cert::encode::DeclarationKind::Def { mut value, .. } = declaration.kind else {
                panic!()
            };
            for _ in 0..3 {
                let TermNode::Lam { body, .. } = c.term_table[value as usize] else {
                    panic!()
                };
                value = body;
            }
            let TermNode::App {
                function,
                arguments,
            } = &c.term_table[value as usize]
            else {
                panic!()
            };
            let TermNode::Const { global, .. } = c.term_table[*function as usize] else {
                panic!()
            };
            assert_eq!(
                c.name_table[c.declarations[global as usize].name as usize],
                "Std.Bool.and"
            );
            let TermNode::App { arguments, .. } = &c.term_table[arguments[1] as usize] else {
                panic!()
            };
            arguments[0]
        };
        let offsets = (0..INDEX_BITS)
            .map(|i| 1u32 << i)
            .chain([0, 3, 15, 31, 255, 16383])
            .collect::<BTreeSet<_>>();
        let mut observations = 0;
        for (token, f, eq, _) in rows {
            // Inspect the actual emitted fold argument, preserving its exact
            // binder environment, rather than rebuilding the offset formula.
            let cube = predicate_cube(&f.forall);
            assert_eq!(cube, predicate_cube(&f.exists));
            let width = if token.ends_with("32") { 32 } else { 64 };
            let signed = token.starts_with('i');
            let max = if signed {
                (1i128 << (width - 1)) - 1
            } else {
                (1i128 << width) - 1
            };
            for lower in [if signed { -8192 } else { 0 }, max - CAPACITY] {
                for &offset in &offsets {
                    for matches in [true, false] {
                        let target = lower + i128::from(offset) + i128::from(!matches);
                        let predicate = run(&c, &eq, vec![word(target, width)]);
                        let mut value = eval(
                            &c,
                            cube,
                            &[predicate, word(lower + CAPACITY, width), word(lower, width)],
                        );
                        for i in 0..INDEX_BITS {
                            value = apply(&c, value, V::Bit(offset & (1 << i) != 0));
                        }
                        assert_eq!(
                            observed(value),
                            matches,
                            "{token} lower{lower} offset{offset} target{target}"
                        );
                        observations += 1;
                    }
                }
            }
        }
        eprintln!("quantifier actual offset selector observations: {observations}");
    }
}
