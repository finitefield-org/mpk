//! Bounded observations execute actual definitions, never theorem bodies.
use super::actual::Interpreter;
use mpk_cert::encode::Certificate;
use serde_json::json;

fn full_bits(word: u64) -> Vec<bool> {
    (0..64).map(|i| word & (1u64 << i) != 0).collect()
}
fn save(retain: fn(&str, &str, &[u8]), name: &str, suffix: &str, value: serde_json::Value) {
    retain(
        name,
        suffix,
        format!("{}\n", serde_json::to_string_pretty(&value).unwrap()).as_bytes(),
    );
}
fn endpoint(eval: &mut Interpreter<'_>, n: u64) -> u32 {
    let raw = eval.raw(n, [0; 256]);
    eval.invoke("Mpk.Go.Practical.String.Length", &[raw])
}
pub(super) fn run(c: &Certificate, retain: fn(&str, &str, &[u8])) {
    let mut count = 0;
    let mut steps = 0;
    for (case, n) in [0usize, 1, 2, 7, 8, 31, 64, 255, 256]
        .into_iter()
        .enumerate()
    {
        let mut bytes = std::array::from_fn(|i| ((i * 73 + case * 29 + 37) % 256) as u8);
        if n >= 8 {
            bytes[..8].copy_from_slice(&[0, 255, 192, 128, 226, 130, 172, 0]);
        }
        let mut endpoints = vec![(0, 0), (0, n), (n, n), (n / 2, n), (n / 3, n * 2 / 3)];
        endpoints.sort();
        endpoints.dedup();
        for (a, b) in endpoints {
            let name = format!("carrier-{n}-{a}-{b}");
            let mut expected = [0u8; 256];
            expected[..b - a].copy_from_slice(&bytes[a..b]);
            save(
                retain,
                &name,
                "input.json",
                json!({"length":n,"all256_source_bytes":bytes.to_vec(),"all64_source_length_bits":full_bits(n as u64),"endpoints":[a,b],"all64_low_bits":full_bits(a as u64),"all64_high_bits":full_bits(b as u64),"expected_length":b-a,"expected_all256_output_bytes":expected.to_vec(),"source_or_nominal_authority":false}),
            );
            for normalized in [false, true] {
                let mut eval = Interpreter::new(c);
                let raw = eval.raw(n as u64, bytes);
                let low = endpoint(&mut eval, a as u64);
                let high = endpoint(&mut eval, b as u64);
                let label = format!("{name}-{}", if normalized { "normalized" } else { "raw" });
                let input = if normalized {
                    eval.invoke("Mpk.Go.Practical.String.Normalization.Value", &[raw])
                } else {
                    raw
                };
                let guard =
                    eval.invoke("Mpk.Go.Practical.String.Slice.Defined", &[input, low, high]);
                let guard = eval.boolean(guard);
                save(
                    retain,
                    &label,
                    "guard.partial.json",
                    json!({"defined":guard,"actual_AST_steps":eval.steps()}),
                );
                assert!(guard, "{label}");
                let value = eval.invoke("Mpk.Go.Practical.String.Slice.Value", &[input, low, high]);
                let length = eval.length(value);
                save(
                    retain,
                    &label,
                    "length.partial.json",
                    json!({"defined":guard,"length":length,"all64_result_length_bits":full_bits(length),"actual_AST_steps":eval.steps()}),
                );
                assert_eq!(length, (b - a) as u64, "{label}");
                if length > 0 {
                    let zero = endpoint(&mut eval, 0);
                    let first = eval.invoke("Mpk.Go.Practical.String.Byte", &[value, zero]);
                    let expected_first = eval.byte_input(expected[0]);
                    let first_equal = eval.invoke(
                        "Mpk.Go.Practical.String.Equality.ByteEqual",
                        &[first, expected_first],
                    );
                    let first_equal = eval.boolean(first_equal);
                    save(
                        retain,
                        &label,
                        "first-byte.partial.json",
                        json!({"first_byte_equal":first_equal,"actual_AST_steps":eval.steps()}),
                    );
                    assert!(first_equal, "{label}");
                }
                let initial_steps = eval.steps();
                let (output, actual_steps) = if length <= 64 {
                    let output = eval.bytes(value);
                    (output, eval.steps())
                } else {
                    drop(eval);
                    let mut output = [0u8; 256];
                    let mut actual_steps = initial_steps;
                    for chunk in 0..4 {
                        // Every independent replay receives the complete carrier
                        // and all64 endpoint bits. Only output observations split.
                        let mut part = Interpreter::new(c);
                        let raw = part.raw(n as u64, bytes);
                        let low = endpoint(&mut part, a as u64);
                        let high = endpoint(&mut part, b as u64);
                        let input = if normalized {
                            part.invoke("Mpk.Go.Practical.String.Normalization.Value", &[raw])
                        } else {
                            raw
                        };
                        let value =
                            part.invoke("Mpk.Go.Practical.String.Slice.Value", &[input, low, high]);
                        let no = part.invoke("Std.Bool.false", &[]);
                        let yes = part.invoke("Std.Bool.true", &[]);
                        for (slot, byte) in output.iter_mut().enumerate().skip(chunk * 64).take(64)
                        {
                            for bit in 0..8 {
                                let mut args = vec![value];
                                args.extend((0..8).rev().map(|i| {
                                    if slot & (1 << i) != 0 {
                                        yes
                                    } else {
                                        no
                                    }
                                }));
                                args.extend((0..3).rev().map(|i| {
                                    if bit & (1 << i) != 0 {
                                        yes
                                    } else {
                                        no
                                    }
                                }));
                                let flag = part.invoke("Mpk.Go.Practical.String.SlotBit", &args);
                                if part.boolean(flag) {
                                    *byte |= 1 << bit;
                                }
                            }
                        }
                        actual_steps += part.steps();
                        save(
                            retain,
                            &label,
                            &format!("chunk-{chunk}.partial.json"),
                            json!({"observed_prefix_bytes":output[..(chunk+1)*64].to_vec(),"fresh_interpreter_complete_input":true,"replay_AST_steps":part.steps(),"actual_AST_steps":actual_steps}),
                        );
                        assert_eq!(
                            &output[chunk * 64..(chunk + 1) * 64],
                            &expected[chunk * 64..(chunk + 1) * 64],
                            "{label}:chunk{chunk}"
                        );
                    }
                    (output, actual_steps)
                };
                save(
                    retain,
                    &label,
                    "observation.json",
                    json!({"defined":guard,"length":length,"all64_result_length_bits":full_bits(length),"all256_result_bytes":output.to_vec(),"expected_all256_result_bytes":expected.to_vec(),"actual_AST_steps":actual_steps,"output_replays":if length>64{4}else{1},"complete_carrier_observed":true,"definition_callbacks":false,"theorem_bodies_executed":false}),
                );
                assert_eq!(output, expected, "{label}");
                count += 1;
                steps += actual_steps;
            }
        }
    }
    let mut guards = 0;
    let mut guard_steps = 0;
    for n in [0u64, 1, 8, 255, 256] {
        let mut values = vec![
            0,
            1,
            n,
            n + 1,
            256,
            257,
            1u64 << 63,
            u64::MAX,
            (1u64 << 63) - 1,
        ];
        values.sort();
        values.dedup();
        for &a in &values {
            for &b in &values {
                let name = format!("guard-{n}-{a}-{b}");
                let expected = (a as i64) >= 0 && (b as i64) >= 0 && a <= b && b <= n;
                save(
                    retain,
                    &name,
                    "input.json",
                    json!({"length":n,"all256_source_bytes":vec![173u8;256],"all64_source_length_bits":full_bits(n),"endpoints":[a,b],"all64_low_bits":full_bits(a),"all64_high_bits":full_bits(b),"expected_defined":expected,"source_or_nominal_authority":false}),
                );
                let mut eval = Interpreter::new(c);
                let s = eval.raw(n, [173; 256]);
                let low = endpoint(&mut eval, a);
                let high = endpoint(&mut eval, b);
                let actual = eval.invoke("Mpk.Go.Practical.String.Slice.Defined", &[s, low, high]);
                let actual = eval.boolean(actual);
                save(
                    retain,
                    &name,
                    "observation.json",
                    json!({"defined":actual,"expected_defined":expected,"actual_AST_steps":eval.steps(),"all64_endpoint_bits_preserved":true,"definition_callbacks":false,"theorem_bodies_executed":false}),
                );
                assert_eq!(actual, expected, "{name}");
                guards += 1;
                guard_steps += eval.steps();
            }
        }
    }
    save(
        retain,
        "observations",
        "summary.json",
        json!({"full_carrier_observations":count,"signed_guard_observations":guards,"full_carrier_AST_steps":steps,"signed_guard_AST_steps":guard_steps,"source_or_nominal_authority":false,"observer_ceilings_changed":false}),
    );
    println!("slice actual DAG: {count} complete raw/normalized results, {guards} signed guard cases, {} steps",steps+guard_steps);
}

