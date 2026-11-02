//! Owned original method signatures and error compatibility, not body/state proof.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoSourceMethod {
    source: Option<String>,
    declaration: Option<usize>,
    body: Option<usize>,
    name: String,
    receiver: String,
    path: Vec<usize>,
    parameters: Vec<String>,
    results: Vec<String>,
}
impl GoSourceMethod {
    /// None denotes the signature of the actual predeclared error interface.
    pub fn declaration_source_object_id(&self) -> Option<&str> {
        self.source.as_deref()
    }
    pub fn declaration_node(&self) -> Option<usize> {
        self.declaration
    }
    pub fn body_node(&self) -> Option<usize> {
        self.body
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn receiver_type_id(&self) -> &str {
        &self.receiver
    }
    /// Indices in each actual underlying struct's ordered original fields.
    pub fn receiver_path(&self) -> &[usize] {
        &self.path
    }
    pub fn parameter_type_ids(&self) -> &[String] {
        &self.parameters
    }
    pub fn result_type_ids(&self) -> &[String] {
        &self.results
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoErrorImplementation {
    value_type: String,
    method: GoSourceMethod,
}
impl GoErrorImplementation {
    pub fn value_type_id(&self) -> &str {
        &self.value_type
    }
    pub fn method(&self) -> &GoSourceMethod {
        &self.method
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoMethodCallHeader {
    source: String,
    call: usize,
    function: usize,
    receiver_node: usize,
    selected_receiver: String,
    expression: bool,
    method: GoSourceMethod,
    pub(super) parameters: Vec<String>,
}
impl GoMethodCallHeader {
    pub fn call_source_object_id(&self) -> &str {
        &self.source
    }
    pub fn call_node(&self) -> usize {
        self.call
    }
    pub fn function_node(&self) -> usize {
        self.function
    }
    pub fn receiver_node(&self) -> usize {
        self.receiver_node
    }
    pub fn selected_receiver_type_id(&self) -> &str {
        &self.selected_receiver
    }
    pub fn method_expression(&self) -> bool {
        self.expression
    }
    pub fn method(&self) -> &GoSourceMethod {
        &self.method
    }
    /// Method expressions include their actual explicit receiver parameter.
    pub fn parameter_type_ids(&self) -> &[String] {
        &self.parameters
    }
    pub fn result_type_ids(&self) -> &[String] {
        &self.method.results
    }
}
#[derive(Clone)]
pub(super) struct OriginalMethod {
    source: String,
    node: usize,
    receiver_node: usize,
    pointer: bool,
}
pub(super) type MethodPackages = BTreeMap<String, BTreeMap<String, Vec<OriginalMethod>>>;
#[derive(Clone)]
pub(super) enum Candidate {
    Source {
        original: OriginalMethod,
        receiver: String,
    },
    ErrorInterface(String),
}
enum Member {
    Field,
    Method(Candidate, Vec<usize>),
}
impl Candidate {
    fn pointer(&self) -> bool {
        matches!(self,Self::Source{original,..}if original.pointer)
    }
}
impl Engine<'_> {
    fn receiver_denotation(&self, source: &str, node: usize) -> Option<(String, bool)> {
        let mut source = source.to_string();
        let mut node = node;
        let mut pointer = false;
        let mut seen = BTreeSet::new();
        loop {
            if !seen.insert((source.clone(), node)) {
                return None;
            }
            let n = self.node(&source, node).ok()?;
            if matches!(n.kind(), "StarExpr" | "ParenExpr") {
                pointer |= n.kind() == "StarExpr";
                node = child(n, "X").ok()?;
                continue;
            }
            if let Ok(GoNameLookup::Original(binding)) = self.symbol(&source, node) {
                if binding.kind() == GoSourceNameKind::Type {
                    let spec = self
                        .node(binding.source_object_id(), binding.declaration_node())
                        .ok()?;
                    if attr(spec, "Alias") == Some("true") {
                        if !list(spec, "TypeParams").is_empty() {
                            return None;
                        }
                        node = child(spec, "Type").ok()?;
                        source = binding.source_object_id().into();
                        continue;
                    }
                }
            }
            let graph = derive_go_source_type(self.scopes, &source, node).ok()?;
            return Some((graph.roots()[0].clone(), pointer));
        }
    }
    pub(super) fn is_error(&self, id: &str) -> bool {
        matches!(self.keys.get(id),Some(GoSourceTypeKey::Builtin{name,..})if name=="error")
    }
    pub(super) fn direct_methods(&mut self, id: &str, name: &str) -> Result<Vec<Candidate>> {
        if self.is_error(id) {
            return Ok(if name == "Error" {
                vec![Candidate::ErrorInterface(id.into())]
            } else {
                vec![]
            });
        }
        let Some(GoSourceTypeKey::Nominal { declaration_id, .. }) = self.keys.get(id) else {
            return Ok(vec![]);
        };
        let package = self
            .declarations
            .get(declaration_id)
            .ok_or(GoScalarTypingError::Type)?
            .package_id
            .clone();
        if !self.method_packages.contains_key(&package) {
            let mut names: BTreeMap<String, Vec<OriginalMethod>> = BTreeMap::new();
            for (source, parsed) in self.scopes.sources() {
                if self.scopes.source_package(source) != Some(package.as_str()) {
                    continue;
                }
                for (node, n) in parsed.nodes().iter().enumerate() {
                    if n.kind() != "FuncDecl" || list(n, "Recv").is_empty() {
                        continue;
                    }
                    let name = attr(self.node(source, child(n, "Name")?)?, "Name")
                        .ok_or(GoScalarTypingError::Locator)?;
                    let receiver = self.node(source, child(n, "Recv")?)?;
                    let fields = list(receiver, "List");
                    if fields.len() != 1 {
                        return Err(GoScalarTypingError::Type);
                    }
                    let mut typ = child(self.node(source, fields[0])?, "Type")?;
                    let mut pointer = false;
                    while matches!(self.node(source, typ)?.kind(), "StarExpr" | "ParenExpr") {
                        pointer |= self.node(source, typ)?.kind() == "StarExpr";
                        typ = child(self.node(source, typ)?, "X")?;
                    }
                    names.entry(name.into()).or_default().push(OriginalMethod {
                        source: source.into(),
                        node,
                        receiver_node: typ,
                        pointer,
                    });
                }
            }
            self.method_packages.insert(package.clone(), names);
        }
        let originals = self.method_packages[&package]
            .get(name)
            .cloned()
            .unwrap_or_default();
        let mut result = vec![];
        for original in originals {
            let key = (original.source.clone(), original.receiver_node);
            if !self.method_receivers.contains_key(&key) {
                // Resolve actual receiver denotation, including aliases from
                // other files. Unrelated unsupported receivers do not poison it.
                let receiver = self.receiver_denotation(&original.source, original.receiver_node);
                self.method_receivers.insert(key.clone(), receiver);
            }
            if let Some((receiver, pointer)) = &self.method_receivers[&key] {
                if receiver == id {
                    let mut original = original;
                    original.pointer |= *pointer;
                    result.push(Candidate::Source {
                        original,
                        receiver: id.into(),
                    });
                }
            }
        }
        Ok(result)
    }
    fn lookup_method_member(&mut self, id: &str, name: &str, package: &str) -> Result<Member> {
        if name == "_" {
            return Err(GoScalarTypingError::Name);
        }
        let mut level = BTreeMap::from([(id.to_string(), (1u8, Vec::<usize>::new()))]);
        for _ in 0..=16 {
            let mut matches = 0u8;
            let mut found = None;
            let mut next: BTreeMap<String, (u8, Vec<usize>)> = BTreeMap::new();
            for (typ, (count, path)) in level {
                for method in self.direct_methods(&typ, name)? {
                    let visible = name.chars().next().is_some_and(validate::exported)
                        || match &method {
                            Candidate::Source { original, .. } => {
                                self.scopes.source_package(&original.source) == Some(package)
                            }
                            Candidate::ErrorInterface(_) => true,
                        };
                    if visible {
                        matches = matches.saturating_add(count).min(2);
                        found = Some(Member::Method(method, path.clone()));
                    }
                }
                if let GoSourceTypeKey::Struct { fields, .. } = self.shape(&typ)? {
                    for (index, f) in fields.iter().enumerate() {
                        if f.name == name
                            && f.unexported_package_id
                                .as_deref()
                                .is_none_or(|p| p == package)
                        {
                            matches = matches.saturating_add(count).min(2);
                            found = Some(Member::Field);
                        }
                        if f.embedded {
                            let mut child_path = path.clone();
                            child_path.push(index);
                            let entry = next.entry(f.type_id.clone()).or_insert((0, child_path));
                            entry.0 = entry.0.saturating_add(count).min(2);
                        }
                    }
                }
            }
            if matches > 1 {
                return Err(GoScalarTypingError::Name);
            }
            if matches == 1 {
                return found.ok_or(GoScalarTypingError::Name);
            }
            if next.is_empty() {
                return Err(GoScalarTypingError::Name);
            }
            level = next;
        }
        Err(GoScalarTypingError::Limit)
    }
    fn source_method(
        &self,
        candidate: &Candidate,
        path: Vec<usize>,
    ) -> Result<(GoSourceMethod, Option<GoSourceSignature>)> {
        match candidate {
            Candidate::ErrorInterface(id) => {
                let string = GoConstantScalarType::builtin("string").map_err(constant_error)?;
                Ok((
                    GoSourceMethod {
                        source: None,
                        declaration: None,
                        body: None,
                        name: "Error".into(),
                        receiver: id.clone(),
                        path,
                        parameters: vec![],
                        results: vec![string.source_type_key().id().into()],
                    },
                    None,
                ))
            }
            Candidate::Source { original, receiver } => {
                if original.pointer {
                    return Err(GoScalarTypingError::Unsupported);
                }
                let signature =
                    derive_go_source_signature(self.scopes, &original.source, original.node)
                        .map_err(GoScalarTypingError::Signature)?;
                let mut parameters = vec![];
                let mut results = vec![];
                let mut actual = None;
                for slot in signature.slots() {
                    match slot.role() {
                        GoSourceSignatureRole::Receiver => actual = Some(slot.type_id()),
                        GoSourceSignatureRole::Parameter => {
                            parameters.push(slot.type_id().to_string())
                        }
                        GoSourceSignatureRole::Result => results.push(slot.type_id().to_string()),
                    }
                }
                if actual != Some(receiver.as_str()) {
                    return Err(GoScalarTypingError::Type);
                }
                let method = GoSourceMethod {
                    source: Some(original.source.clone()),
                    declaration: Some(original.node),
                    body: signature.body_node(),
                    name: signature.name().ok_or(GoScalarTypingError::Name)?.into(),
                    receiver: receiver.clone(),
                    path,
                    parameters,
                    results,
                };
                Ok((method, Some(signature)))
            }
        }
    }
    pub(super) fn ensure_error_implementation(&mut self, id: &str) -> Result<()> {
        if self.error_implementations.contains_key(id)
            || !matches!(
                self.keys.get(id),
                Some(GoSourceTypeKey::Nominal { .. } | GoSourceTypeKey::Struct { .. })
            )
        {
            return Ok(());
        }
        self.error_implementations.insert(id.into(), None);
        let Ok(Member::Method(candidate, path)) = self.lookup_method_member(id, "Error", "") else {
            return Ok(());
        };
        if candidate.pointer() {
            return Ok(());
        }
        let Ok((method, _)) = self.source_method(&candidate, path) else {
            return Ok(());
        };
        let string = GoConstantScalarType::builtin("string").map_err(constant_error)?;
        if !method.parameters.is_empty() || method.results != [string.source_type_key().id()] {
            return Ok(());
        }
        self.intern(string)?;
        self.error_implementations.insert(
            id.into(),
            Some(GoErrorImplementation {
                value_type: id.into(),
                method,
            }),
        );
        Ok(())
    }
    pub(super) fn method_plan(
        &mut self,
        r: &Request,
        selector: usize,
        args: Vec<Request>,
    ) -> Result<Step> {
        let base = child(self.node(&r.source, selector)?, "X")?;
        match derive_go_source_type(self.scopes, &r.source, base) {
            Ok(graph) => {
                let typ = self.graph_type(&graph, &graph.roots()[0])?;
                let header = self.method_header(r, selector, &typ, true)?;
                Ok(Step::More(Plan::MethodFirst(Box::new(header), args)))
            }
            Err(GoSourceTypeError::NotType | GoSourceTypeError::Unsupported) => Ok(Step::More(
                Plan::MethodReceiver(Request::raw(&r.source, base), selector, args),
            )),
            Err(error) => Err(GoScalarTypingError::SourceType(error)),
        }
    }
    pub(super) fn method_header(
        &mut self,
        r: &Request,
        selector: usize,
        typ: &str,
        expression: bool,
    ) -> Result<GoMethodCallHeader> {
        let n = self.node(&r.source, selector)?.clone();
        let name = attr(self.node(&r.source, child(&n, "Sel")?)?, "Name")
            .ok_or(GoScalarTypingError::Locator)?
            .to_string();
        let package = self
            .scopes
            .source_package(&r.source)
            .ok_or(GoScalarTypingError::Locator)?
            .to_string();
        let Member::Method(candidate, path) = self.lookup_method_member(typ, &name, &package)?
        else {
            return Err(GoScalarTypingError::Type);
        };
        if candidate.pointer() {
            return Err(GoScalarTypingError::Unsupported);
        }
        let (method, signature) = self.source_method(&candidate, path)?;
        let signature = signature.ok_or(GoScalarTypingError::Unsupported)?;
        self.graph_type(signature.types(), &method.receiver)?;
        let mut parameters = vec![];
        if expression {
            parameters.push(typ.into())
        }
        parameters.extend(method.parameters.iter().cloned());
        Ok(GoMethodCallHeader {
            source: r.source.clone(),
            call: r.expression()?,
            function: child(self.node(&r.source, r.expression()?)?, "Fun")?,
            receiver_node: child(&n, "X")?,
            selected_receiver: typ.into(),
            expression,
            method,
            parameters,
        })
    }
    pub(super) fn method_first(
        &mut self,
        header: GoMethodCallHeader,
        args: Vec<Request>,
    ) -> Result<Step> {
        let facts = self.expand(&args)?;
        if facts.len() != header.parameters.len() {
            return Err(GoScalarTypingError::Arity);
        }
        if args.len() == 1 && facts.len() != 1 {
            self.check_types(&facts, &header.parameters)?;
            return self.method_result(header);
        }
        let next = args
            .iter()
            .zip(&header.parameters)
            .map(|(r, t)| Ok(Request::value(&r.source, r.expression()?, Some(t.clone()))))
            .collect::<Result<Vec<_>>>()?;
        Ok(Step::More(Plan::MethodReady(Box::new(header), next)))
    }
    pub(super) fn method_result(&mut self, header: GoMethodCallHeader) -> Result<Step> {
        let facts = header
            .method
            .results
            .iter()
            .map(|t| self.atom(t, false))
            .collect();
        self.method_calls.insert(header);
        Ok(Step::Done(facts))
    }
}
