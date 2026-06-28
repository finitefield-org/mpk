//! Theory-certificate dispatch for `ProofNode::Theory`.
//!
//! The dispatcher validates theory evidence only. The proof checker still
//! requires an independently core-checked witness term for the node's requested
//! type, so accepting a theory payload does not introduce a new trusted term
//! constructor or a hidden axiom.

use std::fmt;

use mpk_cert::encode::TheoryCertificate;
use mpk_theory::{
    check_array_certificate, check_bool_certificate_payload, check_linarith_certificate,
    evaluate_bitvec_expr, ArrayCertificate, ArrayClaim, ArrayElement, ArrayExpr, ArrayQuery,
    ArrayResult, BaseArray, BitVecBinaryOp, BitVecComparisonOp, BitVecEvalResult, BitVecExpr,
    BitVecUnaryOp, BitVecValue, FarkasMultiplier, LinarithCertificate, LinearInequality,
    LinearTerm, ARRAY_CERT_FORMAT, BOOL_CERT_FORMAT, LINARITH_CERT_FORMAT,
};

pub(crate) const BITVEC_CERT_FORMAT: &str = "mpk.bitvec-ground.v0";

const BITVEC_CERT_MAGIC: &[u8; 8] = b"MPKBVGC0";
const BITVEC_FORMAT_TAG: u8 = 0;
const LINARITH_CERT_MAGIC: &[u8; 8] = b"MPKLINR0";
const ARRAY_CERT_MAGIC: &[u8; 8] = b"MPKARRY0";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedTheoryCertificate {
    pub format: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TheoryProofError {
    kind: TheoryProofErrorKind,
    detail: String,
}

impl TheoryProofError {
    pub(crate) fn kind(&self) -> TheoryProofErrorKind {
        self.kind
    }

    fn new(kind: TheoryProofErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for TheoryProofError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.kind.as_str(), self.detail)
    }
}

impl std::error::Error for TheoryProofError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub(crate) enum TheoryProofErrorKind {
    UnsupportedFormat,
    InvalidPayload,
    BoolCertificate,
    BitVecCertificate,
    LinarithCertificate,
    ArrayCertificate,
}

impl TheoryProofErrorKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedFormat => "THEORY_UNSUPPORTED_FORMAT",
            Self::InvalidPayload => "THEORY_INVALID_PAYLOAD",
            Self::BoolCertificate => "THEORY_BOOL_CERTIFICATE",
            Self::BitVecCertificate => "THEORY_BITVEC_CERTIFICATE",
            Self::LinarithCertificate => "THEORY_LINARITH_CERTIFICATE",
            Self::ArrayCertificate => "THEORY_ARRAY_CERTIFICATE",
        }
    }
}

pub(crate) fn check_theory_certificate(
    certificate: &TheoryCertificate,
) -> Result<CheckedTheoryCertificate, TheoryProofError> {
    match certificate.format.as_str() {
        BOOL_CERT_FORMAT => {
            check_bool_certificate_payload(&certificate.payload).map_err(|error| {
                TheoryProofError::new(
                    TheoryProofErrorKind::BoolCertificate,
                    format!("bool checker rejected payload: {error}"),
                )
            })?;
            Ok(CheckedTheoryCertificate {
                format: BOOL_CERT_FORMAT,
            })
        }
        BITVEC_CERT_FORMAT => {
            check_bitvec_certificate_payload(&certificate.payload)?;
            Ok(CheckedTheoryCertificate {
                format: BITVEC_CERT_FORMAT,
            })
        }
        LINARITH_CERT_FORMAT => {
            let certificate = decode_linarith_certificate(&certificate.payload)?;
            check_linarith_certificate(&certificate).map_err(|error| {
                TheoryProofError::new(
                    TheoryProofErrorKind::LinarithCertificate,
                    format!("linear arithmetic checker rejected payload: {error}"),
                )
            })?;
            Ok(CheckedTheoryCertificate {
                format: LINARITH_CERT_FORMAT,
            })
        }
        ARRAY_CERT_FORMAT => {
            let certificate = decode_array_certificate(&certificate.payload)?;
            check_array_certificate(&certificate).map_err(|error| {
                TheoryProofError::new(
                    TheoryProofErrorKind::ArrayCertificate,
                    format!("array checker rejected payload: {error}"),
                )
            })?;
            Ok(CheckedTheoryCertificate {
                format: ARRAY_CERT_FORMAT,
            })
        }
        format => Err(TheoryProofError::new(
            TheoryProofErrorKind::UnsupportedFormat,
            format!("unsupported theory certificate format {format:?}"),
        )),
    }
}

