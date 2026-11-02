use super::*;
impl Parser {
    pub(super) fn block(&mut self) -> Result<Id> {
        let start = self.expect("{")?;
        let stmts = self.statement_list()?;
        let end = self.close("}")?;
        let n = self.node("BlockStmt", start, end)?;
        self.list(n, "List", stmts);
        Ok(n)
    }
    fn statement_list(&mut self) -> Result<Vec<Id>> {
        let mut list = vec![];
        while !matches!(self.kind(), "}" | "case" | "default" | "EOF") {
            list.push(self.statement()?)
        }
        Ok(list)
    }
    fn simple(&mut self, labels: bool, ranges: bool) -> Result<(Id, bool)> {
        let lhs = self.exprs()?;
        if matches!(
            self.kind(),
            "=" | ":="
                | "+="
                | "-="
                | "*="
                | "/="
                | "%="
                | "&="
                | "|="
                | "^="
                | "<<="
                | ">>="
                | "&^="
        ) {
            let op = self.kind().to_string();
            self.bump();
            let range = ranges && matches!(op.as_str(), "=" | ":=") && self.eat("range").is_some();
            let rhs = self.exprs()?;
            if range && rhs.len() != 1 {
                return self.err();
            }
            let n = self.node(
                if range { "RangeClause" } else { "AssignStmt" },
                self.s(lhs[0]),
                self.e(*rhs.last().unwrap()),
            )?;
            self.attr(n, "Tok", op);
            self.list(n, "Lhs", lhs);
            self.list(n, "Rhs", rhs);
            return Ok((n, range));
        }
        if lhs.len() != 1 {
            return self.err();
        }
        let x = lhs[0];
        if self.eat(":").is_some() {
            if !labels || self.nk(x) != "Ident" {
                return self.err();
            }
            let stmt = self.statement()?;
            let n = self.node("LabeledStmt", self.s(x), self.e(stmt))?;
            self.edge(n, "Label", x);
            self.edge(n, "Stmt", stmt);
            return Ok((n, false));
        }
        if self.eat("<-").is_some() {
            let value = self.expression()?;
            let n = self.node("SendStmt", self.s(x), self.e(value))?;
            self.edge(n, "Chan", x);
            self.edge(n, "Value", value);
            return Ok((n, false));
        }
        if matches!(self.kind(), "++" | "--") {
            let end = self.end();
            let op = self.kind().to_string();
            self.bump();
            let n = self.node("IncDecStmt", self.s(x), end)?;
            self.attr(n, "Tok", op);
            self.edge(n, "X", x);
            return Ok((n, false));
        }
        let n = self.node("ExprStmt", self.s(x), self.e(x))?;
        self.edge(n, "X", x);
        Ok((n, false))
    }
    fn make_expr(&self, stmt: Option<Id>) -> Result<Option<Id>> {
        match stmt {
            None => Ok(None),
            Some(n) if self.nk(n) == "ExprStmt" => Ok(self.child(n, "X")),
            _ => self.err(),
        }
    }
    fn statement(&mut self) -> Result<Id> {
        self.nested(|p| p.statement_inner())
    }
    fn statement_inner(&mut self) -> Result<Id> {
        let start = self.pos();
        match self.kind() {
            ";" => {
                let end = self.end();
                let implicit = self.lexed.tokens()[self.at].inserted();
                self.bump();
                let n = self.node("EmptyStmt", start, end)?;
                self.attr(n, "Implicit", implicit);
                Ok(n)
            }
            "}" => {
                let n = self.node("EmptyStmt", start, start)?;
                self.attr(n, "Implicit", true);
                Ok(n)
            }
            "const" | "var" | "type" => {
                let decl = self.general_decl()?;
                let n = self.node("DeclStmt", self.s(decl), self.e(decl))?;
                self.edge(n, "Decl", decl);
                Ok(n)
            }
            "{" => {
                let block = self.block()?;
                self.semi()?;
                Ok(block)
            }
            "return" => {
                self.bump();
                let xs = if matches!(self.kind(), ";" | "}") {
                    vec![]
                } else {
                    self.exprs()?
                };
                let end = xs.last().map_or(start + 6, |n| self.e(*n));
                let n = self.node("ReturnStmt", start, end)?;
                self.list(n, "Results", xs);
                self.semi()?;
                Ok(n)
            }
            "break" | "continue" | "goto" | "fallthrough" => {
                let op = self.kind().to_string();
                self.bump();
                let label = if op == "goto" || op != "fallthrough" && self.is("IDENT") {
                    Some(self.ident()?)
                } else {
                    None
                };
                let n = self.node(
                    "BranchStmt",
                    start,
                    label.map_or(start + op.len(), |n| self.e(n)),
                )?;
                self.attr(n, "Tok", op);
                self.optional(n, "Label", label);
                self.semi()?;
                Ok(n)
            }
            "go" | "defer" => {
                let go = self.is("go");
                self.bump();
                let mut call = self.expression()?;
                while self.nk(call) == "ParenExpr" {
                    call = self.child(call, "X").unwrap()
                }
                if self.nk(call) != "CallExpr" {
                    return self.err();
                }
                let n = self.node(if go { "GoStmt" } else { "DeferStmt" }, start, self.e(call))?;
                self.edge(n, "Call", call);
                self.semi()?;
                Ok(n)
            }
            "if" => self.if_statement(),
            "for" => self.for_statement(),
            "switch" => self.switch_statement(),
            "select" => self.select_statement(),
            "IDENT" | "INT" | "FLOAT" | "IMAG" | "CHAR" | "STRING" | "func" | "(" | "["
            | "struct" | "map" | "chan" | "interface" | "+" | "-" | "*" | "&" | "^" | "<-"
            | "!" => {
                let (n, _) = self.simple(true, false)?;
                if self.nk(n) != "LabeledStmt" {
                    self.semi()?
                }
                Ok(n)
            }
            _ => self.err(),
        }
    }
    fn if_statement(&mut self) -> Result<Id> {
        let start = self.expect("if")?;
        let level = self.level;
        self.level = -1;
        let mut init = None;
        let first = if self.is(";") {
            None
        } else {
            Some(self.simple(false, false)?.0)
        };
        let condition = if self.eat(";").is_some() {
            init = first;
            let cond = self.simple(false, false)?.0;
            self.make_expr(Some(cond))?.ok_or(GoParseError {
                offset: self.pos(),
                kind: GoParseErrorKind::Syntax,
            })?
        } else {
            self.make_expr(first)?.ok_or(GoParseError {
                offset: self.pos(),
                kind: GoParseErrorKind::Syntax,
            })?
        };
        self.level = level;
        let body = self.block()?;
        let alternative = if self.eat("else").is_some() {
            if self.is("if") {
                Some(self.if_statement()?)
            } else {
                let b = self.block()?;
                self.semi()?;
                Some(b)
            }
        } else {
            self.semi()?;
            None
        };
        let n = self.node(
            "IfStmt",
            start,
            alternative.map_or(self.e(body), |b| self.e(b)),
        )?;
        self.optional(n, "Init", init);
        self.edge(n, "Cond", condition);
        self.edge(n, "Body", body);
        self.optional(n, "Else", alternative);
        Ok(n)
    }
    fn for_statement(&mut self) -> Result<Id> {
        let start = self.expect("for")?;
        let level = self.level;
        self.level = -1;
        let mut init = None;
        let mut condition = None;
        let mut post = None;
        let mut range = None;
        if !self.is("{") {
            if self.eat("range").is_some() {
                let x = self.expression()?;
                range = Some((vec![], "ILLEGAL".to_string(), x))
            } else if !self.is(";") {
                let (stmt, is_range) = self.simple(false, true)?;
                if is_range {
                    let lhs = self.children(stmt, "Lhs");
                    let rhs = self.child(stmt, "Rhs").unwrap();
                    range = Some((lhs, self.nodes[stmt].attrs["Tok"].clone(), rhs))
                } else {
                    condition = Some(stmt)
                }
            }
            if range.is_none() && self.eat(";").is_some() {
                init = condition.take();
                if !self.is(";") {
                    condition = Some(self.simple(false, false)?.0)
                }
                self.expect(";")?;
                if !self.is("{") {
                    post = Some(self.simple(false, false)?.0)
                }
            }
        }
        self.level = level;
        let body = self.block()?;
        self.semi()?;
        if let Some((lhs, tok, x)) = range {
            if lhs.len() > 2 {
                return self.err();
            }
            let n = self.node("RangeStmt", start, self.e(body))?;
            self.optional(n, "Key", lhs.first().copied());
            self.optional(n, "Value", lhs.get(1).copied());
            self.edge(n, "X", x);
            self.edge(n, "Body", body);
            self.attr(n, "Tok", tok);
            Ok(n)
        } else {
            let cond = self.make_expr(condition)?;
            let n = self.node("ForStmt", start, self.e(body))?;
            self.optional(n, "Init", init);
            self.optional(n, "Cond", cond);
            self.optional(n, "Post", post);
            self.edge(n, "Body", body);
            Ok(n)
        }
    }
    fn type_switch(&self, s: Option<Id>) -> bool {
        let Some(s) = s else { return false };
        let x = if self.nk(s) == "ExprStmt" {
            self.child(s, "X")
        } else if self.nk(s) == "AssignStmt"
            && self.nodes[s].attrs["Tok"] == ":="
            && self.children(s, "Lhs").len() == 1
            && self.children(s, "Rhs").len() == 1
        {
            self.child(s, "Rhs")
        } else {
            None
        };
        x.is_some_and(|x| self.nk(x) == "TypeAssertExpr" && self.child(x, "Type").is_none())
    }
    fn switch_statement(&mut self) -> Result<Id> {
        let start = self.expect("switch")?;
        let old = self.level;
        self.level = -1;
        let mut init = None;
        let mut tag = None;
        if !self.is("{") {
            if !self.is(";") {
                tag = Some(self.simple(false, false)?.0)
            }
            if self.eat(";").is_some() {
                init = tag.take();
                if !self.is("{") {
                    tag = Some(self.simple(false, false)?.0)
                }
            }
        }
        self.level = old;
        let type_switch = self.type_switch(tag);
        let opening = self.expect("{")?;
        let mut clauses = vec![];
        while matches!(self.kind(), "case" | "default") {
            let start = self.bump();
            let values = if self.lexed.tokens()[self.at - 1].kind() == "case" {
                self.exprs()?
            } else {
                vec![]
            };
            let colon = self.expect(":")?;
            let body = self.statement_list()?;
            let end = body.last().map_or(colon + 1, |b| self.e(*b));
            let c = self.node("CaseClause", start, end)?;
            self.list(c, "List", values);
            self.list(c, "Body", body);
            clauses.push(c);
        }
        let end = self.close("}")?;
        self.semi()?;
        let body = self.node("BlockStmt", opening, end)?;
        self.list(body, "List", clauses);
        let n = self.node(
            if type_switch {
                "TypeSwitchStmt"
            } else {
                "SwitchStmt"
            },
            start,
            end,
        )?;
        self.optional(n, "Init", init);
        if type_switch {
            self.optional(n, "Assign", tag)
        } else {
            let x = self.make_expr(tag)?;
            self.optional(n, "Tag", x)
        }
        self.edge(n, "Body", body);
        Ok(n)
    }
    fn select_statement(&mut self) -> Result<Id> {
        let start = self.expect("select")?;
        let opening = self.expect("{")?;
        let mut clauses = vec![];
        while matches!(self.kind(), "case" | "default") {
            let start = self.bump();
            let comm = if self.lexed.tokens()[self.at - 1].kind() == "case" {
                Some(self.simple(false, false)?.0)
            } else {
                None
            };
            let colon = self.expect(":")?;
            let body = self.statement_list()?;
            let end = body.last().map_or(colon + 1, |b| self.e(*b));
            let c = self.node("CommClause", start, end)?;
            self.optional(c, "Comm", comm);
            self.list(c, "Body", body);
            clauses.push(c);
        }
        let end = self.close("}")?;
        self.semi()?;
        let body = self.node("BlockStmt", opening, end)?;
        self.list(body, "List", clauses);
        let n = self.node("SelectStmt", start, end)?;
        self.edge(n, "Body", body);
        Ok(n)
    }
}
