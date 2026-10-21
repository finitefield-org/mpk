//! Actual ordinary metadata/guard evaluation; no source/store authority.
use super::actual::Interpreter;
use mpk_cert::encode::Certificate;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
type Retain = fn(&str, &str, &[u8]);
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    nil: bool,
    offset: u64,
    length: u64,
    capacity: u64,
    index: u64,
    low: u64,
    high: u64,
    max: u64,
    bytes: Vec<u8>,
}
#[derive(Clone)]
struct Model {
    case: Case,
    owner: u64,
    kinds: [u8; 16],
    indices: [u64; 16],
}
fn dc(name: &str) -> String {
    format!("Mpk.Go.Practical.Slice.Descriptor.{name}")
}
fn save(retain: Retain, name: &str, ext: &str, value: Value) {
    retain(
        name,
        ext,
        format!("{}\n", serde_json::to_string_pretty(&value).unwrap()).as_bytes(),
    );
}
fn bits(n: u64) -> Vec<bool> {
    (0..64).map(|i| n & (1 << i) != 0).collect()
}
fn word(e: &mut Interpreter<'_>, n: u64) -> u32 {
    let raw = e.raw(n, [0; 256]);
    assert_eq!(e.length(raw), n);
    e.invoke("Mpk.Go.Practical.String.Length", &[raw])
}
fn nat(e: &mut Interpreter<'_>, n: u64) -> u32 {
    assert!(n <= 1024);
    let mut v = e.invoke("Std.Nat.zero", &[]);
    for _ in 0..n {
        v = e.invoke("Std.Nat.succ", &[v]);
    }
    v
}
fn read_word(e: &mut Interpreter<'_>, w: u32, width: usize) -> u64 {
    let mut result = 0;
    for i in 0..width {
        let mut args = vec![w];
        args.extend(
            (0..width.trailing_zeros())
                .rev()
                .map(|j| u32::from(i & (1 << j) != 0)),
        );
        let v = e.invoke(&dc(&format!("Raw{width}Bit")), &args);
        if e.boolean(v) {
            result |= 1u64 << i;
        }
    }
    result
}
fn path(e: &mut Interpreter<'_>, model: &Model) -> u32 {
    let bool_type = e.invoke("Std.Bool", &[]);
    let a = e.invoke("Mpk.Go.Practical.FiniteVector.D3.Type", &[bool_type]);
    let b = e.invoke("Mpk.Go.Practical.FiniteVector.D6.Type", &[bool_type]);
    let mut cells = vec![];
    for i in 0..16 {
        let kind = e.byte_input(model.kinds[i]);
        let index = word(e, model.indices[i]);
        cells.push(e.invoke("Mpk.Go.Practical.Product.Make", &[a, b, kind, index]));
    }
    e.invoke(&dc("PathMake"), &cells)
}
fn input<'a>(c: &'a Certificate, model: &Model) -> (Interpreter<'a>, u32) {
    let mut e = Interpreter::new(c);
    let row = &model.case;
    assert_eq!(row.bytes.len(), 256);
    let raw = e.raw(row.length, row.bytes.clone().try_into().unwrap());
    assert_eq!(e.length(raw), row.length);
    assert_eq!(e.bytes(raw).to_vec(), row.bytes);
    let owner = nat(&mut e, model.owner);
    let path = path(&mut e, model);
    let offset = word(&mut e, row.offset);
    let length = word(&mut e, row.length);
    let capacity = word(&mut e, row.capacity);
    let s = e.invoke(
        &dc("Make"),
        &[u32::from(row.nil), owner, path, offset, length, capacity],
    );
    (e, s)
}
fn read_path(e: &mut Interpreter<'_>, path: u32) -> (Vec<u8>, Vec<u64>) {
    let (mut kinds, mut indices) = (vec![], vec![]);
    for i in 0..16 {
        let mut args = vec![path];
        args.extend((0..4).rev().map(|j| u32::from(i & (1 << j) != 0)));
        let k = e.invoke(&dc("PathKind"), &args);
        let n = e.invoke(&dc("PathIndex"), &args);
        kinds.push(read_word(e, k, 8) as u8);
        indices.push(read_word(e, n, 64));
    }
    (kinds, indices)
}
fn observe(e: &mut Interpreter<'_>, s: u32, owner: u64) -> Value {
    let nil = e.invoke(&dc("NilTag"), &[s]);
    let nil = e.boolean(nil);
    let actual_owner = e.invoke(&dc("Owner"), &[s]);
    let expected_owner = nat(e, owner);
    let equal = e.invoke(
        "Mpk.Go.Practical.Nat.Equal",
        &[actual_owner, expected_owner],
    );
    let owner_matches = e.boolean(equal);
    let p = e.invoke(&dc("Path"), &[s]);
    let (kinds, indices) = read_path(e, p);
    let mut words = vec![];
    for name in ["Offset", "Length", "Capacity"] {
        let w = e.invoke(&dc(name), &[s]);
        words.push(read_word(e, w, 64));
    }
    json!({"nil":nil,"owner_matches_expected":owner_matches,"path_kinds":kinds,"path_indices":indices,"offset":words[0],"length":words[1],"capacity":words[2]})
}
fn expected(model: &Model) -> Value {
    json!({"nil":model.case.nil,"owner_matches_expected":true,"path_kinds":model.kinds,"path_indices":model.indices,"offset":model.case.offset,"length":model.case.length,"capacity":model.case.capacity})
}
fn retain_input(retain: Retain, name: &str, model: &Model, raw_path: bool) {
    let r = &model.case;
    save(
        retain,
        name,
        "input.json",
        json!({"complete_original_case":r,"owner_nat":model.owner,"all16_path_kinds":model.kinds,"all16_full64_path_indices":model.indices,"all64_path_index_bits":model.indices.map(bits),"all64_offset_bits":bits(r.offset),"all64_length_bits":bits(r.length),"all64_capacity_bits":bits(r.capacity),"all64_index_bits":bits(r.index),"all64_low_bits":bits(r.low),"all64_high_bits":bits(r.high),"all64_max_bits":bits(r.max),"raw_unvalidated_typed_path":raw_path,"ordinary_source_or_nominal_authority":false,"typed_live_storage_or_permission_proof":false,"admitted_source_counterexample":false}),
    );
}
fn sample(seed: u64) -> Model {
    Model {
        case: Case {
            id: format!("raw-{seed:02}"),
            nil: false,
            offset: 3,
            length: 17,
            capacity: 32,
            index: 4,
            low: 1,
            high: 0,
            max: 0,
            bytes: (0..256).map(|i| (i * 73 + seed as usize) as u8).collect(),
        },
        owner: 7,
        kinds: std::array::from_fn(|i| (i + 16 * seed as usize) as u8),
        indices: std::array::from_fn(|i| {
            if i == 15 {
                u64::MAX
            } else {
                ((i as u64) << 60) ^ (seed * 0x0102_0304_0506_0708)
            }
        }),
    }
}
fn guards(e: &mut Interpreter<'_>, s: u32, row: &Case) -> Vec<bool> {
    let i = word(e, row.index);
    let lo = word(e, row.low);
    let hi = word(e, row.high);
    let max = word(e, row.max);
    let mut out = vec![];
    for name in [
        "IndexUnsigned",
        "IndexSigned",
        "TwoUnsigned",
        "TwoSigned",
        "ThreeUnsigned",
        "ThreeSigned",
    ] {
        let args = if name.starts_with("Index") {
            vec![s, i]
        } else if name.starts_with("Two") {
            vec![s, lo, hi]
        } else {
            vec![s, lo, hi, max]
        };
        let v = e.invoke(&dc(name), &args);
        out.push(e.boolean(v));
    }
    out
}
pub(super) fn run(c: &Certificate, retain: Retain) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../develop/migrations/go-practical/t02-w01/verification/slice-descriptor/native-go",
    );
    let cases: Vec<Case> =
        serde_json::from_slice(&std::fs::read(root.join("cases.json")).unwrap()).unwrap();
    let native: Vec<Value> =
        serde_json::from_slice(&std::fs::read(root.join("bounds.stdout.log")).unwrap()).unwrap();
    assert_eq!(cases.len(), 400);
    assert_eq!(native.len(), cases.len());
    let (mut steps, mut outputs, mut guard_results) = (0, 0, 0);
    for (row, go) in cases.into_iter().zip(native) {
        assert_eq!(go["id"].as_str(), Some(row.id.as_str()));
        let name = format!("native-{}", row.id);
        let model = Model {
            owner: u64::from(!row.nil),
            case: row,
            kinds: [0; 16],
            indices: [0; 16],
        };
        retain_input(retain, &name, &model, false);
        let (mut e, s) = input(c, &model);
        let wanted = expected(&model);
        for mode in ["raw", "copy", "reconstruct"] {
            let out = match mode {
                "copy" => e.invoke(&dc("Copy"), &[s]),
                "reconstruct" => e.invoke(&dc("Reconstruct"), &[s]),
                _ => s,
            };
            let actual = observe(&mut e, out, model.owner);
            save(
                retain,
                &format!("{name}-{mode}"),
                "observation.json",
                json!({"actual":actual,"expected":wanted,"actual_AST_steps":e.steps(),"all16_path_cells_and_full64_fields_observed":true,"definition_callbacks":false,"theorem_bodies_executed":false,"source_or_storage_proof":false}),
            );
            assert_eq!(actual, wanted, "{name}-{mode}");
            outputs += 1;
        }
        let actual = guards(&mut e, s, &model.case);
        let wanted = [
            "index_unsigned",
            "index_signed",
            "two_unsigned",
            "two_signed",
            "three_unsigned",
            "three_signed",
        ]
        .iter()
        .map(|n| go[*n].as_bool().unwrap())
        .collect::<Vec<_>>();
        save(
            retain,
            &name,
            "guards.json",
            json!({"actual":actual,"native_Go":wanted,"actual_AST_steps":e.steps(),"out_of_bounds_read_executed_in_ordinary_model":false,"source_admission_claimed":false}),
        );
        assert_eq!(actual, wanted, "{name}");
        guard_results += actual.len();
        steps += e.steps();
    }
    for seed in 0..16 {
        let mut model = sample(seed);
        model.case.offset = (seed << 60) | 3;
        let name = format!("path-{seed:02}");
        retain_input(retain, &name, &model, true);
        let (mut e, s) = input(c, &model);
        for mode in ["raw", "copy", "reconstruct", "with-bounds"] {
            let mut want = model.clone();
            let out = match mode {
                "copy" => e.invoke(&dc("Copy"), &[s]),
                "reconstruct" => e.invoke(&dc("Reconstruct"), &[s]),
                "with-bounds" => {
                    want.case.offset = u64::MAX;
                    want.case.length = 0;
                    want.case.capacity = 256;
                    let o = word(&mut e, want.case.offset);
                    let n = word(&mut e, want.case.length);
                    let cap = word(&mut e, want.case.capacity);
                    e.invoke(&dc("WithBounds"), &[s, o, n, cap])
                }
                _ => s,
            };
            let actual = observe(&mut e, out, want.owner);
            let wanted = expected(&want);
            save(
                retain,
                &format!("{name}-{mode}"),
                "observation.json",
                json!({"actual":actual,"expected":wanted,"actual_AST_steps":e.steps(),"raw_unvalidated_typed_path":true,"all16_path_cells_and_full64_fields_observed":true,"definition_callbacks":false,"theorem_bodies_executed":false,"source_or_storage_proof":false}),
            );
            assert_eq!(actual, wanted, "{name}-{mode}");
            outputs += 1;
        }
        steps += e.steps();
    }
    save(
        retain,
        "observation-inventory",
        "json",
        json!({"native_source_cases":400,"native_bound_results":guard_results,"complete_descriptor_outputs":outputs,"raw_full_path_inputs":16,"path_kind_byte_values_covered":256,"actual_AST_steps":steps,"observer_ceilings_changed":false,"definition_callbacks":false,"theorem_bodies_executed":false}),
    );
    println!("slice descriptor outputs={outputs} native guard results={guard_results} actual_AST_steps={steps}");
}
pub(super) fn countermodel(c: &Certificate, name: &str, retain: Retain) {
    let mut model = sample(0);
    let label = format!("definition-{name}");
    let is_guard =
        name.starts_with("Index") || name.starts_with("Two") || name.starts_with("Three");
    if is_guard {
        model.case.index = u64::MAX;
    }
    retain_input(retain, &label, &model, true);
    let (mut e, s) = input(c, &model);
    let (actual, wanted) = if is_guard {
        let i = word(&mut e, model.case.index);
        let lo = word(&mut e, model.case.low);
        let hi = word(&mut e, model.case.high);
        let max = word(&mut e, model.case.max);
        let args = if name.starts_with("Index") {
            vec![s, i]
        } else if name.starts_with("Two") {
            vec![s, lo, hi]
        } else {
            vec![s, lo, hi, max]
        };
        let out = e.invoke(&dc(name), &args);
        (json!(e.boolean(out)), json!(false))
    } else if [
        "OffsetValue",
        "LengthValue",
        "CapacityValue",
        "AbsoluteIndex",
    ]
    .contains(&name)
    {
        let expected = match name {
            "OffsetValue" => model.case.offset,
            "LengthValue" => model.case.length,
            "CapacityValue" => model.case.capacity,
            _ => model.case.offset + model.case.index,
        };
        let args = if name == "AbsoluteIndex" {
            let i = word(&mut e, model.case.index);
            vec![s, i]
        } else {
            vec![s]
        };
        let out = e.invoke(&dc(name), &args);
        let expected = nat(&mut e, expected);
        let equal = e.invoke("Mpk.Go.Practical.Nat.Equal", &[out, expected]);
        (
            json!({"actual_Nat_equals_expected":e.boolean(equal)}),
            json!({"actual_Nat_equals_expected":true}),
        )
    } else if name == "PathZero" {
        let p = e.invoke(&dc("PathZero"), &[]);
        let (k, i) = read_path(&mut e, p);
        (
            json!({"kinds":k,"indices":i}),
            json!({"kinds":vec![0u8;16],"indices":vec![0u64;16]}),
        )
    } else {
        let mut target = model.clone();
        let out = match name {
            "Copy" => e.invoke(&dc("Copy"), &[s]),
            "Reconstruct" => e.invoke(&dc("Reconstruct"), &[s]),
            "Nil" | "EmptyNonNilRaw" => {
                target.case.nil = name == "Nil";
                target.owner = 0;
                target.kinds = [0; 16];
                target.indices = [0; 16];
                target.case.offset = 0;
                target.case.length = 0;
                target.case.capacity = 0;
                e.invoke(&dc(name), &[])
            }
            "WithBounds" => {
                target.case.offset = 8;
                target.case.length = 9;
                target.case.capacity = 10;
                let o = word(&mut e, 8);
                let n = word(&mut e, 9);
                let cap = word(&mut e, 10);
                e.invoke(&dc("WithBounds"), &[s, o, n, cap])
            }
            _ => s,
        };
        (observe(&mut e, out, target.owner), expected(&target))
    };
    save(
        retain,
        &label,
        "countermodel.json",
        json!({"actual":actual,"expected":wanted,"actual_wrong_definition_countermodel":true,"actual_AST_steps":e.steps(),"definition_callbacks":false,"theorem_bodies_executed":false,"admitted_source_counterexample":false}),
    );
    assert_ne!(actual, wanted, "{name}");
}
