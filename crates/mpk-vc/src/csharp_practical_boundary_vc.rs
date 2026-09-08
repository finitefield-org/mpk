//! T06-W07: pending typed boundary relations and independently reproduced runs.
//! Parsing/evidence checks do not prove serializer correctness or source execution.
use super::control_vc as cv;
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
use crate::csharp_practical_vir_validation::ValidatedPracticalVir;
const BOOL: &str = "mpk.csharp.value.bool.v1";
const STRING: &str = "mpk.csharp.value.string.v1";
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryVcError {
    Linkage,
    Limit,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BoundarySequent {
    pub id: String,
    pub kind: String,
    pub subjects: Vec<TypedValueRef>,
    pub assumptions: Vec<ContractTerm>,
    pub goals: Vec<ContractTerm>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BoundaryVcProgram {
    source_ir_sha256: String,
    /// Lossless canonical documents retain UTF-16 names, codecs and frozen defaults.
    contracts: Vec<String>,
    sequents: Vec<BoundarySequent>,
    definition_names: Vec<String>,
    #[serde(skip)]
    nodes: usize,
    #[serde(skip)]
    depth: usize,
}
impl BoundaryVcProgram {
    pub fn contracts(&self) -> &[String] {
        &self.contracts
    }
    pub fn sequents(&self) -> &[BoundarySequent] {
        &self.sequents
    }
    pub fn definition_names(&self) -> &[String] {
        &self.definition_names
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed boundary VCs")
    }
    pub fn hash(&self) -> String {
        digest(&self.canonical_bytes(), "MPK-CSHARP-BOUNDARY-VC-1.0")
    }
    pub(crate) fn nodes(&self) -> usize {
        self.nodes
    }
    pub(crate) fn binder_depth(&self) -> usize {
        self.depth
    }
    fn push(
        &mut self,
        owner: &str,
        kind: &str,
        subjects: Vec<TypedValueRef>,
        assumptions: Vec<ContractTerm>,
        goals: Vec<ContractTerm>,
    ) {
        self.sequents.push(BoundarySequent {
            id: format!("boundary.{kind}.{owner}"),
            kind: kind.into(),
            subjects,
            assumptions,
            goals,
        });
    }
    fn finish(&mut self) -> Result<(), BoundaryVcError> {
        let mut names = BTreeSet::new();
        self.nodes = 0;
        self.depth = 0;
        for s in &self.sequents {
            for t in s.assumptions.iter().chain(&s.goals) {
                self.nodes += t.nodes();
                cv::register_term(t, &mut names, s.subjects.len(), &mut self.depth);
            }
        }
        self.definition_names = names.into_iter().collect();
        self.sequents.sort_by(|a, b| a.id.cmp(&b.id));
        if self.sequents.windows(2).any(|s| s[0].id == s[1].id) {
            return Err(BoundaryVcError::Linkage);
        }
        if self.nodes > 262144
            || self.depth > 256
            || self.sequents.len() + self.definition_names.len() > 8192
            || self.canonical_bytes().len() > 16 * 1024 * 1024
        {
            return Err(BoundaryVcError::Limit);
        }
        Ok(())
    }
}
fn digest(bytes: &[u8], domain: &'static str) -> String {
    crate::hash::hash_domain_separated_raw(HashDomain::new(domain), bytes)
        .expect("bounded bytes")
        .to_hex()
}
fn text<'a>(v: &'a J, key: &str) -> Result<&'a str, BoundaryVcError> {
    v.get(key)
        .and_then(J::as_str)
        .ok_or(BoundaryVcError::Linkage)
}
fn var(i: usize, ty: &str) -> ContractTerm {
    ContractTerm::Var {
        index: i,
        type_id: ty.into(),
    }
}
fn subject(id: &str, ty: &str) -> TypedValueRef {
    TypedValueRef {
        id: id.into(),
        type_id: ty.into(),
    }
}
fn call(name: &str, args: Vec<ContractTerm>, ty: &str) -> ContractTerm {
    cv::apply(name, args, ty)
}
fn pred(owner: &str, kind: &str, args: Vec<ContractTerm>) -> ContractTerm {
    call(&format!("Mpk.CSharp.Boundary.{kind}.{owner}"), args, BOOL)
}
fn equal(ty: &str, a: ContractTerm, b: ContractTerm) -> ContractTerm {
    call(&format!("Mpk.CSharp.Binding.Equal.{ty}"), vec![a, b], BOOL)
}
// Field-complete source equality normalizes array snapshots to sequences and
// decimal scale, preserving all other storage, tags, order and float bits.
fn observe(ty: &str, a: ContractTerm, b: ContractTerm) -> ContractTerm {
    call(
        &format!("Mpk.CSharp.Boundary.SourceObserveEqual.{ty}"),
        vec![a, b],
        BOOL,
    )
}
fn domain(ty: &str, x: ContractTerm) -> ContractTerm {
    call(&format!("Mpk.CSharp.PublicDomain.{ty}"), vec![x], BOOL)
}
fn implies(a: ContractTerm, b: ContractTerm) -> ContractTerm {
    cv::combine(&[cv::not(a), b], false)
}

