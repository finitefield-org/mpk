//! Ordinal UTF-16 operations via statically composed concrete state transformers.
use super::*;
pub(super) const OPS: &[&str] = &[
    "string.equality.operator",
    "string.inequality.operator",
    "string.equals.ordinal",
    "string.equals.instance.ordinal",
    "string.compare.ordinal",
    "string.contains.ordinal",
    "string.starts_with.ordinal",
    "string.ends_with.ordinal",
];
fn name(s: &str) -> String {
    format!("{PREFIX}.String.Ordinal.{s}")
}
fn word(b: &mut Builder, value: u32) -> R<u32> {
    let f = b.constant("Std.Bool.false")?;
    let t = b.constant("Std.Bool.true")?;
    let bits = (0..32)
        .map(|i| if value & (1 << i) == 0 { f } else { t })
        .collect::<Vec<_>>();
    let body = core_select(b, &bits, 5, 0, 5, f)?;
    b.wrap_selectors(5, body)
}
fn index_word(b: &mut Builder) -> R<u32> {
    let f = b.constant("Std.Bool.false")?;
    let mut bits = vec![f; 32];
    for (i, bit) in bits.iter_mut().enumerate().take(14) {
        *bit = b.var(18 - i as u32)?;
    }
    let body = core_select(b, &bits, 5, 0, 5, f)?;
    b.wrap_selectors(5, body)
}
fn mux_word(b: &mut Builder, condition: u32, yes: u32, no: u32) -> R<u32> {
    call(
        b,
        &format!("{PREFIX}.Cube.D5.Mux"),
        vec![condition, yes, no],
    )
}
pub(super) struct Aux {
    add: String,
    sub: String,
    min: String,
    equal: String,
    difference: String,
    window: String,
}
impl Aux {
    pub(super) fn new(b: &mut Builder, h: &Helpers) -> R<Self> {
        if !b.globals.contains_key(&format!("{PREFIX}.Cube.D5.Mux")) {
            b.helpers(5)?;
        }
        let mut binary = |id: &str| -> R<String> {
            let mut c = Circuit::new(&[32, 32]);
            let a = c.inputs[0].clone();
            let v = c.inputs[1].clone();
            let output = match id {
                "Add" => c.add(&a, &v, F).0,
                "Sub" => c.sub(&a, &v).0,
                "Min" => {
                    let lt = c.lt(&a, &v, false);
                    c.select(lt, &a, &v)
                }
                _ => unreachable!(),
            };
            Ok(construct::word_operation(b, &name(id), c, output)?.result_definition)
        };
        let add = binary("Add")?;
        let sub = binary("Sub")?;
        let min = binary("Min")?;
        let mut c = Circuit::new(&[32, 32]);
        let a = c.inputs[0].clone();
        let v = c.inputs[1].clone();
        let eq = c.equal(&a, &v);
        let equal = small_check(b, &name("EqualLength"), c, eq)?.result_definition;
        let mut c = Circuit::new(&[16, 16]);
        let a = Circuit::extend(&c.inputs[0], 32, false);
        let v = Circuit::extend(&c.inputs[1], 32, false);
        let delta = c.sub(&a, &v).0;
        let p = IntegerCircuit {
            signature: ClosedOperationSignature {
                id: name("CharDifference"),
                tag: ClosedOperationTag::Data,
                argument_type_ids: vec!["mpk.csharp.value.char.v1".into(); 2],
                normal_result_type_id: I32_TYPE_ID.into(),
                ordered_checks: vec![],
            },
            circuit: c,
            output: delta,
            failures: vec![],
        };
        let difference = emit_circuit(b, p, "StringOrdinal")?.result_definition;
        let aux = Self {
            add,
            sub,
            min,
            equal,
            difference,
            window: name("WindowDifference"),
        };
        aux.folds(b, h)?;
        // Four concrete arguments: left text, right text, left offset, count.
        // Fourteen index selectors build the full word-valued predicate cube.
        let left = b.var(17)?;
        let right = b.var(16)?;
        let start = b.var(15)?;
        let index = index_word(b)?;
        let left_index = call(b, &aux.add, vec![start, index])?;
        let a = call(b, &h.read, vec![left, left_index])?;
        let v = call(b, &h.read, vec![right, index])?;
        let delta = call(b, &aux.difference, vec![a, v])?;
        let predicate = b.wrap_selectors(14, delta)?;
        let count = b.var(0)?;
        let result = call(b, &name("First.D14"), vec![predicate, count])?;
        define(
            b,
            &aux.window,
            &[1 << h.depth, 1 << h.depth, 32, 32],
            5,
            result,
        )?;
        Ok(aux)
    }
    fn folds(&self, b: &mut Builder, h: &Helpers) -> R<()> {
        // Read-only predicate and length values are fixed ordinary arguments.
        // Mutable state S=C6 contains just the index and first nonzero result.
        // Every instantiated step has the same concrete S->S type; the outer
        // definition closes all value binders and has no type parameters.
        const STATE: u32 = 6;
        if !b
            .globals
            .contains_key(&format!("{PREFIX}.Cube.D{STATE}.Mux"))
        {
            b.helpers(STATE)?;
        }
        for (field, role) in [("Index", false), ("Value", true)] {
            let source = b.var(5)?;
            let selector = b.constant(if role {
                "Std.Bool.true"
            } else {
                "Std.Bool.false"
            })?;
            let mut args = vec![selector];
            args.extend(b.selectors(5)?);
            let body = b.app(source, args)?;
            let body = b.wrap_selectors(5, body)?;
            define(b, &name(field), &[1 << STATE], 5, body)?;
        }
        // Make(index,value) uses one role bit and five word selectors.
        let index = b.var(7)?;
        let value = b.var(6)?;
        let selectors = b.selectors(5)?;
        let index = b.app(index, selectors.clone())?;
        let value = b.app(value, selectors)?;
        let role = b.var(5)?;
        let body = core_mux(b, role, value, index)?;
        let body = b.wrap_selectors(STATE, body)?;
        define(b, &name("Make"), &[32, 32], STATE, body)?;
        // Active(length,state) requires both an unvisited index and zero result.
        let length = b.var(1)?;
        let state = b.var(0)?;
        let index = call(b, &name("Index"), vec![state])?;
        let value = call(b, &name("Value"), vec![state])?;
        let within = call(b, &h.range.result_definition, vec![index, length])?;
        let empty = call(b, &h.empty.result_definition, vec![value])?;
        let body = call(b, "Std.Bool.and", vec![within, empty])?;
        define(b, &name("Active"), &[32, 1 << STATE], 0, body)?;
        let predicate = b.var(6)?;
        let index = b.var(5)?;
        let mut args = (0..14)
            .map(|i| core_read(b, index, i, 5))
            .collect::<R<Vec<_>>>()?;
        args.extend(b.selectors(5)?);
        let body = b.app(predicate, args)?;
        let body = b.wrap_selectors(5, body)?;
        define(b, &name("ReadAt"), &[1 << 19, 32], 5, body)?;
        // StepTwo(predicate,length) is a concrete S->S value. Two ordered
        // reads are a finite circuit, not two hidden composition occurrences.
        let predicate = b.var(2)?;
        let length = b.var(1)?;
        let state = b.var(0)?;
        let index = call(b, &name("Index"), vec![state])?;
        let one = word(b, 1)?;
        let two = word(b, 2)?;
        let zero = word(b, 0)?;
        let next = call(b, &self.add, vec![index, one])?;
        let after = call(b, &self.add, vec![index, two])?;
        let first = call(b, &name("ReadAt"), vec![predicate, index])?;
        let second = call(b, &name("ReadAt"), vec![predicate, next])?;
        let second_within = call(b, &h.range.result_definition, vec![next, length])?;
        let second = mux_word(b, second_within, second, zero)?;
        let empty = call(b, &h.empty.result_definition, vec![first])?;
        let value = mux_word(b, empty, second, first)?;
        let updated = call(b, &name("Make"), vec![after, value])?;
        let active = call(b, &name("Active"), vec![length, state])?;
        let body = call(
            b,
            &format!("{PREFIX}.Cube.D{STATE}.Mux"),
            vec![active, updated, state],
        )?;
        define(b, &name("StepTwo"), &[1 << 19, 32, 1 << STATE], STATE, body)?;
        // GuardedCompose(length,f,g,state): f precedes g, and a completed
        // intermediate state skips the right subtree by ordinary Bool mux.
        let state_type = b.cube(STATE)?;
        let transformer = b.pi(state_type, state_type)?;
        let word_type = b.cube(5)?;
        let f = b.var(2)?;
        let state = b.var(0)?;
        let first = b.app(f, vec![state])?;
        let length = b.var(4)?;
        let g = b.var(2)?;
        let intermediate = b.var(0)?;
        let next = b.app(g, vec![intermediate])?;
        let active = call(b, &name("Active"), vec![length, intermediate])?;
        let body = call(
            b,
            &format!("{PREFIX}.Cube.D{STATE}.Mux"),
            vec![active, next, intermediate],
        )?;
        let body = b.term(TermNode::Let {
            ty: state_type,
            value: first,
            body,
        })?;
        let body = b.lam(state_type, body)?;
        let body = b.lam(transformer, body)?;
        let body = b.lam(transformer, body)?;
        let body = b.lam(word_type, body)?;
        let ty = b.pi(transformer, transformer)?;
        let ty = b.pi(transformer, ty)?;
        let ty = b.pi(word_type, ty)?;
        b.define(&name("GuardedCompose"), ty, body)?;
        let predicate = b.var(1)?;
        let length = b.var(0)?;
        let step = call(b, &name("StepTwo"), vec![predicate, length])?;
        let composition = call(b, &name("GuardedCompose"), vec![length])?;
        let pipeline = b.compose_term(STATE, composition, &vec![step; 8192])?;
        let body = b.lam(word_type, pipeline)?;
        let predicate_type = b.cube(19)?;
        let body = b.lam(predicate_type, body)?;
        let ty = b.pi(word_type, transformer)?;
        let ty = b.pi(predicate_type, ty)?;
        b.define(&name("Pipeline"), ty, body)?;
        for depth in 0..=14 {
            let input = b.var(20)?;
            let mut args = (0..depth).map(|i| b.var(18 - i)).collect::<R<Vec<_>>>()?;
            args.extend(b.selectors(5)?);
            let body = b.app(input, args)?;
            let predicate = b.wrap_selectors(19, body)?;
            let count = b.var(0)?;
            let cap = word(b, 1 << depth)?;
            let count = call(b, &self.min, vec![count, cap])?;
            let initial = call(b, &name("Make"), vec![zero, zero])?;
            let final_state = call(b, &name("Pipeline"), vec![predicate, count, initial])?;
            let value = call(b, &name("Value"), vec![final_state])?;
            define(
                b,
                &name(&format!("First.D{depth}")),
                &[1 << (depth + 5), 32],
                5,
                value,
            )?;
            let input = b.var(depth + 6)?;
            let args = (0..depth)
                .map(|i| b.var(depth + 4 - i))
                .collect::<R<Vec<_>>>()?;
            let leaf = b.app(input, args)?;
            let f = b.constant("Std.Bool.false")?;
            let mut padding = f;
            for i in 0..5 {
                let bit = b.var(i)?;
                padding = call(b, "Std.Bool.or", vec![padding, bit])?;
            }
            let leaf = core_mux(b, padding, f, leaf)?;
            let predicate = b.wrap_selectors(depth + 5, leaf)?;
            let count = b.var(0)?;
            let first = call(b, &name(&format!("First.D{depth}")), vec![predicate, count])?;
            let empty = call(b, &h.empty.result_definition, vec![first])?;
            let value = call(b, "Std.Bool.not", vec![empty])?;
            define(
                b,
                &name(&format!("Any.D{depth}")),
                &[1 << depth, 32],
                0,
                value,
            )?;
        }
        Ok(())
    }
}

