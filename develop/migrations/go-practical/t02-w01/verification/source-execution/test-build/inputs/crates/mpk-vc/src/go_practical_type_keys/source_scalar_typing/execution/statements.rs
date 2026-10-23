use super::*;
use GoSourceRuntimeData as D;
#[derive(Clone)]
pub(super) enum Place {
    Blank,
    Local(Binding, Vec<usize>),
    Slice(GoSourceRuntimeSlice, usize),
}
impl Machine<'_> {
    fn named_binding(&self, f: &Frame, decl: usize, name: usize) -> Option<GoSourceNameBinding> {
        self.engine
            .scopes
            .bindings()
            .iter()
            .find(|b| {
                b.source_object_id() == f.root.source_object_id()
                    && b.declaration_node() == decl
                    && b.name_node() == name
                    && b.kind() == GoSourceNameKind::Variable
            })
            .cloned()
    }
    fn binding_type(&self, f: &Frame, b: &GoSourceNameBinding) -> Run<String> {
        self.domain
            .selection()
            .checked_local_type(&f.root, b)
            .map(str::to_string)
            .or_else(|| f.get(&binding(b)).map(|v| v.type_id().to_string()))
            .ok_or_else(source)
    }
    pub(super) fn place(&mut self, f: &mut Frame, n: usize) -> Run<Place> {
        let node = self.node(f, n)?;
        match node.kind() {
            "ParenExpr" => self.place(f, child(&node, "X").map_err(|_| source())?),
            "Ident" => {
                if attr(&node, "Name") == Some("_") {
                    return Ok(Place::Blank);
                }
                let GoNameLookup::Original(b) = self
                    .engine
                    .symbol(f.root.source_object_id(), n)
                    .map_err(|_| source())?
                else {
                    return Err(source());
                };
                Ok(Place::Local(binding(&b), vec![]))
            }
            "SelectorExpr" => {
                let p = self.place(f, child(&node, "X").map_err(|_| source())?)?;
                let value = self.read_place(f, &p)?;
                let sel = self.node(f, child(&node, "Sel").map_err(|_| source())?)?;
                let path = self.field_path(
                    value.type_id(),
                    attr(&sel, "Name").ok_or_else(source)?,
                    f.root.source_object_id(),
                )?;
                match p {
                    Place::Local(b, mut prefix) => {
                        prefix.extend(path);
                        Ok(Place::Local(b, prefix))
                    }
                    _ => Err(unavailable()),
                }
            }
            "IndexExpr" => {
                let base = child(&node, "X").map_err(|_| source())?;
                let a = self.fact(f, base, None)?;
                let slice = matches!(
                    self.shape(a.typ.as_deref().ok_or_else(source)?)?,
                    GoSourceTypeKey::Slice { .. }
                );
                let (value, place) = if slice {
                    (self.one(f, base, None)?, None)
                } else {
                    let p = self.place(f, base)?;
                    (self.read_place(f, &p)?, Some(p))
                };
                let index = self.one(f, child(&node, "Index").map_err(|_| source())?, None)?;
                // Capture the index now; the final destination's bounds are
                // checked when it is written in assignment phase two.
                let integer = self.integer(&index)?;
                let i = usize::try_from(integer).unwrap_or(usize::MAX);
                if let D::Slice(s) = value.data() {
                    return Ok(Place::Slice(s.clone(), i));
                }
                match place {
                    Some(Place::Local(b, mut path)) => {
                        path.push(i);
                        Ok(Place::Local(b, path))
                    }
                    _ => Err(source()),
                }
            }
            _ => Err(unavailable()),
        }
    }
    fn read_place(&self, f: &Frame, p: &Place) -> Run<GoSourceRuntimeValue> {
        match p {
            Place::Local(b, path) => {
                let value = f.get(b).ok_or_else(source)?;
                self.project(value, path)
            }
            Place::Slice(s, i) => s
                .read(*i)
                .ok_or(Halt::Fault(GoSourceRuntimeFault::IndexBounds)),
            Place::Blank => Err(source()),
        }
    }
    fn write_place(
        &mut self,
        f: &mut Frame,
        node: usize,
        p: Place,
        v: GoSourceRuntimeValue,
    ) -> Run<()> {
        self.tick(f, node, GoSourceExecutionEventKind::Write)?;
        match p {
            Place::Blank => Ok(()),
            Place::Slice(s, i) => s.write(i, v),
            Place::Local(b, path) => {
                let old = f.get(&b).ok_or_else(source)?;
                let new = self.updated(old, &path, v)?;
                f.put(b, new);
                Ok(())
            }
        }
    }
    fn updated(
        &self,
        old: GoSourceRuntimeValue,
        path: &[usize],
        value: GoSourceRuntimeValue,
    ) -> Run<GoSourceRuntimeValue> {
        let Some((&i, rest)) = path.split_first() else {
            return self.assigned(value, old.type_id());
        };
        let D::Aggregate(xs) = old.data() else {
            return Err(source());
        };
        let mut xs = xs.clone();
        let previous = xs
            .get(i)
            .cloned()
            .ok_or(Halt::Fault(GoSourceRuntimeFault::IndexBounds))?;
        xs[i] = self.updated(previous, rest, value)?;
        Ok(GoSourceRuntimeValue::new(old.type_id(), D::Aggregate(xs)))
    }
    pub(super) fn array_view(
        &self,
        f: &Frame,
        p: Place,
        length: usize,
    ) -> Run<GoSourceRuntimeSlice> {
        match p {
            Place::Local(b, path) => GoSourceRuntimeSlice::view(
                f.locals.get(&b).cloned().ok_or_else(source)?,
                path,
                length,
            ),
            Place::Slice(s, i) => {
                let (cell, path) = s.array_cell(i)?;
                GoSourceRuntimeSlice::view(cell, path, length)
            }
            Place::Blank => Err(source()),
        }
    }
    pub(super) fn statement(&mut self, f: &mut Frame, n: usize) -> Run<Flow> {
        self.tick(f, n, GoSourceExecutionEventKind::Statement)?;
        let node = self.node(f, n)?;
        match node.kind() {
            "BlockStmt" => {
                for &n in list(&node, "List") {
                    match self.statement(f, n)? {
                        Flow::Normal => {}
                        flow => return Ok(flow),
                    }
                }
                Ok(Flow::Normal)
            }
            "ReturnStmt" => {
                let types = f.results.iter().map(|(t, _)| t.clone()).collect::<Vec<_>>();
                let args = list(&node, "Results");
                let values = if args.is_empty() {
                    f.results
                        .iter()
                        .map(|(t, b)| match b {
                            Some(b) => f.get(b).ok_or_else(source),
                            None => self.zero(t),
                        })
                        .collect::<Run<Vec<_>>>()?
                } else {
                    self.arguments(f, args, &types)?
                };
                Ok(Flow::Return(values))
            }
            "DeclStmt" => {
                let decl = child(&node, "Decl").map_err(|_| source())?;
                let d = self.node(f, decl)?;
                if attr(&d, "Tok") == Some("var") {
                    for &spec in list(&d, "Specs") {
                        let row = self.node(f, spec)?;
                        self.declare(f, spec, list(&row, "Names"), list(&row, "Values"))?;
                    }
                }
                Ok(Flow::Normal)
            }
            "AssignStmt" if attr(&node, "Tok") == Some(":=") => {
                self.declare(f, n, list(&node, "Lhs"), list(&node, "Rhs"))?;
                Ok(Flow::Normal)
            }
            "AssignStmt" | "IncDecStmt" => {
                let left = if node.kind() == "IncDecStmt" {
                    list(&node, "X")
                } else {
                    list(&node, "Lhs")
                };
                let mut places = vec![];
                for &n in left {
                    places.push(self.place(f, n)?);
                }
                let op = attr(&node, "Tok").ok_or_else(source)?;
                let values = if op == "=" {
                    let types = left
                        .iter()
                        .zip(&places)
                        .map(|(&target, p)| match p {
                            Place::Blank => Ok(None),
                            _ => Ok(Some(self.fact(f, target, None)?.typ.ok_or_else(source)?)),
                        })
                        .collect::<Run<Vec<_>>>()?;
                    let right = list(&node, "Rhs");
                    if right.len() == 1 && places.len() != 1 {
                        let values = self.expression(f, right[0], None)?;
                        if values.len() != places.len() {
                            return Err(source());
                        }
                        values
                            .into_iter()
                            .zip(types)
                            .map(|(v, t)| {
                                if let Some(t) = t {
                                    self.assigned(v, &t)
                                } else {
                                    Ok(v)
                                }
                            })
                            .collect::<Run<Vec<_>>>()?
                    } else {
                        if right.len() != places.len() {
                            return Err(source());
                        }
                        right
                            .iter()
                            .zip(types)
                            .map(|(&n, t)| self.one(f, n, t.as_deref()))
                            .collect::<Run<Vec<_>>>()?
                    }
                } else {
                    if places.len() != 1 {
                        return Err(source());
                    }
                    let old = self.read_place(f, &places[0])?;
                    let t = old.type_id().to_string();
                    let rhs = if matches!(op, "++" | "--") {
                        GoSourceRuntimeValue::new(&t, D::Word(1))
                    } else {
                        self.one(
                            f,
                            *list(&node, "Rhs").first().ok_or_else(source)?,
                            if matches!(op, "<<=" | ">>=") {
                                None
                            } else {
                                Some(&t)
                            },
                        )?
                    };
                    self.at = GoSourceSelectionRef::new(f.root.source_object_id(), n);
                    let operator = if op == "++" {
                        "+"
                    } else if op == "--" {
                        "-"
                    } else {
                        op.strip_suffix('=').ok_or_else(source)?
                    };
                    vec![self.binary(operator, old, rhs, &t)?]
                };
                for ((p, v), &target) in places.into_iter().zip(values).zip(left) {
                    self.write_place(f, target, p, v)?;
                }
                Ok(Flow::Normal)
            }
            "ExprStmt" => {
                self.expression(f, child(&node, "X").map_err(|_| source())?, None)?;
                Ok(Flow::Normal)
            }
            "IfStmt" => {
                if let Some(&init) = list(&node, "Init").first() {
                    if !matches!(self.statement(f, init)?, Flow::Normal) {
                        return Err(source());
                    }
                }
                let cond = self.one(f, child(&node, "Cond").map_err(|_| source())?, None)?;
                let D::Bool(b) = cond.data() else {
                    return Err(source());
                };
                if *b {
                    self.statement(f, child(&node, "Body").map_err(|_| source())?)
                } else if let Some(&n) = list(&node, "Else").first() {
                    self.statement(f, n)
                } else {
                    Ok(Flow::Normal)
                }
            }
            "ForStmt" => {
                if let Some(&init) = list(&node, "Init").first() {
                    if !matches!(self.statement(f, init)?, Flow::Normal) {
                        return Err(source());
                    }
                }
                loop {
                    self.tick(f, n, GoSourceExecutionEventKind::Statement)?;
                    if let Some(&cond) = list(&node, "Cond").first() {
                        let v = self.one(f, cond, None)?;
                        if v.data() == &D::Bool(false) {
                            break;
                        }
                        if v.data() != &D::Bool(true) {
                            return Err(source());
                        }
                    }
                    match self.statement(f, child(&node, "Body").map_err(|_| source())?)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Normal | Flow::Continue => {}
                    }
                    if let Some(&post) = list(&node, "Post").first() {
                        if !matches!(self.statement(f, post)?, Flow::Normal) {
                            return Err(source());
                        }
                    }
                }
                Ok(Flow::Normal)
            }
            "RangeStmt" => {
                let value = self.one(f, child(&node, "X").map_err(|_| source())?, None)?;
                let len = match value.data() {
                    D::Aggregate(xs) => xs.len(),
                    D::Slice(s) => s.length(),
                    _ => return Err(unavailable()),
                };
                let targets = [
                    list(&node, "Key").first().copied(),
                    list(&node, "Value").first().copied(),
                ];
                for i in 0..len {
                    self.tick(f, n, GoSourceExecutionEventKind::Statement)?;
                    for (position, target) in targets.iter().enumerate() {
                        if let Some(target) = target {
                            if attr(&self.node(f, *target)?, "Name") == Some("_") {
                                continue;
                            }
                            let v = if position == 0 {
                                GoSourceRuntimeValue::new(self.builtin("int")?, D::Word(i as u64))
                            } else {
                                match value.data() {
                                    D::Aggregate(xs) => xs[i].clone(),
                                    D::Slice(s) => s.read(i).ok_or_else(source)?,
                                    _ => return Err(source()),
                                }
                            };
                            if attr(&node, "Tok") == Some(":=") {
                                let b = self.named_binding(f, n, *target).ok_or_else(source)?;
                                f.fresh(binding(&b), v);
                            } else {
                                let p = self.place(f, *target)?;
                                self.write_place(f, *target, p, v)?;
                            }
                        }
                    }
                    match self.statement(f, child(&node, "Body").map_err(|_| source())?)? {
                        Flow::Break => break,
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Normal | Flow::Continue => {}
                    }
                }
                Ok(Flow::Normal)
            }
            "SwitchStmt" => {
                if let Some(&init) = list(&node, "Init").first() {
                    if !matches!(self.statement(f, init)?, Flow::Normal) {
                        return Err(source());
                    }
                }
                let tag = if let Some(&tag) = list(&node, "Tag").first() {
                    self.one(f, tag, None)?
                } else {
                    GoSourceRuntimeValue::new(self.builtin("bool")?, D::Bool(true))
                };
                let body = self.node(f, child(&node, "Body").map_err(|_| source())?)?;
                let mut chosen = None;
                let mut default = None;
                'cases: for &clause in list(&body, "List") {
                    let c = self.node(f, clause)?;
                    let cases = list(&c, "List");
                    if cases.is_empty() {
                        default = Some(clause);
                        continue;
                    }
                    for &case in cases {
                        let value = self.one(f, case, Some(tag.type_id()))?;
                        let equal =
                            self.binary("==", tag.clone(), value, &self.builtin("bool")?)?;
                        if equal.data() == &D::Bool(true) {
                            chosen = Some(clause);
                            break 'cases;
                        }
                    }
                }
                if let Some(c) = chosen.or(default) {
                    let c = self.node(f, c)?;
                    for &n in list(&c, "Body") {
                        match self.statement(f, n)? {
                            Flow::Break => return Ok(Flow::Normal),
                            Flow::Normal => {}
                            flow => return Ok(flow),
                        }
                    }
                }
                Ok(Flow::Normal)
            }
            "BranchStmt" => match attr(&node, "Tok") {
                Some("break") => Ok(Flow::Break),
                Some("continue") => Ok(Flow::Continue),
                _ => Err(unavailable()),
            },
            "EmptyStmt" => Ok(Flow::Normal),
            _ => Err(unavailable()),
        }
    }
    fn declare(&mut self, f: &mut Frame, decl: usize, names: &[usize], rhs: &[usize]) -> Run<()> {
        let mut bindings = vec![];
        let mut types = vec![];
        for &name in names {
            if attr(&self.node(f, name)?, "Name") == Some("_") {
                bindings.push(None);
                types.push(None);
                continue;
            }
            let b = if let Some(b) = self.named_binding(f, decl, name) {
                b
            } else {
                let GoNameLookup::Original(b) = self
                    .engine
                    .symbol(f.root.source_object_id(), name)
                    .map_err(|_| source())?
                else {
                    return Err(source());
                };
                b
            };
            types.push(Some(self.binding_type(f, &b)?));
            bindings.push(Some(b));
        }
        let values = if rhs.is_empty() {
            types
                .iter()
                .map(|t| self.zero(t.as_deref().ok_or_else(source)?))
                .collect::<Run<Vec<_>>>()?
        } else if rhs.len() == 1 && names.len() != 1 {
            let values = self.expression(f, rhs[0], None)?;
            if values.len() != names.len() {
                return Err(source());
            }
            values
                .into_iter()
                .zip(&types)
                .map(|(v, t)| {
                    if let Some(t) = t {
                        self.assigned(v, t)
                    } else {
                        Ok(v)
                    }
                })
                .collect::<Run<Vec<_>>>()?
        } else {
            if rhs.len() != names.len() {
                return Err(source());
            }
            rhs.iter()
                .zip(&types)
                .map(|(&n, t)| self.one(f, n, t.as_deref()))
                .collect::<Run<Vec<_>>>()?
        };
        for ((b, value), &node) in bindings.into_iter().zip(values).zip(names) {
            if let Some(b) = b {
                self.tick(f, node, GoSourceExecutionEventKind::Write)?;
                f.put(binding(&b), value);
            }
        }
        Ok(())
    }
}
