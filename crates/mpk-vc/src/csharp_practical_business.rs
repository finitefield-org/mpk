//! W13 private business-value relations. Only explicit operands participate;
//! source binding and invariant obligations remain pending until T06.
use super::*;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BusinessError {
    Signature,
    OperandType,
    ArgumentOutOfRange,
    Overflow,
    Precision,
    Range,
    InvalidCurrency,
    InvalidScale,
    InvalidPrecision,
    CurrencyMismatch,
    InvalidRounding,
    DivisionByZero,
    DecimalOverflow,
    Binding,
    ObservationLoss,
}
impl BusinessError {
    pub fn exception_type(self) -> Option<&'static str> {
        match self {
            Self::ArgumentOutOfRange => Some("System.ArgumentOutOfRangeException"),
            Self::Overflow => Some("System.OverflowException"),
            _ => None,
        }
    }
    pub fn error_id(self) -> Option<&'static str> {
        Some(match self {
            Self::Precision => "precision",
            Self::Range => "range",
            Self::InvalidCurrency => "invalid_currency",
            Self::InvalidScale => "invalid_scale",
            Self::InvalidPrecision => "invalid_precision",
            Self::CurrencyMismatch => "currency_mismatch",
            Self::InvalidRounding => "invalid_rounding",
            Self::DivisionByZero => "division_by_zero",
            Self::DecimalOverflow => "decimal_overflow",
            _ => return None,
        })
    }
}
fn ty(token: &str) -> String {
    format!("mpk.csharp.value.{token}.v1")
}
fn signed(token: &str, n: i128) -> MonomorphicValue {
    MonomorphicValue::Signed {
        type_id: ty(token),
        value: n.to_string(),
    }
}
const DAY: i128 = 864_000_000_000;
fn days_before_year(y: i128) -> i128 {
    let y = y - 1;
    365 * y + y / 4 - y / 100 + y / 400
}
fn month_days(y: i128, m: i128) -> i128 {
    match m {
        2 => {
            if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
fn date_number(y: i128, m: i128, d: i128) -> Result<u32, BusinessError> {
    if !(1..=9999).contains(&y) || !(1..=12).contains(&m) || d < 1 || d > month_days(y, m) {
        return Err(BusinessError::ArgumentOutOfRange);
    }
    Ok((days_before_year(y) + (1..m).map(|m| month_days(y, m)).sum::<i128>() + d - 1) as u32)
}
fn date_parts(n: u32) -> (i128, i128, i128) {
    let mut lo = 1;
    let mut hi = 10000;
    while lo + 1 < hi {
        let mid = (lo + hi) / 2;
        if days_before_year(mid) <= i128::from(n) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let mut d = i128::from(n) - days_before_year(lo);
    let mut m = 1;
    while d >= month_days(lo, m) {
        d -= month_days(lo, m);
        m += 1;
    }
    (lo, m, d + 1)
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BusinessOperation {
    id: String,
    arguments: Vec<String>,
    result: String,
}
impl BusinessOperation {
    pub fn new(id: &str, arguments: &[String], result: &str) -> Result<Self, BusinessError> {
        let (token, op) = id.split_once('.').ok_or(BusinessError::Signature)?;
        if !matches!(
            token,
            "date" | "time" | "duration" | "guid" | "instant" | "day_of_week"
        ) {
            return Err(BusinessError::Signature);
        }
        let own = ty(token);
        let (args, out) = match (token, op) {
            ("date", "construct") => (vec![ty("i32"); 3], own.clone()),
            ("time" | "duration", "construct") => (vec![ty("i64")], own.clone()),
            ("date", "add_days" | "add_months" | "add_years") => {
                (vec![own.clone(), ty("i32")], own.clone())
            }
            ("date", "year" | "month" | "day" | "day_number")
            | ("time", "hour" | "minute" | "second" | "millisecond")
            | ("duration", "days" | "hours" | "minutes" | "seconds" | "milliseconds") => {
                (vec![own.clone()], ty("i32"))
            }
            ("date", "day_of_week") => (vec![own.clone()], ty("day_of_week")),
            ("time" | "duration", "ticks") | ("instant", "milliseconds") => {
                (vec![own.clone()], ty("i64"))
            }
            ("time", "add_duration") | ("instant", "add_duration" | "subtract_duration") => {
                (vec![own.clone(), ty("duration")], own.clone())
            }
            ("time", "subtract") | ("instant", "difference") => {
                (vec![own.clone(); 2], ty("duration"))
            }
            ("duration", "add" | "subtract") => (vec![own.clone(); 2], own.clone()),
            ("duration", "negate") => (vec![own.clone()], own.clone()),
            ("guid", "empty") => (vec![], own.clone()),
            (_, "compare") => (vec![own.clone(); 2], ty("i32")),
            (_, "equal" | "not_equal") => (vec![own.clone(); 2], ty("bool")),
            (
                "date" | "time" | "duration" | "instant" | "day_of_week",
                "less" | "less_equal" | "greater" | "greater_equal",
            ) => (vec![own.clone(); 2], ty("bool")),
            _ => return Err(BusinessError::Signature),
        };
        if args != arguments || out != result {
            return Err(BusinessError::Signature);
        }
        Ok(Self {
            id: id.into(),
            arguments: args,
            result: out,
        })
    }
    pub fn argument_type_ids(&self) -> &[String] {
        &self.arguments
    }
    pub fn result_type_id(&self) -> &str {
        &self.result
    }
    pub fn exception_types(&self) -> Vec<&'static str> {
        match self.id.as_str() {
            "date.construct" | "date.add_days" | "date.add_months" | "date.add_years"
            | "time.construct" => vec!["System.ArgumentOutOfRangeException"],
            "duration.add" | "duration.subtract" | "duration.negate" => {
                vec!["System.OverflowException"]
            }
            _ => vec![],
        }
    }
    pub fn ordered_errors(&self) -> Vec<&'static str> {
        match self.id.as_str() {
            "instant.add_duration" | "instant.subtract_duration" => vec!["precision", "range"],
            "instant.difference" => vec!["range"],
            _ => vec![],
        }
    }
    pub fn boundary_codec(
        token: &str,
        variant: Option<&str>,
    ) -> Result<BoundaryCodec, BusinessError> {
        let id = match (token, variant) {
            ("date", None) => "date",
            ("time", None) => "time",
            ("duration", None) => "duration_ticks",
            ("instant", None) => "unix_milliseconds",
            ("guid", Some("n")) => "guid.n",
            ("guid", Some("d")) => "guid.d",
            _ => return Err(BusinessError::Signature),
        };
        BoundaryCodec::new(id, &ty(token), None, None).map_err(|_| BusinessError::Signature)
    }
    pub fn evaluate(
        &self,
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        values: &[MonomorphicValue],
    ) -> Result<MonomorphicValue, BusinessError> {
        if values.len() != self.arguments.len() {
            return Err(BusinessError::Signature);
        }
        for (v, t) in values.iter().zip(&self.arguments) {
            if v.type_id() != t || validate_monomorphic_value(b, r, c, v).is_err() {
                return Err(BusinessError::OperandType);
            }
        }
        let (token, op) = self.id.split_once('.').unwrap();
        if matches!(
            op,
            "compare" | "equal" | "not_equal" | "less" | "less_equal" | "greater" | "greater_equal"
        ) {
            let order = generate_structural_program(b, r, c, &self.arguments[0])
                .map_err(|_| BusinessError::Signature)?
                .canonical_compare(&values[0], &values[1])
                .map_err(|_| BusinessError::OperandType)?;
            return Ok(if op == "compare" {
                signed(
                    "i32",
                    match order {
                        Ordering::Less => -1,
                        Ordering::Equal => 0,
                        Ordering::Greater => 1,
                    },
                )
            } else {
                MonomorphicValue::Bool {
                    type_id: ty("bool"),
                    value: match op {
                        "equal" => order == Ordering::Equal,
                        "not_equal" => order != Ordering::Equal,
                        "less" => order == Ordering::Less,
                        "less_equal" => order != Ordering::Greater,
                        "greater" => order == Ordering::Greater,
                        _ => order != Ordering::Less,
                    },
                }
            });
        }
        let number = |i: usize| -> i128 {
            match &values[i] {
                MonomorphicValue::Signed { value, .. } => value.parse().unwrap(),
                MonomorphicValue::Date { day_number, .. } => i128::from(*day_number),
                MonomorphicValue::Time { ticks, .. } | MonomorphicValue::Duration { ticks, .. } => {
                    ticks.parse().unwrap()
                }
                MonomorphicValue::Instant { milliseconds, .. } => milliseconds.parse().unwrap(),
                _ => unreachable!(),
            }
        };
        let result = match token {
            "date" => {
                let n = if op == "construct" {
                    date_number(number(0), number(1), number(2))?
                } else {
                    number(0) as u32
                };
                let (y, m, d) = date_parts(n);
                match op {
                    "year" => signed("i32", y),
                    "month" => signed("i32", m),
                    "day" => signed("i32", d),
                    "day_number" => signed("i32", i128::from(n)),
                    "day_of_week" => MonomorphicValue::Enum {
                        type_id: ty("day_of_week"),
                        underlying: "i32".into(),
                        carrier: ((n + 1) % 7).to_string(),
                    },
                    _ => {
                        let day = match op {
                            "construct" => n,
                            "add_days" => {
                                let n = i128::from(n) + number(1);
                                if !(0..=3652058).contains(&n) {
                                    return Err(BusinessError::ArgumentOutOfRange);
                                }
                                n as u32
                            }
                            "add_months" => {
                                let offset = number(1);
                                if !(-120000..=120000).contains(&offset) {
                                    return Err(BusinessError::ArgumentOutOfRange);
                                }
                                let months = (y - 1) * 12 + m - 1 + offset;
                                let y = months.div_euclid(12) + 1;
                                let m = months.rem_euclid(12) + 1;
                                date_number(y, m, d.min(month_days(y, m)))?
                            }
                            "add_years" => {
                                let offset = number(1);
                                if !(-10000..=10000).contains(&offset) {
                                    return Err(BusinessError::ArgumentOutOfRange);
                                }
                                let y = y + offset;
                                date_number(y, m, d.min(month_days(y, m)))?
                            }
                            _ => unreachable!(),
                        };
                        MonomorphicValue::Date {
                            type_id: ty("date"),
                            day_number: day,
                        }
                    }
                }
            }
            "time" => {
                let ticks = number(0);
                match op {
                    "construct" => {
                        if !(0..DAY).contains(&ticks) {
                            return Err(BusinessError::ArgumentOutOfRange);
                        }
                        MonomorphicValue::Time {
                            type_id: ty("time"),
                            ticks: ticks.to_string(),
                        }
                    }
                    "ticks" => signed("i64", ticks),
                    "hour" => signed("i32", ticks / 36_000_000_000),
                    "minute" => signed("i32", ticks / 600_000_000 % 60),
                    "second" => signed("i32", ticks / 10_000_000 % 60),
                    "millisecond" => signed("i32", ticks / 10_000 % 1000),
                    "subtract" => MonomorphicValue::Duration {
                        type_id: ty("duration"),
                        ticks: (ticks - number(1)).rem_euclid(DAY).to_string(),
                    },
                    "add_duration" => MonomorphicValue::Time {
                        type_id: ty("time"),
                        ticks: (ticks + number(1)).rem_euclid(DAY).to_string(),
                    },
                    _ => unreachable!(),
                }
            }
            "duration" => {
                let ticks = number(0);
                match op {
                    "ticks" => signed("i64", ticks),
                    "days" => signed("i32", ticks / DAY),
                    "hours" => signed("i32", ticks / 36_000_000_000 % 24),
                    "minutes" => signed("i32", ticks / 600_000_000 % 60),
                    "seconds" => signed("i32", ticks / 10_000_000 % 60),
                    "milliseconds" => signed("i32", ticks / 10_000 % 1000),
                    _ => {
                        let n = match op {
                            "construct" => ticks,
                            "add" => ticks + number(1),
                            "subtract" => ticks - number(1),
                            "negate" => -ticks,
                            _ => unreachable!(),
                        };
                        if i64::try_from(n).is_err() {
                            return Err(BusinessError::Overflow);
                        }
                        MonomorphicValue::Duration {
                            type_id: ty("duration"),
                            ticks: n.to_string(),
                        }
                    }
                }
            }
            "instant" => {
                let ms = number(0);
                if op == "milliseconds" {
                    signed("i64", ms)
                } else {
                    let right = number(1);
                    if op != "difference" && right % 10000 != 0 {
                        return Err(BusinessError::Precision);
                    }
                    let n = match op {
                        "difference" => (ms - right) * 10000,
                        "add_duration" => ms + right / 10000,
                        "subtract_duration" => ms - right / 10000,
                        _ => unreachable!(),
                    };
                    if i64::try_from(n).is_err() {
                        return Err(BusinessError::Range);
                    }
                    if op == "difference" {
                        MonomorphicValue::Duration {
                            type_id: ty("duration"),
                            ticks: n.to_string(),
                        }
                    } else {
                        MonomorphicValue::Instant {
                            type_id: ty("instant"),
                            milliseconds: n.to_string(),
                        }
                    }
                }
            }
            "guid" => MonomorphicValue::Guid {
                type_id: ty("guid"),
                n: "0".repeat(32),
            },
            _ => return Err(BusinessError::Signature),
        };
        validate_monomorphic_value(b, r, c, &result).map_err(|_| BusinessError::OperandType)?;
        Ok(result)
    }
}

/// A concrete finite currency predicate for executable-spec evaluation. It is
/// explicit application input, not an ISO table or a proof of a source predicate.
/// The binding's currency/public-invariant obligations must still be discharged.
pub struct CurrencyDomain {
    type_id: String,
    accepted: Vec<MonomorphicValue>,
}
impl CurrencyDomain {
    pub fn new(
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        type_id: &str,
        accepted: Vec<MonomorphicValue>,
    ) -> Result<Self, BusinessError> {
        if type_id != ty("string")
            && !r
                .source_types
                .get(type_id)
                .is_some_and(|s| s.kind == SourceKind::Enum)
        {
            return Err(BusinessError::Signature);
        }
        for v in &accepted {
            if v.type_id() != type_id || validate_monomorphic_value(b, r, c, v).is_err() {
                return Err(BusinessError::OperandType);
            }
        }
        Ok(Self {
            type_id: type_id.into(),
            accepted,
        })
    }
    fn contains(&self, v: &MonomorphicValue) -> bool {
        self.accepted.contains(v)
    }
}
pub struct MoneyModel<'a> {
    b: &'a ValidatedFoundationBundle,
    r: &'a ValidatedClosedRootSet,
    c: &'a ClosedInstanceSet,
    id: String,
    currency: String,
}
impl<'a> MoneyModel<'a> {
    pub fn new(
        b: &'a ValidatedFoundationBundle,
        r: &'a ValidatedClosedRootSet,
        c: &'a ClosedInstanceSet,
        id: &str,
    ) -> Result<Self, BusinessError> {
        let args = require_instance(c, id, "money").map_err(|_| BusinessError::Signature)?;
        if args[0] != ty("string")
            && !r
                .source_types
                .get(&args[0])
                .is_some_and(|s| s.kind == SourceKind::Enum)
        {
            return Err(BusinessError::Signature);
        }
        Ok(Self {
            b,
            r,
            c,
            id: id.into(),
            currency: args[0].clone(),
        })
    }
    fn valid(&self, v: &MonomorphicValue, t: &str) -> Result<(), BusinessError> {
        if v.type_id() != t || validate_monomorphic_value(self.b, self.r, self.c, v).is_err() {
            Err(BusinessError::OperandType)
        } else {
            Ok(())
        }
    }
    fn parts<'v>(
        &self,
        v: &'v MonomorphicValue,
    ) -> Result<(&'v MonomorphicValue, &'v MonomorphicValue), BusinessError> {
        self.valid(v, &self.id)?;
        let MonomorphicValue::Money {
            amount, currency, ..
        } = v
        else {
            return Err(BusinessError::OperandType);
        };
        Ok((amount, currency))
    }
    pub fn amount<'v>(
        &self,
        value: &'v MonomorphicValue,
    ) -> Result<&'v MonomorphicValue, BusinessError> {
        self.parts(value).map(|(amount, _)| amount)
    }
    pub fn currency<'v>(
        &self,
        value: &'v MonomorphicValue,
    ) -> Result<&'v MonomorphicValue, BusinessError> {
        self.parts(value).map(|(_, currency)| currency)
    }
    fn money(&self, amount: MonomorphicValue, currency: MonomorphicValue) -> MonomorphicValue {
        MonomorphicValue::Money {
            type_id: self.id.clone(),
            amount: Box::new(amount),
            currency: Box::new(currency),
        }
    }
    fn decimal(
        &self,
        op: &str,
        args: &[MonomorphicValue],
        rounding: Option<&str>,
    ) -> Result<MonomorphicValue, BusinessError> {
        NumericOperation::new(
            &format!("decimal.{op}"),
            &args
                .iter()
                .map(|v| v.type_id().to_owned())
                .collect::<Vec<_>>(),
            &ty(if matches!(op, "equal" | "less" | "greater") {
                "bool"
            } else {
                "decimal"
            }),
            rounding,
        )
        .map_err(|_| BusinessError::Signature)?
        .evaluate(self.b, self.r, self.c, args)
        .map_err(|e| match e {
            NumericError::Overflow => BusinessError::DecimalOverflow,
            NumericError::DivideByZero => BusinessError::DivisionByZero,
            _ => BusinessError::OperandType,
        })
    }
    pub fn create(
        &self,
        amount: MonomorphicValue,
        currency: MonomorphicValue,
        scale: i32,
        predicate: &CurrencyDomain,
    ) -> Result<MonomorphicValue, BusinessError> {
        self.valid(&amount, &ty("decimal"))?;
        self.valid(&currency, &self.currency)?;
        if predicate.type_id != self.currency {
            return Err(BusinessError::Signature);
        }
        if !predicate.contains(&currency) {
            return Err(BusinessError::InvalidCurrency);
        }
        if !(0..=28).contains(&scale) {
            return Err(BusinessError::InvalidScale);
        }
        let MonomorphicValue::DecimalBits {
            coefficient,
            scale: stored,
            ..
        } = &amount
        else {
            return Err(BusinessError::OperandType);
        };
        if i32::from(*stored) > scale
            && coefficient.parse::<u128>().unwrap() % 10u128.pow(u32::from(*stored) - scale as u32)
                != 0
        {
            return Err(BusinessError::InvalidPrecision);
        }
        Ok(self.money(amount, currency))
    }
    pub fn add_or_subtract(
        &self,
        left: &MonomorphicValue,
        right: &MonomorphicValue,
        subtract: bool,
    ) -> Result<MonomorphicValue, BusinessError> {
        let (a, ac) = self.parts(left)?;
        let (b, bc) = self.parts(right)?;
        if ac != bc {
            return Err(BusinessError::CurrencyMismatch);
        }
        let amount = self.decimal(
            if subtract { "subtract" } else { "add" },
            &[a.clone(), b.clone()],
            None,
        )?;
        Ok(self.money(amount, ac.clone()))
    }
    pub fn scale(
        &self,
        value: &MonomorphicValue,
        quantity: MonomorphicValue,
        scale: i32,
        rounding: i32,
        divide: bool,
    ) -> Result<MonomorphicValue, BusinessError> {
        let (amount, currency) = self.parts(value)?;
        self.valid(&quantity, &ty("decimal"))?;
        if !(0..=28).contains(&scale) {
            return Err(BusinessError::InvalidScale);
        }
        let mode = match rounding {
            0 => "ToEven",
            1 => "AwayFromZero",
            2 => "ToZero",
            3 => "ToNegativeInfinity",
            4 => "ToPositiveInfinity",
            _ => return Err(BusinessError::InvalidRounding),
        };
        if divide
            && matches!(&quantity,MonomorphicValue::DecimalBits{coefficient,..} if coefficient=="0")
        {
            return Err(BusinessError::DivisionByZero);
        }
        let amount = self.decimal(
            if divide { "divide" } else { "multiply" },
            &[amount.clone(), quantity],
            None,
        )?;
        let amount = self.decimal(
            "round",
            &[amount, signed("i32", i128::from(scale))],
            Some(mode),
        )?;
        Ok(self.money(amount, currency.clone()))
    }
    pub fn amount_compare(
        &self,
        left: &MonomorphicValue,
        right: &MonomorphicValue,
    ) -> Result<Ordering, BusinessError> {
        let (a, ac) = self.parts(left)?;
        let (b, bc) = self.parts(right)?;
        if ac != bc {
            return Err(BusinessError::CurrencyMismatch);
        }
        relate_monomorphic_values(self.b, self.r, self.c, false, a, b)
            .map_err(|_| BusinessError::OperandType)
    }
    pub fn structural_equal(
        &self,
        left: &MonomorphicValue,
        right: &MonomorphicValue,
    ) -> Result<bool, BusinessError> {
        self.parts(left)?;
        self.parts(right)?;
        generate_structural_program(self.b, self.r, self.c, &self.id)
            .map_err(|_| BusinessError::Signature)?
            .structural_equal(left, right)
            .map_err(|_| BusinessError::OperandType)
    }
    pub fn storage_compare(
        &self,
        left: &MonomorphicValue,
        right: &MonomorphicValue,
    ) -> Result<Ordering, BusinessError> {
        self.parts(left)?;
        self.parts(right)?;
        relate_monomorphic_values(self.b, self.r, self.c, false, left, right)
            .map_err(|_| BusinessError::OperandType)
    }
    /// Field composition only; T05 owns document attachment. No scalar money codec.
    pub fn encode_amount(&self, value: &MonomorphicValue) -> Result<Vec<u16>, BusinessError> {
        let (amount, _) = self.parts(value)?;
        BoundaryCodec::new("decimal.normalized", &ty("decimal"), None, None)
            .map_err(|_| BusinessError::Signature)?
            .format(self.b, self.r, self.c, amount)
            .map_err(|_| BusinessError::OperandType)
    }
}

