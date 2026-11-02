//! Shared value-type registry, assignment/conversion and comparison rules.
use super::*;
impl Engine<'_> {
    fn contains_call(&self, r: &Request) -> Result<bool> {
        let mut work = vec![r.expression()?];
        let mut seen = BTreeSet::new();
        while let Some(node) = work.pop() {
            if !seen.insert(node) {
                continue;
            }
            let n = self.node(&r.source, node)?;
            if n.kind() == "CallExpr" {
                let fun = child(n, "Fun")?;
                match self.symbol(&r.source, fun) {
                    Ok(GoNameLookup::Original(b)) if b.kind() == GoSourceNameKind::Function => {
                        return Ok(true)
                    }
                    Ok(GoNameLookup::Predeclared {
                        kind: GoSourceNameKind::BuiltinFunction,
                        ..
                    }) => {
                        let fact = self.one(&Request::raw(&r.source, node))?;
                        if fact.value.is_none() {
                            return Ok(true);
                        }
                    }
                    _ => {}
                }
            }
            for edge in n.edges() {
                work.extend(edge.targets());
            }
        }
        Ok(false)
    }
    pub(super) fn len_cap(
        &mut self,
        _r: &Request,
        name: &str,
        args: &[Request],
        values: Vec<Atom>,
    ) -> Result<Step> {
        if values.len() != 1 {
            return Err(GoScalarTypingError::Arity);
        }
        let a = &values[0];
        let length = if a.kind == Some(GoConstantKind::String) {
            if name != "len" {
                return Err(GoScalarTypingError::Type);
            }
            a.value
                .as_ref()
                .and_then(|c| c.bytes())
                .map(|b| b.len() as u64)
        } else {
            match self.shape(a.typ.as_deref().ok_or(GoScalarTypingError::Type)?)? {
                GoSourceTypeKey::Array { length, .. } => {
                    if self.contains_call(&args[0])? {
                        None
                    } else {
                        Some(*length)
                    }
                }
                GoSourceTypeKey::Slice { .. } => None,
                _ => return Err(GoScalarTypingError::Type),
            }
        };
        if let Some(length) = length {
            return Ok(Step::Done(vec![self.constant(
                source_constants::length_constant(length).map_err(constant_error)?,
            )?]));
        }
        let typ = self.builtin("int")?;
        Ok(Step::Done(vec![self.atom(&typ, false)]))
    }
    pub(super) fn put_key(&mut self, key: GoSourceTypeKey) -> Result<String> {
        let id = key.id().to_string();
        if let Some(old) = self.keys.get(&id) {
            if old != &key {
                return Err(GoScalarTypingError::Type);
            }
        } else if self.keys.len() >= 4_194_304 {
            return Err(GoScalarTypingError::Limit);
        }
        self.keys.insert(id.clone(), key);
        Ok(id)
    }
    pub(super) fn shape(&self, id: &str) -> Result<&GoSourceTypeKey> {
        let mut id = id;
        let mut seen = BTreeSet::new();
        while let Some(next) = self.underlying.get(id) {
            if !seen.insert(id) {
                return Err(GoScalarTypingError::Cycle);
            }
            id = next;
        }
        self.keys.get(id).ok_or(GoScalarTypingError::Type)
    }
    pub(super) fn assignable(&self, from: &str, to: &str) -> bool {
        let named = |id: &str| {
            matches!(
                self.keys.get(id),
                Some(GoSourceTypeKey::Builtin { .. } | GoSourceTypeKey::Nominal { .. })
            )
        };
        from == to
            || self.is_error(to)&&self.error_implementations.get(from).is_some_and(Option::is_some)
            || (!named(from) || !named(to))
                && self
                    .shape(from)
                    .ok()
                    .zip(self.shape(to).ok())
                    .is_some_and(|(a, b)| a.id() == b.id())
    }
    pub(super) fn has_nil(&self, id: &str) -> bool {
        matches!(self.shape(id), Ok(GoSourceTypeKey::Slice { .. }))
            || matches!(self.shape(id),Ok(GoSourceTypeKey::Builtin{name,..})if name=="error")
    }
    pub(super) fn aggregate_key(&mut self, length: Option<u64>, element: String) -> Result<String> {
        if length.is_some_and(|n| n > 256) {
            return Err(GoScalarTypingError::Limit);
        }
        let mut key = if let Some(length) = length {
            GoSourceTypeKey::Array {
                schema: TYPE_SCHEMA.into(),
                id: String::new(),
                length,
                element,
            }
        } else {
            GoSourceTypeKey::Slice {
                schema: TYPE_SCHEMA.into(),
                id: String::new(),
                element,
            }
        };
        let id = identity(&key, TYPE_DOMAIN, "id").map_err(|_| GoScalarTypingError::Type)?;
        match &mut key {
            GoSourceTypeKey::Array { id: field, .. } | GoSourceTypeKey::Slice { id: field, .. } => {
                *field = id.clone()
            }
            _ => unreachable!(),
        }
        self.put_key(key)?;
        // The graph is acyclic. Explicit work preserves the aggregate-depth16
        // bound without putting expression/helper-work depth on the Rust stack.
        let mut work = vec![(id.clone(), 0u32)];
        let mut seen = BTreeMap::new();
        while let Some((t, depth)) = work.pop() {
            if seen.get(&t).is_some_and(|old| *old >= depth) {
                continue;
            }
            seen.insert(t.clone(), depth);
            let key = self.shape(&t)?;
            let depth = depth
                + u32::from(matches!(
                    key,
                    GoSourceTypeKey::Array { .. }
                        | GoSourceTypeKey::Slice { .. }
                        | GoSourceTypeKey::Struct { .. }
                ));
            if depth > 16 {
                return Err(GoScalarTypingError::Limit);
            }
            for child in key.edges() {
                work.push((child.into(), depth));
            }
        }
        Ok(id)
    }
    fn same_ignoring_tags(&self, a: &str, b: &str) -> Result<bool> {
        let mut work = vec![(
            self.shape(a)?.id().to_string(),
            self.shape(b)?.id().to_string(),
        )];
        let mut seen = BTreeSet::new();
        while let Some((a, b)) = work.pop() {
            if a == b || !seen.insert((a.clone(), b.clone())) {
                continue;
            }
            match (self.keys.get(&a), self.keys.get(&b)) {
                (
                    Some(GoSourceTypeKey::Array {
                        length: a,
                        element: x,
                        ..
                    }),
                    Some(GoSourceTypeKey::Array {
                        length: b,
                        element: y,
                        ..
                    }),
                ) if a == b => work.push((x.clone(), y.clone())),
                (
                    Some(GoSourceTypeKey::Slice { element: x, .. }),
                    Some(GoSourceTypeKey::Slice { element: y, .. }),
                ) => work.push((x.clone(), y.clone())),
                (
                    Some(GoSourceTypeKey::Struct { fields: a, .. }),
                    Some(GoSourceTypeKey::Struct { fields: b, .. }),
                ) if a.len() == b.len() => {
                    for (x, y) in a.iter().zip(b) {
                        if x.name != y.name
                            || x.unexported_package_id != y.unexported_package_id
                            || x.embedded != y.embedded
                        {
                            return Ok(false);
                        }
                        work.push((x.type_id.clone(), y.type_id.clone()));
                    }
                }
                _ => return Ok(false),
            }
        }
        Ok(true)
    }
    pub(super) fn cast(&mut self, typ: String, arg: Request) -> Result<Step> {
        let a = self.one(&arg)?;
        if a.nil {
            if !self.has_nil(&typ) {
                return Err(GoScalarTypingError::Type);
            }
            return Ok(Step::Done(vec![self.atom(&typ, false)]));
        }
        if self.is_error(&typ){
            if !a.typ.as_ref().is_some_and(|from|self.assignable(from,&typ)){return Err(GoScalarTypingError::Type)}
            return Ok(Step::Done(vec![self.atom(&typ,false)]));
        }
        if let Some(value) = a.value {
            if let Some(scalar) = self.types.get(&typ) {
                return Ok(Step::Done(vec![self.constant(
                    value
                        .convert(scalar.clone(), true)
                        .map_err(constant_error)?,
                )?]));
            }
            // String-to-byte/rune-slice conversion yields a runtime slice.
            if value.kind() == GoConstantKind::String && self.string_slice(&typ) {
                return Ok(Step::Done(vec![self.atom(&typ, false)]));
            }
            return Err(GoScalarTypingError::Type);
        }
        let Some(from) = a.typ else {
            return Ok(Step::More(Plan::Forward(Request::value(
                &arg.source,
                arg.expression()?,
                Some(typ),
            ))));
        };
        if let (Some(from), Some(to)) = (self.types.get(&from), self.types.get(&typ)) {
            if from.underlying_builtin() != to.underlying_builtin()
                && !(from.integer_shape().is_some()
                    && (to.integer_shape().is_some() || to.underlying_builtin() == "string"))
            {
                return Err(GoScalarTypingError::Type);
            }
        } else if !(self.same_ignoring_tags(&from, &typ)?
            || self.is_string(&from) && self.string_slice(&typ)
            || self.string_slice(&from) && self.is_string(&typ))
        {
            // A slice can convert to an array of the identical element type.
            // Its dynamic minimum length is a later definedness obligation.
            if !matches!((self.shape(&from)?,self.shape(&typ)?),(GoSourceTypeKey::Slice{element:a,..},GoSourceTypeKey::Array{element:b,..})if a==b)
            {
                return Err(GoScalarTypingError::Type);
            }
        }
        Ok(Step::Done(vec![self.atom(&typ, false)]))
    }
    fn is_string(&self, id: &str) -> bool {
        matches!(self.shape(id),Ok(GoSourceTypeKey::Builtin{name,..})if name=="string")
    }
    fn string_slice(&self, id: &str) -> bool {
        let Ok(GoSourceTypeKey::Slice { element, .. }) = self.shape(id) else {
            return false;
        };
        matches!(self.shape(element),Ok(GoSourceTypeKey::Builtin{name,..})if matches!(name.as_str(),"uint8"|"int32"))
    }
    fn comparable(&self, id: &str) -> Result<bool> {
        let mut work = vec![id.to_string()];
        let mut seen = BTreeSet::new();
        while let Some(id) = work.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            match self.shape(&id)? {
                GoSourceTypeKey::Builtin { .. } => {}
                GoSourceTypeKey::Array { element, .. } => work.push(element.clone()),
                GoSourceTypeKey::Struct { fields, .. } => {
                    work.extend(fields.iter().map(|f| f.type_id.clone()))
                }
                _ => return Ok(false),
            }
        }
        Ok(true)
    }
    pub(super) fn aggregate_comparison(&self, op: &str, a: &Atom, b: &Atom) -> Result<Atom> {
        if !matches!(op, "==" | "!=") {
            return Err(GoScalarTypingError::Type);
        }
        if a.nil && a.typ.is_none() || b.nil && b.typ.is_none() {
            let other = if a.nil && a.typ.is_none() { b } else { a };
            if !other.typ.as_ref().is_some_and(|t| self.has_nil(t)) {
                return Err(GoScalarTypingError::Type);
            }
        } else {
            let a = a.typ.as_ref().ok_or(GoScalarTypingError::Type)?;
            let b = b.typ.as_ref().ok_or(GoScalarTypingError::Type)?;
            if !(self.assignable(a, b) || self.assignable(b, a))
                || !self.comparable(a)?
                || !self.comparable(b)?
            {
                return Err(GoScalarTypingError::Type);
            }
        }
        Ok(Atom {
            typ: None,
            kind: Some(GoConstantKind::Bool),
            nil: false,
            value: None,
            addressable: false,
        })
    }
}
