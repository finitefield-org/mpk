//! Test-only call-by-need evaluation of ordinary Boolean core terms.
//! This supplies observations, never proof or certificate acceptance.
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeSet, HashMap, VecDeque},
    rc::{Rc, Weak},
    time::{Duration, Instant},
};
// Environment edges share ownership but release unreachable chains through a
// worklist. Evaluation already uses an explicit stack; Rust destructor recursion
// must not reintroduce a native-stack limit when a suspension is evaluated.
#[derive(Clone)]
pub(super) struct EnvRef(Option<Rc<Env>>);
impl EnvRef {
    pub(super) fn new(env: Env) -> Self {
        Self(Some(Rc::new(env)))
    }
}
impl From<Rc<Env>> for EnvRef {
    fn from(env: Rc<Env>) -> Self {
        Self(Some(env))
    }
}
impl std::ops::Deref for EnvRef {
    type Target = Env;
    fn deref(&self) -> &Env {
        self.0.as_ref().expect("live environment edge")
    }
}
thread_local! {
    static ENV_RELEASE: RefCell<(bool, Vec<Env>)> = const { RefCell::new((false, Vec::new())) };
    // Optional diagnostic ceiling over *all* evaluator calls on this thread,
    // including successive output-bit observations. Exhaustion is a failure,
    // never a passing/skipped assertion. No limit is imposed by default.
    static EVAL_REMAINING: Cell<Option<u64>> = Cell::new(
        std::env::var("MPK_CORE_MAX_STEPS").ok().map(|s| {
            s.parse::<u64>().ok().filter(|n| *n > 0)
                .expect("MPK_CORE_MAX_STEPS must be a positive integer")
        })
    );
    static EVAL_DEADLINE: Option<Instant> = std::env::var("MPK_CORE_MAX_SECONDS").ok().map(|s| {
        let seconds = s.parse::<u64>().ok().filter(|n| *n > 0)
            .expect("MPK_CORE_MAX_SECONDS must be a positive integer");
        Instant::now().checked_add(Duration::from_secs(seconds))
            .expect("MPK_CORE_MAX_SECONDS exceeds the clock range")
    });
}
impl Drop for EnvRef {
    fn drop(&mut self) {
        let Some(edge) = self.0.take() else {
            return;
        };
        let Ok(env) = Rc::try_unwrap(edge) else {
            return;
        };
        ENV_RELEASE.with(|release| {
            {
                let mut state = release.borrow_mut();
                state.1.push(env);
                if state.0 {
                    return;
                }
                state.0 = true;
            }
            loop {
                let next = release.borrow_mut().1.pop();
                let Some(env) = next else {
                    break;
                };
                // No queue borrow is held. Nested environment drops append to
                // the same queue instead of recursing through captured values.
                drop(env);
            }
            release.borrow_mut().0 = false;
        });
    }
}
// A binding adds one shared node rather than copying every captured value.
// De Bruijn lookup keeps the newest binding at index zero.
pub(super) enum Env {
    Empty,
    Bind(V, EnvRef),
}
impl Env {
    fn get(&self, mut index: usize) -> V {
        let mut env = self;
        loop {
            match env {
                Self::Bind(value, _) if index == 0 => return value.clone(),
                Self::Bind(_, tail) => {
                    index -= 1;
                    env = tail;
                }
                Self::Empty => panic!("unbound core variable"),
            }
        }
    }
}
// Free-variable information describes runtime terms only: lambda/let types
// are erased by this observer. High variables are conservatively unknown rather
// than truncated, so arbitrarily deep binders cannot produce a false "unused".
#[derive(Clone, Copy, Default)]
struct Usage {
    low: u128,
    high: bool,
}
impl Usage {
    fn unknown() -> Self {
        Self { low: 0, high: true }
    }
    fn union(self, other: Self) -> Self {
        Self {
            low: self.low | other.low,
            high: self.high || other.high,
        }
    }
    fn under_binder(self) -> Self {
        Self {
            low: self.low >> 1,
            high: self.high,
        }
    }
}
#[derive(Default)]
struct UsageAnalysis(HashMap<u32, Usage>);
impl UsageAnalysis {
    // Generated terms refer backward in their DAG. Treat malformed/forward
    // references conservatively; analysis must never demand a poison argument.
    fn usage(&mut self, c: &Certificate, root: u32) -> Usage {
        if let Some(usage) = self.0.get(&root) {
            return *usage;
        }
        let mut pending = vec![(root, false)];
        while let Some((term, ready)) = pending.pop() {
            if self.0.contains_key(&term) {
                continue;
            }
            let Some(node) = c.term_table.get(term as usize) else {
                self.0.insert(term, Usage::unknown());
                continue;
            };
            let children = match node {
                TermNode::Lam { body, .. } => vec![*body],
                TermNode::Let { value, body, .. } => vec![*value, *body],
                TermNode::App {
                    function,
                    arguments,
                } => std::iter::once(*function)
                    .chain(arguments.iter().copied())
                    .collect(),
                _ => vec![],
            };
            if !ready {
                pending.push((term, true));
                pending.extend(
                    children
                        .into_iter()
                        .filter(|child| *child < term)
                        .map(|child| (child, false)),
                );
                continue;
            }
            let child = |id: u32| {
                if id < term {
                    self.0.get(&id).copied().unwrap_or_else(Usage::unknown)
                } else {
                    Usage::unknown()
                }
            };
            let usage = match node {
                TermNode::Var(i) if *i < 128 => Usage {
                    low: 1u128 << i,
                    high: false,
                },
                TermNode::Const { .. } => Usage::default(), // definitions have their own empty environment
                TermNode::Lam { body, .. } => child(*body).under_binder(),
                TermNode::Let { value, body, .. } => {
                    child(*value).union(child(*body).under_binder())
                }
                TermNode::App {
                    function,
                    arguments,
                } => arguments
                    .iter()
                    .fold(child(*function), |usage, id| usage.union(child(*id))),
                _ => Usage::unknown(),
            };
            self.0.insert(term, usage);
        }
        self.0[&root]
    }
}
pub(super) struct LambdaMemo {
    by_bool: [Option<V>; 2],
    independent: Option<V>,
    argument_unused: bool,
    usage: Rc<RefCell<UsageAnalysis>>,
    by_identity: VecDeque<(WeakValue, WeakValue)>,
    observations: Rc<RefCell<BitObservations>>,
}
// Retain only demanded scalar leaves, never closures or environments. A path
// can cover the maximum scalar circuit cube (18 selectors), while the shared
// FIFO bounds each originating lambda to 1,024 observed leaves. Intermediate
// selector closures carry a cheap view into that table, so dropping/rebuilding
// them does not lose deep observations or allocate a full Boolean trie.
struct BitObservations {
    path: u32,
    depth: u8,
    leaves: Rc<RefCell<ObservedBits>>,
}
#[derive(Default)]
struct ObservedBits {
    values: HashMap<(u32, u8), bool>,
    order: VecDeque<(u32, u8)>,
}
impl BitObservations {
    const MAX_DEPTH: u8 = 18;
    const CAPACITY: usize = 1024;
    fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            path: 0,
            depth: 0,
            leaves: Rc::new(RefCell::new(ObservedBits::default())),
        }))
    }
    fn child(&self, key: bool) -> Option<Rc<RefCell<Self>>> {
        (self.depth < Self::MAX_DEPTH).then(|| {
            Rc::new(RefCell::new(Self {
                path: self.path | (u32::from(key) << self.depth),
                depth: self.depth + 1,
                leaves: self.leaves.clone(),
            }))
        })
    }
    fn bit(&self) -> Option<bool> {
        self.leaves
            .borrow()
            .values
            .get(&(self.path, self.depth))
            .copied()
    }
    fn remember(&self, bit: bool) {
        let key = (self.path, self.depth);
        let mut leaves = self.leaves.borrow_mut();
        if !leaves.values.contains_key(&key) {
            if leaves.values.len() == Self::CAPACITY {
                let oldest = leaves
                    .order
                    .pop_front()
                    .expect("bounded scalar observations");
                leaves.values.remove(&oldest);
            }
            leaves.order.push_back(key);
        }
        leaves.values.insert(key, bit);
    }
}
type BoolMemo = Rc<RefCell<LambdaMemo>>;
#[derive(Clone)]
pub(super) enum Suspension {
    Pending(u32, EnvRef),
    Evaluated(V),
}
#[derive(Clone)]
pub(super) enum V {
    Bit(bool),
    // Complete constant truth tables, not assumptions about unused storage.
    UniformCube(bool, u32),
    Cube(Vec<bool>),
    SharedCube(Rc<Vec<bool>>, usize, usize),
    SparseCube(Rc<BTreeSet<usize>>, usize, usize, usize),
    Lambda(u32, EnvRef, BoolMemo),
    Thunk(Rc<RefCell<Suspension>>),
    Rec(Vec<V>),
}
// Sharing a function argument is safe only while the original value remains
// alive. Weak edges prevent address reuse from aliasing a different argument,
// and avoid cycles when a returned closure captures the memoized function.
// No lookup forces a suspension; its identity remains stable after forcing.
enum WeakValue {
    Bit(bool),
    UniformCube(bool, u32),
    Lambda(u32, Weak<Env>, Weak<RefCell<LambdaMemo>>),
    Thunk(Weak<RefCell<Suspension>>),
    Cube(Weak<Vec<bool>>, usize, usize),
    Sparse(Weak<BTreeSet<usize>>, usize, usize, usize),
}
impl WeakValue {
    fn of(value: &V) -> Option<Self> {
        Some(match value {
            V::Bit(b) => Self::Bit(*b),
            V::UniformCube(bit, depth) => Self::UniformCube(*bit, *depth),
            V::Lambda(body, env, memo) => Self::Lambda(
                *body,
                Rc::downgrade(env.0.as_ref().expect("live environment")),
                Rc::downgrade(memo),
            ),
            V::Thunk(memo) => Self::Thunk(Rc::downgrade(memo)),
            V::SharedCube(bits, offset, stride) => {
                Self::Cube(Rc::downgrade(bits), *offset, *stride)
            }
            V::SparseCube(bits, offset, stride, length) => {
                Self::Sparse(Rc::downgrade(bits), *offset, *stride, *length)
            }
            V::Cube(_) | V::Rec(_) => return None,
        })
    }
    fn upgrade(&self) -> Option<V> {
        Some(match self {
            Self::Bit(b) => V::Bit(*b),
            Self::UniformCube(bit, depth) => V::UniformCube(*bit, *depth),
            Self::Lambda(body, env, memo) => {
                V::Lambda(*body, env.upgrade()?.into(), memo.upgrade()?)
            }
            Self::Thunk(memo) => V::Thunk(memo.upgrade()?),
            Self::Cube(bits, offset, stride) => V::SharedCube(bits.upgrade()?, *offset, *stride),
            Self::Sparse(bits, offset, stride, length) => {
                V::SparseCube(bits.upgrade()?, *offset, *stride, *length)
            }
        })
    }
    fn matches(&self, value: &V) -> bool {
        match (self, value) {
            (Self::Bit(a), V::Bit(b)) => a == b,
            (Self::UniformCube(a, ad), V::UniformCube(b, bd)) => a == b && ad == bd,
            (Self::Lambda(a, ae, am), V::Lambda(b, be, bm)) => {
                a == b
                    && ae.as_ptr() == Rc::as_ptr(be.0.as_ref().expect("live environment"))
                    && am.as_ptr() == Rc::as_ptr(bm)
            }
            (Self::Thunk(a), V::Thunk(b)) => a.as_ptr() == Rc::as_ptr(b),
            (Self::Cube(a, ao, ast), V::SharedCube(b, bo, bst)) => {
                a.as_ptr() == Rc::as_ptr(b) && ao == bo && ast == bst
            }
            (Self::Sparse(a, ao, ast, al), V::SparseCube(b, bo, bst, bl)) => {
                a.as_ptr() == Rc::as_ptr(b) && ao == bo && ast == bst && al == bl
            }
            _ => false,
        }
    }
}
impl LambdaMemo {
    fn identity_result(&self, argument: &V) -> Option<V> {
        let argument = available_value(argument);
        self.by_identity
            .iter()
            .find_map(|(key, value)| key.matches(&argument).then(|| value.upgrade()).flatten())
    }
    fn remember_identity(&mut self, argument: &V, value: &V) {
        let argument = available_value(argument);
        let (Some(key), Some(value)) = (WeakValue::of(&argument), WeakValue::of(value)) else {
            return;
        };
        const CAPACITY: usize = 8;
        self.by_identity
            .retain(|(key, value)| key.upgrade().is_some() && value.upgrade().is_some());
        if self.by_identity.len() == CAPACITY {
            self.by_identity.pop_front();
        }
        self.by_identity.push_back((key, value));
    }
}
enum EvalControl {
    Term(u32, EnvRef),
    Force(V),
    Apply(V, V),
    Value(V),
}
enum EvalFrame {
    Arguments(u32, usize, EnvRef),
    Apply(V),
    BoolMemo(V, BoolMemo),
    ThunkMemo(Rc<RefCell<Suspension>>),
    GlobalMemo(u32),
    Select(V, V),
    ReadCube(Rc<Vec<bool>>, usize, usize),
    ReadSparseCube(Rc<BTreeSet<usize>>, usize, usize, usize),
    ReadUniformCube(bool, u32),
}
// Inspect evaluated aliases only. Never demand a pending argument for a key.
fn available_value(value: &V) -> V {
    let mut value = value.clone();
    loop {
        let next = match &value {
            V::Thunk(memo) => match &*memo.borrow() {
                Suspension::Evaluated(value) => Some(value.clone()),
                Suspension::Pending(..) => None,
            },
            _ => None,
        };
        match next {
            Some(next) => value = next,
            None => return value,
        }
    }
}

