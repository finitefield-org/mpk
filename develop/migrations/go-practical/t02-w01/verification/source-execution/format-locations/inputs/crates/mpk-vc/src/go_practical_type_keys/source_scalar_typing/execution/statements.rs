use super::*;
use GoSourceRuntimeData as D;
#[derive(Clone)]
pub(super) enum Place {
    Blank,
    Local(Binding, Vec<usize>),
    Slice(GoSourceRuntimeSlice, usize, Vec<usize>),
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
                let base=child(&node,"X").map_err(|_|source())?;
                let typ=self.fact(f,base,None)?.typ.ok_or_else(source)?;
                let sel=self.node(f,child(&node,"Sel").map_err(|_|source())?)?;
                let path=self.field_path(&typ,attr(&sel,"Name").ok_or_else(source)?,f.root.source_object_id())?;
                self.extend_place(p,&path)
            }
            "IndexExpr"=>{
                let base=child(&node,"X").map_err(|_|source())?;
                let typ=self.fact(f,base,None)?.typ.ok_or_else(source)?;
                let slice=matches!(self.shape(&typ)?,GoSourceTypeKey::Slice{..});
                let mut place=if slice {let v=self.one(f,base,None)?;let D::Slice(s)=v.data()else{return Err(source());};Place::Slice(s.clone(),0,vec![])}else{self.place(f,base)?};
                let index=self.one(f,child(&node,"Index").map_err(|_|source())?,None)?;
                let i=usize::try_from(self.integer(&index)?).unwrap_or(usize::MAX);
                // Final location bounds are checked only in assignment phase
                // two. Reading an intermediate slice descriptor still executes
                // its own source access in phase one.
                if slice {let Place::Slice(_,index,_)=&mut place else{return Err(source());};*index=i;Ok(place)}else{self.extend_place(place,&[i])}
            }
            _ => Err(unavailable()),
        }
    }
    fn extend_place(&self,p:Place,path:&[usize])->Run<Place>{match p {Place::Local(b,mut xs)=>{xs.extend_from_slice(path);Ok(Place::Local(b,xs))},Place::Slice(s,i,mut xs)=>{xs.extend_from_slice(path);Ok(Place::Slice(s,i,xs))},Place::Blank=>Err(source())}}
    fn read_place(&self, f: &Frame, p: &Place) -> Run<GoSourceRuntimeValue> {
        match p {
            Place::Local(b, path) => {
                let value = f.get(b).ok_or_else(source)?;
                self.project(value, path)
            }
            Place::Slice(s,i,path)=>self.project(s.read(*i).ok_or(Halt::Fault(GoSourceRuntimeFault::IndexBounds))?,path),
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
            Place::Slice(s,i,path)=>{if path.is_empty(){s.write(i,v)}else{let old=s.read(i).ok_or(Halt::Fault(GoSourceRuntimeFault::IndexBounds))?;s.write(i,self.updated(old,&path,v)?)}}
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
            Place::Slice(s,i,tail)=>{let(cell,mut path)=s.array_cell(i)?;path.extend(tail);GoSourceRuntimeSlice::view(cell,path,length)}
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
                    // Go1.25 creates fresh for-init variables for the next
                    // iteration before executing its post statement.
                    if let Some(&init)=list(&node,"Init").first(){let initnode=self.node(f,init)?;if attr(&initnode,"Tok")==Some(":="){for &name in list(&initnode,"Lhs"){if let Some(b)=self.named_binding(f,init,name){let key=binding(&b);let value=f.get(&key).ok_or_else(source)?;f.fresh(key,value);}}}}
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
        let request=Request::declaration(f.root.source_object_id(),decl);
        self.engine.compute(request.clone()).map_err(|_|source())?;
        let facts=self.engine.facts(&request).map_err(|_|source())?;
        if facts.len()!=names.len(){return Err(source());}
        let types=facts.iter().map(|a|a.typ.clone()).collect::<Vec<_>>();
        let mut bindings=vec![];
        for &name in names {
            if attr(&self.node(f,name)?,"Name")==Some("_"){bindings.push(None);continue;}
            let (b,new)=if let Some(b)=self.named_binding(f,decl,name){(b,true)}else{let GoNameLookup::Original(b)=self.engine.symbol(f.root.source_object_id(),name).map_err(|_|source())?else{return Err(source());};(b,false)};
            bindings.push(Some((b,new)));
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
            if let Some((b,new))=b {self.tick(f,node,GoSourceExecutionEventKind::Write)?;if new {f.fresh(binding(&b),value);}else{f.put(binding(&b),value);}}
        }
        Ok(())
    }
}
