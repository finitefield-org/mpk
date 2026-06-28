//! Bool normalization certificate checker.
//!
//! A bool certificate proves a small boolean formula is a tautology by carrying
//! an explicit normalization row for every assignment. The checker recomputes
//! each row; claimed results are not trusted.

use std::fmt;

pub const BOOL_CERT_FORMAT: &str = "mpk.bool-normalize.v0";
const BOOL_CERT_MAGIC: &[u8; 8] = b"MPKBOOL0";
pub const MAX_BOOL_VARIABLES: u8 = 8;
pub const MAX_BOOL_EXPR_NODES: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoolCertificate {
    pub variable_count: u8,
    pub root: BoolExpr,
    pub rows: Vec<BoolCertificateRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoolCertificateRow {
    pub assignment: Vec<bool>,
    pub normalized_value: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BoolExpr {
    Const(bool),
    Var(u8),
    Not(Box<BoolExpr>),
    And(Box<BoolExpr>, Box<BoolExpr>),
    Or(Box<BoolExpr>, Box<BoolExpr>),
    Implies(Box<BoolExpr>, Box<BoolExpr>),
    Iff(Box<BoolExpr>, Box<BoolExpr>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoolCertificateSummary {
    pub variable_count: u8,
    pub expression_nodes: usize,
    pub rows_checked: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoolCertError {
    kind: BoolCertErrorKind,
    detail: String,
}

impl BoolCertError {
    pub fn kind(&self) -> BoolCertErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: BoolCertErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for BoolCertError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.kind.as_str(), self.detail)
    }
}

impl std::error::Error for BoolCertError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BoolCertErrorKind {
    InvalidPayload,
    TooManyVariables,
    ExpressionTooLarge,
    InvalidVariableReference,
    RowCountMismatch,
    AssignmentArityMismatch,
    DuplicateAssignment,
    MissingAssignment,
    ClaimedResultMismatch,
    NotTautology,
}

impl BoolCertErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidPayload => "BOOL_CERT_INVALID_PAYLOAD",
            Self::TooManyVariables => "BOOL_CERT_TOO_MANY_VARIABLES",
            Self::ExpressionTooLarge => "BOOL_CERT_EXPRESSION_TOO_LARGE",
            Self::InvalidVariableReference => "BOOL_CERT_INVALID_VARIABLE_REFERENCE",
            Self::RowCountMismatch => "BOOL_CERT_ROW_COUNT_MISMATCH",
            Self::AssignmentArityMismatch => "BOOL_CERT_ASSIGNMENT_ARITY_MISMATCH",
            Self::DuplicateAssignment => "BOOL_CERT_DUPLICATE_ASSIGNMENT",
            Self::MissingAssignment => "BOOL_CERT_MISSING_ASSIGNMENT",
            Self::ClaimedResultMismatch => "BOOL_CERT_CLAIMED_RESULT_MISMATCH",
            Self::NotTautology => "BOOL_CERT_NOT_TAUTOLOGY",
        }
    }
}

pub fn check_bool_certificate_payload(
    payload: &[u8],
) -> Result<BoolCertificateSummary, BoolCertError> {
    let certificate = decode_bool_certificate(payload)?;
    check_bool_certificate(&certificate)
}

pub fn decode_bool_certificate(payload: &[u8]) -> Result<BoolCertificate, BoolCertError> {
    let mut decoder = BoolPayloadDecoder::new(payload);
    decoder.read_magic()?;
    let variable_count = decoder.read_u8("variable_count")?;
    let root = decoder.read_expr()?;
    let row_count = decoder.read_u16("row_count")?;
    let mut rows = Vec::with_capacity(usize::from(row_count));
    for row_index in 0..row_count {
        let assignment_mask = decoder.read_u8("assignment_mask")?;
        let normalized_value = decoder.read_bool("normalized_value")?;
        rows.push(BoolCertificateRow {
            assignment: assignment_from_mask(variable_count, assignment_mask, row_index)?,
            normalized_value,
        });
    }
    decoder.finish()?;

    Ok(BoolCertificate {
        variable_count,
        root,
        rows,
    })
}