fn check_bitvec_certificate_payload(payload: &[u8]) -> Result<(), TheoryProofError> {
    let mut decoder = TheoryPayloadDecoder::new(payload);
    decoder.read_magic(BITVEC_CERT_MAGIC, "bitvec_magic")?;
    let format_tag = decoder.read_u8("format_tag")?;
    if format_tag != BITVEC_FORMAT_TAG {
        return Err(decoder.invalid(format!(
            "invalid bitvec format_tag={format_tag}; expected={BITVEC_FORMAT_TAG}"
        )));
    }

    let expression = decoder.read_bitvec_expr()?;
    let trace_len = decoder.read_leb_u32("trace_len")?;
    let expected = expected_bitvec_trace(&expression)?;
    if trace_len as usize != expected.len() {
        return Err(decoder.bitvec(format!(
            "trace_len={trace_len}; expected={}",
            expected.len()
        )));
    }
    for expected_step in &expected {
        let step = decoder.read_bitvec_trace_step()?;
        if step != *expected_step {
            return Err(decoder.bitvec(format!(
                "trace step mismatch at step_id={}; expected={expected_step:?}; actual={step:?}",
                expected_step.step_id
            )));
        }
    }

    let claimed = decoder.read_bitvec_result()?;
    let actual = evaluate_bitvec_expr(&expression).map_err(|error| {
        TheoryProofError::new(
            TheoryProofErrorKind::BitVecCertificate,
            format!("bitvec evaluator rejected root expression: {error}"),
        )
    })?;
    if claimed != actual {
        return Err(decoder.bitvec(format!(
            "claimed result mismatch; claimed={claimed:?}; actual={actual:?}"
        )));
    }
    if expected
        .last()
        .map(|step| step.output)
        .filter(|output| *output == actual)
        .is_none()
    {
        return Err(decoder.bitvec("trace final output does not match recomputed root result"));
    }
    decoder.finish()?;
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BitVecTraceStep {
    step_id: u32,
    path: Vec<PathSegment>,
    op_domain: u8,
    op_tag: u8,
    inputs: Vec<BitVecEvalResult>,
    output: BitVecEvalResult,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PathSegment {
    UnaryValue,
    BinaryLhs,
    BinaryRhs,
    CompareLhs,
    CompareRhs,
}

fn expected_bitvec_trace(
    expression: &BitVecExpr,
) -> Result<Vec<BitVecTraceStep>, TheoryProofError> {
    let mut steps = Vec::new();
    push_expected_bitvec_trace(expression, Vec::new(), &mut steps)?;
    Ok(steps)
}

fn push_expected_bitvec_trace(
    expression: &BitVecExpr,
    path: Vec<PathSegment>,
    steps: &mut Vec<BitVecTraceStep>,
) -> Result<BitVecEvalResult, TheoryProofError> {
    let (op_domain, op_tag, inputs) = match expression {
        BitVecExpr::Literal(_) => (0x00, 0x00, Vec::new()),
        BitVecExpr::Unary { op, value } => {
            let mut child_path = path.clone();
            child_path.push(PathSegment::UnaryValue);
            let input = push_expected_bitvec_trace(value, child_path, steps)?;
            (0x01, encode_unary_op(*op), vec![input])
        }
        BitVecExpr::Binary { op, lhs, rhs } => {
            let mut lhs_path = path.clone();
            lhs_path.push(PathSegment::BinaryLhs);
            let lhs = push_expected_bitvec_trace(lhs, lhs_path, steps)?;
            let mut rhs_path = path.clone();
            rhs_path.push(PathSegment::BinaryRhs);
            let rhs = push_expected_bitvec_trace(rhs, rhs_path, steps)?;
            (0x02, encode_binary_op(*op), vec![lhs, rhs])
        }
        BitVecExpr::Compare { op, lhs, rhs } => {
            let mut lhs_path = path.clone();
            lhs_path.push(PathSegment::CompareLhs);
            let lhs = push_expected_bitvec_trace(lhs, lhs_path, steps)?;
            let mut rhs_path = path.clone();
            rhs_path.push(PathSegment::CompareRhs);
            let rhs = push_expected_bitvec_trace(rhs, rhs_path, steps)?;
            (0x03, encode_comparison_op(*op), vec![lhs, rhs])
        }
    };

    let output = evaluate_bitvec_expr(expression).map_err(|error| {
        TheoryProofError::new(
            TheoryProofErrorKind::BitVecCertificate,
            format!("bitvec evaluator rejected traced expression: {error}"),
        )
    })?;
    let step_id = u32::try_from(steps.len()).map_err(|_| {
        TheoryProofError::new(
            TheoryProofErrorKind::InvalidPayload,
            "bitvec trace step count exceeds u32",
        )
    })?;
    steps.push(BitVecTraceStep {
        step_id,
        path,
        op_domain,
        op_tag,
        inputs,
        output,
    });
    Ok(output)
}

fn decode_linarith_certificate(payload: &[u8]) -> Result<LinarithCertificate, TheoryProofError> {
    let mut decoder = TheoryPayloadDecoder::new(payload);
    decoder.read_magic(LINARITH_CERT_MAGIC, "linarith_magic")?;
    let premise_count = decoder.read_u8("premise_count")?;
    let mut premises = Vec::with_capacity(usize::from(premise_count));
    for _ in 0..premise_count {
        premises.push(decoder.read_linear_inequality()?);
    }
    let goal = decoder.read_linear_inequality()?;
    let combination_count = decoder.read_u8("combination_count")?;
    let mut combination = Vec::with_capacity(usize::from(combination_count));
    for _ in 0..combination_count {
        combination.push(FarkasMultiplier::new(
            usize::try_from(decoder.read_u32("premise_index")?).expect("u32 fits usize"),
            decoder.read_u64("multiplier")?,
        ));
    }
    decoder.finish()?;

    Ok(LinarithCertificate {
        premises,
        goal,
        combination,
    })
}

fn decode_array_certificate(payload: &[u8]) -> Result<ArrayCertificate, TheoryProofError> {
    let mut decoder = TheoryPayloadDecoder::new(payload);
    decoder.read_magic(ARRAY_CERT_MAGIC, "array_magic")?;
    let base_count = decoder.read_u8("base_count")?;
    let mut base_arrays = Vec::with_capacity(usize::from(base_count));
    for _ in 0..base_count {
        base_arrays.push(BaseArray::new(
            decoder.read_u32("array_id")?,
            decoder.read_u32("length")?,
        ));
    }
    let claim_count = decoder.read_u8("claim_count")?;
    let mut claims = Vec::with_capacity(usize::from(claim_count));
    for _ in 0..claim_count {
        claims.push(decoder.read_array_claim()?);
    }
    decoder.finish()?;

    Ok(ArrayCertificate {
        base_arrays,
        claims,
    })
}

struct TheoryPayloadDecoder<'payload> {
    payload: &'payload [u8],
    offset: usize,
    decoded_bitvec_nodes: usize,
    decoded_array_nodes: usize,
}

impl<'payload> TheoryPayloadDecoder<'payload> {
    fn new(payload: &'payload [u8]) -> Self {
        Self {
            payload,
            offset: 0,
            decoded_bitvec_nodes: 0,
            decoded_array_nodes: 0,
        }
    }

    fn read_magic(&mut self, expected: &[u8], field: &'static str) -> Result<(), TheoryProofError> {
        let bytes = self.read_exact(expected.len(), field)?;
        if bytes == expected {
            Ok(())
        } else {
            Err(self.invalid(format!("invalid {field}")))
        }
    }

    fn read_bitvec_expr(&mut self) -> Result<BitVecExpr, TheoryProofError> {
        self.decoded_bitvec_nodes += 1;
        if self.decoded_bitvec_nodes > mpk_theory::MAX_BITVEC_EXPR_NODES {
            return Err(self.bitvec(format!("nodes>{}", mpk_theory::MAX_BITVEC_EXPR_NODES)));
        }

        let tag = self.read_u8("expr_tag")?;
        match tag {
            0x00 => {
                let width = self.read_width()?;
                let bits = self.read_width_bits(width)?;
                BitVecExpr::literal(width, bits).map_err(|error| {
                    self.bitvec(format!("invalid bitvec literal width/bits: {error}"))
                })
            }
            0x01 => {
                let op = self.read_unary_op()?;
                let value = self.read_bitvec_expr()?;
                Ok(BitVecExpr::unary(op, value))
            }
            0x02 => {
                let op = self.read_binary_op()?;
                let lhs = self.read_bitvec_expr()?;
                let rhs = self.read_bitvec_expr()?;
                Ok(BitVecExpr::binary(op, lhs, rhs))
            }
            0x03 => {
                let op = self.read_comparison_op()?;
                let lhs = self.read_bitvec_expr()?;
                let rhs = self.read_bitvec_expr()?;
                Ok(BitVecExpr::compare(op, lhs, rhs))
            }
            _ => Err(self.invalid(format!("unknown bitvec expr tag=0x{tag:02x}"))),
        }
    }

    fn read_bitvec_trace_step(&mut self) -> Result<BitVecTraceStep, TheoryProofError> {
        let step_id = self.read_leb_u32("step_id")?;
        let path = self.read_path()?;
        let op_domain = self.read_u8("op_tag_domain")?;
        let op_tag = self.read_u8("op_tag")?;
        let input_len = self.read_u8("input_len")?;
        let mut inputs = Vec::with_capacity(usize::from(input_len));
        for _ in 0..input_len {
            inputs.push(self.read_bitvec_result()?);
        }
        let output = self.read_bitvec_result()?;

        Ok(BitVecTraceStep {
            step_id,
            path,
            op_domain,
            op_tag,
            inputs,
            output,
        })
    }

    fn read_bitvec_result(&mut self) -> Result<BitVecEvalResult, TheoryProofError> {
        let tag = self.read_u8("result_tag")?;
        match tag {
            0x00 => {
                let width = self.read_width()?;
                let bits = self.read_width_bits(width)?;
                let value = BitVecValue::new(width, bits)
                    .map_err(|error| self.bitvec(format!("invalid bitvec result: {error}")))?;
                Ok(BitVecEvalResult::BitVec(value))
            }
            0x01 => Ok(BitVecEvalResult::Bool(self.read_bool("bool_result")?)),
            _ => Err(self.invalid(format!("unknown result tag=0x{tag:02x}"))),
        }
    }

    fn read_path(&mut self) -> Result<Vec<PathSegment>, TheoryProofError> {
        let count = self.read_leb_u32("path_segment_count")?;
        if count as usize > mpk_theory::MAX_BITVEC_EXPR_NODES {
            return Err(self.bitvec(format!(
                "path segments={count}; max={}",
                mpk_theory::MAX_BITVEC_EXPR_NODES
            )));
        }
        let mut path = Vec::with_capacity(usize::try_from(count).expect("u32 fits usize"));
        for _ in 0..count {
            let tag = self.read_u8("path_segment")?;
            path.push(match tag {
                0x01 => PathSegment::UnaryValue,
                0x02 => PathSegment::BinaryLhs,
                0x03 => PathSegment::BinaryRhs,
                0x04 => PathSegment::CompareLhs,
                0x05 => PathSegment::CompareRhs,
                _ => return Err(self.invalid(format!("unknown path segment tag=0x{tag:02x}"))),
            });
        }
        Ok(path)
    }

    fn read_width(&mut self) -> Result<u32, TheoryProofError> {
        let tag = self.read_u8("width_tag")?;
        match tag {
            0x08 => Ok(8),
            0x10 => Ok(16),
            0x20 => Ok(32),
            0x40 => Ok(64),
            _ => Err(self.invalid(format!("unknown width tag=0x{tag:02x}"))),
        }
    }

    fn read_width_bits(&mut self, width: u32) -> Result<u64, TheoryProofError> {
        let len = usize::try_from(width / 8).expect("supported width byte length fits usize");
        let bytes = self.read_exact(len, "bits")?;
        let mut padded = [0u8; 8];
        padded[8 - len..].copy_from_slice(bytes);
        Ok(u64::from_be_bytes(padded))
    }

    fn read_unary_op(&mut self) -> Result<BitVecUnaryOp, TheoryProofError> {
        let tag = self.read_u8("unary_op")?;
        match tag {
            0x00 => Ok(BitVecUnaryOp::Not),
            0x01 => Ok(BitVecUnaryOp::Neg),
            _ => Err(self.invalid(format!("unknown unary op tag=0x{tag:02x}"))),
        }
    }

    fn read_binary_op(&mut self) -> Result<BitVecBinaryOp, TheoryProofError> {
        let tag = self.read_u8("binary_op")?;
        match tag {
            0x00 => Ok(BitVecBinaryOp::And),
            0x01 => Ok(BitVecBinaryOp::Or),
            0x02 => Ok(BitVecBinaryOp::Xor),
            0x03 => Ok(BitVecBinaryOp::Add),
            0x04 => Ok(BitVecBinaryOp::Sub),
            0x05 => Ok(BitVecBinaryOp::Mul),
            0x06 => Ok(BitVecBinaryOp::Shl),
            0x07 => Ok(BitVecBinaryOp::Lshr),
            0x08 => Ok(BitVecBinaryOp::Ashr),
            _ => Err(self.invalid(format!("unknown binary op tag=0x{tag:02x}"))),
        }
    }

    fn read_comparison_op(&mut self) -> Result<BitVecComparisonOp, TheoryProofError> {
        let tag = self.read_u8("comparison_op")?;
        match tag {
            0x00 => Ok(BitVecComparisonOp::Ult),
            0x01 => Ok(BitVecComparisonOp::Ule),
            0x02 => Ok(BitVecComparisonOp::Ugt),
            0x03 => Ok(BitVecComparisonOp::Uge),
            0x04 => Ok(BitVecComparisonOp::Slt),
            0x05 => Ok(BitVecComparisonOp::Sle),
            0x06 => Ok(BitVecComparisonOp::Sgt),
            0x07 => Ok(BitVecComparisonOp::Sge),
            _ => Err(self.invalid(format!("unknown comparison op tag=0x{tag:02x}"))),
        }
    }

    fn read_linear_inequality(&mut self) -> Result<LinearInequality, TheoryProofError> {
        let term_count = self.read_u8("term_count")?;
        let mut terms = Vec::with_capacity(usize::from(term_count));
        for _ in 0..term_count {
            terms.push(LinearTerm::new(
                self.read_u32("variable")?,
                self.read_i128("coefficient")?,
            ));
        }
        let constant = self.read_i128("constant")?;
        Ok(LinearInequality::new(terms, constant))
    }

    fn read_array_claim(&mut self) -> Result<ArrayClaim, TheoryProofError> {
        let query = self.read_array_query()?;
        let expected = self.read_array_result()?;
        Ok(ArrayClaim::new(query, expected))
    }

    fn read_array_query(&mut self) -> Result<ArrayQuery, TheoryProofError> {
        let tag = self.read_u8("array_query_tag")?;
        match tag {
            0x00 => Ok(ArrayQuery::Read {
                array: self.read_array_expr()?,
                index: self.read_u32("read_index")?,
            }),
            0x01 => Ok(ArrayQuery::Length {
                array: self.read_array_expr()?,
            }),
            _ => Err(self.invalid(format!("unknown array query tag=0x{tag:02x}"))),
        }
    }

    fn read_array_expr(&mut self) -> Result<ArrayExpr, TheoryProofError> {
        self.decoded_array_nodes += 1;
        if self.decoded_array_nodes > mpk_theory::MAX_ARRAY_EXPR_NODES {
            return Err(TheoryProofError::new(
                TheoryProofErrorKind::ArrayCertificate,
                format!(
                    "array expression nodes>{}",
                    mpk_theory::MAX_ARRAY_EXPR_NODES
                ),
            ));
        }
        let tag = self.read_u8("array_expr_tag")?;
        match tag {
            0x00 => Ok(ArrayExpr::base(self.read_u32("base_array_id")?)),
            0x01 => {
                let array = self.read_array_expr()?;
                let index = self.read_u32("write_index")?;
                let value = self.read_array_element()?;
                Ok(ArrayExpr::write(array, index, value))
            }
            _ => Err(self.invalid(format!("unknown array expr tag=0x{tag:02x}"))),
        }
    }

    fn read_array_element(&mut self) -> Result<ArrayElement, TheoryProofError> {
        let tag = self.read_u8("array_element_tag")?;
        match tag {
            0x00 => Ok(ArrayElement::Symbol(self.read_u32("symbol")?)),
            0x01 => Ok(ArrayElement::BaseRead {
                array_id: self.read_u32("array_id")?,
                index: self.read_u32("index")?,
            }),
            _ => Err(self.invalid(format!("unknown array element tag=0x{tag:02x}"))),
        }
    }

    fn read_array_result(&mut self) -> Result<ArrayResult, TheoryProofError> {
        let tag = self.read_u8("array_result_tag")?;
        match tag {
            0x00 => Ok(ArrayResult::Element(self.read_array_element()?)),
            0x01 => Ok(ArrayResult::Length(self.read_u32("length")?)),
            _ => Err(self.invalid(format!("unknown array result tag=0x{tag:02x}"))),
        }
    }

    fn read_bool(&mut self, field: &'static str) -> Result<bool, TheoryProofError> {
        match self.read_u8(field)? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(self.invalid(format!("invalid bool {field}={value}"))),
        }
    }

    fn read_u8(&mut self, field: &'static str) -> Result<u8, TheoryProofError> {
        Ok(self.read_exact(1, field)?[0])
    }

    fn read_u32(&mut self, field: &'static str) -> Result<u32, TheoryProofError> {
        let bytes = self.read_exact(4, field)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64(&mut self, field: &'static str) -> Result<u64, TheoryProofError> {
        let bytes = self.read_exact(8, field)?;
        Ok(u64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_i128(&mut self, field: &'static str) -> Result<i128, TheoryProofError> {
        let bytes = self.read_exact(16, field)?;
        Ok(i128::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]))
    }

    fn read_leb_u32(&mut self, field: &'static str) -> Result<u32, TheoryProofError> {
        let start = self.offset;
        let mut value = 0u32;
        let mut shift = 0u32;
        for byte_index in 0..5 {
            let byte = self.read_u8(field)?;
            let payload = u32::from(byte & 0x7f);
            if shift == 28 && payload > 0x0f {
                return Err(self.invalid(format!("u32 LEB overflow while reading {field}")));
            }
            value |= payload << shift;
            if byte & 0x80 == 0 {
                if byte_index > 0 {
                    let minimal_limit = 1u32 << (7 * byte_index);
                    if value < minimal_limit {
                        return Err(
                            self.invalid(format!("non-minimal u32 LEB while reading {field}"))
                        );
                    }
                }
                return Ok(value);
            }
            shift += 7;
        }
        Err(self.invalid(format!(
            "unterminated u32 LEB while reading {field}; offset={start}"
        )))
    }

    fn read_exact(
        &mut self,
        len: usize,
        field: &'static str,
    ) -> Result<&'payload [u8], TheoryProofError> {
        let start = self.offset;
        let end = start.checked_add(len).ok_or_else(|| {
            self.invalid(format!(
                "offset overflow while reading {field}; offset={start}"
            ))
        })?;
        let bytes = self.payload.get(start..end).ok_or_else(|| {
            self.invalid(format!(
                "unexpected EOF while reading {field}; offset={start}; len={len}"
            ))
        })?;
        self.offset = end;
        Ok(bytes)
    }

    fn finish(&self) -> Result<(), TheoryProofError> {
        if self.offset == self.payload.len() {
            Ok(())
        } else {
            Err(self.invalid(format!(
                "trailing bytes at offset={}; len={}",
                self.offset,
                self.payload.len()
            )))
        }
    }

    fn invalid(&self, detail: impl Into<String>) -> TheoryProofError {
        TheoryProofError::new(TheoryProofErrorKind::InvalidPayload, detail)
    }

    fn bitvec(&self, detail: impl Into<String>) -> TheoryProofError {
        TheoryProofError::new(TheoryProofErrorKind::BitVecCertificate, detail)
    }
}

