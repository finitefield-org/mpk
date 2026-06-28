//! Ground fixed-width bitvector evaluator.
//!
//! The evaluator is intentionally small and independent: it normalizes ground
//! bitvector expressions without trusting an external solver result.

use std::fmt;

pub const MAX_BITVEC_EXPR_NODES: usize = 256;
pub const SUPPORTED_BITVEC_WIDTHS: [u32; 4] = [8, 16, 32, 64];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitVecValue {
    pub width: u32,
    pub bits: u64,
}

impl BitVecValue {
    pub fn new(width: u32, bits: u64) -> Result<Self, BitVecEvalError> {
        Ok(Self {
            width,
            bits: normalize_bits(width, bits)?,
        })
    }

    pub fn signed_value(self) -> Result<i128, BitVecEvalError> {
        validate_width(self.width)?;
        let sign_bit = 1u64 << (self.width - 1);
        let unsigned = i128::from(self.bits & width_mask(self.width)?);
        if self.bits & sign_bit == 0 {
            Ok(unsigned)
        } else {
            Ok(unsigned - (1i128 << self.width))
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BitVecExpr {
    Literal(BitVecValue),
    Unary {
        op: BitVecUnaryOp,
        value: Box<BitVecExpr>,
    },
    Binary {
        op: BitVecBinaryOp,
        lhs: Box<BitVecExpr>,
        rhs: Box<BitVecExpr>,
    },
    Compare {
        op: BitVecComparisonOp,
        lhs: Box<BitVecExpr>,
        rhs: Box<BitVecExpr>,
    },
}

impl BitVecExpr {
    pub fn literal(width: u32, bits: u64) -> Result<Self, BitVecEvalError> {
        Ok(Self::Literal(BitVecValue::new(width, bits)?))
    }

    pub fn unary(op: BitVecUnaryOp, value: BitVecExpr) -> Self {
        Self::Unary {
            op,
            value: Box::new(value),
        }
    }

    pub fn binary(op: BitVecBinaryOp, lhs: BitVecExpr, rhs: BitVecExpr) -> Self {
        Self::Binary {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    pub fn compare(op: BitVecComparisonOp, lhs: BitVecExpr, rhs: BitVecExpr) -> Self {
        Self::Compare {
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    fn node_count(&self) -> usize {
        match self {
            Self::Literal(_) => 1,
            Self::Unary { value, .. } => 1 + value.node_count(),
            Self::Binary { lhs, rhs, .. } | Self::Compare { lhs, rhs, .. } => {
                1 + lhs.node_count() + rhs.node_count()
            }
        }
    }

    fn eval(&self) -> Result<BitVecEvalResult, BitVecEvalError> {
        match self {
            Self::Literal(value) => Ok(BitVecEvalResult::BitVec(BitVecValue::new(
                value.width,
                value.bits,
            )?)),
            Self::Unary { op, value } => {
                let value = value.eval()?.into_bitvec(op.as_str(), "value")?;
                Ok(BitVecEvalResult::BitVec(eval_unary(*op, value)?))
            }
            Self::Binary { op, lhs, rhs } => {
                let lhs = lhs.eval()?.into_bitvec(op.as_str(), "lhs")?;
                let rhs = rhs.eval()?.into_bitvec(op.as_str(), "rhs")?;
                Ok(BitVecEvalResult::BitVec(eval_binary(*op, lhs, rhs)?))
            }
            Self::Compare { op, lhs, rhs } => {
                let lhs = lhs.eval()?.into_bitvec(op.as_str(), "lhs")?;
                let rhs = rhs.eval()?.into_bitvec(op.as_str(), "rhs")?;
                Ok(BitVecEvalResult::Bool(eval_comparison(*op, lhs, rhs)?))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BitVecUnaryOp {
    Not,
    Neg,
}

impl BitVecUnaryOp {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Not => "not",
            Self::Neg => "neg",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BitVecBinaryOp {
    And,
    Or,
    Xor,
    Add,
    Sub,
    Mul,
    Shl,
    Lshr,
    Ashr,
}

impl BitVecBinaryOp {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Shl => "shl",
            Self::Lshr => "lshr",
            Self::Ashr => "ashr",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BitVecComparisonOp {
    Ult,
    Ule,
    Ugt,
    Uge,
    Slt,
    Sle,
    Sgt,
    Sge,
}

impl BitVecComparisonOp {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ult => "ult",
            Self::Ule => "ule",
            Self::Ugt => "ugt",
            Self::Uge => "uge",
            Self::Slt => "slt",
            Self::Sle => "sle",
            Self::Sgt => "sgt",
            Self::Sge => "sge",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitVecEvalResult {
    BitVec(BitVecValue),
    Bool(bool),
}

impl BitVecEvalResult {
    fn into_bitvec(
        self,
        op: &'static str,
        operand: &'static str,
    ) -> Result<BitVecValue, BitVecEvalError> {
        match self {
            Self::BitVec(value) => Ok(value),
            Self::Bool(_) => Err(BitVecEvalError::new(
                BitVecEvalErrorKind::ExpectedBitVector,
                format!("op={op}; operand={operand}; actual=bool"),
            )),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitVecEvalError {
    kind: BitVecEvalErrorKind,
    detail: String,
}

impl BitVecEvalError {
    pub fn kind(&self) -> BitVecEvalErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: BitVecEvalErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for BitVecEvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.kind.as_str(), self.detail)
    }
}

impl std::error::Error for BitVecEvalError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BitVecEvalErrorKind {
    UnsupportedWidth,
    ExpressionTooLarge,
    ExpectedBitVector,
    WidthMismatch,
}

impl BitVecEvalErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedWidth => "BITVEC_UNSUPPORTED_WIDTH",
            Self::ExpressionTooLarge => "BITVEC_EXPRESSION_TOO_LARGE",
            Self::ExpectedBitVector => "BITVEC_EXPECTED_BITVECTOR",
            Self::WidthMismatch => "BITVEC_WIDTH_MISMATCH",
        }
    }
}

pub fn evaluate_bitvec_expr(input: &BitVecExpr) -> Result<BitVecEvalResult, BitVecEvalError> {
    let node_count = input.node_count();
    if node_count > MAX_BITVEC_EXPR_NODES {
        return Err(BitVecEvalError::new(
            BitVecEvalErrorKind::ExpressionTooLarge,
            format!("nodes={node_count}; max={MAX_BITVEC_EXPR_NODES}"),
        ));
    }

    input.eval()
}

fn eval_unary(op: BitVecUnaryOp, value: BitVecValue) -> Result<BitVecValue, BitVecEvalError> {
    validate_width(value.width)?;
    let mask = width_mask(value.width)?;
    let bits = match op {
        BitVecUnaryOp::Not => !value.bits,
        BitVecUnaryOp::Neg => value.bits.wrapping_neg(),
    };
    BitVecValue::new(value.width, bits & mask)
}

fn eval_binary(
    op: BitVecBinaryOp,
    lhs: BitVecValue,
    rhs: BitVecValue,
) -> Result<BitVecValue, BitVecEvalError> {
    let bits = match op {
        BitVecBinaryOp::And => eval_same_width_binary(op, lhs, rhs, |lhs, rhs| lhs & rhs)?,
        BitVecBinaryOp::Or => eval_same_width_binary(op, lhs, rhs, |lhs, rhs| lhs | rhs)?,
        BitVecBinaryOp::Xor => eval_same_width_binary(op, lhs, rhs, |lhs, rhs| lhs ^ rhs)?,
        BitVecBinaryOp::Add => {
            eval_same_width_binary(op, lhs, rhs, |lhs, rhs| lhs.wrapping_add(rhs))?
        }
        BitVecBinaryOp::Sub => {
            eval_same_width_binary(op, lhs, rhs, |lhs, rhs| lhs.wrapping_sub(rhs))?
        }
        BitVecBinaryOp::Mul => {
            eval_same_width_binary(op, lhs, rhs, |lhs, rhs| lhs.wrapping_mul(rhs))?
        }
        BitVecBinaryOp::Shl => {
            require_shift_operands(op.as_str(), lhs, rhs)?;
            eval_shl(lhs, rhs)
        }
        BitVecBinaryOp::Lshr => {
            require_shift_operands(op.as_str(), lhs, rhs)?;
            eval_lshr(lhs, rhs)
        }
        BitVecBinaryOp::Ashr => {
            require_shift_operands(op.as_str(), lhs, rhs)?;
            return eval_ashr(lhs, rhs);
        }
    };
    BitVecValue::new(lhs.width, bits)
}

fn eval_comparison(
    op: BitVecComparisonOp,
    lhs: BitVecValue,
    rhs: BitVecValue,
) -> Result<bool, BitVecEvalError> {
    require_same_width(op.as_str(), lhs, rhs)?;
    match op {
        BitVecComparisonOp::Ult => Ok(lhs.bits < rhs.bits),
        BitVecComparisonOp::Ule => Ok(lhs.bits <= rhs.bits),
        BitVecComparisonOp::Ugt => Ok(lhs.bits > rhs.bits),
        BitVecComparisonOp::Uge => Ok(lhs.bits >= rhs.bits),
        BitVecComparisonOp::Slt => Ok(lhs.signed_value()? < rhs.signed_value()?),
        BitVecComparisonOp::Sle => Ok(lhs.signed_value()? <= rhs.signed_value()?),
        BitVecComparisonOp::Sgt => Ok(lhs.signed_value()? > rhs.signed_value()?),
        BitVecComparisonOp::Sge => Ok(lhs.signed_value()? >= rhs.signed_value()?),
    }
}

fn eval_same_width_binary(
    op: BitVecBinaryOp,
    lhs: BitVecValue,
    rhs: BitVecValue,
    apply: impl FnOnce(u64, u64) -> u64,
) -> Result<u64, BitVecEvalError> {
    require_same_width(op.as_str(), lhs, rhs)?;
    Ok(apply(lhs.bits, rhs.bits))
}

fn eval_shl(lhs: BitVecValue, rhs: BitVecValue) -> u64 {
    if rhs.bits >= u64::from(lhs.width) {
        0
    } else {
        lhs.bits << rhs.bits
    }
}

fn eval_lshr(lhs: BitVecValue, rhs: BitVecValue) -> u64 {
    if rhs.bits >= u64::from(lhs.width) {
        0
    } else {
        lhs.bits >> rhs.bits
    }
}

fn eval_ashr(lhs: BitVecValue, rhs: BitVecValue) -> Result<BitVecValue, BitVecEvalError> {
    if rhs.bits >= u64::from(lhs.width) {
        let sign_bit = 1u64 << (lhs.width - 1);
        if lhs.bits & sign_bit == 0 {
            return BitVecValue::new(lhs.width, 0);
        }
        return BitVecValue::new(lhs.width, width_mask(lhs.width)?);
    }

    let shifted = lhs.signed_value()? >> rhs.bits;
    BitVecValue::new(lhs.width, shifted as u64)
}

fn require_same_width(
    op: &'static str,
    lhs: BitVecValue,
    rhs: BitVecValue,
) -> Result<(), BitVecEvalError> {
    validate_width(lhs.width)?;
    validate_width(rhs.width)?;
    if lhs.width != rhs.width {
        return Err(BitVecEvalError::new(
            BitVecEvalErrorKind::WidthMismatch,
            format!("op={op}; lhs_width={}; rhs_width={}", lhs.width, rhs.width),
        ));
    }
    Ok(())
}

fn require_shift_operands(
    op: &'static str,
    lhs: BitVecValue,
    rhs: BitVecValue,
) -> Result<(), BitVecEvalError> {
    validate_width(lhs.width).map_err(|error| {
        BitVecEvalError::new(
            error.kind(),
            format!("op={op}; operand=lhs; {}", error.detail()),
        )
    })?;
    validate_width(rhs.width).map_err(|error| {
        BitVecEvalError::new(
            error.kind(),
            format!("op={op}; operand=rhs; {}", error.detail()),
        )
    })
}

fn normalize_bits(width: u32, bits: u64) -> Result<u64, BitVecEvalError> {
    Ok(bits & width_mask(width)?)
}

fn validate_width(width: u32) -> Result<(), BitVecEvalError> {
    if SUPPORTED_BITVEC_WIDTHS.contains(&width) {
        return Ok(());
    }

    Err(BitVecEvalError::new(
        BitVecEvalErrorKind::UnsupportedWidth,
        format!("width={width}; supported={SUPPORTED_BITVEC_WIDTHS:?}"),
    ))
}

fn width_mask(width: u32) -> Result<u64, BitVecEvalError> {
    validate_width(width)?;
    if width == 64 {
        Ok(u64::MAX)
    } else {
        Ok((1u64 << width) - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(width: u32, bits: u64) -> BitVecExpr {
        BitVecExpr::literal(width, bits).expect("test literal width is supported")
    }

    fn bv(width: u32, bits: u64) -> BitVecEvalResult {
        BitVecEvalResult::BitVec(BitVecValue::new(width, bits).expect("expected value"))
    }

    fn eval(input: &BitVecExpr) -> BitVecEvalResult {
        evaluate_bitvec_expr(input).expect("ground expression evaluates")
    }

    #[test]
    fn evaluates_documented_ground_bitvec_fixtures() {
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::Add,
                lit(8, 1),
                lit(8, 1)
            )),
            bv(8, 2)
        );
        assert_eq!(
            eval(&BitVecExpr::unary(BitVecUnaryOp::Not, lit(8, 0))),
            bv(8, 0xff)
        );
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::And,
                lit(8, 0xff),
                lit(8, 1),
            )),
            bv(8, 1)
        );
        assert_eq!(
            eval(&BitVecExpr::compare(
                BitVecComparisonOp::Slt,
                lit(8, 0xff),
                lit(8, 0),
            )),
            BitVecEvalResult::Bool(true)
        );
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::Xor,
                lit(16, 1),
                lit(16, 1),
            )),
            bv(16, 0)
        );
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::Shl,
                lit(32, 1),
                lit(32, 1),
            )),
            bv(32, 2)
        );
        assert_eq!(
            eval(&BitVecExpr::compare(
                BitVecComparisonOp::Ult,
                lit(64, 0),
                lit(64, 1),
            )),
            BitVecEvalResult::Bool(true)
        );
    }

    #[test]
    fn arithmetic_wraps_to_width() {
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::Add,
                lit(8, 0xff),
                lit(8, 1),
            )),
            bv(8, 0)
        );
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::Sub,
                lit(8, 0),
                lit(8, 1)
            )),
            bv(8, 0xff)
        );
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::Mul,
                lit(8, 0x10),
                lit(8, 0x10),
            )),
            bv(8, 0)
        );
        assert_eq!(
            eval(&BitVecExpr::unary(BitVecUnaryOp::Neg, lit(16, 1))),
            bv(16, 0xffff)
        );
    }

    #[test]
    fn shifts_follow_fixed_width_bitvector_semantics() {
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::Lshr,
                lit(8, 0xff),
                lit(8, 1),
            )),
            bv(8, 0x7f)
        );
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::Ashr,
                lit(8, 0xff),
                lit(8, 1),
            )),
            bv(8, 0xff)
        );
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::Shl,
                lit(8, 1),
                lit(8, 8)
            )),
            bv(8, 0)
        );
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::Ashr,
                lit(8, 0x80),
                lit(8, 8),
            )),
            bv(8, 0xff)
        );
        assert_eq!(
            eval(&BitVecExpr::binary(
                BitVecBinaryOp::Shl,
                lit(64, 1),
                lit(8, 63),
            )),
            bv(64, 1u64 << 63)
        );
    }

    #[test]
    fn signed_and_unsigned_comparisons_are_distinct() {
        assert_eq!(
            eval(&BitVecExpr::compare(
                BitVecComparisonOp::Ult,
                lit(8, 0xff),
                lit(8, 0),
            )),
            BitVecEvalResult::Bool(false)
        );
        assert_eq!(
            eval(&BitVecExpr::compare(
                BitVecComparisonOp::Slt,
                lit(8, 0xff),
                lit(8, 0),
            )),
            BitVecEvalResult::Bool(true)
        );
        assert_eq!(
            BitVecValue::new(64, u64::MAX)
                .expect("supported width")
                .signed_value()
                .expect("signed value"),
            -1
        );
    }

    #[test]
    fn rejects_unsupported_width() {
        let expr = BitVecExpr::Literal(BitVecValue { width: 7, bits: 1 });

        let error = evaluate_bitvec_expr(&expr).expect_err("bad width rejects");

        assert_eq!(error.kind(), BitVecEvalErrorKind::UnsupportedWidth);
        assert_eq!(error.detail(), "width=7; supported=[8, 16, 32, 64]");
    }

    #[test]
    fn rejects_width_mismatch() {
        let expr = BitVecExpr::binary(BitVecBinaryOp::Add, lit(8, 1), lit(16, 1));

        let error = evaluate_bitvec_expr(&expr).expect_err("mismatched widths reject");

        assert_eq!(error.kind(), BitVecEvalErrorKind::WidthMismatch);
        assert_eq!(error.detail(), "op=add; lhs_width=8; rhs_width=16");
    }

    #[test]
    fn rejects_bool_result_as_bitvector_operand() {
        let bool_expr = BitVecExpr::compare(BitVecComparisonOp::Ult, lit(8, 0), lit(8, 1));
        let expr = BitVecExpr::binary(BitVecBinaryOp::Add, bool_expr, lit(8, 1));

        let error = evaluate_bitvec_expr(&expr).expect_err("bool operand rejects");

        assert_eq!(error.kind(), BitVecEvalErrorKind::ExpectedBitVector);
        assert_eq!(error.detail(), "op=add; operand=lhs; actual=bool");
    }

    #[test]
    fn rejects_oversized_expression() {
        let mut expr = lit(8, 0);
        for _ in 0..MAX_BITVEC_EXPR_NODES {
            expr = BitVecExpr::unary(BitVecUnaryOp::Not, expr);
        }

        let error = evaluate_bitvec_expr(&expr).expect_err("large expression rejects");

        assert_eq!(error.kind(), BitVecEvalErrorKind::ExpressionTooLarge);
        assert_eq!(error.detail(), "nodes=257; max=256");
    }
}
