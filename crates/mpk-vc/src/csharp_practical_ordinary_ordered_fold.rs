//! Shared ordered finite folds for structural/collection relations. Every
//! pipeline leaf is a concrete C6 -> C6 transformer. No relation is proved here.
use super::*;
const STATE: u32 = 6;
const MAX_INDEX_BITS: u32 = 14;
const PREDICATE: u32 = MAX_INDEX_BITS + 5;
fn name(s: &str) -> String {
    format!("{PREFIX}.OrderedFold.{s}")
}
fn and(b: &mut Builder, x: u32, y: u32) -> R<u32> {
    let f = bit(b, false)?;
    mux(b, x, y, f)
}
fn not(b: &mut Builder, x: u32) -> R<u32> {
    let f = bit(b, false)?;
    let t = bit(b, true)?;
    mux(b, x, f, t)
}
fn read_bit(b: &mut Builder, value: u32, index: u32) -> R<u32> {
    let address = prefix(5, index)
        .into_iter()
        .map(|v| bit(b, v))
        .collect::<R<Vec<_>>>()?;
    b.app(value, address)
}
pub(super) fn word(b: &mut Builder, value: u32) -> R<u32> {
    let mut result = bit(b, false)?;
    for i in 0..32 {
        if value & (1 << i) != 0 {
            let at = equal_address(b, 5, 0, 5, i)?;
            let t = bit(b, true)?;
            result = mux(b, at, t, result)?;
        }
    }
    b.wrap_selectors(5, result)
}
fn make_word(b: &mut Builder, values: &[u32]) -> R<u32> {
    if values.len() != 32 {
        return Err(OrdinaryCarrierError::Shape);
    }
    // Values are built under five selector binders. Select their LSB-first address.
    let mut out = bit(b, false)?;
    for (i, &value) in values.iter().enumerate().rev() {
        let at = equal_address(b, 5, 0, 5, i as u32)?;
        out = mux(b, at, value, out)?;
    }
    b.wrap_selectors(5, out)
}
fn helper(b: &mut Builder, n: &str, args: Vec<u32>) -> R<u32> {
    call(b, &name(n), args)
}
fn cube_mux(b: &mut Builder, d: u32, active: u32, yes: u32, no: u32) -> R<u32> {
    call(b, &format!("{PREFIX}.Cube.D{d}.Mux"), vec![active, yes, no])
}
fn auxiliary(b: &mut Builder) -> R<()> {
    for d in [5, STATE] {
        if !b.globals.contains_key(&format!("{PREFIX}.Cube.D{d}.Mux")) {
            b.helpers(d)?;
        }
    }
    // All word operations are closed finite bit circuits, not host-computed answers.
    let x = b.var(0)?;
    let mut empty = bit(b, true)?;
    for i in 0..32 {
        let v = read_bit(b, x, i)?;
        let z = not(b, v)?;
        empty = and(b, empty, z)?;
    }
    define(b, &name("Empty"), &[5], 0, empty)?;
    let left = b.var(1)?;
    let right = b.var(0)?;
    let mut less = bit(b, false)?;
    for i in 0..32 {
        let a = read_bit(b, left, i)?;
        let v = read_bit(b, right, i)?;
        let f = bit(b, false)?;
        let not_v = not(b, v)?;
        let same = mux(b, a, v, not_v)?;
        let different = mux(b, a, f, v)?;
        // Refer to the lower-bit result once. Duplicating it in both branches
        // makes ordinary syntactic dependency traversal exponential in width.
        less = mux(b, same, less, different)?;
    }
    define(b, &name("Less"), &[5, 5], 0, less)?;
    let left = b.var(1)?;
    let right = b.var(0)?;
    let less = helper(b, "Less", vec![left, right])?;
    let minimum = cube_mux(b, 5, less, left, right)?;
    define(b, &name("Min"), &[5, 5], 5, minimum)?;
    for delta in [1u32, 2] {
        let x = b.var(5)?;
        let mut carry = bit(b, false)?;
        let mut values = vec![];
        for i in 0..32 {
            let a = read_bit(b, x, i)?;
            let inv = not(b, a)?;
            let mut sum = mux(b, carry, inv, a)?;
            let mut next = and(b, a, carry)?;
            if delta & (1 << i) != 0 {
                sum = not(b, sum)?;
                let t = bit(b, true)?;
                next = mux(b, a, t, carry)?;
            }
            values.push(sum);
            carry = next;
        }
        let body = make_word(b, &values)?;
        define(b, &name(&format!("Add{delta}")), &[5], 5, body)?;
    }
    for (field, role) in [("Index", false), ("Value", true)] {
        let x = b.var(5)?;
        let bit = bit(b, role)?;
        let mut args = vec![bit];
        args.extend(b.selectors(5)?);
        let body = b.app(x, args)?;
        let body = b.wrap_selectors(5, body)?;
        define(b, &name(field), &[STATE], 5, body)?;
    }
    let index = b.var(7)?;
    let value = b.var(6)?;
    let selectors = b.selectors(5)?;
    let index = b.app(index, selectors.clone())?;
    let value = b.app(value, selectors)?;
    let role = b.var(5)?;
    let body = mux(b, role, value, index)?;
    let body = b.wrap_selectors(STATE, body)?;
    define(b, &name("Make"), &[5, 5], STATE, body)?;
    let length = b.var(1)?;
    let state = b.var(0)?;
    let index = helper(b, "Index", vec![state])?;
    let value = helper(b, "Value", vec![state])?;
    let within = helper(b, "Less", vec![index, length])?;
    let empty = helper(b, "Empty", vec![value])?;
    let active = and(b, within, empty)?;
    define(b, &name("Active"), &[5, STATE], 0, active)?;
    Ok(())
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryOrderedFoldDefinition {
    pub index_bits: u32,
    pub capacity: u32,
    /// First nonzero i32 word in index order, or zero; count is clamped to capacity.
    pub first_definition: String,
    pub any_definition: String,
    pub all_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryOrderedFoldProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryOrderedFoldDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryOrderedFoldProgram {
    pub fn definitions(&self) -> &[OrdinaryOrderedFoldDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed ordered fold program")
    }
}
fn pipeline(b: &mut Builder) -> R<()> {
    if b.globals.contains_key(&name("Pipeline")) {
        return Ok(());
    }
    auxiliary(b)?;
    let predicate = b.var(6)?;
    let index = b.var(5)?;
    let mut args = (0..MAX_INDEX_BITS)
        .map(|i| read_bit(b, index, i))
        .collect::<R<Vec<_>>>()?;
    args.extend(b.selectors(5)?);
    let body = b.app(predicate, args)?;
    let body = b.wrap_selectors(5, body)?;
    define(b, &name("ReadAt"), &[PREDICATE, 5], 5, body)?;
    let predicate = b.var(2)?;
    let length = b.var(1)?;
    let state = b.var(0)?;
    let index = helper(b, "Index", vec![state])?;
    let next = helper(b, "Add1", vec![index])?;
    let after = helper(b, "Add2", vec![index])?;
    let first = helper(b, "ReadAt", vec![predicate, index])?;
    let second = helper(b, "ReadAt", vec![predicate, next])?;
    let within = helper(b, "Less", vec![next, length])?;
    let zero = word(b, 0)?;
    let second = cube_mux(b, 5, within, second, zero)?;
    let empty = helper(b, "Empty", vec![first])?;
    let value = cube_mux(b, 5, empty, second, first)?;
    let updated = helper(b, "Make", vec![after, value])?;
    let active = helper(b, "Active", vec![length, state])?;
    let body = cube_mux(b, STATE, active, updated, state)?;
    define(b, &name("StepTwo"), &[PREDICATE, 5, STATE], STATE, body)?;
    // f then g, with an ordinary conditional at the intermediate state.
    let state_ty = b.cube(STATE)?;
    let transformer = b.pi(state_ty, state_ty)?;
    let word_ty = b.cube(5)?;
    let f = b.var(2)?;
    let state = b.var(0)?;
    let first = b.app(f, vec![state])?;
    let length = b.var(4)?;
    let g = b.var(2)?;
    let intermediate = b.var(0)?;
    let next = b.app(g, vec![intermediate])?;
    let active = helper(b, "Active", vec![length, intermediate])?;
    let body = cube_mux(b, STATE, active, next, intermediate)?;
    let body = b.term(TermNode::Let {
        ty: state_ty,
        value: first,
        body,
    })?;
    let body = b.lam(state_ty, body)?;
    let body = b.lam(transformer, body)?;
    let body = b.lam(transformer, body)?;
    let body = b.lam(word_ty, body)?;
    let ty = b.pi(transformer, transformer)?;
    let ty = b.pi(transformer, ty)?;
    let ty = b.pi(word_ty, ty)?;
    b.define(&name("Compose"), ty, body)?;
    let predicate = b.var(1)?;
    let length = b.var(0)?;
    let step = helper(b, "StepTwo", vec![predicate, length])?;
    let composition = helper(b, "Compose", vec![length])?;
    let value = b.compose_term(STATE, composition, &vec![step; 8192])?;
    let body = b.lam(word_ty, value)?;
    let predicate_ty = b.cube(PREDICATE)?;
    let body = b.lam(predicate_ty, body)?;
    let ty = b.pi(word_ty, transformer)?;
    let ty = b.pi(predicate_ty, ty)?;
    b.define(&name("Pipeline"), ty, body)
}
/// Private ordinary-code dependency used by subsequent concrete relations.
/// All widths share one maximum-bound pipeline; repeated imports do not expand it again.
pub(super) fn emit_fold(b: &mut Builder, depth: u32) -> R<OrdinaryOrderedFoldDefinition> {
    if depth > MAX_INDEX_BITS {
        return Err(OrdinaryCarrierError::Limit);
    }
    let first_definition = name(&format!("First.D{depth}"));
    let any_definition = name(&format!("Any.D{depth}"));
    let all_definition = name(&format!("All.D{depth}"));
    let definition = OrdinaryOrderedFoldDefinition {
        index_bits: depth,
        capacity: 1 << depth,
        first_definition,
        any_definition,
        all_definition,
    };
    if b.globals.contains_key(&definition.first_definition) {
        return Ok(definition);
    }
    pipeline(b)?;
    // Embedding repeats the smaller predicate outside its range; clamping count
    // prevents any repeated high-index part from becoming observable.
    let input = b.var(PREDICATE + 1)?;
    let mut args = (0..depth)
        .map(|i| b.var(PREDICATE - 1 - i))
        .collect::<R<Vec<_>>>()?;
    args.extend(b.selectors(5)?);
    let body = b.app(input, args)?;
    let predicate = b.wrap_selectors(PREDICATE, body)?;
    let count = b.var(0)?;
    let cap = word(b, 1 << depth)?;
    let count = helper(b, "Min", vec![count, cap])?;
    let zero = word(b, 0)?;
    let initial = helper(b, "Make", vec![zero, zero])?;
    let result = helper(b, "Pipeline", vec![predicate, count, initial])?;
    let result = helper(b, "Value", vec![result])?;
    define(b, &definition.first_definition, &[depth + 5, 5], 5, result)?;
    for (name, invert) in [
        (&definition.any_definition, false),
        (&definition.all_definition, true),
    ] {
        let input = b.var(depth + 6)?;
        let args = (0..depth)
            .map(|i| b.var(depth + 4 - i))
            .collect::<R<Vec<_>>>()?;
        let leaf = b.app(input, args)?;
        let leaf = if invert { not(b, leaf)? } else { leaf };
        let leaf = zero_padding(b, depth + 5, depth, 5, leaf)?;
        let predicate = b.wrap_selectors(depth + 5, leaf)?;
        let count = b.var(0)?;
        let first = call(b, &definition.first_definition, vec![predicate, count])?;
        let empty = helper(b, "Empty", vec![first])?;
        let result = if invert { empty } else { not(b, empty)? };
        define(b, name, &[depth, 5], 0, result)?;
    }
    Ok(definition)
}
fn collect_depths(shape: &OrdinaryShape, depths: &mut BTreeSet<u32>) -> R<()> {
    match shape {
        OrdinaryShape::Bits { .. } | OrdinaryShape::Reference { .. } => {}
        OrdinaryShape::RoleBound { value, .. } => collect_depths(value, depths)?,
        OrdinaryShape::Product { fields } => {
            for f in fields {
                collect_depths(&f.shape, depths)?;
            }
        }
        OrdinaryShape::Sum { arms } => {
            for arm in arms {
                for f in &arm.fields {
                    collect_depths(&f.shape, depths)?;
                }
            }
        }
        OrdinaryShape::Array { capacity, element }
        | OrdinaryShape::Sequence { capacity, element } => {
            let depth = address_bits(*capacity);
            if depth > MAX_INDEX_BITS {
                return Err(OrdinaryCarrierError::Limit);
            }
            depths.insert(depth);
            collect_depths(element, depths)?;
        }
    }
    Ok(())
}
pub fn generate_csharp_practical_ordinary_ordered_folds(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryOrderedFoldProgram> {
    let carriers = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut depths = BTreeSet::new();
    for carrier in carriers.carriers() {
        collect_depths(&carrier.shape, &mut depths)?;
    }
    let mut b = Builder::new()?;
    let mut definitions = vec![];
    for depth in depths {
        definitions.push(emit_fold(&mut b, depth)?);
    }
    let static_transformers = b.static_transformers;
    let certificate = b.finish()?;
    let p = OrdinaryOrderedFoldProgram {
        schema: "mpk.csharp.ordinary_ordered_folds.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_ordered_folds(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryOrderedFoldProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_ordered_folds(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::{apply, bit as observed_bit, run, V};
    use super::*;
    #[test]
    fn ordered_fold_comparator_has_linear_syntactic_traversal() {
        let mut b = Builder::new().unwrap();
        auxiliary(&mut b).unwrap();
        let mut expanded = Vec::<u64>::new();
        for term in &b.c.term_table {
            let children = match term {
                TermNode::App {
                    function,
                    arguments,
                } => std::iter::once(*function)
                    .chain(arguments.iter().copied())
                    .collect::<Vec<_>>(),
                TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => vec![*ty, *body],
                TermNode::Let { ty, value, body } => vec![*ty, *value, *body],
                _ => vec![],
            };
            expanded.push(
                children
                    .into_iter()
                    .fold(1u64, |n, t| n.saturating_add(expanded[t as usize])),
            );
        }
        let declaration = &b.c.declarations[b.globals[&name("Less")] as usize];
        let DeclarationKind::Def { value, .. } = declaration.kind else {
            panic!()
        };
        // Count the expanded syntax without traversing it. Reusing the lower
        // comparison in both cases previously exceeded billions of visits.
        assert!(
            expanded[value as usize] < 8192,
            "comparison syntax grew exponentially"
        );
        let bytes = b.finish().unwrap();
        let c = decode_canonical_certificate(&bytes).unwrap();
        let values = [
            0,
            1,
            2,
            7,
            8,
            31,
            32,
            255,
            256,
            4095,
            4096,
            16383,
            16384,
            0x7fff_ffff,
            0x8000_0000,
            u32::MAX,
        ];
        for a in values {
            for v in values {
                assert_eq!(
                    observed_bit(run(&c, &name("Less"), vec![number(a), number(v)])),
                    a < v
                );
            }
        }
    }
    fn fixture() -> (Certificate, Vec<OrdinaryOrderedFoldDefinition>, Vec<u8>) {
        let mut b = Builder::new().unwrap();
        let defs = [0, 1, 3, 12, 14]
            .into_iter()
            .map(|d| emit_fold(&mut b, d).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(b.static_transformers, 8192);
        assert_eq!(emit_fold(&mut b, 15), Err(OrdinaryCarrierError::Limit));
        let before = b.c.declarations.len();
        assert_eq!(emit_fold(&mut b, 3).unwrap(), defs[2]);
        assert_eq!(b.c.declarations.len(), before);
        assert_eq!(b.static_transformers, 8192);
        let bytes = b.finish().unwrap();
        (decode_canonical_certificate(&bytes).unwrap(), defs, bytes)
    }
    fn number(n: u32) -> V {
        V::Cube((0..32).map(|i| n & (1 << i) != 0).collect())
    }
    fn observed_word(c: &Certificate, value: V) -> u32 {
        (0..32).fold(0, |n, i| {
            let mut v = value.clone();
            for bit in 0..5 {
                v = apply(c, v, V::Bit(i & (1 << bit) != 0));
            }
            n | ((observed_bit(v) as u32) << i)
        })
    }
    fn words(values: &[u32]) -> V {
        assert!(values.len().is_power_of_two());
        V::Cube(
            (0..values.len() * 32)
                .map(|a| values[a % values.len()] & (1 << (a / values.len())) != 0)
                .collect(),
        )
    }
    fn bools(values: Vec<bool>) -> V {
        if values.len() == 1 {
            V::Bit(values[0])
        } else {
            V::Cube(values)
        }
    }
    #[test]
    fn ordered_fold_words_preserve_order_and_clamp_counts() {
        let (c, defs, _) = fixture();
        for def in &defs[..3] {
            let capacity = def.capacity as usize;
            for index in 0..capacity {
                let mut values = vec![0; capacity];
                values[index] = 0x8000_0000;
                if index + 1 < capacity {
                    values[index + 1] = 7;
                }
                for count in [
                    0,
                    1,
                    index as u32,
                    index as u32 + 1,
                    def.capacity,
                    def.capacity + 1,
                    u32::MAX,
                ] {
                    let expected = values[..(count.min(def.capacity) as usize)]
                        .iter()
                        .copied()
                        .find(|v| *v != 0)
                        .unwrap_or(0);
                    let actual = run(
                        &c,
                        &def.first_definition,
                        vec![words(&values), number(count)],
                    );
                    assert_eq!(
                        observed_word(&c, actual),
                        expected,
                        "depth={} index={index} count={count}",
                        def.index_bits
                    );
                }
            }
        }
        // Test the carry transitions used by both ordered reads, including wrap.
        for n in [
            0,
            1,
            2,
            3,
            7,
            8,
            31,
            32,
            4095,
            4096,
            16382,
            16383,
            u32::MAX - 1,
            u32::MAX,
        ] {
            for delta in [1u32, 2] {
                let value = run(&c, &name(&format!("Add{delta}")), vec![number(n)]);
                assert_eq!(observed_word(&c, value), n.wrapping_add(delta));
            }
        }
    }
    #[test]
    fn ordered_fold_boolean_truth_tables() {
        let (c, defs, _) = fixture();
        for def in &defs[..3] {
            for mask in 0..1u32 << def.capacity {
                let values = (0..def.capacity)
                    .map(|i| mask & (1 << i) != 0)
                    .collect::<Vec<_>>();
                for count in [0, 1, def.capacity / 2, def.capacity, def.capacity + 1] {
                    let slice = &values[..count.min(def.capacity) as usize];
                    let args = vec![bools(values.clone()), number(count)];
                    assert_eq!(
                        observed_bit(run(&c, &def.any_definition, args.clone())),
                        slice.iter().any(|v| *v)
                    );
                    assert_eq!(
                        observed_bit(run(&c, &def.all_definition, args)),
                        slice.iter().all(|v| *v)
                    );
                }
            }
        }
    }
    #[test]
    fn ordered_fold_reaches_full_16384_bound() {
        let (c, defs, _) = fixture();
        let def = defs.last().unwrap();
        let mut values = vec![0; 16384];
        values[16383] = 0xffff_ffff;
        let value = run(
            &c,
            &def.first_definition,
            vec![words(&values), number(16384)],
        );
        assert_eq!(observed_word(&c, value), u32::MAX);
        // Odd count excludes the second read of the final transformer.
        let value = run(
            &c,
            &def.first_definition,
            vec![words(&values), number(16383)],
        );
        assert_eq!(observed_word(&c, value), 0);
    }
    #[test]
    fn ordered_fold_certificate_replays_and_limits() {
        let (c, defs, bytes) = fixture();
        let (_, again, second) = fixture();
        assert_eq!(defs, again);
        assert_eq!(bytes, second);
        let mut b = Builder::new().unwrap();
        emit_fold(&mut b, 0).unwrap();
        let identity = b.constant(&format!("{PREFIX}.Cube.D6.Identity")).unwrap();
        b.compose(6, &vec![identity; 8192]).unwrap();
        assert_eq!(b.static_transformers, 16384);
        assert_eq!(b.compose(6, &[identity]), Err(OrdinaryCarrierError::Limit));
        let metadata = serde_json::json!({"definitions":defs,"terms":c.term_table.len(),"declarations":c.declarations.len(),"static_transformers":8192,"inclusive_transformer_bound":16384,"rejected_transformer_count":16385,"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))});
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(out) = std::env::var_os("MPK_W09_ORDERED_FOLD_OUT") {
            let out = std::path::PathBuf::from(out);
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(out.join("ordered-fold.hex"), hex).unwrap();
            std::fs::write(
                out.join("core-metrics.json"),
                serde_json::to_vec_pretty(&metadata).unwrap(),
            )
            .unwrap();
        } else {
            let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/ordered-folds");
            assert_eq!(
                std::fs::read_to_string(out.join("ordered-fold.hex")).unwrap(),
                hex
            );
            assert_eq!(
                serde_json::from_slice::<Value>(
                    &std::fs::read(out.join("core-metrics.json")).unwrap()
                )
                .unwrap(),
                metadata
            );
        }
    }
}
