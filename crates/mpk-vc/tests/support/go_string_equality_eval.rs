//! Finite observation of actual ordinary definitions, including Nat recursion.
//! Only core constructors/eliminators and the complete input carrier are native.
//! No arithmetic/string definition or theorem body is replaced by a callback.
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Primitive {
    BoolRec,
    BoolCases,
    BoolType,
    NatSucc,
    NatRec,
    NatInduct,
}
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum Value {
    Bit(bool),
    Nat(u64),
    Type,
    Closure(u32, u32),
    Thunk(u32, u32),
    Primitive(Primitive, Vec<u32>),
    Recursion {
        base: u32,
        step: u32,
        count: u64,
    },
    Word {
        bits: u64,
        remaining: u8,
        address: u8,
    },
    Slots {
        input: usize,
        remaining: u8,
        address: u8,
    },
    Raw {
        length: u64,
        input: usize,
    },
}
pub struct Interpreter<'a> {
    certificate: &'a Certificate,
    free_variables: Vec<usize>,
    values: Vec<Value>,
    intern: HashMap<Value, u32>,
    environments: Vec<Option<(u32, u32)>>,
    environment_intern: HashMap<(u32, u32), u32>,
    constants: Vec<Option<u32>>,
    terms: HashMap<(u32, u32), u32>,
    applications: HashMap<(u32, u32), u32>,
    forced: HashMap<u32, u32>,
    inputs: Vec<[u8; 256]>,
    steps: usize,
}
impl<'a> Interpreter<'a> {
    pub fn new(certificate: &'a Certificate) -> Self {
        // Canonical environments retain exactly the de Bruijn variables read
        // by a term. Closed recursion steps share one closure across inputs.
        let mut free_variables: Vec<usize> = vec![];
        for node in &certificate.term_table {
            let need = match node {
                TermNode::Sort(_) | TermNode::Pi { .. } | TermNode::Const { .. } => 0,
                TermNode::Var(i) => *i as usize + 1,
                TermNode::Lam { body, .. } => free_variables[*body as usize].saturating_sub(1),
                TermNode::App {
                    function,
                    arguments,
                } => arguments
                    .iter()
                    .fold(free_variables[*function as usize], |n, a| {
                        n.max(free_variables[*a as usize])
                    }),
                TermNode::Let { value, body, .. } => free_variables[*value as usize]
                    .max(free_variables[*body as usize].saturating_sub(1)),
            };
            free_variables.push(need);
        }
        let mut s = Self {
            certificate,
            free_variables,
            values: vec![],
            intern: HashMap::new(),
            environments: vec![None],
            environment_intern: HashMap::new(),
            constants: vec![None; certificate.declarations.len()],
            terms: HashMap::new(),
            applications: HashMap::new(),
            forced: HashMap::new(),
            inputs: vec![],
            steps: 0,
        };
        s.intern(Value::Bit(false));
        s.intern(Value::Bit(true));
        s.intern(Value::Type);
        s
    }
    fn tick(&mut self) {
        self.steps += 1;
        assert!(
            self.steps <= 50_000_000
                && self.values.len() <= 2_000_000
                && self.environments.len() <= 2_000_000,
            "finite observation ceiling: steps={} values={} environments={}; universal core proof is checked separately",
            self.steps,
            self.values.len(),
            self.environments.len(),
        );
    }
    fn intern(&mut self, value: Value) -> u32 {
        if let Some(&id) = self.intern.get(&value) {
            return id;
        }
        let id = self.values.len() as u32;
        self.values.push(value.clone());
        self.intern.insert(value, id);
        id
    }
    fn extend(&mut self, parent: u32, value: u32) -> u32 {
        if let Some(&id) = self.environment_intern.get(&(parent, value)) {
            return id;
        }
        let id = self.environments.len() as u32;
        self.environments.push(Some((parent, value)));
        self.environment_intern.insert((parent, value), id);
        id
    }
    fn variable(&self, mut env: u32, index: u32) -> u32 {
        for _ in 0..index {
            env = self.environments[env as usize].unwrap().0;
        }
        self.environments[env as usize].unwrap().1
    }
    fn trimmed(&mut self, env: u32, count: usize) -> u32 {
        if count == 0 {
            return 0;
        }
        let (parent, value) = self.environments[env as usize].unwrap();
        let value = self.forced.get(&value).copied().unwrap_or(value);
        let parent = self.trimmed(parent, count - 1);
        self.extend(parent, value)
    }
    fn thunk(&mut self, term: u32, env: u32) -> u32 {
        if let TermNode::Var(i) = self.certificate.term_table[term as usize] {
            return self.variable(env, i);
        }
        let env = self.trimmed(env, self.free_variables[term as usize]);
        self.intern(Value::Thunk(term, env))
    }
    fn force(&mut self, id: u32) -> u32 {
        self.tick();
        if let Some(&value) = self.forced.get(&id) {
            return value;
        }
        let next = match self.values[id as usize].clone() {
            Value::Thunk(term, env) => self.evaluate(term, env),
            Value::Recursion { base, step, count } => {
                if count == 0 {
                    base
                } else {
                    let previous = self.intern(Value::Nat(count - 1));
                    let rec = self.intern(Value::Recursion {
                        base,
                        step,
                        count: count - 1,
                    });
                    let f = self.apply(step, previous);
                    self.apply(f, rec)
                }
            }
            _ => return id,
        };
        let value = self.force(next);
        self.forced.insert(id, value);
        value
    }
    fn constant(&mut self, global: u32) -> u32 {
        if let Some(id) = self.constants[global as usize] {
            return id;
        }
        let d = &self.certificate.declarations[global as usize];
        let name = &self.certificate.name_table[d.name as usize];
        let primitive = match name.as_str() {
            "Std.Bool.rec" => Some(Primitive::BoolRec),
            "Std.Bool.cases" => Some(Primitive::BoolCases),
            "Std.Bool.recType" => Some(Primitive::BoolType),
            "Std.Nat.succ" => Some(Primitive::NatSucc),
            "Std.Nat.rec" => Some(Primitive::NatRec),
            "Std.Nat.induct" => Some(Primitive::NatInduct),
            _ => None,
        };
        let value = if let Some(p) = primitive {
            self.intern(Value::Primitive(p, vec![]))
        } else {
            match name.as_str() {
                "Std.Bool.false" => 0,
                "Std.Bool.true" => 1,
                "Std.Bool" | "Std.Nat" => 2,
                "Std.Nat.zero" => self.intern(Value::Nat(0)),
                _ => {
                    let DeclarationKind::Def { value, .. } = d.kind else {
                        panic!("no theorem-body observation: {name}")
                    };
                    self.evaluate(value, 0)
                }
            }
        };
        self.constants[global as usize] = Some(value);
        value
    }
    fn evaluate(&mut self, term: u32, env: u32) -> u32 {
        self.tick();
        let env = self.trimmed(env, self.free_variables[term as usize]);
        if let Some(&id) = self.terms.get(&(term, env)) {
            return id;
        }
        let id = match self.certificate.term_table[term as usize].clone() {
            TermNode::Sort(_) | TermNode::Pi { .. } => 2,
            TermNode::Var(i) => self.variable(env, i),
            TermNode::Const { global, levels } => {
                assert!(levels.is_empty());
                self.constant(global)
            }
            TermNode::Lam { body, .. } => self.intern(Value::Closure(body, env)),
            TermNode::App {
                function,
                arguments,
            } => {
                let mut f = self.evaluate(function, env);
                for a in arguments {
                    let value = self.thunk(a, env);
                    f = self.apply(f, value);
                }
                f
            }
            TermNode::Let { value, body, .. } => {
                let value = self.thunk(value, env);
                let env = self.extend(env, value);
                self.evaluate(body, env)
            }
        };
        self.terms.insert((term, env), id);
        id
    }
    pub fn boolean(&mut self, id: u32) -> bool {
        let id = self.force(id);
        let Value::Bit(value) = self.values[id as usize] else {
            panic!("expected actual Bool")
        };
        value
    }
    fn natural(&mut self, id: u32) -> u64 {
        let id = self.force(id);
        let Value::Nat(value) = self.values[id as usize] else {
            panic!("expected actual Nat")
        };
        value
    }
    fn word(&mut self, bits: u64, remaining: u8) -> u32 {
        self.intern(Value::Word {
            bits,
            remaining,
            address: 0,
        })
    }
    fn apply(&mut self, function: u32, argument: u32) -> u32 {
        self.tick();
        let function = self.force(function);
        let argument = self.forced.get(&argument).copied().unwrap_or(argument);
        if let Some(&id) = self.applications.get(&(function, argument)) {
            return id;
        }
        let value = match self.values[function as usize].clone() {
            Value::Closure(body, env) => {
                let env = self.extend(env, argument);
                self.evaluate(body, env)
            }
            Value::Word {
                bits,
                remaining,
                address,
            } => {
                let address = (address << 1) | u8::from(self.boolean(argument));
                if remaining == 1 {
                    u32::from(bits & (1u64 << address) != 0)
                } else {
                    self.intern(Value::Word {
                        bits,
                        remaining: remaining - 1,
                        address,
                    })
                }
            }
            Value::Slots {
                input,
                remaining,
                address,
            } => {
                let address = (address << 1) | u8::from(self.boolean(argument));
                if remaining == 1 {
                    self.word(self.inputs[input][address as usize] as u64, 3)
                } else {
                    self.intern(Value::Slots {
                        input,
                        remaining: remaining - 1,
                        address,
                    })
                }
            }
            Value::Raw { length, input } => {
                if self.boolean(argument) {
                    self.intern(Value::Slots {
                        input,
                        remaining: 8,
                        address: 0,
                    })
                } else {
                    self.word(length, 6)
                }
            }
            Value::Primitive(kind, mut args) => {
                args.push(argument);
                let arity = match kind {
                    Primitive::BoolRec | Primitive::BoolType | Primitive::NatRec => 3,
                    Primitive::BoolCases | Primitive::NatInduct => 4,
                    Primitive::NatSucc => 1,
                };
                if args.len() != arity {
                    self.intern(Value::Primitive(kind, args))
                } else {
                    match kind {
                        Primitive::BoolType => 2,
                        Primitive::BoolRec | Primitive::BoolCases => {
                            let offset = usize::from(matches!(kind, Primitive::BoolCases));
                            args[offset + usize::from(self.boolean(args[arity - 1]))]
                        }
                        Primitive::NatSucc => {
                            let n = self.natural(args[0]).checked_add(1).unwrap();
                            self.intern(Value::Nat(n))
                        }
                        Primitive::NatRec | Primitive::NatInduct => {
                            let offset = usize::from(matches!(kind, Primitive::NatInduct));
                            let count = self.natural(args[arity - 1]);
                            self.intern(Value::Recursion {
                                base: args[offset],
                                step: args[offset + 1],
                                count,
                            })
                        }
                    }
                }
            }
            other => panic!("actual ordinary application is not a function: {other:?}"),
        };
        self.applications.insert((function, argument), value);
        value
    }
    pub fn byte_input(&mut self, bits: u8) -> u32 {
        self.word(u64::from(bits), 3)
    }
    pub fn raw(&mut self, length: u64, bytes: [u8; 256]) -> u32 {
        let input = self.inputs.len();
        self.inputs.push(bytes);
        self.intern(Value::Raw { length, input })
    }
    pub fn invoke(&mut self, name: &str, args: &[u32]) -> u32 {
        let global = self
            .certificate
            .declarations
            .iter()
            .position(|d| self.certificate.name_table[d.name as usize] == name)
            .unwrap() as u32;
        let mut f = self.constant(global);
        for &arg in args {
            f = self.apply(f, arg);
        }
        self.force(f)
    }
    fn word_bits(&mut self, width: u8, value: u32) -> u64 {
        let mut bits = 0;
        for i in 0..width {
            let mut flag = value;
            for j in (0..width.trailing_zeros()).rev() {
                flag = self.apply(flag, u32::from(i & (1 << j) != 0));
            }
            if self.boolean(flag) {
                bits |= 1u64 << i;
            }
        }
        bits
    }
    pub fn length(&mut self, value: u32) -> u64 {
        let length = self.invoke("Mpk.Go.Practical.String.Length", &[value]);
        self.word_bits(64, length)
    }
    pub fn bytes(&mut self, value: u32) -> [u8; 256] {
        let data = self.invoke("Mpk.Go.Practical.String.Data", &[value]);
        std::array::from_fn(|i| {
            let mut byte = data;
            for j in (0..8).rev() {
                byte = self.apply(byte, u32::from(i & (1 << j) != 0));
            }
            self.word_bits(8, byte) as u8
        })
    }
    pub fn steps(&self) -> usize {
        self.steps
    }
}
