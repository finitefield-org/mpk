//! Original JSON ordering and wrapper-cell checks; explicit wide storage probes.
use super::*;
fn word_bits(n: u32) -> BTreeSet<usize> {
    (0..32).filter(|i| n & (1 << i) != 0).collect()
}
fn string_bits(text: &str) -> BTreeSet<usize> {
    let units = text.encode_utf16().collect::<Vec<_>>();
    let mut bits = word_bits(units.len() as u32)
        .into_iter()
        .map(|i| i << 14)
        .collect::<BTreeSet<_>>();
    for (at, c) in units.into_iter().enumerate() {
        bits.extend(
            (0..16)
                .filter(|i| c & (1 << i) != 0)
                .map(|i| 1 | (at << 1) | (i << 15)),
        );
    }
    bits
}
fn keys(
    id: &str,
) -> (
    String,
    String,
    u32,
    [u32; 2],
    BTreeSet<usize>,
    BTreeSet<usize>,
) {
    match id {
        "integer" => (
            "-2".into(),
            "3".into(),
            5,
            [1, 1],
            word_bits((-2i32) as u32),
            word_bits(3),
        ),
        "decimal" => (
            "\"2\"".into(),
            "\"10\"".into(),
            9,
            [1, 1],
            BTreeSet::from([6]),
            BTreeSet::from([6, 14]),
        ),
        "string" => (
            "\"a\"".into(),
            "\"aa\"".into(),
            19,
            [2, 3],
            string_bits("a"),
            string_bits("aa"),
        ),
        "compound" => {
            let value = |n| {
                let mut b = string_bits("a")
                    .into_iter()
                    .map(|i| i << 1)
                    .collect::<BTreeSet<_>>();
                b.extend(word_bits(n).into_iter().map(|i| 1 | (i << 15)));
                b
            };
            (
                "{\"Name\":\"a\",\"Code\":2}".into(),
                "{\"Name\":\"a\",\"Code\":10}".into(),
                20,
                [4, 4],
                value(2),
                value(10),
            )
        }
        _ => panic!("unexpected fixture {id}"),
    }
}
fn storage(depth: u32, values: &[BTreeSet<usize>]) -> BTreeSet<usize> {
    let mut ones = word_bits(values.len() as u32)
        .into_iter()
        .map(|i| i << (depth - 5))
        .collect::<BTreeSet<_>>();
    for (index, value) in values.iter().enumerate() {
        ones.extend(value.iter().map(|bit| 1 | (index << 1) | (bit << 13)));
    }
    ones
}
#[allow(clippy::too_many_arguments)]
fn check(
    cert: &mpk_cert::encode::Certificate,
    d: &Value,
    text: &str,
    start: u32,
    ending: u8,
    depth: u32,
    cells: Option<u32>,
    values: &[BTreeSet<usize>],
    element_depth: u32,
) {
    let pd = d["packet_depth"].as_u64().unwrap() as usize;
    let vd = d["carrier"]["depth"].as_u64().unwrap() as u32;
    let value_ones = storage(vd, values);
    let mut expected = BTreeSet::new();
    if let Some(cells) = cells {
        expected.insert(0);
        if ending == 0 {
            expected.insert(2);
        }
        let end = text.len() - usize::from(ending != 0);
        expected.extend(word_bits(end as u32).into_iter().map(|i| (2 + i) << 1));
        expected.extend(word_bits(cells).into_iter().map(|i| (34 + i) << 1));
        expected.extend(value_ones.iter().map(|i| 1 | (i << 1)));
    }
    let result = run(
        cert,
        d["parse_definition"].as_str().unwrap(),
        vec![
            document(
                text.len() as u32,
                &text.bytes().enumerate().collect::<Vec<_>>(),
            ),
            word(start),
            word_tag(ending),
            word(depth),
        ],
    );
    let mut probes = (0..128).map(|i| i << 1).collect::<BTreeSet<_>>();
    probes.extend((7..vd).map(|i| (1 << i) << 1));
    probes.extend((0..32).map(|i| 1 | ((i << (vd - 5)) << 1)));
    probes.extend((0..512).map(|i| 1 | (i << 1)));
    let mut element_probes = (0..(1 << element_depth.min(7))).collect::<BTreeSet<_>>();
    for v in values {
        element_probes.extend(v);
        for bit in v {
            for i in 0..element_depth {
                element_probes.insert(bit ^ (1 << i));
            }
        }
    }
    for slot in [0, 1, 2, 4095] {
        for bit in &element_probes {
            probes.insert(1 | ((1 | (slot << 1) | (bit << 13)) << 1));
        }
    }
    probes.extend(expected.iter().copied());
    for i in &probes {
        assert_eq!(
            leaf(cert, result.clone(), pd, *i),
            expected.contains(i),
            "packet bit {i},depth{depth}, {text}"
        );
    }
    eprintln!(
        "ordered JSON depth{depth}: accepted {},{} selected wide packet bits",
        cells.is_some(),
        probes.len()
    );
}
#[test]
fn csharp_03_t06_w09_json_ordered_actual_core() {
    let root = pins();
    let rows: Value =
        serde_json::from_slice(&fs::read(root.join("certificates.json")).unwrap()).unwrap();
    let selected = std::env::var("MPK_W09_JSON_ORDERED_SELECT").ok();
    let mut seen = 0;
    for row in rows.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        if selected.as_deref().is_some_and(|s| s != id) {
            continue;
        }
        seen += 1;
        let hex = fs::read_to_string(root.join(format!("{id}.hex"))).unwrap();
        let bytes = (0..hex.trim().len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        let (a, b, key_depth, key_cells, ka, kb) = keys(id);
        let defs = row["program"]["collections"].as_array().unwrap();
        for map in [false, true] {
            let role = if map { "ordered_map" } else { "ordered_set" };
            let d = &defs
                .iter()
                .find(|d| d["template_id"] == format!("mpk.csharp.semantic.{role}.v1"))
                .unwrap()["sequence"];
            let entry = |key: &str, value: bool| format!("{{\"key\":{key},\"value\":{value}}}");
            let ea = if map { entry(&a, true) } else { a.clone() };
            let eb = if map { entry(&b, false) } else { b.clone() };
            let duplicate = if map { entry(&a, false) } else { a.clone() };
            let encode = |key: &BTreeSet<usize>, value| {
                if !map {
                    return key.clone();
                }
                let mut bits = key.iter().map(|i| i << 1).collect::<BTreeSet<_>>();
                if value {
                    bits.insert(1);
                }
                bits
            };
            let va = encode(&ka, true);
            let vb = encode(&kb, false);
            let ed = key_depth + u32::from(map);
            let cells = 1 + key_cells.iter().sum::<u32>() + 2 * u32::from(map);
            eprintln!("ordered {id} {role}: empty,ascending,descending,duplicate,syntax and absolute ending");
            for (text, start, ending, good, values) in [
                ("[]".into(), 0, 0, Some(1), vec![]),
                (
                    format!("[{ea},{eb}]"),
                    0,
                    0,
                    Some(cells),
                    vec![va.clone(), vb.clone()],
                ),
                (
                    format!("[{eb},{ea}]"),
                    0,
                    0,
                    None,
                    vec![vb.clone(), va.clone()],
                ),
                (
                    format!("[{ea},{duplicate}]"),
                    0,
                    0,
                    None,
                    vec![va.clone(), encode(&ka, false)],
                ),
                (format!("[{ea},]"), 0, 0, None, vec![]),
                (
                    format!("xx[{ea},{eb}],"),
                    2,
                    1,
                    Some(cells),
                    vec![va.clone(), vb.clone()],
                ),
            ] {
                check(&cert, d, &text, start, ending, 0, good, &values, ed);
            }
            if id == "integer" {
                let text = format!("[{ea}]");
                let one = 1 + key_cells[0] + u32::from(map);
                check(
                    &cert,
                    d,
                    &text,
                    0,
                    0,
                    if map { 30 } else { 31 },
                    Some(one),
                    std::slice::from_ref(&va),
                    ed,
                );
                check(
                    &cert,
                    d,
                    &text,
                    0,
                    0,
                    if map { 31 } else { 32 },
                    None,
                    std::slice::from_ref(&va),
                    ed,
                );
            }
        }
    }
    assert_eq!(seen, if selected.is_some() { 1 } else { 4 });
}