pub(super) fn emit(
    b: &mut Builder,
    h: &Helpers,
    a: &Aux,
    signature: ClosedOperationSignature,
) -> R<OrdinaryStringDefinition> {
    let id = signature.id.as_str();
    let left = b.var(1)?;
    let right = b.var(0)?;
    let lp = call(b, &h.present, vec![left])?;
    let rp = call(b, &h.present, vec![right])?;
    let llen = call(b, &h.length, vec![left])?;
    let rlen = call(b, &h.length, vec![right])?;
    let f = b.constant("Std.Bool.false")?;
    let t = b.constant("Std.Bool.true")?;
    let zero = word(b, 0)?;
    let search = matches!(
        id,
        "string.contains.ordinal" | "string.starts_with.ordinal" | "string.ends_with.ordinal"
    );
    let instance = id == "string.equals.instance.ordinal";
    let compare = id == "string.compare.ordinal";
    let (success, failures) = if search {
        let ok = call(b, "Std.Bool.and", vec![lp, rp])?;
        let null_receiver = call(b, "Std.Bool.not", vec![lp])?;
        let null_arg = call(b, "Std.Bool.not", vec![rp])?;
        let null_arg = call(b, "Std.Bool.and", vec![lp, null_arg])?;
        (ok, vec![null_receiver, null_arg])
    } else if instance {
        let null_receiver = call(b, "Std.Bool.not", vec![lp])?;
        (lp, vec![null_receiver])
    } else {
        (t, vec![])
    };
    let result = if search {
        let too_short = call(b, &h.range.result_definition, vec![llen, rlen])?;
        let available = call(b, &a.sub, vec![llen, rlen])?;
        let matched = if id == "string.contains.ordinal" {
            // Predicate(start) checks the whole needle. The outer finite Any
            // visits precisely length(receiver)-length(needle)+1 starts.
            let l = b.var(15)?;
            let r = b.var(14)?;
            let start = index_word(b)?;
            let count = call(b, &h.length, vec![r])?;
            let delta = call(b, &a.window, vec![l, r, start, count])?;
            let same = call(b, &h.empty.result_definition, vec![delta])?;
            let predicate = b.wrap_selectors(14, same)?;
            let one = word(b, 1)?;
            let count = call(b, &a.add, vec![available, one])?;
            call(b, &name("Any.D14"), vec![predicate, count])?
        } else {
            let start = if id == "string.ends_with.ordinal" {
                available
            } else {
                zero
            };
            let delta = call(b, &a.window, vec![left, right, start, rlen])?;
            call(b, &h.empty.result_definition, vec![delta])?
        };
        // Empty needles match without accessing a 16,385th start position.
        let empty = call(b, &h.empty.result_definition, vec![rlen])?;
        let value = core_mux(b, empty, t, matched)?;
        core_mux(b, too_short, f, value)?
    } else {
        let count = call(b, &a.min, vec![llen, rlen])?;
        let delta = call(b, &a.window, vec![left, right, zero, count])?;
        let same = call(b, &h.empty.result_definition, vec![delta])?;
        if compare {
            let len_delta = call(b, &a.sub, vec![llen, rlen])?;
            let value = mux_word(b, same, len_delta, delta)?;
            let one = word(b, 1)?;
            let minus_one = word(b, u32::MAX)?;
            let present = mux_word(b, rp, value, one)?;
            let absent = mux_word(b, rp, minus_one, zero)?;
            mux_word(b, lp, present, absent)?
        } else {
            let lengths = call(b, &a.equal, vec![llen, rlen])?;
            let same = call(b, "Std.Bool.and", vec![lengths, same])?;
            let both = core_mux(b, rp, same, f)?;
            let neither = call(b, "Std.Bool.not", vec![rp])?;
            let eq = core_mux(b, lp, both, neither)?;
            if id == "string.inequality.operator" {
                call(b, "Std.Bool.not", vec![eq])?
            } else {
                eq
            }
        }
    };
    if failures.len() != signature.ordered_checks.len() {
        return Err(OrdinaryCarrierError::Shape);
    }
    let result = if compare {
        mux_word(b, success, result, zero)?
    } else {
        core_mux(b, success, result, f)?
    };
    let prefix = construct::name(id);
    let result_definition = format!("{prefix}.Result");
    let success_definition = format!("{prefix}.Success");
    let ordered_failure_definitions = (0..failures.len())
        .map(|i| format!("{prefix}.Failure.F{i}"))
        .collect::<Vec<_>>();
    let inputs = [1 << h.depth, 1 << h.depth];
    define(
        b,
        &result_definition,
        &inputs,
        if compare { 5 } else { 0 },
        result,
    )?;
    define(b, &success_definition, &inputs, 0, success)?;
    for (name, value) in ordered_failure_definitions.iter().zip(failures) {
        define(b, name, &inputs, 0, value)?;
    }
    Ok(OrdinaryStringDefinition {
        operation: signature,
        result_definition,
        success_definition,
        ordered_failure_definitions,
    })
}

