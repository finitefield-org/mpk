//! Fixed-array read/write certificate checker.
//!
//! The checker normalizes ground read/write queries over symbolic fixed arrays.
//! Claims are accepted only when recomputation matches the requested result.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const ARRAY_CERT_FORMAT: &str = "mpk.array-read-write.v0";
pub const MAX_ARRAY_BASES: usize = 64;
pub const MAX_ARRAY_CLAIMS: usize = 64;
pub const MAX_ARRAY_EXPR_NODES: usize = 256;
pub const MAX_ARRAY_ELEMENT_SYMBOLS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrayCertificate {
    pub base_arrays: Vec<BaseArray>,
    pub claims: Vec<ArrayClaim>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct BaseArray {
    pub array_id: u32,
    pub length: u32,
}

impl BaseArray {
    pub fn new(array_id: u32, length: u32) -> Self {
        Self { array_id, length }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrayClaim {
    pub query: ArrayQuery,
    pub expected: ArrayResult,
}

impl ArrayClaim {
    pub fn new(query: ArrayQuery, expected: ArrayResult) -> Self {
        Self { query, expected }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArrayQuery {
    Read { array: ArrayExpr, index: u32 },
    Length { array: ArrayExpr },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArrayExpr {
    Base {
        array_id: u32,
    },
    Write {
        array: Box<ArrayExpr>,
        index: u32,
        value: ArrayElement,
    },
}

impl ArrayExpr {
    pub fn base(array_id: u32) -> Self {
        Self::Base { array_id }
    }

    pub fn write(array: ArrayExpr, index: u32, value: ArrayElement) -> Self {
        Self::Write {
            array: Box::new(array),
            index,
            value,
        }
    }

    fn node_count(&self) -> usize {
        match self {
            Self::Base { .. } => 1,
            Self::Write { array, .. } => 1 + array.node_count(),
        }
    }

    fn write_depth(&self) -> usize {
        match self {
            Self::Base { .. } => 0,
            Self::Write { array, .. } => 1 + array.write_depth(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ArrayElement {
    Symbol(u32),
    BaseRead { array_id: u32, index: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArrayResult {
    Element(ArrayElement),
    Length(u32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrayCertificateSummary {
    pub base_array_count: usize,
    pub claims_checked: usize,
    pub expression_nodes: usize,
    pub max_write_depth: usize,
    pub element_symbol_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrayCertError {
    kind: ArrayCertErrorKind,
    detail: String,
}

impl ArrayCertError {
    pub fn kind(&self) -> ArrayCertErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: ArrayCertErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for ArrayCertError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.kind.as_str(), self.detail)
    }
}

impl std::error::Error for ArrayCertError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ArrayCertErrorKind {
    TooManyBaseArrays,
    TooManyClaims,
    TooManyElementSymbols,
    ExpressionTooLarge,
    DuplicateBaseArray,
    UnknownBaseArray,
    ReadOutOfBounds,
    WriteOutOfBounds,
    ResultKindMismatch,
    ClaimedResultMismatch,
}

impl ArrayCertErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TooManyBaseArrays => "ARRAY_TOO_MANY_BASE_ARRAYS",
            Self::TooManyClaims => "ARRAY_TOO_MANY_CLAIMS",
            Self::TooManyElementSymbols => "ARRAY_TOO_MANY_ELEMENT_SYMBOLS",
            Self::ExpressionTooLarge => "ARRAY_EXPRESSION_TOO_LARGE",
            Self::DuplicateBaseArray => "ARRAY_DUPLICATE_BASE_ARRAY",
            Self::UnknownBaseArray => "ARRAY_UNKNOWN_BASE_ARRAY",
            Self::ReadOutOfBounds => "ARRAY_READ_OUT_OF_BOUNDS",
            Self::WriteOutOfBounds => "ARRAY_WRITE_OUT_OF_BOUNDS",
            Self::ResultKindMismatch => "ARRAY_RESULT_KIND_MISMATCH",
            Self::ClaimedResultMismatch => "ARRAY_CLAIMED_RESULT_MISMATCH",
        }
    }
}

pub fn check_array_certificate(
    certificate: &ArrayCertificate,
) -> Result<ArrayCertificateSummary, ArrayCertError> {
    if certificate.base_arrays.len() > MAX_ARRAY_BASES {
        return Err(ArrayCertError::new(
            ArrayCertErrorKind::TooManyBaseArrays,
            format!(
                "base_arrays={}; max={MAX_ARRAY_BASES}",
                certificate.base_arrays.len()
            ),
        ));
    }
    if certificate.claims.len() > MAX_ARRAY_CLAIMS {
        return Err(ArrayCertError::new(
            ArrayCertErrorKind::TooManyClaims,
            format!(
                "claims={}; max={MAX_ARRAY_CLAIMS}",
                certificate.claims.len()
            ),
        ));
    }

    let context = ArrayContext::new(&certificate.base_arrays)?;
    let mut summary = ArrayCertificateSummary {
        base_array_count: certificate.base_arrays.len(),
        claims_checked: 0,
        expression_nodes: 0,
        max_write_depth: 0,
        element_symbol_count: 0,
    };
    let mut element_symbols = BTreeSet::new();

    for (claim_index, claim) in certificate.claims.iter().enumerate() {
        let nodes = claim.query.node_count();
        if nodes > MAX_ARRAY_EXPR_NODES {
            return Err(ArrayCertError::new(
                ArrayCertErrorKind::ExpressionTooLarge,
                format!("claim={claim_index}; nodes={nodes}; max={MAX_ARRAY_EXPR_NODES}"),
            ));
        }
        summary.expression_nodes += nodes;
        summary.max_write_depth = summary.max_write_depth.max(claim.query.write_depth());

        collect_query_symbols(&claim.query, &mut element_symbols);
        collect_result_symbols(&claim.expected, &mut element_symbols);
        if element_symbols.len() > MAX_ARRAY_ELEMENT_SYMBOLS {
            return Err(ArrayCertError::new(
                ArrayCertErrorKind::TooManyElementSymbols,
                format!(
                    "claim={claim_index}; symbols={}; max={MAX_ARRAY_ELEMENT_SYMBOLS}",
                    element_symbols.len()
                ),
            ));
        }

        validate_expected(&context, &claim.expected, claim_index)?;
        let actual = normalize_query(&context, &claim.query, claim_index)?;
        ensure_same_result_kind(actual, claim.expected, claim_index)?;
        if actual != claim.expected {
            return Err(ArrayCertError::new(
                ArrayCertErrorKind::ClaimedResultMismatch,
                format!(
                    "claim={claim_index}; expected={:?}; actual={actual:?}",
                    claim.expected
                ),
            ));
        }
        summary.claims_checked += 1;
    }

    summary.element_symbol_count = element_symbols.len();
    Ok(summary)
}

impl ArrayQuery {
    fn node_count(&self) -> usize {
        match self {
            Self::Read { array, .. } | Self::Length { array } => array.node_count(),
        }
    }

    fn write_depth(&self) -> usize {
        match self {
            Self::Read { array, .. } | Self::Length { array } => array.write_depth(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ArrayContext {
    lengths: BTreeMap<u32, u32>,
}

impl ArrayContext {
    fn new(base_arrays: &[BaseArray]) -> Result<Self, ArrayCertError> {
        let mut lengths = BTreeMap::new();
        for base in base_arrays {
            if lengths.insert(base.array_id, base.length).is_some() {
                return Err(ArrayCertError::new(
                    ArrayCertErrorKind::DuplicateBaseArray,
                    format!("array_id={}", base.array_id),
                ));
            }
        }
        Ok(Self { lengths })
    }

    fn length_of_base(&self, array_id: u32) -> Result<u32, ArrayCertError> {
        self.lengths.get(&array_id).copied().ok_or_else(|| {
            ArrayCertError::new(
                ArrayCertErrorKind::UnknownBaseArray,
                format!("array_id={array_id}"),
            )
        })
    }
}

fn normalize_query(
    context: &ArrayContext,
    query: &ArrayQuery,
    claim_index: usize,
) -> Result<ArrayResult, ArrayCertError> {
    match query {
        ArrayQuery::Read { array, index } => {
            let length = normalize_length(context, array, claim_index)?;
            ensure_read_index_in_bounds(*index, length, claim_index)?;
            normalize_read(context, array, *index, claim_index).map(ArrayResult::Element)
        }
        ArrayQuery::Length { array } => {
            normalize_length(context, array, claim_index).map(ArrayResult::Length)
        }
    }
}

fn normalize_length(
    context: &ArrayContext,
    array: &ArrayExpr,
    claim_index: usize,
) -> Result<u32, ArrayCertError> {
    match array {
        ArrayExpr::Base { array_id } => context.length_of_base(*array_id),
        ArrayExpr::Write {
            array,
            index,
            value,
        } => {
            let length = normalize_length(context, array, claim_index)?;
            ensure_write_index_in_bounds(*index, length, claim_index)?;
            validate_element(context, value, claim_index)?;
            Ok(length)
        }
    }
}

fn normalize_read(
    context: &ArrayContext,
    array: &ArrayExpr,
    index: u32,
    claim_index: usize,
) -> Result<ArrayElement, ArrayCertError> {
    match array {
        ArrayExpr::Base { array_id } => Ok(ArrayElement::BaseRead {
            array_id: *array_id,
            index,
        }),
        ArrayExpr::Write {
            array,
            index: write_index,
            value,
        } => {
            if index == *write_index {
                validate_element(context, value, claim_index)?;
                Ok(*value)
            } else {
                normalize_read(context, array, index, claim_index)
            }
        }
    }
}

fn validate_expected(
    context: &ArrayContext,
    expected: &ArrayResult,
    claim_index: usize,
) -> Result<(), ArrayCertError> {
    match expected {
        ArrayResult::Element(element) => validate_element(context, element, claim_index),
        ArrayResult::Length(_) => Ok(()),
    }
}

fn validate_element(
    context: &ArrayContext,
    element: &ArrayElement,
    claim_index: usize,
) -> Result<(), ArrayCertError> {
    match element {
        ArrayElement::Symbol(_) => Ok(()),
        ArrayElement::BaseRead { array_id, index } => {
            let length = context.length_of_base(*array_id)?;
            ensure_read_index_in_bounds(*index, length, claim_index)
        }
    }
}

fn ensure_same_result_kind(
    actual: ArrayResult,
    expected: ArrayResult,
    claim_index: usize,
) -> Result<(), ArrayCertError> {
    match (actual, expected) {
        (ArrayResult::Element(_), ArrayResult::Element(_))
        | (ArrayResult::Length(_), ArrayResult::Length(_)) => Ok(()),
        (actual, expected) => Err(ArrayCertError::new(
            ArrayCertErrorKind::ResultKindMismatch,
            format!("claim={claim_index}; expected={expected:?}; actual={actual:?}"),
        )),
    }
}

fn ensure_read_index_in_bounds(
    index: u32,
    length: u32,
    claim_index: usize,
) -> Result<(), ArrayCertError> {
    if index < length {
        return Ok(());
    }

    Err(ArrayCertError::new(
        ArrayCertErrorKind::ReadOutOfBounds,
        format!("claim={claim_index}; index={index}; length={length}"),
    ))
}

fn ensure_write_index_in_bounds(
    index: u32,
    length: u32,
    claim_index: usize,
) -> Result<(), ArrayCertError> {
    if index < length {
        return Ok(());
    }

    Err(ArrayCertError::new(
        ArrayCertErrorKind::WriteOutOfBounds,
        format!("claim={claim_index}; index={index}; length={length}"),
    ))
}

fn collect_query_symbols(query: &ArrayQuery, symbols: &mut BTreeSet<u32>) {
    match query {
        ArrayQuery::Read { array, .. } | ArrayQuery::Length { array } => {
            collect_expr_symbols(array, symbols);
        }
    }
}

fn collect_expr_symbols(expr: &ArrayExpr, symbols: &mut BTreeSet<u32>) {
    match expr {
        ArrayExpr::Base { .. } => {}
        ArrayExpr::Write { array, value, .. } => {
            collect_expr_symbols(array, symbols);
            collect_element_symbols(value, symbols);
        }
    }
}

fn collect_result_symbols(result: &ArrayResult, symbols: &mut BTreeSet<u32>) {
    if let ArrayResult::Element(element) = result {
        collect_element_symbols(element, symbols);
    }
}

fn collect_element_symbols(element: &ArrayElement, symbols: &mut BTreeSet<u32>) {
    if let ArrayElement::Symbol(symbol) = element {
        symbols.insert(*symbol);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cert(claims: Vec<ArrayClaim>) -> ArrayCertificate {
        ArrayCertificate {
            base_arrays: vec![BaseArray::new(0, 3)],
            claims,
        }
    }

    fn base() -> ArrayExpr {
        ArrayExpr::base(0)
    }

    fn write(index: u32, value: u32) -> ArrayExpr {
        ArrayExpr::write(base(), index, ArrayElement::Symbol(value))
    }

    fn read(array: ArrayExpr, index: u32, expected: ArrayElement) -> ArrayClaim {
        ArrayClaim::new(
            ArrayQuery::Read { array, index },
            ArrayResult::Element(expected),
        )
    }

    fn length(array: ArrayExpr, expected: u32) -> ArrayClaim {
        ArrayClaim::new(ArrayQuery::Length { array }, ArrayResult::Length(expected))
    }

    #[test]
    fn accepts_same_index_read_after_write() {
        let certificate = cert(vec![read(write(1, 42), 1, ArrayElement::Symbol(42))]);

        let summary = check_array_certificate(&certificate).expect("array certificate checks");

        assert_eq!(
            summary,
            ArrayCertificateSummary {
                base_array_count: 1,
                claims_checked: 1,
                expression_nodes: 2,
                max_write_depth: 1,
                element_symbol_count: 1,
            }
        );
    }

    #[test]
    fn accepts_different_index_read_after_write() {
        let certificate = cert(vec![read(
            write(1, 42),
            2,
            ArrayElement::BaseRead {
                array_id: 0,
                index: 2,
            },
        )]);

        let summary = check_array_certificate(&certificate).expect("array certificate checks");

        assert_eq!(summary.claims_checked, 1);
        assert_eq!(summary.element_symbol_count, 1);
    }

    #[test]
    fn accepts_nested_write_read_normalization() {
        let array = ArrayExpr::write(write(1, 10), 2, ArrayElement::Symbol(20));
        let certificate = cert(vec![
            read(array.clone(), 1, ArrayElement::Symbol(10)),
            read(array, 2, ArrayElement::Symbol(20)),
        ]);

        let summary = check_array_certificate(&certificate).expect("array certificate checks");

        assert_eq!(summary.claims_checked, 2);
        assert_eq!(summary.max_write_depth, 2);
        assert_eq!(summary.element_symbol_count, 2);
    }

    #[test]
    fn accepts_last_write_for_same_index() {
        let array = ArrayExpr::write(write(1, 10), 1, ArrayElement::Symbol(20));
        let certificate = cert(vec![read(array, 1, ArrayElement::Symbol(20))]);

        let summary = check_array_certificate(&certificate).expect("array certificate checks");

        assert_eq!(summary.max_write_depth, 2);
    }

    #[test]
    fn accepts_length_preserved_by_write() {
        let certificate = cert(vec![length(write(1, 42), 3)]);

        let summary = check_array_certificate(&certificate).expect("length certificate checks");

        assert_eq!(summary.claims_checked, 1);
        assert_eq!(summary.element_symbol_count, 1);
    }

    #[test]
    fn rejects_claimed_element_mismatch() {
        let certificate = cert(vec![read(write(1, 42), 1, ArrayElement::Symbol(7))]);

        let error = check_array_certificate(&certificate).expect_err("bad claim rejects");

        assert_eq!(error.kind(), ArrayCertErrorKind::ClaimedResultMismatch);
        assert_eq!(
            error.detail(),
            "claim=0; expected=Element(Symbol(7)); actual=Element(Symbol(42))"
        );
    }

    #[test]
    fn rejects_result_kind_mismatch() {
        let certificate = cert(vec![ArrayClaim::new(
            ArrayQuery::Read {
                array: write(1, 42),
                index: 1,
            },
            ArrayResult::Length(3),
        )]);

        let error = check_array_certificate(&certificate).expect_err("kind mismatch rejects");

        assert_eq!(error.kind(), ArrayCertErrorKind::ResultKindMismatch);
        assert_eq!(
            error.detail(),
            "claim=0; expected=Length(3); actual=Element(Symbol(42))"
        );
    }

    #[test]
    fn rejects_out_of_bounds_read() {
        let certificate = cert(vec![read(
            base(),
            3,
            ArrayElement::BaseRead {
                array_id: 0,
                index: 3,
            },
        )]);

        let error = check_array_certificate(&certificate).expect_err("oob read rejects");

        assert_eq!(error.kind(), ArrayCertErrorKind::ReadOutOfBounds);
        assert_eq!(error.detail(), "claim=0; index=3; length=3");
    }

    #[test]
    fn rejects_out_of_bounds_write() {
        let certificate = cert(vec![length(write(3, 42), 3)]);

        let error = check_array_certificate(&certificate).expect_err("oob write rejects");

        assert_eq!(error.kind(), ArrayCertErrorKind::WriteOutOfBounds);
        assert_eq!(error.detail(), "claim=0; index=3; length=3");
    }

    #[test]
    fn rejects_unknown_base_array() {
        let certificate = cert(vec![read(ArrayExpr::base(1), 0, ArrayElement::Symbol(0))]);

        let error = check_array_certificate(&certificate).expect_err("unknown base rejects");

        assert_eq!(error.kind(), ArrayCertErrorKind::UnknownBaseArray);
        assert_eq!(error.detail(), "array_id=1");
    }

    #[test]
    fn rejects_duplicate_base_array() {
        let certificate = ArrayCertificate {
            base_arrays: vec![BaseArray::new(0, 3), BaseArray::new(0, 4)],
            claims: Vec::new(),
        };

        let error = check_array_certificate(&certificate).expect_err("duplicate rejects");

        assert_eq!(error.kind(), ArrayCertErrorKind::DuplicateBaseArray);
        assert_eq!(error.detail(), "array_id=0");
    }

    #[test]
    fn rejects_oversized_expression() {
        let mut array = base();
        for index in 0..MAX_ARRAY_EXPR_NODES {
            array = ArrayExpr::write(
                array,
                (index % 3) as u32,
                ArrayElement::Symbol(index as u32),
            );
        }
        let certificate = cert(vec![length(array, 3)]);

        let error = check_array_certificate(&certificate).expect_err("large expression rejects");

        assert_eq!(error.kind(), ArrayCertErrorKind::ExpressionTooLarge);
        assert_eq!(error.detail(), "claim=0; nodes=257; max=256");
    }
}
