//! Recursive representation membership and all compiled source public clauses.
//! This defines domain predicates, not construction or application proofs.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryPublicDomainDefinition {
    pub carrier: OrdinaryCarrier,
    /// Symbol in reconstructed construction/boundary/transition VC equations.
    pub symbol: String,
    /// Exact logical cells or 65,537 for any representation/public-clause failure.
    pub count_definition: String,
    pub valid_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryPublicDefaultDefinition {
    pub default: OrdinaryDefaultDefinition,
    /// Closed Bool: declared admission and public membership of the actual default.
    /// This is a definition, not an application proof of the Bool's truth.
    pub valid_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryPublicDomainProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    construction_sha256: String,
    source_clauses: Vec<OrdinarySourceClauseDefinition>,
    definitions: Vec<OrdinaryPublicDomainDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    public_defaults: Option<Vec<OrdinaryPublicDefaultDefinition>>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryPublicDomainProgram {
    pub fn definitions(&self) -> &[OrdinaryPublicDomainDefinition] {
        &self.definitions
    }
    pub fn source_clauses(&self) -> &[OrdinarySourceClauseDefinition] {
        &self.source_clauses
    }
    pub fn public_defaults(&self) -> Option<&[OrdinaryPublicDefaultDefinition]> {
        self.public_defaults.as_deref()
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("public domain program")
    }
}
pub(super) fn clauses_by_type(
    clauses: &[OrdinarySourceClauseDefinition],
) -> R<BTreeMap<String, Vec<String>>> {
    let mut by_type: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for c in clauses {
        // W03 definedness must be discharged at the original use point. Until
        // that owner is connected, do not admit a zeroed failure result or
        // silently strengthen PublicDomain by assuming the separate condition.
        if c.argument_types != [c.source_type_id.clone()] || c.definedness_definition.is_some() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        by_type
            .entry(c.source_type_id.clone())
            .or_default()
            .push(c.definition.clone());
    }
    Ok(by_type)
}
pub fn generate_csharp_practical_ordinary_public_domains(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryPublicDomainProgram> {
    generate(vir, false)
}

pub fn generate_csharp_practical_ordinary_public_defaults(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryPublicDomainProgram> {
    generate(vir, true)
}

fn generate(vir: &ValidatedPracticalVir, with_defaults: bool) -> R<OrdinaryPublicDomainProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let (builder, storage, relation_cache, source_clauses, construction_sha256) =
        super::super::super::source_clauses::emit_clauses(vir, &layouts, Builder::new()?)?;
    let mut d = Domains {
        public_clauses: Some(clauses_by_type(&source_clauses)?),
        r: Relations {
            vir,
            shared_folds: true,
            observations: false,
            carriers: layouts
                .carriers()
                .iter()
                .map(|c| (c.type_id.clone(), c.clone()))
                .collect(),
            b: builder,
            nodes: BTreeMap::new(),
            active: BTreeSet::new(),
            raw: BTreeMap::new(),
            special: BTreeMap::new(),
            storage,
        },
        counts: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    relation_cache.seed(&mut d.r)?;
    if !d.r.b.globals.contains_key(&format!("{PREFIX}.Cube.D5.Mux")) {
        d.r.b.helpers(5)?;
    }
    ordered_fold::auxiliary(&mut d.r.b)?;
    let definitions = emit_membership(&mut d, layouts.carriers())?;
    let public_defaults = if with_defaults {
        let defaults = super::super::super::super::defaults::emit_defaults(
            vir,
            layouts.carriers(),
            &mut d.r.b,
        )?;
        Some(emit_default_predicates(&mut d, defaults, &definitions)?)
    } else {
        None
    };
    let static_transformers = d.r.b.static_transformers;
    let certificate = d.r.b.finish()?;
    let p = OrdinaryPublicDomainProgram {
        schema: "mpk.csharp.ordinary_public_domains.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        construction_sha256,
        source_clauses,
        definitions,
        public_defaults,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_public_domains(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryPublicDomainProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_public_domains(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

pub fn import_csharp_practical_ordinary_public_defaults(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryPublicDomainProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_public_defaults(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

pub(super) fn emit_membership(
    d: &mut Domains<'_>,
    carriers: &[OrdinaryCarrier],
) -> R<Vec<OrdinaryPublicDomainDefinition>> {
    let mut definitions = vec![];
    for carrier in carriers {
        if d.r.internal(&carrier.type_id) {
            continue;
        }
        let count = d.ty(&carrier.type_id)?;
        let value = d.r.b.var(0)?;
        let cells = call(&mut d.r.b, &count.definition, vec![value])?;
        let valid = valid_count(&mut d.r.b, cells)?;
        let valid_definition = format!("{}.Valid", d.type_name(&carrier.type_id));
        define(&mut d.r.b, &valid_definition, &[carrier.depth], 0, valid)?;
        definitions.push(OrdinaryPublicDomainDefinition {
            carrier: carrier.clone(),
            symbol: format!("Mpk.CSharp.PublicDomain.{}", carrier.type_id),
            count_definition: count.definition,
            valid_definition,
        });
    }
    Ok(definitions)
}
pub(super) fn emit_default_predicates(
    d: &mut Domains<'_>,
    defaults: Vec<OrdinaryDefaultDefinition>,
    definitions: &[OrdinaryPublicDomainDefinition],
) -> R<Vec<OrdinaryPublicDefaultDefinition>> {
    let mut rows = vec![];
    for default in defaults {
        let Some(domain) = definitions
            .iter()
            .find(|p| p.carrier.type_id == default.carrier.type_id)
        else {
            if d.r.internal(&default.carrier.type_id) {
                continue;
            }
            return Err(OrdinaryCarrierError::Linkage);
        };
        let valid = if let Some(candidate) = &default.structural_candidate {
            let value = d.r.b.constant(&candidate.definition)?;
            let member = call(&mut d.r.b, &domain.valid_definition, vec![value])?;
            let declared = d.r.b.constant(&default.public_admission_definition)?;
            and(&mut d.r.b, declared, member)?
        } else {
            bit(&mut d.r.b, false)?
        };
        let valid_definition = format!("{}.Default.Valid", d.type_name(&default.carrier.type_id));
        define(&mut d.r.b, &valid_definition, &[], 0, valid)?;
        rows.push(OrdinaryPublicDefaultDefinition {
            default,
            valid_definition,
        });
    }
    Ok(rows)
}
