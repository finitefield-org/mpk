use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::policy_evidence::{
    PolicyCallSitePreconditionEvidence, PolicyCallSitePreconditionStatus,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyCallSiteAnalysisRequest {
    pub source_path: PathBuf,
    pub contract_path: PathBuf,
    pub function_id: String,
    pub upstream_invariants: Vec<PolicyCallSiteInvariant>,
}

impl PolicyCallSiteAnalysisRequest {
    pub fn new(
        source_path: impl Into<PathBuf>,
        contract_path: impl Into<PathBuf>,
        function_id: impl Into<String>,
    ) -> Self {
        Self {
            source_path: source_path.into(),
            contract_path: contract_path.into(),
            function_id: function_id.into(),
            upstream_invariants: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyCallSiteTextRequest<'a> {
    pub source: &'a str,
    pub source_path: Option<String>,
    pub contract_json: &'a str,
    pub function_id: &'a str,
    pub upstream_invariants: &'a [PolicyCallSiteInvariant],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyCallSiteInvariant {
    pub expression: String,
    pub summary: String,
}

impl PolicyCallSiteInvariant {
    pub fn new(expression: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            expression: expression.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyCallSiteAnalysis {
    pub function_id: String,
    pub source_path: Option<String>,
    pub preconditions: Vec<PolicyCallSitePreconditionEvidence>,
}

pub fn analyze_policy_call_site(
    request: &PolicyCallSiteAnalysisRequest,
) -> Result<PolicyCallSiteAnalysis, PolicyCallSiteAnalysisError> {
    let source = fs::read_to_string(&request.source_path).map_err(|source| {
        PolicyCallSiteAnalysisError::io(
            format!("read Go source {}", request.source_path.display()),
            source,
        )
    })?;
    let contract_json = fs::read_to_string(&request.contract_path).map_err(|source| {
        PolicyCallSiteAnalysisError::io(
            format!("read contract {}", request.contract_path.display()),
            source,
        )
    })?;
    let source_path = Some(normalize_slashes(
        &request.source_path.display().to_string(),
    ));
    analyze_policy_call_site_text(&PolicyCallSiteTextRequest {
        source: &source,
        source_path,
        contract_json: &contract_json,
        function_id: &request.function_id,
        upstream_invariants: &request.upstream_invariants,
    })
}

pub fn analyze_policy_call_site_text(
    request: &PolicyCallSiteTextRequest<'_>,
) -> Result<PolicyCallSiteAnalysis, PolicyCallSiteAnalysisError> {
    let requirements = contract_requirements(request.contract_json)?;
    let call_sites = visible_call_sites(request.source, request.function_id);
    let mut preconditions = Vec::new();

    if call_sites.is_empty() {
        for requirement in &requirements {
            preconditions.push(PolicyCallSitePreconditionEvidence::new(
                format!("callsite[none].requires[{}]", requirement.index),
                requirement.expression.clone(),
                PolicyCallSitePreconditionStatus::NotObserved,
                request.source_path.clone(),
                Some(request.function_id.to_owned()),
                "helper analysis did not observe a visible policy call site; this is not proof evidence",
            ));
        }
        return Ok(PolicyCallSiteAnalysis {
            function_id: request.function_id.to_owned(),
            source_path: request.source_path.clone(),
            preconditions,
        });
    }

    for (call_index, call_site) in call_sites.iter().enumerate() {
        let unsupported_reason = unsupported_reason(request.source, call_site);
        for requirement in &requirements {
            let argument = call_site.args.get(requirement.index).map(String::as_str);
            let (status, summary) = classify_requirement(
                request.source,
                requirement,
                argument,
                unsupported_reason.as_deref(),
                request.upstream_invariants,
            );
            preconditions.push(PolicyCallSitePreconditionEvidence::new(
                format!("callsite[{call_index}].requires[{}]", requirement.index),
                requirement.expression.clone(),
                status,
                request.source_path.clone(),
                Some(request.function_id.to_owned()),
                summary,
            ));
        }
    }

    Ok(PolicyCallSiteAnalysis {
        function_id: request.function_id.to_owned(),
        source_path: request.source_path.clone(),
        preconditions,
    })
}

fn classify_requirement(
    source: &str,
    requirement: &ContractRequirement,
    argument: Option<&str>,
    unsupported_reason: Option<&str>,
    upstream_invariants: &[PolicyCallSiteInvariant],
) -> (PolicyCallSitePreconditionStatus, String) {
    let Some(argument) = argument else {
        return (
            PolicyCallSitePreconditionStatus::UnsupportedControlFlow,
            "helper analysis could not match the contract precondition to a call argument; this is not proof evidence".to_owned(),
        );
    };

    if let Some(reason) = unsupported_reason {
        return (
            PolicyCallSitePreconditionStatus::UnsupportedControlFlow,
            format!("helper analysis cannot classify this call site: {reason}; this is not proof evidence"),
        );
    }

    if let Some(invariant) = matching_invariant(requirement, argument, upstream_invariants) {
        return (
            PolicyCallSitePreconditionStatus::DeclaredUpstreamInvariant,
            format!(
                "caller-declared upstream invariant: {}; helper analysis only, not proof evidence",
                invariant.summary
            ),
        );
    }

    if let Some(guard) = requirement.guard {
        if observes_rejecting_guard(source, argument, guard) {
            return (
                PolicyCallSitePreconditionStatus::CheckedByLocalGuard,
                format!(
                    "helper analysis observed a local guard for call argument `{}`; this is not proof evidence",
                    argument.trim()
                ),
            );
        }
    } else {
        return (
            PolicyCallSitePreconditionStatus::UnsupportedControlFlow,
            "helper analysis does not support this contract precondition shape; this is not proof evidence".to_owned(),
        );
    }

    (
        PolicyCallSitePreconditionStatus::NotObserved,
        format!(
            "helper analysis did not observe a local guard or explicit upstream invariant for call argument `{}`; this is not proof evidence",
            argument.trim()
        ),
    )
}

fn matching_invariant<'a>(
    requirement: &ContractRequirement,
    argument: &str,
    upstream_invariants: &'a [PolicyCallSiteInvariant],
) -> Option<&'a PolicyCallSiteInvariant> {
    let required = normalize_expression(&requirement.expression);
    let instantiated = requirement
        .parameter
        .as_ref()
        .map(|parameter| requirement.expression.replace(parameter, argument.trim()))
        .map(|expression| normalize_expression(&expression));

    upstream_invariants.iter().find(|invariant| {
        let declared = normalize_expression(&invariant.expression);
        declared == required
            || instantiated
                .as_ref()
                .is_some_and(|value| declared == *value)
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContractRequirement {
    index: usize,
    parameter: Option<String>,
    expression: String,
    guard: Option<GuardPattern>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuardPattern {
    RejectLessThanZero,
    RejectLessOrEqualZero,
}

fn contract_requirements(
    contract_json: &str,
) -> Result<Vec<ContractRequirement>, PolicyCallSiteAnalysisError> {
    let contract =
        serde_json::from_str::<Value>(contract_json).map_err(PolicyCallSiteAnalysisError::Json)?;
    let requires = contract
        .get("requires")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            PolicyCallSiteAnalysisError::InvalidContract(
                "contract JSON must contain a requires array".to_owned(),
            )
        })?;

    Ok(requires
        .iter()
        .enumerate()
        .map(|(index, value)| ContractRequirement {
            index,
            parameter: value
                .get("lhs")
                .and_then(|lhs| lhs.get("var"))
                .and_then(Value::as_str)
                .map(str::to_owned),
            expression: format_contract_expr(value),
            guard: guard_pattern(value),
        })
        .collect())
}

fn guard_pattern(value: &Value) -> Option<GuardPattern> {
    let op = value.get("op").and_then(Value::as_str)?;
    let lhs_is_var = value
        .get("lhs")
        .and_then(|lhs| lhs.get("var"))
        .and_then(Value::as_str)
        .is_some();
    if !lhs_is_var || !rhs_is_zero(value) {
        return None;
    }
    match op {
        "signed_ge" | "unsigned_ge" => Some(GuardPattern::RejectLessThanZero),
        "signed_gt" | "unsigned_gt" => Some(GuardPattern::RejectLessOrEqualZero),
        _ => None,
    }
}

fn rhs_is_zero(value: &Value) -> bool {
    value
        .get("rhs")
        .and_then(|rhs| rhs.get("int"))
        .and_then(|int| int.get("value"))
        .and_then(Value::as_str)
        == Some("0")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CallSite {
    start: usize,
    args: Vec<String>,
}

fn visible_call_sites(source: &str, function_id: &str) -> Vec<CallSite> {
    let function = function_short_name(function_id);
    let pattern = format!("{function}(");
    let mut call_sites = Vec::new();
    let mut offset = 0;

    while let Some(relative) = source[offset..].find(&pattern) {
        let start = offset + relative;
        let open_paren = start + function.len();
        offset = open_paren + 1;

        if is_function_declaration(source, start) {
            continue;
        }
        let Some(args) = parse_call_arguments(source, open_paren) else {
            continue;
        };
        call_sites.push(CallSite { start, args });
    }

    call_sites
}

fn function_short_name(function_id: &str) -> &str {
    function_id
        .rsplit('.')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(function_id)
}

fn is_function_declaration(source: &str, function_start: usize) -> bool {
    let line_start = source[..function_start]
        .rfind('\n')
        .map(|index| index + 1)
        .unwrap_or(0);
    source[line_start..function_start]
        .trim_start()
        .starts_with("func ")
}

fn parse_call_arguments(source: &str, open_paren: usize) -> Option<Vec<String>> {
    let mut args = Vec::new();
    let mut depth = 0usize;
    let mut arg_start = open_paren + 1;

    for (relative, character) in source[open_paren + 1..].char_indices() {
        let index = open_paren + 1 + relative;
        match character {
            '(' | '[' | '{' => depth += 1,
            ')' if depth == 0 => {
                let argument = source[arg_start..index].trim();
                if !argument.is_empty() {
                    args.push(argument.to_owned());
                }
                return Some(args);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
            }
            ',' if depth == 0 => {
                args.push(source[arg_start..index].trim().to_owned());
                arg_start = index + 1;
            }
            _ => {}
        }
    }

    None
}

fn unsupported_reason(source: &str, call_site: &CallSite) -> Option<String> {
    if contains_unsupported_control_flow(source) {
        return Some("loops or unsupported control flow are present in the source".to_owned());
    }
    for argument in &call_site.args {
        if !is_simple_go_path(argument) {
            return Some(format!(
                "call argument `{}` is not a simple identifier or selector",
                argument.trim()
            ));
        }
        if has_alias_assignment(source, argument) {
            return Some(format!(
                "call argument `{}` is assigned through an alias pattern outside helper v0",
                argument.trim()
            ));
        }
    }
    None
}

fn contains_unsupported_control_flow(source: &str) -> bool {
    let compact = format!(" {} ", source.replace(['\n', '\t', '\r'], " "));
    [" for ", " range ", " switch ", " select ", " go "]
        .iter()
        .any(|needle| compact.contains(needle))
}

fn is_simple_go_path(argument: &str) -> bool {
    argument
        .trim()
        .split('.')
        .all(|segment| is_go_identifier(segment.trim()))
}

fn is_go_identifier(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn has_alias_assignment(source: &str, argument: &str) -> bool {
    let trimmed = argument.trim();
    if trimmed.contains('.') || !is_go_identifier(trimmed) {
        return false;
    }
    let compact_source = normalize_expression(source);
    compact_source.contains(&format!("{trimmed}:="))
        || compact_source.contains(&format!("var{trimmed}="))
}

fn observes_rejecting_guard(source: &str, argument: &str, guard: GuardPattern) -> bool {
    let compact_source = normalize_expression(source);
    let compact_argument = normalize_expression(argument);
    match guard {
        GuardPattern::RejectLessThanZero => {
            compact_source.contains(&format!("{compact_argument}<0"))
                || compact_source.contains(&format!("0>{compact_argument}"))
                || compact_source.contains(&format!("{compact_argument}<=-1"))
                || compact_source.contains(&format!("-1>={compact_argument}"))
        }
        GuardPattern::RejectLessOrEqualZero => {
            compact_source.contains(&format!("{compact_argument}<=0"))
                || compact_source.contains(&format!("0>={compact_argument}"))
        }
    }
}

fn format_contract_expr(value: &Value) -> String {
    let Some(op) = value.get("op").and_then(Value::as_str) else {
        return compact_json(value);
    };
    let binary = value
        .get("lhs")
        .zip(value.get("rhs"))
        .and_then(|(lhs, rhs)| Some((format_contract_atom(lhs)?, format_contract_atom(rhs)?)));
    if let Some((lhs, rhs)) = binary {
        let symbol = match op {
            "eq" => "==",
            "signed_ge" | "unsigned_ge" => ">=",
            "signed_gt" | "unsigned_gt" => ">",
            "signed_le" | "unsigned_le" => "<=",
            "signed_lt" | "unsigned_lt" => "<",
            _ => return compact_json(value),
        };
        return format!("{lhs} {symbol} {rhs}");
    }
    compact_json(value)
}

fn format_contract_atom(value: &Value) -> Option<String> {
    if let Some(var) = value.get("var").and_then(Value::as_str) {
        return Some(var.to_owned());
    }
    if let Some(result) = value.get("result").and_then(Value::as_u64) {
        return Some(format!("result[{result}]"));
    }
    if let Some(int) = value.get("int") {
        if let Some(raw) = int.get("value").and_then(Value::as_str) {
            return Some(raw.to_owned());
        }
    }
    if let Some(bool_value) = value.get("bool").and_then(Value::as_bool) {
        return Some(bool_value.to_string());
    }
    None
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_owned())
}

fn normalize_expression(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn normalize_slashes(path: &str) -> String {
    path.replace('\\', "/")
}

#[derive(Debug)]
pub enum PolicyCallSiteAnalysisError {
    Io {
        context: String,
        source: std::io::Error,
    },
    Json(serde_json::Error),
    InvalidContract(String),
}

impl PolicyCallSiteAnalysisError {
    fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Self::Io {
            context: context.into(),
            source,
        }
    }
}

impl fmt::Display for PolicyCallSiteAnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { context, source } => write!(formatter, "{context}: {source}"),
            Self::Json(error) => write!(formatter, "invalid policy contract JSON: {error}"),
            Self::InvalidContract(message) => formatter.write_str(message),
        }
    }
}

impl Error for PolicyCallSiteAnalysisError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json(error) => Some(error),
            Self::InvalidContract(_) => None,
        }
    }
}
