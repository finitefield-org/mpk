use super::*;
impl Parser {
    pub(super) fn file(&mut self) -> Result<Id> {
        let start = self.expect("package")?;
        let name = self.ident()?;
        self.semi()?;
        let mut decls = vec![];
        let mut imports = true;
        while !self.is("EOF") {
            if self.is("import") && !imports {
                return self.err();
            }
            if !self.is("import") {
                imports = false
            }
            decls.push(if self.is("func") {
                self.function_decl()?
            } else {
                self.general_decl()?
            });
        }
        let end = decls.last().map_or(self.e(name), |n| self.e(*n));
        let n = self.node("File", start, end)?;
        self.edge(n, "Name", name);
        self.list(n, "Decls", decls);
        Ok(n)
    }
    pub(super) fn general_decl(&mut self) -> Result<Id> {
        let keyword = match self.kind() {
            "import" => "import",
            "const" => "const",
            "var" => "var",
            "type" => "type",
            _ => return self.err(),
        };
        let start = self.bump();
        let grouped = self.eat("(").is_some();
        let mut specs = vec![];
        if grouped {
            while !self.is(")") {
                specs.push(self.spec(keyword)?)
            }
            let end = self.close(")")?;
            self.semi()?;
            let n = self.node("GenDecl", start, end)?;
            self.attr(n, "Tok", keyword);
            self.attr(n, "Grouped", true);
            self.list(n, "Specs", specs);
            Ok(n)
        } else {
            let spec = self.spec(keyword)?;
            let n = self.node("GenDecl", start, self.e(spec))?;
            self.attr(n, "Tok", keyword);
            self.attr(n, "Grouped", false);
            self.edge(n, "Specs", spec);
            Ok(n)
        }
    }
    fn spec(&mut self, keyword: &str) -> Result<Id> {
        if keyword == "import" {
            let name = if self.is("IDENT") {
                Some(self.ident()?)
            } else if self.eat(".").is_some() {
                let p = self.lexed.tokens()[self.at - 1].start();
                let n = self.node("Ident", p, p + 1)?;
                self.attr(n, "Name", ".");
                Some(n)
            } else {
                None
            };
            if !self.is("STRING") {
                return self.err();
            }
            let path = self.literal()?;
            let n = self.node(
                "ImportSpec",
                name.map_or(self.s(path), |n| self.s(n)),
                self.e(path),
            )?;
            self.optional(n, "Name", name);
            self.edge(n, "Path", path);
            self.semi()?;
            return Ok(n);
        }
        if keyword == "type" {
            return self.type_spec();
        }
        let names = self.identifiers()?;
        let typ = if self.type_start() {
            Some(self.typ()?)
        } else {
            None
        };
        let values = if self.eat("=").is_some() {
            self.exprs()?
        } else {
            vec![]
        };
        if keyword == "var" && typ.is_none() && values.is_empty() {
            return self.err();
        }
        let end = values.last().map_or_else(
            || typ.map_or(self.e(*names.last().unwrap()), |t| self.e(t)),
            |v| self.e(*v),
        );
        let n = self.node("ValueSpec", self.s(names[0]), end)?;
        self.list(n, "Names", names);
        self.optional(n, "Type", typ);
        self.list(n, "Values", values);
        self.semi()?;
        Ok(n)
    }
    fn type_spec(&mut self) -> Result<Id> {
        let name = self.ident()?;
        let mut alias = false;
        let mut params = None;
        let typ = if self.is("[") {
            let opening = self.bump();
            if self.is("IDENT") {
                let first = self.ident()?;
                let mut x = first;
                if !self.is("[") {
                    self.level += 1;
                    x = self.primary(first)?;
                    x = self.binary(Some(x), 1)?;
                    self.level -= 1
                }
                let (pname, ptype) = self.extract_name(x, self.is(","))?;
                if let Some(pname) = pname.filter(|_| ptype.is_some() || !self.is("]")) {
                    params = Some(self.params(
                        opening,
                        "]",
                        true,
                        false,
                        Some((pname, ptype)),
                    )?);
                    alias = self.eat("=").is_some();
                    self.typ()?
                } else {
                    self.array_after_open(opening, Some(x))?
                }
            } else {
                self.array_after_open(opening, None)?
            }
        } else {
            alias = self.eat("=").is_some();
            self.typ()?
        };
        let n = self.node("TypeSpec", self.s(name), self.e(typ))?;
        self.attr(n, "Alias", alias);
        self.edge(n, "Name", name);
        self.optional(n, "TypeParams", params);
        self.edge(n, "Type", typ);
        self.semi()?;
        Ok(n)
    }
    fn function_decl(&mut self) -> Result<Id> {
        let start = self.expect("func")?;
        let receiver = if self.is("(") {
            let open = self.bump();
            Some(self.params(open, ")", false, false, None)?)
        } else {
            None
        };
        let name = self.ident()?;
        let params = if self.is("[") {
            let open = self.bump();
            Some(self.params(open, "]", true, false, None)?)
        } else {
            None
        };
        let typ = self.signature(start, params)?;
        let body = if self.is("{") {
            Some(self.block()?)
        } else {
            None
        };
        let n = self.node("FuncDecl", start, body.map_or(self.e(typ), |b| self.e(b)))?;
        self.optional(n, "Recv", receiver);
        self.edge(n, "Name", name);
        self.edge(n, "Type", typ);
        self.optional(n, "Body", body);
        self.semi()?;
        Ok(n)
    }
}
