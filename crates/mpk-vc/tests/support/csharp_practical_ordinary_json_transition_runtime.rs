//! Actual Transition packets, source-product contrast and role-specific cell counts.
use super::*;
fn integer(n: i32) -> BTreeSet<usize> {
    (0..32).filter(|i| (n as u32) & (1 << i) != 0).collect()
}
fn string(s: &str) -> BTreeSet<usize> {
    let units = s.encode_utf16().collect::<Vec<_>>();
    let mut ones = integer(units.len() as i32)
        .into_iter()
        .map(|i| i << 14)
        .collect::<BTreeSet<_>>();
    for (at, c) in units.into_iter().enumerate() {
        ones.extend(
            (0..16)
                .filter(|i| c & (1 << i) != 0)
                .map(|i| 1 | (at << 1) | (i << 15)),
        );
    }
    ones
}
fn events(values: &[i32]) -> BTreeSet<usize> {
    let mut ones = integer(values.len() as i32)
        .into_iter()
        .map(|i| i << 13)
        .collect::<BTreeSet<_>>();
    for (at, c) in values.iter().enumerate() {
        ones.extend(integer(*c).into_iter().map(|i| 1 | (at << 1) | (i << 13)));
    }
    ones
}
fn value(compound: bool, values: &[i32], some: bool) -> BTreeSet<usize> {
    let mut ones = if compound {
        let mut state = string("A😀")
            .into_iter()
            .map(|i| i << 1)
            .collect::<BTreeSet<_>>();
        state.extend(integer(9).into_iter().map(|i| 1 | (i << 15)));
        state.into_iter().map(|i| i << 2).collect()
    } else {
        integer(-2)
            .into_iter()
            .map(|i| i << 15)
            .collect::<BTreeSet<_>>()
    };
    ones.extend(
        events(values)
            .into_iter()
            .map(|i| 1 | (i << if compound { 4 } else { 2 })),
    );
    if some {
        ones.insert(2);
        if compound {
            ones.insert(2 | (1 << 16));
        }
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
    compound: bool,
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
    let mut expected = BTreeSet::new();
    if let Some(cells) = cells {
        expected.insert(0);
        if ending == 0 {
            expected.insert(2);
        }
        let end = text.len() - usize::from(ending != 0);
        expected.extend(integer(end as i32).into_iter().map(|i| (2 + i) << 1));
        expected.extend(integer(cells as i32).into_iter().map(|i| (34 + i) << 1));
        expected.extend(ones.iter().map(|i| 1 | (i << 1)));
    }
    let mut probes = (0..128).map(|i| i << 1).collect::<BTreeSet<_>>();
    probes.extend((7..vd).map(|i| (1 << i) << 1));
    let mut values = (0..1024).collect::<BTreeSet<_>>();
    values.extend(ones);
    for i in ones {
        for bit in 0..vd {
            values.insert(i ^ (1 << bit));
        }
    }
    // Complete events length and selected first/end integer slots.
    let shift = if compound { 4 } else { 2 };
    values.extend((0..32).map(|i| 1 | ((i << 13) << shift)));
    for at in [0, 1, 2, 3, 4095] {
        values.extend((0..32).map(|i| 1 | ((1 | (at << 1) | (i << 13)) << shift)));
    }
    if compound {
        // State.Version, State.Name length/UTF16 (including both surrogate units),
        // response Option tag/value and source/product role padding.
        values.extend((0..32).map(|i| (1 | (i << 15)) << 2));
        values.extend((0..32).map(|i| i << 17));
        for at in [0, 1, 2, 3, 16383] {
            values.extend((0..16).map(|i| (1 | (at << 1) | (i << 15)) << 3));
        }
        values.extend((0..64).map(|i| 2 | (i << 16)));
    } else {
        values.extend((0..32).map(|i| i << 15));
        values.insert(2);
    }
    values.extend((0..64).map(|i| (1 << vd) - 1 - i));
    probes.extend(values.into_iter().map(|i| 1 | (i << 1)));
    for i in &probes {
        assert_eq!(
            leaf(cert, result.clone(), pd, *i),
            expected.contains(i),
            "depth{depth} bit{i}: {text}"
        );
    }
    eprintln!("Transition/source compound{compound} depth{depth}: accepted {},{} selected wide packet bits",cells.is_some(),probes.len());
}
#[test]
fn csharp_03_t06_w09_json_transition_actual_core() {
    let root = pins();
    let rows: Value =
        serde_json::from_slice(&fs::read(root.join("certificates.json")).unwrap()).unwrap();
    for row in rows.as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        if std::env::var("MPK_W09_JSON_TRANSITION_SELECT").is_ok_and(|selected| selected != id) {
            continue;
        }
        let compound = id == "compound";
        let hex = fs::read_to_string(root.join(format!("{id}.hex"))).unwrap();
        let bytes = (0..hex.trim().len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect::<Vec<_>>();
        let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        let products = row["program"]["products"].as_array().unwrap();
        let d = products
            .iter()
            .find(|d| d["template_id"] == "mpk.csharp.semantic.transition.v1")
            .unwrap();
        let source = products
            .iter()
            .find(|d| {
                d.get("template_id").is_none()
                    && d["member_names"] == json!(["State", "Events", "Response"])
            })
            .unwrap();
        assert_eq!(d["carrier"]["depth"], if compound { 22 } else { 20 });
        let state = if compound {
            "{\"Name\":\"A😀\",\"Version\":9}"
        } else {
            "-2"
        };
        let event = |n| {
            if compound {
                format!("{{\"Code\":{n}}}")
            } else {
                format!("{n}")
            }
        };
        let response = |some| {
            if compound {
                if some {
                    "{\"tag\":\"some\",\"payload\":true}"
                } else {
                    "{\"tag\":\"none\"}"
                }
            } else if some {
                "true"
            } else {
                "false"
            }
        };
        let make = |xs: &[i32], some| {
            format!(
                "{{\"state\":{state},\"events\":[{}],\"response\":{}}}",
                xs.iter().map(|n| event(*n)).collect::<Vec<_>>().join(","),
                response(some)
            )
        };
        let cells = |n: u32, some| {
            if compound {
                // State includes product + string + its three UTF-16 units + i32.
                1 + 6 + 2 * n + if some { 2 } else { 1 }
            } else {
                3 + n
            }
        };
        let xs = vec![3, -1, 3];
        let text = make(&xs, true);
        let ones = value(compound, &xs, true);
        check(
            &cert,
            d,
            &text,
            0,
            0,
            0,
            Some(cells(3, true)),
            &ones,
            compound,
        );
        let source_text = text
            .replace("\"state\":", "\"State\":")
            .replace("\"events\":", "\"Events\":")
            .replace("\"response\":", "\"Response\":");
        check(
            &cert,
            source,
            &source_text,
            0,
            0,
            0,
            Some(cells(3, true) + 1),
            &ones,
            compound,
        );
        check(
            &cert,
            d,
            &format!("xx{text},"),
            2,
            1,
            0,
            Some(cells(3, true)),
            &ones,
            compound,
        );
        let empty = make(&[], false);
        let empty_ones = value(compound, &[], false);
        check(
            &cert,
            d,
            &empty,
            0,
            0,
            0,
            Some(cells(0, false)),
            &empty_ones,
            compound,
        );
        check(
            &cert,
            d,
            &empty,
            0,
            0,
            if compound { 30 } else { 31 },
            Some(cells(0, false)),
            &empty_ones,
            compound,
        );
        check(
            &cert,
            d,
            &text,
            0,
            0,
            if compound { 29 } else { 30 },
            Some(cells(3, true)),
            &ones,
            compound,
        );
        check(
            &cert,
            d,
            &text,
            0,
            0,
            if compound { 30 } else { 31 },
            None,
            &ones,
            compound,
        );
        for bad in [
            format!("{{\"state\":{state},\"response\":{}}}", response(true)),
            format!(
                "{{\"state\":{state},\"events\":null,\"response\":{}}}",
                response(true)
            ),
            format!(
                "{{\"events\":[],\"state\":{state},\"response\":{}}}",
                response(true)
            ),
            format!(
                "{{\"state\":{state},\"events\":[null],\"response\":{}}}",
                response(true)
            ),
            format!("{text}x"),
        ] {
            check(&cert, d, &bad, 0, 0, 0, None, &ones, compound);
        }
    }
}
