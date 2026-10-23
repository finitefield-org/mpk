//! Original fields, indices and slice bounds. No runtime access is authorized.
use super::*;
impl Engine<'_> {
    pub(super) fn is_package_selector(&self, source: &str, node: usize) -> Result<bool> {
        let base = child(self.node(source, node)?, "X")?;
        Ok(self.node(source, base)?.kind() == "Ident"
            && matches!(self.scopes.lookup_identifier(source,base),Ok(Some(GoNameLookup::Original(b)))if b.kind()==GoSourceNameKind::Import))
    }
    fn method_blocks(&mut self, id: &str, name: &str) -> Result<bool> {
        Ok(!self.direct_methods(id, name)?.is_empty())
    }
    pub(super) fn field(&mut self, source: &str, base: Atom, name: &str) -> Result<Atom> {
        if name == "_" {
            return Err(GoScalarTypingError::Name);
        }
        let package = self
            .scopes
            .source_package(source)
            .ok_or(GoScalarTypingError::Locator)?;
        // Count distinct embedding paths. Deduplicating a repeated type would
        // incorrectly make a diamond embedding's ambiguous selector unique.
        let mut level = BTreeMap::from([(base.typ.clone().ok_or(GoScalarTypingError::Type)?, 1u8)]);
        for _ in 0..=16 {
            let mut found = None;
            let mut matches = 0u8;
            let mut next: BTreeMap<String, u8> = BTreeMap::new();
            for (typ, count) in level {
                if self.method_blocks(&typ, name)? {
                    return Err(GoScalarTypingError::Unsupported);
                }
                if let GoSourceTypeKey::Struct { fields, .. } = self.shape(&typ)? {
                    for f in fields {
                        if f.name == name
                            && f.unexported_package_id
                                .as_deref()
                                .is_none_or(|p| p == package)
                        {
                            matches = matches.saturating_add(count).min(2);
                            found = Some(f.type_id.clone());
                        }
                        if f.embedded {
                            let entry = next.entry(f.type_id.clone()).or_default();
                            *entry = entry.saturating_add(count).min(2);
                        }
                    }
                }
            }
            if matches == 1 {
                return Ok(self.atom(
                    found.as_deref().ok_or(GoScalarTypingError::Type)?,
                    base.addressable,
                ));
            }
            if matches > 1 {
                return Err(GoScalarTypingError::Name);
            }
            if next.is_empty() {
                return Err(GoScalarTypingError::Name);
            }
            level = next;
        }
        Err(GoScalarTypingError::Limit)
    }
    pub(super) fn index_number(&self, a: &Atom) -> Result<Option<u64>> {
        if !matches!(a.kind, Some(GoConstantKind::Integer | GoConstantKind::Rune)) {
            return Err(GoScalarTypingError::Type);
        }
        if let Some(value) = &a.value {
            let n = value
                .integer_decimal()
                .ok_or(GoScalarTypingError::Type)?
                .parse::<u64>()
                .map_err(|_| GoScalarTypingError::Type)?;
            if n > i64::MAX as u64 {
                return Err(GoScalarTypingError::Type);
            }
            Ok(Some(n))
        } else {
            Ok(None)
        }
    }
    pub(super) fn index(&mut self, base: Atom, index: Atom) -> Result<Atom> {
        let i = self.index_number(&index)?;
        let typ = base.typ.as_deref();
        if base.kind == Some(GoConstantKind::String) {
            if let (Some(i), Some(value)) = (i, &base.value) {
                if i >= value.bytes().ok_or(GoScalarTypingError::Type)?.len() as u64 {
                    return Err(GoScalarTypingError::Type);
                }
            }
            // Runtime indexing materializes an otherwise untyped string
            // operand. Retain that actual source type in the selected domain.
            if base.typ.is_none() {
                self.builtin("string")?;
            }
            let id = self.builtin("uint8")?;
            return Ok(self.atom(&id, false));
        }
        let (element, addressable, bound) =
            match self.shape(typ.ok_or(GoScalarTypingError::Type)?)? {
                GoSourceTypeKey::Array {
                    element, length, ..
                } => (element.clone(), base.addressable, Some(*length)),
                GoSourceTypeKey::Slice { element, .. } => (element.clone(), true, None),
                _ => return Err(GoScalarTypingError::Type),
            };
        if i.zip(bound).is_some_and(|(i, n)| i >= n) {
            return Err(GoScalarTypingError::Type);
        }
        Ok(self.atom(&element, addressable))
    }
    pub(super) fn slice(
        &mut self,
        base: Atom,
        bounds: &[Option<Atom>],
        full: bool,
    ) -> Result<Atom> {
        let mut limit = None;
        let typ = if base.kind == Some(GoConstantKind::String) {
            if full {
                return Err(GoScalarTypingError::Type);
            }
            if let Some(value) = &base.value {
                limit = Some(value.bytes().ok_or(GoScalarTypingError::Type)?.len() as u64);
            }
            match base.typ {
                Some(t) => t,
                None => self.builtin("string")?,
            }
        } else {
            let typ = base.typ.ok_or(GoScalarTypingError::Type)?;
            match self.shape(&typ)?.clone() {
                GoSourceTypeKey::Array {
                    length, element, ..
                } => {
                    if !base.addressable {
                        return Err(GoScalarTypingError::Type);
                    }
                    limit = Some(length);
                    self.aggregate_key(None, element)?
                }
                GoSourceTypeKey::Slice { .. } => typ,
                _ => return Err(GoScalarTypingError::Type),
            }
        };
        if full && (bounds[1].is_none() || bounds[2].is_none()) {
            return Err(GoScalarTypingError::Type);
        }
        let mut indices = vec![];
        for (i, a) in bounds.iter().enumerate() {
            let n = match a {
                Some(a) => self.index_number(a)?,
                None if i == 0 => Some(0),
                None => limit,
            };
            if n.zip(limit).is_some_and(|(i, n)| i > n) {
                return Err(GoScalarTypingError::Type);
            }
            indices.push(n);
        }
        for i in 0..3 {
            for j in i + 1..3 {
                if indices[i].zip(indices[j]).is_some_and(|(a, b)| a > b) {
                    return Err(GoScalarTypingError::Type);
                }
            }
        }
        Ok(self.atom(&typ, false))
    }
}