/// Content-bound application projection. Universal obligations and the source
/// helper bodies remain pending; this private plan is not an admission proof.
#[derive(Clone, Debug)]
pub struct BusinessBindingPlan {
    source: String,
    semantic: String,
    role: String,
    members: BTreeMap<String, String>,
    obligations: Vec<OutcomeObligation>,
}
impl BusinessBindingPlan {
    /// The enclosing T02 artifact and captured signatures must already be
    /// validated. Error maps are explicit semantic error -> source enum carrier;
    /// their commutation with the actual helper body is never inferred.
    pub fn new(
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        input: &crate::csharp_practical_source_artifacts::SemanticBindingInput,
        captured: &BTreeMap<String, ClosedOperationSignature>,
        outcomes: &[OutcomeBindingPlan],
        error_arms: &BTreeMap<String, BTreeMap<String, String>>,
    ) -> Result<Self, BusinessError> {
        let fail = BusinessError::Binding;
        let source = r
            .source_types
            .get(&input.source_type_id)
            .filter(|s| {
                s.kind != SourceKind::Enum && s.source_sha256 == input.source_content_sha256
            })
            .ok_or(fail)?;
        if !matches!(input.role.as_str(), "instant" | "money")
            || input.role == "money" && source.kind != SourceKind::ReadonlyStruct
            || !input.tag_arms.is_empty()
            || !input.bounds.is_empty()
            || input.default_arm != "ineligible"
        {
            return Err(fail);
        }
        let members: BTreeMap<_, _> = input
            .member_map
            .iter()
            .map(|m| (m.role.clone(), m.member_id.clone()))
            .collect();
        let expected: BTreeSet<_> = if input.role == "instant" {
            ["milliseconds"].into_iter().collect()
        } else {
            ["amount", "currency"].into_iter().collect()
        };
        if members.len() != input.member_map.len()
            || members.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected
            || members.values().collect::<BTreeSet<_>>().len() != members.len()
        {
            return Err(fail);
        }
        let member = |role: &str| {
            source
                .members
                .iter()
                .find(|m| m.id == members[role])
                .ok_or(fail)
        };
        let member_type = |role: &str| closed_type_id(b, &member(role)?.ty).map_err(|_| fail);
        let semantic = if input.role == "instant" {
            if member_type("milliseconds")? != ty("i64") || !input.inferred_argument_ids.is_empty()
            {
                return Err(fail);
            }
            ty("instant")
        } else {
            let currency = member_type("currency")?;
            if member_type("amount")? != ty("decimal")
                || currency != ty("string")
                    && !r
                        .source_types
                        .get(&currency)
                        .is_some_and(|s| s.kind == SourceKind::Enum)
                || input.inferred_argument_ids != [currency.clone()]
            {
                return Err(fail);
            }
            c.metadata
                .iter()
                .find(|(_, m)| {
                    template_name(&m.template_id) == Some("money")
                        && m.argument_ids == [currency.clone()]
                })
                .map(|(id, _)| id.clone())
                .ok_or(fail)?
        };
        let mut plan = Self {
            source: source.id.clone(),
            semantic: semantic.clone(),
            role: input.role.clone(),
            members: members.clone(),
            obligations: vec![],
        };
        for kind in [
            "source_invariant_implies_projection",
            "semantic_invariant_implies_reconstruction",
            "source_round_trip",
            "semantic_round_trip",
            "distinct_arms",
            "public_invariant",
            "identity_unobservable",
        ] {
            plan.obligation(kind, "");
        }
        if input.role == "money" {
            plan.obligation("application_currency_predicate", &members["currency"]);
            plan.obligation("default_ineligible", "");
        }
        for m in &source.members {
            plan.obligation("field_complete_reconstruction", &m.id);
        }
        let mut seen = BTreeSet::new();
        for mapping in &input.operation_map {
            if !seen.insert(&mapping.operation) {
                return Err(fail);
            }
            let own = semantic.clone();
            let (args, result, errors): (Vec<String>, String, Vec<&str>) =
                match (input.role.as_str(), mapping.operation.as_str()) {
                    ("instant", "milliseconds") => (vec![own], ty("i64"), vec![]),
                    (_, "compare") => (vec![own; 2], ty("i32"), vec![]),
                    ("instant", "add_duration" | "subtract_duration") => (
                        vec![own.clone(), ty("duration")],
                        own,
                        vec!["precision", "range"],
                    ),
                    ("instant", "difference") => (vec![own; 2], ty("duration"), vec!["range"]),
                    ("money", "amount") => (vec![own], ty("decimal"), vec![]),
                    ("money", "currency") => (vec![own], member_type("currency")?, vec![]),
                    ("money", "equal") => (vec![own; 2], ty("bool"), vec![]),
                    ("money", "create") => (
                        vec![ty("decimal"), member_type("currency")?, ty("i32")],
                        own,
                        vec!["invalid_currency", "invalid_scale", "invalid_precision"],
                    ),
                    ("money", "add" | "subtract") => (
                        vec![own.clone(); 2],
                        own,
                        vec!["currency_mismatch", "decimal_overflow"],
                    ),
                    ("money", "multiply" | "divide") => (
                        vec![own.clone(), ty("decimal"), ty("i32"), ty("u32")],
                        own,
                        if mapping.operation == "divide" {
                            vec![
                                "invalid_scale",
                                "invalid_rounding",
                                "division_by_zero",
                                "decimal_overflow",
                            ]
                        } else {
                            vec!["invalid_scale", "invalid_rounding", "decimal_overflow"]
                        },
                    ),
                    ("money", "amount_compare") => {
                        (vec![own; 2], ty("i32"), vec!["currency_mismatch"])
                    }
                    _ => return Err(fail),
                };
            let call = captured
                .get(&mapping.member_id)
                .filter(|s| s.id == mapping.member_id && s.tag == ClosedOperationTag::SourceCall)
                .ok_or(fail)?;
            validate_closed_operation_signature(r, c, call).map_err(|_| fail)?;
            let project_id = |id: &String| {
                if id == &source.id {
                    semantic.clone()
                } else {
                    id.clone()
                }
            };
            let mut projected_args = call
                .argument_type_ids
                .iter()
                .map(project_id)
                .collect::<Vec<_>>();
            if matches!(mapping.operation.as_str(), "multiply" | "divide") {
                if projected_args.len() != 4 {
                    return Err(fail);
                }
                let en = r
                    .source_types
                    .get(&projected_args[3])
                    .filter(|s| s.kind == SourceKind::Enum)
                    .ok_or(fail)?;
                let modes = error_arms.get(&en.id).ok_or(fail)?;
                let names: BTreeSet<_> = [
                    "ToEven",
                    "AwayFromZero",
                    "ToZero",
                    "ToNegativeInfinity",
                    "ToPositiveInfinity",
                ]
                .into_iter()
                .collect();
                if modes.keys().map(String::as_str).collect::<BTreeSet<_>>() != names
                    || modes.values().collect::<BTreeSet<_>>() != en.enum_values.iter().collect()
                    || modes.len() != en.enum_values.len()
                {
                    return Err(fail);
                }
                projected_args[3] = ty("u32");
                plan.obligation("exhaustive_rounding_projection", &en.id);
            }
            if projected_args != args {
                return Err(fail);
            }
            if errors.is_empty() {
                if project_id(&call.normal_result_type_id) != result {
                    return Err(fail);
                }
            } else {
                let outcome = outcomes
                    .iter()
                    .find(|p| p.source_type_id() == call.normal_result_type_id)
                    .ok_or(fail)?;
                let meta = c
                    .metadata
                    .get(outcome.semantic_type_id())
                    .filter(|m| template_name(&m.template_id) == Some("result"))
                    .ok_or(fail)?;
                if project_id(&meta.argument_ids[0]) != result {
                    return Err(fail);
                }
                let en = r
                    .source_types
                    .get(&meta.argument_ids[1])
                    .filter(|s| s.kind == SourceKind::Enum)
                    .ok_or(fail)?;
                let arms = error_arms.get(&en.id).ok_or(fail)?;
                let allowed: &[&str] = if input.role == "instant" {
                    &["precision", "range"]
                } else {
                    &[
                        "invalid_currency",
                        "invalid_scale",
                        "invalid_precision",
                        "currency_mismatch",
                        "invalid_rounding",
                        "division_by_zero",
                        "decimal_overflow",
                    ]
                };
                if errors.iter().any(|e| !arms.contains_key(*e))
                    || arms.keys().any(|k| !allowed.contains(&k.as_str()))
                    || arms.values().collect::<BTreeSet<_>>() != en.enum_values.iter().collect()
                    || arms.len() != en.enum_values.len()
                {
                    return Err(fail);
                }
                plan.obligation(
                    "separate_closed_result_projection",
                    outcome.source_type_id(),
                );
                plan.obligation("exhaustive_error_projection", &en.id);
            }
            for kind in [
                "operation_normal_commutation",
                "operation_error_commutation",
                "operation_exception_commutation",
            ] {
                plan.obligation(kind, &mapping.member_id);
            }
        }
        if plan.obligations.len() > PROJECTION_OBLIGATIONS_PER_BINDING_MAX as usize {
            return Err(fail);
        }
        Ok(plan)
    }
    fn obligation(&mut self, kind: &str, member: &str) {
        self.obligations.push(OutcomeObligation {
            source_type_id: self.source.clone(),
            semantic_type_id: self.semantic.clone(),
            kind: kind.into(),
            member_id: member.into(),
            discharged: false,
        });
    }
    pub fn semantic_type_id(&self) -> &str {
        &self.semantic
    }
    pub fn obligations(&self) -> &[OutcomeObligation] {
        &self.obligations
    }
    pub fn default_eligible(&self) -> bool {
        false
    }
    pub fn project(
        &self,
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        value: &MonomorphicValue,
    ) -> Result<MonomorphicValue, BusinessError> {
        validate_monomorphic_value(b, r, c, value).map_err(|_| BusinessError::OperandType)?;
        let MonomorphicValue::Product { type_id, fields } = value else {
            return Err(BusinessError::OperandType);
        };
        if type_id != &self.source {
            return Err(BusinessError::OperandType);
        }
        let get = |role: &str| {
            let name = &r.source_types[&self.source]
                .members
                .iter()
                .find(|m| m.id == self.members[role])
                .unwrap()
                .name;
            fields
                .iter()
                .find(|f| &f.name == name)
                .unwrap()
                .value
                .as_ref()
        };
        let result = if self.role == "instant" {
            let MonomorphicValue::Signed { value, .. } = get("milliseconds") else {
                return Err(BusinessError::OperandType);
            };
            MonomorphicValue::Instant {
                type_id: self.semantic.clone(),
                milliseconds: value.clone(),
            }
        } else {
            MonomorphicValue::Money {
                type_id: self.semantic.clone(),
                amount: Box::new(get("amount").clone()),
                currency: Box::new(get("currency").clone()),
            }
        };
        validate_monomorphic_value(b, r, c, &result).map_err(|_| BusinessError::OperandType)?;
        Ok(result)
    }
    /// Includes extra stored fields. Decimal representation is unobservable;
    /// all other storage remains part of the reconstruction obligation.
    pub fn check_source_round_trip(
        &self,
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        original: &MonomorphicValue,
        reconstructed: &MonomorphicValue,
    ) -> Result<(), BusinessError> {
        self.project(b, r, c, original)?;
        self.project(b, r, c, reconstructed)?;
        if source_observations_equal(original, reconstructed) {
            Ok(())
        } else {
            Err(BusinessError::ObservationLoss)
        }
    }
}
