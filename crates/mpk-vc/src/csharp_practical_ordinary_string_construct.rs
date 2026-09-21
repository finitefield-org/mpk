//! Bounded string construction by ordinary address selection, without host text.
use super::*;
pub(super) fn supports(id: &str) -> bool {
    matches!(
        id,
        "string.substring.start_length"
            | "string.concat.string2"
            | "string.concat.string3"
            | "string.concat.string4"
            | "string.concat.operator.string_string"
            | "string.concat.operator.string_char"
            | "string.concat.operator.char_string"
    ) || id.starts_with("string.interpolation.restricted.")
}
pub(super) fn word_operation(
    b: &mut Builder,
    id: &str,
    c: Circuit,
    output: Word,
) -> R<OrdinaryScalarDefinition> {
    let n = c.inputs.len();
    emit_circuit(
        b,
        IntegerCircuit {
            signature: ClosedOperationSignature {
                id: id.into(),
                tag: ClosedOperationTag::Data,
                argument_type_ids: vec![I32_TYPE_ID.into(); n],
                normal_result_type_id: I32_TYPE_ID.into(),
                ordered_checks: vec![],
            },
            circuit: c,
            output,
            failures: vec![],
        },
        "StringConstruct",
    )
}
pub(super) struct Aux {
    add: OrdinaryScalarDefinition,
    sub: OrdinaryScalarDefinition,
    range: OrdinaryScalarDefinition,
    bound: OrdinaryScalarDefinition,
    zero: String,
    one: String,
}
pub(super) fn name(id: &str) -> String {
    format!(
        "{PREFIX}.String.O{}",
        id.bytes().map(|x| format!("{x:02x}")).collect::<String>()
    )
}
impl Aux {
    pub(super) fn new(b: &mut Builder) -> R<Self> {
        for depth in [5, 19] {
            if !b
                .globals
                .contains_key(&format!("{PREFIX}.Cube.D{depth}.Mux"))
            {
                b.helpers(depth)?;
            }
        }
        let mut c = Circuit::new(&[32, 32]);
        let a = c.inputs[0].clone();
        let v = c.inputs[1].clone();
        let sum = c.add(&a, &v, F).0;
        let add = word_operation(b, "String.Add32", c, sum)?;
        let mut c = Circuit::new(&[32, 32]);
        let a = c.inputs[0].clone();
        let v = c.inputs[1].clone();
        let delta = c.sub(&a, &v).0;
        let sub = word_operation(b, "String.Sub32", c, delta)?;
        let mut c = Circuit::new(&[32, 32, 32]);
        let start = c.inputs[0].clone();
        let len = c.inputs[1].clone();
        let size = c.inputs[2].clone();
        let start_gt = c.lt(&size, &start, false);
        let start_ok = c.not(start_gt);
        let available = c.sub(&size, &start).0;
        let len_gt = c.lt(&available, &len, false);
        let len_ok = c.not(len_gt);
        let negative = c.or(start[31], len[31]);
        let positive = c.not(negative);
        let valid = c.and(start_ok, len_ok);
        let valid = c.and(positive, valid);
        let range = small_check(b, "String.SubstringRange", c, valid)?;
        let mut c = Circuit::new(&[32]);
        let len = c.inputs[0].clone();
        let maximum = (0..32)
            .map(|i| if 16384u32 & (1 << i) != 0 { T } else { F })
            .collect::<Word>();
        let large = c.lt(&maximum, &len, false);
        let bound = c.not(large);
        let bound = small_check(b, "String.OutputBound", c, bound)?;
        let zero = format!("{PREFIX}.Cube.D5.Zero");
        let one = format!("{PREFIX}.String.OneLength");
        let f = b.constant("Std.Bool.false")?;
        let t = b.constant("Std.Bool.true")?;
        let mut bits = vec![f; 32];
        bits[0] = t;
        let body = core_select(b, &bits, 5, 0, 5, f)?;
        let body = b.wrap_selectors(5, body)?;
        define(b, &one, &[], 5, body)?;
        Ok(Self {
            add,
            sub,
            range,
            bound,
            zero,
            one,
        })
    }
    fn length(&self, b: &mut Builder, h: &Helpers, arg: u32, char_: bool) -> R<u32> {
        if char_ {
            return b.constant(&self.one);
        }
        let present = call(b, &h.present, vec![arg])?;
        let length = call(b, &h.length, vec![arg])?;
        let zero = b.constant(&self.zero)?;
        call(
            b,
            &format!("{PREFIX}.Cube.D5.Mux"),
            vec![present, length, zero],
        )
    }
}
fn args(b: &mut Builder, n: usize, extra: u32) -> R<Vec<u32>> {
    (0..n).rev().map(|i| b.var(extra + i as u32)).collect()
}
// At the fifteen-selector prefix (role plus element index), bind the index
// word before the four UTF-16 bit selectors. Its five internal selectors
// capture exactly the fourteen index bits.
fn output_index(b: &mut Builder) -> R<u32> {
    let f = b.constant("Std.Bool.false")?;
    let mut bits = vec![f; 32];
    for (i, bit) in bits.iter_mut().enumerate().take(14) {
        *bit = b.var(18 - i as u32)?;
    }
    let value = core_select(b, &bits, 5, 0, 5, f)?;
    b.wrap_selectors(5, value)
}
pub(super) fn emit(
    b: &mut Builder,
    h: &Helpers,
    aux: &Aux,
    signature: ClosedOperationSignature,
) -> R<OrdinaryStringDefinition> {
    let n = signature.argument_type_ids.len();
    // Every argument contributes a binder. More precise composed binder/term
    // bounds are measured by Builder; this prevents oversized temporary work.
    if n > crate::csharp_practical_vc_model::BINDER_DEPTH_MAX as usize {
        return Err(OrdinaryCarrierError::Limit);
    }
    let substring = signature.id == "string.substring.start_length";
    let chars = signature
        .argument_type_ids
        .iter()
        .map(|id| id == "mpk.csharp.value.char.v1")
        .collect::<Vec<_>>();
    let inputs = signature
        .argument_type_ids
        .iter()
        .map(|id| {
            if id == I32_TYPE_ID {
                32
            } else if id == "mpk.csharp.value.char.v1" {
                16
            } else {
                1 << h.depth
            }
        })
        .collect::<Vec<_>>();
    let prefix = name(&signature.id);
    let total_name = format!("{prefix}.TotalLength");
    let outer = args(b, n, 0)?;
    let mut total = b.constant(&aux.zero)?;
    if substring {
        total = outer[2];
    } else {
        for (&arg, &char_) in outer.iter().zip(&chars) {
            let len = aux.length(b, h, arg, char_)?;
            total = call(b, &aux.add.result_definition, vec![total, len])?;
        }
    }
    define(b, &total_name, &inputs, 5, total)?;

    let value_name = format!("{prefix}.Value");
    let f = b.constant("Std.Bool.false")?;
    let t = b.constant("Std.Bool.true")?;
    let inner = args(b, n, 22)?; // nineteen selectors plus length/index/range lets.
    let len = b.var(21)?;
    let index = b.var(5)?;
    let inside = b.var(4)?;
    let mut content = f;
    if substring {
        let source_index = call(b, &aux.add.result_definition, vec![inner[1], index])?;
        let unit = call(b, &h.read, vec![inner[0], source_index])?;
        let selectors = b.selectors(4)?;
        content = b.app(unit, selectors)?;
    } else {
        let mut start = b.constant(&aux.zero)?;
        let mut choices = vec![];
        for (&arg, &char_) in inner.iter().zip(&chars) {
            let len = aux.length(b, h, arg, char_)?;
            let end = call(b, &aux.add.result_definition, vec![start, len])?;
            let select = call(b, &h.range.result_definition, vec![index, end])?;
            let unit = if char_ {
                arg
            } else {
                let local = call(b, &aux.sub.result_definition, vec![index, start])?;
                call(b, &h.read, vec![arg, local])?
            };
            let selectors = b.selectors(4)?;
            let leaf = b.app(unit, selectors)?;
            choices.push((select, leaf));
            start = end;
        }
        for (select, leaf) in choices.into_iter().rev() {
            content = core_mux(b, select, leaf, content)?;
        }
    }
    content = core_mux(b, inside, content, f)?;
    // The fifth header bit precedes the two per-index lets.
    let selectors = [6, 3, 2, 1, 0]
        .into_iter()
        .map(|i| b.var(i))
        .collect::<R<Vec<_>>>()?;
    let length_bit = b.app(len, selectors)?;
    let mut padding = f;
    for i in 1..14 {
        let bit = b.var(20 - i)?;
        padding = call(b, "Std.Bool.or", vec![padding, bit])?;
    }
    let header = core_mux(b, padding, f, length_bit)?;
    let role = b.var(20)?;
    let leaf = core_mux(b, role, content, header)?;
    let body = b.wrap_selectors(4, leaf)?;
    let prefix_length = b.var(16)?;
    // Compare the address bits directly, rather than projecting a synthesized
    // word through a scalar arithmetic circuit at every physical position.
    // Each more significant bit overrides the lower comparison on inequality.
    // Address bits above 13 are zero; retain all 32 length bits so Value also
    // has the same meaning before the output-bound guard is applied.
    let mut in_range = f;
    for i in 0..32 {
        let length_bit = core_read(b, prefix_length, i, 5)?;
        if i < 14 {
            let address_bit = b.var(14 - i as u32)?;
            let when_one = call(b, "Std.Bool.and", vec![length_bit, in_range])?;
            let when_zero = call(b, "Std.Bool.or", vec![length_bit, in_range])?;
            in_range = core_mux(b, address_bit, when_one, when_zero)?;
        } else {
            in_range = call(b, "Std.Bool.or", vec![length_bit, in_range])?;
        }
    }
    let body = b.term(TermNode::Let {
        ty: b.boolean,
        value: in_range,
        body,
    })?;
    let index_word = output_index(b)?;
    let word_ty = b.cube(5)?;
    let body = b.term(TermNode::Let {
        ty: word_ty,
        value: index_word,
        body,
    })?;
    let body = b.wrap_selectors(15, body)?;
    let total = call(b, &total_name, outer.clone())?;
    let word_ty = b.cube(5)?;
    let body = b.term(TermNode::Let {
        ty: word_ty,
        value: total,
        body,
    })?;
    define(b, &value_name, &inputs, 19, body)?;

    let bounded = call(b, &aux.bound.result_definition, vec![total])?;
    let (success, failures) = if substring {
        let present = call(b, &h.present, vec![outer[0]])?;
        let length = call(b, &h.length, vec![outer[0]])?;
        let range = call(
            b,
            &aux.range.result_definition,
            vec![outer[1], outer[2], length],
        )?;
        let normal = call(b, "Std.Bool.and", vec![present, range])?;
        let success = call(b, "Std.Bool.and", vec![normal, bounded])?;
        let null = call(b, "Std.Bool.not", vec![present])?;
        let not_range = call(b, "Std.Bool.not", vec![range])?;
        let range_failure = call(b, "Std.Bool.and", vec![present, not_range])?;
        let not_bound = call(b, "Std.Bool.not", vec![bounded])?;
        let bound_failure = call(b, "Std.Bool.and", vec![normal, not_bound])?;
        (success, vec![null, range_failure, bound_failure])
    } else {
        let failure = core_mux(b, bounded, f, t)?;
        (bounded, vec![failure])
    };
    if failures.len() != signature.ordered_checks.len() {
        return Err(OrdinaryCarrierError::Shape);
    }
    // Preserve the shared four-bit unit beneath the success guard. A pointwise
    // C19 mux would reapply Value's whole index prefix for each output bit and
    // lose the per-index lets above as intermediate closures are released.
    let unit = b.var(4)?;
    let selectors = b.selectors(4)?;
    let leaf = b.app(unit, selectors)?;
    let guard = b.var(21)?;
    let leaf = core_mux(b, guard, leaf, f)?;
    let result = b.wrap_selectors(4, leaf)?;
    let value = b.var(15)?;
    let selectors = b.selectors(15)?;
    let unit = b.app(value, selectors)?;
    let unit_ty = b.cube(4)?;
    let result = b.term(TermNode::Let {
        ty: unit_ty,
        value: unit,
        body: result,
    })?;
    let result = b.wrap_selectors(15, result)?;
    let value_args = args(b, n, 1)?;
    let value = call(b, &value_name, value_args)?;
    let text_ty = b.cube(19)?;
    let result = b.term(TermNode::Let {
        ty: text_ty,
        value,
        body: result,
    })?;
    let result = b.term(TermNode::Let {
        ty: b.boolean,
        value: success,
        body: result,
    })?;
    let result_definition = format!("{prefix}.Result");
    let success_definition = format!("{prefix}.Success");
    define(b, &result_definition, &inputs, 19, result)?;
    define(b, &success_definition, &inputs, 0, success)?;
    let ordered_failure_definitions = (0..failures.len())
        .map(|i| format!("{prefix}.Failure.F{i}"))
        .collect::<Vec<_>>();
    for (name, failure) in ordered_failure_definitions.iter().zip(failures) {
        define(b, name, &inputs, 0, failure)?;
    }
    Ok(OrdinaryStringDefinition {
        operation: signature,
        result_definition,
        success_definition,
        ordered_failure_definitions,
    })
}

