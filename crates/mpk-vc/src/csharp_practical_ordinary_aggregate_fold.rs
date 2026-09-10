//! Shared first/saturated-sum folds for recursive value domains. The two modes
//! share one counted concrete C6 -> C6 pipeline; no host predicate is trusted.
use super::*;
use ordered_fold::{cube_mux, helper as base, read_bit, word};
const STATE: u32 = 6;
const PREDICATE: u32 = 19;
const INVALID: u32 = (TOTAL_VALUE_CELLS_MAX + 1) as u32;
fn name(s: &str) -> String {
    format!("{PREFIX}.AggregateFold.{s}")
}
fn helper(b: &mut Builder, s: &str, args: Vec<u32>) -> R<u32> {
    call(b, &name(s), args)
}
fn pipeline(b: &mut Builder) -> R<()> {
    if b.globals.contains_key(&name("Pipeline")) {
        return Ok(());
    }
    ordered_fold::auxiliary(b)?;
    let add = super::super::scalar_bits::count_addition(b)?;
    let mode = b.var(2)?;
    let length = b.var(1)?;
    let state = b.var(0)?;
    let index = base(b, "Index", vec![state])?;
    let value = base(b, "Value", vec![state])?;
    let within = base(b, "Less", vec![index, length])?;
    let empty = base(b, "Empty", vec![value])?;
    let bound = word(b, INVALID)?;
    let not_saturated = base(b, "Less", vec![value, bound])?;
    let can_continue = mux(b, mode, not_saturated, empty)?;
    let no = bit(b, false)?;
    let active = mux(b, within, can_continue, no)?;
    define(b, &name("Active"), &[0, 5, STATE], 0, active)?;
    let predicate = b.var(1)?;
    let index = b.var(0)?;
    let args = (0..14)
        .map(|i| read_bit(b, index, i))
        .collect::<R<Vec<_>>>()?;
    // Preserve the resulting word as one value. Eta-expanding its five
    // selectors would reapply a computed predicate for every output bit.
    let body = b.app(predicate, args)?;
    define(b, &name("ReadAt"), &[PREDICATE, 5], 5, body)?;
    let mode = b.var(3)?;
    let predicate = b.var(2)?;
    let length = b.var(1)?;
    let state = b.var(0)?;
    let index = base(b, "Index", vec![state])?;
    let previous = base(b, "Value", vec![state])?;
    let next = base(b, "Add1", vec![index])?;
    let after = base(b, "Add2", vec![index])?;
    let first = helper(b, "ReadAt", vec![predicate, index])?;
    let second = helper(b, "ReadAt", vec![predicate, next])?;
    let within = base(b, "Less", vec![next, length])?;
    let zero = word(b, 0)?;
    let second = cube_mux(b, 5, within, second, zero)?;
    let empty = base(b, "Empty", vec![first])?;
    let first_result = cube_mux(b, 5, empty, second, first)?;
    let sum = call(b, &add, vec![previous, first])?;
    let sum = call(b, &add, vec![sum, second])?;
    let value = cube_mux(b, 5, mode, sum, first_result)?;
    let updated = base(b, "Make", vec![after, value])?;
    let active = helper(b, "Active", vec![mode, length, state])?;
    let body = cube_mux(b, STATE, active, updated, state)?;
    define(b, &name("StepTwo"), &[0, PREDICATE, 5, STATE], STATE, body)?;
    let state_ty = b.cube(STATE)?;
    let transformer = b.pi(state_ty, state_ty)?;
    let word_ty = b.cube(5)?;
    let f = b.var(2)?;
    let state = b.var(0)?;
    let first = b.app(f, vec![state])?;
    // Under the intermediate-state let: mode, length, f, g, state, intermediate.
    let mode = b.var(5)?;
    let length = b.var(4)?;
    let g = b.var(2)?;
    let intermediate = b.var(0)?;
    let next = b.app(g, vec![intermediate])?;
    let active = helper(b, "Active", vec![mode, length, intermediate])?;
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
    let body = b.lam(b.boolean, body)?;
    let ty = b.pi(transformer, transformer)?;
    let ty = b.pi(transformer, ty)?;
    let ty = b.pi(word_ty, ty)?;
    let ty = b.pi(b.boolean, ty)?;
    b.define(&name("Compose"), ty, body)?;
    let mode = b.var(2)?;
    let predicate = b.var(1)?;
    let length = b.var(0)?;
    let step = helper(b, "StepTwo", vec![mode, predicate, length])?;
    let composition = helper(b, "Compose", vec![mode, length])?;
    let predicate_ty = b.cube(PREDICATE)?;
    let ty = b.pi(word_ty, transformer)?;
    let ty = b.pi(predicate_ty, ty)?;
    let ty = b.pi(b.boolean, ty)?;
    // Four original two-element steps form one ordinary guarded transformer.
    // Compose still checks Active after every intermediate result. The full
    // pipeline expands to exactly 8,192 ordered StepTwo calls (16,384 cells),
    // charging the four group occurrences and all 2,048 pipeline occurrences.
    let group = b.compose_term(STATE, composition, &[step; 4])?;
    let body = b.lam(word_ty, group)?;
    let body = b.lam(predicate_ty, body)?;
    let body = b.lam(b.boolean, body)?;
    b.define(&name("StepEight"), ty, body)?;
    let step = helper(b, "StepEight", vec![mode, predicate, length])?;
    let value = b.compose_term(STATE, composition, &vec![step; 2048])?;
    let body = b.lam(word_ty, value)?;
    let body = b.lam(predicate_ty, body)?;
    let body = b.lam(b.boolean, body)?;
    b.define(&name("Pipeline"), ty, body)
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryAggregateFoldDefinition {
    pub index_bits: u32,
    pub capacity: u32,
    pub first_definition: String,
    pub any_definition: String,
    pub all_definition: String,
    /// Sum of u32 words, saturated at 65,537. Count is clamped to capacity.
    /// Neither clamping nor this helper discharges a value's domain or role bound.
    pub sum_definition: String,
}
pub(in super::super) fn emit_fold(
    b: &mut Builder,
    depth: u32,
) -> R<OrdinaryAggregateFoldDefinition> {
    if depth > 14 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let d = OrdinaryAggregateFoldDefinition {
        index_bits: depth,
        capacity: 1 << depth,
        first_definition: name(&format!("First.D{depth}")),
        any_definition: name(&format!("Any.D{depth}")),
        all_definition: name(&format!("All.D{depth}")),
        sum_definition: name(&format!("Sum.D{depth}")),
    };
    if b.globals.contains_key(&d.first_definition) {
        return Ok(d);
    }
    pipeline(b)?;
    for (name, sum) in [(&d.first_definition, false), (&d.sum_definition, true)] {
        let input = b.var(15)?;
        let args = (0..depth).map(|i| b.var(13 - i)).collect::<R<Vec<_>>>()?;
        let body = b.app(input, args)?;
        // Lift only the index group; the original predicate already returns
        // C5. Keep that application shared across the word's output bits.
        let predicate = b.wrap_selectors(14, body)?;
        let count = b.var(0)?;
        let cap = word(b, 1 << depth)?;
        let count = base(b, "Min", vec![count, cap])?;
        let zero = word(b, 0)?;
        let initial = base(b, "Make", vec![zero, zero])?;
        let mode = bit(b, sum)?;
        let result = helper(b, "Pipeline", vec![mode, predicate, count, initial])?;
        let result = base(b, "Value", vec![result])?;
        define(b, name, &[depth + 5, 5], 5, result)?;
    }
    for (name, invert) in [(&d.any_definition, false), (&d.all_definition, true)] {
        let input = b.var(depth + 6)?;
        let args = (0..depth)
            .map(|i| b.var(depth + 4 - i))
            .collect::<R<Vec<_>>>()?;
        let leaf = b.app(input, args)?;
        let leaf = if invert {
            call(b, "Std.Bool.not", vec![leaf])?
        } else {
            leaf
        };
        let leaf = zero_padding(b, depth + 5, depth, 5, leaf)?;
        let predicate = b.wrap_selectors(depth + 5, leaf)?;
        let count = b.var(0)?;
        let first = call(b, &d.first_definition, vec![predicate, count])?;
        let empty = base(b, "Empty", vec![first])?;
        let result = if invert {
            empty
        } else {
            call(b, "Std.Bool.not", vec![empty])?
        };
        define(b, name, &[depth, 5], 0, result)?;
    }
    Ok(d)
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryAggregateFoldProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryAggregateFoldDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryAggregateFoldProgram {
    pub fn definitions(&self) -> &[OrdinaryAggregateFoldDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed aggregate program")
    }
}
pub fn generate_csharp_practical_ordinary_aggregate_folds(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryAggregateFoldProgram> {
    let carriers = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut depths = BTreeSet::new();
    for carrier in carriers.carriers() {
        ordered_fold::collect_depths(&carrier.shape, &mut depths)?;
    }
    let mut b = Builder::new()?;
    let mut definitions = vec![];
    for depth in depths {
        definitions.push(emit_fold(&mut b, depth)?);
    }
    let static_transformers = b.static_transformers;
    let certificate = b.finish()?;
    let p = OrdinaryAggregateFoldProgram {
        schema: "mpk.csharp.ordinary_aggregate_folds.v1".into(),
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
pub fn import_csharp_practical_ordinary_aggregate_folds(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryAggregateFoldProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_aggregate_folds(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::{apply, bit as observed_bit, run, V};
    use super::*;
    use std::{fs, path::Path};
    fn number(n: u32) -> V {
        V::Cube((0..32).map(|i| n & (1 << i) != 0).collect())
    }
    fn observed(c: &Certificate, value: V) -> u32 {
        (0..32).fold(0, |n, i| {
            let mut v = value.clone();
            for k in 0..5 {
                v = apply(c, v, V::Bit(i & (1 << k) != 0));
            }
            n | ((observed_bit(v) as u32) << i)
        })
    }
    fn words(values: &[u32]) -> V {
        V::Cube(
            (0..32)
                .flat_map(|bit| values.iter().map(move |n| n & (1 << bit) != 0))
                .collect(),
        )
    }
    fn fixture() -> (
        Certificate,
        Vec<OrdinaryAggregateFoldDefinition>,
        Vec<u8>,
        usize,
    ) {
        let mut b = Builder::new().unwrap();
        let mut defs = vec![];
        for d in 0..=14 {
            defs.push(emit_fold(&mut b, d).unwrap());
        }
        let count = b.static_transformers;
        assert!(
            count > 2048 && count < 2150,
            "one pipeline plus finite scalar addition"
        );
        for d in 0..=14 {
            assert_eq!(emit_fold(&mut b, d).unwrap(), defs[d as usize]);
        }
        assert_eq!(
            b.static_transformers, count,
            "both modes and all widths share one pipeline"
        );
        assert_eq!(emit_fold(&mut b, 15), Err(OrdinaryCarrierError::Limit));
        let bytes = b.finish().unwrap();
        (
            mpk_cert::decode_canonical_certificate(&bytes).unwrap(),
            defs,
            bytes,
            count,
        )
    }
    #[test]
    fn aggregate_fold_decimal_json_collection_budget() {
        use super::super::super::scalar_bits::{
            emit_boundary_json, emit_decimal_parsers_for_shared_test,
        };
        let mut b = Builder::new().unwrap();
        ordered_fold::emit_fold(&mut b, 14).unwrap();
        let collection_count = b.static_transformers;
        assert!(collection_count >= 8192);
        let json = emit_boundary_json(&mut b).unwrap();
        let before_decimal = b.static_transformers;
        let decimal = emit_decimal_parsers_for_shared_test(&mut b);
        eprintln!("Shared decimal budget: collection {collection_count}, collection/JSON {before_decimal}, after decimal attempt {}; result {}", b.static_transformers, match &decimal { Ok(_) => "ok", Err(OrdinaryCarrierError::Limit) => "limit", Err(_) => "other error" });
        let decimal =
            decimal.expect("complete decimal parsing must coexist with collection and JSON");
        assert_eq!(decimal.len(), 146);
        assert_eq!(
            b.static_transformers, before_decimal,
            "JSON already emitted the same decimal parsers"
        );
        assert_eq!(
            decimal,
            json.quoted_decimals
                .iter()
                .map(|d| d.codec.clone())
                .collect::<Vec<_>>()
        );
        assert!(b.static_transformers <= 16_384);
        let static_transformers = b.static_transformers;
        let bytes = b.finish().unwrap();
        let cert = mpk_cert::decode_canonical_certificate(&bytes).unwrap();
        let metadata = serde_json::json!({
            "scope":"full collection, JSON and decimal definition integration; not application VC proofs",
            "collection_transformers":collection_count,
            "before_decimal_transformers":before_decimal,
            "static_transformers":static_transformers,
            "terms":cert.term_table.len(),"declarations":cert.declarations.len(),
            "json":json,"decimal":decimal,
            "certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes)),
        });
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(dir) = std::env::var_os("MPK_W09_DECIMAL_SHARED_OUT") {
            let dir = std::path::PathBuf::from(dir);
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("certificate.json"),
                serde_json::to_vec_pretty(&metadata).unwrap(),
            )
            .unwrap();
            fs::write(dir.join("collection-json-decimal.hex"), hex).unwrap();
        } else {
            let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../develop/migrations/csharp-03/ordinary-foundation/decimal-json-collection-shared");
            assert_eq!(
                fs::read_to_string(dir.join("collection-json-decimal.hex")).unwrap(),
                hex
            );
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(
                    &fs::read(dir.join("certificate.json")).unwrap()
                )
                .unwrap(),
                metadata
            );
        }
    }
    #[test]
    fn aggregate_fold_group_boundaries() {
        let (c, defs, _, _) = fixture();
        let d = &defs[4];
        for position in [6, 7, 8, 9] {
            for marker in [17, INVALID, u32::MAX] {
                let mut values = vec![0; 16];
                values[position] = marker;
                for length in [7, 8, 9, 10] {
                    let visible = position < length as usize;
                    for (method, expected) in [
                        (&d.first_definition, if visible { marker } else { 0 }),
                        (
                            &d.sum_definition,
                            if visible { marker.min(INVALID) } else { 0 },
                        ),
                    ] {
                        assert_eq!(
                            observed(&c, run(&c, method, vec![words(&values), number(length)])),
                            expected,
                            "group boundary position {position} length {length} marker {marker} {method}",
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn aggregate_fold_modes_saturation_and_odd_counts() {
        let (c, defs, _, _) = fixture();
        for d in &defs[..=3] {
            let capacity = d.capacity as usize;
            for raw in [
                vec![0; capacity],
                vec![1; capacity],
                (0..capacity)
                    .map(|i| if i == capacity - 1 { 65536 } else { 0 })
                    .collect(),
                (0..capacity)
                    .map(|i| if i == 0 { 65536 } else { 1 })
                    .collect(),
                vec![u32::MAX; capacity],
            ] {
                for length in [
                    0,
                    1,
                    d.capacity.saturating_sub(1),
                    d.capacity,
                    d.capacity + 1,
                    u32::MAX,
                ] {
                    let values = &raw[..(length.min(d.capacity) as usize)];
                    let sum = values
                        .iter()
                        .map(|&x| x as u64)
                        .sum::<u64>()
                        .min(INVALID as u64) as u32;
                    let first = values.iter().copied().find(|&x| x != 0).unwrap_or(0);
                    assert_eq!(
                        observed(
                            &c,
                            run(&c, &d.sum_definition, vec![words(&raw), number(length)])
                        ),
                        sum,
                        "sum depth {} len {length}",
                        d.index_bits
                    );
                    assert_eq!(
                        observed(
                            &c,
                            run(&c, &d.first_definition, vec![words(&raw), number(length)])
                        ),
                        first
                    );
                }
            }
            for mask in 0..(1usize << capacity) {
                let values = (0..capacity)
                    .map(|i| mask & (1 << i) != 0)
                    .collect::<Vec<_>>();
                let predicate = if capacity == 1 {
                    V::Bit(values[0])
                } else {
                    V::Cube(values.clone())
                };
                for length in [0, 1, d.capacity] {
                    let visible = &values[..length as usize];
                    assert_eq!(
                        observed_bit(run(
                            &c,
                            &d.any_definition,
                            vec![predicate.clone(), number(length)]
                        )),
                        visible.iter().any(|v| *v)
                    );
                    assert_eq!(
                        observed_bit(run(
                            &c,
                            &d.all_definition,
                            vec![predicate.clone(), number(length)]
                        )),
                        visible.iter().all(|v| *v)
                    );
                }
            }
        }
    }
    #[test]
    fn aggregate_fold_certificates_replay_and_limits() {
        let (c, definitions, bytes, static_transformers) = fixture();
        let metadata = serde_json::json!({"definitions":definitions, "static_transformers":static_transformers,
            "terms": c.term_table.len(), "declarations": c.declarations.len(),
            "certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))});
        let output = std::env::var_os("MPK_W09_AGGREGATE_OUT").map(std::path::PathBuf::from);
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/aggregate-folds");
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(output) = output {
            fs::create_dir_all(&output).unwrap();
            fs::write(output.join("core.hex"), hex).unwrap();
            fs::write(
                output.join("core.json"),
                serde_json::to_vec_pretty(&metadata).unwrap(),
            )
            .unwrap();
        } else {
            assert_eq!(fs::read_to_string(path.join("core.hex")).unwrap(), hex);
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(
                    &fs::read(path.join("core.json")).unwrap()
                )
                .unwrap(),
                metadata
            );
        }
        let mut b = Builder::new().unwrap();
        b.static_transformers = 16384 - static_transformers;
        emit_fold(&mut b, 14).unwrap();
        assert_eq!(b.static_transformers, 16384);
        let mut b = Builder::new().unwrap();
        b.static_transformers = 16385 - static_transformers;
        assert_eq!(emit_fold(&mut b, 14), Err(OrdinaryCarrierError::Limit));
    }
    #[test]
    fn aggregate_fold_short_circuits_invalid_and_empty_inputs() {
        use super::super::super::test_eval::{thunk, Env};
        use std::rc::Rc;
        let mut b = Builder::new().unwrap();
        let d = emit_fold(&mut b, 1).unwrap();
        let poison_word = b.var(1).unwrap();
        let index = b.var(0).unwrap();
        let invalid = word(&mut b, INVALID).unwrap();
        let body = cube_mux(&mut b, 5, index, poison_word, invalid).unwrap();
        let body = b.lam(b.boolean, body).unwrap();
        define(&mut b, "Test.AfterFirst", &[5], 6, body).unwrap();
        let c = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        // Poison is an observer argument, never part of an accepted certificate.
        let poison = || thunk(u32::MAX, Rc::new(Env::Empty));
        for method in [&d.sum_definition, &d.first_definition] {
            assert_eq!(observed(&c, run(&c, method, vec![poison(), number(0)])), 0);
            let predicate = run(&c, "Test.AfterFirst", vec![poison()]);
            assert_eq!(
                observed(&c, run(&c, method, vec![predicate, number(2)])),
                INVALID
            );
        }
        assert!(!observed_bit(run(
            &c,
            &d.any_definition,
            vec![poison(), number(0)]
        )));
        assert!(observed_bit(run(
            &c,
            &d.all_definition,
            vec![poison(), number(0)]
        )));
    }
    #[test]
    fn aggregate_fold_full_capacity_sums_exact_logical_bound() {
        let (c, defs, _, _) = fixture();
        let d = &defs[14];
        let mut values = vec![4; 16384];
        values[0] = 5;
        values[16383] = 3;
        // Both endpoints matter: aliasing the high final index would saturate,
        // rather than produce the inclusive valid logical-cell bound.
        assert_eq!(
            observed(
                &c,
                run(
                    &c,
                    &d.sum_definition,
                    vec![words(&values), number(u32::MAX)]
                )
            ),
            TOTAL_VALUE_CELLS_MAX as u32
        );
    }
    #[test]
    fn aggregate_fold_shares_computed_predicate_words() {
        use super::super::super::test_eval::run_counted;
        let mut b = Builder::new().unwrap();
        let fold = emit_fold(&mut b, 1).unwrap();
        let add = super::super::super::scalar_bits::count_addition(&mut b).unwrap();
        let one = word(&mut b, 1).unwrap();
        let zero = word(&mut b, 0).unwrap();
        let result = call(&mut b, &add, vec![one, zero]).unwrap();
        define(&mut b, "Test.Computed", &[0], 5, result).unwrap();
        // Negative control: extensionally the same predicate, but applying it
        // again inside every word selector repeats its finite calculation.
        let index = b.var(5).unwrap();
        let result = call(&mut b, "Test.Computed", vec![index]).unwrap();
        let selectors = b.selectors(5).unwrap();
        let result = b.app(result, selectors).unwrap();
        let result = b.wrap_selectors(5, result).unwrap();
        define(&mut b, "Test.Repeated", &[0], 5, result).unwrap();
        for (name, predicate) in [
            ("Test.SharedCheck", "Test.Computed"),
            ("Test.RepeatedCheck", "Test.Repeated"),
        ] {
            let predicate = b.constant(predicate).unwrap();
            let count = word(&mut b, 2).unwrap();
            let sum = call(&mut b, &fold.sum_definition, vec![predicate, count]).unwrap();
            // Bind the resulting C5 value once before checking all 32 leaves.
            let value = b.var(0).unwrap();
            let mut equal = bit(&mut b, true).unwrap();
            for i in 0..32 {
                let observed = read_bit(&mut b, value, i).unwrap();
                let expected = if i == 1 {
                    observed
                } else {
                    call(&mut b, "Std.Bool.not", vec![observed]).unwrap()
                };
                equal = call(&mut b, "Std.Bool.and", vec![equal, expected]).unwrap();
            }
            let ty = b.cube(5).unwrap();
            let body = b
                .term(TermNode::Let {
                    ty,
                    value: sum,
                    body: equal,
                })
                .unwrap();
            define(&mut b, name, &[], 0, body).unwrap();
        }
        let cert = mpk_cert::decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        let (shared, shared_steps) = run_counted(&cert, "Test.SharedCheck");
        let (repeated, repeated_steps) = run_counted(&cert, "Test.RepeatedCheck");
        assert!(observed_bit(shared) && observed_bit(repeated));
        eprintln!(
            "computed predicate word steps: shared {shared_steps}, repeated {repeated_steps}"
        );
        assert!(
            repeated_steps > 2 * shared_steps,
            "predicate word sharing regressed"
        );
    }
}
