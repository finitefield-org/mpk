//! Full narrow packets; explicitly bounded observations of wide validation storage.
use super::*;

fn fixture() -> (Value, mpk_cert::encode::Certificate) {
    let root = source_tests::pins();
    let row = serde_json::from_slice(&fs::read(root.join("certificate.json")).unwrap()).unwrap();
    let hex = fs::read_to_string(root.join("all-sums.hex")).unwrap();
    let bytes = (0..hex.trim().len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    (row, mpk_cert::decode_canonical_certificate(&bytes).unwrap())
}
fn sum<'a>(row: &'a Value, role: &str, argument: &str) -> &'a Value {
    row["program"]["sums"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| {
            s["template_id"] == format!("mpk.csharp.semantic.{role}.v1")
                && s["arms"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|a| a["payload_type_id"] == argument)
        })
        .unwrap()
}
fn integer(n: i32) -> BTreeSet<usize> {
    (0..32).filter(|i| (n as u32) & (1 << i) != 0).collect()
}
fn tagged(depth: usize, tag: u32, payload: Option<(usize, &BTreeSet<usize>)>) -> BTreeSet<usize> {
    let mut ones = (0..32)
        .filter(|i| tag & (1 << i) != 0)
        .map(|i| i << (depth - 5))
        .collect::<BTreeSet<_>>();
    if let Some((pd, payload)) = payload {
        ones.extend(payload.iter().map(|i| 1 | (i << (depth - pd))));
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
    ones: &BTreeSet<usize>,
    extra_value_probes: &BTreeSet<usize>,
) {
    let pd = d["packet_depth"].as_u64().unwrap() as usize;
    let vd = d["carrier"]["depth"].as_u64().unwrap() as usize;
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
    let mut wanted = BTreeSet::new();
    if let Some(cells) = cells {
        wanted.insert(0);
        if ending == 0 {
            wanted.insert(2);
        }
        let end = text.len() - usize::from(ending != 0);
        for i in 0..32 {
            if end & (1 << i) != 0 {
                wanted.insert((2 + i) << 1);
            }
            if cells & (1 << i) != 0 {
                wanted.insert((34 + i) << 1);
            }
        }
        wanted.extend(ones.iter().map(|i| 1 | (i << 1)));
    }
    let full = pd <= 10;
    let mut probes = if full {
        (0..1 << pd).collect::<BTreeSet<_>>()
    } else {
        (0..128).map(|i| i << 1).collect::<BTreeSet<_>>()
    };
    if !full {
        probes.extend((7..pd - 1).map(|i| (1 << i) << 1));
        probes.extend((0..2048).map(|i| 1 | (i << 1)));
        probes.extend((0..32).map(|i| 1 | ((i << (vd - 5)) << 1))); // Full tag.
        probes.extend((0..64).map(|i| 1 | (((1 << vd) - 1 - i) << 1)));
        probes.extend(extra_value_probes.iter().map(|i| 1 | (i << 1)));
        probes.extend(wanted.iter().copied());
        for i in ones {
            for shift in 0..vd {
                probes.insert(1 | ((i ^ (1 << shift)) << 1));
            }
        }
    }
    for i in &probes {
        assert_eq!(
            leaf(cert, result.clone(), pd, *i),
            wanted.contains(i),
            "{} depth{depth} bit{i}: {text}",
            d["template_id"]
        );
    }
    eprintln!(
        "sum JSON {}: accepted {},{} {} packet bits",
        d["template_id"],
        cells.is_some(),
        probes.len(),
        if full { "complete" } else { "selected" }
    );
}
#[test]
fn csharp_03_t06_w09_json_sum_roles_actual_core() {
    let (row, cert) = fixture();
    let empty = BTreeSet::new();
    let no = &empty;
    let int = ty("i32");
    let lookup = sum(&row, "lookup", &int);
    let result = sum(&row, "result", &int);
    let field = sum(&row, "boundary_field", &int);
    for (d, text, tag, payload, cells) in [
        (lookup, "{\"tag\":\"missing_key\"}", 0, None, Some(1)),
        (
            lookup,
            "{\"tag\":\"found\",\"payload\":-2}",
            1,
            Some((5, integer(-2))),
            Some(2),
        ),
        (lookup, "{\"tag\":\"some\",\"payload\":1}", 0, None, None),
        (
            lookup,
            "{\"tag\":\"missing_key\",\"payload\":0}",
            0,
            None,
            None,
        ),
        (lookup, "{\"tag\":\"found\"}", 0, None, None),
        (
            result,
            "{\"tag\":\"ok\",\"payload\":-2}",
            0,
            Some((5, integer(-2))),
            Some(2),
        ),
        (
            result,
            "{\"tag\":\"error\",\"payload\":true}",
            1,
            Some((0, BTreeSet::from([0]))),
            Some(2),
        ),
        (
            result,
            "{\"tag\":\"error\",\"payload\":false}",
            1,
            Some((0, BTreeSet::new())),
            Some(2),
        ),
        (result, "{\"tag\":\"error\",\"payload\":1}", 0, None, None),
        (result, "{\"tag\":\"ok\",\"payload\":false}", 0, None, None),
        (result, "{\"tag\":\"ok\"}", 0, None, None),
        (field, "{\"tag\":\"missing\"}", 0, None, Some(1)),
        (field, "{\"tag\":\"null\"}", 1, None, Some(1)),
        (
            field,
            "{\"tag\":\"value\",\"payload\":0}",
            2,
            Some((5, integer(0))),
            Some(2),
        ),
        (
            field,
            "{\"tag\":\"value\",\"payload\":-2}",
            2,
            Some((5, integer(-2))),
            Some(2),
        ),
        (field, "{\"tag\":\"null\",\"payload\":null}", 0, None, None),
        (field, "{\"tag\":\"value\",\"payload\":null}", 0, None, None),
        (field, "null", 0, None, None),
        (field, "{}", 0, None, None),
        (
            field,
            "{\"tag\":\"missing\",\"tag\":\"null\"}",
            0,
            None,
            None,
        ),
    ] {
        assert_eq!(d["carrier"]["depth"], 6);
        let ones = tagged(6, tag, payload.as_ref().map(|(pd, p)| (*pd, p)));
        check(&cert, d, text, 0, 0, 0, cells, &ones, no);
    }
    let option = sum(&row, "option", &int);
    let nested = sum(
        &row,
        "result",
        option["carrier"]["type_id"].as_str().unwrap(),
    );
    assert_eq!(nested["carrier"]["depth"], 7);
    let some_minus_two = tagged(6, 1, Some((5, &integer(-2))));
    for (text, depth, cells, ones) in [
        (
            "{\"tag\":\"error\",\"payload\":false}",
            31,
            Some(2),
            tagged(7, 1, Some((0, no))),
        ),
        (
            "{\"tag\":\"ok\",\"payload\":{\"tag\":\"none\"}}",
            30,
            Some(2),
            tagged(7, 0, Some((6, no))),
        ),
        (
            "{\"tag\":\"ok\",\"payload\":{\"tag\":\"some\",\"payload\":-2}}",
            30,
            Some(3),
            tagged(7, 0, Some((6, &some_minus_two))),
        ),
        (
            "{\"tag\":\"ok\",\"payload\":{\"tag\":\"none\"}}",
            31,
            None,
            BTreeSet::new(),
        ),
        (
            "{\"tag\":\"ok\",\"payload\":null}",
            0,
            None,
            BTreeSet::new(),
        ),
        (
            "{\"tag\":\"ok\",\"payload\":{\"tag\":\"some\"}}",
            0,
            None,
            BTreeSet::new(),
        ),
    ] {
        check(&cert, nested, text, 0, 0, depth, cells, &ones, no);
    }
}

fn validation_sequence(values: &[i32]) -> BTreeSet<usize> {
    let mut ones = (0..32)
        .filter(|i| values.len() & (1 << i) != 0)
        .map(|i| i << 13)
        .collect::<BTreeSet<_>>();
    for (index, n) in values.iter().enumerate() {
        ones.extend(integer(*n).iter().map(|i| 1 | (index << 1) | (i << 13)));
    }
    ones
}
#[test]
fn csharp_03_t06_w09_json_sum_validation_original_document() {
    let (row, cert) = fixture();
    let d = sum(&row, "validation", &ty("i32"));
    assert_eq!(d["carrier"]["depth"], 19);
    let mut probes = (0..32)
        .map(|i| 1 | ((i << 13) << 1))
        .collect::<BTreeSet<_>>(); // All sequence length bits inside sum.
    for index in [0, 1, 255, 256, 257, 4095] {
        probes.extend((0..32).map(|i| 1 | ((1 | (index << 1) | (i << 13)) << 1)));
    }
    for n in [0, 1, 256, 257] {
        eprintln!("Validation original document error count {n}");
        let mut values = vec![0; n];
        if let Some(last) = values.last_mut() {
            *last = -2;
        }
        let text = format!(
            "{{\"tag\":\"invalid\",\"payload\":[{}]}}",
            values
                .iter()
                .map(i32::to_string)
                .collect::<Vec<_>>()
                .join(",")
        );
        let sequence = validation_sequence(&values);
        let ones = tagged(19, 1, Some((18, &sequence)));
        let cells = (1..=256).contains(&n).then_some(n as u32 + 2);
        // Every active error's full i32 bits; inactive capacity endpoints above.
        let mut active_probes = probes.clone();
        for index in 0..n {
            active_probes.extend((0..32).map(|i| 1 | ((1 | (index << 1) | (i << 13)) << 1)));
        }
        check(&cert, d, &text, 0, 0, 0, cells, &ones, &active_probes);
    }
    let minus_two = tagged(19, 0, Some((5, &integer(-2))));
    for (text, depth, cells, ones) in [
        ("{\"tag\":\"valid\",\"payload\":-2}", 31, Some(2), minus_two),
        (
            "{\"tag\":\"invalid\",\"payload\":[-2]}",
            30,
            Some(3),
            tagged(19, 1, Some((18, &validation_sequence(&[-2])))),
        ),
        (
            "{\"tag\":\"invalid\",\"payload\":[-2]}",
            31,
            None,
            BTreeSet::new(),
        ),
        (
            "{\"tag\":\"invalid\",\"payload\":-2}",
            0,
            None,
            BTreeSet::new(),
        ),
        (
            "{\"tag\":\"invalid\",\"payload\":[true]}",
            0,
            None,
            BTreeSet::new(),
        ),
        ("{\"tag\":\"invalid\"}", 0, None, BTreeSet::new()),
    ] {
        check(&cert, d, text, 0, 0, depth, cells, &ones, &probes);
    }
}

#[test]
fn csharp_03_t06_w09_json_sum_string_payload_core() {
    let root = std::env::var_os("MPK_W09_JSON_PRODUCTS_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/json-products")
        });
    let rows: Value =
        serde_json::from_slice(&fs::read(root.join("certificates.json")).unwrap()).unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|r| {
            r["id"] == "document-7a157feb27ed416752b5dde8aa2bf373b9a2c82755d2caa89e955057176ee57b"
        })
        .unwrap();
    let hex =
        fs::read_to_string(root.join(format!("{}.hex", row["id"].as_str().unwrap()))).unwrap();
    let bytes = (0..hex.trim().len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
    let d = sum(row, "option", &ty("string"));
    assert_eq!(d["carrier"]["depth"], 20);
    let mut probes = (0..32)
        .map(|i| 1 | ((i << 14) << 1))
        .collect::<BTreeSet<_>>();
    for index in [0, 1, 2, 3, 16383] {
        probes.extend((0..16).map(|i| 1 | ((1 | (index << 1) | (i << 15)) << 1)));
    }
    for (text, tag, string, cells) in [
        ("{\"tag\":\"none\"}", 0, "", Some(1)),
        ("{\"tag\":\"some\",\"payload\":\"\"}", 1, "", Some(2)),
        ("{\"tag\":\"some\",\"payload\":\"A😀\"}", 1, "A😀", Some(5)),
        ("{\"tag\":\"some\",\"payload\":null}", 0, "", None),
    ] {
        let units = string.encode_utf16().collect::<Vec<_>>();
        let mut string = (0..32)
            .filter(|i| units.len() & (1 << i) != 0)
            .map(|i| i << 14)
            .collect::<BTreeSet<_>>();
        for (index, unit) in units.iter().enumerate() {
            string.extend(
                (0..16)
                    .filter(|i| unit & (1 << i) != 0)
                    .map(|i| 1 | (index << 1) | (i << 15)),
            );
        }
        let ones = tagged(20, tag, Some((19, &string)));
        check(&cert, d, text, 0, 0, 0, cells, &ones, &probes);
    }
}
