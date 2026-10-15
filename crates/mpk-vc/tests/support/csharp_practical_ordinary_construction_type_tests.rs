use super::*;
use core_eval::{bit, run, sparse_cube};
use sequence_tests::number;

// Independently encode the original three-field storage layout in low-first
// addresses. Values are sparse so nested full-capacity carriers stay practical.
fn storage(
    length: u32,
    child: u32,
    assigned: &BTreeMap<usize, BTreeSet<usize>>,
) -> BTreeSet<usize> {
    let mut ones = BTreeSet::new();
    for bit in 0..32 {
        if length & (1 << bit) != 0 {
            ones.insert(bit << (11 + child));
        }
    }
    for (&index, value) in assigned {
        ones.insert(2 | (index << (2 + child)));
        for &leaf in value {
            ones.insert(1 | (index << 2) | (leaf << 16));
        }
    }
    ones
}

// Retain the exact source product and its nullable member. Only the existing
// string payload changes; nominal/source/closed instance identities stay fixed.
fn set_string_length(value: &mut MonomorphicValue, length: usize) -> usize {
    match value {
        MonomorphicValue::String { utf16, .. } => {
            *utf16 = vec![0; length];
            1
        }
        MonomorphicValue::Product { fields, .. } => fields
            .iter_mut()
            .map(|f| set_string_length(&mut f.value, length))
            .sum(),
        MonomorphicValue::Option {
            value: Some(value), ..
        } => set_string_length(value, length),
        MonomorphicValue::TaggedSum { payload, .. } => payload
            .iter_mut()
            .map(|v| set_string_length(v, length))
            .sum(),
        _ => 0,
    }
}
fn logical_cells(value: &MonomorphicValue) -> u32 {
    match value {
        MonomorphicValue::String { utf16, .. } => 1 + utf16.len() as u32,
        MonomorphicValue::Product { fields, .. } => {
            1 + fields.iter().map(|f| logical_cells(&f.value)).sum::<u32>()
        }
        MonomorphicValue::Option { value, .. } => {
            1 + value.as_deref().map(logical_cells).unwrap_or(0)
        }
        MonomorphicValue::TaggedSum { payload, .. } => {
            1 + payload.iter().map(logical_cells).sum::<u32>()
        }
        _ => 1,
    }
}