fn sparse_view(bits: Rc<BTreeSet<usize>>, offset: usize, stride: usize, length: usize) -> V {
    if stride == length {
        V::Bit(bits.contains(&offset))
    } else if !bits
        .range(offset..length)
        .any(|index| (index - offset).is_multiple_of(stride))
    {
        V::UniformCube(false, (length / stride).trailing_zeros())
    } else {
        V::SparseCube(bits, offset, stride, length)
    }
}
// Normalize only concrete truth tables. A sparse backing preserves every
// address, and lets empty projections share the existing exact uniform value.
// Cap index storage relative to the dense allocation; small/dense tables keep
// their original representation. Selectors are still demanded by the machine.
pub(super) fn dense_cube(bits: Vec<bool>) -> V {
    if bits.len() >= 1024 && bits.len().is_power_of_two() {
        let mut ones = BTreeSet::new();
        for (index, bit) in bits.iter().enumerate() {
            if *bit {
                ones.insert(index);
                if ones.len() > bits.len() / 64 {
                    return V::SharedCube(Rc::new(bits), 0, 1);
                }
            }
        }
        return sparse_cube(bits.len().trailing_zeros(), ones);
    }
    V::SharedCube(Rc::new(bits), 0, 1)
}
// Inspect only an already available result; memoization must not introduce a
// demand for an unused (or not yet needed) argument.
fn available_bool(value: &V) -> Option<bool> {
    match value {
        V::Bit(bit) => Some(*bit),
        V::Thunk(delayed) => match &*delayed.borrow() {
            Suspension::Evaluated(V::Bit(bit)) => Some(*bit),
            _ => None,
        },
        _ => None,
    }
}
pub(super) fn thunk(term: u32, env: impl Into<EnvRef>) -> V {
    V::Thunk(Rc::new(RefCell::new(Suspension::Pending(term, env.into()))))
}
/// A concrete Boolean truth table with absent entries equal to false. This
/// changes only input storage: every demanded selector still executes normally.
pub(super) fn sparse_cube(depth: u32, ones: BTreeSet<usize>) -> V {
    let length = 1usize.checked_shl(depth).expect("host cube address width");
    assert!(ones.iter().all(|&i| i < length));
    if depth == 0 {
        V::Bit(ones.contains(&0))
    } else {
        sparse_view(Rc::new(ones), 0, 1, length)
    }
}
// Suspensions retain their own environment. Variables and Bool constants
// need no new suspension; no generated operation name is recognized here.
fn defer(c: &Certificate, term: u32, env: &EnvRef) -> V {
    match &c.term_table[term as usize] {
        TermNode::Var(i) => env.get(*i as usize),
        TermNode::Const { global, .. } => {
            let d = &c.declarations[*global as usize];
            match c.name_table[d.name as usize].as_str() {
                "Std.Bool.false" => V::Bit(false),
                "Std.Bool.true" => V::Bit(true),
                "Std.Bool.rec" => V::Rec(vec![]),
                _ => thunk(term, EnvRef::new(Env::Empty)),
            }
        }
        _ => thunk(term, env.clone()),
    }
}
// A call-by-need core machine with an explicit continuation stack. Deep
// finite arithmetic must not consume the native stack while reducing terms.
fn evaluate_counted<const COUNT: bool>(
    c: &Certificate,
    mut control: EvalControl,
    steps: &mut u64,
) -> V {
    let mut frames = vec![];
    let mut remaining = EVAL_REMAINING.with(Cell::get);
    let deadline = EVAL_DEADLINE.with(|deadline| *deadline);
    let mut clock_ticks = 0u16;
    if let Some(deadline) = deadline {
        assert!(Instant::now() < deadline, "MPK_CORE_MAX_SECONDS exhausted: ordinary evaluation deadline exceeded; no semantic verdict");
    }
    // Closed definitions always evaluate in their own empty environment.
    // This cache is local to this call and Certificate, has a fixed entry
    // bound, and owns no callbacks into itself. It cannot leak across runs or
    // recognize a generated operation. Unrequested definitions are not forced.
    const GLOBAL_CACHE_CAPACITY: usize = 64;
    let mut globals = HashMap::<u32, V>::new();
    let mut global_order = VecDeque::new();
    // A long body can evict its own closed function from the value cache.
    // Retain only bounded memo records across such reconstruction: these own
    // scalar bits/weak result handles and usage analysis, never an environment
    // or a strong closure. Otherwise a padding predicate loses every learned
    // constant-argument result before the next empty region invokes it.
    const GLOBAL_MEMO_CAPACITY: usize = 256;
    let mut global_memos = HashMap::<u32, BoolMemo>::new();
    let mut global_memo_order = VecDeque::new();
    // Lambda values retain this analysis across subsequent selector applications.
    // It contains only term usage, never certificate/global values or arguments.
    let mut usage = None::<Rc<RefCell<UsageAnalysis>>>;
    loop {
        if let Some(deadline) = deadline {
            clock_ticks = clock_ticks.wrapping_add(1);
            if clock_ticks == 0 {
                assert!(Instant::now() < deadline, "MPK_CORE_MAX_SECONDS exhausted: ordinary evaluation deadline exceeded; no semantic verdict");
            }
        }
        if let Some(left) = &mut remaining {
            if *left == 0 {
                EVAL_REMAINING.with(|budget| budget.set(Some(0)));
            }
            assert!(*left > 0, "MPK_CORE_MAX_STEPS exhausted: ordinary evaluation work budget exceeded; no semantic verdict");
            *left -= 1;
        }
        if COUNT {
            *steps += 1;
        }
        control = match control {
            EvalControl::Term(term, env) => match &c.term_table[term as usize] {
                TermNode::Var(i) => EvalControl::Force(env.get(*i as usize)),
                TermNode::Const { global, .. } => {
                    let d = &c.declarations[*global as usize];
                    match c.name_table[d.name as usize].as_str() {
                        "Std.Bool.false" => EvalControl::Value(V::Bit(false)),
                        "Std.Bool.true" => EvalControl::Value(V::Bit(true)),
                        "Std.Bool.rec" => EvalControl::Value(V::Rec(vec![])),
                        _ => match d.kind {
                            DeclarationKind::Def { value, .. } => {
                                if let Some(value) = globals.get(global) {
                                    EvalControl::Value(value.clone())
                                } else {
                                    frames.push(EvalFrame::GlobalMemo(*global));
                                    EvalControl::Term(value, EnvRef::new(Env::Empty))
                                }
                            }
                            _ => panic!("non-value constant"),
                        },
                    }
                }
                TermNode::Lam { body, .. } => {
                    let usage = usage
                        .get_or_insert_with(|| Rc::new(RefCell::new(UsageAnalysis::default())))
                        .clone();
                    let used = usage.borrow_mut().usage(c, *body);
                    let memo = LambdaMemo {
                        by_bool: [None, None],
                        independent: None,
                        argument_unused: !used.high && used.low & 1 == 0,
                        usage,
                        by_identity: VecDeque::new(),
                        observations: BitObservations::new(),
                    };
                    EvalControl::Value(V::Lambda(*body, env, Rc::new(RefCell::new(memo))))
                }
                TermNode::App {
                    function,
                    arguments,
                } => {
                    if !arguments.is_empty() {
                        frames.push(EvalFrame::Arguments(term, 0, env.clone()));
                    }
                    EvalControl::Term(*function, env)
                }
                TermNode::Let { value, body, .. } => {
                    let next = Env::Bind(defer(c, *value, &env), env);
                    EvalControl::Term(*body, EnvRef::new(next))
                }
                _ => panic!("evaluated a type"),
            },
            EvalControl::Force(V::Thunk(memo)) => {
                let state = memo.borrow().clone();
                match state {
                    Suspension::Evaluated(value) => EvalControl::Value(value),
                    Suspension::Pending(term, env) => {
                        frames.push(EvalFrame::ThunkMemo(memo));
                        EvalControl::Term(term, env)
                    }
                }
            }
            EvalControl::Force(value) => EvalControl::Value(value),
            EvalControl::Apply(function, argument) => {
                frames.push(EvalFrame::Apply(argument));
                EvalControl::Force(function)
            }
            EvalControl::Value(value) => match frames.pop() {
                None => {
                    EVAL_REMAINING.with(|budget| budget.set(remaining));
                    return value;
                }
                Some(EvalFrame::Arguments(term, index, env)) => {
                    // The certificate is immutable for the evaluation. Retain
                    // its application index instead of cloning its argument
                    // vector on every reduction; order and laziness are exact.
                    let TermNode::App { arguments, .. } = &c.term_table[term as usize] else {
                        unreachable!("application continuation")
                    };
                    let argument = defer(c, arguments[index], &env);
                    if index + 1 < arguments.len() {
                        frames.push(EvalFrame::Arguments(term, index + 1, env));
                    }
                    EvalControl::Apply(value, argument)
                }
                Some(EvalFrame::Apply(argument)) => match value {
                    V::Lambda(body, env, memo) => {
                        usage = Some(memo.borrow().usage.clone());
                        let argument = match argument {
                            V::Cube(bits) => dense_cube(bits),
                            x => x,
                        };
                        // Reuse a Bool already forced by another consumer, but
                        // never force a pending argument to obtain a cache key.
                        let key = available_bool(&argument);
                        let unused = memo.borrow().argument_unused;
                        let observed = key
                            .and_then(|key| memo.borrow().observations.borrow_mut().child(key))
                            .and_then(|node| node.borrow().bit().map(V::Bit));
                        let cached = observed.or_else(|| {
                            if unused {
                                memo.borrow()
                                    .independent
                                    .clone()
                                    .or_else(|| memo.borrow().identity_result(&V::Bit(false)))
                            } else {
                                key.and_then(|k| {
                                    memo.borrow().by_bool[usize::from(k)]
                                        .clone()
                                        .or_else(|| memo.borrow().identity_result(&V::Bit(k)))
                                })
                                .or_else(|| memo.borrow().identity_result(&argument))
                            }
                        });
                        if let Some(value) = cached {
                            EvalControl::Value(value)
                        } else {
                            if unused || WeakValue::of(&argument).is_some() {
                                frames.push(EvalFrame::BoolMemo(argument.clone(), memo));
                            }
                            // Keep the De Bruijn slot, but do not retain a value
                            // that cannot occur anywhere in the runtime body.
                            let next =
                                Env::Bind(if unused { V::Bit(false) } else { argument }, env);
                            EvalControl::Term(body, EnvRef::new(next))
                        }
                    }
                    V::Rec(mut arguments) => {
                        arguments.push(argument);
                        if arguments.len() == 3 {
                            let major = arguments.pop().unwrap();
                            let yes = arguments.pop().unwrap();
                            let no = arguments.pop().unwrap();
                            frames.push(EvalFrame::Select(no, yes));
                            EvalControl::Force(major)
                        } else {
                            EvalControl::Value(V::Rec(arguments))
                        }
                    }
                    V::Cube(bits) => EvalControl::Apply(dense_cube(bits), argument),
                    V::SharedCube(bits, offset, stride) => {
                        frames.push(EvalFrame::ReadCube(bits, offset, stride));
                        EvalControl::Force(argument)
                    }
                    V::SparseCube(bits, offset, stride, length) => {
                        frames.push(EvalFrame::ReadSparseCube(bits, offset, stride, length));
                        EvalControl::Force(argument)
                    }
                    V::UniformCube(bit, depth) => {
                        frames.push(EvalFrame::ReadUniformCube(bit, depth));
                        EvalControl::Force(argument)
                    }
                    V::Bit(_) => panic!("applied a leaf"),
                    V::Thunk(..) => unreachable!("forced function"),
                },
                Some(EvalFrame::BoolMemo(argument, memo)) => {
                    // Only a demanded/already-known Bool selects an observation
                    // path. Rebuilt intermediate closures share its scalar-only
                    // subtree, never the old captured environment.
                    let key = available_bool(&argument);
                    if let Some(node) =
                        key.and_then(|key| memo.borrow().observations.borrow_mut().child(key))
                    {
                        match &value {
                            V::Bit(bit) => node.borrow().remember(*bit),
                            V::Lambda(_, _, result_memo) => {
                                result_memo.borrow_mut().observations = node;
                            }
                            _ => (),
                        }
                    }
                    // The body may have demanded a formerly pending Bool.
                    // Record that result too, without forcing anything here.
                    // Only scalar bits are retained strongly: function results
                    // can capture large trees of further memoized closures.
                    // Reuse them while live elsewhere, then recompute on demand.
                    if memo.borrow().argument_unused {
                        if !matches!(value, V::Bit(_)) {
                            memo.borrow_mut().remember_identity(&V::Bit(false), &value);
                        } else {
                            memo.borrow_mut().independent = Some(value.clone());
                        }
                    } else if let Some(key) = available_bool(&argument) {
                        if !matches!(value, V::Bit(_)) {
                            memo.borrow_mut().remember_identity(&V::Bit(key), &value);
                        } else {
                            memo.borrow_mut().by_bool[usize::from(key)] = Some(value.clone());
                        }
                    } else {
                        memo.borrow_mut().remember_identity(&argument, &value);
                    }
                    EvalControl::Value(value)
                }
                Some(EvalFrame::GlobalMemo(global)) => {
                    let value = if let V::Lambda(body, env, fresh) = value {
                        let memo = if let Some(memo) = global_memos.get(&global) {
                            memo.clone()
                        } else {
                            if global_memos.len() == GLOBAL_MEMO_CAPACITY {
                                global_memos.remove(
                                    &global_memo_order.pop_front().expect("bounded global memo"),
                                );
                            }
                            global_memo_order.push_back(global);
                            global_memos.insert(global, fresh.clone());
                            fresh
                        };
                        V::Lambda(body, env, memo)
                    } else {
                        value
                    };
                    if !globals.contains_key(&global) {
                        if globals.len() == GLOBAL_CACHE_CAPACITY {
                            globals
                                .remove(&global_order.pop_front().expect("bounded global cache"));
                        }
                        global_order.push_back(global);
                    }
                    globals.insert(global, value.clone());
                    EvalControl::Value(value)
                }
                Some(EvalFrame::ThunkMemo(memo)) => {
                    // All aliases now own only the result. A returned closure
                    // retains any environment it still needs; the obsolete
                    // suspension environment must not stay alive separately.
                    *memo.borrow_mut() = Suspension::Evaluated(value.clone());
                    // A fresh suspension may hide the same complete constant
                    // cube seen by this function before. Reuse its result only
                    // after the body actually demanded the argument. Retain
                    // the owning memo frame and discard its unfinished body.
                    // No generated declaration/operation name is inspected.
                    let hit = if matches!(value, V::UniformCube(..)) {
                        frames.iter().enumerate().find_map(|(i, frame)| {
                            let EvalFrame::BoolMemo(V::Thunk(argument), owner) = frame else {
                                return None;
                            };
                            if !Rc::ptr_eq(argument, &memo) {
                                return None;
                            }
                            owner.borrow().identity_result(&value).map(|v| (i, v))
                        })
                    } else {
                        None
                    };
                    if let Some((index, cached)) = hit {
                        frames.truncate(index + 1);
                        EvalControl::Value(cached)
                    } else {
                        EvalControl::Value(value)
                    }
                }
                Some(EvalFrame::Select(no, yes)) => {
                    let V::Bit(choice) = value else {
                        panic!("non-Bool major")
                    };
                    EvalControl::Force(if choice { yes } else { no })
                }
                Some(EvalFrame::ReadCube(bits, offset, stride)) => {
                    let V::Bit(choice) = value else {
                        panic!("non-Bool selector")
                    };
                    let offset = offset + usize::from(choice) * stride;
                    let stride = 2 * stride;
                    EvalControl::Value(if stride >= bits.len() {
                        V::Bit(bits[offset])
                    } else {
                        V::SharedCube(bits, offset, stride)
                    })
                }
                Some(EvalFrame::ReadSparseCube(bits, offset, stride, length)) => {
                    let V::Bit(choice) = value else {
                        panic!("non-Bool selector")
                    };
                    let offset = offset + usize::from(choice) * stride;
                    let stride = 2 * stride;
                    EvalControl::Value(sparse_view(bits, offset, stride, length))
                }
                Some(EvalFrame::ReadUniformCube(bit, depth)) => {
                    let V::Bit(_) = value else {
                        panic!("non-Bool selector")
                    };
                    EvalControl::Value(if depth == 1 {
                        V::Bit(bit)
                    } else {
                        V::UniformCube(bit, depth - 1)
                    })
                }
            },
        };
    }
}
fn evaluate(c: &Certificate, control: EvalControl) -> V {
    evaluate_counted::<false>(c, control, &mut 0)
}
/// Generic interpreter work measurement. No generated operation is recognized;
/// the same reduction machine runs with an extra transition counter.
pub(super) fn run_counted(c: &Certificate, name: &str) -> (V, u64) {
    let d = c
        .declarations
        .iter()
        .find(|d| c.name_table[d.name as usize] == name)
        .unwrap();
    let DeclarationKind::Def { value, .. } = d.kind else {
        panic!()
    };
    let mut steps = 0;
    let result = evaluate_counted::<true>(
        c,
        EvalControl::Term(value, EnvRef::new(Env::Empty)),
        &mut steps,
    );
    (result, steps)
}
pub(super) fn eval(c: &Certificate, term: u32, env: &[V]) -> V {
    let env = env
        .iter()
        .rev()
        .fold(EnvRef::new(Env::Empty), |tail, value| {
            EnvRef::new(Env::Bind(value.clone(), tail))
        });
    evaluate(c, EvalControl::Term(term, env))
}
pub(super) fn apply(c: &Certificate, f: V, x: V) -> V {
    evaluate(c, EvalControl::Apply(f, x))
}
pub(super) fn apply_counted(c: &Certificate, f: V, x: V) -> (V, u64) {
    let mut steps = 0;
    let value = evaluate_counted::<true>(c, EvalControl::Apply(f, x), &mut steps);
    (value, steps)
}
pub(super) fn run(c: &Certificate, name: &str, args: Vec<V>) -> V {
    let d = c
        .declarations
        .iter()
        .find(|d| c.name_table[d.name as usize] == name)
        .unwrap();
    let DeclarationKind::Def { value, .. } = d.kind else {
        panic!()
    };
    args.into_iter().fold(
        evaluate(c, EvalControl::Term(value, EnvRef::new(Env::Empty))),
        |f, a| apply(c, f, a),
    )
}
pub(super) fn bit(v: V) -> bool {
    let V::Bit(v) = v else { panic!() };
    v
}