pub(super) fn definition_countermodel(
    c: &Certificate,
    changed: &str,
    retain: fn(&str, &str, &[u8]),
) {
    let name = format!("definition-{changed}");
    let mut bytes = [173u8; 256];
    bytes[..3].copy_from_slice(&[128, 33, 255]);
    let (a, b) = if changed == "SourceWord" {
        (1u64, 2u64)
    } else if changed == "Defined" {
        (0, 4)
    } else {
        (0, 3)
    };
    save(
        retain,
        &name,
        "input.json",
        json!({"changed_definition":changed,"length":3,"all256_source_bytes":bytes.to_vec(),"all64_source_length_bits":full_bits(3),"endpoints":[a,b],"all64_low_bits":full_bits(a),"all64_high_bits":full_bits(b),"source_or_nominal_authority":false}),
    );
    let mut eval = Interpreter::new(c);
    let s = eval.raw(3, bytes);
    let low = endpoint(&mut eval, a);
    let high = endpoint(&mut eval, b);
    if changed == "Defined" {
        let actual = eval.invoke("Mpk.Go.Practical.String.Slice.Defined", &[s, low, high]);
        let actual = eval.boolean(actual);
        save(
            retain,
            &name,
            "countermodel.json",
            json!({"defined":actual,"expected_defined":false,"actual_wrong_definition_countermodel":true,"actual_AST_steps":eval.steps(),"definition_callbacks":false,"theorem_bodies_executed":false}),
        );
        assert!(actual);
        return;
    }
    let value = eval.invoke("Mpk.Go.Practical.String.Slice.Value", &[s, low, high]);
    let length = eval.length(value);
    let output = eval.bytes(value);
    let mut expected = [0u8; 256];
    expected[..(b - a) as usize].copy_from_slice(&bytes[a as usize..b as usize]);
    save(
        retain,
        &name,
        "countermodel.json",
        json!({"length":length,"all64_result_length_bits":full_bits(length),"all256_result_bytes":output.to_vec(),"expected_length":b-a,"expected_all256_result_bytes":expected.to_vec(),"actual_wrong_definition_countermodel":true,"actual_AST_steps":eval.steps(),"definition_callbacks":false,"theorem_bodies_executed":false}),
    );
    assert!(length != b - a || output != expected, "{changed}");
}