#[cfg(test)]
mod tests {
    use super::super::super::super::tests::{apply, bit, run, V};
    use super::super::tests::physical;
    use super::*;
    fn signature(id: &str, nullable: bool) -> ClosedOperationSignature {
        let text = if nullable {
            "test.option.string"
        } else {
            STRING_TYPE_ID
        };
        let checks = if matches!(
            id,
            "string.contains.ordinal" | "string.starts_with.ordinal" | "string.ends_with.ordinal"
        ) {
            vec!["exception.null_receiver", "exception.null_argument"]
        } else if id == "string.equals.instance.ordinal" {
            vec!["exception.null_receiver"]
        } else {
            vec![]
        };
        ClosedOperationSignature {
            id: id.into(),
            tag: ClosedOperationTag::Data,
            argument_type_ids: vec![text.into(); 2],
            normal_result_type_id: if id == "string.compare.ordinal" {
                I32_TYPE_ID
            } else {
                BOOL_TYPE_ID
            }
            .into(),
            ordered_checks: checks
                .into_iter()
                .map(|id| {
                    let c = check_contract(id).unwrap();
                    RequiredCheck {
                        id: id.into(),
                        tag: c.tag,
                        failure_type_id: match c.failure {
                            CheckFailureType::None => None,
                            CheckFailureType::Exact(t) => Some(t.into()),
                        },
                    }
                })
                .collect(),
        }
    }
    fn generated(nullable: bool) -> (Certificate, Vec<OrdinaryStringDefinition>, Vec<u8>, usize) {
        let mut b = Builder::new().unwrap();
        let mut h = Helpers::new(&mut b, nullable).unwrap();
        let definitions = OPS
            .iter()
            .map(|id| h.emit(&mut b, signature(id, nullable)).unwrap())
            .collect::<Vec<_>>();
        let transformers = b.static_transformers;
        assert!((8192..=16_384).contains(&transformers));
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(&cert)
            .unwrap();
        (cert, definitions, bytes, transformers)
    }
    fn observe(
        cert: &Certificate,
        d: &OrdinaryStringDefinition,
        a: Option<&[u16]>,
        v: Option<&[u16]>,
        nullable: bool,
    ) {
        let operands = [
            StringOperand::Text {
                utf16: a.map(<[u16]>::to_vec),
            },
            StringOperand::Text {
                utf16: v.map(<[u16]>::to_vec),
            },
        ];
        let expected = evaluate_string_operation(&d.operation.id, &operands, false);
        let args = vec![
            V::Cube(physical(a, nullable)),
            V::Cube(physical(v, nullable)),
        ];
        assert_eq!(
            bit(run(cert, &d.success_definition, args.clone())),
            expected.is_ok(),
            "{} {operands:?}",
            d.operation.id
        );
        for (i, failure) in d.ordered_failure_definitions.iter().enumerate() {
            let expected = matches!(
                (&expected, i),
                (Err(StringError::NullReceiver), 0) | (Err(StringError::NullArgument), 1)
            );
            assert_eq!(
                bit(run(cert, failure, args.clone())),
                expected,
                "{} failure {i}",
                d.operation.id
            );
        }
        let value = run(cert, &d.result_definition, args);
        match expected {
            Ok(MonomorphicValue::Bool {
                value: expected, ..
            }) => assert_eq!(bit(value), expected, "{} {operands:?}", d.operation.id),
            Ok(MonomorphicValue::Signed {
                value: expected, ..
            }) => {
                let expected = expected.parse::<i32>().unwrap() as u32;
                for i in 0..32 {
                    let mut bit_value = value.clone();
                    for j in 0..5 {
                        bit_value = apply(cert, bit_value, V::Bit(i & (1 << j) != 0));
                    }
                    assert_eq!(
                        bit(bit_value),
                        expected & (1 << i) != 0,
                        "{} bit {i} {operands:?}",
                        d.operation.id
                    );
                }
            }
            Err(_) => assert!(!bit(value)),
            other => panic!("unexpected oracle {other:?}"),
        }
    }
    #[test]
    fn string_ordinal_core_matches_utf16_oracle() {
        let texts = [
            None,
            Some(vec![]),
            Some(vec![0]),
            Some(vec![0, 0xd800, 0xffff]),
            Some(vec![0, 0xd800, 0xfffe]),
            Some(vec![0xd800]),
            Some(vec![0xffff, 0, 0xd800, 0xffff]),
        ];
        let mut cases = 0;
        for nullable in [false, true] {
            let (cert, definitions, _, _) = generated(nullable);
            for d in &definitions {
                for a in &texts {
                    for v in &texts {
                        if !nullable && (a.is_none() || v.is_none()) {
                            continue;
                        }
                        observe(&cert, d, a.as_deref(), v.as_deref(), nullable);
                        cases += 1;
                    }
                }
                eprintln!(
                    "ordinal matrix passed: {}, nullable={nullable}",
                    d.operation.id
                );
            }
        }
        assert_eq!(cases, 680);
    }
    #[test]
    fn string_ordinal_core_address_boundaries() {
        let maximum = (0..16384)
            .map(|i| (i as u16).wrapping_add(0x8000))
            .collect::<Vec<_>>();
        for nullable in [false, true] {
            let (cert, definitions, _, _) = generated(nullable);
            for d in &definitions {
                // Compare/search a maximum-size receiver with an early match.
                observe(&cert, d, Some(&maximum), Some(&maximum[..1]), nullable);
            }
            for id in ["string.ends_with.ordinal", "string.starts_with.ordinal"] {
                let d = definitions.iter().find(|d| d.operation.id == id).unwrap();
                observe(&cert, d, Some(&maximum), Some(&maximum[16382..]), nullable);
            }
            let contains = definitions
                .iter()
                .find(|d| d.operation.id == "string.contains.ordinal")
                .unwrap();
            observe(&cert, contains, Some(&maximum), Some(&[]), nullable);
            let compare = definitions
                .iter()
                .find(|d| d.operation.id == "string.compare.ordinal")
                .unwrap();
            for shift in 1..=8 {
                let a = vec![0x8000; (1 << shift) + 1];
                let mut v = a.clone();
                v[1 << shift] = 0x8001;
                observe(&cert, compare, Some(&a), Some(&v), nullable);
            }
            // Earlier low-index differences must dominate later differences
            // even when the latter lies in the other binary half.
            observe(
                &cert,
                compare,
                Some(&[0, 2, 0, 9]),
                Some(&[0, 1, 9, 0]),
                nullable,
            );
        }
    }
    #[test]
    fn string_ordinal_core_fold_boundaries() {
        let (cert, _, _, _) = generated(false);
        let length = |n: usize| V::Cube((0..32).map(|i| n & (1 << i) != 0).collect());
        let mut cases = 0;
        for depth in 0..=4 {
            let capacity = 1 << depth;
            for mark in 0..capacity {
                for count in 0..=capacity + 1 {
                    let mut predicate = vec![false; capacity];
                    predicate[mark] = true;
                    // Cube.D0 is a Bool leaf, without a selector function.
                    let predicate = if depth == 0 {
                        V::Bit(predicate[0])
                    } else {
                        V::Cube(predicate)
                    };
                    assert_eq!(
                        bit(run(
                            &cert,
                            &name(&format!("Any.D{depth}")),
                            vec![predicate, length(count)]
                        )),
                        mark < count
                    );
                    cases += 1;
                }
            }
        }
        assert_eq!(cases, 403);
        // The last address must remain reachable through all fourteen high
        // halves, and count=16,383 must exclude it.
        let mut predicate = vec![false; 16384];
        predicate[16383] = true;
        for count in [16383, 16384] {
            assert_eq!(
                bit(run(
                    &cert,
                    &name("Any.D14"),
                    vec![V::Cube(predicate.clone()), length(count)]
                )),
                count == 16384
            );
        }
    }
    #[test]
    fn string_ordinal_transformer_pair_order_and_terminal_identity() {
        let (cert, _, _, _) = generated(false);
        let word = |n: u32| V::Cube((0..32).map(|i| n & (1 << i) != 0).collect());
        let observe_word = |value: V| -> u32 {
            (0..32).fold(0, |out, i| {
                let mut value = value.clone();
                for j in 0..5 {
                    value = apply(&cert, value, V::Bit(i & (1 << j) != 0));
                }
                out | (u32::from(bit(value)) << i)
            })
        };
        for (first, second, index, length, prior, expected, expected_index) in [
            (0, 9, 0, 1, 0, 0, 2),
            (7, 9, 0, 2, 0, 7, 2),
            (0, 9, 0, 2, 0, 9, 2),
            (0, 0, 0, 2, 0, 0, 2),
            (7, 9, 5, 3, 0, 0, 5),
            (7, 9, 0, 3, 11, 11, 0),
        ] {
            let mut predicate = vec![false; 1 << 19];
            for (i, value) in [first, second].into_iter().enumerate() {
                for j in 0..32 {
                    predicate[i + (j << 14)] = value & (1 << j) != 0;
                }
            }
            let predicate = V::Cube(predicate);
            let state = run(&cert, &name("Make"), vec![word(index), word(prior)]);
            let next = run(
                &cert,
                &name("StepTwo"),
                vec![predicate.clone(), word(length), state],
            );
            for (field, expected) in [("Index", expected_index), ("Value", expected)] {
                assert_eq!(
                    observe_word(run(&cert, &name(field), vec![next.clone()])),
                    expected,
                    "{field}"
                );
            }
            // Every case is now complete by length or first nonzero result.
            let again = run(
                &cert,
                &name("StepTwo"),
                vec![predicate.clone(), word(length), next.clone()],
            );
            for field in ["Index", "Value"] {
                assert_eq!(
                    observe_word(run(&cert, &name(field), vec![again.clone()])),
                    observe_word(run(&cert, &name(field), vec![next.clone()]))
                );
            }
            for (i, expected) in [first, second].into_iter().enumerate() {
                let actual = run(
                    &cert,
                    &name("ReadAt"),
                    vec![predicate.clone(), word(i as u32)],
                );
                assert_eq!(observe_word(actual), expected);
            }
        }
        // Different concrete steps expose reversed composition and a wrong
        // f/g binder even though the production pipeline repeats StepTwo.
        for (first, length, expected, expected_index) in [(0, 4, 9, 4), (7, 4, 7, 2), (0, 2, 0, 2)]
        {
            let predicate = |at: usize, value: u32| {
                let mut bits = vec![false; 1 << 19];
                for j in 0..32 {
                    bits[at + (j << 14)] = value & (1 << j) != 0;
                }
                V::Cube(bits)
            };
            let f = run(
                &cert,
                &name("StepTwo"),
                vec![predicate(0, first), word(length)],
            );
            let g = run(&cert, &name("StepTwo"), vec![predicate(2, 9), word(length)]);
            let state = run(&cert, &name("Make"), vec![word(0), word(0)]);
            let next = run(
                &cert,
                &name("GuardedCompose"),
                vec![word(length), f, g, state],
            );
            for (field, expected) in [("Index", expected_index), ("Value", expected)] {
                assert_eq!(
                    observe_word(run(&cert, &name(field), vec![next.clone()])),
                    expected,
                    "composition {field}"
                );
            }
        }
    }
    #[test]
    fn string_ordinal_core_full_capacity() {
        let (cert, definitions, _, _) = generated(true);
        let maximum = vec![0x1234; 16384];
        let equal = definitions
            .iter()
            .find(|d| d.operation.id == "string.equality.operator")
            .unwrap();
        observe(&cert, equal, Some(&maximum), Some(&maximum), true);
    }
    #[test]
    fn string_ordinal_certificates_are_reproducible() {
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/string-ordinal-circuits");
        let output = std::env::var_os("MPK_W09_STRING_ORDINAL_OUT").map(std::path::PathBuf::from);
        let mut metrics = vec![];
        for nullable in [false, true] {
            let (cert, _, bytes, transformers) = generated(nullable);
            let file = format!("string.ordinal.n{}.hex", u8::from(nullable));
            let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
            if let Some(out) = &output {
                std::fs::create_dir_all(out).unwrap();
                std::fs::write(out.join(&file), hex).unwrap();
            } else {
                assert_eq!(std::fs::read_to_string(directory.join(&file)).unwrap(), hex);
            }
            metrics.push(serde_json::json!({"nullable":nullable,"static_transformers":transformers,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))}));
        }
        if let Some(out) = output {
            std::fs::write(
                out.join("core-metrics.json"),
                serde_json::to_vec_pretty(&metrics).unwrap(),
            )
            .unwrap();
        } else {
            let expected: Value = serde_json::from_slice(
                &std::fs::read(directory.join("core-metrics.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(expected, serde_json::json!(metrics));
        }
    }
}
