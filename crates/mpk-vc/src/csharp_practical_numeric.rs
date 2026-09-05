//! W11 private numeric relations. No host floating arithmetic or BCL oracle.
//! These bounded integer algorithms are the concrete T03 semantics; T06 owns
//! expansion into checked core definitions and discharge, never a theory node.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericError {
    Signature,
    OperandType,
    Overflow,
    DivideByZero,
    Range,
}
impl NumericError {
    pub fn exception_type(self) -> Option<&'static str> {
        match self {
            Self::Overflow => Some("System.OverflowException"),
            Self::DivideByZero => Some("System.DivideByZeroException"),
            Self::Range => Some("System.ArgumentOutOfRangeException"),
            _ => None,
        }
    }
}
// The largest aligned binary64 intermediate has < 2200 bits. This private
// natural has no source-visible arbitrary-precision type or unbounded input.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Nat(Vec<u32>);
impl Ord for Nat {
    fn cmp(&self, b: &Self) -> Ordering {
        self.0
            .len()
            .cmp(&b.0.len())
            .then_with(|| self.0.iter().rev().cmp(b.0.iter().rev()))
    }
}
impl PartialOrd for Nat {
    fn partial_cmp(&self, b: &Self) -> Option<Ordering> {
        Some(self.cmp(b))
    }
}
impl Nat {
    fn new(mut words: Vec<u32>) -> Self {
        while words.last() == Some(&0) {
            words.pop();
        }
        assert!(words.len() <= 72);
        Self(words)
    }
    fn from(n: u128) -> Self {
        Self::new((0..4).map(|i| (n >> (i * 32)) as u32).collect())
    }
    fn zero(&self) -> bool {
        self.0.is_empty()
    }
    fn bits(&self) -> usize {
        self.0
            .last()
            .map_or(0, |n| self.0.len() * 32 - n.leading_zeros() as usize)
    }
    fn small(&self) -> u128 {
        assert!(self.bits() <= 128);
        self.0
            .iter()
            .rev()
            .fold(0, |n, w| (n << 32) | u128::from(*w))
    }
    fn shl(&self, n: usize) -> Self {
        let mut out = vec![0; self.0.len() + n / 32 + 1];
        for (i, w) in self.0.iter().enumerate() {
            let v = u64::from(*w) << (n % 32);
            out[i + n / 32] |= v as u32;
            out[i + n / 32 + 1] |= (v >> 32) as u32;
        }
        Self::new(out)
    }
    fn add(&self, b: &Self) -> Self {
        let mut out = Vec::new();
        let mut carry = 0u64;
        for i in 0..self.0.len().max(b.0.len()) {
            let n = u64::from(*self.0.get(i).unwrap_or(&0))
                + u64::from(*b.0.get(i).unwrap_or(&0))
                + carry;
            out.push(n as u32);
            carry = n >> 32;
        }
        out.push(carry as u32);
        Self::new(out)
    }
    fn sub(&self, b: &Self) -> Self {
        assert!(self >= b);
        let mut out = Vec::new();
        let mut borrow = 0i64;
        for (i, w) in self.0.iter().enumerate() {
            let n = i64::from(*w) - i64::from(*b.0.get(i).unwrap_or(&0)) - borrow;
            out.push(n as u32);
            borrow = i64::from(n < 0);
        }
        Self::new(out)
    }
    fn mul(&self, b: &Self) -> Self {
        let mut out = vec![0; self.0.len() + b.0.len()];
        for (i, a) in self.0.iter().enumerate() {
            let mut carry = 0u64;
            for (j, b) in b.0.iter().enumerate() {
                let n = u64::from(*a) * u64::from(*b) + u64::from(out[i + j]) + carry;
                out[i + j] = n as u32;
                carry = n >> 32;
            }
            if !b.zero() {
                out[i + b.0.len()] = carry as u32;
            }
        }
        Self::new(out)
    }
    fn div(&self, b: &Self) -> (Self, Self) {
        assert!(!b.zero());
        let mut rem = self.clone();
        let mut q = vec![0; self.0.len()];
        if self >= b {
            for shift in (0..=self.bits() - b.bits()).rev() {
                let d = b.shl(shift);
                if rem >= d {
                    rem = rem.sub(&d);
                    q[shift / 32] |= 1 << (shift % 32);
                }
            }
        }
        (Self::new(q), rem)
    }
    fn pow10(n: u32) -> Self {
        (0..n).fold(Self::from(1), |v, _| v.mul(&Self::from(10)))
    }
}
fn rounded(n: &Nat, d: &Nat, negative: bool, mode: CodecRounding) -> Nat {
    let (q, r) = n.div(d);
    let cmp = r.shl(1).cmp(d);
    let up = match mode {
        CodecRounding::ToEven => {
            cmp == Ordering::Greater
                || (cmp == Ordering::Equal && q.0.first().is_some_and(|x| x & 1 != 0))
        }
        CodecRounding::AwayFromZero => cmp != Ordering::Less,
        CodecRounding::ToZero => false,
        CodecRounding::ToNegativeInfinity => negative && !r.zero(),
        CodecRounding::ToPositiveInfinity => !negative && !r.zero(),
    };
    if up {
        q.add(&Nat::from(1))
    } else {
        q
    }
}
fn signed_add(an: bool, a: &Nat, bn: bool, b: &Nat) -> (bool, Nat) {
    if an == bn {
        (an, a.add(b))
    } else if a >= b {
        (an, a.sub(b))
    } else {
        (bn, b.sub(a))
    }
}
fn ty(token: &str) -> String {
    format!("mpk.csharp.value.{token}.v1")
}
fn boolean(v: bool) -> MonomorphicValue {
    MonomorphicValue::Bool {
        type_id: ty("bool"),
        value: v,
    }
}
fn comparison(op: &str, order: Option<Ordering>) -> Option<bool> {
    Some(match op {
        "equal" | "value_equality" => order == Some(Ordering::Equal),
        "not_equal" => order != Some(Ordering::Equal),
        "less" => order == Some(Ordering::Less),
        "greater" => order == Some(Ordering::Greater),
        "less_equal" => matches!(order, Some(Ordering::Less | Ordering::Equal)),
        "greater_equal" => matches!(order, Some(Ordering::Greater | Ordering::Equal)),
        _ => return None,
    })
}
#[derive(Clone, Copy)]
struct Float {
    bits: u64,
    frac: u32,
    eb: u32,
}
impl Float {
    fn width(self) -> u32 {
        1 + self.frac + self.eb
    }
    fn sign(self) -> u64 {
        1 << (self.width() - 1)
    }
    fn negative(self) -> bool {
        self.bits & self.sign() != 0
    }
    fn mask(self) -> u64 {
        (1 << self.frac) - 1
    }
    fn exponent(self) -> u64 {
        (self.bits >> self.frac) & ((1 << self.eb) - 1)
    }
    fn special(self) -> bool {
        self.exponent() == (1 << self.eb) - 1
    }
    fn nan(self) -> bool {
        self.special() && self.bits & self.mask() != 0
    }
    fn quiet(self) -> u64 {
        1 << (self.frac - 1)
    }
    fn signaling(self) -> bool {
        self.nan() && self.bits & self.quiet() == 0
    }
    fn infinity(self) -> u64 {
        ((1 << self.eb) - 1) << self.frac
    }
    fn zero(self) -> bool {
        self.bits & !self.sign() == 0
    }
    fn parts(self) -> (Nat, i32) {
        let exponent = self.exponent();
        (
            Nat::from(u128::from(
                (self.bits & self.mask()) | if exponent == 0 { 0 } else { 1 << self.frac },
            )),
            exponent.max(1) as i32 - ((1 << (self.eb - 1)) - 1) - self.frac as i32,
        )
    }
    fn value(self, bits: u64) -> MonomorphicValue {
        if self.frac == 23 {
            MonomorphicValue::F32Bits {
                type_id: ty("f32"),
                bits: format!("{bits:08x}"),
            }
        } else {
            MonomorphicValue::F64Bits {
                type_id: ty("f64"),
                bits: format!("{bits:016x}"),
            }
        }
    }
    fn pack(self, negative: bool, n: Nat, d: Nat, exponent: i32) -> u64 {
        let sign = if negative { self.sign() } else { 0 };
        if n.zero() {
            return sign;
        }
        let mut log = n.bits() as i32 - d.bits() as i32;
        if if log >= 0 {
            n < d.shl(log as usize)
        } else {
            n.shl((-log) as usize) < d
        } {
            log -= 1;
        }
        let bias = (1 << (self.eb - 1)) - 1;
        let min = 1 - bias;
        let max = bias;
        let mut e = log + exponent;
        if e > max {
            return sign | self.infinity();
        }
        let shift = exponent - e.max(min) + self.frac as i32;
        let q = if shift >= 0 {
            rounded(&n.shl(shift as usize), &d, false, CodecRounding::ToEven)
        } else {
            rounded(&n, &d.shl((-shift) as usize), false, CodecRounding::ToEven)
        };
        let mut sig = q.small() as u64;
        if e < min {
            e = min;
        }
        if sig >= 1 << (self.frac + 1) {
            sig >>= 1;
            e += 1;
        }
        if e > max {
            return sign | self.infinity();
        }
        let exp = if sig < 1 << self.frac {
            0
        } else {
            (e + bias) as u64
        };
        sign | (exp << self.frac) | (sig & self.mask())
    }
    fn order(self, b: Self) -> Option<Ordering> {
        if self.nan() || b.nan() {
            None
        } else if self.zero() && b.zero() {
            Some(Ordering::Equal)
        } else {
            Some(if self.negative() != b.negative() {
                b.negative().cmp(&self.negative())
            } else if self.negative() {
                b.bits.cmp(&self.bits)
            } else {
                self.bits.cmp(&b.bits)
            })
        }
    }
    fn run(self, op: &str, b: Option<Self>) -> MonomorphicValue {
        let raw = match op {
            "plus" => Some(self.bits),
            "negate" => Some(self.bits ^ self.sign()),
            "abs" => Some(self.bits & !self.sign()),
            _ => None,
        };
        if let Some(raw) = raw {
            return self.value(raw);
        }
        if matches!(op, "is_nan" | "is_infinity" | "is_finite") {
            return boolean(match op {
                "is_nan" => self.nan(),
                "is_infinity" => self.special() && !self.nan(),
                _ => !self.special(),
            });
        }
        let b = b.expect("validated binary");
        if let Some(v) = comparison(op, self.order(b)) {
            return boolean(v);
        }
        if matches!(op, "min" | "max") {
            return self.value(if self.nan() {
                self.bits
            } else if b.nan() {
                b.bits
            } else if self.zero() && b.zero() {
                if op == "min" {
                    self.bits | b.bits
                } else {
                    self.bits & b.bits
                }
            } else if (self.order(b) == Some(Ordering::Less)) == (op == "min") {
                self.bits
            } else {
                b.bits
            });
        }
        // Frozen target propagates signaling operands before quiet operands;
        // Min/Max and unary sign preserve the original, unquieted payload.
        if self.nan() || b.nan() {
            let n = if self.signaling() {
                self
            } else if b.signaling() {
                b
            } else if self.nan() {
                self
            } else {
                b
            };
            return self.value(n.bits | self.quiet());
        }
        let (a, ae) = self.parts();
        let (bv, be) = b.parts();
        let an = self.negative();
        let bn = b.negative();
        let sign = if an ^ bn { self.sign() } else { 0 };
        let invalid = self.infinity() | self.quiet();
        let raw = match op {
            "add" | "subtract" => {
                let bn = bn ^ (op == "subtract");
                if self.special() && b.special() && an != bn {
                    invalid
                } else if self.special() {
                    self.bits
                } else if b.special() {
                    (b.bits & !self.sign()) | if bn { self.sign() } else { 0 }
                } else {
                    let e = ae.min(be);
                    let (neg, n) = signed_add(
                        an,
                        &a.shl((ae - e) as usize),
                        bn,
                        &bv.shl((be - e) as usize),
                    );
                    self.pack(if n.zero() { an && bn } else { neg }, n, Nat::from(1), e)
                }
            }
            "multiply" => {
                if self.special() && b.zero() || b.special() && self.zero() {
                    invalid
                } else if self.special() || b.special() {
                    sign | self.infinity()
                } else {
                    self.pack(an ^ bn, a.mul(&bv), Nat::from(1), ae + be)
                }
            }
            "divide" => {
                if self.special() && b.special() || self.zero() && b.zero() {
                    invalid
                } else if self.special() || b.zero() {
                    sign | self.infinity()
                } else if b.special() {
                    sign
                } else {
                    self.pack(an ^ bn, a, bv, ae - be)
                }
            }
            "remainder" => {
                if self.special() || b.zero() {
                    invalid
                } else if b.special() {
                    self.bits
                } else {
                    let e = ae.min(be);
                    let (_, rem) = a.shl((ae - e) as usize).div(&bv.shl((be - e) as usize));
                    self.pack(an, rem, Nat::from(1), e)
                }
            }
            _ => unreachable!("validated op"),
        };
        self.value(raw)
    }
}
fn as_float(v: &MonomorphicValue) -> Float {
    match v {
        MonomorphicValue::F32Bits { bits, .. } => Float {
            bits: u64::from_str_radix(bits, 16).unwrap(),
            frac: 23,
            eb: 8,
        },
        MonomorphicValue::F64Bits { bits, .. } => Float {
            bits: u64::from_str_radix(bits, 16).unwrap(),
            frac: 52,
            eb: 11,
        },
        _ => unreachable!("validated float"),
    }
}
#[derive(Clone)]
struct Decimal {
    negative: bool,
    n: Nat,
    scale: u32,
}
impl Decimal {
    fn read(v: &MonomorphicValue) -> Self {
        let MonomorphicValue::DecimalBits {
            negative,
            coefficient,
            scale,
            ..
        } = v
        else {
            unreachable!("validated decimal")
        };
        Self {
            negative: *negative,
            n: Nat::from(coefficient.parse().unwrap()),
            scale: u32::from(*scale),
        }
    }
    fn value(self) -> MonomorphicValue {
        MonomorphicValue::DecimalBits {
            type_id: ty("decimal"),
            negative: self.negative,
            coefficient: self.n.small().to_string(),
            scale: self.scale as u8,
        }
    }
    fn fit(negative: bool, n: Nat, scale: u32) -> Result<Self, NumericError> {
        for target in (0..=scale.min(28)).rev() {
            let q = rounded(
                &n,
                &Nat::pow10(scale - target),
                negative,
                CodecRounding::ToEven,
            );
            if q.bits() <= 96 {
                return Ok(Self {
                    negative,
                    n: q,
                    scale: target,
                });
            }
        }
        Err(NumericError::Overflow)
    }
    fn run(
        mut self,
        op: &str,
        b: Option<Self>,
        digits: i32,
        mode: CodecRounding,
    ) -> Result<MonomorphicValue, NumericError> {
        if matches!(op, "plus" | "negate" | "literal") {
            self.negative ^= op == "negate";
            return Ok(self.value());
        }
        if matches!(op, "round" | "truncate" | "floor" | "ceiling") {
            if !(0..=28).contains(&digits) {
                return Err(NumericError::Range);
            }
            let (target, mode) = match op {
                "truncate" => (0, CodecRounding::ToZero),
                "floor" => (0, CodecRounding::ToNegativeInfinity),
                "ceiling" => (0, CodecRounding::ToPositiveInfinity),
                _ => (digits as u32, mode),
            };
            if target < self.scale {
                self.n = rounded(
                    &self.n,
                    &Nat::pow10(self.scale - target),
                    self.negative,
                    mode,
                );
                self.scale = target;
            }
            return Ok(self.value());
        }
        let b = b.expect("validated decimal binary");
        let scale = self.scale.max(b.scale);
        let a = self.n.mul(&Nat::pow10(scale - self.scale));
        let bv = b.n.mul(&Nat::pow10(scale - b.scale));
        let order = if a.zero() && bv.zero() {
            Ordering::Equal
        } else if self.negative != b.negative {
            b.negative.cmp(&self.negative)
        } else if self.negative {
            bv.cmp(&a)
        } else {
            a.cmp(&bv)
        };
        if let Some(v) = comparison(op, Some(order)) {
            return Ok(boolean(v));
        }
        let neg = self.negative ^ b.negative;
        Ok(match op {
            "add" | "subtract" => {
                let (neg, n) = signed_add(self.negative, &a, b.negative ^ (op == "subtract"), &bv);
                Self::fit(neg, n, scale)?
            }
            "multiply" => Self::fit(neg, self.n.mul(&b.n), self.scale + b.scale)?,
            "remainder" => {
                if bv.zero() {
                    return Err(NumericError::DivideByZero);
                }
                Self::fit(self.negative, a.div(&bv).1, scale)?
            }
            "divide" => {
                if bv.zero() {
                    return Err(NumericError::DivideByZero);
                }
                let mut result = None;
                for target in (0..=28).rev() {
                    let q = rounded(&a.mul(&Nat::pow10(target)), &bv, neg, CodecRounding::ToEven);
                    if q.bits() <= 96 {
                        result = Some(Self {
                            negative: neg,
                            n: q,
                            scale: target,
                        });
                        break;
                    }
                }
                result.ok_or(NumericError::Overflow)?
            }
            _ => unreachable!("validated decimal op"),
        }
        .value())
    }
}

