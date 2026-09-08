//! Finite Boolean circuits. No host arithmetic result becomes a core definition.
use super::*;
type Bit = usize;
type Word = Vec<Bit>;
const F: Bit = 0;
const T: Bit = 1;
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Gate {
    False,
    True,
    Input(usize, usize),
    Not(Bit),
    And(Bit, Bit),
    Xor(Bit, Bit),
    Mux(Bit, Bit, Bit),
}
struct Circuit {
    gates: Vec<Gate>,
    intern: BTreeMap<Gate, Bit>,
    inputs: Vec<Word>,
    start: usize,
}
impl Circuit {
    fn new(widths: &[usize]) -> Self {
        let mut gates = vec![Gate::False, Gate::True];
        let inputs = widths
            .iter()
            .enumerate()
            .map(|(arg, &w)| {
                (0..w)
                    .map(|bit| {
                        let id = gates.len();
                        gates.push(Gate::Input(arg, bit));
                        id
                    })
                    .collect()
            })
            .collect();
        while gates.len() % 32 != 0 {
            gates.push(Gate::False);
        }
        let start = gates.len();
        Self {
            gates,
            intern: BTreeMap::new(),
            inputs,
            start,
        }
    }
    fn gate(&mut self, g: Gate) -> Bit {
        if let Some(&id) = self.intern.get(&g) {
            return id;
        }
        let id = self.gates.len();
        self.gates.push(g);
        self.intern.insert(g, id);
        id
    }
    fn not(&mut self, a: Bit) -> Bit {
        match a {
            F => T,
            T => F,
            _ => match self.gates[a] {
                Gate::Not(b) => b,
                _ => self.gate(Gate::Not(a)),
            },
        }
    }
    fn and(&mut self, a: Bit, b: Bit) -> Bit {
        if a == F || b == F {
            F
        } else if a == T || a == b {
            b
        } else if b == T {
            a
        } else {
            self.gate(Gate::And(a.min(b), a.max(b)))
        }
    }
    fn xor(&mut self, a: Bit, b: Bit) -> Bit {
        if a == b {
            F
        } else if a == F {
            b
        } else if b == F {
            a
        } else if a == T {
            self.not(b)
        } else if b == T {
            self.not(a)
        } else {
            self.gate(Gate::Xor(a.min(b), a.max(b)))
        }
    }
    fn or(&mut self, a: Bit, b: Bit) -> Bit {
        let n = self.xor(a, b);
        let c = self.and(a, b);
        self.xor(n, c)
    }
    fn mux(&mut self, c: Bit, t: Bit, e: Bit) -> Bit {
        if c == T || t == e {
            t
        } else if c == F {
            e
        } else if t == T && e == F {
            c
        } else {
            self.gate(Gate::Mux(c, t, e))
        }
    }
    fn select(&mut self, c: Bit, t: &[Bit], e: &[Bit]) -> Word {
        assert_eq!(t.len(), e.len());
        t.iter().zip(e).map(|(&t, &e)| self.mux(c, t, e)).collect()
    }
    fn nonzero(&mut self, a: &[Bit]) -> Bit {
        a.iter().fold(F, |x, &y| self.or(x, y))
    }
    fn equal(&mut self, a: &[Bit], b: &[Bit]) -> Bit {
        assert_eq!(a.len(), b.len());
        let ds = a
            .iter()
            .zip(b)
            .map(|(&a, &b)| self.xor(a, b))
            .collect::<Word>();
        let n = self.nonzero(&ds);
        self.not(n)
    }
    fn add(&mut self, a: &[Bit], b: &[Bit], mut carry: Bit) -> (Word, Bit) {
        assert_eq!(a.len(), b.len());
        let mut out = vec![];
        for (&a, &b) in a.iter().zip(b) {
            let x = self.xor(a, b);
            out.push(self.xor(x, carry));
            let u = self.and(x, carry);
            let v = self.and(a, b);
            carry = self.or(u, v);
        }
        (out, carry)
    }
    fn sub(&mut self, a: &[Bit], b: &[Bit]) -> (Word, Bit) {
        let b = b.iter().map(|&b| self.not(b)).collect::<Word>();
        self.add(a, &b, T)
    }
    fn neg(&mut self, a: &[Bit]) -> Word {
        self.sub(&vec![F; a.len()], a).0
    }
    fn lt(&mut self, a: &[Bit], b: &[Bit], signed: bool) -> Bit {
        let c = self.sub(a, b).1;
        let less = self.not(c);
        if signed {
            let i = a.len() - 1;
            let signs = self.xor(a[i], b[i]);
            self.mux(signs, a[i], less)
        } else {
            less
        }
    }
    fn extend(a: &[Bit], n: usize, signed: bool) -> Word {
        let mut out = a[..a.len().min(n)].to_vec();
        out.resize(n, if signed { *a.last().unwrap() } else { F });
        out
    }
    fn multiply(&mut self, a: &[Bit], b: &[Bit], n: usize) -> Word {
        let mut sum = vec![F; n];
        for (i, &bit) in b.iter().take(n).enumerate() {
            let mut row = vec![F; i];
            row.extend(a.iter().take(n - i).map(|&x| self.and(x, bit)));
            row.resize(n, F);
            sum = self.add(&sum, &row, F).0;
        }
        sum
    }
    fn divrem(&mut self, a: &[Bit], b: &[Bit]) -> (Word, Word) {
        let n = a.len();
        let divisor = Self::extend(b, n + 1, false);
        let mut rem = vec![F; n + 1];
        let mut q = vec![F; n];
        for i in (0..n).rev() {
            rem.insert(0, a[i]);
            rem.pop();
            let (diff, ge) = self.sub(&rem, &divisor);
            q[i] = ge;
            rem = self.select(ge, &diff, &rem);
        }
        rem.pop();
        (q, rem)
    }
    fn shift(&mut self, a: &[Bit], count: &[Bit], right: bool, arithmetic: bool) -> Word {
        let mut x = a.to_vec();
        for (i, &c) in count.iter().take(a.len().ilog2() as usize).enumerate() {
            let k = 1usize << i;
            let fill = if arithmetic { *a.last().unwrap() } else { F };
            let y = (0..a.len())
                .map(|j| {
                    if right {
                        x.get(j + k).copied().unwrap_or(fill)
                    } else {
                        j.checked_sub(k).map_or(F, |j| x[j])
                    }
                })
                .collect::<Word>();
            x = self.select(c, &y, &x);
        }
        x
    }
    fn prune(&mut self, outputs: &mut [Bit]) {
        let mut live = vec![false; self.gates.len()];
        for &id in outputs.iter() {
            live[id] = true;
        }
        for i in (self.start..self.gates.len()).rev() {
            if live[i] {
                match self.gates[i] {
                    Gate::Not(a) => live[a] = true,
                    Gate::And(a, b) | Gate::Xor(a, b) => {
                        live[a] = true;
                        live[b] = true
                    }
                    Gate::Mux(c, t, e) => {
                        live[c] = true;
                        live[t] = true;
                        live[e] = true
                    }
                    _ => {}
                }
            }
        }
        let mut ids = (0..self.gates.len()).collect::<Vec<_>>();
        let mut gates = self.gates[..self.start].to_vec();
        for i in self.start..self.gates.len() {
            if live[i] {
                ids[i] = gates.len();
                gates.push(match self.gates[i] {
                    Gate::Not(a) => Gate::Not(ids[a]),
                    Gate::And(a, b) => Gate::And(ids[a], ids[b]),
                    Gate::Xor(a, b) => Gate::Xor(ids[a], ids[b]),
                    Gate::Mux(c, t, e) => Gate::Mux(ids[c], ids[t], ids[e]),
                    _ => unreachable!(),
                });
            }
        }
        for id in outputs {
            *id = ids[*id];
        }
        self.gates = gates;
        self.intern.clear();
        for (i, &g) in self.gates.iter().enumerate().skip(self.start) {
            self.intern.insert(g, i);
        }
    }
    #[cfg(test)]
    fn evaluate(&self, inputs: &[u128]) -> Vec<bool> {
        let mut v: Vec<bool> = Vec::with_capacity(self.gates.len());
        for g in &self.gates {
            let x = match *g {
                Gate::False => false,
                Gate::True => true,
                Gate::Input(a, b) => (inputs[a] >> b) & 1 != 0,
                Gate::Not(a) => !v[a],
                Gate::And(a, b) => v[a] && v[b],
                Gate::Xor(a, b) => v[a] ^ v[b],
                Gate::Mux(c, t, e) => {
                    if v[c] {
                        v[t]
                    } else {
                        v[e]
                    }
                }
            };
            v.push(x);
        }
        v
    }
}
fn integral_shape(id: &str) -> R<(usize, bool)> {
    let p = id
        .strip_prefix("mpk.csharp.value.")
        .and_then(|s| s.strip_suffix(".v1"))
        .ok_or(OrdinaryCarrierError::Shape)?;
    Ok(match p {
        "bool" => (1, false),
        "i8" => (8, true),
        "u8" => (8, false),
        "i16" => (16, true),
        "u16" | "char" => (16, false),
        "i32" => (32, true),
        "u32" => (32, false),
        "i64" => (64, true),
        "u64" => (64, false),
        _ => return Err(OrdinaryCarrierError::Shape),
    })
}
struct IntegerCircuit {
    signature: ClosedOperationSignature,
    circuit: Circuit,
    output: Word,
    failures: Vec<Bit>,
}
fn integer_circuit(id: &str) -> R<IntegerCircuit> {
    let signature = scalar_operation_signature(id).map_err(|_| OrdinaryCarrierError::Shape)?;
    let shapes = signature
        .argument_type_ids
        .iter()
        .map(|t| integral_shape(t))
        .collect::<R<Vec<_>>>()?;
    let mut c = Circuit::new(&shapes.iter().map(|s| s.0).collect::<Vec<_>>());
    let a = c.inputs[0].clone();
    let b = c.inputs.get(1).cloned().unwrap_or_default();
    let n = a.len();
    let signed = shapes[0].1;
    let mut overflow = F;
    let mut zero = F;
    let output = if id.starts_with("integer.convert.") {
        let (w, s) = integral_shape(&signature.normal_result_type_id)?;
        let out = Circuit::extend(&a, w, signed);
        let wide = n.max(w) + 1;
        let original = Circuit::extend(&a, wide, signed);
        let roundtrip = Circuit::extend(&out, wide, s);
        let eq = c.equal(&original, &roundtrip);
        overflow = c.not(eq);
        out
    } else {
        let op = if id.starts_with("boolean.") {
            id.split('.').nth(1).unwrap()
        } else {
            id.split('.').nth(2).unwrap()
        };
        match op {
            "plus" => a.clone(),
            "not" => a.iter().map(|&a| c.not(a)).collect(),
            "and" | "or" | "xor" => a
                .iter()
                .zip(&b)
                .map(|(&a, &b)| match op {
                    "and" => c.and(a, b),
                    "or" => c.or(a, b),
                    _ => c.xor(a, b),
                })
                .collect(),
            "equal" | "not_equal" => {
                let eq = c.equal(&a, &b);
                vec![if op == "equal" { eq } else { c.not(eq) }]
            }
            "less" | "less_equal" | "greater" | "greater_equal" => {
                let swap = matches!(op, "less_equal" | "greater");
                let less = if swap {
                    c.lt(&b, &a, signed)
                } else {
                    c.lt(&a, &b, signed)
                };
                vec![if matches!(op, "less_equal" | "greater_equal") {
                    c.not(less)
                } else {
                    less
                }]
            }
            "add" | "subtract" | "negate" | "multiply" => {
                let wide = if op == "multiply" { 2 * n } else { n + 1 };
                let x = Circuit::extend(&a, wide, signed);
                let y = Circuit::extend(&b, wide, signed && !b.is_empty());
                let value = match op {
                    "add" => c.add(&x, &y, F).0,
                    "subtract" => c.sub(&x, &y).0,
                    "negate" => c.neg(&x),
                    _ => {
                        // (a - sign(a)*2^n)(b - sign(b)*2^n), modulo 2^(2n).
                        // The sign(a)*sign(b)*2^(2n) term vanishes. This avoids
                        // multiplying duplicated sign-extension bits.
                        let mut product = c.multiply(&a, &b, wide);
                        if signed {
                            for (sign, word) in [(a[n - 1], &b), (b[n - 1], &a)] {
                                let correction = std::iter::repeat_n(F, n)
                                    .chain(word.iter().copied())
                                    .collect::<Word>();
                                let correction = c.select(sign, &correction, &vec![F; wide]);
                                product = c.sub(&product, &correction).0;
                            }
                        }
                        product
                    }
                };
                let out = value[..n].to_vec();
                let ext = Circuit::extend(&out, wide, signed);
                let eq = c.equal(&value, &ext);
                overflow = c.not(eq);
                out
            }
            "divide" | "remainder" => {
                let nz = c.nonzero(&b);
                zero = c.not(nz);
                let sa = if signed { a[n - 1] } else { F };
                let sb = if signed { b[n - 1] } else { F };
                let na = c.neg(&a);
                let nb = c.neg(&b);
                let x = c.select(sa, &na, &a);
                let y = c.select(sb, &nb, &b);
                let (q, r) = c.divrem(&x, &y);
                let out = if op == "divide" {
                    let sign = c.xor(sa, sb);
                    let neg = c.neg(&q);
                    c.select(sign, &neg, &q)
                } else {
                    let neg = c.neg(&r);
                    c.select(sa, &neg, &r)
                };
                if signed {
                    let mut min = vec![F; n];
                    min[n - 1] = T;
                    let is_min = c.equal(&a, &min);
                    let is_neg1 = c.equal(&b, &vec![T; n]);
                    overflow = c.and(is_min, is_neg1);
                }
                out
            }
            "left_shift" | "right_shift" | "unsigned_right_shift" => {
                c.shift(&a, &b, op != "left_shift", signed && op == "right_shift")
            }
            _ => return Err(OrdinaryCarrierError::Shape),
        }
    };
    let failures = signature
        .ordered_checks
        .iter()
        .map(|s| match s.id.as_str() {
            "exception.overflow" => Ok(overflow),
            "exception.division_by_zero" => Ok(zero),
            _ => Err(OrdinaryCarrierError::Shape),
        })
        .collect::<R<Vec<_>>>()?;
    Ok(IntegerCircuit {
        signature,
        circuit: c,
        output,
        failures,
    })
}

