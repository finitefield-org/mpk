//! Compare actual ordinary definitions with intact native Go array observations.
use super::actual::Interpreter;
use mpk_cert::encode::Certificate;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    length: usize,
    left: Vec<u8>,
    right: Vec<u8>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Native {
    id: String,
    length: usize,
    equal: bool,
    not_equal: bool,
}
fn save(retain: fn(&str, &str, &[u8]), name: &str, ext: &str, v: serde_json::Value) {
    retain(
        name,
        ext,
        format!("{}\n", serde_json::to_string_pretty(&v).unwrap()).as_bytes(),
    );
}
fn bits(n: usize) -> Vec<bool> {
    (0..64).map(|i| (n as u64) & (1 << i) != 0).collect()
}
fn depth(n: usize) -> usize {
    n.max(1).next_power_of_two().trailing_zeros() as usize
}
fn inputs<'a>(c: &'a Certificate, row: &Case, mode: u8) -> (Interpreter<'a>, u32, u32, u32, u32) {
    assert!(row.length <= 256 && row.left.len() == 256 && row.right.len() == 256);
    let d = depth(row.length);
    let mut e = Interpreter::new(c);
    let bool_type = e.invoke("Std.Bool", &[]);
    let byte_type = e.invoke("Mpk.Go.Practical.Array.D3.Carrier", &[bool_type]);
    let a = e.raw(row.length as u64, row.left.clone().try_into().unwrap());
    let b = e.raw(row.length as u64, row.right.clone().try_into().unwrap());
    assert_eq!(e.length(a), row.length as u64);
    assert_eq!(e.length(b), row.length as u64);
    assert_eq!(e.bytes(a).to_vec(), row.left);
    assert_eq!(e.bytes(b).to_vec(), row.right);
    let n = e.invoke("Mpk.Go.Practical.String.Size", &[a]);
    let mut aa = vec![a];
    aa.extend(vec![0; 8 - d]);
    let mut bb = vec![b];
    bb.extend(vec![0; 8 - d]);
    let mut f = e.invoke("Mpk.Go.Practical.String.Data", &aa);
    let mut g = e.invoke("Mpk.Go.Practical.String.Data", &bb);
    let z = e.byte_input(0);
    if row.length > 0 {
        if mode & 1 != 0 {
            f = e.invoke(
                &format!("Mpk.Go.Practical.Array.D{d}.Normalize"),
                &[byte_type, z, n, f],
            );
        }
        if mode & 2 != 0 {
            g = e.invoke(
                &format!("Mpk.Go.Practical.Array.D{d}.Normalize"),
                &[byte_type, z, n, g],
            );
        }
    }
    (e, byte_type, n, f, g)
}
fn byte_operator(e: &mut Interpreter<'_>, row: &Case, args: [u32; 3], negative: bool) -> bool {
    let value = if row.length == 0 {
        e.invoke(
            if negative {
                "Mpk.Go.Practical.Array.Equality.Empty.NotEqual"
            } else {
                "Mpk.Go.Practical.Array.Equality.Empty.Equal"
            },
            &[0, 0],
        )
    } else {
        e.invoke(
            &format!(
                "Mpk.Go.Practical.Array.Equality.D{}.{}",
                depth(row.length),
                if negative {
                    "ByteNotEqual"
                } else {
                    "ByteEqual"
                }
            ),
            &args,
        )
    };
    e.boolean(value)
}
pub(super) fn run(c: &Certificate, retain: fn(&str, &str, &[u8])) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../develop/migrations/go-practical/t02-w01/verification/array-equality/native-go",
    );
    let cases: Vec<Case> =
        serde_json::from_slice(&std::fs::read(root.join("cases.json")).unwrap()).unwrap();
    let native: Vec<Native> =
        serde_json::from_slice(&std::fs::read(root.join("comparisons.stdout.log")).unwrap())
            .unwrap();
    assert_eq!(cases.len(), 1028);
    assert_eq!(cases.len(), native.len());
    let boundary = [
        1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256,
    ];
    let (mut results, mut total) = (0, 0usize);
    // Full-capacity true folds run first; every original length/case still executes.
    let mut order = (0..cases.len()).collect::<Vec<_>>();
    order.sort_by_key(|i| (cases[*i].length != 256, *i));
    for index in order {
        let row = &cases[index];
        let go = &native[index];
        assert_eq!(row.id, go.id);
        assert_eq!(row.length, go.length);
        assert_eq!(go.equal, !go.not_equal);
        let name = format!("case-{}", row.id);
        save(
            retain,
            &name,
            "input.json",
            json!({"id":row.id,"logical_length":row.length,"all64_length_bits":bits(row.length),"all256_left_bytes":row.left,"all256_right_bytes":row.right,"source_or_nominal_authority":false,"registered_profile_or_program_proof":false}),
        );
        for mode in 0..if boundary.contains(&row.length) { 4 } else { 1 } {
            let label = format!("{name}-mode{mode}");
            let (mut e, _, n, f, g) = inputs(c, row, mode);
            let equal = byte_operator(&mut e, row, [n, f, g], false);
            save(
                retain,
                &label,
                "equal.partial.json",
                json!({"equal":equal,"native_Go_equal":go.equal,"actual_AST_steps":e.steps()}),
            );
            assert_eq!(equal, go.equal, "{label}");
            let unequal = byte_operator(&mut e, row, [n, f, g], true);
            save(
                retain,
                &label,
                "observation.json",
                json!({"equal":equal,"not_equal":unequal,"native_Go_equal":go.equal,"native_Go_not_equal":go.not_equal,"actual_AST_steps":e.steps(),"complete_input_carriers_observed":true,"definition_callbacks":false,"theorem_bodies_executed":false,"source_or_profile_proof":false}),
            );
            assert_eq!(unequal, go.not_equal, "{label}");
            total += e.steps();
            results += 1;
        }
    }
    save(
        retain,
        "observation-inventory",
        "json",
        json!({"all_lengths_0_through_256":true,"whole_native_input_pairs":cases.len(),"raw_and_normalized_case_pairs":results,"operator_results":results*2,"actual_AST_steps":total,"observer_ceilings_changed":false,"definition_callbacks":false,"theorem_bodies_executed":false}),
    );
    println!(
        "array comparison pairs={results} operator results={} actual_AST_steps={total}",
        results * 2
    );
}
pub(super) fn countermodel(c: &Certificate, short: &str, retain: fn(&str, &str, &[u8])) {
    let (d, kind) = if short.starts_with('D') {
        let (a, b) = short.split_once('.').unwrap();
        (a[1..].parse::<usize>().unwrap(), b)
    } else {
        (0, short)
    };
    let n = if short.starts_with("Empty.") {
        0
    } else if d == 0 {
        1
    } else {
        (1 << (d - 1)) + 1
    };
    let mut a = vec![0; 256];
    a[..n].fill(128);
    let mut b = a.clone();
    let negative = kind.contains("NotEqual");
    if !negative && n > 0 {
        b[0] = 33;
    }
    let row = Case {
        id: short.into(),
        length: n,
        left: std::mem::take(&mut a),
        right: b,
    };
    let name = format!("definition-{short}");
    save(
        retain,
        &name,
        "input.json",
        json!({"logical_length":n,"depth":d,"all64_length_bits":bits(n),"all256_left_bytes":row.left,"all256_right_bytes":row.right,"source_or_nominal_authority":false,"admitted_source_counterexample":false}),
    );
    let (mut e, ty, n, f, g) = inputs(c, &row, 0);
    let actual = if kind.starts_with("Empty.") || kind.starts_with("Byte") {
        byte_operator(&mut e, &row, [n, f, g], negative)
    } else {
        let cmp = e.invoke("Mpk.Go.Practical.String.Equality.ByteEqual", &[]);
        let v = e.invoke(
            &format!(
                "Mpk.Go.Practical.Array.Equality.D{d}.{}",
                if negative { "NotEqualBy" } else { "EqualBy" }
            ),
            &[ty, n, f, g, cmp],
        );
        e.boolean(v)
    };
    let same = row.left[..row.length] == row.right[..row.length];
    let expected = if negative { !same } else { same };
    save(
        retain,
        &name,
        "countermodel.json",
        json!({"actual":actual,"expected":expected,"actual_wrong_definition_countermodel":true,"actual_AST_steps":e.steps(),"definition_callbacks":false,"theorem_bodies_executed":false}),
    );
    assert_ne!(actual, expected, "{short}");
}
