//! Full ordinary packets for the original-source Option<i32> JSON consumer.
use super::*;
#[path = "csharp_practical_ordinary_json_sum_runtime.rs"]
mod runtime;
#[path = "csharp_practical_ordinary_json_sum_source_tests.rs"]
mod source_tests;

#[test]
fn csharp_03_t06_w09_json_sum_option_actual_core() {
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
    let sum = row["program"]["sums"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["template_id"] == "mpk.csharp.semantic.option.v1")
        .unwrap();
    assert_eq!(sum["carrier"]["depth"], 6);
    assert_eq!(sum["packet_depth"], 8);
    assert_eq!(sum["arms"][1]["payload_type_id"], "mpk.csharp.value.i32.v1");
    let cases = [
        ("{\"tag\":\"none\"}", 0, 0, 0, Some((0, 0_u32, 1))),
        (
            "{\"tag\":\"some\",\"payload\":-2}",
            0,
            0,
            0,
            Some((1, (-2_i32) as u32, 2)),
        ),
        (
            "{\"tag\":\"some\",\"payload\":0}",
            0,
            0,
            31,
            Some((1, 0, 2)),
        ),
        ("{\"tag\":\"none\"}", 0, 0, 31, Some((0, 0, 1))),
        ("{\"tag\":\"none\"}", 0, 0, 32, None),
        ("{\"tag\":\"some\",\"payload\":1}", 0, 0, 32, None),
        (
            "xx{\"tag\":\"some\",\"payload\":3},",
            2,
            1,
            0,
            Some((1, 3, 2)),
        ),
        ("{\"tag\":\"none\",\"payload\":0}", 0, 0, 0, None),
        ("{\"tag\":\"some\"}", 0, 0, 0, None),
        ("{\"tag\":\"some\",\"payload\":null}", 0, 0, 0, None),
        ("{\"tag\":\"Some\",\"payload\":1}", 0, 0, 0, None),
        ("{\"payload\":1,\"tag\":\"some\"}", 0, 0, 0, None),
        ("{\"tag\":\"none\",\"tag\":\"none\"}", 0, 0, 0, None),
        ("{\"tag\": \"none\"}", 0, 0, 0, None),
        ("{\"tag\":\"some\",\"payload\":2147483648}", 0, 0, 0, None),
        ("{\"tag\":\"none\"}x", 0, 0, 0, None),
        ("{\"tag\":\"none\"", 0, 0, 0, None),
    ];
    for (n, (text, start, ending, depth, accepted)) in cases.into_iter().enumerate() {
        let result = run(
            &cert,
            sum["parse_definition"].as_str().unwrap(),
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
        let mut expected = [false; 256];
        if let Some((tag, payload, cells)) = accepted {
            expected[0] = true;
            expected[2] = ending == 0;
            let end = text.len() - usize::from(ending != 0);
            for i in 0..32 {
                expected[(2 + i) << 1] = end & (1 << i) != 0;
                expected[(34 + i) << 1] = cells & (1 << i) != 0;
                // C6 sum: role zero tag, role one i32 payload. C8 packet
                // adds a role selector; unused higher value-address bits are zero.
                expected[1 | ((i << 1) << 1)] = tag & (1 << i) != 0;
                expected[1 | ((1 | (i << 1)) << 1)] = payload & (1 << i) != 0;
            }
        }
        for (i, wanted) in expected.into_iter().enumerate() {
            assert_eq!(
                leaf(&cert, result.clone(), 8, i),
                wanted,
                "case {n}, bit {i}, {text}"
            );
        }
        eprintln!("Option JSON core case {n}: accepted {}", accepted.is_some());
    }
}
