//! Actual ordinary definitions with complete carriers and unchanged finite ceilings.
use super::actual::Interpreter;
use mpk_cert::encode::Certificate;
use serde_json::json;
fn save(retain: fn(&str, &str, &[u8]), name: &str, ext: &str, v: serde_json::Value) {
    retain(
        name,
        ext,
        format!("{}\n", serde_json::to_string_pretty(&v).unwrap()).as_bytes(),
    );
}
fn bits(w: u64) -> Vec<bool> {
    (0..64).map(|i| w & (1 << i) != 0).collect()
}
fn word(e: &mut Interpreter<'_>, w: u64) -> u32 {
    let s = e.raw(w, [0; 256]);
    e.invoke("Mpk.Go.Practical.String.Length", &[s])
}
fn byte(e: &mut Interpreter<'_>, ty: u32, value: u32) -> u8 {
    let mut out = 0;
    for b in 0..8 {
        let mut args = vec![ty, value];
        args.extend((0..3).rev().map(|i| u32::from(b & (1 << i) != 0)));
        let v = e.invoke("Mpk.Go.Practical.Array.D0.Copy", &args);
        if e.boolean(v) {
            out |= 1 << b;
        }
    }
    out
}
fn array(e: &mut Interpreter<'_>, d: usize, name: &str, args: &[u32]) -> u32 {
    e.invoke(&format!("Mpk.Go.Practical.Array.D{d}.{name}"), args)
}
fn input<'a>(
    c: &'a Certificate,
    n: u64,
    d: usize,
    raw: [u8; 256],
) -> (Interpreter<'a>, u32, u32, u32, u32) {
    let mut e = Interpreter::new(c);
    let b = e.invoke("Std.Bool", &[]);
    let ty = array(&mut e, 3, "Carrier", &[b]);
    let s = e.raw(n, raw);
    assert_eq!(e.length(s), n);
    assert_eq!(e.bytes(s), raw);
    let count = e.invoke("Mpk.Go.Practical.String.Size", &[s]);
    let mut args = vec![s];
    args.extend(vec![0; 8 - d]);
    let f = e.invoke("Mpk.Go.Practical.String.Data", &args);
    let z = e.byte_input(0);
    (e, ty, f, z, count)
}
fn operation(
    e: &mut Interpreter<'_>,
    d: usize,
    name: &str,
    ty: u32,
    f: u32,
    z: u32,
    bounds: (u32, usize),
) -> u32 {
    match name {
        "raw" => f,
        "copy" => array(e, d, "Copy", &[ty, f]),
        "zero" => array(e, d, "Zero", &[ty, z]),
        "normalize" => array(e, d, "Normalize", &[ty, z, bounds.0, f]),
        _ => {
            let i = word(
                e,
                if name == "set-first" {
                    0
                } else {
                    (bounds.1 - 1) as u64
                },
            );
            let x = e.byte_input(7);
            array(e, d, "Set", &[ty, f, i, x])
        }
    }
}
pub(super) fn run(c: &Certificate, retain: fn(&str, &str, &[u8])) {
    let lengths = [
        1usize, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128, 129, 255, 256,
    ];
    let (mut total, mut outputs, mut reads) = (0usize, 0, 0);
    for n in lengths {
        let cap = n.next_power_of_two();
        let d = cap.trailing_zeros() as usize;
        let raw = std::array::from_fn(|i| ((i * 73 + n * 17 + 128) % 256) as u8);
        let name = format!("array-{n:03}-depth{d}");
        save(
            retain,
            &name,
            "input.json",
            json!({"logical_length":n,"depth":d,"capacity":cap,"all256_source_bytes":raw.to_vec(),"all64_length_bits":bits(n as u64),"ordinary_source_or_nominal_proof":false,"source_or_storage_authority":false}),
        );
        for op in ["raw", "copy", "zero", "normalize", "set-first", "set-last"] {
            let mut expected = raw[..cap].to_vec();
            match op {
                "zero" => expected.fill(0),
                "normalize" => expected[n..].fill(0),
                "set-first" => expected[0] = 7,
                "set-last" => expected[n - 1] = 7,
                _ => {}
            }
            let label = format!("{name}-{op}");
            let mut output = vec![];
            let mut used = 0;
            for start in (0..cap).step_by(32) {
                let (mut e, ty, f, z, count) = input(c, n as u64, d, raw);
                let out = operation(&mut e, d, op, ty, f, z, (count, n));
                for j in start..cap.min(start + 32) {
                    let mut args = vec![ty, out];
                    args.extend((0..d).rev().map(|i| u32::from(j & (1 << i) != 0)));
                    let cell = array(&mut e, d, "Copy", &args);
                    output.push(byte(&mut e, ty, cell));
                }
                used += e.steps();
                save(
                    retain,
                    &label,
                    &format!("chunk-{start}.partial.json"),
                    json!({"observed_prefix_bytes":output,"actual_AST_steps":used,"complete_original_input_retained":true}),
                );
                assert_eq!(output, expected[..output.len()], "{label}");
            }
            for j in [0, n / 2, n - 1] {
                let (mut e, ty, f, z, count) = input(c, n as u64, d, raw);
                let out = operation(&mut e, d, op, ty, f, z, (count, n));
                let index = word(&mut e, j as u64);
                let cell = array(&mut e, d, "Read", &[ty, out, index]);
                let actual = byte(&mut e, ty, cell);
                let original = array(&mut e, d, "Read", &[ty, f, index]);
                let original = byte(&mut e, ty, original);
                used += e.steps();
                save(
                    retain,
                    &label,
                    &format!("read-{reads}.partial.json"),
                    json!({"full64_index":j,"all64_index_bits":bits(j as u64),"actual":actual,"expected":expected[j],"original_unchanged_byte":original,"expected_original_byte":raw[j],"actual_AST_steps":e.steps()}),
                );
                assert_eq!(actual, expected[j], "{label}");
                assert_eq!(original, raw[j], "{label}");
                reads += 1;
            }
            save(
                retain,
                &label,
                "observation.json",
                json!({"all_capacity_bytes":output,"expected_all_capacity_bytes":expected,"actual_AST_steps":used,"fresh_output_probes_keep_complete_inputs":true,"definition_callbacks":false,"theorem_bodies_executed":false,"source_or_Go_storage_proof":false}),
            );
            total += used;
            outputs += 1;
        }
    }
    for (case, (n, i)) in [
        (0, 0),
        (1, 0),
        (1, 1),
        (256, 255),
        (256, 256),
        (3, 1u64 << 32),
        (256, 1u64 << 63),
        (256, u64::MAX),
        (u64::MAX, 1u64 << 63),
    ]
    .into_iter()
    .enumerate()
    {
        let name = format!("guard-{case}");
        save(
            retain,
            &name,
            "input.json",
            json!({"length":n,"index":i,"all64_length_bits":bits(n),"all64_index_bits":bits(i),"raw_outside_profile":n>256,"admitted_source_counterexample":false}),
        );
        let mut e = Interpreter::new(c);
        let nw = word(&mut e, n);
        let iw = word(&mut e, i);
        let u = e.invoke("Mpk.Go.Practical.Array.Index.Unsigned", &[nw, iw]);
        let u = e.boolean(u);
        let s = e.invoke("Mpk.Go.Practical.Array.Index.Signed", &[nw, iw]);
        let s = e.boolean(s);
        save(
            retain,
            &name,
            "observation.json",
            json!({"unsigned":u,"expected_unsigned":i<n,"signed":s,"expected_signed":i<n&&i<1u64<<63,"actual_AST_steps":e.steps(),"out_of_bounds_read_executed":false,"definition_callbacks":false,"theorem_bodies_executed":false}),
        );
        assert_eq!(u, i < n);
        assert_eq!(s, i < n && i < 1u64 << 63);
        total += e.steps();
    }
    let mut e = Interpreter::new(c);
    let zero = e.invoke("Mpk.Go.Practical.Array.Empty.Zero", &[]);
    let empty = e.boolean(zero);
    save(
        retain,
        "empty",
        "observation.json",
        json!({"canonical_dummy":empty,"expected_dummy":false,"actual_AST_steps":e.steps(),"source_or_storage_authority":false}),
    );
    assert!(!empty);
    total += e.steps();
    save(
        retain,
        "observation-inventory",
        "json",
        json!({"positive_lengths":22,"all_depths_0_through_8":true,"complete_array_outputs":outputs,"full64_reads":reads,"full64_guard_cases":9,"empty_dummy_cases":1,"actual_AST_steps":total,"observer_ceilings_changed":false,"definition_callbacks":false,"theorem_bodies_executed":false}),
    );
    println!("array outputs={outputs} full64 reads={reads} actual_AST_steps={total}");
}
pub(super) fn countermodel(c: &Certificate, short: &str, retain: fn(&str, &str, &[u8])) {
    let name = format!("definition-{short}");
    let raw = std::array::from_fn(|i| ((i * 73 + 128) % 256) as u8);
    let (d, kind) = if short.starts_with('D') {
        let (a, b) = short.split_once('.').unwrap();
        (a[1..].parse::<usize>().unwrap(), b)
    } else {
        (0, short)
    };
    let n = if kind.starts_with("Index.") {
        0
    } else if d == 0 {
        1
    } else {
        (1 << (d - 1)) + 1
    };
    let index = if kind == "Copy" || kind == "Read" {
        1
    } else {
        0
    };
    save(
        retain,
        &name,
        "input.json",
        json!({"logical_length":n,"depth":d,"all256_source_bytes":raw.to_vec(),"all64_length_bits":bits(n),"all64_index_bits":bits(index),"source_or_nominal_authority":false,"admitted_source_counterexample":false}),
    );
    let (mut e, ty, f, z, count) = input(c, n, d, raw);
    let (actual, expected) = if kind == "Empty.Zero" {
        let v = e.invoke("Mpk.Go.Practical.Array.Empty.Zero", &[]);
        (u64::from(e.boolean(v)), 0)
    } else if kind.starts_with("Index.") {
        let w = word(&mut e, 0);
        let v = e.invoke(&format!("Mpk.Go.Practical.Array.{kind}"), &[w, w]);
        (u64::from(e.boolean(v)), 0)
    } else {
        let i = word(&mut e, index);
        let out = match kind {
            "Normalize" => array(&mut e, d, "Normalize", &[ty, z, count, f]),
            "Set" => {
                let x = e.byte_input(7);
                array(&mut e, d, "Set", &[ty, f, i, x])
            }
            "Copy" => array(&mut e, d, "Copy", &[ty, f]),
            _ => f,
        };
        let cell = array(&mut e, d, "Read", &[ty, out, i]);
        (
            byte(&mut e, ty, cell) as u64,
            if kind == "Set" {
                7
            } else {
                raw[index as usize] as u64
            },
        )
    };
    save(
        retain,
        &name,
        "countermodel.json",
        json!({"actual":actual,"expected":expected,"actual_wrong_definition_countermodel":true,"actual_AST_steps":e.steps(),"definition_callbacks":false,"theorem_bodies_executed":false}),
    );
    assert_ne!(actual, expected, "{short}");
}
