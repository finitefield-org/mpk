use super::*;
impl Parser {
    pub(super) fn type_start(&self) -> bool {
        matches!(
            self.kind(),
            "IDENT" | "[" | "struct" | "*" | "func" | "interface" | "map" | "chan" | "<-" | "("
        )
    }
    pub(super) fn typ(&mut self) -> Result<Id> {
        self.nested(|p| p.type_inner())
    }
    fn type_inner(&mut self) -> Result<Id> {
        let start = self.pos();
        match self.kind() {
            "IDENT" => {
                let x = self.ident()?;
                self.qualified(x)
            }
            "[" => {
                self.bump();
                self.array_after_open(start, None)
            }
            "*" => {
                self.bump();
                let x = self.typ()?;
                let n = self.node("StarExpr", start, self.e(x))?;
                self.edge(n, "X", x);
                Ok(n)
            }
            "(" => {
                self.bump();
                let x = self.typ()?;
                let end = self.close(")")?;
                let n = self.node("ParenExpr", start, end)?;
                self.edge(n, "X", x);
                Ok(n)
            }
            "struct" => self.structure(),
            "interface" => self.interface(),
            "func" => self.function_type(true),
            "map" => {
                self.bump();
                self.expect("[")?;
                let k = self.typ()?;
                self.expect("]")?;
                let value = self.typ()?;
                let n = self.node("MapType", start, self.e(value))?;
                self.edge(n, "Key", k);
                self.edge(n, "Value", value);
                Ok(n)
            }
            "chan" | "<-" => {
                let dir = if self.eat("<-").is_some() {
                    self.expect("chan")?;
                    2
                } else {
                    self.bump();
                    if self.eat("<-").is_some() {
                        1
                    } else {
                        3
                    }
                };
                let value = self.typ()?;
                let n = self.node("ChanType", start, self.e(value))?;
                self.edge(n, "Value", value);
                self.attr(n, "Dir", dir);
                Ok(n)
            }
            _ => self.err(),
        }
    }
    pub(super) fn qualified(&mut self, mut x: Id) -> Result<Id> {
        if self.eat(".").is_some() {
            let sel = self.ident()?;
            let n = self.node("SelectorExpr", self.s(x), self.e(sel))?;
            self.edge(n, "X", x);
            self.edge(n, "Sel", sel);
            x = n
        }
        if self.is("[") {
            self.bump();
            self.level += 1;
            let mut xs = vec![self.expression()?];
            while self.eat(",").is_some() {
                if self.is("]") {
                    break;
                }
                xs.push(self.expression()?)
            }
            let end = self.close("]")?;
            self.level -= 1;
            x = self.pack_index(x, xs, end)?
        }
        Ok(x)
    }
    pub(super) fn array_after_open(&mut self, start: usize, initial: Option<Id>) -> Result<Id> {
        self.level += 1;
        let length = if initial.is_some() {
            initial
        } else if self.eat("...").is_some() {
            let p = self.lexed.tokens()[self.at - 1].start();
            Some(self.node("Ellipsis", p, p + 3)?)
        } else if self.is("]") {
            None
        } else {
            Some(self.expression()?)
        };
        self.level -= 1;
        self.expect("]")?;
        let elt = self.typ()?;
        let n = self.node("ArrayType", start, self.e(elt))?;
        self.optional(n, "Len", length);
        self.edge(n, "Elt", elt);
        Ok(n)
    }
    pub(super) fn array_field(&mut self, x: Id) -> Result<(Option<Id>, Id)> {
        let start = self.expect("[")?;
        self.level += 1;
        let mut args = vec![];
        let mut trailing = false;
        if !self.is("]") {
            loop {
                args.push(self.expression()?);
                if self.eat(",").is_none() {
                    break;
                }
                if self.is("]") {
                    trailing = true;
                    break;
                }
            }
        }
        let end = self.close("]")?;
        self.level -= 1;
        if args.is_empty() || args.len() == 1 && self.type_start() {
            if trailing {
                return self.err();
            }
            let elt = self.typ()?;
            let n = self.node("ArrayType", start, self.e(elt))?;
            if let Some(length) = args.first() {
                self.edge(n, "Len", *length)
            }
            self.edge(n, "Elt", elt);
            Ok((Some(x), n))
        } else {
            Ok((None, self.pack_index(x, args, end)?))
        }
    }
    fn structure(&mut self) -> Result<Id> {
        let start = self.expect("struct")?;
        let opening = self.expect("{")?;
        let mut fields = vec![];
        while !self.is("}") {
            let mut names = vec![];
            let typ = if self.is("IDENT") {
                let name = self.ident()?;
                if matches!(self.kind(), "." | "STRING" | ";" | "}") {
                    self.qualified(name)?
                } else {
                    names.push(name);
                    while self.eat(",").is_some() {
                        names.push(self.ident()?)
                    }
                    if names.len() == 1 && self.is("[") {
                        let (name, typ) = self.array_field(name)?;
                        if name.is_none() {
                            names.clear()
                        }
                        typ
                    } else {
                        self.typ()?
                    }
                }
            } else if self.eat("*").is_some() {
                let p = self.lexed.tokens()[self.at - 1].start();
                let name = self.ident()?;
                let x = self.qualified(name)?;
                let n = self.node("StarExpr", p, self.e(x))?;
                self.edge(n, "X", x);
                n
            } else {
                return self.err();
            };
            let tag = if self.is("STRING") {
                Some(self.literal()?)
            } else {
                None
            };
            let first = names.first().copied().unwrap_or(typ);
            let n = self.node(
                "Field",
                self.s(first),
                tag.map_or(self.e(typ), |t| self.e(t)),
            )?;
            self.list(n, "Names", names);
            self.edge(n, "Type", typ);
            self.optional(n, "Tag", tag);
            fields.push(n);
            self.semi()?;
        }
        let end = self.close("}")?;
        let list = self.node("FieldList", opening, end)?;
        self.list(list, "List", fields);
        let n = self.node("StructType", start, end)?;
        self.edge(n, "Fields", list);
        self.attr(n, "Incomplete", false);
        Ok(n)
    }
    fn dots_type(&mut self) -> Result<Id> {
        let start = self.expect("...")?;
        let typ = self.typ()?;
        let n = self.node("Ellipsis", start, self.e(typ))?;
        self.edge(n, "Elt", typ);
        Ok(n)
    }
    fn union(&mut self, initial: Option<Id>) -> Result<Id> {
        let term = |p: &mut Self| -> Result<Id> {
            if p.eat("~").is_some() {
                let start = p.lexed.tokens()[p.at - 1].start();
                let x = p.typ()?;
                let n = p.node("UnaryExpr", start, p.e(x))?;
                p.attr(n, "Op", "~");
                p.edge(n, "X", x);
                Ok(n)
            } else {
                p.typ()
            }
        };
        let mut x = if let Some(x) = initial {
            x
        } else {
            term(self)?
        };
        while self.eat("|").is_some() {
            let y = term(self)?;
            let n = self.node("BinaryExpr", self.s(x), self.e(y))?;
            self.attr(n, "Op", "|");
            self.edge(n, "X", x);
            self.edge(n, "Y", y);
            x = n
        }
        Ok(x)
    }
    fn param(&mut self, initial: Option<Id>, sets: bool) -> Result<(Option<Id>, Option<Id>)> {
        let mut name = None;
        let mut typ = None;
        if initial.is_some() || self.is("IDENT") {
            let x = if let Some(x) = initial {
                x
            } else {
                self.ident()?
            };
            name = Some(x);
            match self.kind() {
                "." => {
                    typ = Some(self.qualified(x)?);
                    name = None
                }
                "[" => {
                    let pair = self.array_field(x)?;
                    name = pair.0;
                    typ = Some(pair.1)
                }
                "..." => typ = Some(self.dots_type()?),
                "~" if sets => typ = Some(self.union(None)?),
                "|" if sets => {
                    typ = Some(self.union(Some(x))?);
                    name = None
                }
                _ if self.type_start() => typ = Some(self.typ()?),
                _ => {}
            }
        } else if self.is("...") {
            typ = Some(self.dots_type()?)
        } else if sets && self.is("~") {
            typ = Some(self.union(None)?)
        } else {
            typ = Some(self.typ()?)
        }
        if sets && self.is("|") && typ.is_some() {
            typ = Some(self.union(typ)?)
        }
        Ok((name, typ))
    }
    pub(super) fn params(
        &mut self,
        opening: usize,
        closing: &str,
        sets: bool,
        variadic: bool,
        initial: Option<(Id, Option<Id>)>,
    ) -> Result<Id> {
        let mut rows = vec![];
        let mut first = initial;
        while first.is_some() || !self.is(closing) {
            let row = if let Some((name, typ)) = first.take() {
                if let Some(typ) = typ {
                    (Some(name), Some(self.union(Some(typ))?))
                } else {
                    self.param(Some(name), sets)?
                }
            } else {
                self.param(None, sets)?
            };
            rows.push(row);
            if self.eat(",").is_none() {
                break;
            }
        }
        let end = self.close(closing)?;
        let named = rows
            .iter()
            .filter(|r| r.0.is_some() && r.1.is_some())
            .count();
        if sets && named == 0 {
            return self.err();
        }
        if named == 0 {
            for row in &mut rows {
                if row.1.is_none() {
                    row.1 = row.0.take()
                }
            }
        } else {
            let mut current = None;
            for row in rows.iter_mut().rev() {
                if row.1.is_some() {
                    current = row.1
                } else {
                    row.1 = current
                }
                if row.0.is_none() || row.1.is_none() {
                    return self.err();
                }
            }
        }
        for (i, (_, typ)) in rows.iter().enumerate() {
            if typ.is_some_and(|t| self.nk(t) == "Ellipsis") && (!variadic || i + 1 < rows.len()) {
                return self.err();
            }
        }
        let mut fields = vec![];
        let mut i = 0;
        while i < rows.len() {
            let (name, typ) = rows[i];
            let typ = typ.ok_or(GoParseError {
                offset: self.pos(),
                kind: GoParseErrorKind::Syntax,
            })?;
            let mut names = vec![];
            if let Some(name) = name {
                names.push(name)
            };
            i += 1;
            if named > 0 {
                while i < rows.len() && rows[i].1 == Some(typ) {
                    names.push(rows[i].0.unwrap());
                    i += 1
                }
            }
            let start = names.first().map_or(self.s(typ), |n| self.s(*n));
            let field = self.node("Field", start, self.e(typ))?;
            self.list(field, "Names", names);
            self.edge(field, "Type", typ);
            fields.push(field);
        }
        let n = self.node("FieldList", opening, end)?;
        self.list(n, "List", fields);
        Ok(n)
    }
    pub(super) fn signature(&mut self, start: usize, type_params: Option<Id>) -> Result<Id> {
        let open = self.expect("(")?;
        let params = self.params(open, ")", false, true, None)?;
        let results = if self.is("(") {
            let open = self.bump();
            Some(self.params(open, ")", false, false, None)?)
        } else if self.type_start() {
            let typ = self.typ()?;
            let field = self.node("Field", self.s(typ), self.e(typ))?;
            self.edge(field, "Type", typ);
            let list = self.node("FieldList", self.s(typ), self.e(typ))?;
            self.edge(list, "List", field);
            Some(list)
        } else {
            None
        };
        let n = self.node(
            "FuncType",
            start,
            results.map_or(self.e(params), |r| self.e(r)),
        )?;
        self.optional(n, "TypeParams", type_params);
        self.edge(n, "Params", params);
        self.optional(n, "Results", results);
        Ok(n)
    }
    pub(super) fn function_type(&mut self, keyword: bool) -> Result<Id> {
        let start = if keyword {
            self.expect("func")?
        } else {
            self.pos()
        };
        self.signature(start, None)
    }
    fn interface(&mut self) -> Result<Id> {
        let start = self.expect("interface")?;
        let opening = self.expect("{")?;
        let mut methods = vec![];
        while !self.is("}") {
            let (name, typ) = if self.is("IDENT") {
                let ident = self.ident()?;
                if self.is("(") {
                    (Some(ident), self.function_type(false)?)
                } else {
                    let t = self.qualified(ident)?;
                    (None, self.union(Some(t))?)
                }
            } else {
                (None, self.union(None)?)
            };
            let field = self.node(
                "Field",
                name.map_or(self.s(typ), |n| self.s(n)),
                self.e(typ),
            )?;
            self.optional(field, "Names", name);
            self.edge(field, "Type", typ);
            methods.push(field);
            self.semi()?;
        }
        let end = self.close("}")?;
        let list = self.node("FieldList", opening, end)?;
        self.list(list, "List", methods);
        let n = self.node("InterfaceType", start, end)?;
        self.edge(n, "Methods", list);
        self.attr(n, "Incomplete", false);
        Ok(n)
    }
    pub(super) fn type_element(&self, x: Id) -> bool {
        match self.nk(x) {
            "ArrayType" | "StructType" | "FuncType" | "InterfaceType" | "MapType" | "ChanType" => {
                true
            }
            "UnaryExpr" => self.nodes[x].attrs.get("Op").is_some_and(|o| o == "~"),
            "BinaryExpr" => {
                self.type_element(self.child(x, "X").unwrap())
                    || self.type_element(self.child(x, "Y").unwrap())
            }
            "ParenExpr" => self.type_element(self.child(x, "X").unwrap()),
            _ => false,
        }
    }
    pub(super) fn extract_name(&mut self, x: Id, force: bool) -> Result<(Option<Id>, Option<Id>)> {
        if self.nk(x) == "Ident" {
            return Ok((Some(x), None));
        }
        if self.nk(x) == "BinaryExpr" {
            let left = self.child(x, "X").unwrap();
            let right = self.child(x, "Y").unwrap();
            let op = self.nodes[x].attrs["Op"].clone();
            if op == "*" && self.nk(left) == "Ident" && (force || self.type_element(right)) {
                let star = self
                    .lexed
                    .tokens()
                    .iter()
                    .find(|t| {
                        t.start() >= self.e(left) && t.end() <= self.s(right) && t.kind() == "*"
                    })
                    .ok_or(GoParseError {
                        offset: self.s(x),
                        kind: GoParseErrorKind::Syntax,
                    })?
                    .start();
                let n = self.node("StarExpr", star, self.e(right))?;
                self.edge(n, "X", right);
                return Ok((Some(left), Some(n)));
            }
            if op == "|" {
                let (name, typ) = self.extract_name(left, force || self.type_element(right))?;
                if let (Some(name),Some(typ)) = (name,typ) {
                    let n = self.node("BinaryExpr", self.s(typ), self.e(right))?;
                    self.edge(n, "X", typ);
                    self.edge(n, "Y", right);
                    self.attr(n, "Op", "|");
                    return Ok((Some(name), Some(n)));
                }
            }
        }
        if self.nk(x) == "CallExpr" {
            let fun = self.child(x, "Fun").unwrap();
            let args = self.children(x, "Args");
            if self.nk(fun) == "Ident"
                && args.len() == 1
                && self.nodes[x].attrs["Ellipsis"] == "false"
                && (force || self.type_element(args[0]))
            {
                let open = self
                    .lexed
                    .tokens()
                    .iter()
                    .find(|t| t.start() >= self.e(fun) && t.end() <= self.e(x) && t.kind() == "(")
                    .unwrap()
                    .start();
                let n = self.node("ParenExpr", open, self.e(x))?;
                self.edge(n, "X", args[0]);
                return Ok((Some(fun), Some(n)));
            }
        }
        Ok((None, Some(x)))
    }
}
