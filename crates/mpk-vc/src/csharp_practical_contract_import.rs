//! T06-W01 verification-side expression import. No source parser, VC proof or
//! external evaluator is invoked. Ordinary constants are definition references;
//! their bodies/discharge remain with T06-W02..W09.
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "form", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContractTerm {
    Var {
        index: usize,
        type_id: String,
    },
    Const {
        name: String,
        type_id: String,
    },
    App {
        function: Box<ContractTerm>,
        argument: Box<ContractTerm>,
        type_id: String,
    },
    Lam {
        parameter_type: String,
        body: Box<ContractTerm>,
        type_id: String,
    },
    Let {
        value: Box<ContractTerm>,
        body: Box<ContractTerm>,
        type_id: String,
    },
}
impl ContractTerm {
    pub fn type_id(&self) -> &str {
        match self {
            Self::Var { type_id, .. }
            | Self::Const { type_id, .. }
            | Self::App { type_id, .. }
            | Self::Lam { type_id, .. }
            | Self::Let { type_id, .. } => type_id,
        }
    }
    pub fn nodes(&self) -> usize {
        match self {
            Self::App {
                function, argument, ..
            } => 1 + function.nodes() + argument.nodes(),
            Self::Lam { body, .. } => 1 + body.nodes(),
            Self::Let { value, body, .. } => 1 + value.nodes() + body.nodes(),
            _ => 1,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContractDefinition {
    pub name: String,
    pub tag: String,
    /// Canonical non-expression fields, including exact source/binding/codec IDs.
    pub parameters: String,
    pub argument_types: Vec<String>,
    pub result_type: String,
    pub ordered_checks: Vec<RequiredCheck>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VerifiedContractExpression {
    #[serde(skip)]
    owner: String,
    #[serde(skip)]
    allows_old: bool,
    #[serde(skip)]
    exception_scope: Option<String>,
    expression_sha256: String,
    attachment_sha256: String,
    canonical_expression: String,
    term: ContractTerm,
    definitions: Vec<ContractDefinition>,
    subject_bindings: Vec<(String, String)>,
}
impl VerifiedContractExpression {
    pub(crate) fn exception_scope(&self) -> Option<&str> {
        self.exception_scope.as_deref()
    }
    pub(crate) fn allows_old(&self) -> bool {
        self.allows_old
    }
    pub(crate) fn owner(&self) -> &str {
        &self.owner
    }
    pub(crate) fn expression(&self) -> &str {
        &self.canonical_expression
    }
    pub(crate) fn subjects(&self) -> &[(String, String)] {
        &self.subject_bindings
    }
    pub fn term(&self) -> &ContractTerm {
        &self.term
    }
    pub fn binder_depth(&self) -> usize {
        fn local(t: &ContractTerm) -> usize {
            match t {
                ContractTerm::App {
                    function, argument, ..
                } => local(function).max(local(argument)),
                ContractTerm::Lam { body, .. } => 1 + local(body),
                ContractTerm::Let { value, body, .. } => local(value).max(1 + local(body)),
                _ => 0,
            }
        }
        self.subject_bindings.len() + local(&self.term)
    }

    pub fn attachment_sha256(&self) -> &str {
        &self.attachment_sha256
    }
    pub fn expression_sha256(&self) -> &str {
        &self.expression_sha256
    }
    pub fn definitions(&self) -> &[ContractDefinition] {
        &self.definitions
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed expression")
    }
}
/// Scope-local collection used only while reattaching original inputs. A failed
/// attachment discards the entire collection; callers cannot publish a prefix.
#[derive(Clone, Debug, Default)]
pub struct ContractExpressionCapture(Arc<Mutex<CaptureState>>);
#[derive(Debug, Default)]
struct CaptureState {
    rows: Vec<VerifiedContractExpression>,
    nodes: usize,
}
impl ContractExpressionCapture {
    pub(crate) fn record(
        &self,
        expression: VerifiedContractExpression,
    ) -> Result<(), DataPhaseError> {
        let mut rows = self.0.lock().map_err(|_| DataPhaseError::Contract)?;
        let count = rows.nodes + expression.term.nodes();
        if count > crate::csharp_practical_vc_model::ORDINARY_TERM_NODES_MAX as usize {
            return Err(DataPhaseError::Contract);
        }
        rows.nodes = count;
        rows.rows.push(expression);
        Ok(())
    }
    pub(crate) fn finish(&self) -> Result<Vec<VerifiedContractExpression>, DataPhaseError> {
        Ok(self
            .0
            .lock()
            .map_err(|_| DataPhaseError::Contract)?
            .rows
            .clone())
    }
}
fn hash(domain: &str, bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(domain);
    h.update([0]);
    h.update(bytes);
    format!("{:x}", h.finalize())
}
fn text<'a>(v: &'a J, k: &str) -> Result<&'a str, DataPhaseError> {
    v.get(k).and_then(J::as_str).ok_or(DataPhaseError::Contract)
}
fn arrow(a: &str, b: &str) -> String {
    format!("({a}->{b})")
}
fn shapes() -> &'static Vec<Value> {
    static SHAPES: std::sync::OnceLock<Vec<Value>> = std::sync::OnceLock::new();
    SHAPES.get_or_init(|| {
        serde_json::from_str::<Value>(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../develop/specs/vectors/csharp-practical-profile-v1.json"
        )))
        .expect("frozen union")["frozen_contract"]["expression_union"]["variants"]
            .as_array()
            .unwrap()
            .clone()
    })
}
/// Independent strict byte import; shares concrete typing/value semantics with
/// the frontend, but never calls its expression parser or sidecar attachment.
pub fn import_verification_contract_expression(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    env: &DataContractEnvironment,
    bytes: &[u8],
) -> Result<VerifiedContractExpression, DataPhaseError> {
    if bytes.len() > data_phase::data_contract_limit("contract_file_bytes") {
        return Err(DataPhaseError::Contract);
    }
    let value = a::parse_canonical_practical_json(a::PracticalArtifactKind::MethodContract, bytes)
        .map_err(|_| DataPhaseError::Contract)?;
    fn shape(v: &J, depth: usize, nodes: &mut usize) -> Result<(), DataPhaseError> {
        *nodes += 1;
        if depth > data_phase::data_contract_limit("contract_depth")
            || *nodes > data_phase::data_contract_limit("contract_nodes_per_method")
        {
            return Err(DataPhaseError::Contract);
        }
        let tag = text(v, "tag")?;
        let variant = shapes()
            .iter()
            .find(|s| s["tag"] == tag)
            .ok_or(DataPhaseError::Contract)?;
        let fields = v.as_object().ok_or(DataPhaseError::Contract)?;
        if fields
            .iter()
            .map(|(k, _)| k.as_str())
            .ne(variant["ordered_fields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s.as_str().unwrap()))
        {
            return Err(DataPhaseError::Contract);
        }
        for (key, child) in fields {
            match variant["field_types"][key].as_str().unwrap() {
                "contract_expression" | "contract_expression_bool" => {
                    shape(child, depth + 1, nodes)?
                }
                "contract_expression_or_null" if child != &J::Null => {
                    shape(child, depth + 1, nodes)?
                }
                "ordered_array<contract_expression>" => {
                    for child in child.as_array().ok_or(DataPhaseError::Contract)? {
                        shape(child, depth + 1, nodes)?;
                    }
                }
                "codec_parameters"
                    if child
                        .as_object()
                        .ok_or(DataPhaseError::Contract)?
                        .iter()
                        .map(|(k, _)| k.as_str())
                        .ne(["scale", "rounding"]) =>
                {
                    return Err(DataPhaseError::Contract)
                }
                _ => {}
            }
        }
        Ok(())
    }
    shape(&value, 1, &mut 0)?;
    data_phase::validate_data_contract_value(b, r, c, env, &value)?;
    let mut subjects = env
        .variables
        .iter()
        .map(|(id, t)| (format!("current:{id}"), t.clone()))
        .collect::<Vec<_>>();
    if env.allow_old {
        subjects.extend(
            env.variables
                .iter()
                .filter(|(_, t)| t.as_str() != EXCEPTION_TYPE_ID)
                .map(|(id, t)| (format!("entry:{id}"), t.clone())),
        );
    }
    if let Some(t) = &env.result {
        subjects.push(("result".into(), t.clone()));
    }
    if subjects.len() > crate::csharp_practical_vc_model::BINDER_DEPTH_MAX as usize {
        return Err(DataPhaseError::Contract);
    }
    let mut compiler = Compiler {
        env,
        subjects: &subjects,
        locals: vec![],
        definitions: BTreeMap::new(),
    };
    let term = compiler.compile(&value, false)?;
    if term.nodes() > crate::csharp_practical_vc_model::ORDINARY_TERM_NODES_MAX as usize {
        return Err(DataPhaseError::Contract);
    }
    let expression_sha256 = hash("MPK-CSHARP-CONTRACT-EXPRESSION-1.0", bytes);
    // This binds scopes, pure signature identities and the exact closed closure.
    let scope = json!({"owner":env.verification_owner,"partial_callables":env.partial_callables,"subjects":subjects,"old":env.allow_old,"exception_type":env.exception_type,"exception_universe":env.exception_universe.as_ref().map(|u|u.arms().iter().map(|a|json!({"tag":a.tag,"type_id":a.type_id,"payload_members":a.payload_member_ids,"payload_types":a.payload_type_ids,"ancestry":a.ancestry})).collect::<Vec<_>>()),"operations":env.operations,"constructors":env.constructors,"properties":env.properties,"bindings":env.bindings,"roots":hash("roots",r.canonical_json()),"closed":hash("closed",c.canonical_json())});
    let attachment_sha256 = hash(
        "MPK-CSHARP-CONTRACT-ATTACHMENT-1.0",
        &serde_json::to_vec(&(expression_sha256.clone(), scope))
            .map_err(|_| DataPhaseError::Contract)?,
    );
    let definitions = compiler.definitions.into_values().collect();
    Ok(VerifiedContractExpression {
        owner: env.verification_owner.clone(),
        allows_old: env.allow_old,
        exception_scope: env.exception_type.clone(),
        expression_sha256,
        attachment_sha256,
        canonical_expression: String::from_utf8(bytes.to_vec())
            .map_err(|_| DataPhaseError::Contract)?,
        term,
        definitions,
        subject_bindings: subjects,
    })
}
/// Rebuild the complete typed encoding and attachment identity instead of
/// trusting caller-supplied term types, constants, hashes or subject binders.
pub fn import_verification_contract_encoding(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    env: &DataContractEnvironment,
    expression: &[u8],
    encoding: &[u8],
) -> Result<VerifiedContractExpression, DataPhaseError> {
    let expected = import_verification_contract_expression(b, r, c, env, expression)?;
    if encoding != expected.canonical_bytes() {
        return Err(DataPhaseError::Contract);
    }
    Ok(expected)
}
struct Compiler<'a> {
    env: &'a DataContractEnvironment,
    subjects: &'a [(String, String)],
    locals: Vec<(String, String)>,
    definitions: BTreeMap<String, ContractDefinition>,
}
impl Compiler<'_> {
    fn compile(&mut self, v: &J, old: bool) -> Result<ContractTerm, DataPhaseError> {
        if self.subjects.len() + self.locals.len()
            > crate::csharp_practical_vc_model::BINDER_DEPTH_MAX as usize
        {
            return Err(DataPhaseError::Contract);
        }
        let tag = text(v, "tag")?;
        let ty = text(v, "type_id")?.to_owned();
        if tag == "construct"
            && self
                .env
                .partial_callables
                .contains(text(v, "constructor_id")?)
        {
            return Err(DataPhaseError::Contract);
        }
        match tag {
            "variable" | "result" => {
                let name = if tag == "result" {
                    if old {
                        return Err(DataPhaseError::Contract);
                    }
                    "result".to_owned()
                } else {
                    let id = text(v, "binding_id")?;
                    if let Some(index) = self.locals.iter().rev().position(|(k, _)| k == id) {
                        return Ok(ContractTerm::Var { index, type_id: ty });
                    }
                    format!("{}:{id}", if old { "entry" } else { "current" })
                };
                let index = self
                    .subjects
                    .iter()
                    .rev()
                    .position(|(id, _)| id == &name)
                    .ok_or(DataPhaseError::Contract)?
                    + self.locals.len();
                return Ok(ContractTerm::Var { index, type_id: ty });
            }
            "old" => {
                return self.compile(v.get("expression").ok_or(DataPhaseError::Contract)?, true)
            }
            "let" => {
                let value = self.compile(v.get("value").ok_or(DataPhaseError::Contract)?, old)?;
                self.locals
                    .push((text(v, "binding_id")?.into(), value.type_id().into()));
                let body = self.compile(v.get("body").ok_or(DataPhaseError::Contract)?, old)?;
                self.locals.pop();
                return Ok(ContractTerm::Let {
                    value: Box::new(value),
                    body: Box::new(body),
                    type_id: ty,
                });
            }
            _ => {}
        }
        let variant = shapes()
            .iter()
            .find(|s| s["tag"] == tag)
            .ok_or(DataPhaseError::Contract)?;
        let mut args = vec![];
        let mut params = vec![];
        for (key, child) in v.as_object().ok_or(DataPhaseError::Contract)? {
            if key == "tag" || key == "type_id" {
                continue;
            }
            if matches!(tag, "bounded_forall" | "bounded_exists") && key == "binding_id" {
                continue;
            }
            match variant["field_types"][key].as_str().unwrap() {
                "contract_expression"
                | "contract_expression_bool"
                | "contract_expression_or_null"
                    if child != &J::Null =>
                {
                    if key == "body" && matches!(tag, "bounded_forall" | "bounded_exists") {
                        let t = text(v.get("lower").ok_or(DataPhaseError::Contract)?, "type_id")?
                            .to_owned();
                        self.locals.push((text(v, "binding_id")?.into(), t.clone()));
                        let body = self.compile(child, old)?;
                        self.locals.pop();
                        args.push(ContractTerm::Lam {
                            parameter_type: t.clone(),
                            type_id: arrow(&t, body.type_id()),
                            body: Box::new(body),
                        });
                    } else {
                        args.push(self.compile(child, old)?);
                    }
                }
                "ordered_array<contract_expression>" => {
                    for e in child.as_array().ok_or(DataPhaseError::Contract)? {
                        args.push(self.compile(e, old)?);
                    }
                }
                _ => params.push((key.clone(), child.clone())),
            }
        }
        let parameters = String::from_utf8(
            a::canonical_practical_json_bytes(&J::Object(params))
                .map_err(|_| DataPhaseError::Contract)?,
        )
        .map_err(|_| DataPhaseError::Contract)?;
        let argument_types = args
            .iter()
            .map(|a| a.type_id().into())
            .collect::<Vec<String>>();
        let signature = argument_types
            .iter()
            .rev()
            .fold(ty.clone(), |out, a| arrow(a, &out));
        let name = format!(
            "contract.def.{}",
            hash(
                "MPK-CSHARP-CONTRACT-DEFINITION-1.0",
                &serde_json::to_vec(&(tag, &parameters, &argument_types, &ty))
                    .map_err(|_| DataPhaseError::Contract)?
            )
        );
        let ordered_checks = if matches!(tag, "unary" | "binary") {
            let id = text(v, "operation_id")?;
            self.env
                .operations
                .get(id)
                .cloned()
                .or_else(|| scalar_operation_signature(id).ok())
                .ok_or(DataPhaseError::Contract)?
                .ordered_checks
        } else {
            vec![]
        };
        self.definitions.insert(
            name.clone(),
            ContractDefinition {
                name: name.clone(),
                tag: tag.into(),
                parameters,
                argument_types: argument_types.clone(),
                result_type: ty.clone(),
                ordered_checks,
            },
        );
        let mut term = ContractTerm::Const {
            name,
            type_id: signature,
        };
        for (n, arg) in args.into_iter().enumerate() {
            let result = argument_types[n + 1..]
                .iter()
                .rev()
                .fold(ty.clone(), |out, a| arrow(a, &out));
            term = ContractTerm::App {
                function: Box::new(term),
                argument: Box::new(arg),
                type_id: result,
            };
        }
        Ok(term)
    }
}
