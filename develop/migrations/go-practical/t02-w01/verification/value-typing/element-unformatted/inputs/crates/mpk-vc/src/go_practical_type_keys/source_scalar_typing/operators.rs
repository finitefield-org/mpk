use super::*;
pub(super) fn comparison(op: &str) -> bool {
    matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=")
}
fn integer(kind: Option<GoConstantKind>) -> bool {
    matches!(kind, Some(GoConstantKind::Integer | GoConstantKind::Rune))
}
impl Engine<'_> {
    pub(super) fn compatible_kind(&self, k: Option<GoConstantKind>, typ: &str) -> bool {
        let Some(t) = self.types.get(typ) else {
            return false;
        };
        match k {
            Some(GoConstantKind::Bool) => t.underlying_builtin() == "bool",
            Some(GoConstantKind::String) => t.underlying_builtin() == "string",
            Some(_) => t.integer_shape().is_some(),
            None => false,
        }
    }
    pub(super) fn default_type(&mut self, k: GoConstantKind) -> Result<String> {
        self.builtin(match k {
            GoConstantKind::Integer => "int",
            GoConstantKind::Rune => "int32",
            GoConstantKind::Bool => "bool",
            GoConstantKind::String => "string",
        })
    }
    fn common_kind(&self, values: &[Atom]) -> Result<GoConstantKind> {
        let mut kinds = values
            .iter()
            .map(|a| a.kind.ok_or(GoScalarTypingError::Type));
        let mut k = kinds.next().ok_or(GoScalarTypingError::Arity)??;
        for next in kinds {
            let next = next?;
            if k == next {
                continue;
            }
            if integer(Some(k)) && integer(Some(next)) {
                k = GoConstantKind::Rune
            } else {
                return Err(GoScalarTypingError::Type);
            }
        }
        Ok(k)
    }
    fn common_type(&self, values: &[Atom]) -> Result<Option<String>> {
        let mut typ = None;
        for a in values {
            if let Some(t) = &a.typ {
                if typ.as_ref().is_some_and(|old| old != t) {
                    return Err(GoScalarTypingError::Type);
                }
                typ = Some(t.clone())
            }
        }
        Ok(typ)
    }
    fn check_binary(&self, op: &str, a: &Atom, b: &Atom) -> Result<()> {
        let common = self.common_kind(&[a.clone(), b.clone()])?;
        let allowed = match op {
            "+" => integer(Some(common)) || common == GoConstantKind::String,
            "-" | "*" | "/" | "%" | "&" | "|" | "^" | "&^" => integer(Some(common)),
            "&&" | "||" => common == GoConstantKind::Bool,
            "==" | "!=" => true,
            "<" | "<=" | ">" | ">=" => integer(Some(common)) || common == GoConstantKind::String,
            _ => false,
        };
        if !allowed {
            return Err(GoScalarTypingError::Type);
        }
        if matches!(op, "/" | "%")
            && b.value
                .as_ref()
                .and_then(GoEvaluatedConstant::integer_decimal)
                .as_deref()
                == Some("0")
        {
            return Err(GoScalarTypingError::Constant(GoConstantError::DivideByZero));
        }
        Ok(())
    }
    pub(super) fn unary(&mut self, op: &str, mut a: Atom) -> Result<Atom> {
        if let Some(value) = a.value {
            return self.constant(source_constants::unary(op, value).map_err(constant_error)?);
        }
        if !(matches!(op, "+" | "-" | "^") && integer(a.kind)
            || op == "!" && a.kind == Some(GoConstantKind::Bool))
        {
            return Err(GoScalarTypingError::Type);
        }
        a.addressable = false;
        Ok(a)
    }
    pub(super) fn binary_first(&mut self, op: String, x: Request, y: Request) -> Result<Step> {
        let a = self.one(&x)?;
        let b = self.one(&y)?;
        if a.kind.is_none() || b.kind.is_none() {
            return Ok(Step::Done(vec![self.aggregate_comparison(&op, &a, &b)?]));
        }
        if let (Some(left), Some(right)) = (&a.value, &b.value) {
            return Ok(Step::Done(vec![self.constant(
                source_constants::binary(&op, left.clone(), right.clone())
                    .map_err(constant_error)?,
            )?]));
        }
        if matches!(op.as_str(), "<<" | ">>") {
            if !integer(a.kind) || !integer(b.kind) {
                return Err(GoScalarTypingError::Type);
            }
            if let Some(value) = &b.value {
                let text = value.integer_decimal().ok_or(GoScalarTypingError::Type)?;
                if text.starts_with('-') {
                    return Err(GoScalarTypingError::Constant(GoConstantError::ShiftCount));
                }
                if b.typ.is_none() {
                    value
                        .clone()
                        .convert(
                            GoConstantScalarType::builtin("uint").map_err(constant_error)?,
                            false,
                        )
                        .map_err(constant_error)?;
                }
            }
            if b.typ.is_none() && b.value.is_none() {
                let uint = self.builtin("uint")?;
                let next = Request::value(&y.source, y.expression()?, Some(uint));
                return Ok(Step::More(Plan::BinaryReady(op, x, next)));
            }
            return Ok(Step::Done(vec![self.shift(a)?]));
        }
        self.check_binary(&op, &a, &b)?;
        let k = self.common_kind(&[a.clone(), b.clone()])?;
        let typ = self.common_type(&[a.clone(), b.clone()])?;
        if let Some(typ) = typ {
            let left = if a.typ.is_none() {
                Request::value(&x.source, x.expression()?, Some(typ.clone()))
            } else {
                x
            };
            let right = if b.typ.is_none() {
                Request::value(&y.source, y.expression()?, Some(typ))
            } else {
                y
            };
            return Ok(Step::More(Plan::BinaryReady(op, left, right)));
        }
        if comparison(&op) {
            let typ = self.default_type(k)?;
            return Ok(Step::More(Plan::BinaryReady(
                op,
                Request::value(&x.source, x.expression()?, Some(typ.clone())),
                Request::value(&y.source, y.expression()?, Some(typ)),
            )));
        }
        Ok(Step::Done(vec![Atom {
            typ: None,
            kind: Some(k),
            nil: false,
            value: None,
            addressable: false,
        }]))
    }
    fn shift(&self, mut a: Atom) -> Result<Atom> {
        if !integer(a.kind) {
            return Err(GoScalarTypingError::Type);
        }
        a.value = None;
        a.addressable = false;
        Ok(a)
    }
    pub(super) fn binary(&mut self, op: &str, a: Atom, b: Atom) -> Result<Atom> {
        if matches!(op, "<<" | ">>") {
            return self.shift(a);
        }
        self.check_binary(op, &a, &b)?;
        if a.typ != b.typ {
            return Err(GoScalarTypingError::Type);
        }
        if let (Some(left), Some(right)) = (a.value.clone(), b.value.clone()) {
            return self
                .constant(source_constants::binary(op, left, right).map_err(constant_error)?);
        }
        if comparison(op) {
            return Ok(Atom {
                typ: None,
                kind: Some(GoConstantKind::Bool),
                nil: false,
                value: None,
                addressable: false,
            });
        }
        let k = self.common_kind(&[a.clone(), b])?;
        Ok(Atom {
            typ: a.typ,
            kind: Some(k),
            nil: false,
            value: None,
            addressable: false,
        })
    }
    pub(super) fn builtin_first(
        &mut self,
        r: &Request,
        name: String,
        args: Vec<Request>,
    ) -> Result<Step> {
        let values = args
            .iter()
            .map(|a| self.one(a))
            .collect::<Result<Vec<_>>>()?;
        if matches!(name.as_str(), "len" | "cap") {
            return self.len_cap(r, &name, &args, values);
        }
        if values.is_empty() {
            return Err(GoScalarTypingError::Arity);
        }
        let k = self.common_kind(&values)?;
        if k == GoConstantKind::Bool {
            return Err(GoScalarTypingError::Type);
        }
        if values.iter().all(|a| a.value.is_some()) {
            return Ok(Step::Done(vec![self.minmax(&name, values)?]));
        }
        // Unlike a bare nonconstant shift, a nonconstant min/max call is a
        // materialized value. Go defaults an otherwise untyped result inside
        // the call, before an enclosing assignment or conversion is checked.
        let typ = match self.common_type(&values)? {
            Some(typ) => typ,
            None => self.default_type(k)?,
        };
        {
            let next = args
                .iter()
                .zip(values)
                .map(|(r, a)| {
                    if a.typ.is_none() {
                        Ok(Request::value(
                            &r.source,
                            r.expression()?,
                            Some(typ.clone()),
                        ))
                    } else {
                        Ok(r.clone())
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Step::More(Plan::BuiltinReady(name, next)))
        }
    }
    pub(super) fn minmax(&mut self, name: &str, values: Vec<Atom>) -> Result<Atom> {
        let k = self.common_kind(&values)?;
        if k == GoConstantKind::Bool {
            return Err(GoScalarTypingError::Type);
        }
        let typ = self.common_type(&values)?;
        if values.iter().all(|a| a.value.is_some()) {
            let args = values
                .into_iter()
                .map(|a| a.value.ok_or(GoScalarTypingError::Type))
                .collect::<Result<Vec<_>>>()?;
            return self.constant(source_constants::min_max(name, args).map_err(constant_error)?);
        }
        if values.iter().any(|a| a.typ != typ) {
            return Err(GoScalarTypingError::Type);
        }
        Ok(Atom {
            typ,
            kind: Some(k),
            nil: false,
            value: None,
            addressable: false,
        })
    }
}