fn carrier(name: &str) -> Option<&'static str> {
    Some(match name {
        "sbyte" => "i8",
        "byte" => "u8",
        "int16" => "i16",
        "uint16" => "u16",
        "int32" => "i32",
        "uint32" => "u32",
        "int64" => "i64",
        "uint64" => "u64",
        "char" => "char",
        "single" => "f32",
        "double" => "f64",
        "decimal" => "decimal",
        _ => return None,
    })
}
fn integral(token: &str) -> bool {
    matches!(
        token,
        "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" | "char"
    )
}
fn conversion(id: &str) -> Option<(&'static str, &'static str)> {
    let text = id
        .strip_prefix("numeric.conversion.")
        .or_else(|| id.strip_prefix("decimal.conversion."))?;
    let checked = text.ends_with(".checked");
    let text = text.strip_suffix(".checked").unwrap_or(text);
    let (a, b) = text.split_once("_to_")?;
    let a = carrier(a)?;
    let b = carrier(b)?;
    let legal = if id.starts_with("decimal.") {
        !checked && ((a == "decimal" && integral(b)) || (integral(a) && b == "decimal"))
    } else {
        matches!(
            (a, b, checked),
            ("i32", "f32", false)
                | ("i64", "f64", false)
                | ("f32", "f64", false)
                | ("f64", "f32", false)
                | ("f32", "i32", true)
                | ("f64", "i64", true)
        )
    };
    legal.then_some((a, b))
}
/// A closed typed operation recipe. Regenerate this from exact Roslyn bindings;
/// configuration is not a user-extensible operation/theory registration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NumericOperation {
    id: String,
    arguments: Vec<String>,
    result: String,
    rounding: CodecRounding,
}
impl NumericOperation {
    pub fn new(
        id: &str,
        arguments: &[String],
        result: &str,
        rounding: Option<&str>,
    ) -> Result<Self, NumericError> {
        let mode = rounding
            .map(CodecRounding::from_id)
            .transpose()
            .map_err(|_| NumericError::Signature)?
            .unwrap_or(CodecRounding::ToEven);
        if rounding.is_some() && id != "decimal.round" {
            return Err(NumericError::Signature);
        }
        let mut expected = Vec::new();
        let out;
        if let Some((a, b)) = conversion(id) {
            expected.push(ty(a));
            out = ty(b);
        } else {
            let (token, op) = if let Some(op) = id.strip_prefix("floating.single.") {
                ("f32", op)
            } else if let Some(op) = id.strip_prefix("floating.double.") {
                ("f64", op)
            } else if let Some(op) = id.strip_prefix("decimal.") {
                ("decimal", op)
            } else {
                return Err(NumericError::Signature);
            };
            let compare = comparison(op, None).is_some();
            let unary = matches!(op, "plus" | "negate" | "literal")
                || token != "decimal"
                    && matches!(op, "abs" | "is_nan" | "is_infinity" | "is_finite")
                || token == "decimal" && matches!(op, "round" | "truncate" | "floor" | "ceiling");
            let binary = compare
                || matches!(op, "add" | "subtract" | "multiply" | "divide" | "remainder")
                || token != "decimal" && matches!(op, "min" | "max");
            if !unary && !binary || op == "value_equality" && token != "decimal" {
                return Err(NumericError::Signature);
            }
            expected.push(ty(token));
            if binary {
                expected.push(ty(token));
            }
            if op == "round" && arguments.len() == 2 {
                expected.push(ty("i32"));
            }
            out = ty(if compare || op.starts_with("is_") {
                "bool"
            } else {
                token
            });
        }
        if arguments != expected || result != out {
            return Err(NumericError::Signature);
        }
        Ok(Self {
            id: id.into(),
            arguments: expected,
            result: out,
            rounding: mode,
        })
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn result_type_id(&self) -> &str {
        &self.result
    }
    pub fn argument_type_ids(&self) -> &[String] {
        &self.arguments
    }
    pub fn exception_types(&self) -> Vec<&'static str> {
        if self.id == "decimal.round" {
            if self.arguments.len() == 2 {
                vec!["System.ArgumentOutOfRangeException"]
            } else {
                vec![]
            }
        } else if matches!(self.id.as_str(), "decimal.divide" | "decimal.remainder") {
            if self.id.ends_with("remainder") {
                vec!["System.DivideByZeroException"]
            } else {
                vec!["System.DivideByZeroException", "System.OverflowException"]
            }
        } else if matches!(
            self.id.as_str(),
            "decimal.add" | "decimal.subtract" | "decimal.multiply"
        ) || self.id.ends_with(".checked")
            || self.id.starts_with("decimal.conversion.decimal_to_")
        {
            vec!["System.OverflowException"]
        } else {
            vec![]
        }
    }
    /// The only scalar grammar stays in W10, including representation-sensitive
    /// float bits and value-based normalized decimal boundary equality.
    pub fn boundary_codec(type_id: &str) -> Result<BoundaryCodec, CodecError> {
        let id = match type_id {
            "mpk.csharp.value.f32.v1" => "binary32",
            "mpk.csharp.value.f64.v1" => "binary64",
            "mpk.csharp.value.decimal.v1" => "decimal.normalized",
            _ => return Err(CodecError::OperandType),
        };
        BoundaryCodec::new(id, type_id, None, None)
    }
    pub fn evaluate(
        &self,
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        operands: &[MonomorphicValue],
    ) -> Result<MonomorphicValue, NumericError> {
        if operands.len() != self.arguments.len() {
            return Err(NumericError::Signature);
        }
        for (v, t) in operands.iter().zip(&self.arguments) {
            if v.type_id() != t || validate_monomorphic_value(b, r, c, v).is_err() {
                return Err(NumericError::OperandType);
            }
        }
        let value = if self.id.ends_with(".literal") {
            operands[0].clone()
        } else if let Some((from, to)) = conversion(&self.id) {
            convert(&operands[0], from, to)?
        } else if self.id.starts_with("floating.") {
            as_float(&operands[0]).run(
                self.id.rsplit('.').next().unwrap(),
                operands.get(1).map(as_float),
            )
        } else {
            let digits = if let Some(MonomorphicValue::Signed { value, .. }) = operands.get(1) {
                value.parse().unwrap()
            } else {
                0
            };
            Decimal::read(&operands[0]).run(
                self.id.strip_prefix("decimal.").unwrap(),
                operands
                    .get(1)
                    .filter(|v| v.type_id() == ty("decimal"))
                    .map(Decimal::read),
                digits,
                self.rounding,
            )?
        };
        if value.type_id() != self.result || validate_monomorphic_value(b, r, c, &value).is_err() {
            return Err(NumericError::OperandType);
        }
        Ok(value)
    }
}
fn integer_parts(v: &MonomorphicValue) -> (bool, Nat) {
    match v {
        MonomorphicValue::Signed { value, .. } => {
            let n: i128 = value.parse().unwrap();
            (n < 0, Nat::from(n.unsigned_abs()))
        }
        MonomorphicValue::Unsigned { value, .. } => (false, Nat::from(value.parse().unwrap())),
        MonomorphicValue::Char { utf16, .. } => (false, Nat::from(u128::from(*utf16))),
        _ => unreachable!("validated integer"),
    }
}
fn integer_value(negative: bool, n: Nat, to: &str) -> Result<MonomorphicValue, NumericError> {
    let signed = to.starts_with('i');
    let width: u32 = if to == "char" {
        16
    } else {
        to[1..].parse().unwrap()
    };
    let max = if signed {
        (1u128 << (width - 1)) - u128::from(!negative)
    } else {
        (1u128 << width) - 1
    };
    if (!signed && negative && !n.zero()) || n > Nat::from(max) {
        return Err(NumericError::Overflow);
    }
    let value = format!(
        "{}{}",
        if negative && !n.zero() { "-" } else { "" },
        n.small()
    );
    Ok(if to == "char" {
        MonomorphicValue::Char {
            type_id: ty(to),
            utf16: n.small() as u16,
        }
    } else if signed {
        MonomorphicValue::Signed {
            type_id: ty(to),
            value,
        }
    } else {
        MonomorphicValue::Unsigned {
            type_id: ty(to),
            value,
        }
    })
}
fn convert(v: &MonomorphicValue, from: &str, to: &str) -> Result<MonomorphicValue, NumericError> {
    if to == "decimal" {
        let (negative, n) = integer_parts(v);
        return Ok(Decimal {
            negative,
            n,
            scale: 0,
        }
        .value());
    }
    if from == "decimal" {
        let dec = Decimal::read(v);
        return integer_value(dec.negative, dec.n.div(&Nat::pow10(dec.scale)).0, to);
    }
    if integral(to) {
        let f = as_float(v);
        if f.special() {
            return Err(NumericError::Overflow);
        }
        let (n, e) = f.parts();
        let n = if e >= 0 {
            n.shl(e as usize)
        } else {
            n.div(&Nat::from(1).shl((-e) as usize)).0
        };
        return integer_value(f.negative(), n, to);
    }
    let target = if to == "f32" {
        Float {
            bits: 0,
            frac: 23,
            eb: 8,
        }
    } else {
        Float {
            bits: 0,
            frac: 52,
            eb: 11,
        }
    };
    let raw = if integral(from) {
        let (negative, n) = integer_parts(v);
        target.pack(negative, n, Nat::from(1), 0)
    } else {
        let f = as_float(v);
        let sign = if f.negative() { target.sign() } else { 0 };
        if f.special() {
            let payload = if target.frac > f.frac {
                (f.bits & f.mask()) << (target.frac - f.frac)
            } else {
                (f.bits & f.mask()) >> (f.frac - target.frac)
            };
            sign | target.infinity() | payload | if f.nan() { target.quiet() } else { 0 }
        } else {
            let (n, e) = f.parts();
            target.pack(f.negative(), n, Nat::from(1), e)
        }
    };
    Ok(target.value(raw))
}