pub fn check_bool_certificate(
    certificate: &BoolCertificate,
) -> Result<BoolCertificateSummary, BoolCertError> {
    validate_variable_count(certificate.variable_count)?;
    let node_count = certificate.root.node_count();
    if node_count > MAX_BOOL_EXPR_NODES {
        return Err(BoolCertError::new(
            BoolCertErrorKind::ExpressionTooLarge,
            format!("nodes={node_count}; max={MAX_BOOL_EXPR_NODES}"),
        ));
    }
    certificate
        .root
        .validate_variables(certificate.variable_count)?;

    let expected_rows = expected_row_count(certificate.variable_count);
    if certificate.rows.len() != expected_rows {
        return Err(BoolCertError::new(
            BoolCertErrorKind::RowCountMismatch,
            format!(
                "expected={expected_rows}; actual={}",
                certificate.rows.len()
            ),
        ));
    }

    let mut seen = vec![false; expected_rows];
    for (row_index, row) in certificate.rows.iter().enumerate() {
        if row.assignment.len() != usize::from(certificate.variable_count) {
            return Err(BoolCertError::new(
                BoolCertErrorKind::AssignmentArityMismatch,
                format!(
                    "row={row_index}; expected={}; actual={}",
                    certificate.variable_count,
                    row.assignment.len()
                ),
            ));
        }

        let assignment_index = assignment_index(&row.assignment);
        if seen[assignment_index] {
            return Err(BoolCertError::new(
                BoolCertErrorKind::DuplicateAssignment,
                format!("row={row_index}; assignment={assignment_index}"),
            ));
        }
        seen[assignment_index] = true;

        let actual = certificate.root.eval(&row.assignment);
        if actual != row.normalized_value {
            return Err(BoolCertError::new(
                BoolCertErrorKind::ClaimedResultMismatch,
                format!(
                    "row={row_index}; assignment={assignment_index}; claimed={}; actual={actual}",
                    row.normalized_value
                ),
            ));
        }
        if !actual {
            return Err(BoolCertError::new(
                BoolCertErrorKind::NotTautology,
                format!("row={row_index}; assignment={assignment_index}"),
            ));
        }
    }

    if let Some(missing) = seen.iter().position(|present| !present) {
        return Err(BoolCertError::new(
            BoolCertErrorKind::MissingAssignment,
            format!("assignment={missing}"),
        ));
    }

    Ok(BoolCertificateSummary {
        variable_count: certificate.variable_count,
        expression_nodes: node_count,
        rows_checked: certificate.rows.len(),
    })
}

impl BoolExpr {
    fn eval(&self, assignment: &[bool]) -> bool {
        match self {
            Self::Const(value) => *value,
            Self::Var(index) => assignment[usize::from(*index)],
            Self::Not(inner) => !inner.eval(assignment),
            Self::And(lhs, rhs) => lhs.eval(assignment) && rhs.eval(assignment),
            Self::Or(lhs, rhs) => lhs.eval(assignment) || rhs.eval(assignment),
            Self::Implies(lhs, rhs) => !lhs.eval(assignment) || rhs.eval(assignment),
            Self::Iff(lhs, rhs) => lhs.eval(assignment) == rhs.eval(assignment),
        }
    }

    fn node_count(&self) -> usize {
        match self {
            Self::Const(_) | Self::Var(_) => 1,
            Self::Not(inner) => 1 + inner.node_count(),
            Self::And(lhs, rhs)
            | Self::Or(lhs, rhs)
            | Self::Implies(lhs, rhs)
            | Self::Iff(lhs, rhs) => 1 + lhs.node_count() + rhs.node_count(),
        }
    }

