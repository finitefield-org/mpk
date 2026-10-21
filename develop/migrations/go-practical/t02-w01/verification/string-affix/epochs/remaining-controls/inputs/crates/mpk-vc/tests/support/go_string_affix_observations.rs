//! Execute complete affix predicates without callbacks or theorem execution.
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
fn save(retain: fn(&str, &str, &[u8]), name: &str, ext: &str, value: serde_json::Value) {
    retain(
        name,
        ext,
        format!("{}\n", serde_json::to_string_pretty(&value).unwrap()).as_bytes(),
    );
}
fn bits(n: u64) -> Vec<bool> {
    (0..64).map(|i| n & (1 << i) != 0).collect()
}
fn save_input(retain: fn(&str, &str, &[u8]), name: &str, i: Input) {
    save(
        retain,
        name,
        "input.json",
        json!({"lengths":[i.n,i.m],"all256_left_bytes":i.a.to_vec(),"all256_right_bytes":i.b.to_vec(),"all64_left_length_bits":bits(i.n),"all64_right_length_bits":bits(i.m),"source_or_nominal_authority":false,"admitted_source_counterexample":false}),
    );
}
fn inputs<'a>(c: &'a Certificate, i: Input, mode: u8) -> (Interpreter<'a>, u32, u32) {
    let mut e = Interpreter::new(c);
    let mut s = e.raw(i.n, i.a);
    let mut p = e.raw(i.m, i.b);
    if mode & 1 != 0 {
        s = e.invoke("Mpk.Go.Practical.String.Normalization.Value", &[s]);
    }
    if mode & 2 != 0 {
        p = e.invoke("Mpk.Go.Practical.String.Normalization.Value", &[p]);
    }
    (e, s, p)
}
fn af(e: &mut Interpreter<'_>, short: &str, args: &[u32]) -> u32 {
    e.invoke(&format!("Mpk.Go.Practical.String.Affix.{short}"), args)
}
fn word(e: &mut Interpreter<'_>, n: u64) -> u32 {
    let s = e.raw(n, [0; 256]);
    e.invoke("Mpk.Go.Practical.String.Length", &[s])
}
fn read_word(e: &mut Interpreter<'_>, w: u32) -> u64 {
    let zero = e.raw(0, [0; 256]);
    let data = e.invoke("Mpk.Go.Practical.String.Data", &[zero]);
    let boxed = e.invoke("Mpk.Go.Practical.String.Make", &[w, data]);
    e.length(boxed)
}
pub(super) fn run(c: &Certificate, retain: fn(&str, &str, &[u8])) {
    let mut cases = vec![];
    // Full-capacity true folds run first, with a fresh complete interpreter for each predicate.
    for (n, m, kind) in [
        (256, 256, 0),
        (256, 255, 1),
        (255, 255, 0),
        (256, 1, 2),
        (0, 0, 0),
        (0, 1, 0),
        (1, 0, 0),
        (256, 0, 0),
        (1, 1, 0),
        (1, 1, 3),
        (2, 3, 0),
        (8, 3, 1),
        (8, 3, 2),
        (8, 8, 3),
        (8, 8, 4),
        (256, 256, 3),
        (256, 256, 4),
        (128, 127, 1),
        (128, 127, 2),
    ] {
        let a = std::array::from_fn(|j| ((j * 73 + 128) % 256) as u8);
        let mut b = std::array::from_fn(|j| ((j * 61 + 11) % 256) as u8);
        if m <= n {
            let start = if kind == 2 { n - m } else { 0 };
            b[..m].copy_from_slice(&a[start..start + m]);
        }
        if m > 0 && kind == 3 {
            b[0] ^= 1;
        }
        if m > 0 && kind == 4 {
            b[m - 1] ^= 1;
        }
        cases.push(Input {
            n: n as u64,
            m: m as u64,
            a,
            b,
        });
    }
    for pattern in ["日本語", "世界", "Go🙂", "日本語Go🙂世界", ""] {
        let mut a = [173; 256];
        let mut b = [91; 256];
        let bytes = "日本語Go🙂世界".as_bytes();
        a[..bytes.len()].copy_from_slice(bytes);
        let p = pattern.as_bytes();
        b[..p.len()].copy_from_slice(p);
        cases.push(Input {
            n: bytes.len() as u64,
            m: p.len() as u64,
            a,
            b,
        });
    }
    for p in [[0, 255, 192, 128], [128, 0, 0, 255], [0, 0, 0, 0]] {
        let mut a = [23; 256];
        let mut b = [99; 256];
        a[..8].copy_from_slice(&[0, 255, 192, 128, 128, 0, 0, 255]);
        b[..4].copy_from_slice(&p);
        cases.push(Input { n: 8, m: 4, a, b });
    }
    let (mut total, mut results, mut positions) = (0usize, 0, 0);
    for (case, i) in cases.into_iter().enumerate() {
        let name = format!("carrier-{case:02}-{}-{}", i.n, i.m);
        save_input(retain, &name, i);
        for mode in 0..4 {
            for suffix in [false, true] {
                let label = format!(
                    "{name}-mode{mode}-{}",
                    if suffix { "suffix" } else { "prefix" }
                );
                let expected = i.m <= i.n
                    && if suffix {
                        i.a[..i.n as usize].ends_with(&i.b[..i.m as usize])
                    } else {
                        i.a[..i.n as usize].starts_with(&i.b[..i.m as usize])
                    };
                let (mut e, s, p) = inputs(c, i, mode);
                let value = af(
                    &mut e,
                    if suffix { "HasSuffix" } else { "HasPrefix" },
                    &[s, p],
                );
                let actual = e.boolean(value);
                save(
                    retain,
                    &label,
                    "predicate.partial.json",
                    json!({"actual":actual,"expected":expected,"actual_AST_steps":e.steps()}),
                );
                assert_eq!(actual, expected, "{label}");
                total += e.steps();
                results += 1;
                drop(e);
                // Metadata/output probes use a separate fresh interpreter with exactly the same inputs.
                let (mut e, s, p) = inputs(c, i, mode);
                let ns = e.length(s);
                let np = e.length(p);
                let a = e.bytes(s);
                let b = e.bytes(p);
                let mut wanted_a = i.a;
                let mut wanted_b = i.b;
                if mode & 1 != 0 {
                    wanted_a[i.n as usize..].fill(0);
                }
                if mode & 2 != 0 {
                    wanted_b[i.m as usize..].fill(0);
                }
                save(
                    retain,
                    &label,
                    "inputs.partial.json",
                    json!({"all256_left_bytes":a.to_vec(),"all256_right_bytes":b.to_vec(),"lengths":[ns,np],"actual_AST_steps":e.steps()}),
                );
                assert_eq!((ns, np), (i.n, i.m), "{label}");
                assert_eq!(a, wanted_a, "{label}");
                assert_eq!(b, wanted_b, "{label}");
                if i.m > 0 {
                    let zero = word(&mut e, 0);
                    let first = e.invoke("Mpk.Go.Practical.String.Byte", &[p, zero]);
                    let want = e.byte_input(i.b[0]);
                    let h = e.invoke("Mpk.Go.Practical.String.Equality.ByteEqual", &[first, want]);
                    let first = e.boolean(h);
                    save(
                        retain,
                        &label,
                        "first-byte.partial.json",
                        json!({"first_byte_equal":first,"actual_AST_steps":e.steps()}),
                    );
                    assert!(first, "{label}");
                }
                total += e.steps();
                drop(e);
                let mut observed = vec![];
                if i.m <= i.n {
                    let (mut e, s, p) = inputs(c, i, mode);
                    let k = e.invoke(
                        if suffix {
                            "Std.Bool.true"
                        } else {
                            "Std.Bool.false"
                        },
                        &[],
                    );
                    let off = af(&mut e, "Offset", &[s, p, k]);
                    let offset = read_word(&mut e, off);
                    let expected_offset = if suffix { i.n - i.m } else { 0 };
                    save(
                        retain,
                        &label,
                        "offset.partial.json",
                        json!({"offset":offset,"expected_offset":expected_offset,"all64_offset_bits":bits(offset),"actual_AST_steps":e.steps()}),
                    );
                    assert_eq!(offset, expected_offset, "{label}");
                    for j in [0, i.m / 2, i.m.saturating_sub(1)] {
                        if j >= i.m {
                            continue;
                        }
                        let index = word(&mut e, j);
                        let at = af(&mut e, "Index", &[s, p, k, index]);
                        let value = read_word(&mut e, at);
                        observed.push(json!({"index":j,"actual_source_index":value,"expected_source_index":expected_offset+j,"all64_source_index_bits":bits(value)}));
                        save(
                            retain,
                            &label,
                            &format!("positions-{}.partial.json", observed.len()),
                            json!({"positions":observed,"actual_AST_steps":e.steps()}),
                        );
                        assert_eq!(value, expected_offset + j, "{label}");
                        positions += 1;
                    }
                    total += e.steps();
                }
                save(
                    retain,
                    &label,
                    "observation.json",
                    json!({"actual":actual,"expected":expected,"positions":observed,"complete_input_carriers_observed":true,"definition_callbacks":false,"theorem_bodies_executed":false}),
                );
            }
        }
    }
    for (case, (n, m)) in [(0, 1u64 << 63), (256, u64::MAX), (255, 256)]
        .into_iter()
        .enumerate()
    {
        let i = Input {
            n,
            m,
            a: [128; 256],
            b: [128; 256],
        };
        let name = format!("wide-guard-{case}");
        save_input(retain, &name, i);
        for suffix in [false, true] {
            let (mut e, s, p) = inputs(c, i, 0);
            let value = af(
                &mut e,
                if suffix { "HasSuffix" } else { "HasPrefix" },
                &[s, p],
            );
            let actual = e.boolean(value);
            save(
                retain,
                &format!("{name}-{suffix}"),
                "observation.json",
                json!({"actual":actual,"expected":false,"raw_outside_pattern_capacity":m>256,"definition_callbacks":false,"theorem_bodies_executed":false,"actual_AST_steps":e.steps()}),
            );
            assert!(!actual);
            total += e.steps();
        }
    }
    save(
        retain,
        "observation-inventory",
        "json",
        json!({"whole_input_pairs":27,"normalization_modes":4,"complete_predicate_results":results,"full64_positions":positions,"full64_guard_cases":3,"full64_guard_results":6,"actual_AST_steps":total,"definition_callbacks":false,"theorem_bodies_executed":false,"unchanged_finite_ceilings":true}),
    );
    println!(
        "affix actual predicates={results} full64 positions={positions} actual_AST_steps={total}"
    );
}
pub(super) fn definition_countermodel(c: &Certificate, short: &str, retain: fn(&str, &str, &[u8])) {
    let mut i = Input {
        n: 2,
        m: 1,
        a: [0; 256],
        b: [0; 256],
    };
    i.a[..2].copy_from_slice(&[128, 33]);
    i.b[0] = 33;
    if matches!(short, "Bound" | "Match" | "HasPrefix" | "HasSuffix") {
        i.n = 0;
        i.m = 1;
        i.b[0] = 0;
    }
    if short == "Flags" {
        i.n = 1;
        i.m = 1;
        i.b[0] = 33;
    }
    let name = format!("definition-{short}");
    save_input(retain, &name, i);
    let (mut e, s, p) = inputs(c, i, 0);
    let k = e.invoke("Std.Bool.true", &[]);
    let (actual, expected) = if short == "Bound" {
        let x = af(&mut e, "Bound", &[s, p]);
        (e.boolean(x), false)
    } else if short == "OffsetNat" {
        let x = af(&mut e, "OffsetNat", &[s, p, k]);
        let one = e.raw(1, [0; 256]);
        let y = e.invoke("Mpk.Go.Practical.String.Size", &[one]);
        let equal = e.invoke("Mpk.Go.Practical.Nat.Equal", &[x, y]);
        (e.boolean(equal), true)
    } else {
        let op = if short == "HasPrefix" {
            "HasPrefix"
        } else {
            "HasSuffix"
        };
        let x = af(&mut e, op, &[s, p]);
        (e.boolean(x), matches!(short, "Read" | "Offset" | "Index"))
    };
    save(
        retain,
        &name,
        "countermodel.json",
        json!({"actual":actual,"expected":expected,"observed_expression":if short=="OffsetNat" {"Nat.Equal(actual offset,1)"}else if short=="Bound" {"Bound"}else if short=="HasPrefix" {"HasPrefix"}else {"HasSuffix"},"actual_wrong_definition_countermodel":true,"actual_AST_steps":e.steps(),"definition_callbacks":false,"theorem_bodies_executed":false}),
    );
    assert_ne!(actual, expected, "{short}");
}