// Register-state lowering prevents a long arithmetic DAG from becoming a deep
// core expression. Each block binds at most 32 Bool gates and updates one
// aligned register block. Only Bool recursors occur at the leaves.
fn core_mux(b: &mut Builder, c: u32, t: u32, e: u32) -> R<u32> {
    let rec = b.constant("Std.Bool.rec")?;
    b.app(rec, vec![e, t, c])
}
fn core_read(b: &mut Builder, state: u32, index: usize, d: u32) -> R<u32> {
    let f = b.constant("Std.Bool.false")?;
    let t = b.constant("Std.Bool.true")?;
    b.app(
        state,
        (0..d)
            .map(|i| if index & (1usize << i) == 0 { f } else { t })
            .collect(),
    )
}
fn core_select(
    b: &mut Builder,
    values: &[u32],
    selector_depth: u32,
    bit: u32,
    end: u32,
    default: u32,
) -> R<u32> {
    if values.is_empty() {
        return Ok(default);
    }
    if bit == end {
        return Ok(values[0]);
    }
    let even = values.iter().step_by(2).copied().collect::<Vec<_>>();
    let odd = values
        .iter()
        .skip(1)
        .step_by(2)
        .copied()
        .collect::<Vec<_>>();
    let no = core_select(b, &even, selector_depth, bit + 1, end, default)?;
    let yes = core_select(b, &odd, selector_depth, bit + 1, end, default)?;
    if yes == no {
        return Ok(yes);
    }
    let cond = b.var(selector_depth - 1 - bit)?;
    core_mux(b, cond, yes, no)
}
fn core_gate(b: &mut Builder, g: Gate, start: usize, bound: usize, d: u32) -> R<u32> {
    let mut get = |id: usize| {
        if id >= start {
            b.var((bound - 1 - (id - start)) as u32)
        } else {
            let state = b.var(bound as u32)?;
            core_read(b, state, id, d)
        }
    };
    let (name, args) = match g {
        Gate::Not(a) => ("Std.Bool.not", vec![get(a)?]),
        Gate::And(a, c) => ("Std.Bool.and", vec![get(a)?, get(c)?]),
        Gate::Xor(a, c) => {
            let a = get(a)?;
            let c = get(c)?;
            let not = b.constant("Std.Bool.not")?;
            let n = b.app(not, vec![a])?;
            return core_mux(b, c, n, a);
        }
        Gate::Mux(c, t, e) => {
            let c = get(c)?;
            let t = get(t)?;
            let e = get(e)?;
            return core_mux(b, c, t, e);
        }
        _ => return Err(OrdinaryCarrierError::Shape),
    };
    let head = b.constant(name)?;
    b.app(head, args)
}
fn bind_inputs(b: &mut Builder, shapes: &[usize], mut body: u32) -> R<u32> {
    for &w in shapes.iter().rev() {
        let ty = b.cube(address_bits(w as u32))?;
        body = b.lam(ty, body)?;
    }
    Ok(body)
}
fn input_type(b: &mut Builder, shapes: &[usize], mut result: u32) -> R<u32> {
    for &w in shapes.iter().rev() {
        let ty = b.cube(address_bits(w as u32))?;
        result = b.pi(ty, result)?;
    }
    Ok(result)
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryScalarDefinition {
    pub operation: ClosedOperationSignature,
    pub result_definition: String,
    pub success_definition: String,
    pub ordered_failure_definitions: Vec<String>,
    pub boolean_gates: usize,
    pub state_depth: u32,
    pub static_transformers: usize,
}
pub type OrdinaryIntegerDefinition = OrdinaryScalarDefinition;
fn emit_integer(b: &mut Builder, id: &str) -> R<OrdinaryIntegerDefinition> {
    emit_circuit(b, integer_circuit(id)?, "Integer")
}
fn emit_circuit(
    b: &mut Builder,
    mut p: IntegerCircuit,
    namespace: &str,
) -> R<OrdinaryScalarDefinition> {
    let mut success = T;
    for f in &mut p.failures {
        let raw = *f;
        *f = p.circuit.and(success, raw);
        let no = p.circuit.not(raw);
        success = p.circuit.and(success, no);
    }
    p.output = p
        .circuit
        .select(success, &p.output, &vec![F; p.output.len()]);
    let mut roots = p
        .output
        .iter()
        .copied()
        .chain(p.failures.iter().copied())
        .chain(std::iter::once(success))
        .collect::<Vec<_>>();
    p.circuit.prune(&mut roots);
    let output_len = p.output.len();
    p.output.copy_from_slice(&roots[..output_len]);
    let offset = p.output.len();
    let failures = p.failures.len();
    p.failures
        .copy_from_slice(&roots[offset..offset + failures]);
    success = *roots.last().unwrap();
    let c = &p.circuit;
    if c.gates.len() > 262144 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let d = address_bits(c.gates.len() as u32);
    let s = b.cube(d)?;
    if !b
        .globals
        .contains_key(&format!("{PREFIX}.Cube.D{d}.Compose"))
    {
        b.helpers(d)?;
    }
    let name = format!(
        "{PREFIX}.{namespace}.O{}",
        p.signature
            .id
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let mut steps = vec![];
    for start in (c.start..c.gates.len()).step_by(32) {
        let count = (c.gates.len() - start).min(32);
        let mut bindings = vec![];
        for j in 0..count {
            bindings.push(core_gate(b, c.gates[start + j], start, j, d)?);
        }
        let old = b.var(d + count as u32)?;
        let args = b.selectors(d)?;
        let old = b.app(old, args)?;
        let values = (0..count)
            .map(|j| b.var(d + (count - 1 - j) as u32))
            .collect::<R<Vec<_>>>()?;
        let f = b.constant("Std.Bool.false")?;
        // Select low address bits while retaining the complete selector scope.
        let value = core_select(b, &values, d, 0, 5, f)?;
        let mut block = b.constant("Std.Bool.true")?;
        for i in 5..d {
            let v = b.var(d - 1 - i)?;
            let v = if start & (1usize << i) == 0 {
                let not = b.constant("Std.Bool.not")?;
                b.app(not, vec![v])?
            } else {
                v
            };
            let and = b.constant("Std.Bool.and")?;
            block = b.app(and, vec![block, v])?;
        }
        let body = core_mux(b, block, value, old)?;
        let mut body = b.wrap_selectors(d, body)?;
        for &value in bindings.iter().rev() {
            body = b.term(TermNode::Let {
                ty: b.boolean,
                value,
                body,
            })?;
        }
        let body = b.lam(s, body)?;
        let ty = b.pi(s, s)?;
        let step = format!("{name}.Block.B{}", start / 32);
        b.define(&step, ty, body)?;
        steps.push(b.constant(&step)?);
    }
    let composed = b.compose(d, &steps)?;
    let widths = c.inputs.iter().map(Vec::len).collect::<Vec<_>>();
    let f = b.constant("Std.Bool.false")?;
    let t = b.constant("Std.Bool.true")?;
    let mut initial = vec![];
    for g in &c.gates[..c.start] {
        initial.push(match *g {
            Gate::False => f,
            Gate::True => t,
            Gate::Input(arg, bit) => {
                let x = b.var(d + (widths.len() - 1 - arg) as u32)?;
                core_read(b, x, bit, address_bits(widths[arg] as u32))?
            }
            _ => return Err(OrdinaryCarrierError::Shape),
        });
    }
    let initial = core_select(b, &initial, d, 0, d, f)?;
    let initial = b.wrap_selectors(d, initial)?;
    let final_state = b.app(composed, vec![initial])?;
    let body = bind_inputs(b, &widths, final_state)?;
    let ty = input_type(b, &widths, s)?;
    let state_name = format!("{name}.State");
    b.define(&state_name, ty, body)?;
    let state = b.constant(&state_name)?;
    let args = (0..widths.len())
        .rev()
        .map(|i| b.var(i as u32))
        .collect::<R<Vec<_>>>()?;
    let state = b.app(state, args)?;
    let result_definition = format!("{name}.Result");
    let success_definition = format!("{name}.Success");
    let ordered_failure_definitions = (0..p.failures.len())
        .map(|i| format!("{name}.Failure.F{i}"))
        .collect::<Vec<_>>();
    let outputs = std::iter::once((result_definition.clone(), p.output.clone()))
        .chain(std::iter::once((success_definition.clone(), vec![success])))
        .chain(
            ordered_failure_definitions
                .iter()
                .cloned()
                .zip(p.failures.iter().map(|&f| vec![f])),
        );
    for (name, ids) in outputs {
        let q = address_bits(ids.len() as u32);
        let x = b.var(q)?;
        let values = ids
            .iter()
            .map(|&i| core_read(b, x, i, d))
            .collect::<R<Vec<_>>>()?;
        let body = core_select(b, &values, q, 0, q, f)?;
        let body = b.wrap_selectors(q, body)?;
        let body = b.term(TermNode::Let {
            ty: s,
            value: state,
            body,
        })?;
        let body = bind_inputs(b, &widths, body)?;
        let ty = b.cube(q)?;
        let ty = input_type(b, &widths, ty)?;
        b.define(&name, ty, body)?;
    }
    Ok(OrdinaryIntegerDefinition {
        operation: p.signature,
        result_definition,
        success_definition,
        ordered_failure_definitions,
        boolean_gates: c.gates.len() - c.start,
        state_depth: d,
        static_transformers: steps.len(),
    })
}
/// Partial W09 scalar component. This capability covers exactly the Boolean and
/// integer signatures in the original VIR, not numeric/business operations or VCs.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryIntegerProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryIntegerDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryIntegerProgram {
    pub fn definitions(&self) -> &[OrdinaryIntegerDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed integer program")
    }
}
pub fn generate_csharp_practical_ordinary_integers(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryIntegerProgram> {
    let ids = vir
        .operation_signatures()
        .iter()
        .filter(|s| s.id.starts_with("boolean.") || s.id.starts_with("integer."))
        .map(|s| s.id.clone())
        .collect::<BTreeSet<_>>();
    let mut b = Builder::new()?;
    let definitions = ids
        .iter()
        .map(|id| emit_integer(&mut b, id))
        .collect::<R<Vec<_>>>()?;
    let certificate = b.finish()?;
    let p = OrdinaryIntegerProgram {
        schema: "mpk.csharp.ordinary_integers.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_integers(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryIntegerProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_integers(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn observed(p: &IntegerCircuit, args: &[u128]) -> (u128, Option<usize>) {
        let v = p.circuit.evaluate(args);
        (
            p.output
                .iter()
                .enumerate()
                .fold(0, |n, (i, &b)| n | ((v[b] as u128) << i)),
            p.failures.iter().position(|&b| v[b]),
        )
    }
    #[test]
    fn integer_circuits_match_widened_arithmetic_and_error_order() {
        for token in ["i32", "u32", "i64", "u64"] {
            let signed = token.starts_with('i');
            let n = if token.ends_with("32") { 32 } else { 64 };
            let mask = (1u128 << n) - 1;
            let signed_value = |x: u128| {
                if signed && x >> (n - 1) != 0 {
                    (x as i128) - (1i128 << n)
                } else {
                    x as i128
                }
            };
            let mut samples = vec![
                0,
                1,
                2,
                mask,
                mask - 1,
                1u128 << (n - 1),
                (1u128 << (n - 1)) - 1,
            ];
            let mut seed = 0x6a09e667f3bcc909u64;
            for _ in 0..12 {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                samples.push(u128::from(seed) & mask);
            }
            for mode in ["checked", "unchecked"] {
                for op in [
                    "add",
                    "subtract",
                    "multiply",
                    "divide",
                    "remainder",
                    "less",
                    "greater_equal",
                    "less_equal",
                    "greater",
                    "equal",
                    "not_equal",
                    "and",
                    "or",
                    "xor",
                    "left_shift",
                    "right_shift",
                    "unsigned_right_shift",
                ] {
                    let p = integer_circuit(&format!("integer.{token}.{op}.{mode}")).unwrap();
                    for &a in &samples {
                        for &b in &samples {
                            let x = signed_value(a);
                            let y = signed_value(b);
                            let min = if signed { -(1i128 << (n - 1)) } else { 0 };
                            let max = if signed {
                                (1i128 << (n - 1)) - 1
                            } else {
                                mask as i128
                            };
                            let shift = b as u32 & (n - 1);
                            let exact = match op {
                                "add" => Some(x + y),
                                "subtract" => Some(x - y),
                                "multiply" => x.checked_mul(y),
                                "divide" if y != 0 => Some(x / y),
                                "remainder" if y != 0 => Some(x % y),
                                "less" => Some((x < y) as i128),
                                "less_equal" => Some((x <= y) as i128),
                                "greater" => Some((x > y) as i128),
                                "equal" => Some((x == y) as i128),
                                "not_equal" => Some((x != y) as i128),
                                "and" => Some((a & b) as i128),
                                "or" => Some((a | b) as i128),
                                "xor" => Some((a ^ b) as i128),
                                "greater_equal" => Some((x >= y) as i128),
                                "left_shift" => Some((a << shift) as i128),
                                "right_shift" => Some(x >> shift),
                                "unsigned_right_shift" => Some((a >> shift) as i128),
                                _ => Some(0),
                            };
                            let divide = matches!(op, "divide" | "remainder");
                            let error = if divide && b == 0 {
                                Some(0)
                            } else if divide && signed && x == min && y == -1 {
                                Some(1)
                            } else if mode == "checked"
                                && matches!(op, "add" | "subtract" | "multiply")
                                && exact.is_none_or(|v| v < min || v > max)
                            {
                                Some(0)
                            } else {
                                None
                            };
                            let mut args = vec![a, b];
                            if op.contains("shift") {
                                args[1] = b & 0xffff_ffff;
                            }
                            let (out, err) = observed(&p, &args);
                            assert_eq!(err, error, "{token}.{op}.{mode} {a} {b}");
                            if error.is_none() {
                                let expected = if op == "multiply" {
                                    a.wrapping_mul(b) & mask
                                } else {
                                    exact.unwrap() as u128 & mask
                                };
                                assert_eq!(out, expected, "{token}.{op}.{mode} {a} {b}");
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn integer_circuits_conversions_and_unary_edges() {
        let tokens = ["i8", "u8", "i16", "u16", "char", "i32", "u32", "i64", "u64"];
        for from in tokens {
            for to in tokens {
                for mode in ["checked", "unchecked"] {
                    let id = format!("integer.convert.{from}.{to}.{mode}");
                    let p = integer_circuit(&id).unwrap();
                    let (n, s) = integral_shape(&p.signature.argument_type_ids[0]).unwrap();
                    let (w, t) = integral_shape(&p.signature.normal_result_type_id).unwrap();
                    for a in [
                        0,
                        1,
                        (1u128 << n) - 1,
                        1u128 << (n - 1),
                        (1u128 << (n - 1)) - 1,
                    ] {
                        let x = if s && a >> (n - 1) != 0 {
                            a as i128 - (1i128 << n)
                        } else {
                            a as i128
                        };
                        let lo = if t { -(1i128 << (w - 1)) } else { 0 };
                        let hi = if t {
                            (1i128 << (w - 1)) - 1
                        } else {
                            (1i128 << w) - 1
                        };
                        let (out, err) = observed(&p, &[a]);
                        assert_eq!(
                            err,
                            if mode == "checked" && (x < lo || x > hi) {
                                Some(0)
                            } else {
                                None
                            },
                            "{id} {a}"
                        );
                        assert_eq!(out, x as u128 & ((1u128 << w) - 1));
                    }
                }
            }
        }
        for token in ["i32", "u32", "i64", "u64"] {
            for op in ["plus", "negate", "not"] {
                for mode in ["checked", "unchecked"] {
                    let p = integer_circuit(&format!("integer.{token}.{op}.{mode}")).unwrap();
                    let (n, s) = integral_shape(&p.signature.normal_result_type_id).unwrap();
                    let mask = (1u128 << n) - 1;
                    for a in [0, 1, mask, 1u128 << (n - 1)] {
                        let (out, err) = observed(&p, &[a]);
                        let overflow = op == "negate"
                            && mode == "checked"
                            && if s { a == 1u128 << (n - 1) } else { a != 0 };
                        assert_eq!(err, overflow.then_some(0));
                        assert_eq!(
                            out,
                            match op {
                                "plus" => a,
                                "not" => !a & mask,
                                _ => 0u128.wrapping_sub(a) & mask,
                            }
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn boolean_circuits_core_exhaustive_truth_tables() {
        use super::super::tests::{bit, run, V};
        let mut b = Builder::new().unwrap();
        let defs = ["not", "and", "or", "xor", "equal", "not_equal"]
            .map(|op| emit_integer(&mut b, &format!("boolean.{op}")).unwrap());
        let bytes = b.finish().unwrap();
        let cert = decode_canonical_certificate(&bytes).unwrap();
        let mut cases = 0;
        for d in &defs {
            for a in [false, true] {
                for v in [false, true] {
                    let op = d.operation.id.strip_prefix("boolean.").unwrap();
                    if op == "not" && v {
                        continue;
                    }
                    let expected = match op {
                        "not" => !a,
                        "and" => a && v,
                        "or" => a || v,
                        "xor" => a ^ v,
                        "equal" => a == v,
                        "not_equal" => a != v,
                        _ => unreachable!(),
                    };
                    let args = if op == "not" {
                        vec![V::Bit(a)]
                    } else {
                        vec![V::Bit(a), V::Bit(v)]
                    };
                    assert!(d.ordered_failure_definitions.is_empty());
                    assert!(bit(run(&cert, &d.success_definition, args.clone())));
                    assert_eq!(
                        bit(run(&cert, &d.result_definition, args)),
                        expected,
                        "{op} {a} {v}"
                    );
                    cases += 1;
                }
            }
        }
        assert_eq!(cases, 22);
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/boolean-circuits");
        let out = std::env::var_os("MPK_W09_BOOLEAN_OUT").map(std::path::PathBuf::from);
        let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
        let metrics = serde_json::json!({"operations":defs,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"certificate_sha256":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes)),"truth_cases":cases});
        if let Some(out) = out {
            std::fs::create_dir_all(&out).unwrap();
            std::fs::write(out.join("boolean.all.hex"), hex).unwrap();
            std::fs::write(
                out.join("metrics.json"),
                serde_json::to_vec_pretty(&metrics).unwrap(),
            )
            .unwrap();
        } else {
            assert_eq!(
                std::fs::read_to_string(root.join("boolean.all.hex")).unwrap(),
                hex
            );
            let expected: Value =
                serde_json::from_slice(&std::fs::read(root.join("metrics.json")).unwrap()).unwrap();
            assert_eq!(metrics, expected);
        }
    }
    #[test]
    fn integer_circuits_core_evaluation_matches_network() {
        use super::super::tests::{apply, bit, run, V};
        for id in [
            "boolean.xor",
            "integer.i32.add.checked",
            "integer.convert.i64.u8.checked",
            "integer.i64.right_shift.unchecked",
        ] {
            let p = integer_circuit(id).unwrap();
            let mut b = Builder::new().unwrap();
            let def = emit_integer(&mut b, id).unwrap();
            let cert = decode_canonical_certificate(&b.finish().unwrap()).unwrap();
            for raw in [
                [0, 1],
                [127, 1],
                [u32::MAX as u128, 7],
                [1u128 << 31, 1u128 << 31],
                [u64::MAX as u128, 63],
            ] {
                let args = p
                    .circuit
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(a, w)| {
                        if w.len() == 1 {
                            V::Bit(raw[a] & 1 != 0)
                        } else {
                            V::Cube((0..w.len()).map(|i| raw[a] & (1u128 << i) != 0).collect())
                        }
                    })
                    .collect::<Vec<_>>();
                let (out, error) = observed(&p, &raw);
                assert_eq!(
                    bit(run(&cert, &def.success_definition, args.clone())),
                    error.is_none(),
                    "{id}"
                );
                for (i, name) in def.ordered_failure_definitions.iter().enumerate() {
                    assert_eq!(bit(run(&cert, name, args.clone())), error == Some(i));
                }
                let value = run(&cert, &def.result_definition, args);
                let width = p.output.len();
                let depth = address_bits(width as u32);
                for i in 0..width {
                    let mut v = value.clone();
                    for j in 0..depth {
                        v = apply(&cert, v, V::Bit(i & (1usize << j) != 0));
                    }
                    assert_eq!(
                        bit(v),
                        error.is_none() && out & (1u128 << i) != 0,
                        "{id} bit {i}"
                    );
                }
            }
        }
    }
    #[test]
    fn integer_circuits_emit_every_closed_signature_within_limits() {
        // The seven pinned checker examples do not establish that every
        // retained width/mode/conversion can actually be lowered within the
        // frozen certificate limits. Exercise the complete closed family.
        let mut ids = vec![];
        for token in ["i32", "u32", "i64", "u64"] {
            for op in [
                "plus",
                "negate",
                "not",
                "equal",
                "not_equal",
                "less",
                "less_equal",
                "greater",
                "greater_equal",
                "left_shift",
                "right_shift",
                "unsigned_right_shift",
                "add",
                "subtract",
                "multiply",
                "divide",
                "remainder",
                "and",
                "or",
                "xor",
            ] {
                for mode in ["checked", "unchecked"] {
                    ids.push(format!("integer.{token}.{op}.{mode}"));
                }
            }
        }
        let tokens = ["i8", "u8", "i16", "u16", "char", "i32", "u32", "i64", "u64"];
        for from in tokens {
            for to in tokens {
                for mode in ["checked", "unchecked"] {
                    ids.push(format!("integer.convert.{from}.{to}.{mode}"));
                }
            }
        }
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 322);
        let mut metrics = vec![];
        for id in ids {
            let mut b = Builder::new().unwrap();
            let definition = emit_integer(&mut b, &id).unwrap_or_else(|e| panic!("{id}: {e:?}"));
            let static_transformers = b.static_transformers;
            let bytes = b.finish().unwrap_or_else(|e| panic!("{id}: {e:?}"));
            let cert = decode_canonical_certificate(&bytes).unwrap();
            crate::csharp_practical_vc_model::validate_csharp_practical_certificate_structure(
                &cert,
            )
            .unwrap_or_else(|e| panic!("{id}: {e:?}"));
            metrics.push(serde_json::json!({
                "id": id, "definition": definition,
                "terms": cert.term_table.len(), "declarations": cert.declarations.len(),
                "static_transformers": static_transformers,
                "certificate_sha256": mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))
            }));
        }
        let metrics = serde_json::json!(metrics);
        if let Some(out) = std::env::var_os("MPK_W09_INTEGER_COVERAGE_OUT") {
            std::fs::write(out, serde_json::to_vec_pretty(&metrics).unwrap()).unwrap();
        } else {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
                "../../develop/migrations/csharp-03/ordinary-foundation/integer-coverage.json",
            );
            let expected: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
            assert_eq!(metrics, expected);
        }
    }
    #[test]
    fn integer_circuits_emit_actual_core() {
        let fixture_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/integer-circuits");
        let mut metrics = vec![];
        for id in [
            "boolean.xor",
            "integer.i32.add.checked",
            "integer.i64.multiply.checked",
            "integer.u64.divide.unchecked",
            "integer.i32.remainder.unchecked",
            "integer.convert.i64.u8.checked",
            "integer.i64.right_shift.unchecked",
        ] {
            let mut b = Builder::new().unwrap();
            let p = emit_integer(&mut b, id).unwrap_or_else(|e| panic!("{id}: {e:?}"));
            let bytes = b.finish().unwrap_or_else(|e| panic!("{id}: {e:?}"));
            let c = decode_canonical_certificate(&bytes).unwrap();
            metrics.push(serde_json::json!({"id":id,"definition":p,"terms":c.term_table.len(),"declarations":c.declarations.len(),"hash":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))}));
            let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
            if std::env::var_os("MPK_W09_INTEGER_OUT").is_none() {
                assert_eq!(
                    std::fs::read_to_string(fixture_root.join(format!("{id}.hex"))).unwrap(),
                    hex,
                    "{id}"
                );
            }
            if let Ok(dir) = std::env::var("MPK_W09_INTEGER_OUT") {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(
                    std::path::Path::new(&dir).join(format!("{id}.hex")),
                    bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n",
                )
                .unwrap();
            }
        }
        if std::env::var_os("MPK_W09_INTEGER_OUT").is_none() {
            let expected: Value =
                serde_json::from_slice(&std::fs::read(fixture_root.join("metrics.json")).unwrap())
                    .unwrap();
            assert_eq!(serde_json::json!(metrics), expected);
        }
        if let Ok(dir) = std::env::var("MPK_W09_INTEGER_OUT") {
            std::fs::write(
                std::path::Path::new(&dir).join("metrics.json"),
                serde_json::to_vec_pretty(&metrics).unwrap(),
            )
            .unwrap();
        }
    }
}

#[path = "csharp_practical_ordinary_temporal.rs"]
mod temporal;
pub use temporal::{
    generate_csharp_practical_ordinary_temporal, import_csharp_practical_ordinary_temporal,
    OrdinaryTemporalProgram,
};

#[path = "csharp_practical_ordinary_calendar.rs"]
mod calendar;
pub use calendar::{
    generate_csharp_practical_ordinary_calendar, import_csharp_practical_ordinary_calendar,
    OrdinaryCalendarProgram,
};

#[path = "csharp_practical_ordinary_float.rs"]
mod floating;
pub use floating::{
    generate_csharp_practical_ordinary_floating, import_csharp_practical_ordinary_floating,
    OrdinaryFloatingProgram,
};

#[path = "csharp_practical_ordinary_decimal.rs"]
mod decimal;
pub use decimal::{
    generate_csharp_practical_ordinary_decimal, import_csharp_practical_ordinary_decimal,
    OrdinaryDecimalProgram,
};

#[path = "csharp_practical_ordinary_string.rs"]
mod utf16;
pub use utf16::{
    generate_csharp_practical_ordinary_strings, import_csharp_practical_ordinary_strings,
    OrdinaryStringDefinition, OrdinaryStringProgram,
};
