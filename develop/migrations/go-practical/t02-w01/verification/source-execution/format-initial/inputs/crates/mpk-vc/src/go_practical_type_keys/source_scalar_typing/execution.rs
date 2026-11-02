//! Independent execution of original checked source. This untrusted observer
//! grants no proof, profile, IR correspondence or universal evaluation-order
//! authority. The explicit observation budget is not a Go semantic limit.
use super::*;
use std::cell::RefCell;
use std::rc::Rc;

mod expressions;
mod statements;
mod values;
pub use values::{GoSourceRuntimeData, GoSourceRuntimeSlice, GoSourceRuntimeValue};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceRuntimeFault {
    IndexBounds,
    SliceBounds,
    DivideByZero,
    NegativeShift,
    MakeBounds,
    NilInterfaceCall,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceExecutionErrorKind {
    Entry,
    Input,
    Source,
    Unavailable,
    HostCapacity,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoSourceExecutionStatus {
    Returned(Vec<GoSourceRuntimeValue>),
    Fault(GoSourceRuntimeFault),
    ObservationLimit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoSourceExecutionEventKind {
    Expression,
    Enter,
    Return,
    Statement,
    Write,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceExecutionEvent {
    pub origin: GoSourceSelectionRef,
    pub invocation: u64,
    pub kind: GoSourceExecutionEventKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceExecution {
    pub status: GoSourceExecutionStatus,
    pub stopped_at: GoSourceSelectionRef,
    pub steps: u64,
    pub events: Vec<GoSourceExecutionEvent>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceExecutionError {
    pub kind: GoSourceExecutionErrorKind,
    pub stopped_at: GoSourceSelectionRef,
    pub steps: u64,
    pub events: Vec<GoSourceExecutionEvent>,
}
impl std::fmt::Display for GoSourceExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Go source execution unavailable at {:?}: {:?}", self.stopped_at, self.kind)
    }
}
impl std::error::Error for GoSourceExecutionError {}

enum Halt {
    Fault(GoSourceRuntimeFault),
    Limit,
    Error(GoSourceExecutionErrorKind),
}
type Run<T> = std::result::Result<T, Halt>;
fn source() -> Halt { Halt::Error(GoSourceExecutionErrorKind::Source) }
fn unavailable() -> Halt { Halt::Error(GoSourceExecutionErrorKind::Unavailable) }
type Binding = (String, usize, usize);
fn binding(b: &GoSourceNameBinding) -> Binding {
    (b.source_object_id().into(), b.declaration_node(), b.name_node())
}
struct Frame {
    root: GoSourceSelectionRef,
    invocation: u64,
    locals: BTreeMap<Binding, GoSourceRuntimeValue>,
    results: Vec<(String, Option<Binding>)>,
}
enum Flow { Normal, Return(Vec<GoSourceRuntimeValue>), Break, Continue }
struct Machine<'a> {
    domain: &'a GoSourceValueDomain,
    engine: Engine<'a>,
    budget: u64,
    steps: u64,
    invocations: u64,
    at: GoSourceSelectionRef,
    events: Vec<GoSourceExecutionEvent>,
}
impl Machine<'_> {
    fn tick(&mut self, frame: &Frame, node: usize, kind: GoSourceExecutionEventKind) -> Run<()> {
        self.at=GoSourceSelectionRef::new(frame.root.source_object_id(),node);
        if self.steps==self.budget { return Err(Halt::Limit); }
        self.steps+=1;
        self.events.try_reserve(1).map_err(|_| Halt::Error(GoSourceExecutionErrorKind::HostCapacity))?;
        self.events.push(GoSourceExecutionEvent{origin:self.at.clone(),invocation:frame.invocation,kind});
        Ok(())
    }
    fn node(&self, frame: &Frame, node: usize) -> Run<GoSyntaxNode> {
        self.engine.node(frame.root.source_object_id(),node).cloned().map_err(|_| source())
    }
    fn fact(&mut self, frame: &Frame, node: usize, expected: Option<&str>) -> Run<Atom> {
        let r=Request::value(frame.root.source_object_id(),node,expected.map(str::to_string));
        self.engine.compute(r.clone()).map_err(|_| source())?;
        self.engine.one(&r).map_err(|_| source())
    }
    fn invoke(&mut self, root: GoSourceSelectionRef, args: Vec<GoSourceRuntimeValue>) -> Run<Vec<GoSourceRuntimeValue>> {
        self.at=root.clone();
        if self.domain.selection().checked_body_types(&root).is_none() { return Err(source()); }
        let signature=derive_go_source_signature(self.engine.scopes,root.source_object_id(),root.node()).map_err(|_| source())?;
        for id in signature.types().roots() { self.engine.graph_type(signature.types(),id).map_err(|_| source())?; }
        let slots=signature.slots().iter().filter(|s|s.role()!=GoSourceSignatureRole::Result).collect::<Vec<_>>();
        if slots.len()!=args.len() { return Err(Halt::Error(GoSourceExecutionErrorKind::Input)); }
        let mut frame=Frame{root:root.clone(),invocation:self.invocations,locals:BTreeMap::new(),results:vec![]};
        self.invocations=self.invocations.checked_add(1).ok_or(Halt::Limit)?;
        self.tick(&frame,root.node(),GoSourceExecutionEventKind::Enter)?;
        for (slot,arg) in slots.into_iter().zip(args) {
            if arg.type_id()!=slot.type_id() || !self.valid(&arg) { return Err(Halt::Error(GoSourceExecutionErrorKind::Input)); }
            if let Some(b)=self.slot_binding(&signature,slot) { frame.locals.insert(binding(&b),arg); }
        }
        for slot in signature.slots().iter().filter(|s|s.role()==GoSourceSignatureRole::Result) {
            let b=self.slot_binding(&signature,slot).map(|b|binding(&b));
            if let Some(b)=&b { frame.locals.insert(b.clone(),self.zero(slot.type_id())?); }
            frame.results.push((slot.type_id().into(),b));
        }
        let values=match self.statement(&mut frame,signature.body_node().ok_or_else(source)?)? {
            Flow::Return(values)=>values,
            Flow::Normal if frame.results.is_empty()=>vec![],
            _=>return Err(source()),
        };
        if values.len()!=frame.results.len() || values.iter().zip(&frame.results).any(|(v,(t,_))|v.type_id()!=t || !self.valid(v)) { return Err(source()); }
        self.tick(&frame,root.node(),GoSourceExecutionEventKind::Return)?;
        Ok(values)
    }
    fn slot_binding(&self, signature:&GoSourceSignature, slot:&GoSourceSignatureSlot)->Option<GoSourceNameBinding> {
        let name=slot.name_node()?;
        self.engine.scopes.bindings().iter().find(|b|b.source_object_id()==signature.source_object_id() && b.declaration_node()==signature.function_node() && b.name_node()==name && b.name()!="_").cloned()
    }
}

/// Execute the actual selected entry and its checked original callees. No
/// native results, caller evaluator hooks, arbitrary callee or proof is used.
/// A run records one source-permitted evaluation choice; it is not a universal
/// theorem about expressions with interfering reads and writes. Such source
/// admission/effect/order proofs remain required by the frozen profile.
pub fn execute_go_source(
    domain:&GoSourceValueDomain,
    entry:&GoSourceSelectionRef,
    arguments:Vec<GoSourceRuntimeValue>,
    observation_steps:u64,
)->std::result::Result<GoSourceExecution,GoSourceExecutionError> {
    let mut machine=Machine{domain,engine:Engine::new(domain.selection().scopes()),budget:observation_steps,steps:0,invocations:0,at:entry.clone(),events:vec![]};
    let result=if !domain.selection().roots().contains(entry) { Err(Halt::Error(GoSourceExecutionErrorKind::Entry)) } else { machine.invoke(entry.clone(),arguments) };
    let status=match result {
        Ok(values)=>GoSourceExecutionStatus::Returned(values),
        Err(Halt::Fault(f))=>GoSourceExecutionStatus::Fault(f),
        Err(Halt::Limit)=>GoSourceExecutionStatus::ObservationLimit,
        Err(Halt::Error(kind))=>return Err(GoSourceExecutionError{kind,stopped_at:machine.at,steps:machine.steps,events:machine.events}),
    };
    Ok(GoSourceExecution{status,stopped_at:machine.at,steps:machine.steps,events:machine.events})
}