/// Requested allocation sizes before allocator size-class rounding. Rc uses
/// two count words; RefCell adds its borrow flag. This reports layouts only.
#[allow(dead_code)] // The same file also serves lib tests without the integration probe.
pub(super) fn allocation_layouts() -> [(&'static str, usize); 3] {
    [
        ("Rc<Env>", std::mem::size_of::<(usize, usize, Env)>()),
        (
            "Rc<RefCell<LambdaMemo>>",
            std::mem::size_of::<(usize, usize, RefCell<LambdaMemo>)>(),
        ),
        (
            "Rc<RefCell<Suspension>>",
            std::mem::size_of::<(usize, usize, RefCell<Suspension>)>(),
        ),
    ]
}

#[test]
fn ordinary_core_deep_observation_cache_is_bounded_and_isolated() {
    let root = BitObservations::new();
    let path = |address: u32, depth: u8| {
        let mut node = root.clone();
        for selector in 0..depth {
            let next = node.borrow().child(address & (1 << selector) != 0).unwrap();
            node = next;
        }
        node
    };
    // Same bit pattern with a different number of arguments must not alias.
    path(0, 1).borrow().remember(true);
    assert_eq!(path(0, 2).borrow().bit(), None);
    for address in 0..2048 {
        let node = path(address, 18);
        node.borrow().remember(address % 2 != 0);
        assert_eq!(node.borrow().bit(), Some(address % 2 != 0));
        assert!(root.borrow().leaves.borrow().values.len() <= BitObservations::CAPACITY);
    }
    assert_eq!(path(0, 18).borrow().bit(), None);
    for address in 1024..2048 {
        assert_eq!(path(address, 18).borrow().bit(), Some(address % 2 != 0));
    }
    assert!(path(0, 18).borrow().child(false).is_none());
    let fresh = BitObservations::new();
    assert!(fresh.borrow().leaves.borrow().values.is_empty());
    let weak = Rc::downgrade(&root.borrow().leaves);
    drop(root);
    assert!(weak.upgrade().is_none(), "observation store leaked");
}
