//! Source membership of candidate declaration keys, independently parsed from
//! original owned bytes. Type expressions are retained, not presumed to denote
//! the supplied canonical type graph or a complete semantic context.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoBoundTypeDeclaration {
    declaration_id: String,
    package_id: String,
    source_object_id: String,
    type_spec_node: usize,
    name_node: usize,
    type_node: usize,
    alias: bool,
}
impl GoBoundTypeDeclaration {
    pub fn declaration_id(&self) -> &str { &self.declaration_id }
    pub fn package_id(&self) -> &str { &self.package_id }
    pub fn source_object_id(&self) -> &str { &self.source_object_id }
    pub fn type_spec_node(&self) -> usize { self.type_spec_node }
    pub fn name_node(&self) -> usize { self.name_node }
    pub fn type_node(&self) -> usize { self.type_node }
    pub fn is_alias(&self) -> bool { self.alias }
}
/// Complete parsed candidate package files and exact original TypeSpec members.
/// This proves neither Go name/type correctness nor module/import/build identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceTypeDeclarations {
    syntax: BTreeMap<String, GoParsedSource>,
    declarations: BTreeMap<String, GoBoundTypeDeclaration>,
}
impl GoSourceTypeDeclarations {
    pub fn source(&self, id: &str) -> Option<&GoParsedSource> { self.syntax.get(id) }
    pub fn declaration(&self, id: &str) -> Option<&GoBoundTypeDeclaration> { self.declarations.get(id) }
    pub fn declarations(&self) -> impl Iterator<Item = &GoBoundTypeDeclaration> { self.declarations.values() }
}
fn child(node: &GoSyntaxNode, role: &str) -> Option<usize> {
    node.edges().iter().find(|e|e.role()==role).and_then(|e| {
        if e.targets().len()==1 { e.targets().first().copied() } else { None }
    })
}
fn attribute<'a>(node: &'a GoSyntaxNode, key: &str) -> Option<&'a str> {
    node.attributes().find_map(|(k,v)|(k==key).then_some(v))
}
pub(super) fn bind(w: &WireComponents, sources: &[GoTypeSourceBuffer]) -> Result<GoSourceTypeDeclarations> {
    let buffers: BTreeMap<_,_> = sources.iter().map(|s|(s.object().id.as_str(),s)).collect();
    let mut syntax = BTreeMap::new();
    for package in &w.packages {
        for id in &package.source_objects {
            if !syntax.contains_key(id) {
                let source = buffers.get(id.as_str()).ok_or(GoTypeKeyError::Source)?;
                let parsed = parse_go_source(source).map_err(|_|GoTypeKeyError::Source)?;
                syntax.insert(id.clone(),parsed);
            }
            let parsed = &syntax[id];
            let file = &parsed.nodes()[parsed.root()];
            let name = child(file,"Name").ok_or(GoTypeKeyError::Source)?;
            if file.kind()!="File" || parsed.nodes()[name].kind()!="Ident"
                || attribute(&parsed.nodes()[name],"Name")!=Some(package.package_name.as_str()) {
                return Err(GoTypeKeyError::Source)
            }
        }
    }
    let mut specs = BTreeMap::new();
    for (source,parsed) in &syntax {
        for (id,node) in parsed.nodes().iter().enumerate() {
            if node.kind()=="TypeSpec" && specs.insert((source.as_str(),node.start(),node.end()),id).is_some() {
                return Err(GoTypeKeyError::Source)
            }
        }
    }
    let mut declarations = BTreeMap::new();
    for declaration in &w.declarations {
        let parsed = syntax.get(&declaration.source_object_id).ok_or(GoTypeKeyError::Source)?;
        let start = usize::try_from(declaration.start_byte).map_err(|_|GoTypeKeyError::Source)?;
        let end = usize::try_from(declaration.end_byte).map_err(|_|GoTypeKeyError::Source)?;
        let id = *specs.get(&(declaration.source_object_id.as_str(),start,end)).ok_or(GoTypeKeyError::Source)?;
        let node = &parsed.nodes()[id];
        let name = child(node,"Name").ok_or(GoTypeKeyError::Source)?;
        let typ = child(node,"Type").ok_or(GoTypeKeyError::Source)?;
        let alias = attribute(node,"Alias")==Some("true");
        // Candidate v2 has no type argument/parameter semantics. Original
        // generic declarations elsewhere in a package remain fully parsed.
        if node.edges().iter().any(|e|e.role()=="TypeParams")
            || parsed.nodes()[name].kind()!="Ident"
            || attribute(&parsed.nodes()[name],"Name")!=Some(declaration.name.as_str())
            || alias!=(declaration.kind=="alias_type") {
            return Err(GoTypeKeyError::Source)
        }
        declarations.insert(declaration.id.clone(),GoBoundTypeDeclaration {
            declaration_id:declaration.id.clone(),package_id:declaration.package_id.clone(),
            source_object_id:declaration.source_object_id.clone(),type_spec_node:id,
            name_node:name,type_node:typ,alias,
        });
    }
    Ok(GoSourceTypeDeclarations{syntax,declarations})
}