/// W03 stores each failed condition independently; ordered failures stay intact.
pub(super) fn emit_raw_substring_checks(
    b: &mut Builder,
    h: &Helpers,
    d: &OrdinaryStringDefinition,
) -> R<[String; 2]> {
    if d.operation.argument_type_ids.len() != 3 || d.ordered_failure_definitions.len() != 3 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let aux = h.construct.as_ref().ok_or(OrdinaryCarrierError::Shape)?;
    let arguments = args(b, 3, 0)?;
    let length = call(b, &h.length, vec![arguments[0]])?;
    let range = call(
        b,
        &aux.range.result_definition,
        vec![arguments[1], arguments[2], length],
    )?;
    let bound = call(b, &aux.bound.result_definition, vec![arguments[2]])?;
    let names = [
        format!("{}.NativeRaw.F1", d.result_definition),
        format!("{}.NativeRaw.F2", d.result_definition),
    ];
    for (name, condition) in names.iter().zip([range, bound]) {
        let failed = call(b, "Std.Bool.not", vec![condition])?;
        define(b, name, &[1 << h.depth, 32, 32], 0, failed)?;
    }
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::tests::{apply, bit, run, V};
    use super::super::tests::physical;
    use super::*;
    fn text(s: &[u16]) -> StringOperand {
        StringOperand::Text {
            utf16: Some(s.to_vec()),
        }
    }
    fn index(value: i32) -> StringOperand {
        StringOperand::Index { value }
    }
    fn signature(id: &str, operands: &[StringOperand], nullable: bool) -> ClosedOperationSignature {
        let checks = if id == "string.substring.start_length" {
            vec![
                "exception.null_receiver",
                "exception.range",
                "obligation.output_bound",
            ]
        } else {
            vec!["obligation.output_bound"]
        };
        ClosedOperationSignature {
            id: id.into(),
            tag: ClosedOperationTag::Data,
            argument_type_ids: operands
                .iter()
                .map(|x| {
                    match x {
                        StringOperand::Text { .. } => {
                            if nullable {
                                "test.option.string"
                            } else {
                                STRING_TYPE_ID
                            }
                        }
                        StringOperand::Char { .. } => "mpk.csharp.value.char.v1",
                        StringOperand::Index { .. } => I32_TYPE_ID,
                    }
                    .into()
                })
                .collect(),
            normal_result_type_id: STRING_TYPE_ID.into(),
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
    fn input(v: &StringOperand, nullable: bool) -> V {
        V::Cube(match v {
            StringOperand::Text { utf16 } => physical(utf16.as_deref(), nullable),
            StringOperand::Index { value } => {
                (0..32).map(|i| (*value as u32) & (1 << i) != 0).collect()
            }
            StringOperand::Char { utf16 } => (0..16).map(|i| utf16 & (1 << i) != 0).collect(),
        })
    }
    fn leaf(c: &Certificate, value: &V, address: u32) -> bool {
        let mut v = value.clone();
        for j in 0..19 {
            v = apply(c, v, V::Bit(address & (1 << j) != 0));
        }
        bit(v)
    }
    #[test]
    fn string_construction_shares_index_and_range_per_code_unit() {
        use super::super::super::super::test_eval::apply_counted;
        let xs = vec![
            StringOperand::Char { utf16: 0xd800 },
            text(&[97, 0xd800, 0]),
        ];
        let id = "string.interpolation.restricted.cs";
        let mut b = Builder::new().unwrap();
        let mut h = Helpers::new(&mut b, false).unwrap();
        let d = h.emit(&mut b, signature(id, &xs, false)).unwrap();
        let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        let value = run(
            &c,
            &d.result_definition,
            xs.iter().map(|x| input(x, false)).collect(),
        );
        for index in [0usize, 1, 2, 3, 4, 8192, 16383] {
            let mut word = apply(&c, value.clone(), V::Bit(true));
            for bit in 0..14 {
                word = apply(&c, word, V::Bit(index & (1 << bit) != 0));
            }
            let expected = [0xd800u16, 97, 0xd800, 0].get(index).copied().unwrap_or(0);
            let mut costs = [0u64; 16];
            for (selector, cost) in costs.iter_mut().enumerate() {
                let mut actual = word.clone();
                for bit_index in 0..4 {
                    let (next, steps) =
                        apply_counted(&c, actual, V::Bit(selector & (1 << bit_index) != 0));
                    actual = next;
                    *cost += steps;
                }
                assert_eq!(
                    bit(actual),
                    expected & (1 << selector) != 0,
                    "index {index} bit {selector}"
                );
            }
            eprintln!(
                "shared UTF-16 index {index}: first {} transitions, next fifteen {}",
                costs[0],
                costs[1..].iter().sum::<u64>()
            );
            if index >= 4 {
                assert!(
                    costs[1..].iter().all(|&cost| cost < costs[0] && cost < 200),
                    "range check was recomputed for each bit: {costs:?}"
                );
            }
        }
    }

    #[test]
    fn string_construction_core_matches_utf16_oracle() {
        let null = StringOperand::Text { utf16: None };
        let a = text(&[0, 0xd800, 65, 0xdc00, 0xffff]);
        let maximum = text(
            &(0..16384)
                .map(|i| (i as u16).wrapping_mul(4051))
                .collect::<Vec<_>>(),
        );
        let cases = vec![
            (
                "string.substring.start_length",
                vec![a.clone(), index(1), index(3)],
            ),
            (
                "string.substring.start_length",
                vec![a.clone(), index(5), index(0)],
            ),
            (
                "string.substring.start_length",
                vec![a.clone(), index(6), index(0)],
            ),
            (
                "string.substring.start_length",
                vec![a.clone(), index(-1), index(0)],
            ),
            (
                "string.substring.start_length",
                vec![a.clone(), index(0), index(-1)],
            ),
            (
                "string.substring.start_length",
                vec![a.clone(), index(i32::MAX), index(1)],
            ),
            (
                "string.substring.start_length",
                vec![a.clone(), index(1), index(i32::MAX)],
            ),
            (
                "string.substring.start_length",
                vec![null.clone(), index(-1), index(-1)],
            ),
            (
                "string.substring.start_length",
                vec![maximum.clone(), index(8191), index(8193)],
            ),
            ("string.concat.string2", vec![a.clone(), a.clone()]),
            ("string.concat.string2", vec![maximum.clone(), text(&[])]),
            ("string.concat.string2", vec![maximum.clone(), text(&[1])]),
            ("string.concat.string2", vec![null.clone(), a.clone()]),
            (
                "string.concat.string3",
                vec![a.clone(), text(&[]), a.clone()],
            ),
            (
                "string.concat.string3",
                vec![null.clone(), null.clone(), null.clone()],
            ),
            (
                "string.concat.string4",
                vec![a.clone(), text(&[0]), text(&[]), a.clone()],
            ),
            (
                "string.concat.operator.string_string",
                vec![a.clone(), text(&[])],
            ),
            (
                "string.concat.operator.char_string",
                vec![StringOperand::Char { utf16: 0xdfff }, a.clone()],
            ),
            (
                "string.concat.operator.string_char",
                vec![a.clone(), StringOperand::Char { utf16: 0 }],
            ),
            (
                "string.interpolation.restricted.scs",
                vec![
                    a.clone(),
                    StringOperand::Char { utf16: 0xd800 },
                    null.clone(),
                ],
            ),
            (
                "string.interpolation.restricted.cc",
                vec![
                    StringOperand::Char { utf16: 0 },
                    StringOperand::Char { utf16: 0xffff },
                ],
            ),
            (
                "string.interpolation.restricted.sc",
                vec![a.clone(), StringOperand::Char { utf16: 0xd800 }],
            ),
            (
                "string.interpolation.restricted.sscs",
                vec![
                    a.clone(),
                    text(&[]),
                    StringOperand::Char { utf16: 0xffff },
                    a.clone(),
                ],
            ),
            ("string.interpolation.restricted.empty", vec![]),
        ];
        let mut count = 0;
        for nullable in [false, true] {
            let cases = cases
                .iter()
                .filter(|(_, xs)| {
                    nullable
                        || !xs
                            .iter()
                            .any(|v| matches!(v, StringOperand::Text { utf16: None }))
                })
                .collect::<Vec<_>>();
            let mut b = Builder::new().unwrap();
            let mut h = Helpers::new(&mut b, nullable).unwrap();
            let mut definitions = BTreeMap::new();
            for (id, xs) in &cases {
                if !definitions.contains_key(*id) {
                    definitions.insert(*id, h.emit(&mut b, signature(id, xs, nullable)).unwrap());
                }
            }
            let bytes = b.finish().unwrap();
            let cert = decode_canonical_certificate(&bytes).unwrap();
            crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(
                &cert,
            )
            .unwrap();
            for (id, xs) in cases {
                let d = &definitions[id];
                let args = xs.iter().map(|v| input(v, nullable)).collect::<Vec<_>>();
                let expected = evaluate_string_operation(id, xs, false);
                assert_eq!(
                    bit(run(&cert, &d.success_definition, args.clone())),
                    expected.is_ok(),
                    "{id} {nullable}"
                );
                for (i, name) in d.ordered_failure_definitions.iter().enumerate() {
                    let flag = match &expected {
                        Err(StringError::NullReceiver) => i == 0,
                        Err(StringError::ArgumentOutOfRange) => i == 1,
                        Err(StringError::OutputBound) => {
                            i == if *id == "string.substring.start_length" {
                                2
                            } else {
                                0
                            }
                        }
                        _ => false,
                    };
                    assert_eq!(
                        bit(run(&cert, name, args.clone())),
                        flag,
                        "{id} failure {i}"
                    );
                }
                let value = run(&cert, &d.result_definition, args);
                let text = match expected {
                    Ok(MonomorphicValue::String { utf16, .. }) => utf16,
                    Err(_) => vec![],
                    other => panic!("{other:?}"),
                };
                for i in 0..32 {
                    assert_eq!(
                        leaf(&cert, &value, i << 14),
                        text.len() & (1 << i) != 0,
                        "{id} length {i}"
                    );
                }
                for padding in 1..14 {
                    assert!(!leaf(&cert, &value, 1 << padding), "{id} header padding");
                }
                let mut cells = BTreeSet::from([0, 1, 16383]);
                cells.extend((0..text.len().min(32)).map(|i| i as u32));
                for i in [text.len().saturating_sub(1), text.len(), text.len() + 1] {
                    if i < 16384 {
                        cells.insert(i as u32);
                    }
                }
                // Probe both sides of every binary carry in the dynamic address.
                for shift in 1..14 {
                    for i in [(1 << shift) - 1, 1 << shift, (1 << shift) + 1] {
                        cells.insert(i);
                    }
                }
                for i in cells {
                    for bit in 0..16 {
                        let expected = text.get(i as usize).is_some_and(|x| x & (1 << bit) != 0);
                        assert_eq!(
                            leaf(&cert, &value, 1 + (i << 1) + (bit << 15)),
                            expected,
                            "{id} cell {i} bit {bit}"
                        );
                    }
                }
                count += 1;
                eprintln!(
                    "string construction case passed: {id}, nullable={nullable}, length={}",
                    text.len()
                );
            }
        }
        assert_eq!(count, 44);
    }
    #[test]
    fn string_construction_rejects_oversized_binders() {
        // The index word is now outside the last four selectors. N arguments,
        // three shared lets and nineteen selectors reach the binder cap.
        let cap = crate::csharp_practical_vc_model::BINDER_DEPTH_MAX as usize;
        let largest = cap - 22;
        for n in [65, largest, largest + 1, cap + 1] {
            let xs = vec![StringOperand::Char { utf16: 0 }; n];
            let id = format!("string.interpolation.restricted.{}", "c".repeat(n));
            let mut b = Builder::new().unwrap();
            let mut h = Helpers::new(&mut b, false).unwrap();
            let result = h.emit(&mut b, signature(&id, &xs, false));
            if n <= largest {
                result.unwrap();
                let bytes = b.finish().unwrap();
                let cert = decode_canonical_certificate(&bytes).unwrap();
                crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(
                    &cert,
                )
                .unwrap();
                if n == largest {
                    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../develop/migrations/csharp-03/ordinary-foundation/string-construction-circuits");
                    let out = std::env::var_os("MPK_W09_STRING_CONSTRUCT_OUT")
                        .map(std::path::PathBuf::from);
                    let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
                    let metrics = serde_json::json!({"arguments":n,"binder_cap":cap,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))});
                    if let Some(out) = out {
                        std::fs::create_dir_all(&out).unwrap();
                        std::fs::write(out.join("string.interpolation.binder_limit.hex"), &hex)
                            .unwrap();
                        std::fs::write(
                            out.join("binder-limit.json"),
                            serde_json::to_vec_pretty(&metrics).unwrap(),
                        )
                        .unwrap();
                    } else {
                        assert_eq!(
                            std::fs::read_to_string(
                                directory.join("string.interpolation.binder_limit.hex")
                            )
                            .unwrap(),
                            hex
                        );
                        let expected: Value = serde_json::from_slice(
                            &std::fs::read(directory.join("binder-limit.json")).unwrap(),
                        )
                        .unwrap();
                        assert_eq!(expected, metrics);
                    }
                }
            } else {
                assert_eq!(result, Err(OrdinaryCarrierError::Limit), "{n}");
            }
        }
    }
}