    fn validate_variables(&self, variable_count: u8) -> Result<(), BoolCertError> {
        match self {
            Self::Const(_) => Ok(()),
            Self::Var(index) if *index < variable_count => Ok(()),
            Self::Var(index) => Err(BoolCertError::new(
                BoolCertErrorKind::InvalidVariableReference,
                format!("var={index}; variable_count={variable_count}"),
            )),
            Self::Not(inner) => inner.validate_variables(variable_count),
            Self::And(lhs, rhs)
            | Self::Or(lhs, rhs)
            | Self::Implies(lhs, rhs)
            | Self::Iff(lhs, rhs) => {
                lhs.validate_variables(variable_count)?;
                rhs.validate_variables(variable_count)
            }
        }
    }
}

struct BoolPayloadDecoder<'payload> {
    payload: &'payload [u8],
    offset: usize,
    decoded_nodes: usize,
}

impl<'payload> BoolPayloadDecoder<'payload> {
    fn new(payload: &'payload [u8]) -> Self {
        Self {
            payload,
            offset: 0,
            decoded_nodes: 0,
        }
    }

    fn read_magic(&mut self) -> Result<(), BoolCertError> {
        let bytes = self.read_exact(BOOL_CERT_MAGIC.len(), "magic")?;
        if bytes == BOOL_CERT_MAGIC {
            return Ok(());
        }

        Err(BoolCertError::new(
            BoolCertErrorKind::InvalidPayload,
            "invalid magic",
        ))
    }

    fn read_expr(&mut self) -> Result<BoolExpr, BoolCertError> {
        self.decoded_nodes += 1;
        if self.decoded_nodes > MAX_BOOL_EXPR_NODES {
            return Err(BoolCertError::new(
                BoolCertErrorKind::ExpressionTooLarge,
                format!("nodes>{MAX_BOOL_EXPR_NODES}"),
            ));
        }

        let tag = self.read_u8("expr_tag")?;
        match tag {
            0x00 => Ok(BoolExpr::Const(false)),
            0x01 => Ok(BoolExpr::Const(true)),
            0x02 => Ok(BoolExpr::Var(self.read_u8("var_index")?)),
            0x03 => Ok(BoolExpr::Not(Box::new(self.read_expr()?))),
            0x04 => Ok(BoolExpr::And(
                Box::new(self.read_expr()?),
                Box::new(self.read_expr()?),
            )),
            0x05 => Ok(BoolExpr::Or(
                Box::new(self.read_expr()?),
                Box::new(self.read_expr()?),
            )),
            0x06 => Ok(BoolExpr::Implies(
                Box::new(self.read_expr()?),
                Box::new(self.read_expr()?),
            )),
            0x07 => Ok(BoolExpr::Iff(
                Box::new(self.read_expr()?),
                Box::new(self.read_expr()?),
            )),
            _ => Err(BoolCertError::new(
                BoolCertErrorKind::InvalidPayload,
                format!("unknown expr tag=0x{tag:02x}; offset={}", self.offset - 1),
            )),
        }
    }

    fn read_bool(&mut self, field: &'static str) -> Result<bool, BoolCertError> {
        match self.read_u8(field)? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(BoolCertError::new(
                BoolCertErrorKind::InvalidPayload,
                format!("invalid bool {field}={value}; offset={}", self.offset - 1),
            )),
        }
    }

    fn read_u8(&mut self, field: &'static str) -> Result<u8, BoolCertError> {
        Ok(self.read_exact(1, field)?[0])
    }

    fn read_u16(&mut self, field: &'static str) -> Result<u16, BoolCertError> {
        let bytes = self.read_exact(2, field)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_exact(
        &mut self,
        len: usize,
        field: &'static str,
    ) -> Result<&'payload [u8], BoolCertError> {
        let start = self.offset;
        let end = start.checked_add(len).ok_or_else(|| {
            BoolCertError::new(
                BoolCertErrorKind::InvalidPayload,
                format!("offset overflow while reading {field}"),
            )
        })?;
        let bytes = self.payload.get(start..end).ok_or_else(|| {
            BoolCertError::new(
                BoolCertErrorKind::InvalidPayload,
                format!("unexpected EOF while reading {field}; offset={start}; len={len}"),
            )
        })?;
        self.offset = end;
        Ok(bytes)
    }

    fn finish(&self) -> Result<(), BoolCertError> {
        if self.offset == self.payload.len() {
            return Ok(());
        }

        Err(BoolCertError::new(
            BoolCertErrorKind::InvalidPayload,
            format!(
                "trailing bytes at offset={}; len={}",
                self.offset,
                self.payload.len()
            ),
        ))
    }
}