fn encode_unary_op(op: BitVecUnaryOp) -> u8 {
    match op {
        BitVecUnaryOp::Not => 0x00,
        BitVecUnaryOp::Neg => 0x01,
    }
}

fn encode_binary_op(op: BitVecBinaryOp) -> u8 {
    match op {
        BitVecBinaryOp::And => 0x00,
        BitVecBinaryOp::Or => 0x01,
        BitVecBinaryOp::Xor => 0x02,
        BitVecBinaryOp::Add => 0x03,
        BitVecBinaryOp::Sub => 0x04,
        BitVecBinaryOp::Mul => 0x05,
        BitVecBinaryOp::Shl => 0x06,
        BitVecBinaryOp::Lshr => 0x07,
        BitVecBinaryOp::Ashr => 0x08,
    }
}

fn encode_comparison_op(op: BitVecComparisonOp) -> u8 {
    match op {
        BitVecComparisonOp::Ult => 0x00,
        BitVecComparisonOp::Ule => 0x01,
        BitVecComparisonOp::Ugt => 0x02,
        BitVecComparisonOp::Uge => 0x03,
        BitVecComparisonOp::Slt => 0x04,
        BitVecComparisonOp::Sle => 0x05,
        BitVecComparisonOp::Sgt => 0x06,
        BitVecComparisonOp::Sge => 0x07,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn certificate(format: &str, payload: Vec<u8>) -> TheoryCertificate {
        TheoryCertificate {
            format: format.to_owned(),
            payload,
        }
    }

    fn bool_tautology_payload() -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"MPKBOOL0");
        payload.push(0);
        payload.push(0x01);
        payload.extend_from_slice(&1u16.to_be_bytes());
        payload.push(0);
        payload.push(1);
        payload
    }

    fn push_bv_result(payload: &mut Vec<u8>, width: u8, bits: u64) {
        payload.push(0x00);
        payload.push(width);
        let len = usize::from(width / 8);
        payload.extend_from_slice(&bits.to_be_bytes()[8 - len..]);
    }

    fn bitvec_add_payload() -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"MPKBVGC0");
        payload.push(0);
        payload.extend_from_slice(&[0x02, 0x03, 0x00, 0x08, 0x01, 0x00, 0x08, 0x01]);
        payload.push(3);

        payload.push(0);
        payload.push(1);
        payload.push(0x02);
        payload.push(0x00);
        payload.push(0x00);
        payload.push(0);
        push_bv_result(&mut payload, 0x08, 1);

        payload.push(1);
        payload.push(1);
        payload.push(0x03);
        payload.push(0x00);
        payload.push(0x00);
        payload.push(0);
        push_bv_result(&mut payload, 0x08, 1);

        payload.push(2);
        payload.push(0);
        payload.push(0x02);
        payload.push(0x03);
        payload.push(2);
        push_bv_result(&mut payload, 0x08, 1);
        push_bv_result(&mut payload, 0x08, 1);
        push_bv_result(&mut payload, 0x08, 2);

        push_bv_result(&mut payload, 0x08, 2);
        payload
    }

    fn linarith_trivial_payload() -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"MPKLINR0");
        payload.push(0);
        payload.push(0);
        payload.extend_from_slice(&0i128.to_be_bytes());
        payload.push(0);
        payload
    }

    fn array_read_write_payload() -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(b"MPKARRY0");
        payload.push(1);
        payload.extend_from_slice(&0u32.to_be_bytes());
        payload.extend_from_slice(&2u32.to_be_bytes());
        payload.push(1);
        payload.push(0x00);
        payload.push(0x01);
        payload.push(0x00);
        payload.extend_from_slice(&0u32.to_be_bytes());
        payload.extend_from_slice(&1u32.to_be_bytes());
        payload.push(0x00);
        payload.extend_from_slice(&7u32.to_be_bytes());
        payload.extend_from_slice(&1u32.to_be_bytes());
        payload.push(0x00);
        payload.push(0x00);
        payload.extend_from_slice(&7u32.to_be_bytes());
        payload
    }

    #[test]
    fn dispatches_supported_theory_certificate_formats() {
        for (format, payload) in [
            (BOOL_CERT_FORMAT, bool_tautology_payload()),
            (BITVEC_CERT_FORMAT, bitvec_add_payload()),
            (LINARITH_CERT_FORMAT, linarith_trivial_payload()),
            (ARRAY_CERT_FORMAT, array_read_write_payload()),
        ] {
            let checked =
                check_theory_certificate(&certificate(format, payload)).expect("format checks");

            assert_eq!(checked.format, format);
        }
    }

    #[test]
    fn rejects_unknown_theory_certificate_format() {
        let error = check_theory_certificate(&certificate("mpk.unknown.v0", Vec::new()))
            .expect_err("unknown format rejects");

        assert_eq!(error.kind(), TheoryProofErrorKind::UnsupportedFormat);
    }
}
