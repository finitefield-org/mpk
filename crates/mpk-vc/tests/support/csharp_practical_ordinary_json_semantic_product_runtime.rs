//! Full small packets and explicit wide-carrier probes for semantic products.
use super::*;

fn decimal_125() -> BTreeSet<usize> {
    let mut ones = BTreeSet::from([65]); // Scale2 in the existing C9 decimal product.
    ones.extend(
        (0..96)
            .filter(|i| 125_u128 & (1 << i) != 0)
            .map(|i| 2 | (i << 2)),
    );
    ones
}
fn money_enum() -> BTreeSet<usize> {
    let mut ones = decimal_125()
        .into_iter()
        .map(|i| i << 1)
        .collect::<BTreeSet<_>>();
    ones.extend((0..3).map(|i| 1 | (i << 5))); // Enum7, padding precedes its five selectors.
    ones
}
fn money_text() -> BTreeSet<usize> {
    let mut ones = decimal_125()
        .into_iter()
        .map(|i| i << 11)
        .collect::<BTreeSet<_>>();
    let units = "U😀".encode_utf16().collect::<Vec<_>>();
    // String sequence: one role bit, then 13 length-padding bits or 14 index bits.
    let mut string = BTreeSet::new();
    for i in 0..32 {
        if units.len() & (1 << i) != 0 {
            string.insert(i << 14);
        }
    }
    for (at, unit) in units.into_iter().enumerate() {
        for i in 0..16 {
            if unit & (1 << i) != 0 {
                string.insert(1 | (at << 1) | (i << 15));
            }
        }
    }
    ones.extend(string.into_iter().map(|i| 1 | (i << 1)));
    ones
}
fn entry() -> BTreeSet<usize> {
    let mut ones = (1..32).map(|i| i << 1).collect::<BTreeSet<_>>(); // i32 -2.
    ones.insert(1); // Bool true, padded after the field role and before its scalar leaf.
    ones
}
#[test]
fn csharp_03_t06_w09_json_semantic_products_actual_core() {
    let root = pins();
    let row: Value =
        serde_json::from_slice(&fs::read(root.join("certificate.json")).unwrap()).unwrap();
    let hex = fs::read_to_string(root.join("money-entry.hex")).unwrap();
    let bytes = (0..hex.trim().len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect::<Vec<_>>();
    let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
    let products = row["program"]["products"].as_array().unwrap();
    let string_only = std::env::var_os("MPK_W09_JSON_SEMANTIC_STRING_ONLY").is_some();
    let mut cases = 0;
    let mut check = |d: &Value,
                     text: &str,
                     start: u32,
                     ending: u8,
                     depth: u32,
                     cells: Option<u32>,
                     ones: &BTreeSet<usize>| {
        let packet_depth = d["packet_depth"].as_u64().unwrap() as usize;
        let value_depth = d["carrier"]["depth"].as_u64().unwrap() as usize;
        if string_only && value_depth < 20 {
            return;
        }
        let result = run(
            &cert,
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
        let mut probes = BTreeSet::new();
        if packet_depth <= 12 {
            probes.extend(0..1 << packet_depth);
        } else {
            probes.extend((0..128).map(|i| i << 1)); // Complete logical header.
            probes.extend((7..packet_depth - 1).map(|i| (1 << i) << 1)); // Header high padding.
            let mut values = (0..2048).collect::<BTreeSet<_>>();
            values.extend(ones); // All expected occupied leaves, including both surrogate code units.
            for i in ones {
                if *i > 0 {
                    values.insert(i - 1);
                }
                values.insert(i + 1);
                for bit in 0..value_depth {
                    values.insert(i ^ (1 << bit));
                }
            }
            values.extend((0..value_depth).map(|i| 1 << i));
            values.extend((0..64).map(|i| (1 << value_depth) - 1 - i));
            if value_depth == 20 {
                values.extend((0..512).map(|i| i << 11)); // Every physical decimal leaf.
                for bit in 0..16 {
                    values.insert(1 | ((1 | (16_383 << 1) | (bit << 15)) << 1));
                }
            } else {
                assert_eq!(value_depth, 22); // Source envelope: four role encodings, three actual fields.
                values.extend((2..22).map(|i| 3 | (1 << i)));
            }
            probes.extend(
                values
                    .into_iter()
                    .filter(|i| *i < 1 << value_depth)
                    .map(|i| 1 | (i << 1)),
            );
        }
        let observed = probes.len();
        for i in probes {
            let expected = if let Some(cells) = cells {
                if i & 1 != 0 {
                    ones.contains(&(i >> 1))
                } else {
                    let bit = i >> 1;
                    let end = text.len() as u32 - u32::from(ending != 0);
                    match bit {
                        0 => true,
                        1 => ending == 0,
                        2..=33 => end & (1 << (bit - 2)) != 0,
                        34..=65 => cells & (1 << (bit - 34)) != 0,
                        _ => false,
                    }
                }
            } else {
                false
            };
            assert_eq!(
                leaf(&cert, result.clone(), packet_depth, i),
                expected,
                "case{cases} depth{depth} {text} bit{i}"
            );
        }
        cases += 1;
        eprintln!(
            "JSON semantic core {cases}: depth{value_depth}, {observed} packet probes, accepted {}",
            cells.is_some()
        );
    };
    let entry_def = products
        .iter()
        .find(|d| d["template_id"] == "mpk.csharp.semantic.ordered_entry.v1")
        .unwrap();
    assert_eq!(entry_def["carrier"]["depth"], 6);
    let entry_bits = entry();
    check(
        entry_def,
        "{\"key\":-2,\"value\":true}",
        0,
        0,
        31,
        Some(3),
        &entry_bits,
    );
    check(
        entry_def,
        "x{\"key\":-2,\"value\":true}]",
        1,
        2,
        0,
        Some(3),
        &entry_bits,
    );
    for text in [
        "{\"value\":true,\"key\":-2}",
        "{\"key\":-2}",
        "{\"key\":-2,\"value\":1}",
    ] {
        check(entry_def, text, 0, 0, 0, None, &entry_bits);
    }
    let enum_def = products
        .iter()
        .find(|d| d["template_id"] == "mpk.csharp.semantic.money.v1" && d["carrier"]["depth"] == 10)
        .unwrap();
    let enum_bits = money_enum();
    check(
        enum_def,
        "{\"amount\":\"1.25\",\"currency\":\"7\"}",
        0,
        0,
        31,
        Some(3),
        &enum_bits,
    );
    for text in [
        "{\"amount\":\"1.250\",\"currency\":\"7\"}",
        "{\"amount\":\"1.25\",\"currency\":7}",
        "{\"amount\":\"1.25\",\"currency\":\"2\"}",
        "{\"Amount\":\"1.25\",\"Currency\":\"7\"}",
        "{\"currency\":\"7\",\"amount\":\"1.25\"}",
    ] {
        check(enum_def, text, 0, 0, 0, None, &enum_bits);
    }
    let text_def = products
        .iter()
        .find(|d| d["template_id"] == "mpk.csharp.semantic.money.v1" && d["carrier"]["depth"] == 20)
        .unwrap();
    let text_bits = money_text();
    check(
        text_def,
        "{\"amount\":\"1.25\",\"currency\":\"U😀\"}",
        0,
        0,
        31,
        Some(6),
        &text_bits,
    );
    check(
        text_def,
        "x{\"amount\":\"1.25\",\"currency\":\"U😀\"},",
        1,
        1,
        0,
        Some(6),
        &text_bits,
    );
    check(
        text_def,
        "{\"amount\":\"1.25\",\"currency\":U😀}",
        0,
        0,
        0,
        None,
        &text_bits,
    );
    let parent = products
        .iter()
        .find(|d| d["member_names"] == json!(["EnumValue", "TextValue", "Entry"]))
        .unwrap();
    assert_eq!(parent["carrier"]["depth"], 22);
    let mut parent_bits = enum_bits
        .into_iter()
        .map(|i| i << 12)
        .collect::<BTreeSet<_>>();
    parent_bits.extend(text_bits.into_iter().map(|i| 1 | (i << 2)));
    parent_bits.extend(entry_bits.into_iter().map(|i| 2 | (i << 16)));
    let text="{\"EnumValue\":{\"Amount\":\"1.25\",\"Currency\":\"7\"},\"TextValue\":{\"Amount\":\"1.25\",\"Currency\":\"U😀\"},\"Entry\":{\"Key\":-2,\"Value\":true}}";
    check(parent, text, 0, 0, 30, Some(13), &parent_bits);
    check(parent, text, 0, 0, 31, None, &parent_bits);
    assert_eq!(cases, if string_only { 5 } else { 16 });
}