fn validate_variable_count(variable_count: u8) -> Result<(), BoolCertError> {
    if variable_count <= MAX_BOOL_VARIABLES {
        return Ok(());
    }

    Err(BoolCertError::new(
        BoolCertErrorKind::TooManyVariables,
        format!("variable_count={variable_count}; max={MAX_BOOL_VARIABLES}"),
    ))
}

fn expected_row_count(variable_count: u8) -> usize {
    1usize << usize::from(variable_count)
}

fn assignment_index(assignment: &[bool]) -> usize {
    assignment.iter().enumerate().fold(
        0usize,
        |mask, (index, bit)| {
            if *bit {
                mask | (1usize << index)
            } else {
                mask
            }
        },
    )
}

fn assignment_from_mask(
    variable_count: u8,
    mask: u8,
    row_index: u16,
) -> Result<Vec<bool>, BoolCertError> {
    validate_variable_count(variable_count)?;
    let unused_bits = if variable_count == 8 {
        0
    } else {
        mask >> variable_count
    };
    if unused_bits != 0 {
        return Err(BoolCertError::new(
            BoolCertErrorKind::AssignmentArityMismatch,
            format!("row={row_index}; mask={mask}; variable_count={variable_count}"),
        ));
    }
    Ok((0..variable_count)
        .map(|index| mask & (1u8 << index) != 0)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tautology_expr() -> BoolExpr {
        BoolExpr::Implies(
            Box::new(BoolExpr::And(
                Box::new(BoolExpr::Var(0)),
                Box::new(BoolExpr::Var(1)),
            )),
            Box::new(BoolExpr::Var(0)),
        )
    }

    fn rows_for(variable_count: u8, root: &BoolExpr) -> Vec<BoolCertificateRow> {
        (0..expected_row_count(variable_count))
            .map(|mask| {
                let assignment = (0..variable_count)
                    .map(|index| mask & (1usize << index) != 0)
                    .collect::<Vec<_>>();
                BoolCertificateRow {
                    normalized_value: root.eval(&assignment),
                    assignment,
                }
            })
            .collect()
    }

    fn encode_payload(variable_count: u8, expr: &[u8], rows: &[(u8, bool)]) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(BOOL_CERT_MAGIC);
        payload.push(variable_count);
        payload.extend_from_slice(expr);
        payload.extend_from_slice(&(rows.len() as u16).to_be_bytes());
        for (mask, value) in rows {
            payload.push(*mask);
            payload.push(u8::from(*value));
        }
        payload
    }

    #[test]
    fn accepts_small_bool_tautology_certificate() {
        let root = tautology_expr();
        let certificate = BoolCertificate {
            variable_count: 2,
            rows: rows_for(2, &root),
            root,
        };

        let summary = check_bool_certificate(&certificate).expect("tautology checks");

        assert_eq!(
            summary,
            BoolCertificateSummary {
                variable_count: 2,
                expression_nodes: 5,
                rows_checked: 4,
            }
        );
    }

    #[test]
    fn decodes_and_checks_payload_certificate() {
        let payload = encode_payload(
            2,
            &[0x06, 0x04, 0x02, 0x00, 0x02, 0x01, 0x02, 0x00],
            &[(0, true), (1, true), (2, true), (3, true)],
        );

        let summary = check_bool_certificate_payload(&payload).expect("payload checks");

        assert_eq!(summary.rows_checked, 4);
        assert_eq!(summary.expression_nodes, 5);
    }

    #[test]
    fn rejects_non_tautology_certificate() {
        let root = BoolExpr::Var(0);
        let certificate = BoolCertificate {
            variable_count: 1,
            rows: rows_for(1, &root),
            root,
        };

        let error = check_bool_certificate(&certificate).expect_err("non-tautology rejects");

        assert_eq!(error.kind(), BoolCertErrorKind::NotTautology);
        assert_eq!(error.detail(), "row=0; assignment=0");
    }

    #[test]
    fn rejects_claimed_result_mismatch() {
        let root = BoolExpr::Const(true);
        let certificate = BoolCertificate {
            variable_count: 0,
            root,
            rows: vec![BoolCertificateRow {
                assignment: Vec::new(),
                normalized_value: false,
            }],
        };

        let error = check_bool_certificate(&certificate).expect_err("bad claim rejects");

        assert_eq!(error.kind(), BoolCertErrorKind::ClaimedResultMismatch);
        assert_eq!(
            error.detail(),
            "row=0; assignment=0; claimed=false; actual=true"
        );
    }

    #[test]
    fn rejects_duplicate_assignment() {
        let root = BoolExpr::Const(true);
        let certificate = BoolCertificate {
            variable_count: 1,
            root,
            rows: vec![
                BoolCertificateRow {
                    assignment: vec![false],
                    normalized_value: true,
                },
                BoolCertificateRow {
                    assignment: vec![false],
                    normalized_value: true,
                },
            ],
        };

        let error = check_bool_certificate(&certificate).expect_err("duplicate rejects");

        assert_eq!(error.kind(), BoolCertErrorKind::DuplicateAssignment);
        assert_eq!(error.detail(), "row=1; assignment=0");
    }

    #[test]
    fn rejects_invalid_variable_reference() {
        let certificate = BoolCertificate {
            variable_count: 1,
            root: BoolExpr::Var(1),
            rows: vec![
                BoolCertificateRow {
                    assignment: vec![false],
                    normalized_value: true,
                },
                BoolCertificateRow {
                    assignment: vec![true],
                    normalized_value: true,
                },
            ],
        };

        let error = check_bool_certificate(&certificate).expect_err("bad var rejects");

        assert_eq!(error.kind(), BoolCertErrorKind::InvalidVariableReference);
        assert_eq!(error.detail(), "var=1; variable_count=1");
    }

    #[test]
    fn rejects_malformed_payloads() {
        let mut bad_tag = encode_payload(0, &[0xff], &[(0, true)]);
        let error = decode_bool_certificate(&bad_tag).expect_err("bad tag rejects");
        assert_eq!(error.kind(), BoolCertErrorKind::InvalidPayload);
        assert_eq!(error.detail(), "unknown expr tag=0xff; offset=9");

        bad_tag = encode_payload(0, &[0x01], &[(0, true)]);
        bad_tag.push(0);
        let error = decode_bool_certificate(&bad_tag).expect_err("trailing bytes reject");
        assert_eq!(error.kind(), BoolCertErrorKind::InvalidPayload);
        assert_eq!(error.detail(), "trailing bytes at offset=14; len=15");

        let invalid_bool = encode_payload(0, &[0x01], &[(0, true)])
            .into_iter()
            .enumerate()
            .map(|(index, value)| if index == 13 { 2 } else { value })
            .collect::<Vec<_>>();
        let error = decode_bool_certificate(&invalid_bool).expect_err("invalid bool rejects");
        assert_eq!(error.kind(), BoolCertErrorKind::InvalidPayload);
        assert_eq!(error.detail(), "invalid bool normalized_value=2; offset=13");
    }
}
