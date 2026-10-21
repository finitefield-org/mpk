//! Flat dependency scheduling across original constant, value and type readers.
//! Sessions are thread-local, own no external callbacks and cache only facts
//! derived from the one immutable original source universe supplied by entry.
use super::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(super) struct Read {
    pub source: String,
    pub node: usize,
    pub definition_source: String,
    pub declaration: usize,
    pub name: usize,
}
pub(super) type Reads = BTreeSet<Read>;
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Context {
    source: String,
    spec: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Kind {
    Expression(usize),
    Binding(usize),
    Spec(usize),
    Measure(usize),
    Types(Vec<usize>),
    Declaration(usize),
}
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Query {
    source: String,
    context: Option<Context>,
    kind: Kind,
}
#[derive(Clone)]
enum Value {
    Constant(GoEvaluatedConstant),
    Constants(Vec<GoEvaluatedConstant>),
    Types(GoDerivedSourceTypes),
}
#[derive(Clone, Copy)]
enum Failure {
    Constant(GoConstantError),
    Type(GoSourceTypeError),
}
#[derive(Clone)]
struct Reply {
    value: std::result::Result<Value, Failure>,
    reads: Reads,
}
struct State {
    owner: usize,
    sources: Arc<GoSourceScopes>,
    cache: BTreeMap<Query, Reply>,
    active: BTreeSet<Query>,
    pending: Option<Query>,
    current: Option<Query>,
    reads: Reads,
    contexts: BTreeMap<Context, BTreeSet<usize>>,
}
thread_local! { static SESSION: RefCell<Option<Rc<RefCell<State>>>> = const { RefCell::new(None) }; }
fn session(scopes: &GoSourceScopes) -> Option<Rc<RefCell<State>>> {
    SESSION.with(|slot| {
        slot.borrow()
            .as_ref()
            .filter(|s| s.borrow().owner == scopes as *const GoSourceScopes as usize)
            .cloned()
    })
}
pub(super) fn shared_sources(scopes: &GoSourceScopes) -> Arc<GoSourceScopes> {
    session(scopes).map_or_else(|| Arc::new(scopes.clone()), |s| s.borrow().sources.clone())
}
pub(super) fn context(scopes: &GoSourceScopes, source: &str, node: usize) -> Option<usize> {
    let s = session(scopes)?;
    let state = s.borrow();
    let context = state.current.as_ref()?.context.as_ref()?;
    (context.source == source && state.contexts.get(context)?.contains(&node))
        .then_some(context.spec)
}
fn query(scopes: &GoSourceScopes, source: &str, kind: Kind) -> Query {
    // Only reused syntax belongs to the inherited ConstSpec context. A query
    // for a referenced declaration starts in that declaration's own scope.
    let context = session(scopes).and_then(|s| {
        let state = s.borrow();
        let current = state.current.as_ref()?.context.as_ref()?;
        if current.source != source {
            return None;
        }
        let nodes = state.contexts.get(current)?;
        let contextual = match &kind {
            Kind::Expression(node) | Kind::Measure(node) => nodes.contains(node),
            Kind::Types(types) => types.iter().any(|node| nodes.contains(node)),
            Kind::Binding(_) | Kind::Spec(_) | Kind::Declaration(_) => false,
        };
        contextual.then(|| current.clone())
    });
    Query {
        source: source.into(),
        context,
        kind,
    }
}
fn cycle(q: &Query) -> Reply {
    let failure = match q.kind {
        Kind::Types(_) | Kind::Declaration(_) => Failure::Type(GoSourceTypeError::Cycle),
        _ => Failure::Constant(GoConstantError::Cycle),
    };
    Reply {
        value: Err(failure),
        reads: Reads::new(),
    }
}
fn probe(s: &Rc<RefCell<State>>, q: Query) -> Reply {
    let mut state = s.borrow_mut();
    if let Some(reply) = state.cache.get(&q).cloned() {
        state.reads.extend(reply.reads.clone());
        return reply;
    }
    if state.pending.is_none() && !state.active.contains(&q) {
        state.pending = Some(q.clone());
    }
    cycle(&q)
}
struct Guard(Option<Rc<RefCell<State>>>);
impl Drop for Guard {
    fn drop(&mut self) {
        SESSION.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}
fn run(scopes: &GoSourceScopes, root: Query) -> Reply {
    if let Some(s) = session(scopes) {
        return probe(&s, root);
    }
    let state = Rc::new(RefCell::new(State {
        owner: scopes as *const GoSourceScopes as usize,
        sources: Arc::new(scopes.clone()),
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
        pending: None,
        current: None,
        reads: Reads::new(),
        contexts: BTreeMap::new(),
    }));
    let previous = SESSION.with(|slot| slot.borrow_mut().replace(state.clone()));
    let _guard = Guard(previous);
    let mut work = vec![root.clone()];
    while let Some(q) = work.last().cloned() {
        if state.borrow().cache.contains_key(&q) {
            work.pop();
            continue;
        }
        {
            let mut s = state.borrow_mut();
            s.active.insert(q.clone());
            s.current = Some(q.clone());
            s.pending = None;
            s.reads.clear();
        }
        let value = dispatch(scopes, &q);
        let mut s = state.borrow_mut();
        if let Some(next) = s.pending.take() {
            drop(s);
            work.push(next);
            continue;
        }
        let reads = std::mem::take(&mut s.reads);
        let value = value.map(|value| match value {
            Value::Types(mut types) => {
                types.compilation_reads.extend(reads.clone());
                Value::Types(types)
            }
            other => other,
        });
        s.cache.insert(q.clone(), Reply { value, reads });
        s.active.remove(&q);
        work.pop();
    }
    let reply = state.borrow().cache[&root].clone();
    reply
}
fn dispatch(scopes: &GoSourceScopes, q: &Query) -> std::result::Result<Value, Failure> {
    match &q.kind {
        Kind::Expression(node) => {
            source_constants::evaluate::expression_raw(scopes, &q.source, *node)
                .map(Value::Constant)
                .map_err(Failure::Constant)
        }
        Kind::Binding(index) => scopes
            .bindings()
            .get(*index)
            .ok_or(Failure::Constant(GoConstantError::Locator))
            .and_then(|b| {
                source_constants::evaluate::binding_raw(scopes, b)
                    .map(Value::Constant)
                    .map_err(Failure::Constant)
            }),
        Kind::Spec(node) => source_constants::evaluate::declaration_raw(scopes, &q.source, *node)
            .map(Value::Constants)
            .map_err(Failure::Constant),
        Kind::Measure(node) => source_scalar_typing::constant_measure(scopes, &q.source, *node)
            .map(|(value, reads)| {
                if let Some(s) = session(scopes) {
                    s.borrow_mut().reads.extend(reads);
                }
                Value::Constant(value)
            })
            .map_err(Failure::Constant),
        Kind::Types(nodes) => source_types::derive_nodes_raw(scopes, &q.source, nodes)
            .map(Value::Types)
            .map_err(Failure::Type),
        Kind::Declaration(node) => source_types::derive_declaration_raw(scopes, &q.source, *node)
            .map(Value::Types)
            .map_err(Failure::Type),
    }
}
fn constant(reply: Reply) -> std::result::Result<(GoEvaluatedConstant, Reads), GoConstantError> {
    match reply.value {
        Ok(Value::Constant(v)) => Ok((v, reply.reads)),
        Err(Failure::Constant(e)) => Err(e),
        Err(Failure::Type(_)) | Ok(_) => Err(GoConstantError::Type),
    }
}
pub(super) fn expression(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
) -> std::result::Result<(GoEvaluatedConstant, Reads), GoConstantError> {
    constant(run(scopes, query(scopes, source, Kind::Expression(node))))
}
pub(super) fn binding(
    scopes: &GoSourceScopes,
    binding: &GoSourceNameBinding,
) -> std::result::Result<(GoEvaluatedConstant, Reads), GoConstantError> {
    let index = scopes
        .bindings()
        .iter()
        .position(|b| b == binding && b.kind() == GoSourceNameKind::Constant)
        .ok_or(GoConstantError::Locator)?;
    constant(run(
        scopes,
        query(scopes, binding.source_object_id(), Kind::Binding(index)),
    ))
}
pub(super) fn declaration(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
) -> std::result::Result<(Vec<GoEvaluatedConstant>, Reads), GoConstantError> {
    let reply = run(scopes, query(scopes, source, Kind::Spec(node)));
    match reply.value {
        Ok(Value::Constants(v)) => Ok((v, reply.reads)),
        Err(Failure::Constant(e)) => Err(e),
        _ => Err(GoConstantError::Type),
    }
}
pub(super) fn measure(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
    context: Option<usize>,
) -> std::result::Result<GoEvaluatedConstant, GoConstantError> {
    let mut q = query(scopes, source, Kind::Measure(node));
    if let Some(spec) = context {
        let context = Context {
            source: source.into(),
            spec,
        };
        if let Some(s) = session(scopes) {
            if !s.borrow().contexts.contains_key(&context) {
                let nodes = source_constants::evaluate::context_nodes(scopes, source, spec)?;
                s.borrow_mut().contexts.insert(context.clone(), nodes);
            }
        }
        q.context = Some(context);
    }
    constant(run(scopes, q)).map(|r| r.0)
}
fn types(reply: Reply) -> std::result::Result<GoDerivedSourceTypes, GoSourceTypeError> {
    match reply.value {
        Ok(Value::Types(types)) => Ok(types),
        Err(Failure::Type(e)) => Err(e),
        _ => Err(GoSourceTypeError::Constant),
    }
}
pub(super) fn type_nodes(
    scopes: &GoSourceScopes,
    source: &str,
    nodes: &[usize],
) -> std::result::Result<GoDerivedSourceTypes, GoSourceTypeError> {
    types(run(
        scopes,
        query(scopes, source, Kind::Types(nodes.to_vec())),
    ))
}
pub(super) fn type_declaration(
    scopes: &GoSourceScopes,
    source: &str,
    node: usize,
) -> std::result::Result<GoDerivedSourceTypes, GoSourceTypeError> {
    types(run(scopes, query(scopes, source, Kind::Declaration(node))))
}
