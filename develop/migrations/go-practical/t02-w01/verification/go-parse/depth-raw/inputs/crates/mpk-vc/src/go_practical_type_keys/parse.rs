//! Independent original-source syntax. No type resolution, capture context or
//! source/program proof is granted by a successfully constructed syntax tree.
use super::{lex_go_source, GoLexedSource, GoTypeSourceBuffer};
use std::collections::BTreeMap;
mod declarations;
mod expressions;
mod statements;
mod types;
type Id = usize;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoParseErrorKind {
    Lexical,
    Syntax,
    Nesting,
    Limit,
    Worker,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoParseError {
    pub offset: usize,
    pub kind: GoParseErrorKind,
}
impl std::fmt::Display for GoParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go parse error {:?} at byte {}", self.kind, self.offset)
    }
}
impl std::error::Error for GoParseError {}
type Result<T> = std::result::Result<T, GoParseError>;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSyntaxEdge {
    role: &'static str,
    targets: Vec<Id>,
}
impl GoSyntaxEdge {
    pub fn role(&self) -> &str {
        self.role
    }
    pub fn targets(&self) -> &[Id] {
        &self.targets
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSyntaxNode {
    kind: &'static str,
    start: usize,
    end: usize,
    attrs: BTreeMap<&'static str, String>,
    edges: Vec<GoSyntaxEdge>,
}
impl GoSyntaxNode {
    pub fn kind(&self) -> &str {
        self.kind
    }
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
    pub fn attributes(&self) -> impl Iterator<Item = (&str, &str)> {
        self.attrs.iter().map(|(k, v)| (*k, v.as_str()))
    }
    pub fn edges(&self) -> &[GoSyntaxEdge] {
        &self.edges
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoParsedSource {
    lexed: GoLexedSource,
    nodes: Vec<GoSyntaxNode>,
    root: Id,
}
impl GoParsedSource {
    pub fn source(&self) -> &GoTypeSourceBuffer {
        self.lexed.source()
    }
    pub fn lexed(&self) -> &GoLexedSource {
        &self.lexed
    }
    pub fn nodes(&self) -> &[GoSyntaxNode] {
        &self.nodes
    }
    pub fn root(&self) -> Id {
        self.root
    }
    pub fn spelling(&self, id: Id) -> Option<&[u8]> {
        self.nodes
            .get(id)
            .map(|n| &self.source().bytes()[n.start..n.end])
    }
}
struct Parser {
    lexed: GoLexedSource,
    at: usize,
    nodes: Vec<GoSyntaxNode>,
    level: i32,
    nesting: usize,
}
impl Parser {
    fn kind(&self) -> &str {
        self.lexed.tokens()[self.at].kind()
    }
    fn pos(&self) -> usize {
        self.lexed.tokens()[self.at].start()
    }
    fn end(&self) -> usize {
        self.lexed.tokens()[self.at].end()
    }
    fn is(&self, k: &str) -> bool {
        self.kind() == k
    }
    fn err<T>(&self) -> Result<T> {
        Err(GoParseError {
            offset: self.pos(),
            kind: GoParseErrorKind::Syntax,
        })
    }
    fn bump(&mut self) -> usize {
        let p = self.pos();
        if !self.is("EOF") {
            self.at += 1
        }
        p
    }
    fn eat(&mut self, k: &str) -> Option<usize> {
        self.is(k).then(|| self.bump())
    }
    fn expect(&mut self, k: &str) -> Result<usize> {
        if self.is(k) {
            Ok(self.bump())
        } else {
            self.err()
        }
    }
    fn close(&mut self, k: &str) -> Result<usize> {
        let end = self.end();
        self.expect(k)?;
        Ok(end)
    }
    fn semi(&mut self) -> Result<()> {
        if self.eat(";").is_some() || self.is(")") || self.is("}") {
            Ok(())
        } else {
            self.err()
        }
    }
    fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.nesting += 1;
        if self.nesting > 100_000 {
            return Err(GoParseError {
                offset: self.pos(),
                kind: GoParseErrorKind::Nesting,
            });
        }
        let result = f(self);
        self.nesting -= 1;
        result
    }
    fn step_nesting(&mut self) -> Result<()> {
        self.nesting += 1;
        if self.nesting > 100_000 {
            Err(GoParseError {
                offset: self.pos(),
                kind: GoParseErrorKind::Nesting,
            })
        } else {
            Ok(())
        }
    }
    fn node(&mut self, kind: &'static str, start: usize, end: usize) -> Result<Id> {
        // Transient grammar disambiguation nodes aren't final AST nodes. Bound
        // construction by source tokens; apply the frozen actual-node budget only
        // to the complete reachable syntax tree below.
        if self.nodes.len()
            > self
                .lexed
                .tokens()
                .len()
                .saturating_mul(4)
                .saturating_add(8)
        {
            return Err(GoParseError {
                offset: start,
                kind: GoParseErrorKind::Limit,
            });
        }
        if start > end || end > self.lexed.source().bytes().len() {
            return self.err();
        }
        let id = self.nodes.len();
        self.nodes.push(GoSyntaxNode {
            kind,
            start,
            end,
            attrs: BTreeMap::new(),
            edges: vec![],
        });
        Ok(id)
    }
    fn s(&self, n: Id) -> usize {
        self.nodes[n].start
    }
    fn e(&self, n: Id) -> usize {
        self.nodes[n].end
    }
    fn nk(&self, n: Id) -> &str {
        self.nodes[n].kind
    }
    fn attr(&mut self, n: Id, k: &'static str, v: impl ToString) {
        self.nodes[n].attrs.insert(k, v.to_string());
    }
    fn edge(&mut self, n: Id, role: &'static str, target: Id) {
        self.list(n, role, vec![target]);
    }
    fn optional(&mut self, n: Id, role: &'static str, target: Option<Id>) {
        if let Some(target) = target {
            self.edge(n, role, target)
        }
    }
    fn list(&mut self, n: Id, role: &'static str, targets: Vec<Id>) {
        if !targets.is_empty() {
            self.nodes[n].edges.push(GoSyntaxEdge { role, targets });
        }
    }
    fn child(&self, n: Id, role: &str) -> Option<Id> {
        self.nodes[n]
            .edges
            .iter()
            .find(|e| e.role == role)
            .and_then(|e| e.targets.first().copied())
    }
    fn children(&self, n: Id, role: &str) -> Vec<Id> {
        self.nodes[n]
            .edges
            .iter()
            .find(|e| e.role == role)
            .map_or(vec![], |e| e.targets.clone())
    }
    fn ident(&mut self) -> Result<Id> {
        if !self.is("IDENT") {
            return self.err();
        }
        let start = self.pos();
        let end = self.end();
        let name =
            String::from_utf8(self.lexed.tokens()[self.at].literal().to_vec()).map_err(|_| {
                GoParseError {
                    offset: start,
                    kind: GoParseErrorKind::Lexical,
                }
            })?;
        self.bump();
        let n = self.node("Ident", start, end)?;
        self.attr(n, "Name", name);
        Ok(n)
    }
    fn literal(&mut self) -> Result<Id> {
        let kind = match self.kind() {
            "INT" => "INT",
            "FLOAT" => "FLOAT",
            "IMAG" => "IMAG",
            "CHAR" => "CHAR",
            "STRING" => "STRING",
            _ => return self.err(),
        };
        let start = self.pos();
        let end = self.end();
        let value = self.lexed.tokens()[self.at]
            .literal()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        self.bump();
        let n = self.node("BasicLit", start, end)?;
        self.attr(n, "Kind", kind);
        self.attr(n, "ValueHex", value);
        Ok(n)
    }
    fn identifiers(&mut self) -> Result<Vec<Id>> {
        let mut list = vec![self.ident()?];
        while self.eat(",").is_some() {
            list.push(self.ident()?)
        }
        Ok(list)
    }
    fn finish(self, root: Id) -> Result<GoParsedSource> {
        // Stable preorder by field role, preserving source order inside each list.
        // Only reachable parser output is counted. No caller-supplied graph exists.
        let mut ids = BTreeMap::new();
        let mut order = vec![];
        let mut todo = vec![root];
        while let Some(id) = todo.pop() {
            if ids.contains_key(&id) {
                continue;
            }
            ids.insert(id, order.len());
            order.push(id);
            let mut edges = self.nodes[id].edges.iter().collect::<Vec<_>>();
            edges.sort_by_key(|e| e.role);
            for edge in edges.into_iter().rev() {
                todo.extend(edge.targets.iter().rev().copied())
            }
        }
        if order.len() > 4_194_304 {
            return Err(GoParseError {
                offset: 0,
                kind: GoParseErrorKind::Limit,
            });
        }
        let nodes = order
            .into_iter()
            .map(|old| {
                let mut n = self.nodes[old].clone();
                n.edges.sort_by_key(|e| e.role);
                for edge in &mut n.edges {
                    for target in &mut edge.targets {
                        *target = ids[target]
                    }
                }
                n
            })
            .collect();
        Ok(GoParsedSource {
            lexed: self.lexed,
            nodes,
            root: 0,
        })
    }
}
/// Pure syntax over owned bytes. The worker stack accommodates the pinned Go
/// parser's deep syntax domain; stack reservation isn't a language/profile cap.
pub fn parse_go_source(source: &GoTypeSourceBuffer) -> Result<GoParsedSource> {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("mpk-go-parse".into())
            .stack_size(1 << 30)
            .spawn_scoped(scope, || {
                let lexed = lex_go_source(source).map_err(|e| GoParseError {
                    offset: e.offset,
                    kind: GoParseErrorKind::Lexical,
                })?;
                let mut parser = Parser {
                    lexed,
                    at: 0,
                    nodes: vec![],
                    level: 0,
                    nesting: 0,
                };
                let root = parser.file()?;
                parser.finish(root)
            })
            .map_err(|_| GoParseError {
                offset: 0,
                kind: GoParseErrorKind::Worker,
            })?;
        worker.join().map_err(|_| GoParseError {
            offset: 0,
            kind: GoParseErrorKind::Worker,
        })?
    })
}
