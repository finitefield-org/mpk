//! Original make/append/copy type facts, without allocation/access/state proof.
use super::*;
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct GoSliceBuiltinHeader {
    source: String,
    call: usize,
    function: usize,
    name: String,
    arguments: Vec<usize>,
    slice_type: String,
    element_type: String,
    result_type: String,
    ellipsis: bool,
    explicit_capacity: bool,
}
impl GoSliceBuiltinHeader {
    pub fn source_object_id(&self) -> &str {
        &self.source
    }
    pub fn call_node(&self) -> usize {
        self.call
    }
    pub fn function_node(&self) -> usize {
        self.function
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn argument_nodes(&self) -> &[usize] {
        &self.arguments
    }
    pub fn slice_type_id(&self) -> &str {
        &self.slice_type
    }
    pub fn element_type_id(&self) -> &str {
        &self.element_type
    }
    pub fn result_type_id(&self) -> &str {
        &self.result_type
    }
    pub fn ellipsis(&self) -> bool {
        self.ellipsis
    }
    pub fn explicit_capacity(&self) -> bool {
        self.explicit_capacity
    }
}
impl Engine<'_> {
    pub(super) fn slice_builtin_plan(
        &mut self,
        source: &str,
        call: usize,
        name: &str,
        dots: bool,
    ) -> Result<Plan> {
        let n = self.node(source, call)?.clone();
        let args = list(&n, "Args");
        match name {
            "make" => {
                if dots || !(2..=3).contains(&args.len()) {
                    return Err(GoScalarTypingError::Arity);
                }
                let typ = self.annotation(source, args[0])?;
                self.slice_element(&typ)?;
                Ok(Plan::Make(
                    typ,
                    args[1..]
                        .iter()
                        .map(|&n| Request::index(source, n))
                        .collect(),
                ))
            }
            "append" => Ok(Plan::AppendFirst(
                args.iter().map(|&n| Request::raw(source, n)).collect(),
                dots,
            )),
            "copy" => {
                if dots {
                    return Err(GoScalarTypingError::Arity);
                }
                Ok(Plan::Copy(
                    args.iter().map(|&n| Request::raw(source, n)).collect(),
                ))
            }
            _ => Err(GoScalarTypingError::Unsupported),
        }
    }
    fn slice_element(&self, typ: &str) -> Result<String> {
        match self.shape(typ)? {
            GoSourceTypeKey::Slice { element, .. } => Ok(element.clone()),
            _ => Err(GoScalarTypingError::Type),
        }
    }
    fn byte_element(&self, typ: &str) -> bool {
        matches!(self.keys.get(typ),Some(GoSourceTypeKey::Builtin{name,..})if name=="uint8")
    }
    fn slice_result(&mut self, r: &Request, name: &str, typ: String) -> Result<Step> {
        let node = r.expression()?;
        let call = self.node(&r.source, node)?;
        let arguments = list(call, "Args").to_vec();
        let function = child(call, "Fun")?;
        let ellipsis = attr(call, "Ellipsis") == Some("true");
        let element_type = self.slice_element(&typ)?;
        let result_type = if name == "copy" {
            self.builtin("int")?
        } else {
            typ.clone()
        };
        self.slice_builtins.insert(GoSliceBuiltinHeader {
            source: r.source.clone(),
            call: node,
            function,
            name: name.into(),
            explicit_capacity: name == "make" && arguments.len() == 3,
            arguments,
            slice_type: typ,
            element_type,
            result_type: result_type.clone(),
            ellipsis,
        });
        Ok(Step::Done(vec![self.atom(&result_type, false)]))
    }
    pub(super) fn make_slice(
        &mut self,
        r: &Request,
        typ: String,
        args: Vec<Request>,
    ) -> Result<Step> {
        let mut sizes = vec![];
        for arg in args {
            sizes.push(self.index_number(&self.one(&arg)?)?)
        }
        if sizes.len() == 2 && sizes[0].zip(sizes[1]).is_some_and(|(n, c)| n > c) {
            return Err(GoScalarTypingError::Type);
        }
        self.slice_result(r, "make", typ)
    }
    pub(super) fn append_first(
        &mut self,
        r: &Request,
        args: Vec<Request>,
        dots: bool,
    ) -> Result<Step> {
        let facts = self.expand(&args)?;
        let first = facts.first().ok_or(GoScalarTypingError::Arity)?;
        let typ = first.typ.clone().ok_or(GoScalarTypingError::Type)?;
        let element = self.slice_element(&typ)?;
        // Go1.25 handles the byte/string form before constructing the ordinary
        // variadic signature. It uses the actual expanded operand list.
        if dots
            && facts.len() == 2
            && self.byte_element(&element)
            && facts[1].kind == Some(GoConstantKind::String)
        {
            return self.slice_result(r, "append", typ);
        }
        if dots && (args.len() != 2 || facts.len() != 2) {
            return Err(GoScalarTypingError::Arity);
        }
        let target = if dots {
            self.aggregate_key(None, element)?
        } else {
            element
        };
        let mut targets = vec![typ.clone()];
        targets.extend((1..facts.len()).map(|_| target.clone()));
        if args.len() == 1 && facts.len() != 1 {
            self.check_types(&facts, &targets)?;
            return self.slice_result(r, "append", typ);
        }
        let next = args
            .iter()
            .zip(targets)
            .map(|(arg, t)| Ok(Request::value(&arg.source, arg.expression()?, Some(t))))
            .collect::<Result<Vec<_>>>()?;
        Ok(Step::More(Plan::AppendReady(typ, next, dots)))
    }
    pub(super) fn append_ready(
        &mut self,
        r: &Request,
        typ: String,
        args: Vec<Request>,
        dots: bool,
    ) -> Result<Step> {
        let facts = self.expand(&args)?;
        let element = self.slice_element(&typ)?;
        let target = if dots {
            self.aggregate_key(None, element)?
        } else {
            element
        };
        let mut targets = vec![typ.clone()];
        targets.extend((1..facts.len()).map(|_| target.clone()));
        self.check_types(&facts, &targets)?;
        self.slice_result(r, "append", typ)
    }
    pub(super) fn copy_slice(&mut self, r: &Request, args: Vec<Request>) -> Result<Step> {
        let facts = self.expand(&args)?;
        if facts.len() != 2 {
            return Err(GoScalarTypingError::Arity);
        }
        // The string special form requires a destination assignable to []byte;
        // unlike append's first slice operand, that includes untyped nil.
        if facts[0].typ.is_none() && facts[0].nil && facts[1].kind == Some(GoConstantKind::String) {
            if args.len() != 2 {
                return Err(GoScalarTypingError::Arity);
            }
            let byte = self.builtin("uint8")?;
            let slice = self.aggregate_key(None, byte)?;
            return Ok(Step::More(Plan::Copy(vec![
                Request::value(&args[0].source, args[0].expression()?, Some(slice)),
                args[1].clone(),
            ])));
        }
        let typ = facts[0].typ.clone().ok_or(GoScalarTypingError::Type)?;
        let element = self.slice_element(&typ)?;
        let source = &facts[1];
        if source.kind == Some(GoConstantKind::String) {
            if !self.byte_element(&element) {
                return Err(GoScalarTypingError::Type);
            }
        } else if self.slice_element(source.typ.as_deref().ok_or(GoScalarTypingError::Type)?)?
            != element
        {
            return Err(GoScalarTypingError::Type);
        }
        self.slice_result(r, "copy", typ)
    }
}
