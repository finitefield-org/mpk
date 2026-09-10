//! Concrete CLR-default candidates and declaration-level admission.
//! Source default/invariant requirements remain pending application obligations.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinarySourceDefaultRequirement {
    pub source_type_id: String,
    pub source_sha256: String,
    pub declared_public_default: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDefaultCandidate {
    pub definition: String,
    pub logical_cells: u64,
    /// Unique source types on the actual default path, not inactive payloads.
    /// None of these source conditions is discharged by this program.
    pub source_requirements: Vec<OrdinarySourceDefaultRequirement>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDefaultDefinition {
    pub carrier: OrdinaryCarrier,
    pub structural_candidate: Option<OrdinaryDefaultCandidate>,
    /// A declaration/admission flag, not a source-invariant proof.
    pub public_candidate_admitted: bool,
    pub public_admission_definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryDefaultProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryDefaultDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryDefaultProgram {
    pub fn definitions(&self) -> &[OrdinaryDefaultDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed ordinary default program")
    }
}

#[derive(Clone)]
enum DefaultNode {
    Unavailable,
    Leaf,
    Source {
        requirement: OrdinarySourceDefaultRequirement,
        members: Vec<String>,
    },
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Eligible {
    cells: u64,
    requirements: BTreeMap<String, OrdinarySourceDefaultRequirement>,
}
struct Eligibility {
    nodes: BTreeMap<String, DefaultNode>,
    cache: BTreeMap<String, Option<Eligible>>,
    active: BTreeSet<String>,
}
impl Eligibility {
    fn get(&mut self, id: &str) -> R<Option<Eligible>> {
        if let Some(value) = self.cache.get(id) {
            return Ok(value.clone());
        }
        if !self.active.insert(id.into()) {
            return Err(OrdinaryCarrierError::Cycle);
        }
        let node = self
            .nodes
            .get(id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        let value = match node {
            DefaultNode::Unavailable => None,
            DefaultNode::Leaf => Some(Eligible {
                cells: 1,
                requirements: BTreeMap::new(),
            }),
            DefaultNode::Source {
                requirement,
                members,
            } => {
                let mut value = Eligible {
                    cells: 1,
                    requirements: [(id.into(), requirement)].into_iter().collect(),
                };
                let mut available = true;
                for member in members {
                    let Some(child) = self.get(&member)? else {
                        available = false;
                        break;
                    };
                    // A reused type has a shared definition, but each stored
                    // occurrence contributes its complete default cell count.
                    value.cells = value
                        .cells
                        .checked_add(child.cells)
                        .ok_or(OrdinaryCarrierError::Limit)?;
                    if value.cells > TOTAL_VALUE_CELLS_MAX {
                        available = false;
                        break;
                    }
                    value.requirements.extend(child.requirements);
                }
                available.then_some(value)
            }
        };
        self.active.remove(id);
        self.cache.insert(id.into(), value.clone());
        Ok(value)
    }
}

fn node(vir: &ValidatedPracticalVir, carrier: &OrdinaryCarrier) -> R<DefaultNode> {
    let id = &carrier.type_id;
    let (bundle, roots, _) = vir.construction_context();
    if let Some(source) = roots.source_types.get(id) {
        if source.kind == SourceKind::SealedClass
            || source.members.iter().any(|m| m.required)
            || source.kind == SourceKind::Enum && !source.enum_values.iter().any(|v| v == "0")
        {
            return Ok(DefaultNode::Unavailable);
        }
        let members = source
            .members
            .iter()
            .map(|m| closed_type_id(bundle, &m.ty).map_err(|_| OrdinaryCarrierError::Linkage))
            .collect::<R<Vec<_>>>()?;
        return Ok(DefaultNode::Source {
            requirement: OrdinarySourceDefaultRequirement {
                source_type_id: id.clone(),
                source_sha256: source.source_sha256.clone(),
                declared_public_default: source.public_default,
            },
            members,
        });
    }
    if let Some(metadata) = vir.data_closed().metadata.get(id) {
        let arm = match template_name(&metadata.template_id) {
            Some("option") => "none",
            Some("lookup") => "missing_key",
            _ => return Ok(DefaultNode::Unavailable),
        };
        let OrdinaryShape::Sum { arms } = &carrier.shape else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if !arms
            .iter()
            .any(|a| a.id == arm && a.tag == 0 && a.fields.is_empty())
        {
            return Err(OrdinaryCarrierError::Shape);
        }
        // The absent arm is the entire default path. Its argument need not
        // have a default, and it contributes no cells or source requirements.
        return Ok(DefaultNode::Leaf);
    }
    let scalar = id
        .strip_prefix("mpk.csharp.value.")
        .and_then(|s| s.strip_suffix(".v1"));
    Ok(
        if matches!(
            scalar,
            Some(
                "bool"
                    | "unit"
                    | "day_of_week"
                    | "char"
                    | "f32"
                    | "f64"
                    | "decimal"
                    | "date"
                    | "time"
                    | "duration"
                    | "instant"
                    | "guid"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
            )
        ) {
            DefaultNode::Leaf
        } else {
            DefaultNode::Unavailable
        },
    )
}

pub fn generate_csharp_practical_ordinary_defaults(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDefaultProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut b = Builder::new()?;
    let definitions = emit_defaults(vir, layouts.carriers(), &mut b)?;
    let certificate = b.finish()?;
    let program = OrdinaryDefaultProgram {
        schema: "mpk.csharp.ordinary_defaults.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if program.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(program)
}
pub fn import_csharp_practical_ordinary_defaults(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryDefaultProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_defaults(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}

pub(super) fn emit_defaults(
    vir: &ValidatedPracticalVir,
    layouts: &[OrdinaryCarrier],
    b: &mut Builder,
) -> R<Vec<OrdinaryDefaultDefinition>> {
    let mut eligibility = Eligibility {
        nodes: layouts
            .iter()
            .map(|c| Ok((c.type_id.clone(), node(vir, c)?)))
            .collect::<R<_>>()?,
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    let mut definitions = vec![];
    for carrier in layouts {
        let prefix = format!(
            "{PREFIX}.Default.T{}",
            carrier
                .type_id
                .as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        );
        let candidate = eligibility.get(&carrier.type_id)?;
        let public_candidate_admitted = candidate
            .as_ref()
            .is_some_and(|e| e.requirements.values().all(|r| r.declared_public_default));
        let structural_candidate = if let Some(value) = candidate {
            // All admitted defaults have zero scalar bits, zero product
            // padding, or an absent tag-zero arm. Emit the actual closed cube.
            let definition = format!("{prefix}.Value");
            let zero = b.constant("Std.Bool.false")?;
            let body = b.wrap_selectors(carrier.depth, zero)?;
            let ty = b.cube(carrier.depth)?;
            b.define(&definition, ty, body)?;
            Some(OrdinaryDefaultCandidate {
                definition,
                logical_cells: value.cells,
                source_requirements: value.requirements.into_values().collect(),
            })
        } else {
            None
        };
        let public_admission_definition = format!("{prefix}.DeclaredPublicAdmission");
        let flag = b.constant(if public_candidate_admitted {
            "Std.Bool.true"
        } else {
            "Std.Bool.false"
        })?;
        b.define(&public_admission_definition, b.boolean, flag)?;
        definitions.push(OrdinaryDefaultDefinition {
            carrier: carrier.clone(),
            structural_candidate,
            public_candidate_admitted,
            public_admission_definition,
        });
    }
    Ok(definitions)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_graph_counts_occurrences_and_retains_source_conditions() {
        let source = |id: &str, members: Vec<String>, public| DefaultNode::Source {
            requirement: OrdinarySourceDefaultRequirement {
                source_type_id: id.into(),
                source_sha256: "0".repeat(64),
                declared_public_default: public,
            },
            members,
        };
        let mut nodes = BTreeMap::from([
            ("leaf".into(), DefaultNode::Leaf),
            ("unavailable".into(), DefaultNode::Unavailable),
        ]);
        // Reusing each child twice counts it twice; source requirements are
        // unique because every occurrence has the same closed default value.
        let mut previous = "leaf".to_owned();
        for level in 1..=15 {
            let id = format!("pair{level}");
            nodes.insert(
                id.clone(),
                source(&id, vec![previous.clone(), previous], level != 1),
            );
            previous = id;
        }
        nodes.insert("bound".into(), source("bound", vec![previous], true));
        nodes.insert(
            "excess".into(),
            source("excess", vec!["bound".into()], true),
        );
        nodes.insert(
            "dependent".into(),
            source("dependent", vec!["unavailable".into()], true),
        );
        let mut e = Eligibility {
            nodes,
            cache: BTreeMap::new(),
            active: BTreeSet::new(),
        };
        let value = e.get("bound").unwrap().unwrap();
        assert_eq!(value.cells, 65536);
        assert_eq!(value.requirements.len(), 16);
        assert!(!value.requirements["pair1"].declared_public_default);
        assert!(e.get("excess").unwrap().is_none());
        assert!(e.get("dependent").unwrap().is_none());
        assert_eq!(e.get("bound").unwrap(), Some(value));
        e.nodes
            .insert("cycle".into(), source("cycle", vec!["cycle".into()], true));
        assert!(matches!(e.get("cycle"), Err(OrdinaryCarrierError::Cycle)));
    }
}
