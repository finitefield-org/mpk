//! Ordinary values and W03 definedness for every original contract attachment.
//! Subjects retain their original order and identities for use-point owners.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryContractExpressionDefinition {
    pub data_contract_id: String,
    /// Copied verbatim; frozen loop captures may have an empty owner. Their
    /// concrete program point still comes from the control VC's bindings.
    pub owner: String,
    pub attachment_sha256: String,
    pub expression_sha256: String,
    pub allows_old: bool,
    pub exception_scope: Option<String>,
    pub subjects: Vec<(String, String)>,
    pub result_type: String,
    pub value_definition: String,
    pub definedness_definition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryContractExpressionProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    data_vc_sha256: String,
    definitions: Vec<OrdinaryContractExpressionDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryContractExpressionProgram {
    pub fn definitions(&self) -> &[OrdinaryContractExpressionDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("ordinary contract expressions")
    }
}
fn bound(c: &mut Clauses<'_>, name: &str, args: &[String], term: &ContractTerm) -> R<()> {
    let mut body = c.lower(term, &mut args.to_vec(), 0)?;
    for arg in args.iter().rev() {
        let ty = c.ty(arg, 0)?;
        body = c.b.lam(ty, body)?;
    }
    if !c.b.globals.contains_key(name) {
        let ty = c.ty(&signature(args, term.type_id()), 0)?;
        c.b.define(name, ty, body)?;
    }
    Ok(())
}
pub fn generate_csharp_practical_ordinary_contract_expressions(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryContractExpressionProgram> {
    let data = crate::csharp_practical_vir_model::data_vc::generate_data_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let expressions = data.contract_expressions().iter().collect::<Vec<_>>();
    if data.contracts().len() != expressions.len() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut c = compiler(vir, &layouts, Builder::new()?, &expressions)?;
    let mut definitions = vec![];
    for (expression, obligation) in expressions.into_iter().zip(data.contracts()) {
        if expression.attachment_sha256() != obligation.attachment_sha256
            || expression.subjects() != obligation.subjects
            || obligation.definedness.type_id() != SOURCE_BOOL
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        for d in expression.definitions() {
            c.recipe(d)?;
        }
        let args = expression
            .subjects()
            .iter()
            .map(|(_, ty)| ty.clone())
            .collect::<Vec<_>>();
        // Attachment identity includes owner,old/exception mode,all subjects,
        // nominal signatures and closed context. Expression identity alone
        // cannot distinguish,for example,requires from ensures with a result.
        let value_definition = format!(
            "{PREFIX}.ContractExpression.A{}",
            expression.attachment_sha256()
        );
        let definedness_definition = format!(
            "{PREFIX}.ContractDefined.A{}",
            expression.attachment_sha256()
        );
        bound(&mut c, &value_definition, &args, expression.term())?;
        c.definedness_logic()?;
        bound(
            &mut c,
            &definedness_definition,
            &args,
            &obligation.definedness,
        )?;
        definitions.push(OrdinaryContractExpressionDefinition {
            data_contract_id: obligation.id.clone(),
            owner: expression.owner().into(),
            attachment_sha256: expression.attachment_sha256().into(),
            expression_sha256: expression.expression_sha256().into(),
            allows_old: expression.allows_old(),
            exception_scope: expression.exception_scope().map(str::to_owned),
            subjects: expression.subjects().to_vec(),
            result_type: expression.term().type_id().into(),
            value_definition,
            definedness_definition,
        });
    }
    let certificate = c.b.finish()?;
    let p = OrdinaryContractExpressionProgram {
        schema: "mpk.csharp.ordinary_contract_expressions.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        data_vc_sha256: data.hash(),
        definitions,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_contract_expressions(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryContractExpressionProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let expected = generate_csharp_practical_ordinary_contract_expressions(vir)?;
    if input != expected.canonical_bytes() || certificate != expected.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(expected)
}
