//! Test-only call-by-need evaluation of ordinary Boolean core terms.
//! This supplies observations, never proof or certificate acceptance.
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};
use std::{cell::RefCell, rc::Rc};
// A binding adds one shared node rather than copying every captured value.
// De Bruijn lookup keeps the newest binding at index zero.
pub(super) enum Env {
    Empty,
    Bind(V, Rc<Env>),
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
type BoolMemo = Rc<RefCell<[Option<V>; 2]>>;
#[derive(Clone)]
pub(super) enum V {
    Bit(bool),
    Cube(Vec<bool>),
    SharedCube(Rc<Vec<bool>>, usize, usize),
    Lambda(u32, Rc<Env>, BoolMemo),
    Thunk(u32, Rc<Env>, Rc<RefCell<Option<V>>>),
    Rec(Vec<V>),
}
enum EvalControl {
    Term(u32, Rc<Env>),
    Force(V),
    Apply(V, V),
    Value(V),
}
enum EvalFrame {
    Arguments(Vec<u32>, usize, Rc<Env>),
    Apply(V),
    BoolMemo(bool, BoolMemo),
    ThunkMemo(Rc<RefCell<Option<V>>>),
    Select(V, V),
    ReadCube(Rc<Vec<bool>>, usize, usize),
}
// Suspensions retain their own environment. Variables and Bool constants
// need no new suspension; no generated operation name is recognized here.
fn defer(c: &Certificate, term: u32, env: &Rc<Env>) -> V {
    match &c.term_table[term as usize] {
        TermNode::Var(i) => env.get(*i as usize),
        TermNode::Const { global, .. } => {
            let d = &c.declarations[*global as usize];
            match c.name_table[d.name as usize].as_str() {
                "Std.Bool.false" => V::Bit(false),
                "Std.Bool.true" => V::Bit(true),
                "Std.Bool.rec" => V::Rec(vec![]),
                _ => V::Thunk(term, Rc::new(Env::Empty), Rc::new(RefCell::new(None))),
            }
        }
        _ => V::Thunk(term, env.clone(), Rc::new(RefCell::new(None))),
    }
}
// A call-by-need core machine with an explicit continuation stack. Deep
// finite arithmetic must not consume the native stack while reducing terms.
fn evaluate(c: &Certificate, mut control: EvalControl) -> V {
    let mut frames = vec![];
    loop {
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
                                EvalControl::Term(value, Rc::new(Env::Empty))
                            }
                            _ => panic!("non-value constant"),
                        },
                    }
                }
                TermNode::Lam { body, .. } => {
                    EvalControl::Value(V::Lambda(*body, env, Rc::new(RefCell::new([None, None]))))
                }
                TermNode::App {
                    function,
                    arguments,
                } => {
                    if !arguments.is_empty() {
                        frames.push(EvalFrame::Arguments(arguments.clone(), 0, env.clone()));
                    }
                    EvalControl::Term(*function, env)
                }
                TermNode::Let { value, body, .. } => {
                    let next = Env::Bind(defer(c, *value, &env), env);
                    EvalControl::Term(*body, Rc::new(next))
                }
                _ => panic!("evaluated a type"),
            },
            EvalControl::Force(V::Thunk(term, env, memo)) => {
                let cached = memo.borrow().clone();
                if let Some(value) = cached {
                    EvalControl::Value(value)
                } else {
                    frames.push(EvalFrame::ThunkMemo(memo));
                    EvalControl::Term(term, env)
                }
            }
            EvalControl::Force(value) => EvalControl::Value(value),
            EvalControl::Apply(function, argument) => {
                frames.push(EvalFrame::Apply(argument));
                EvalControl::Force(function)
            }
            EvalControl::Value(value) => match frames.pop() {
                None => return value,
                Some(EvalFrame::Arguments(arguments, index, env)) => {
                    let argument = defer(c, arguments[index], &env);
                    if index + 1 < arguments.len() {
                        frames.push(EvalFrame::Arguments(arguments, index + 1, env));
                    }
                    EvalControl::Apply(value, argument)
                }
                Some(EvalFrame::Apply(argument)) => match value {
                    V::Lambda(body, env, memo) => {
                        let argument = match argument {
                            V::Cube(bits) => V::SharedCube(Rc::new(bits), 0, 1),
                            x => x,
                        };
                        let key = if let V::Bit(bit) = argument {
                            Some(bit)
                        } else {
                            None
                        };
                        let cached = key.and_then(|k| memo.borrow()[usize::from(k)].clone());
                        if let Some(value) = cached {
                            EvalControl::Value(value)
                        } else {
                            if let Some(key) = key {
                                frames.push(EvalFrame::BoolMemo(key, memo));
                            }
                            let next = Env::Bind(argument, env);
                            EvalControl::Term(body, Rc::new(next))
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
                    V::Cube(bits) => {
                        EvalControl::Apply(V::SharedCube(Rc::new(bits), 0, 1), argument)
                    }
                    V::SharedCube(bits, offset, stride) => {
                        frames.push(EvalFrame::ReadCube(bits, offset, stride));
                        EvalControl::Force(argument)
                    }
                    V::Bit(_) => panic!("applied a leaf"),
                    V::Thunk(..) => unreachable!("forced function"),
                },
                Some(EvalFrame::BoolMemo(key, memo)) => {
                    memo.borrow_mut()[usize::from(key)] = Some(value.clone());
                    EvalControl::Value(value)
                }
                Some(EvalFrame::ThunkMemo(memo)) => {
                    *memo.borrow_mut() = Some(value.clone());
                    EvalControl::Value(value)
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
            },
        };
    }
}
pub(super) fn eval(c: &Certificate, term: u32, env: &[V]) -> V {
    let env = env.iter().rev().fold(Rc::new(Env::Empty), |tail, value| {
        Rc::new(Env::Bind(value.clone(), tail))
    });
    evaluate(c, EvalControl::Term(term, env))
}
pub(super) fn apply(c: &Certificate, f: V, x: V) -> V {
    evaluate(c, EvalControl::Apply(f, x))
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
        evaluate(c, EvalControl::Term(value, Rc::new(Env::Empty))),
        |f, a| apply(c, f, a),
    )
}
pub(super) fn bit(v: V) -> bool {
    let V::Bit(v) = v else { panic!() };
    v
}
