use super::*;
use values::mask;
use GoSourceRuntimeData as D;
impl Machine<'_> {
    pub(super) fn one(
        &mut self,
        f: &mut Frame,
        n: usize,
        expected: Option<&str>,
    ) -> Run<GoSourceRuntimeValue> {
        let mut vs = self.expression(f, n, expected)?;
        if vs.len() != 1 {
            return Err(source());
        }
        Ok(vs.remove(0))
    }
    pub(super) fn expression(
        &mut self,
        f: &mut Frame,
        n: usize,
        expected: Option<&str>,
    ) -> Run<Vec<GoSourceRuntimeValue>> {
        self.tick(f, n, GoSourceExecutionEventKind::Expression)?;
        let owned_source = f.root.source_object_id().to_string();
        let src = owned_source.as_str();
        let node = self.node(f, n)?;
        let r = if node.kind() == "CompositeLit" && list(&node, "Type").is_empty() {
            Request::element(src, n, expected.ok_or_else(source)?.into())
        } else {
            Request::raw(src, n)
        };
        self.engine.compute(r.clone()).map_err(|_| source())?;
        let facts = self.engine.facts(&r).map_err(|_| source())?;
        let mut typ = facts.first().and_then(|a| a.typ.clone());
        if facts.len() == 1 {
            if facts[0].nil {
                let v = GoSourceRuntimeValue::new("", D::Nil);
                return Ok(vec![if let Some(t) = expected {
                    self.assigned(v, t)?
                } else {
                    v
                }]);
            }
            if typ.is_none() {
                typ = self.fact(f, n, expected)?.typ;
            }
            if let Some(c) = &facts[0].value {
                let mut v = self.constant(typ.as_deref().ok_or_else(source)?, c)?;
                if let Some(t) = expected {
                    v = self.assigned(v, t)?;
                }
                return Ok(vec![v]);
            }
        }
        let node = self.node(f, n)?;
        let result = match node.kind() {
            "ParenExpr" => {
                self.expression(f, child(&node, "X").map_err(|_| source())?, expected)?
            }
            "Ident" | "SelectorExpr"
                if node.kind() == "Ident"
                    || self
                        .engine
                        .is_package_selector(src, n)
                        .map_err(|_| source())? =>
            {
                let GoNameLookup::Original(b) = self.engine.symbol(src, n).map_err(|_| source())?
                else {
                    return Err(source());
                };
                vec![f.get(&binding(&b)).ok_or_else(source)?]
            }
            "SelectorExpr" => {
                let x = self.one(f, child(&node, "X").map_err(|_| source())?, None)?;
                let s = self.node(f, child(&node, "Sel").map_err(|_| source())?)?;
                let path =
                    self.field_path(x.type_id(), attr(&s, "Name").ok_or_else(source)?, src)?;
                vec![self.project(x, &path)?]
            }
            "UnaryExpr" => {
                let t = typ.as_deref().ok_or_else(source)?;
                let x = self.one(f, child(&node, "X").map_err(|_| source())?, Some(t))?;
                let data = match (attr(&node, "Op"), x.data()) {
                    (Some("!"), D::Bool(b)) => D::Bool(!b),
                    (Some("+"), D::Word(v)) => D::Word(*v),
                    (Some("-"), D::Word(v)) => {
                        D::Word(v.wrapping_neg() & mask(self.word_shape(t)?.0))
                    }
                    (Some("^"), D::Word(v)) => D::Word(!v & mask(self.word_shape(t)?.0)),
                    _ => return Err(unavailable()),
                };
                vec![GoSourceRuntimeValue::new(t, data)]
            }
            "BinaryExpr" => {
                let op = attr(&node, "Op").ok_or_else(source)?;
                let comparison = operators::comparison(op);
                let left = child(&node, "X").map_err(|_| source())?;
                let right = child(&node, "Y").map_err(|_| source())?;
                // A typed operand determines the untyped sibling's context.
                // A shift count retains its own source signedness and width.
                let lr = Request::raw(src, left);
                self.engine.compute(lr.clone()).map_err(|_| source())?;
                let rr = Request::raw(src, right);
                self.engine.compute(rr.clone()).map_err(|_| source())?;
                let la = self.engine.one(&lr).map_err(|_| source())?;
                let ra = self.engine.one(&rr).map_err(|_| source())?;
                let operand = if comparison {
                    la.typ.or(ra.typ)
                } else {
                    typ.clone()
                };
                let x = self.one(f, left, operand.as_deref())?;
                if matches!(op, "&&" | "||") {
                    let D::Bool(b) = x.data() else {
                        return Err(source());
                    };
                    if (op == "&&" && !b) || (op == "||" && *b) {
                        return Ok(vec![self.assigned(x, typ.as_deref().ok_or_else(source)?)?]);
                    }
                }
                let y = self.one(
                    f,
                    right,
                    if matches!(op, "<<" | ">>") {
                        None
                    } else {
                        operand.as_deref()
                    },
                )?;
                self.at = GoSourceSelectionRef::new(src, n);
                vec![self.binary(op, x, y, typ.as_deref().ok_or_else(source)?)?]
            }
            "CallExpr" => self.call(f, n, &node, typ.as_deref())?,
            "IndexExpr" => {
                let x = self.one(f, child(&node, "X").map_err(|_| source())?, None)?;
                let i = self.one(f, child(&node, "Index").map_err(|_| source())?, None)?;
                self.at = GoSourceSelectionRef::new(src, n);
                let i = self.index(&i, GoSourceRuntimeFault::IndexBounds)?;
                vec![match x.data() {
                    D::Bytes(s) => GoSourceRuntimeValue::new(
                        typ.as_deref().ok_or_else(source)?,
                        D::Word(
                            *s.get(i)
                                .ok_or(Halt::Fault(GoSourceRuntimeFault::IndexBounds))?
                                as u64,
                        ),
                    ),
                    D::Aggregate(xs) => xs
                        .get(i)
                        .cloned()
                        .ok_or(Halt::Fault(GoSourceRuntimeFault::IndexBounds))?,
                    D::Slice(s) => s
                        .read(i)
                        .ok_or(Halt::Fault(GoSourceRuntimeFault::IndexBounds))?,
                    _ => return Err(source()),
                }]
            }
            "SliceExpr" => {
                let base = child(&node, "X").map_err(|_| source())?;
                let base_type = self.fact(f, base, None)?.typ.ok_or_else(source)?;
                let x = if let GoSourceTypeKey::Array { length, .. } = self.shape(&base_type)? {
                    let length = usize::try_from(*length).map_err(|_| source())?;
                    let p = self.place(f, base)?;
                    GoSourceRuntimeValue::new(
                        typ.as_deref().ok_or_else(source)?,
                        D::Slice(self.array_view(f, p, length)?),
                    )
                } else {
                    self.one(f, base, None)?
                };
                let (length, capacity) = match x.data() {
                    D::Bytes(b) => (b.len(), b.len()),
                    D::Slice(s) => (s.length(), s.capacity()),
                    _ => return Err(unavailable()),
                };
                let mut bounds = [Some(0), Some(length), Some(capacity)];
                for (i, role) in ["Low", "High", "Max"].iter().enumerate() {
                    if let Some(&b) = list(&node, role).first() {
                        let v = self.one(f, b, None)?;
                        self.at = GoSourceSelectionRef::new(src, n);
                        bounds[i] = Some(self.index(&v, GoSourceRuntimeFault::SliceBounds)?);
                    }
                }
                let [Some(lo), Some(hi), Some(cap)] = bounds else {
                    return Err(source());
                };
                self.at = GoSourceSelectionRef::new(src, n);
                let data = match x.data() {
                    D::Bytes(b) => {
                        if lo > hi || hi > b.len() {
                            return Err(Halt::Fault(GoSourceRuntimeFault::SliceBounds));
                        }
                        D::Bytes(b[lo..hi].to_vec())
                    }
                    D::Slice(s) => D::Slice(s.subslice(lo, hi, cap)?),
                    _ => return Err(source()),
                };
                vec![GoSourceRuntimeValue::new(
                    typ.as_deref().ok_or_else(source)?,
                    data,
                )]
            }
            "CompositeLit" => vec![self.literal(f, &node, typ.as_deref().ok_or_else(source)?)?],
            _ => return Err(unavailable()),
        };
        if let Some(t) = expected {
            if result.len() != 1 {
                return Err(source());
            }
            return Ok(vec![
                self.assigned(result.into_iter().next().ok_or_else(source)?, t)?
            ]);
        }
        Ok(result)
    }
    pub(super) fn binary(
        &self,
        op: &str,
        x: GoSourceRuntimeValue,
        y: GoSourceRuntimeValue,
        typ: &str,
    ) -> Run<GoSourceRuntimeValue> {
        let data = match (x.data(), y.data()) {
            (D::Bool(a), D::Bool(b)) => D::Bool(match op {
                "&&" => *a && *b,
                "||" => *a || *b,
                "==" => a == b,
                "!=" => a != b,
                _ => return Err(source()),
            }),
            (D::Bytes(a), D::Bytes(b)) => match op {
                "+" => {
                    let mut out = a.clone();
                    out.try_reserve(b.len())
                        .map_err(|_| Halt::Error(GoSourceExecutionErrorKind::HostCapacity))?;
                    out.extend_from_slice(b);
                    D::Bytes(out)
                }
                "==" => D::Bool(a == b),
                "!=" => D::Bool(a != b),
                "<" => D::Bool(a < b),
                "<=" => D::Bool(a <= b),
                ">" => D::Bool(a > b),
                ">=" => D::Bool(a >= b),
                _ => return Err(source()),
            },
            (D::Word(a), D::Word(b)) => {
                let (w, signed) = self.word_shape(x.type_id())?;
                let aint = self.integer(&x)?;
                let bint = self.integer(&y)?;
                match op {
                    "==" => D::Bool(a == b),
                    "!=" => D::Bool(a != b),
                    "<" => D::Bool(aint < bint),
                    "<=" => D::Bool(aint <= bint),
                    ">" => D::Bool(aint > bint),
                    ">=" => D::Bool(aint >= bint),
                    _ => D::Word(
                        (match op {
                            "+" => a.wrapping_add(*b),
                            "-" => a.wrapping_sub(*b),
                            "*" => a.wrapping_mul(*b),
                            "&" => a & b,
                            "|" => a | b,
                            "^" => a ^ b,
                            "&^" => a & !b,
                            "/" | "%" => {
                                if *b == 0 {
                                    return Err(Halt::Fault(GoSourceRuntimeFault::DivideByZero));
                                }
                                if op == "/" {
                                    (aint / bint) as u64
                                } else {
                                    (aint % bint) as u64
                                }
                            }
                            "<<" | ">>" => {
                                if bint < 0 {
                                    return Err(Halt::Fault(GoSourceRuntimeFault::NegativeShift));
                                }
                                if bint >= w as i128 {
                                    if op == ">>" && signed && aint < 0 {
                                        mask(w)
                                    } else {
                                        0
                                    }
                                } else if op == "<<" {
                                    a << bint as u32
                                } else if signed {
                                    (aint >> bint as u32) as u64
                                } else {
                                    a >> bint as u32
                                }
                            }
                            _ => return Err(source()),
                        }) & mask(w),
                    ),
                }
            }
            (D::Nil, D::Nil) => D::Bool(op == "=="),
            (D::Slice(s), D::Nil) | (D::Nil, D::Slice(s)) => D::Bool(if op == "==" {
                s.is_nil()
            } else {
                !s.is_nil()
            }),
            (D::Error(s), D::Nil) | (D::Nil, D::Error(s)) => {
                D::Bool(if op == "==" { s.is_none() } else { s.is_some() })
            }
            (D::Slice(s), D::Slice(t)) if s.is_nil() || t.is_nil() => D::Bool(if op == "==" {
                s.is_nil() && t.is_nil()
            } else {
                !(s.is_nil() && t.is_nil())
            }),
            (D::Error(s), D::Error(t)) if s.is_none() || t.is_none() => D::Bool(if op == "==" {
                s.is_none() && t.is_none()
            } else {
                !(s.is_none() && t.is_none())
            }),
            (D::Aggregate(a), D::Aggregate(b)) => {
                let equal = a.len() == b.len() && a.iter().zip(b).all(|(x, y)| self.equal(x, y));
                D::Bool(if op == "==" {
                    equal
                } else if op == "!=" {
                    !equal
                } else {
                    return Err(source());
                })
            }
            _ => return Err(source()),
        };
        Ok(GoSourceRuntimeValue::new(typ, data))
    }
    fn equal(&self, x: &GoSourceRuntimeValue, y: &GoSourceRuntimeValue) -> bool {
        match (x.data(), y.data()) {
            (D::Bool(a), D::Bool(b)) => a == b,
            (D::Word(a), D::Word(b)) => a == b,
            (D::Bytes(a), D::Bytes(b)) => a == b,
            (D::Aggregate(a), D::Aggregate(b)) => {
                a.len()==b.len() && a.iter().zip(b).enumerate().all(|(i,(a,b))| {
                    matches!(self.shape(x.type_id()),Ok(GoSourceTypeKey::Struct{fields,..}) if fields.get(i).is_some_and(|field|field.name=="_")) || self.equal(a,b)
                })
            }
            _ => false,
        }
    }
    pub(super) fn field_path(&self, typ: &str, name: &str, src: &str) -> Run<Vec<usize>> {
        let mut level = vec![(typ.to_string(), vec![])];
        let package = self.engine.scopes.source_package(src).ok_or_else(source)?;
        while !level.is_empty() {
            let mut found = vec![];
            let mut next = vec![];
            for (id, path) in level {
                if let GoSourceTypeKey::Struct { fields, .. } = self.shape(&id)? {
                    for (i, field) in fields.iter().enumerate() {
                        let mut p = path.clone();
                        p.push(i);
                        if field.name == name
                            && field
                                .unexported_package_id
                                .as_deref()
                                .is_none_or(|v| v == package)
                        {
                            found.push(p.clone());
                        }
                        if field.embedded {
                            next.push((field.type_id.clone(), p));
                        }
                    }
                }
            }
            if found.len() == 1 {
                return Ok(found.remove(0));
            }
            if !found.is_empty() {
                return Err(source());
            }
            level = next;
        }
        Err(source())
    }
    pub(super) fn project(
        &self,
        mut v: GoSourceRuntimeValue,
        path: &[usize],
    ) -> Run<GoSourceRuntimeValue> {
        for &i in path {
            let D::Aggregate(xs) = v.data() else {
                return Err(source());
            };
            v = xs
                .get(i)
                .cloned()
                .ok_or(Halt::Fault(GoSourceRuntimeFault::IndexBounds))?;
        }
        Ok(v)
    }
    fn call(
        &mut self,
        f: &mut Frame,
        n: usize,
        node: &GoSyntaxNode,
        typ: Option<&str>,
    ) -> Run<Vec<GoSourceRuntimeValue>> {
        let owned_source = f.root.source_object_id().to_string();
        let src = owned_source.as_str();
        let fun = child(node, "Fun").map_err(|_| source())?;
        let args = list(node, "Args");
        if let Ok(GoNameLookup::Predeclared {
            name,
            kind: GoSourceNameKind::BuiltinFunction,
        }) = self.engine.symbol(src, fun)
        {
            return self.builtin_call(f, n, node, &name, typ);
        }
        let ordinary = self
            .domain
            .selection()
            .calls()
            .iter()
            .find(|c| c.caller() == &f.root && c.call_node() == n)
            .map(|c| c.callee().clone());
        if let Some(target) = ordinary {
            let header = self
                .engine
                .calls
                .iter()
                .find(|c| c.call_source_object_id() == src && c.call_node() == n)
                .cloned();
            if let Some(header) = header {
                let values = self.arguments(f, args, header.parameter_type_ids())?;
                return self.invoke(target, values);
            }
        }
        if let Some(header) = self
            .engine
            .method_calls
            .iter()
            .find(|c| c.call_source_object_id() == src && c.call_node() == n)
            .cloned()
        {
            let mut values = if header.method_expression() {
                self.arguments(f, args, header.parameter_type_ids())?
            } else {
                let receiver = self.one(
                    f,
                    header.receiver_node(),
                    Some(header.selected_receiver_type_id()),
                )?;
                let mut values = vec![receiver];
                values.extend(self.arguments(f, args, header.parameter_type_ids())?);
                values
            };
            let mut receiver = values.remove(0);
            let mut method = header.method().clone();
            if let D::Error(payload) = receiver.data() {
                self.at = GoSourceSelectionRef::new(src, n);
                receiver = payload
                    .as_ref()
                    .map(|p| p.as_ref().clone())
                    .ok_or(Halt::Fault(GoSourceRuntimeFault::NilInterfaceCall))?;
                self.at = GoSourceSelectionRef::new(src, n);
                method = self
                    .domain
                    .error_cases()
                    .iter()
                    .find(|c| c.value_type_id() == receiver.type_id())
                    .ok_or_else(source)?
                    .method()
                    .clone();
            }
            receiver = self.project(receiver, method.receiver_path())?;
            values.insert(0, self.assigned(receiver, method.receiver_type_id())?);
            return self.invoke(
                GoSourceSelectionRef::new(
                    method.declaration_source_object_id().ok_or_else(source)?,
                    method.declaration_node().ok_or_else(source)?,
                ),
                values,
            );
        }
        // The checked CallExpr has no selected callable: it is a conversion.
        if args.len() != 1 {
            return Err(source());
        }
        let target = typ.ok_or_else(source)?;
        let value = self.one(f, args[0], None)?;
        Ok(vec![self.convert(value, target)?])
    }
    pub(super) fn arguments(
        &mut self,
        f: &mut Frame,
        args: &[usize],
        types: &[String],
    ) -> Run<Vec<GoSourceRuntimeValue>> {
        if args.len() == 1 && types.len() != 1 {
            let values = self.expression(f, args[0], None)?;
            if values.len() != types.len() {
                return Err(source());
            }
            return values
                .into_iter()
                .zip(types)
                .map(|(v, t)| self.assigned(v, t))
                .collect();
        }
        if args.len() != types.len() {
            return Err(source());
        }
        args.iter()
            .zip(types)
            .map(|(&n, t)| self.one(f, n, Some(t)))
            .collect()
    }
    fn convert(&self, value: GoSourceRuntimeValue, target: &str) -> Run<GoSourceRuntimeValue> {
        if matches!(value.data(), D::Nil) {
            return self.assigned(value, target);
        }
        let data = match (self.shape(target)?, value.data()) {
            (GoSourceTypeKey::Builtin { width: Some(w), .. }, D::Word(_)) => {
                D::Word((self.integer(&value)? as u64) & mask(*w))
            }
            (GoSourceTypeKey::Builtin { name, .. }, D::Word(_)) if name == "string" => {
                let n = self.integer(&value)?;
                let c = u32::try_from(n)
                    .ok()
                    .and_then(char::from_u32)
                    .unwrap_or('\u{fffd}');
                let mut b = [0; 4];
                D::Bytes(c.encode_utf8(&mut b).as_bytes().to_vec())
            }
            (GoSourceTypeKey::Slice { element, .. }, D::Bytes(bytes)) => {
                let rune = matches!(
                    self.shape(element)?,
                    GoSourceTypeKey::Builtin {
                        width: Some(32),
                        signed: Some(true),
                        ..
                    }
                );
                let numbers = if rune {
                    decode_runes(bytes)
                        .into_iter()
                        .map(|n| n as u64)
                        .collect::<Vec<_>>()
                } else {
                    bytes.iter().map(|n| *n as u64).collect()
                };
                let xs = numbers
                    .into_iter()
                    .map(|n| GoSourceRuntimeValue::new(element, D::Word(n)))
                    .collect::<Vec<_>>();
                let len = xs.len();
                D::Slice(GoSourceRuntimeSlice::from_backing(xs, len).ok_or_else(source)?)
            }
            (GoSourceTypeKey::Builtin { name, .. }, D::Slice(slice)) if name == "string" => {
                let GoSourceTypeKey::Slice { element, .. } = self.shape(value.type_id())? else {
                    return Err(source());
                };
                let rune = matches!(
                    self.shape(element)?,
                    GoSourceTypeKey::Builtin {
                        width: Some(32),
                        signed: Some(true),
                        ..
                    }
                );
                let mut bytes = vec![];
                for x in slice.elements() {
                    let n = self.integer(&x)?;
                    if rune {
                        let c = u32::try_from(n)
                            .ok()
                            .and_then(char::from_u32)
                            .unwrap_or('\u{fffd}');
                        let mut b = [0; 4];
                        bytes.extend_from_slice(c.encode_utf8(&mut b).as_bytes());
                    } else {
                        bytes.push(n as u8);
                    }
                }
                D::Bytes(bytes)
            }
            _ => return self.assigned(value, target),
        };
        Ok(GoSourceRuntimeValue::new(target, data))
    }
    fn literal(
        &mut self,
        f: &mut Frame,
        node: &GoSyntaxNode,
        typ: &str,
    ) -> Run<GoSourceRuntimeValue> {
        let shape = self.shape(typ)?.clone();
        let mut values = match &shape {
            GoSourceTypeKey::Slice { element, .. } => {
                let indices = self
                    .engine
                    .literal_indices(f.root.source_object_id(), self.at.node())
                    .map_err(|_| source())?;
                let len = indices.iter().map(|(i, _)| i + 1).max().unwrap_or(0);
                let mut values = vec![];
                values
                    .try_reserve_exact(
                        usize::try_from(len)
                            .map_err(|_| Halt::Error(GoSourceExecutionErrorKind::HostCapacity))?,
                    )
                    .map_err(|_| Halt::Error(GoSourceExecutionErrorKind::HostCapacity))?;
                for _ in 0..len {
                    values.push(self.zero(element)?);
                }
                values
            }
            _ => {
                let z = self.zero(typ)?;
                let D::Aggregate(xs) = z.data() else {
                    return Err(source());
                };
                xs.clone()
            }
        };
        let mut next = 0usize;
        for (ordinal, &n) in list(node, "Elts").iter().enumerate() {
            let e = self.node(f, n)?;
            let (index, value) = if e.kind() == "KeyValueExpr" {
                let key = child(&e, "Key").map_err(|_| source())?;
                let value = child(&e, "Value").map_err(|_| source())?;
                let index = if let GoSourceTypeKey::Struct { fields, .. } = &shape {
                    let key = self.node(f, key)?;
                    fields
                        .iter()
                        .position(|f| Some(f.name.as_str()) == attr(&key, "Name"))
                        .ok_or_else(source)?
                } else {
                    let c = self
                        .engine
                        .exact_constant(f.root.source_object_id(), key)
                        .map_err(|_| source())?;
                    c.integer_decimal()
                        .ok_or_else(source)?
                        .parse()
                        .map_err(|_| source())?
                };
                (index, value)
            } else {
                (
                    if matches!(shape, GoSourceTypeKey::Struct { .. }) {
                        ordinal
                    } else {
                        next
                    },
                    n,
                )
            };
            next = index + 1;
            let target = values.get(index).ok_or_else(source)?.type_id().to_string();
            let v = self.one(f, value, Some(&target))?;
            values[index] = v;
        }
        let data = if matches!(shape, GoSourceTypeKey::Slice { .. }) {
            let len = values.len();
            D::Slice(GoSourceRuntimeSlice::from_backing(values, len).ok_or_else(source)?)
        } else {
            D::Aggregate(values)
        };
        Ok(GoSourceRuntimeValue::new(typ, data))
    }
    fn builtin_call(
        &mut self,
        f: &mut Frame,
        n: usize,
        node: &GoSyntaxNode,
        name: &str,
        typ: Option<&str>,
    ) -> Run<Vec<GoSourceRuntimeValue>> {
        let args = list(node, "Args");
        match name {
            "len" | "cap" => {
                let x = self.one(f, *args.first().ok_or_else(source)?, None)?;
                let n = match x.data() {
                    D::Bytes(b) => b.len(),
                    D::Aggregate(a) => a.len(),
                    D::Slice(s) => {
                        if name == "len" {
                            s.length()
                        } else {
                            s.capacity()
                        }
                    }
                    _ => return Err(source()),
                };
                Ok(vec![GoSourceRuntimeValue::new(
                    typ.ok_or_else(source)?,
                    D::Word(n as u64),
                )])
            }
            "min" | "max" => {
                let t = typ.ok_or_else(source)?;
                let mut xs = args
                    .iter()
                    .map(|&n| self.one(f, n, Some(t)))
                    .collect::<Run<Vec<_>>>()?;
                let mut v = xs.remove(0);
                for x in xs {
                    let b = self.binary(
                        if name == "min" { "<" } else { ">" },
                        x.clone(),
                        v.clone(),
                        &self.builtin("bool")?,
                    )?;
                    if b.data() == &D::Bool(true) {
                        v = x;
                    }
                }
                Ok(vec![v])
            }
            "make" => {
                let t = typ.ok_or_else(source)?;
                let GoSourceTypeKey::Slice { element, .. } = self.shape(t)?.clone() else {
                    return Err(unavailable());
                };
                let mut sizes = vec![];
                for &arg in args.iter().skip(1) {
                    let v = self.one(f, arg, None)?;
                    self.at = GoSourceSelectionRef::new(f.root.source_object_id(), n);
                    sizes.push(self.index(&v, GoSourceRuntimeFault::MakeBounds)?);
                }
                let len = *sizes.first().ok_or_else(source)?;
                let cap = *sizes.get(1).unwrap_or(&len);
                if cap < len {
                    return Err(Halt::Fault(GoSourceRuntimeFault::MakeBounds));
                }
                let mut xs = vec![];
                xs.try_reserve_exact(cap)
                    .map_err(|_| Halt::Error(GoSourceExecutionErrorKind::HostCapacity))?;
                for _ in 0..cap {
                    xs.push(self.zero(&element)?);
                }
                Ok(vec![GoSourceRuntimeValue::new(
                    t,
                    D::Slice(GoSourceRuntimeSlice::from_backing(xs, len).ok_or_else(source)?),
                )])
            }
            "append" => {
                let t = typ.ok_or_else(source)?;
                let x = self.one(f, args[0], Some(t))?;
                let D::Slice(slice) = x.data() else {
                    return Err(source());
                };
                let GoSourceTypeKey::Slice { element, .. } = self.shape(t)?.clone() else {
                    return Err(source());
                };
                let mut extra = vec![];
                if attr(node, "Ellipsis") == Some("true") {
                    let x = self.one(f, args[1], None)?;
                    match x.data() {
                        D::Slice(s) => extra = s.elements(),
                        D::Bytes(b) => {
                            extra = b
                                .iter()
                                .map(|n| GoSourceRuntimeValue::new(&element, D::Word(*n as u64)))
                                .collect()
                        }
                        _ => return Err(source()),
                    }
                } else {
                    for &a in &args[1..] {
                        extra.push(self.one(f, a, Some(&element))?);
                    }
                }
                let out = slice.append(extra, &self.zero(&element)?)?;
                Ok(vec![GoSourceRuntimeValue::new(t, D::Slice(out))])
            }
            "copy" => {
                let dst = self.one(f, args[0], None)?;
                let src = self.one(f, args[1], None)?;
                let D::Slice(dst) = dst.data() else {
                    return Err(source());
                };
                let xs = match src.data() {
                    D::Slice(s) => s.elements(),
                    D::Bytes(b) => {
                        let dsttype = self.fact(f, args[0], None)?.typ.ok_or_else(source)?;
                        let GoSourceTypeKey::Slice { element, .. } = self.shape(&dsttype)?.clone()
                        else {
                            return Err(source());
                        };
                        b.iter()
                            .map(|n| GoSourceRuntimeValue::new(&element, D::Word(*n as u64)))
                            .collect()
                    }
                    _ => return Err(source()),
                };
                let count = dst.length().min(xs.len());
                for (i, v) in xs.into_iter().take(count).enumerate() {
                    dst.write(i, v)?;
                }
                Ok(vec![GoSourceRuntimeValue::new(
                    typ.ok_or_else(source)?,
                    D::Word(count as u64),
                )])
            }
            _ => Err(unavailable()),
        }
    }
}
fn decode_runes(mut bytes: &[u8]) -> Vec<u32> {
    let mut out = vec![];
    while !bytes.is_empty() {
        match std::str::from_utf8(bytes) {
            Ok(s) => {
                out.extend(s.chars().map(|c| c as u32));
                break;
            }
            Err(e) => {
                let valid = e.valid_up_to();
                if valid > 0 {
                    out.extend(
                        std::str::from_utf8(&bytes[..valid])
                            .expect("validated prefix")
                            .chars()
                            .map(|c| c as u32),
                    );
                    bytes = &bytes[valid..];
                }
                out.push(0xfffd);
                bytes = &bytes[1..];
            }
        }
    }
    out
}
