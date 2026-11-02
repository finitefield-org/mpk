//! Independent finite observation of actual decoded Certificate-v0 terms.
//! Only Bool constructors/recursors and externally assigned input-word bits
//! are primitives. Arithmetic names never substitute a high-level operation.
use mpk_cert::encode::{Certificate, DeclarationKind, TermNode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Primitive {
    Rec,
    Cases,
}
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum Value {
    Bit(bool),
    Type,
    Closure(u32, u32),
    InputWord {
        bits: u64,
        remaining: u8,
        address: u8,
    },
    Primitive(Primitive, Vec<u32>),
}

pub struct Interpreter<'a> {
    certificate: &'a Certificate,
    values: Vec<Value>,
    intern: HashMap<Value, u32>,
    environments: Vec<Option<(u32, u32)>>,
    environment_intern: HashMap<(u32, u32), u32>,
    constants: Vec<Option<u32>>,
    terms: HashMap<(u32, u32), u32>,
    applications: HashMap<(u32, u32), u32>,
    steps: usize,
}

impl<'a> Interpreter<'a> {
    pub fn new(certificate: &'a Certificate) -> Self {
        let mut s = Self {
            certificate,
            values: vec![],
            intern: HashMap::new(),
            environments: vec![None],
            environment_intern: HashMap::new(),
            constants: vec![None; certificate.declarations.len()],
            terms: HashMap::new(),
            applications: HashMap::new(),
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
            "private observation resource ceiling, not a Go arithmetic failure"
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
    fn variable(&self, mut environment: u32, index: u32) -> u32 {
        for _ in 0..index {
            environment = self.environments[environment as usize]
                .expect("bound de Bruijn variable")
                .0;
        }
        self.environments[environment as usize]
            .expect("bound de Bruijn variable")
            .1
    }
    fn constant(&mut self, global: u32) -> u32 {
        if let Some(id) = self.constants[global as usize] {
            return id;
        }
        let declaration = &self.certificate.declarations[global as usize];
        let name = &self.certificate.name_table[declaration.name as usize];
        let id = match name.as_str() {
            "Std.Bool.false" => 0,
            "Std.Bool.true" => 1,
            "Std.Bool" => 2,
            "Std.Bool.rec" => self.intern(Value::Primitive(Primitive::Rec, vec![])),
            "Std.Bool.cases" => self.intern(Value::Primitive(Primitive::Cases, vec![])),
            _ => {
                let DeclarationKind::Def { value, .. } = declaration.kind else {
                    panic!("unsupported non-Boolean model constant {name}");
                };
                self.evaluate(value, 0)
            }
        };
        self.constants[global as usize] = Some(id);
        id
    }
    fn evaluate(&mut self, index: u32, environment: u32) -> u32 {
        self.tick();
        if let Some(&id) = self.terms.get(&(index, environment)) {
            return id;
        }
        let id = match self.certificate.term_table[index as usize].clone() {
            TermNode::Sort(_) | TermNode::Pi { .. } => 2,
            TermNode::Var(i) => self.variable(environment, i),
            TermNode::Const { global, levels } => {
                assert!(levels.is_empty(), "monomorphic word observation");
                self.constant(global)
            }
            TermNode::Lam { body, .. } => self.intern(Value::Closure(body, environment)),
            TermNode::App {
                function,
                arguments,
            } => {
                let mut f = self.evaluate(function, environment);
                for argument in arguments {
                    let a = self.evaluate(argument, environment);
                    f = self.apply(f, a);
                }
                f
            }
            TermNode::Let { value, body, .. } => {
                let a = self.evaluate(value, environment);
                let env = self.extend(environment, a);
                self.evaluate(body, env)
            }
        };
        self.terms.insert((index, environment), id);
        id
    }
    fn apply(&mut self, function: u32, argument: u32) -> u32 {
        self.tick();
        if let Some(&id) = self.applications.get(&(function, argument)) {
            return id;
        }
        let id = match self.values[function as usize].clone() {
            Value::Closure(body, environment) => {
                let env = self.extend(environment, argument);
                self.evaluate(body, env)
            }
            Value::InputWord {
                bits,
                remaining,
                address,
            } => {
                let address = (address << 1) | u8::from(self.boolean(argument));
                if remaining == 1 {
                    u32::from(bits & (1u64 << address) != 0)
                } else {
                    self.intern(Value::InputWord {
                        bits,
                        remaining: remaining - 1,
                        address,
                    })
                }
            }
            Value::Primitive(kind, mut args) => {
                args.push(argument);
                let (arity, offset) = match kind {
                    Primitive::Rec => (3, 0),
                    Primitive::Cases => (4, 1),
                };
                if args.len() == arity {
                    args[offset + usize::from(self.boolean(args[arity - 1]))]
                } else {
                    self.intern(Value::Primitive(kind, args))
                }
            }
            other => panic!("apply a nonfunction {other:?}"),
        };
        self.applications.insert((function, argument), id);
        id
    }
    pub fn word(&mut self, width: u8, bits: u64) -> u32 {
        assert!([8, 16, 32, 64].contains(&width));
        self.intern(Value::InputWord {
            bits,
            remaining: width.trailing_zeros() as u8,
            address: 0,
        })
    }
    pub fn invoke(&mut self, name: &str, args: &[u32]) -> u32 {
        let global = self
            .certificate
            .declarations
            .iter()
            .position(|d| self.certificate.name_table[d.name as usize] == name)
            .expect("actual compiled declaration") as u32;
        let mut f = self.constant(global);
        for &a in args {
            f = self.apply(f, a);
        }
        f
    }
    pub fn boolean(&self, id: u32) -> bool {
        let Value::Bit(value) = self.values[id as usize] else {
            panic!("Boolean observation result");
        };
        value
    }
    pub fn bits(&mut self, width: u8, value: u32) -> u64 {
        let mut bits = 0;
        for i in 0..width {
            let mut bit = value;
            for j in (0..width.trailing_zeros()).rev() {
                bit = self.apply(bit, u32::from(i & (1 << j) != 0));
            }
            if self.boolean(bit) {
                bits |= 1u64 << i;
            }
        }
        bits
    }
    pub fn steps(&self) -> usize {
        self.steps
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub id: String,
    pub width: u8,
    pub operation: String,
    pub x_hex: String,
    pub y_hex: String,
    pub count_hex: String,
    pub signed_count: bool,
    pub target_width: u8,
}
#[derive(Debug, Serialize, Eq, PartialEq)]
pub struct Observation {
    pub id: String,
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
}
fn mask(width: u8) -> u64 {
    assert!([8, 16, 32, 64].contains(&width));
    u64::MAX >> (64 - width)
}
fn signed(width: u8, bits: u64) -> i128 {
    if bits & (1u64 << (width - 1)) != 0 {
        i128::from(bits) - (1i128 << width)
    } else {
        i128::from(bits)
    }
}
fn hex(s: &str, width: u8) -> u64 {
    assert_eq!(s.len(), usize::from(width / 4));
    assert!(s
        .bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
    u64::from_str_radix(s, 16).unwrap()
}
fn word_observation(q: &Query, width: u8, bits: u64) -> Observation {
    Observation {
        id: q.id.clone(),
        kind: "word",
        value: None,
        value_hex: Some(format!(
            "{:0width$x}",
            bits & mask(width),
            width = usize::from(width / 4)
        )),
        reason: None,
    }
}
fn boolean_observation(q: &Query, value: bool) -> Observation {
    Observation {
        id: q.id.clone(),
        kind: "boolean",
        value: Some(value),
        value_hex: None,
        reason: None,
    }
}
fn failed(q: &Query, reason: &'static str) -> Observation {
    Observation {
        id: q.id.clone(),
        kind: "runtime_safety_failure",
        value: None,
        value_hex: None,
        reason: Some(reason),
    }
}

pub fn oracle(q: &Query) -> Observation {
    let (x, y, count) = (
        hex(&q.x_hex, q.width),
        hex(&q.y_hex, q.width),
        hex(&q.count_hex, 64),
    );
    let (sx, sy) = (signed(q.width, x), signed(q.width, y));
    let shift = matches!(
        q.operation.as_str(),
        "ShiftLeft" | "ShiftRight" | "SignedShiftRight"
    );
    if shift && q.signed_count && count >> 63 != 0 {
        return failed(q, "negative_shift");
    }
    if matches!(
        q.operation.as_str(),
        "UnsignedDiv" | "UnsignedRem" | "SignedDiv" | "SignedRem"
    ) && y == 0
    {
        return failed(q, "division_by_zero");
    }
    let value = match q.operation.as_str() {
        "Add" => x.wrapping_add(y),
        "Sub" => x.wrapping_sub(y),
        "Mul" => x.wrapping_mul(y),
        "Neg" => x.wrapping_neg(),
        "Not" => !x,
        "And" => x & y,
        "Or" => x | y,
        "Xor" => x ^ y,
        "AndNot" => x & !y,
        "Equal" => return boolean_observation(q, x == y),
        "NotEqual" => return boolean_observation(q, x != y),
        "UnsignedLess" => return boolean_observation(q, x < y),
        "UnsignedLessEqual" => return boolean_observation(q, x <= y),
        "SignedLess" => return boolean_observation(q, sx < sy),
        "SignedLessEqual" => return boolean_observation(q, sx <= sy),
        "UnsignedDiv" => x / y,
        "UnsignedRem" => x % y,
        "SignedDiv" => (sx / sy) as u64,
        "SignedRem" => (sx % sy) as u64,
        "ShiftLeft" => {
            if count >= u64::from(q.width) {
                0
            } else {
                x << count
            }
        }
        "ShiftRight" => {
            if count >= u64::from(q.width) {
                0
            } else {
                x >> count
            }
        }
        "SignedShiftRight" => {
            if count >= u64::from(q.width) {
                if sx < 0 {
                    u64::MAX
                } else {
                    0
                }
            } else {
                (sx >> count) as u64
            }
        }
        name if name.starts_with("SignedTo") => {
            return word_observation(q, q.target_width, sx as u64)
        }
        name if name.starts_with("UnsignedTo") => return word_observation(q, q.target_width, x),
        _ => panic!("closed ordinary word observation operation"),
    };
    word_observation(q, q.width, value)
}

pub fn observe(c: &Certificate, q: &Query) -> (Observation, usize) {
    let mut model = Interpreter::new(c);
    let (x, y, count) = (
        hex(&q.x_hex, q.width),
        hex(&q.y_hex, q.width),
        hex(&q.count_hex, 64),
    );
    let x = model.word(q.width, x);
    let y = model.word(q.width, y);
    let count = model.word(64, count);
    let prefix = format!("Mpk.Go.Practical.Word.W{}.", q.width);
    let shift = matches!(
        q.operation.as_str(),
        "ShiftLeft" | "ShiftRight" | "SignedShiftRight"
    );
    if shift && q.signed_count {
        let value = model.invoke(&(prefix.clone() + "SignedCountDefined"), &[count]);
        if !model.boolean(value) {
            return (failed(q, "negative_shift"), model.steps());
        }
    }
    if matches!(
        q.operation.as_str(),
        "UnsignedDiv" | "UnsignedRem" | "SignedDiv" | "SignedRem"
    ) {
        let value = model.invoke(&(prefix.clone() + "DivideDefined"), &[y]);
        if !model.boolean(value) {
            return (failed(q, "division_by_zero"), model.steps());
        }
    }
    let args = if shift {
        vec![x, count]
    } else if matches!(q.operation.as_str(), "Neg" | "Not") || q.operation.contains("To") {
        vec![x]
    } else {
        vec![x, y]
    };
    let result = model.invoke(&(prefix + &q.operation), &args);
    let observation = if matches!(
        q.operation.as_str(),
        "Equal"
            | "NotEqual"
            | "UnsignedLess"
            | "UnsignedLessEqual"
            | "SignedLess"
            | "SignedLessEqual"
    ) {
        boolean_observation(q, model.boolean(result))
    } else {
        let width = if q.target_width == 0 {
            q.width
        } else {
            q.target_width
        };
        let bits = model.bits(width, result);
        word_observation(q, width, bits)
    };
    (observation, model.steps())
}
