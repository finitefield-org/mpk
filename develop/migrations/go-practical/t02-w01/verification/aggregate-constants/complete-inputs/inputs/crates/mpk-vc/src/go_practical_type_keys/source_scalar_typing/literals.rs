//! Check actual literal element syntax, including contextual elided types.
use super::*;
impl Engine<'_> {
    fn literal_indices(&mut self, source: &str, node: usize) -> Result<Vec<(u64, usize)>> {
        let mut result = vec![];
        let mut seen = BTreeSet::new();
        let mut next = 0u64;
        for element in list(self.node(source, node)?, "Elts").to_vec() {
            let e = self.node(source, element)?.clone();
            let value = if e.kind() == "KeyValueExpr" {
                let key = self.exact_constant(source, child(&e, "Key")?)?;
                next = self
                    .index_number(&Atom {
                        typ: None,
                        kind: Some(key.kind()),
                        nil: false,
                        value: Some(key),
                        addressable: false,
                    })?
                    .ok_or(GoScalarTypingError::Type)?;
                child(&e, "Value")?
            } else {
                element
            };
            if !seen.insert(next) {
                return Err(GoScalarTypingError::Type);
            }
            result.push((next, value));
            next = next.checked_add(1).ok_or(GoScalarTypingError::Limit)?;
        }
        Ok(result)
    }
    pub(super) fn literal(
        &mut self,
        source: &str,
        node: usize,
        expected: Option<&str>,
    ) -> Result<Plan> {
        let n = self.node(source, node)?.clone();
        let typ = if let Some(&t) = list(&n, "Type").first() {
            let tn = self.node(source, t)?;
            if tn.kind() == "ArrayType"
                && list(tn, "Len")
                    .first()
                    .is_some_and(|&n| self.node(source, n).is_ok_and(|n| n.kind() == "Ellipsis"))
            {
                let element = self.annotation(source, child(tn, "Elt")?)?;
                let length = self
                    .literal_indices(source, node)?
                    .iter()
                    .map(|(i, _)| i + 1)
                    .max()
                    .unwrap_or(0);
                self.aggregate_key(Some(length), element)?
            } else {
                self.annotation(source, t)?
            }
        } else {
            expected.ok_or(GoScalarTypingError::Type)?.to_string()
        };
        let mut args = vec![];
        match self.shape(&typ)?.clone() {
            GoSourceTypeKey::Array {
                element, length, ..
            } => {
                for (i, n) in self.literal_indices(source, node)? {
                    if i >= length {
                        return Err(GoScalarTypingError::Type);
                    }
                    args.push(Request::element(source, n, element.clone()));
                }
            }
            GoSourceTypeKey::Slice { element, .. } => {
                for (_, n) in self.literal_indices(source, node)? {
                    args.push(Request::element(source, n, element.clone()));
                }
            }
            GoSourceTypeKey::Struct { fields, .. } => {
                let elements = list(&n, "Elts");
                let keyed = elements.first().is_some_and(|&n| {
                    self.node(source, n)
                        .is_ok_and(|n| n.kind() == "KeyValueExpr")
                });
                let mut seen = BTreeSet::new();
                if !keyed && !elements.is_empty() && elements.len() != fields.len() {
                    return Err(GoScalarTypingError::Arity);
                }
                let package = self
                    .scopes
                    .source_package(source)
                    .ok_or(GoScalarTypingError::Locator)?;
                for (ordinal, &element) in elements.iter().enumerate() {
                    let e = self.node(source, element)?;
                    let (index, value) = if keyed {
                        if e.kind() != "KeyValueExpr" {
                            return Err(GoScalarTypingError::Type);
                        }
                        let key = self.node(source, child(e, "Key")?)?;
                        if key.kind() != "Ident" {
                            return Err(GoScalarTypingError::Type);
                        }
                        let name = attr(key, "Name").ok_or(GoScalarTypingError::Locator)?;
                        if name == "_" {
                            return Err(GoScalarTypingError::Name);
                        }
                        (
                            fields
                                .iter()
                                .position(|f| f.name == name)
                                .ok_or(GoScalarTypingError::Name)?,
                            child(e, "Value")?,
                        )
                    } else {
                        if e.kind() == "KeyValueExpr" {
                            return Err(GoScalarTypingError::Type);
                        }
                        (ordinal, element)
                    };
                    let field = &fields[index];
                    if field
                        .unexported_package_id
                        .as_deref()
                        .is_some_and(|p| p != package)
                        || !seen.insert(index)
                    {
                        return Err(GoScalarTypingError::Name);
                    }
                    args.push(Request::value(source, value, Some(field.type_id.clone())));
                }
            }
            _ => return Err(GoScalarTypingError::Type),
        }
        Ok(Plan::Literal(typ, args))
    }
}