#[test]
fn csharp_03_t06_w09_construction_storage_domains_original_semantics() {
    std::thread::Builder::new().stack_size(64 * 1024 * 1024).spawn(|| {
        let bundle = b();
        let mut contexts = 0;
        let mut observations = 0;
        let mut original = sources();
        original.sort_by_key(|(id, _, _)| id != "nullable-string-default");
        for (id, row, facts) in original {
            if !["bool-construction", "enum-construction", "float-sequence", "nested-box", "nullable-string-default", "product-construction"].contains(&id.as_str()) {
                continue;
            }
            let (context, captures) = support::replay_context(&bundle, &row);
            let source = ValidatedDataSource::import_captured_facts(&bundle, &context, &captures, &serde_json::to_vec(&facts).unwrap()).unwrap();
            let emitted = emit_data_phase(&bundle, &context, &captures, &source).unwrap();
            let p = generate_csharp_practical_ordinary_concrete_types_with_construction_storage(emitted.vir()).unwrap();
            let cert = mpk_cert::decode_canonical_certificate(p.certificate_bytes()).unwrap();
            let independent = generate_csharp_practical_ordinary_public_domains(emitted.vir()).unwrap();
            let public = mpk_cert::decode_canonical_certificate(independent.certificate_bytes()).unwrap();
            let types = p.public_domains().iter().map(|d| (d.carrier.type_id.clone(), d.carrier.clone())).collect::<BTreeMap<_, _>>();
            assert_eq!(p.construction_storage_domains().len(), 1);
            let d = &p.construction_storage_domains()[0];
            assert!(d.private_storage_only && d.ownership_pending);
            assert!(!p.public_domains().iter().any(|x| x.carrier.type_id == d.carrier.type_id));
            let child = types[&d.element_type_id].depth;
            let depth = d.carrier.depth;
            let element = independent.definitions().iter().find(|x| x.carrier.type_id == d.element_type_id).unwrap();
            let mut check = |label: &str, ones: BTreeSet<usize>, expected: bool, cells: u32| {
                eprintln!("{id}: private domain {label}");
                let value = sparse_cube(depth, ones);
                assert_eq!(bit(run(&cert, &d.valid_definition, vec![value.clone()])), expected, "{id}: {label}");
                assert_eq!(number(&cert, run(&cert, &d.count_definition, vec![value])), cells, "{id}: {label}");
                observations += 1;
            };
            for length in [0, 1, 2] {
                check("uninitialized zero storage", storage(length, child, &BTreeMap::new()), true, 1 + length);
            }
            for length in [16385, u32::MAX, 1 << 31] {
                check("invalid complete length word", storage(length, child, &BTreeMap::new()), false, 65537);
            }
            for seed in 0..3 {
                let value = relation_tests::sample(&d.element_type_id, seed, &types, &facts, emitted.closure().closed());
                validate_monomorphic_value(&bundle, emitted.closure().roots(), emitted.closure().closed(), &value).unwrap();
                let (value_depth, ones) = projection_tests::sparse_storage(&value, &types);
                assert_eq!(value_depth, child);
                let valid = bit(run(&public, &element.valid_definition, vec![sparse_cube(child, ones.clone())]));
                let cells = number(&public, run(&public, &element.count_definition, vec![sparse_cube(child, ones.clone())]));
                let expected_count = (2 + cells).min(65537);
                check("assigned exact child public domain and recursive count", storage(2, child, &BTreeMap::from([(1, ones)])), valid && expected_count < 65537, expected_count);
            }
            for leaf in [0, (1usize << child) - 1] {
                let mut ones = storage(1, child, &BTreeMap::new());
                ones.insert(1 | (leaf << 16));
                check("nonzero uninitialized cell", ones, false, 65537);
                let mut ones = storage(1, child, &BTreeMap::new());
                ones.insert(1 | (16383 << 2) | (leaf << 16));
                check("inactive final physical cell", ones, false, 65537);
            }
            for index in [1usize, 16383] {
                let mut ones = storage(1, child, &BTreeMap::new());
                ones.insert(2 | (index << (2 + child)));
                check("inactive initialization bitmap", ones, false, 65537);
            }
            for (role, width) in [(0usize, 5), (1, 14 + child), (2, 14)] {
                for bit_index in 2..depth - width {
                    let mut ones = storage(1, child, &BTreeMap::new());
                    ones.insert(role | (1 << bit_index));
                    check("field padding selector branch", ones, false, 65537);
                }
            }
            for leaf in [3, (1usize << depth) - 1] {
                let mut ones = storage(1, child, &BTreeMap::new());
                ones.insert(leaf);
                check("unused fourth role", ones, false, 65537);
            }
            if id == "bool-construction" {
                for length in [16383, 16384] {
                    check("full construction bound with every uninitialized cell", storage(length, child, &BTreeMap::new()), true, 1 + length);
                }
            }
            if id == "nullable-string-default" {
                // Four exact source products with their actual nullable-string
                // members reach the independently counted 65,536-cell bound.
                let mut empty = relation_tests::sample(&d.element_type_id, 1, &types, &facts, emitted.closure().closed());
                assert_eq!(set_string_length(&mut empty, 0), 1);
                let fixed = logical_cells(&empty);
                let at = (65536 - 1 - 3 * 16384 - 4 * fixed) as usize;
                for last in [at - 1, at, at + 1] {
                    let mut assigned = BTreeMap::new();
                    let mut total = 1;
                    for (index, length) in [16384, 16384, 16384, last].into_iter().enumerate() {
                        let mut value = empty.clone();
                        assert_eq!(set_string_length(&mut value, length), 1);
                        validate_monomorphic_value(&bundle, emitted.closure().roots(), emitted.closure().closed(), &value).unwrap();
                        let (_, ones) = projection_tests::sparse_storage(&value, &types);
                        let actual = sparse_cube(child, ones.clone());
                        assert!(bit(run(&public, &element.valid_definition, vec![actual.clone()])));
                        assert_eq!(number(&public, run(&public, &element.count_definition, vec![actual])), logical_cells(&value));
                        total += logical_cells(&value);
                        assigned.insert(index, ones);
                    }
                    assert_eq!(total, 65536 + last as u32 - at as u32);
                    let count = total.min(65537);
                    check("aggregate live-cell bound minus/at/plus one", storage(4, child, &assigned), count < 65537, count);
                }
            }
            contexts += 1;
        }
        assert_eq!(contexts, 6);
        assert!(observations > 100);
        eprintln!("private construction domains: {contexts} original contexts, {observations} complete observations");
    }).unwrap().join().unwrap();
}
