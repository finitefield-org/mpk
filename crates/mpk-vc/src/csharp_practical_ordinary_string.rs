//! Ordinary UTF-16 operations over the complete fixed-capacity string carrier.
//! Length, indexing, construction, ordinal comparison and search are lowered
//! to finite ordinary definitions; domains and literal bodies remain separate.
use super::*;

#[path = "csharp_practical_ordinary_string_construct.rs"]
mod construct;
#[path = "csharp_practical_ordinary_string_ordinal.rs"]
mod ordinal;

const TEXT_DEPTH: u32 = 19;
const BASIC: &[&str] = &["string.length", "string.index", "string.is_null_or_empty"];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryStringDefinition {
    pub operation: ClosedOperationSignature,
    pub result_definition: String,
    pub success_definition: String,
    pub ordered_failure_definitions: Vec<String>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryStringProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryStringDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
    #[serde(skip)]
    static_transformers: usize,
}
impl OrdinaryStringProgram {
    pub fn definitions(&self) -> &[OrdinaryStringDefinition] {
        &self.definitions
    }
    /// Explicit transformer occurrences, counted before DAG sharing.
    pub fn static_transformers(&self) -> usize {
        self.static_transformers
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed ordinary string program")
    }
}

struct Helpers {
    depth: u32,
    present: String,
    length: String,
    read: String,
    range: OrdinaryScalarDefinition,
    empty: OrdinaryScalarDefinition,
    construct: Option<construct::Aux>,
    ordinal: Option<ordinal::Aux>,
}
/// One contract compiler owns one closed context and its selected nullable
/// string representation. Existing operation bodies remain unchanged.
#[derive(Default)]
pub(in super::super) struct ContractStringCache {
    helpers: Option<Helpers>,
    definitions: BTreeMap<String, (OrdinaryStringDefinition, Vec<String>)>,
}
impl ContractStringCache {
    pub(in super::super) fn emit(
        &mut self,
        b: &mut Builder,
        signature: ClosedOperationSignature,
        nullable: bool,
    ) -> R<(OrdinaryStringDefinition, Vec<String>)> {
        // The admitted unary/binary contract tags have exactly one/two
        // operands. Larger native-call arities use the standalone emitter.
        if !matches!(signature.argument_type_ids.len(), 1 | 2) {
            return Err(OrdinaryCarrierError::Shape);
        }
        if let Some(h) = &self.helpers {
            if h.depth != TEXT_DEPTH + u32::from(nullable) {
                return Err(OrdinaryCarrierError::Linkage);
            }
        }
        if let Some(existing) = self.definitions.get(&signature.id) {
            return if existing.0.operation == signature {
                Ok(existing.clone())
            } else {
                Err(OrdinaryCarrierError::Linkage)
            };
        }
        if self.helpers.is_none() {
            self.helpers = Some(Helpers::new(b, nullable)?);
        }
        let h = self.helpers.as_mut().unwrap();
        let definition = h.emit(b, signature)?;
        let mut checks = definition.ordered_failure_definitions.clone();
        // Contract failure symbols denote individual failed checks, without
        // masking by an earlier exception. Keep legacy ordered failures intact.
        if checks.len() > 1 {
            if checks.len() != 2 {
                return Err(OrdinaryCarrierError::Shape);
            }
            let left = b.var(1)?;
            let right = b.var(0)?;
            let (valid, inputs) = match definition.operation.id.as_str() {
                "string.index" => {
                    let length = call(b, &h.length, vec![left])?;
                    let valid = call(b, &h.range.result_definition, vec![right, length])?;
                    (valid, vec![1 << h.depth, 32])
                }
                "string.contains.ordinal"
                | "string.starts_with.ordinal"
                | "string.ends_with.ordinal" => {
                    (call(b, &h.present, vec![right])?, vec![1 << h.depth; 2])
                }
                _ => return Err(OrdinaryCarrierError::Shape),
            };
            let failed = call(b, "Std.Bool.not", vec![valid])?;
            let name = format!("{}.ContractRaw.F1", definition.result_definition);
            define(b, &name, &inputs, 0, failed)?;
            checks[1] = name;
        }
        let result = (definition, checks);
        self.definitions
            .insert(result.0.operation.id.clone(), result.clone());
        Ok(result)
    }
}
fn call(b: &mut Builder, name: &str, args: Vec<u32>) -> R<u32> {
    let f = b.constant(name)?;
    b.app(f, args)
}
fn define(b: &mut Builder, name: &str, inputs: &[usize], output: u32, body: u32) -> R<()> {
    let body = bind_inputs(b, inputs, body)?;
    let ty = b.cube(output)?;
    let ty = input_type(b, inputs, ty)?;
    b.define(name, ty, body)
}
fn small_check(b: &mut Builder, id: &str, c: Circuit, output: Bit) -> R<OrdinaryScalarDefinition> {
    let args = vec![I32_TYPE_ID.into(); c.inputs.len()];
    emit_circuit(
        b,
        IntegerCircuit {
            signature: ClosedOperationSignature {
                id: id.into(),
                tag: ClosedOperationTag::Data,
                argument_type_ids: args,
                normal_result_type_id: BOOL_TYPE_ID.into(),
                ordered_checks: vec![],
            },
            circuit: c,
            output: vec![output],
            failures: vec![],
        },
        "StringChecks",
    )
}
impl Helpers {
    fn new(b: &mut Builder, nullable: bool) -> R<Self> {
        let depth = TEXT_DEPTH + u32::from(nullable);
        let name = format!("{PREFIX}.String.N{}", u8::from(nullable));
        let present = format!("{name}.Present");
        let length = format!("{name}.Length");
        let read = format!("{name}.Read");
        let f = b.constant("Std.Bool.false")?;
        let t = b.constant("Std.Bool.true")?;
        let source = b.var(0)?;
        let exists = if nullable {
            core_read(b, source, 0, depth)?
        } else {
            t
        };
        define(b, &present, &[1 << depth], 0, exists)?;

        // Sequence role, thirteen padding bits, then the five length selectors.
        // An option adds its payload-role selector before the sequence role.
        let source = b.var(5)?;
        let mut args = if nullable { vec![t] } else { vec![] };
        args.push(f);
        args.extend(vec![f; 13]);
        args.extend(b.selectors(5)?);
        let value = b.app(source, args)?;
        let value = b.wrap_selectors(5, value)?;
        define(b, &length, &[1 << depth], 5, value)?;

        // Dynamic indexing applies the actual string cube to fourteen index
        // bits and four character-bit selectors. No array is truncated or
        // multiplexed through a host-observed result.
        let source = b.var(5)?;
        let index = b.var(4)?;
        let mut args = if nullable { vec![t] } else { vec![] };
        args.push(t);
        for bit in 0..14 {
            args.push(core_read(b, index, bit, 5)?);
        }
        args.extend(b.selectors(4)?);
        let value = b.app(source, args)?;
        let value = b.wrap_selectors(4, value)?;
        define(b, &read, &[1 << depth, 32], 4, value)?;

        let mut c = Circuit::new(&[32, 32]);
        let index = c.inputs[0].clone();
        let len = c.inputs[1].clone();
        let less = c.lt(&index, &len, false);
        let nonnegative = c.not(index[31]);
        let valid = c.and(nonnegative, less);
        let range = small_check(b, &format!("{name}.IndexRange"), c, valid)?;
        let mut c = Circuit::new(&[32]);
        let input = c.inputs[0].clone();
        let nonzero = c.nonzero(&input);
        let empty = c.not(nonzero);
        let empty = small_check(b, &format!("{name}.Empty"), c, empty)?;
        Ok(Self {
            depth,
            present,
            length,
            read,
            range,
            empty,
            construct: None,
            ordinal: None,
        })
    }
    fn emit(
        &mut self,
        b: &mut Builder,
        signature: ClosedOperationSignature,
    ) -> R<OrdinaryStringDefinition> {
        if construct::supports(&signature.id) {
            if self.construct.is_none() {
                self.construct = Some(construct::Aux::new(b)?);
            }
            return construct::emit(b, self, self.construct.as_ref().unwrap(), signature);
        }
        if ordinal::OPS.contains(&signature.id.as_str()) {
            if self.ordinal.is_none() {
                self.ordinal = Some(ordinal::Aux::new(b, self)?);
            }
            return ordinal::emit(b, self, self.ordinal.as_ref().unwrap(), signature);
        }
        let id = signature.id.as_str();
        if !BASIC.contains(&id) {
            return Err(OrdinaryCarrierError::Shape);
        }
        let indexed = id == "string.index";
        let inputs = if indexed {
            vec![1 << self.depth, 32]
        } else {
            vec![1 << self.depth]
        };
        let args = (0..inputs.len())
            .rev()
            .map(|i| b.var(i as u32))
            .collect::<R<Vec<_>>>()?;
        let present = call(b, &self.present, vec![args[0]])?;
        let absent = call(b, "Std.Bool.not", vec![present])?;
        let length = call(b, &self.length, vec![args[0]])?;
        let t = b.constant("Std.Bool.true")?;
        let f = b.constant("Std.Bool.false")?;
        let (result, success, failures, output_depth) = match id {
            "string.length" => (length, present, vec![absent], 5),
            "string.index" => {
                let range = call(b, &self.range.result_definition, vec![args[1], length])?;
                let valid = call(b, "Std.Bool.and", vec![present, range])?;
                let invalid = call(b, "Std.Bool.not", vec![range])?;
                let range_failure = call(b, "Std.Bool.and", vec![present, invalid])?;
                let result = call(b, &self.read, args.clone())?;
                (result, valid, vec![absent, range_failure], 4)
            }
            "string.is_null_or_empty" => {
                let empty = call(b, &self.empty.result_definition, vec![length])?;
                let result = core_mux(b, present, empty, t)?;
                (result, t, vec![], 0)
            }
            _ => unreachable!(),
        };
        if failures.len() != signature.ordered_checks.len() {
            return Err(OrdinaryCarrierError::Shape);
        }
        let name = format!(
            "{PREFIX}.String.O{}",
            id.bytes().map(|x| format!("{x:02x}")).collect::<String>()
        );
        let result_definition = format!("{name}.Result");
        let success_definition = format!("{name}.Success");
        let ordered_failure_definitions = (0..failures.len())
            .map(|i| format!("{name}.Failure.F{i}"))
            .collect::<Vec<_>>();
        let normal = if output_depth == 0 {
            core_mux(b, success, result, f)?
        } else {
            let helper = format!("{PREFIX}.Cube.D{output_depth}.Mux");
            if !b.globals.contains_key(&helper) {
                b.helpers(output_depth)?;
            }
            let zero = b.constant(&format!("{PREFIX}.Cube.D{output_depth}.Zero"))?;
            call(b, &helper, vec![success, result, zero])?
        };
        define(b, &result_definition, &inputs, output_depth, normal)?;
        define(b, &success_definition, &inputs, 0, success)?;
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
}

pub fn generate_csharp_practical_ordinary_strings(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStringProgram> {
    let signatures = vir
        .operation_signatures()
        .iter()
        .filter(|s| s.id.starts_with("string."))
        .map(|s| (s.id.clone(), s))
        .collect::<BTreeMap<_, _>>();
    let mut b = Builder::new()?;
    let mut helpers = None;
    let mut definitions = vec![];
    for (id, expected) in signatures {
        let signature = strings::operation_signature(vir.data_closed(), &id)
            .map_err(|_| OrdinaryCarrierError::Shape)?;
        if &signature != expected {
            return Err(OrdinaryCarrierError::Linkage);
        }
        if !BASIC.contains(&id.as_str())
            && !construct::supports(&id)
            && !ordinal::OPS.contains(&id.as_str())
        {
            return Err(OrdinaryCarrierError::Shape);
        }
        if helpers.is_none() {
            helpers = Some(Helpers::new(
                &mut b,
                strings::operation_signature(vir.data_closed(), "string.length")
                    .map_err(|_| OrdinaryCarrierError::Shape)?
                    .argument_type_ids[0]
                    != STRING_TYPE_ID,
            )?);
        }
        definitions.push(helpers.as_mut().unwrap().emit(&mut b, signature)?);
    }
    let static_transformers = b.static_transformers;
    let certificate = b.finish()?;
    let p = OrdinaryStringProgram {
        schema: "mpk.csharp.ordinary_strings.v1".into(),
        static_transformers,
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_strings(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryStringProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_strings(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::super::super::tests::{apply, bit, run, V};
    use super::*;
    fn signature(id: &str, nullable: bool) -> ClosedOperationSignature {
        let text = if nullable {
            "test.option.string"
        } else {
            STRING_TYPE_ID
        };
        let (arguments, result, checks) = match id {
            "string.length" => (
                vec![text.into()],
                I32_TYPE_ID,
                vec!["exception.null_receiver"],
            ),
            "string.index" => (
                vec![text.into(), I32_TYPE_ID.into()],
                "mpk.csharp.value.char.v1",
                vec!["exception.null_receiver", "index_range"],
            ),
            "string.is_null_or_empty" => (vec![text.into()], BOOL_TYPE_ID, vec![]),
            _ => unreachable!(),
        };
        ClosedOperationSignature {
            id: id.into(),
            tag: ClosedOperationTag::Data,
            argument_type_ids: arguments,
            normal_result_type_id: result.into(),
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
    #[test]
    fn string_contract_cache_preserves_legacy_bodies_and_context() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation");
        let mut signatures = BTreeMap::new();
        for family in [
            "string-basic-circuits",
            "string-construction-circuits",
            "string-ordinal-circuits",
        ] {
            let rows: Value = serde_json::from_slice(
                &std::fs::read(root.join(family).join("metrics.json")).unwrap(),
            )
            .unwrap();
            for row in rows.as_array().unwrap() {
                let definitions = row["metadata"]["definitions"].as_array().unwrap();
                let nullable = definitions
                    .iter()
                    .flat_map(|d| d["operation"]["argument_type_ids"].as_array().unwrap())
                    .any(|id| id.as_str().unwrap().starts_with("mpk.csharp.instance."));
                for definition in definitions {
                    let op: ClosedOperationSignature =
                        serde_json::from_value(definition["operation"].clone()).unwrap();
                    if matches!(op.argument_type_ids.len(), 1 | 2) {
                        signatures.insert((op.id.clone(), nullable), op);
                    }
                }
            }
        }
        let frozen = signatures.clone();
        // Cover every one/two-operand interpolation shape and static concat,
        // including variants absent from the retained source pin corpus.
        let bound = check_contract("obligation.output_bound").unwrap();
        let extra = ["s", "c", "ss", "sc", "cs", "cc"];
        for shape in extra {
            let id = format!("string.interpolation.restricted.{shape}");
            signatures.insert(
                (id.clone(), false),
                ClosedOperationSignature {
                    id,
                    tag: ClosedOperationTag::Data,
                    argument_type_ids: shape
                        .chars()
                        .map(|kind| {
                            if kind == 's' {
                                STRING_TYPE_ID.into()
                            } else {
                                "mpk.csharp.value.char.v1".into()
                            }
                        })
                        .collect(),
                    normal_result_type_id: STRING_TYPE_ID.into(),
                    ordered_checks: vec![RequiredCheck {
                        id: "obligation.output_bound".into(),
                        tag: bound.tag,
                        failure_type_id: None,
                    }],
                },
            );
        }
        let mut concat = signatures
            .iter()
            .find(|((id, _), _)| id == "string.concat.operator.string_string")
            .unwrap()
            .1
            .clone();
        concat.id = "string.concat.string2".into();
        signatures.insert((concat.id.clone(), true), concat);
        let originals = signatures.values().cloned().collect::<Vec<_>>();
        for operation in originals {
            for nullable in [false, true] {
                let mut operation = operation.clone();
                for arg in &mut operation.argument_type_ids {
                    if arg == STRING_TYPE_ID || arg.starts_with("mpk.csharp.instance.") {
                        *arg = if nullable {
                            "test.option.string".into()
                        } else {
                            STRING_TYPE_ID.into()
                        };
                    }
                }
                signatures.insert((operation.id.clone(), nullable), operation);
            }
        }
        let mut checked = 0;
        for ((id, nullable), operation) in signatures.into_iter().chain(frozen) {
            let mut legacy = Builder::new().unwrap();
            let expected = Helpers::new(&mut legacy, nullable)
                .unwrap()
                .emit(&mut legacy, operation.clone())
                .unwrap();
            let mut current = Builder::new().unwrap();
            let mut cache = ContractStringCache::default();
            let actual = cache
                .emit(&mut current, operation.clone(), nullable)
                .unwrap();
            assert_eq!(actual.0, expected);
            // Raw-check adapters only append terms and declarations. Before
            // final name-table sorting, the complete legacy DAG stays exact.
            assert_eq!(
                &current.c.term_table[..legacy.c.term_table.len()],
                &legacy.c.term_table
            );
            assert_eq!(
                &current.c.declarations[..legacy.c.declarations.len()],
                &legacy.c.declarations
            );
            assert_eq!(
                &current.c.name_table[..legacy.c.name_table.len()],
                &legacy.c.name_table
            );
            assert_eq!(current.c.level_table, legacy.c.level_table);
            let sizes = (current.c.term_table.len(), current.c.declarations.len());
            assert_eq!(
                cache
                    .emit(&mut current, operation.clone(), nullable)
                    .unwrap(),
                actual
            );
            assert_eq!(
                (current.c.term_table.len(), current.c.declarations.len()),
                sizes
            );
            assert!(matches!(
                cache.emit(&mut current, operation.clone(), !nullable),
                Err(OrdinaryCarrierError::Linkage)
            ));
            let mut changed = operation;
            changed.normal_result_type_id = "test.changed".into();
            assert!(
                matches!(
                    cache.emit(&mut current, changed, nullable),
                    Err(OrdinaryCarrierError::Linkage)
                ),
                "{id}"
            );
            current.finish().unwrap();
            checked += 1;
        }
        assert_eq!(checked, 57);
        eprintln!("string contract cache preserved {checked} legacy operation/nullable DAGs and rejected context/signature reuse");
    }
    #[test]
    fn string_contract_failures_are_raw_and_preserve_priority() {
        let text = |present| {
            V::Cube(physical(
                if present { Some(&[b'x' as u16]) } else { None },
                true,
            ))
        };
        let word = |n: i32| V::Cube((0..32).map(|i| (n as u32) & (1 << i) != 0).collect());
        let mut b = Builder::new().unwrap();
        let mut cache = ContractStringCache::default();
        let indexed = cache
            .emit(&mut b, signature("string.index", true), true)
            .unwrap();
        let check = |id: &str| {
            let c = check_contract(id).unwrap();
            RequiredCheck {
                id: id.into(),
                tag: c.tag,
                failure_type_id: match c.failure {
                    CheckFailureType::None => None,
                    CheckFailureType::Exact(t) => Some(t.into()),
                },
            }
        };
        let search = cache
            .emit(
                &mut b,
                ClosedOperationSignature {
                    id: "string.contains.ordinal".into(),
                    tag: ClosedOperationTag::Data,
                    argument_type_ids: vec!["test.option.string".into(); 2],
                    normal_result_type_id: BOOL_TYPE_ID.into(),
                    ordered_checks: vec![
                        check("exception.null_receiver"),
                        check("exception.null_argument"),
                    ],
                },
                true,
            )
            .unwrap();
        let concat = cache
            .emit(
                &mut b,
                ClosedOperationSignature {
                    id: "string.concat.string2".into(),
                    tag: ClosedOperationTag::Data,
                    argument_type_ids: vec!["test.option.string".into(); 2],
                    normal_result_type_id: STRING_TYPE_ID.into(),
                    ordered_checks: vec![check("obligation.output_bound")],
                },
                true,
            )
            .unwrap();
        let c = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
        let mut observations = 0;
        for present in [false, true] {
            for index in [-1, 0, 1] {
                let args = vec![text(present), word(index)];
                let bad = index < 0 || index >= i32::from(present);
                assert_eq!(bit(run(&c, &indexed.1[0], args.clone())), !present);
                assert_eq!(bit(run(&c, &indexed.1[1], args.clone())), bad);
                assert_eq!(
                    bit(run(&c, &indexed.0.ordered_failure_definitions[1], args)),
                    present && bad
                );
                observations += 3;
            }
        }
        for left in [false, true] {
            for right in [false, true] {
                let args = vec![text(left), text(right)];
                assert_eq!(bit(run(&c, &search.1[0], args.clone())), !left);
                assert_eq!(bit(run(&c, &search.1[1], args.clone())), !right);
                assert_eq!(
                    bit(run(&c, &search.0.ordered_failure_definitions[1], args)),
                    left && !right
                );
                observations += 3;
            }
        }
        for (left, right, failed) in [
            (Some(16383), Some(1), false),
            (Some(16384), Some(1), true),
            (None, Some(16384), false),
        ] {
            let input = |n: Option<usize>| {
                let units = n.map(|n| vec![0; n]);
                V::Cube(physical(units.as_deref(), true))
            };
            assert_eq!(
                bit(run(&c, &concat.1[0], vec![input(left), input(right)])),
                failed
            );
            observations += 1;
        }
        assert_eq!(observations, 33);
        eprintln!("string contract raw checks: {observations} null/range/priority/output-capacity observations");
    }
    // Independent physical encoding of the unit-1 sequence and option layouts.
    pub(super) fn physical(text: Option<&[u16]>, nullable: bool) -> Vec<bool> {
        let mut raw = vec![false; 1 << (TEXT_DEPTH + u32::from(nullable))];
        let Some(text) = text else {
            assert!(nullable);
            return raw;
        };
        let mut set = |i, value| {
            let i = if nullable { 1 + 2 * i } else { i };
            raw[i] = value;
        };
        for i in 0..32 {
            set(i << 14, text.len() & (1 << i) != 0);
        }
        for (i, &unit) in text.iter().enumerate() {
            for bit in 0..16 {
                set(1 + (i << 1) + (bit << 15), unit & (1 << bit) != 0);
            }
        }
        if nullable {
            raw[0] = true;
        }
        raw
    }
    #[test]
    fn string_basic_core_matches_utf16_oracle() {
        let maximum = (0..16384)
            .map(|i| (i as u16).wrapping_mul(4051))
            .collect::<Vec<_>>();
        let texts = [
            None,
            Some(vec![]),
            Some(vec![0, 0xd800, 0xffff, 0xdc00]),
            Some(maximum),
        ];
        let mut total = 0;
        for nullable in [false, true] {
            let mut b = Builder::new().unwrap();
            let mut helpers = Helpers::new(&mut b, nullable).unwrap();
            let definitions = BASIC
                .iter()
                .map(|id| helpers.emit(&mut b, signature(id, nullable)).unwrap())
                .collect::<Vec<_>>();
            let bytes = b.finish().unwrap();
            let cert = decode_canonical_certificate(&bytes).unwrap();
            crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(
                &cert,
            )
            .unwrap();
            for text in &texts {
                if !nullable && text.is_none() {
                    continue;
                }
                let physical = V::Cube(physical(text.as_deref(), nullable));
                for d in &definitions {
                    let len = text.as_ref().map_or(0, Vec::len) as i32;
                    let indices = if d.operation.id == "string.index" {
                        vec![i32::MIN, -1, 0, 1, len - 1, len, 16383, 16384, i32::MAX]
                    } else {
                        vec![0]
                    };
                    for index in indices {
                        let mut args = vec![physical.clone()];
                        let mut operands = vec![StringOperand::Text {
                            utf16: text.clone(),
                        }];
                        if d.operation.id == "string.index" {
                            args.push(V::Cube(
                                (0..32).map(|i| (index as u32) & (1 << i) != 0).collect(),
                            ));
                            operands.push(StringOperand::Index { value: index });
                        }
                        let expected = evaluate_string_operation(&d.operation.id, &operands, false);
                        assert_eq!(
                            bit(run(&cert, &d.success_definition, args.clone())),
                            expected.is_ok(),
                            "{} {nullable} {len} {index}",
                            d.operation.id
                        );
                        for (i, name) in d.ordered_failure_definitions.iter().enumerate() {
                            let flag = matches!(
                                (&expected, i),
                                (Err(StringError::NullReceiver), 0)
                                    | (Err(StringError::IndexOutOfRange), 1)
                            );
                            assert_eq!(bit(run(&cert, name, args.clone())), flag);
                        }
                        let width = match d.operation.normal_result_type_id.as_str() {
                            BOOL_TYPE_ID => 1,
                            I32_TYPE_ID => 32,
                            _ => 16,
                        };
                        let expected = match expected {
                            Ok(MonomorphicValue::Bool { value, .. }) => u32::from(value),
                            Ok(MonomorphicValue::Signed { value, .. }) => {
                                value.parse::<i32>().unwrap() as u32
                            }
                            Ok(MonomorphicValue::Char { utf16, .. }) => u32::from(utf16),
                            Err(_) => 0,
                            other => panic!("unexpected string result {other:?}"),
                        };
                        let result = run(&cert, &d.result_definition, args);
                        for i in 0..width {
                            let mut value = result.clone();
                            for j in 0..address_bits(width) {
                                value = apply(&cert, value, V::Bit(i & (1 << j) != 0));
                            }
                            assert_eq!(
                                bit(value),
                                expected & (1 << i) != 0,
                                "{} {nullable} {len} {index} bit {i}",
                                d.operation.id
                            );
                        }
                        total += 1;
                    }
                }
            }
        }
        assert_eq!(total, 77);
    }
}