#[test]
fn csharp_03_t06_w09_json_ordered_high_slot_admission() {
    // Legal ascending storage prefixes isolate the new previous-slot reader.
    // These observations do not claim full-document maximum-capacity parsing.
    let root = pins();
    let rows: Value =
        serde_json::from_slice(&fs::read(root.join("certificates.json")).unwrap()).unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "integer")
        .unwrap();
    let hex = fs::read_to_string(root.join("integer.hex")).unwrap();
    let bytes = (0..hex.trim().len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
    for c in row["program"]["collections"].as_array().unwrap() {
        let map = c["template_id"] == "mpk.csharp.semantic.ordered_map.v1";
        let d = c["sequence"]["carrier"]["depth"].as_u64().unwrap() as u32;
        let element_depth = if map { 6 } else { 5 };
        let element = |n: u32| {
            let mut bits = word_bits(n);
            if map {
                bits = bits.into_iter().map(|i| i << 1).collect();
                bits.insert(1);
            }
            bits
        };
        for n in [0_u32, 1, 2048, 2049, 4095] {
            let values = (0..n).map(element).collect::<Vec<_>>();
            let prefix = core_eval::sparse_cube(d, storage(d, &values));
            let cases = if n == 0 {
                vec![((-2_i32) as u32, true)]
            } else {
                vec![(n, true), (n - 1, false), (u32::MAX, false)]
            };
            for (next, expected) in cases {
                let value = core_eval::sparse_cube(element_depth, element(next));
                let accepted = run(
                    &cert,
                    c["order_definition"].as_str().unwrap(),
                    vec![prefix.clone(), value],
                );
                assert_eq!(
                    core_eval::bit(accepted),
                    expected,
                    "map{map} prefix{n} next{next}"
                );
            }
        }
    }
}