pub(crate) fn generate_boundary_vcs(
    vir: &ValidatedPracticalVir,
) -> Result<BoundaryVcProgram, BoundaryVcError> {
    let mut p = BoundaryVcProgram {
        source_ir_sha256: vir.hash().into(),
        contracts: vec![],
        sequents: vec![],
        definition_names: vec![],
        nodes: 0,
        depth: 0,
    };
    for doc in vir.data_contracts() {
        let v = a::parse_canonical_practical_json(
            a::PracticalArtifactKind::BoundaryContract,
            doc.as_bytes(),
        )
        .map_err(|_| BoundaryVcError::Linkage)?;
        if text(&v, "schema")? != a::BOUNDARY_CONTRACT_SCHEMA {
            continue;
        }
        let id = text(&v, "contract_sha256")?;
        p.contracts.push(doc.clone());
        // JSON text uses the ordinary string carrier without PublicDomain: a
        // document can exceed the 16384-unit limit of an application string.
        let document = var(0, STRING);
        // The exact grammar/codec recipes are keyed by the retained contract.
        // Canonical UTF-8, duplicate keys (after escape decoding), declaration
        // order and all fixed byte/depth/cell/string/collection limits are required.
        let accepted = pred(id, "AcceptInput", vec![document.clone()]);
        let mut constraints = [
            "CanonicalUtf8Json",
            "RootObject",
            "UniqueDecodedNames",
            "KnownNamesInDeclarationOrder",
            "DocumentBytes1048576",
            "Depth32",
            "JsonCells262144",
            "DecodedValueCells65536",
            "Utf16Units16384",
            "CollectionCount4096",
        ]
        .iter()
        .map(|k| pred(id, k, vec![document.clone()]))
        .collect::<Vec<_>>();
        for f in v
            .get("input_fields")
            .and_then(J::as_array)
            .ok_or(BoundaryVcError::Linkage)?
        {
            let fid = format!("{id}.{}", text(f, "field_id")?);
            let present = pred(&fid, "Present", vec![document.clone()]);
            let null = pred(&fid, "Null", vec![document.clone()]);
            let value = cv::combine(&[present.clone(), cv::not(null.clone())], true);
            let missing = cv::not(present.clone());
            let required = f.get("required") == Some(&J::Bool(true));
            let nullable = f.get("nullable") == Some(&J::Bool(true));
            let missing_rule = text(
                f.get("missing_rule").ok_or(BoundaryVcError::Linkage)?,
                "mode",
            )?;
            let legal = cv::combine(
                &[
                    implies(
                        missing.clone(),
                        cv::boolean(!required && missing_rule != "reject"),
                    ),
                    implies(null.clone(), cv::boolean(nullable)),
                    implies(null.clone(), present.clone()),
                    implies(
                        value.clone(),
                        pred(&fid, "CanonicalTypedPayload", vec![document.clone()]),
                    ),
                ],
                true,
            );
            constraints.push(legal.clone());
            let ty = text(f, "type_id")?;
            let decoded = call(
                &format!("Mpk.CSharp.Boundary.DecodeSource.{fid}"),
                vec![document.clone()],
                ty,
            );
            p.push(
                &fid,
                "input_field",
                vec![subject("document", STRING)],
                vec![accepted.clone()],
                vec![
                    legal,
                    domain(ty, decoded.clone()),
                    pred(
                        &fid,
                        "TypedConversionAndReconstruction",
                        vec![document.clone(), decoded.clone()],
                    ),
                    implies(missing, pred(&fid, "MissingRule", vec![decoded.clone()])),
                    implies(null, pred(&fid, "NullRule", vec![decoded.clone()])),
                    implies(
                        value,
                        pred(&fid, "ValueRule", vec![document.clone(), decoded]),
                    ),
                ],
            );
        }
        p.push(
            id,
            "input_acceptance",
            vec![subject("document", STRING)],
            vec![],
            vec![equal(BOOL, accepted, cv::combine(&constraints, true))],
        );
        let outputs = v
            .get("output_fields")
            .and_then(J::as_array)
            .ok_or(BoundaryVcError::Linkage)?;
        let ty = if let Some(f) = outputs.first() {
            text(f, "type_id")?
        } else {
            "mpk.csharp.value.unit.v1"
        };
        let returned = var(0, ty);
        let encoded = call(
            &format!("Mpk.CSharp.Boundary.EncodeOutput.{id}"),
            vec![returned.clone()],
            STRING,
        );
        let reparsed = call(
            &format!("Mpk.CSharp.Boundary.ReparseOutput.{id}"),
            vec![encoded.clone()],
            ty,
        );
        // Defined means formatting succeeds, never that reparse equality holds.
        // A lossy codec cannot establish this goal merely by producing bytes.
        p.push(
            id,
            "output_round_trip",
            vec![subject("returned", ty)],
            vec![
                domain(ty, returned.clone()),
                // Exact selected-source summary under accepted boundary inputs
                // and method preconditions; W02-W06 dependencies remain pending.
                pred(id, "BoundaryReachableReturn", vec![returned.clone()]),
                pred(id, "OutputCodecDefined", vec![returned.clone()]),
            ],
            vec![
                pred(id, "CanonicalCompleteOutput", vec![encoded.clone()]),
                pred(id, "OutputFixedLimits", vec![encoded]),
                domain(ty, reparsed.clone()),
                observe(ty, returned, reparsed),
            ],
        );
    }
    p.contracts.sort();
    p.finish()?;
    Ok(p)
}

