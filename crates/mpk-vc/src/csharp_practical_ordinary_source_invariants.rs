//! Construction public-body equations and the original W06 source-invariant
//! conditions. Domain definitions do not discharge their universal proofs.
use super::super::super::source_clauses;
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinarySourceInvariantEnumCase {
    pub value: String,
    pub symbol: String,
    pub definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinarySourceInvariantDefinition {
    pub invariant: ConstructionTypeInvariant,
    /// SourceShape uses the independent representation-only domain. Child
    /// PublicDomain tests and the declared clauses remain in public_body.
    pub representation_domain: String,
    pub member_reads: Vec<OrdinaryProjection>,
    pub enum_cases: Vec<OrdinarySourceInvariantEnumCase>,
    pub clause_definitions: Vec<String>,
    pub body_definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinarySourceInvariantProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    construction_sha256: String,
    binding_vc_sha256: String,
    source_clauses: Vec<OrdinarySourceClauseDefinition>,
    representation_domains: Vec<OrdinaryDomainDefinition>,
    public_domains: Vec<OrdinaryPublicDomainDefinition>,
    definitions: Vec<OrdinarySourceInvariantDefinition>,
    conditions: Vec<OrdinaryBindingCondition>,
    pending_condition_ids: Vec<String>,
    pending_proof_ids: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinarySourceInvariantProgram {
    pub fn definitions(&self) -> &[OrdinarySourceInvariantDefinition] {
        &self.definitions
    }
    pub fn representation_domains(&self) -> &[OrdinaryDomainDefinition] {
        &self.representation_domains
    }
    pub fn public_domains(&self) -> &[OrdinaryPublicDomainDefinition] {
        &self.public_domains
    }
    pub fn conditions(&self) -> &[OrdinaryBindingCondition] {
        &self.conditions
    }
    pub fn pending_condition_ids(&self) -> &[String] {
        &self.pending_condition_ids
    }
    pub fn pending_proof_ids(&self) -> &[String] {
        &self.pending_proof_ids
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary source invariants")
    }
}

fn emit_enum_case(
    b: &mut Builder,
    carrier: &OrdinaryCarrier,
    value: &str,
) -> R<OrdinarySourceInvariantEnumCase> {
    let OrdinaryShape::Bits { width } = carrier.shape else {
        return Err(OrdinaryCarrierError::Shape);
    };
    if !matches!(width, 8 | 16 | 32 | 64) || address_bits(width) != carrier.depth {
        return Err(OrdinaryCarrierError::Shape);
    }
    // Canonical spelling, signedness and range are validated by the source
    // importer. Compare the exact underlying two's-complement representation.
    let number = parse_canonical_integer(value).ok_or(OrdinaryCarrierError::Shape)? as u128;
    let symbol = format!("Mpk.CSharp.EnumCarrierEquals.{}.{value}", carrier.type_id);
    let definition = name(&symbol);
    let mut equal = bit(b, true)?;
    for i in 0..width {
        let receiver = b.var(0)?;
        let address = prefix(carrier.depth, i)
            .into_iter()
            .map(|v| bit(b, v))
            .collect::<R<Vec<_>>>()?;
        let x = b.app(receiver, address)?;
        let same = if number & (1 << i) != 0 {
            x
        } else {
            let yes = bit(b, true)?;
            let no = bit(b, false)?;
            mux(b, x, no, yes)?
        };
        equal = and(b, equal, same)?;
    }
    define(b, &definition, &[carrier.depth], 0, equal)?;
    Ok(OrdinarySourceInvariantEnumCase {
        value: value.into(),
        symbol,
        definition,
    })
}

pub fn generate_csharp_practical_ordinary_source_invariants(
    vir: &ValidatedPracticalVir,
) -> R<OrdinarySourceInvariantProgram> {
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (b, storage, cache, source_clauses, construction_sha256) =
        source_clauses::emit_clauses(vir, &layouts, Builder::new()?)?;
    let mut r = Relations {
        vir,
        shared_folds: true,
        observations: false,
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect(),
        b,
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
        raw: BTreeMap::new(),
        special: BTreeMap::new(),
        storage,
    };
    cache.seed(&mut r)?;
    let (r, representation_domains) =
        super::super::domains::emit_representation_domains(r, layouts.carriers())?;
    // The existing public-domain emitter retains its separate W03 definedness
    // requirement for partial clauses. It never assumes that requirement true.
    let (mut r, public_domains) =
        super::super::domains::emit_binding_domains(r, layouts.carriers(), &source_clauses)?;
    let mut symbols = public_domains
        .iter()
        .map(|d| (d.symbol.clone(), d.valid_definition.clone()))
        .collect::<BTreeMap<_, _>>();
    conditions::boolean_symbols(&mut symbols);
    let equality = r.raw(1, false)?.equal;
    symbols.insert(
        "Mpk.CSharp.Binding.Equal.mpk.csharp.value.bool.v1".into(),
        equality,
    );
    let refs = layouts
        .carriers()
        .iter()
        .map(|c| (c.type_id.as_str(), c))
        .collect::<BTreeMap<_, _>>();
    let mut definitions = vec![];
    for invariant in construction.types() {
        let carrier = refs
            .get(invariant.type_id.as_str())
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let domain = representation_domains
            .iter()
            .find(|d| d.carrier == **carrier)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        symbols.insert(
            format!("Mpk.CSharp.SourceShape.{}", invariant.type_id),
            domain.valid_definition.clone(),
        );
        let mut member_reads = vec![];
        let mut enum_cases = vec![];
        if let Some(values) = &invariant.enum_values {
            if !invariant.members.is_empty() {
                return Err(OrdinaryCarrierError::Linkage);
            }
            for value in values {
                let case = emit_enum_case(&mut r.b, carrier, value)?;
                symbols.insert(case.symbol.clone(), case.definition.clone());
                enum_cases.push(case);
            }
        } else {
            let OrdinaryShape::Product { fields } = &carrier.shape else {
                return Err(OrdinaryCarrierError::Shape);
            };
            if fields.len() != invariant.members.len() {
                return Err(OrdinaryCarrierError::Linkage);
            }
            for member in &invariant.members {
                let (index, field) = fields
                    .iter()
                    .enumerate()
                    .find(|(_, f)| f.id == member.id)
                    .ok_or(OrdinaryCarrierError::Linkage)?;
                if reference(&field.shape)? != member.type_id {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let depth = refs
                    .get(member.type_id.as_str())
                    .ok_or(OrdinaryCarrierError::Linkage)?
                    .depth;
                let selectors = carrier
                    .depth
                    .checked_sub(depth)
                    .ok_or(OrdinaryCarrierError::Shape)?;
                let mut address = prefix(address_bits(fields.len() as u32), index as u32);
                if address.len() > selectors as usize {
                    return Err(OrdinaryCarrierError::Shape);
                }
                address.resize(selectors as usize, false);
                // Return the selected member cube directly. Eta-expanding all
                // remaining selectors hides shared constant subcubes and makes
                // deep inactive-storage checks repeat the same observations.
                // Source/member identity and every selector stay explicit.
                let definition = name(&format!(
                    "source_invariant.{}.member.{}",
                    invariant.type_id, member.id
                ));
                let definition = r.getter(&definition, carrier.depth, depth, &address)?;
                let get = OrdinaryProjection {
                    field_id: member.id.clone(),
                    depth,
                    definition,
                };
                symbols.insert(format!("field.read.{}", member.id), get.definition.clone());
                member_reads.push(get);
            }
        }
        let mut fragments = vec![];
        for expression in &invariant.public_clauses {
            let clause = source_clauses
                .iter()
                .find(|c| {
                    c.source_type_id == invariant.type_id
                        && c.source_sha256 == invariant.source_sha256
                        && c.attachment_sha256 == expression.attachment_sha256()
                        && c.expression_sha256 == expression.expression_sha256()
                })
                .ok_or(OrdinaryCarrierError::Linkage)?;
            if clause.argument_types != [invariant.type_id.clone()]
                || clause.definedness_definition.is_some()
            {
                return Err(OrdinaryCarrierError::Linkage);
            }
            fragments.push((expression.term().clone(), clause.definition.clone()));
        }
        let body_definition = name(&format!("source_invariant.{}.body", invariant.type_id));
        let subject = TypedValueRef {
            id: "source_invariant.receiver".into(),
            type_id: invariant.type_id.clone(),
        };
        let body = reconstruction::term(
            &mut r.b,
            &invariant.public_body,
            &subject,
            &symbols,
            &fragments,
        )?;
        define(&mut r.b, &body_definition, &[carrier.depth], 0, body)?;
        definitions.push(OrdinarySourceInvariantDefinition {
            invariant: invariant.clone(),
            representation_domain: domain.valid_definition.clone(),
            member_reads,
            enum_cases,
            clause_definitions: fragments.into_iter().map(|(_, d)| d).collect(),
            body_definition,
        });
    }
    let mut conditions = vec![];
    let mut pending_condition_ids = vec![];
    for sequent in vc.sequents() {
        if sequent.kind != "source_invariant" {
            pending_condition_ids.push(sequent.id.clone());
            continue;
        }
        let [subject] = sequent.subjects.as_slice() else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let definition = definitions
            .iter()
            .find(|d| d.invariant.type_id == subject.type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        conditions.push(reconstruction::obligation_with_fragments(
            &mut r,
            sequent,
            &symbols,
            "source_invariant",
            &[(
                definition.invariant.public_body.clone(),
                definition.body_definition.clone(),
            )],
        )?);
    }
    let pending_proof_ids = vc.sequents().iter().map(|s| s.id.clone()).collect();
    let static_transformers = r.b.static_transformers;
    let certificate = r.b.finish()?;
    let p = OrdinarySourceInvariantProgram {
        schema: "mpk.csharp.ordinary_source_invariants.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        construction_sha256,
        binding_vc_sha256: vc.hash(),
        source_clauses,
        representation_domains,
        public_domains,
        definitions,
        conditions,
        pending_condition_ids,
        pending_proof_ids,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}

pub fn import_csharp_practical_ordinary_source_invariants(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinarySourceInvariantProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_source_invariants(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

#[cfg(test)]
mod tests {
    use super::super::super::super::super::tests::{bit as observed_bit, run, V};
    use super::*;

    #[test]
    fn csharp_03_t06_w09_source_invariant_enum_underlying_bits() {
        let mut b = Builder::new().unwrap();
        let mut rows = vec![];
        for width in [8, 16, 32, 64] {
            for signed in [false, true] {
                let carrier = OrdinaryCarrier {
                    type_id: format!("test.enum.{width}.{signed}"),
                    depth: address_bits(width),
                    shape: OrdinaryShape::Bits { width },
                };
                let (min, max) = if signed {
                    (-(1i128 << (width - 1)), (1i128 << (width - 1)) - 1)
                } else {
                    (0, (1i128 << width) - 1)
                };
                let mut declared = vec![min, 0, 7, max];
                declared.sort();
                declared.dedup();
                let cases = declared
                    .iter()
                    .map(|n| emit_enum_case(&mut b, &carrier, &n.to_string()).unwrap())
                    .collect::<Vec<_>>();
                rows.push((carrier, declared, cases));
            }
        }
        let bytes = b.finish().unwrap();
        let certificate = decode_canonical_certificate(&bytes).unwrap();
        let mut observations = 0;
        for (carrier, declared, cases) in &rows {
            let OrdinaryShape::Bits { width } = carrier.shape else {
                panic!()
            };
            for (n, case) in declared.iter().zip(cases) {
                // Flip every physical bit as well as testing the exact arm.
                // This detects a skipped bit, reversed address or signed cast.
                for flip in std::iter::once(None).chain((0..width).map(Some)) {
                    let bits = *n as u128 ^ flip.map_or(0, |i| 1 << i);
                    let value = V::Cube((0..width).map(|i| bits & (1 << i) != 0).collect());
                    assert_eq!(
                        observed_bit(run(&certificate, &case.definition, vec![value])),
                        flip.is_none(),
                        "{}:{n}:{flip:?}",
                        carrier.type_id
                    );
                    observations += 1;
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/source-invariant-enums");
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        if let Some(out) = std::env::var_os("MPK_W09_SOURCE_INVARIANT_ENUMS_OUT") {
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(std::path::Path::new(&out).join("enum-cases.hex"), hex).unwrap();
        } else {
            assert_eq!(
                std::fs::read_to_string(root.join("enum-cases.hex")).unwrap(),
                hex
            );
        }
        assert_eq!(observations, 868);
        eprintln!("source invariant enum underlying bits: {observations} observations");
    }
}
