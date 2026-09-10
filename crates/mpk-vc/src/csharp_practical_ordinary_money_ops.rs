//! Concrete Money operations. The application currency predicate is an explicit
//! ordinary parameter, never an invented whitelist or an admitted source proof.
use super::super::super::scalar_bits::emit_decimal;
use super::*;

const DECIMAL: &str = "mpk.csharp.value.decimal.v1";
const U32: &str = "mpk.csharp.value.u32.v1";
const TEMPLATE: &str = "mpk.csharp.semantic.money.v1";
const MODES: [&str; 5] = [
    "ToEven",
    "AwayFromZero",
    "ToZero",
    "ToNegativeInfinity",
    "ToPositiveInfinity",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryMoneyFailure {
    pub label: String,
    pub definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryMoneyOperation {
    pub operation_id: String,
    pub argument_type_ids: Vec<String>,
    pub result_type_id: String,
    /// When present, every definition first binds (this concrete carrier -> Bool).
    /// Assembly must supply the proven source currency predicate before admission.
    pub currency_predicate_argument_type_id: Option<String>,
    pub normal_definition: String,
    pub failures: Vec<OrdinaryMoneyFailure>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryMoneyDefinition {
    pub carrier: OrdinaryCarrier,
    pub currency_type_id: String,
    pub operations: Vec<OrdinaryMoneyOperation>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryMoneyProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryMoneyDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryMoneyProgram {
    pub fn definitions(&self) -> &[OrdinaryMoneyDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed money program")
    }
}

fn neg(b: &mut Builder, v: u32) -> R<u32> {
    let no = bit(b, false)?;
    let yes = bit(b, true)?;
    mux(b, v, no, yes)
}
fn exceeds(b: &mut Builder, v: u32, maximum: u32) -> R<u32> {
    // Scale is i32: interpreting its complete word as unsigned also rejects
    // every negative value when comparing it against 28.
    let limit = ordered_fold::word(b, maximum)?;
    ordered_fold::helper(b, "Less", vec![limit, v])
}
fn define_money(
    b: &mut Builder,
    name: &str,
    predicate: Option<u32>,
    args: &[u32],
    output: u32,
    mut body: u32,
) -> R<()> {
    let mut ty = b.cube(output)?;
    for depth in args.iter().rev() {
        let arg = b.cube(*depth)?;
        body = b.lam(arg, body)?;
        ty = b.pi(arg, ty)?;
    }
    if let Some(depth) = predicate {
        let arg = b.cube(depth)?;
        let pred = b.pi(arg, b.boolean)?;
        body = b.lam(pred, body)?;
        ty = b.pi(pred, ty)?;
    }
    b.define(name, ty, body)
}

fn expected(id: &str, currency: &str) -> Value {
    let row = |op: &str, args: Vec<&str>, result: &str, equation: &str, errors: Vec<&str>| json!({"id":format!("{id}.{op}"),"argument_type_ids":args,"normal_result_type_id":result,"equation":equation,"error_precedence":errors});
    json!([
        row(
            "create",
            vec![DECIMAL, currency, I32_TYPE_ID],
            id,
            "validate_currency_scale_and_exact_amount(amount,currency,scale)",
            vec!["invalid_currency", "invalid_scale", "invalid_precision"]
        ),
        row("amount", vec![id], DECIMAL, "field(x,0)", vec![]),
        row("currency", vec![id], currency, "field(x,1)", vec![]),
        row(
            "add",
            vec![id, id],
            id,
            "same_currency_checked_decimal_add(x,y)",
            vec!["currency_mismatch", "decimal_overflow"]
        ),
        row(
            "subtract",
            vec![id, id],
            id,
            "same_currency_checked_decimal_subtract(x,y)",
            vec!["currency_mismatch", "decimal_overflow"]
        ),
        row(
            "multiply",
            vec![id, DECIMAL, I32_TYPE_ID, U32],
            id,
            "checked_decimal_product_then_explicit_round(x,q,scale,mode)",
            vec!["invalid_scale", "invalid_rounding", "decimal_overflow"]
        ),
        row(
            "divide",
            vec![id, DECIMAL, I32_TYPE_ID, U32],
            id,
            "checked_decimal_quotient_then_explicit_round(x,q,scale,mode)",
            vec![
                "invalid_scale",
                "invalid_rounding",
                "division_by_zero",
                "decimal_overflow"
            ]
        ),
        row(
            "amount_compare",
            vec![id, id],
            I32_TYPE_ID,
            "same_currency_decimal_compare(x,y)",
            vec!["currency_mismatch"]
        ),
        row(
            "equal",
            vec![id, id],
            BOOL_TYPE_ID,
            "currency_equal_and_decimal_value_equal(x,y)",
            vec![]
        ),
        row(
            "compare",
            vec![id, id],
            I32_TYPE_ID,
            "currency_first_then_decimal_value_compare(x,y)",
            vec![]
        )
    ])
}

struct Money<'r, 'v> {
    r: &'r mut Relations<'v>,
    decimal: BTreeMap<String, OrdinaryScalarDefinition>,
}
impl Money<'_, '_> {
    fn decimal(&mut self, op: &str) -> R<OrdinaryScalarDefinition> {
        let id = format!("decimal.{op}");
        if let Some(d) = self.decimal.get(&id) {
            return Ok(d.clone());
        }
        let d = emit_decimal(&mut self.r.b, &id)?;
        self.decimal.insert(id, d.clone());
        Ok(d)
    }
    fn round(&mut self) -> R<String> {
        let name = format!("{PREFIX}.Money.RoundSelected");
        if self.r.b.globals.contains_key(&name) {
            return Ok(name);
        }
        let definitions = MODES
            .iter()
            .map(|mode| self.decimal(&format!("round.{mode}.2")))
            .collect::<R<Vec<_>>>()?;
        if !self
            .r
            .b
            .globals
            .contains_key(&format!("{PREFIX}.Cube.D9.Mux"))
        {
            self.r.b.helpers(9)?;
        }
        let amount = self.r.b.var(2)?;
        let scale = self.r.b.var(1)?;
        let mode = self.r.b.var(0)?;
        let mut value = self.r.b.constant(&format!("{PREFIX}.Cube.D9.Zero"))?;
        for (i, d) in definitions.iter().enumerate().rev() {
            let tag = ordered_fold::word(&mut self.r.b, i as u32)?;
            let below = ordered_fold::helper(&mut self.r.b, "Less", vec![mode, tag])?;
            let above = ordered_fold::helper(&mut self.r.b, "Less", vec![tag, mode])?;
            let not_above = neg(&mut self.r.b, above)?;
            let no = bit(&mut self.r.b, false)?;
            let enabled = mux(&mut self.r.b, below, no, not_above)?;
            let rounded = call(&mut self.r.b, &d.result_definition, vec![amount, scale])?;
            value = ordered_fold::cube_mux(&mut self.r.b, 9, enabled, rounded, value)?;
        }
        define(&mut self.r.b, &name, &[9, 5, 5], 9, value)?;
        Ok(name)
    }
    fn emit(&mut self, entry: &Value) -> R<OrdinaryMoneyDefinition> {
        let id = text(entry, "instance_id")?;
        let metadata = self
            .r
            .vir
            .data_closed()
            .metadata
            .get(id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if metadata.argument_ids.len() != 1 || !metadata.dependency_ids.is_empty() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let currency = metadata.argument_ids[0].clone();
        let carrier = self
            .r
            .carriers
            .get(id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        if carrier.shape
            != product(vec![
                field("amount", reference(DECIMAL)),
                field("currency", reference(&currency)),
            ])
            || entry["operation_definitions"] != expected(id, &currency)
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let money_relation = self.r.ty(id)?;
        let decimal_relation = self.r.ty(DECIMAL)?;
        let currency_relation = self.r.ty(&currency)?;
        let currency_depth = self.r.carriers[&currency].depth;
        if self.r.carriers[DECIMAL].depth != 9 {
            return Err(OrdinaryCarrierError::Shape);
        }
        let references = self
            .r
            .carriers
            .iter()
            .map(|(id, c)| (id.as_str(), c))
            .collect();
        let storage = self
            .r
            .storage
            .get(&mut self.r.b, &carrier, &references)?
            .ok_or(OrdinaryCarrierError::Shape)?;
        let OrdinaryStructuralOperations::Product { operations: fields } = storage.operations
        else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if fields.fields.len() != 2
            || fields.fields[0].field_id != "amount"
            || fields.fields[1].field_id != "currency"
        {
            return Err(OrdinaryCarrierError::Shape);
        }
        let amount_get = &fields.fields[0].definition;
        let currency_get = &fields.fields[1].definition;
        let mut operations = vec![];
        for operation in array(entry, "operation_definitions")? {
            let operation_id = text(operation, "id")?;
            let op = operation_id
                .strip_prefix(&format!("{id}."))
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let argument_type_ids = array(operation, "argument_type_ids")?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or(OrdinaryCarrierError::Linkage)
                })
                .collect::<R<Vec<_>>>()?;
            let args = (0..argument_type_ids.len())
                .rev()
                .map(|i| self.r.b.var(i as u32))
                .collect::<R<Vec<_>>>()?;
            let depths = argument_type_ids
                .iter()
                .map(|t| {
                    self.r
                        .carriers
                        .get(t)
                        .map(|c| c.depth)
                        .or_else(|| matches!(t.as_str(), I32_TYPE_ID | U32).then_some(5))
                        .ok_or(OrdinaryCarrierError::Linkage)
                })
                .collect::<R<Vec<_>>>()?;
            let result_type_id = text(operation, "normal_result_type_id")?.to_owned();
            let output = match result_type_id.as_str() {
                BOOL_TYPE_ID => 0,
                I32_TYPE_ID => 5,
                _ => {
                    self.r
                        .carriers
                        .get(&result_type_id)
                        .ok_or(OrdinaryCarrierError::Linkage)?
                        .depth
                }
            };
            let mut failures = vec![];
            let normal = match op {
                "create" => {
                    let rounded = self.decimal("round.ToZero.2")?;
                    let predicate = self.r.b.var(3)?;
                    let allowed = self.r.b.app(predicate, vec![args[1]])?;
                    failures.push(neg(&mut self.r.b, allowed)?);
                    failures.push(exceeds(&mut self.r.b, args[2], 28)?);
                    let rounded = call(
                        &mut self.r.b,
                        &rounded.result_definition,
                        vec![args[0], args[2]],
                    )?;
                    let precise = call(
                        &mut self.r.b,
                        &decimal_relation.equal,
                        vec![args[0], rounded],
                    )?;
                    failures.push(neg(&mut self.r.b, precise)?);
                    call(
                        &mut self.r.b,
                        &fields.make_definition,
                        vec![args[0], args[1]],
                    )?
                }
                "amount" => call(&mut self.r.b, amount_get, args.clone())?,
                "currency" => call(&mut self.r.b, currency_get, args.clone())?,
                "equal" => call(&mut self.r.b, &money_relation.equal, args.clone())?,
                "compare" => call(
                    &mut self.r.b,
                    money_relation
                        .compare
                        .as_ref()
                        .ok_or(OrdinaryCarrierError::Shape)?,
                    args.clone(),
                )?,
                "add" | "subtract" | "amount_compare" => {
                    let left_amount = call(&mut self.r.b, amount_get, vec![args[0]])?;
                    let right_amount = call(&mut self.r.b, amount_get, vec![args[1]])?;
                    let left_currency = call(&mut self.r.b, currency_get, vec![args[0]])?;
                    let right_currency = call(&mut self.r.b, currency_get, vec![args[1]])?;
                    let equal = call(
                        &mut self.r.b,
                        &currency_relation.equal,
                        vec![left_currency, right_currency],
                    )?;
                    failures.push(neg(&mut self.r.b, equal)?);
                    if op == "amount_compare" {
                        call(
                            &mut self.r.b,
                            decimal_relation
                                .compare
                                .as_ref()
                                .ok_or(OrdinaryCarrierError::Shape)?,
                            vec![left_amount, right_amount],
                        )?
                    } else {
                        let d = self.decimal(op)?;
                        if d.operation.ordered_checks.len() != 1
                            || d.operation.ordered_checks[0].id != "exception.overflow"
                        {
                            return Err(OrdinaryCarrierError::Linkage);
                        }
                        failures.push(call(
                            &mut self.r.b,
                            &d.ordered_failure_definitions[0],
                            vec![left_amount, right_amount],
                        )?);
                        let result = call(
                            &mut self.r.b,
                            &d.result_definition,
                            vec![left_amount, right_amount],
                        )?;
                        call(
                            &mut self.r.b,
                            &fields.make_definition,
                            vec![result, left_currency],
                        )?
                    }
                }
                "multiply" | "divide" => {
                    let d = self.decimal(op)?;
                    let rounding = self.round()?;
                    let amount = call(&mut self.r.b, amount_get, vec![args[0]])?;
                    let currency = call(&mut self.r.b, currency_get, vec![args[0]])?;
                    failures.push(exceeds(&mut self.r.b, args[2], 28)?);
                    failures.push(exceeds(&mut self.r.b, args[3], 4)?);
                    let labels = if op == "divide" {
                        vec!["exception.division_by_zero", "exception.overflow"]
                    } else {
                        vec!["exception.overflow"]
                    };
                    if d.operation
                        .ordered_checks
                        .iter()
                        .map(|f| f.id.as_str())
                        .collect::<Vec<_>>()
                        != labels
                    {
                        return Err(OrdinaryCarrierError::Linkage);
                    }
                    for failure in &d.ordered_failure_definitions {
                        failures.push(call(&mut self.r.b, failure, vec![amount, args[1]])?);
                    }
                    let result = call(&mut self.r.b, &d.result_definition, vec![amount, args[1]])?;
                    let rounded = call(&mut self.r.b, &rounding, vec![result, args[2], args[3]])?;
                    call(
                        &mut self.r.b,
                        &fields.make_definition,
                        vec![rounded, currency],
                    )?
                }
                _ => return Err(OrdinaryCarrierError::Linkage),
            };
            let labels = array(operation, "error_precedence")?;
            if labels.len() != failures.len() {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let prefix = format!(
                "{PREFIX}.Money.O{}",
                operation_id
                    .as_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            );
            let normal_definition = format!("{prefix}.Normal");
            let predicate = (op == "create").then_some(currency_depth);
            define_money(
                &mut self.r.b,
                &normal_definition,
                predicate,
                &depths,
                output,
                normal,
            )?;
            let mut named_failures = vec![];
            for (i, (label, body)) in labels.iter().zip(failures).enumerate() {
                let definition = format!("{prefix}.Failure.F{i}");
                define_money(&mut self.r.b, &definition, predicate, &depths, 0, body)?;
                named_failures.push(OrdinaryMoneyFailure {
                    label: label.as_str().ok_or(OrdinaryCarrierError::Linkage)?.into(),
                    definition,
                });
            }
            operations.push(OrdinaryMoneyOperation {
                operation_id: operation_id.into(),
                argument_type_ids,
                result_type_id,
                currency_predicate_argument_type_id: predicate.map(|_| currency.clone()),
                normal_definition,
                failures: named_failures,
            });
        }
        Ok(OrdinaryMoneyDefinition {
            carrier,
            currency_type_id: currency,
            operations,
        })
    }
}

// Reuse the owning program's relation/storage caches and counted pipeline.
// The currency predicate remains an explicit source-proof obligation.
pub(super) fn emit_money(r: &mut Relations<'_>) -> R<Vec<OrdinaryMoneyDefinition>> {
    let vir = r.vir;
    let mut money = Money {
        r,
        decimal: BTreeMap::new(),
    };
    vir.data_closed()
        .entries()
        .iter()
        .filter(|e| e["template_id"] == TEMPLATE)
        .map(|entry| money.emit(entry))
        .collect()
}

pub fn generate_csharp_practical_ordinary_money(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryMoneyProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut r = Relations {
        vir,
        shared_folds: true,
        observations: false,
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect(),
        b: Builder::new()?,
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
        raw: BTreeMap::new(),
        special: BTreeMap::new(),
        storage: StorageCache::default(),
    };
    r.b.helpers(5)?;
    ordered_fold::auxiliary(&mut r.b)?;
    let definitions = emit_money(&mut r)?;
    let static_transformers = r.b.static_transformers;
    let certificate = r.b.finish()?;
    let p = OrdinaryMoneyProgram {
        schema: "mpk.csharp.ordinary_money.v1".into(),
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
pub fn import_csharp_practical_ordinary_money(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryMoneyProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_money(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