/// Exact literal bodies for W09, never caller-provided theorem constants.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BoundaryValueDefinition {
    pub name: String,
    pub value: MonomorphicValue,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BoundaryRunVcProgram {
    boundary_program_sha256: String,
    source_ir_sha256: String,
    boundary_contract: String,
    input_capture: String,
    output_capture: String,
    manifest: String,
    artifacts: String,
    canonical_input: String,
    canonical_output: String,
    values: Vec<BoundaryValueDefinition>,
    literal_value_cells: u64,
    relations: BoundaryVcProgram,
}
impl BoundaryRunVcProgram {
    pub fn sequents(&self) -> &[BoundarySequent] {
        self.relations.sequents()
    }
    pub fn values(&self) -> &[BoundaryValueDefinition] {
        &self.values
    }
    pub fn definition_names(&self) -> &[String] {
        self.relations.definition_names()
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed run VCs")
    }
    pub fn hash(&self) -> String {
        digest(&self.canonical_bytes(), "MPK-CSHARP-BOUNDARY-RUN-VC-1.0")
    }
    fn literal(&mut self, v: &MonomorphicValue) -> ContractTerm {
        let name = format!(
            "Mpk.CSharp.Boundary.Value.{}",
            digest(
                &serde_json::to_vec(v).expect("typed literal"),
                "MPK-CSHARP-BOUNDARY-LITERAL-1.0"
            )
        );
        if !self.values.iter().any(|d| d.name == name) {
            self.values.push(BoundaryValueDefinition {
                name: name.clone(),
                value: v.clone(),
            });
        }
        ContractTerm::Const {
            name,
            type_id: v.type_id().into(),
        }
    }
}
fn utf8(bytes: &[u8]) -> Result<String, BoundaryVcError> {
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| BoundaryVcError::Linkage)
}
impl EmittedDataPhase {
    /// Reparse original input and complete output and revalidate both artifact
    /// chains before constructing pending relations over exact typed values.
    pub fn generate_boundary_run_vcs(
        &self,
        b: &ValidatedFoundationBundle,
        context: &a::PracticalArtifactContext,
        captures: &a::CapturedInputSet,
        run: &CapturedBoundaryOutputRun,
    ) -> Result<BoundaryRunVcProgram, BoundaryVcError> {
        let verified = self
            .import_boundary_output_run(
                b,
                context,
                captures,
                run.input(),
                run.returned_value(),
                BoundaryOutputEvidence {
                    canonical_document: run.capture().canonical_document(),
                    capture: run.capture().artifact().canonical_bytes(),
                    manifest: run.manifest().canonical_bytes(),
                    artifacts: run.artifacts().canonical_bytes(),
                },
            )
            .map_err(|_| BoundaryVcError::Linkage)?;
        let static_program = generate_boundary_vcs(self.vir())?;
        let contract_hash = text(
            verified.capture().artifact().value(),
            "boundary_contract_sha256",
        )?;
        let boundary = self
            .boundaries()
            .iter()
            .find(|c| c.artifact().hash() == contract_hash)
            .ok_or(BoundaryVcError::Linkage)?;
        let mut p = BoundaryRunVcProgram {
            boundary_program_sha256: static_program.hash(),
            source_ir_sha256: self.vir().hash().into(),
            boundary_contract: utf8(boundary.artifact().canonical_bytes())?,
            input_capture: utf8(verified.input().capture().artifact().canonical_bytes())?,
            output_capture: utf8(verified.capture().artifact().canonical_bytes())?,
            manifest: utf8(verified.manifest().canonical_bytes())?,
            artifacts: utf8(verified.artifacts().canonical_bytes())?,
            canonical_input: utf8(verified.input().capture().canonical_document())?,
            canonical_output: utf8(verified.capture().canonical_document())?,
            values: vec![],
            literal_value_cells: 0,
            relations: BoundaryVcProgram {
                source_ir_sha256: self.vir().hash().into(),
                contracts: vec![],
                sequents: vec![],
                definition_names: vec![],
                nodes: 0,
                depth: 0,
            },
        };
        // Bytes/hashes/provenance are evidence linkage, not ordinary assertions
        // that an adapter interpreted its original media correctly.
        for arg in verified.input().arguments() {
            let value = p.literal(arg.value());
            let source = if let Some(op) = arg.reconstruction() {
                call(&op.id, vec![value.clone()], arg.source_type_id())
            } else {
                value.clone()
            };
            p.relations.push(
                arg.field_id(),
                "captured_input",
                vec![],
                vec![],
                vec![
                    domain(arg.value().type_id(), value.clone()),
                    domain(arg.source_type_id(), source.clone()),
                    pred(
                        &format!("{contract_hash}.{}", arg.field_id()),
                        "CapturedTypedInput",
                        vec![value, source],
                    ),
                ],
            );
        }
        let returned = p.literal(verified.returned_value());
        let reparsed = p.literal(verified.reparsed_value());
        p.relations.push(
            contract_hash,
            "captured_output_round_trip",
            vec![],
            vec![],
            vec![
                domain(verified.returned_value().type_id(), returned.clone()),
                domain(verified.reparsed_value().type_id(), reparsed.clone()),
                observe(verified.returned_value().type_id(), returned, reparsed),
            ],
        );
        p.values.sort_by(|a, b| a.name.cmp(&b.name));
        p.literal_value_cells = p
            .values
            .iter()
            .map(|d| {
                validate_value_inner(
                    b,
                    self.closure().roots(),
                    self.closure().closed(),
                    &d.value,
                    false,
                )
                .map_err(|_| BoundaryVcError::Linkage)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .sum();
        p.relations.finish()?;
        if p.literal_value_cells + p.relations.nodes() as u64 > 262144 {
            return Err(BoundaryVcError::Limit);
        }
        if p.canonical_bytes().len() > 16 * 1024 * 1024 {
            return Err(BoundaryVcError::Limit);
        }
        Ok(p)
    }
    pub fn import_boundary_run_vcs(
        &self,
        b: &ValidatedFoundationBundle,
        context: &a::PracticalArtifactContext,
        captures: &a::CapturedInputSet,
        run: &CapturedBoundaryOutputRun,
        input: &[u8],
    ) -> Result<BoundaryRunVcProgram, BoundaryVcError> {
        if input.len() > 16 * 1024 * 1024 {
            return Err(BoundaryVcError::Limit);
        }
        let p = self.generate_boundary_run_vcs(b, context, captures, run)?;
        if input != p.canonical_bytes() {
            return Err(BoundaryVcError::Linkage);
        }
        Ok(p)
    }
}
