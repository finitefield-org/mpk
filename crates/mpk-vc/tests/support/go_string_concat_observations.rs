//! Actual definitions only: full inputs, exact bits, unchanged finite ceilings.
use super::actual::Interpreter;
use mpk_cert::encode::Certificate;
use serde_json::json;
#[derive(Clone, Copy)]
struct Input {
    n: u64,
    m: u64,
    a: [u8; 256],
    b: [u8; 256],
}
fn bits(word: u64) -> Vec<bool> {
    (0..64).map(|i| word & (1u64 << i) != 0).collect()
}
fn save(retain: fn(&str, &str, &[u8]), name: &str, ext: &str, value: serde_json::Value) {
    retain(
        name,
        ext,
        format!("{}\n", serde_json::to_string_pretty(&value).unwrap()).as_bytes(),
    );
}
fn inputs<'a>(c: &'a Certificate, input: Input, mode: u8) -> (Interpreter<'a>, u32, u32) {
    let mut e = Interpreter::new(c);
    let mut a = e.raw(input.n, input.a);
    let mut b = e.raw(input.m, input.b);
    if mode & 1 != 0 {
        a = e.invoke("Mpk.Go.Practical.String.Normalization.Value", &[a]);
    }
    if mode & 2 != 0 {
        b = e.invoke("Mpk.Go.Practical.String.Normalization.Value", &[b]);
    }
    (e, a, b)
}
fn save_input(retain: fn(&str, &str, &[u8]), name: &str, input: Input) {
    save(
        retain,
        name,
        "input.json",
        json!({"lengths":[input.n,input.m],"all256_left_bytes":input.a.to_vec(),"all256_right_bytes":input.b.to_vec(),"all64_left_length_bits":bits(input.n),"all64_right_length_bits":bits(input.m),"source_or_nominal_authority":false,"admitted_source_counterexample":false}),
    );
}
fn chunk(e: &mut Interpreter<'_>, value: u32, output: &mut [u8; 256], start: usize) {
    let no = e.invoke("Std.Bool.false", &[]);
    let yes = e.invoke("Std.Bool.true", &[]);
    for (slot, byte) in output.iter_mut().enumerate().skip(start).take(64) {
        for bit in 0..8 {
            let mut args = vec![value];
            args.extend(
                (0..8)
                    .rev()
                    .map(|i| if slot & (1 << i) != 0 { yes } else { no }),
            );
            args.extend(
                (0..3)
                    .rev()
                    .map(|i| if bit & (1 << i) != 0 { yes } else { no }),
            );
            let flag = e.invoke("Mpk.Go.Practical.String.SlotBit", &args);
            if e.boolean(flag) {
                *byte |= 1 << bit;
            }
        }
    }
}
pub(super) fn run(c: &Certificate, retain: fn(&str, &str, &[u8])) {
    let pairs = [
        (0, 0),
        (0, 1),
        (1, 0),
        (1, 1),
        (2, 3),
        (3, 2),
        (8, 8),
        (9, 12),
        (31, 32),
        (64, 64),
        (0, 255),
        (255, 0),
        (0, 256),
        (256, 0),
        (1, 255),
        (255, 1),
        (128, 128),
        (127, 129),
        (129, 127),
        (64, 192),
        (192, 64),
        (127, 128),
        (128, 127),
        (65, 191),
        (191, 65),
        (128, 129),
        (129, 128),
        (1, 256),
        (256, 1),
        (256, 256),
        (0, 257),
        (257, 0),
    ];
    let (mut outputs, mut rejected, mut steps) = (0, 0, 0);
    for (case, (n, m)) in pairs.into_iter().enumerate() {
        let mut a = std::array::from_fn(|i| ((i * 73 + case * 29 + 37) % 256) as u8);
        let mut b = std::array::from_fn(|i| ((i * 61 + case * 11 + 91) % 256) as u8);
        if n >= 8 {
            a[..8].copy_from_slice(&[0, 255, 192, 128, 226, 130, 172, 0]);
        }
        if m >= 8 {
            b[..8].copy_from_slice(&[255, 0, 128, 192, 0, 226, 130, 172]);
        }
        if n == 9 {
            a[..9].copy_from_slice("日本語".as_bytes());
        }
        if m == 12 {
            b[..12].copy_from_slice("Go🙂世界".as_bytes());
        }
        let input = Input {
            n: n as u64,
            m: m as u64,
            a,
            b,
        };
        let name = format!("carrier-{case:02}-{n}-{m}");
        save_input(retain, &name, input);
        let expected_fit = n + m <= 256;
        for mode in 0..4 {
            let label = format!("{name}-mode{mode}");
            let (mut e, s, t) = inputs(c, input, mode);
            let fit = e.invoke("Mpk.Go.Practical.String.Concat.Fits", &[s, t]);
            let fit = e.boolean(fit);
            save(
                retain,
                &label,
                "fit.partial.json",
                json!({"fits":fit,"expected_fits":expected_fit,"Go_panic_claimed":false,"actual_AST_steps":e.steps()}),
            );
            assert_eq!(fit, expected_fit, "{label}");
            let carry = e.invoke("Mpk.Go.Practical.String.Concat.Carry", &[s, t]);
            let carry = e.boolean(carry);
            let value = e.invoke("Mpk.Go.Practical.String.Concat.Value", &[s, t]);
            let length = e.length(value);
            save(
                retain,
                &label,
                "length.partial.json",
                json!({"fits":fit,"carry":carry,"length":length,"all64_result_length_bits":bits(length),"actual_AST_steps":e.steps()}),
            );
            assert!(!carry, "{label}");
            assert_eq!(length, (n + m) as u64, "{label}");
            if !fit {
                save(
                    retain,
                    &label,
                    "domain.json",
                    json!({"fits":false,"expected_fits":false,"length":length,"carry":carry,"Go_panic_claimed":false,"full_input_preserved":true,"actual_AST_steps":e.steps(),"definition_callbacks":false,"theorem_bodies_executed":false}),
                );
                steps += e.steps();
                rejected += 1;
                continue;
            }
            let mut expected = [0u8; 256];
            expected[..n].copy_from_slice(&a[..n]);
            expected[n..n + m].copy_from_slice(&b[..m]);
            if length > 0 {
                let zero = e.raw(0, [0; 256]);
                let index = e.invoke("Mpk.Go.Practical.String.Length", &[zero]);
                let first = e.invoke("Mpk.Go.Practical.String.Byte", &[value, index]);
                let expected_first = e.byte_input(expected[0]);
                let first_equal = e.invoke(
                    "Mpk.Go.Practical.String.Equality.ByteEqual",
                    &[first, expected_first],
                );
                let first_equal = e.boolean(first_equal);
                save(
                    retain,
                    &label,
                    "first-byte.partial.json",
                    json!({"first_byte_equal":first_equal,"actual_AST_steps":e.steps()}),
                );
                assert!(first_equal, "{label}");
            }
            let initial = e.steps();
            let (output, actual_steps) = if length <= 64 {
                let output = e.bytes(value);
                (output, e.steps())
            } else {
                drop(e);
                let mut output = [0u8; 256];
                let mut actual_steps = initial;
                for part in 0..4 {
                    let (mut replay, s, t) = inputs(c, input, mode);
                    let value = replay.invoke("Mpk.Go.Practical.String.Concat.Value", &[s, t]);
                    chunk(&mut replay, value, &mut output, part * 64);
                    actual_steps += replay.steps();
                    save(
                        retain,
                        &label,
                        &format!("chunk-{part}.partial.json"),
                        json!({"observed_prefix_bytes":output[..(part+1)*64].to_vec(),"fresh_interpreter_complete_inputs":true,"replay_AST_steps":replay.steps(),"actual_AST_steps":actual_steps}),
                    );
                    assert_eq!(
                        &output[part * 64..(part + 1) * 64],
                        &expected[part * 64..(part + 1) * 64],
                        "{label}:chunk{part}"
                    );
                }
                (output, actual_steps)
            };
            save(
                retain,
                &label,
                "observation.json",
                json!({"fits":fit,"carry":carry,"length":length,"all64_result_length_bits":bits(length),"all256_result_bytes":output.to_vec(),"expected_all256_result_bytes":expected.to_vec(),"actual_AST_steps":actual_steps,"output_replays":if length>64{4}else{1},"complete_carrier_observed":true,"definition_callbacks":false,"theorem_bodies_executed":false}),
            );
            assert_eq!(output, expected, "{label}");
            outputs += 1;
            steps += actual_steps;
        }
    }
    let (mut carries, mut carry_steps) = (0, 0);
    for (n, m) in [
        (u64::MAX, 1),
        (1, u64::MAX),
        (1u64 << 63, 1u64 << 63),
        (u64::MAX, u64::MAX),
        (u64::MAX, 0),
        (0, u64::MAX),
        (1u64 << 63, 0),
        (0, 1u64 << 63),
    ] {
        let name = format!("carry-{n}-{m}");
        let input = Input {
            n,
            m,
            a: [128; 256],
            b: [33; 256],
        };
        save_input(retain, &name, input);
        let (mut e, s, t) = inputs(c, input, 0);
        let carry = e.invoke("Mpk.Go.Practical.String.Concat.Carry", &[s, t]);
        let carry = e.boolean(carry);
        let value = e.invoke("Mpk.Go.Practical.String.Concat.Value", &[s, t]);
        let length = e.length(value);
        let natural_sum = u128::from(n) + u128::from(m);
        let expected_carry = natural_sum > u128::from(u64::MAX);
        let expected_length = n.wrapping_add(m);
        save(
            retain,
            &name,
            "observation.json",
            json!({"carry":carry,"expected_carry":expected_carry,"length":length,"expected_length":expected_length,"all64_result_length_bits":bits(length),"natural_sum_decimal":natural_sum.to_string(),"fits_evaluated":false,"Go_panic_claimed":false,"actual_AST_steps":e.steps(),"definition_callbacks":false,"theorem_bodies_executed":false}),
        );
        assert_eq!(carry, expected_carry, "{name}");
        assert_eq!(length, expected_length, "{name}");
        carries += 1;
        carry_steps += e.steps();
    }
    save(
        retain,
        "observations",
        "summary.json",
        json!({"input_pairs":pairs.len(),"whole_carrier_observations":outputs,"capacity_rejections":rejected,"full64_carry_cases":carries,"whole_and_domain_AST_steps":steps,"full64_carry_AST_steps":carry_steps,"observer_ceilings_changed":false,"source_or_nominal_authority":false}),
    );
    println!("concat actual DAG: {outputs} full outputs, {rejected} capacity rejections, {carries} full64 carry cases, {} steps",steps+carry_steps);
}
pub(super) fn definition_countermodel(
    c: &Certificate,
    changed: &str,
    retain: fn(&str, &str, &[u8]),
) {
    let name = format!("definition-{changed}");
    let (mut a, mut b) = ([173; 256], [207; 256]);
    a[0] = 128;
    b[..2].copy_from_slice(&[33, 255]);
    let (n, m) = match changed {
        "Fits" => (128, 129),
        "Carry" => (u64::MAX, 1),
        _ => (1, 2),
    };
    let input = Input { n, m, a, b };
    save_input(retain, &name, input);
    let (mut e, s, t) = inputs(c, input, 0);
    if changed == "CapacityWord" {
        let word = e.invoke("Mpk.Go.Practical.String.Concat.CapacityWord", &[]);
        let zero = e.invoke("Mpk.Go.Practical.String.Zero", &[]);
        let data = e.invoke("Mpk.Go.Practical.String.Data", &[zero]);
        let boxed = e.invoke("Mpk.Go.Practical.String.Make", &[word, data]);
        let actual = e.length(boxed);
        save(
            retain,
            &name,
            "countermodel.json",
            json!({"capacity_word":actual,"expected_capacity_word":256,"all64_capacity_word_bits":bits(actual),"actual_wrong_definition_countermodel":true,"actual_AST_steps":e.steps(),"definition_callbacks":false,"theorem_bodies_executed":false}),
        );
        assert_ne!(actual, 256);
        return;
    }
    if matches!(changed, "Fits" | "Carry") {
        let actual = e.invoke(
            &format!("Mpk.Go.Practical.String.Concat.{changed}"),
            &[s, t],
        );
        let actual = e.boolean(actual);
        let expected = changed == "Carry";
        save(
            retain,
            &name,
            "countermodel.json",
            json!({"actual":actual,"expected":expected,"actual_wrong_definition_countermodel":true,"Go_panic_claimed":false,"actual_AST_steps":e.steps(),"definition_callbacks":false,"theorem_bodies_executed":false}),
        );
        assert_ne!(actual, expected);
        return;
    }
    let value = e.invoke("Mpk.Go.Practical.String.Concat.Value", &[s, t]);
    let length = e.length(value);
    let output = e.bytes(value);
    let mut expected = [0u8; 256];
    expected[..3].copy_from_slice(&[128, 33, 255]);
    save(
        retain,
        &name,
        "countermodel.json",
        json!({"length":length,"all64_result_length_bits":bits(length),"all256_result_bytes":output.to_vec(),"expected_length":3,"expected_all256_result_bytes":expected.to_vec(),"actual_wrong_definition_countermodel":true,"actual_AST_steps":e.steps(),"definition_callbacks":false,"theorem_bodies_executed":false}),
    );
    assert!(length != 3 || output != expected, "{changed}");
}
