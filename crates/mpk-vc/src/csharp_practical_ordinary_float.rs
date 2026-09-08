//! Finite IEEE binary32/binary64 definitions. Arithmetic is ordinary Boolean
//! circuitry; the T03 evaluator is used only as an independent test oracle.
use super::temporal::literal;
use super::*;
const EW: usize = 16;
fn exp(n: i32) -> Word {
    literal(n as u128, EW)
}
fn add_exp(c: &mut Circuit, a: &[Bit], n: i32) -> Word {
    c.add(a, &exp(n), F).0
}
#[derive(Clone, Copy)]
struct Format {
    f: usize,
    e: usize,
}
impl Format {
    fn width(self) -> usize {
        self.f + self.e + 1
    }
    fn precision(self) -> usize {
        self.f + 1
    }
    fn bias(self) -> i32 {
        (1 << (self.e - 1)) - 1
    }
    fn infinity(self, sign: Bit) -> Word {
        let mut v = vec![F; self.f];
        v.extend(vec![T; self.e]);
        v.push(sign);
        v
    }
    fn invalid(self) -> Word {
        let mut v = self.infinity(F);
        v[self.f - 1] = T;
        v
    }
    fn bound(self) -> usize {
        (1 << self.e) - 3 + self.f
    }
}
struct Parts {
    sign: Bit,
    sig: Word,
    exponent: Word,
    special: Bit,
    nan: Bit,
    signaling: Bit,
    zero: Bit,
}
fn parts(c: &mut Circuit, raw: &[Bit], fmt: Format) -> Parts {
    let stored = &raw[fmt.f..fmt.f + fmt.e];
    let normal = c.nonzero(stored);
    let sig = raw[..fmt.f]
        .iter()
        .copied()
        .chain(std::iter::once(normal))
        .collect();
    let full = c.equal(stored, &vec![T; fmt.e]);
    let fraction = c.nonzero(&raw[..fmt.f]);
    let nan = c.and(full, fraction);
    let quiet = c.not(raw[fmt.f - 1]);
    let signaling = c.and(nan, quiet);
    let any = c.nonzero(&raw[..fmt.width() - 1]);
    let zero = c.not(any);
    let ex = Circuit::extend(stored, EW, false);
    let ex = c.select(normal, &ex, &exp(1));
    let ex = add_exp(c, &ex, -fmt.bias() - fmt.f as i32);
    Parts {
        sign: raw[fmt.width() - 1],
        sig,
        exponent: ex,
        special: full,
        nan,
        signaling,
        zero,
    }
}
// Saturating right shift with a sticky least-significant bit. Every discarded
// one remains observable to round-to-nearest-even, even at huge exponent gaps.
fn shift_jam(c: &mut Circuit, a: &[Bit], count: &[Bit]) -> Word {
    let mut x = a.to_vec();
    for (i, &select) in count.iter().enumerate() {
        let k = 1usize << i;
        let sticky = c.nonzero(&x[..k.min(x.len())]);
        let mut shifted = if k >= x.len() {
            vec![F; x.len()]
        } else {
            x[k..]
                .iter()
                .copied()
                .chain(std::iter::repeat_n(F, k))
                .collect()
        };
        shifted[0] = c.or(shifted[0], sticky);
        x = c.select(select, &shifted, &x);
    }
    x
}
fn normalize(c: &mut Circuit, a: &[Bit]) -> (Word, Word) {
    let mut x = a.to_vec();
    let mut count = exp(0);
    for i in (0..=a.len().ilog2()).rev() {
        let k = 1usize << i;
        let nz = c.nonzero(&x[x.len() - k..]);
        let take = c.not(nz);
        let shifted = std::iter::repeat_n(F, k)
            .chain(x[..x.len() - k].iter().copied())
            .collect::<Word>();
        x = c.select(take, &shifted, &x);
        let n = add_exp(c, &count, k as i32);
        count = c.select(take, &n, &count);
    }
    (x, count)
}
fn pack(c: &mut Circuit, fmt: Format, sign: Bit, mag: &[Bit], exponent: &[Bit]) -> Word {
    let nonzero = c.nonzero(mag);
    let (normalized, leading) = normalize(c, mag);
    let e = add_exp(c, exponent, mag.len() as i32 - 1 + fmt.bias());
    let e = c.sub(&e, &leading).0;
    let retain = fmt.precision() + 3;
    let mut rounded = if normalized.len() > retain {
        let shift = normalized.len() - retain;
        let mut out = normalized[shift..].to_vec();
        let sticky = c.nonzero(&normalized[..shift]);
        out[0] = c.or(out[0], sticky);
        out
    } else {
        std::iter::repeat_n(F, retain - normalized.len())
            .chain(normalized)
            .collect()
    };
    let under = c.lt(&e, &exp(1), true);
    let shift = c.sub(&exp(1), &e).0;
    let shift = c.select(under, &shift, &exp(0));
    rounded = shift_jam(c, &rounded, &shift);
    let tail = c.or(rounded[0], rounded[1]);
    let odd_or_tail = c.or(tail, rounded[3]);
    let up = c.and(rounded[2], odd_or_tail);
    let (sig, carry) = c.add(&rounded[3..], &vec![F; fmt.precision()], up);
    let sig_up = sig[1..]
        .iter()
        .copied()
        .chain(std::iter::once(carry))
        .collect::<Word>();
    let sig = c.select(carry, &sig_up, &sig);
    let e = c.select(under, &exp(1), &e);
    let next = add_exp(c, &e, 1);
    let e = c.select(carry, &next, &e);
    let normal = sig[fmt.f];
    let stored = c.select(normal, &e[..fmt.e], &vec![F; fmt.e]);
    let raw = sig[..fmt.f]
        .iter()
        .copied()
        .chain(stored)
        .chain(std::iter::once(sign))
        .collect::<Word>();
    let overflow = c.lt(&exp((1 << fmt.e) - 2), &e, true);
    let raw = c.select(overflow, &fmt.infinity(sign), &raw);
    let mut zero = vec![F; fmt.width()];
    zero[fmt.width() - 1] = sign;
    c.select(nonzero, &raw, &zero)
}
fn nan_result(c: &mut Circuit, fmt: Format, a: &[Bit], b: &[Bit], ap: &Parts, bp: &Parts) -> Word {
    let quiet = c.select(ap.nan, a, b);
    let right = c.select(bp.signaling, b, &quiet);
    let mut out = c.select(ap.signaling, a, &right);
    out[fmt.f - 1] = T;
    out
}
fn comparison(
    c: &mut Circuit,
    fmt: Format,
    a: &[Bit],
    b: &[Bit],
    ap: &Parts,
    bp: &Parts,
) -> (Bit, Bit) {
    let zeros = c.and(ap.zero, bp.zero);
    let bits_equal = c.equal(a, b);
    let eq = c.or(zeros, bits_equal);
    let mag_lt = c.lt(&a[..fmt.width() - 1], &b[..fmt.width() - 1], false);
    let mag_gt = c.lt(&b[..fmt.width() - 1], &a[..fmt.width() - 1], false);
    let same_sign = c.mux(ap.sign, mag_gt, mag_lt);
    let different = c.xor(ap.sign, bp.sign);
    let lt = c.mux(different, ap.sign, same_sign);
    let nonzero = c.not(zeros);
    let lt = c.and(lt, nonzero);
    let nan = c.or(ap.nan, bp.nan);
    let ordered = c.not(nan);
    (c.and(eq, ordered), c.and(lt, ordered))
}
fn floating_signature(id: &str) -> R<(Format, ClosedOperationSignature)> {
    let (fmt, op, token) = if let Some(op) = id.strip_prefix("floating.single.") {
        (Format { f: 23, e: 8 }, op, "f32")
    } else if let Some(op) = id.strip_prefix("floating.double.") {
        (Format { f: 52, e: 11 }, op, "f64")
    } else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let ty = |t: &str| format!("mpk.csharp.value.{t}.v1");
    let unary = matches!(
        op,
        "plus" | "negate" | "abs" | "is_nan" | "is_infinity" | "is_finite"
    );
    let boolean = matches!(
        op,
        "equal"
            | "not_equal"
            | "less"
            | "less_equal"
            | "greater"
            | "greater_equal"
            | "is_nan"
            | "is_infinity"
            | "is_finite"
    );
    let args = vec![ty(token); if unary { 1 } else { 2 }];
    let result = ty(if boolean { "bool" } else { token });
    let recipe =
        NumericOperation::new(id, &args, &result, None).map_err(|_| OrdinaryCarrierError::Shape)?;
    if !recipe.exception_types().is_empty() {
        return Err(OrdinaryCarrierError::Shape);
    }
    Ok((
        fmt,
        ClosedOperationSignature {
            id: id.into(),
            tag: ClosedOperationTag::Data,
            argument_type_ids: args,
            normal_result_type_id: result,
            ordered_checks: vec![],
        },
    ))
}
fn divide(c: &mut Circuit, n: &[Bit], d: &[Bit]) -> (Word, Word) {
    let width = d.len() + 1;
    let divisor = Circuit::extend(d, width, false);
    let mut r = vec![F; width];
    let mut q = vec![F; n.len()];
    for i in (0..n.len()).rev() {
        r.insert(0, n[i]);
        r.pop();
        let (diff, ge) = c.sub(&r, &divisor);
        r = c.select(ge, &diff, &r);
        q[i] = ge;
    }
    r.pop();
    (q, r)
}
fn floating_circuit(id: &str) -> R<IntegerCircuit> {
    let (fmt, signature) = floating_signature(id)?;
    let op = id.rsplit('.').next().unwrap();
    if op == "remainder" {
        return Err(OrdinaryCarrierError::Shape);
    }
    let mut c = Circuit::new(&vec![fmt.width(); signature.argument_type_ids.len()]);
    let a = c.inputs[0].clone();
    let ap = parts(&mut c, &a, fmt);
    let output = match op {
        "plus" => a,
        "negate" => {
            let mut raw = a;
            raw[fmt.width() - 1] = c.not(ap.sign);
            raw
        }
        "abs" => {
            let mut raw = a;
            raw[fmt.width() - 1] = F;
            raw
        }
        "is_nan" => vec![ap.nan],
        "is_finite" => vec![c.not(ap.special)],
        "is_infinity" => {
            let not_nan = c.not(ap.nan);
            vec![c.and(ap.special, not_nan)]
        }
        _ => {
            let b = c.inputs[1].clone();
            let bp = parts(&mut c, &b, fmt);
            let (eq, lt) = comparison(&mut c, fmt, &a, &b, &ap, &bp);
            let (_, gt) = comparison(&mut c, fmt, &b, &a, &bp, &ap);
            match op {
                "equal" => vec![eq],
                "not_equal" => vec![c.not(eq)],
                "less" => vec![lt],
                "less_equal" => vec![c.or(eq, lt)],
                "greater" => vec![gt],
                "greater_equal" => vec![c.or(eq, gt)],
                "min" | "max" => {
                    let select = if op == "min" { lt } else { c.not(lt) };
                    let v = c.select(select, &a, &b);
                    let zeros = c.and(ap.zero, bp.zero);
                    let mut zero = vec![F; fmt.width()];
                    zero[fmt.width() - 1] = if op == "min" {
                        c.or(ap.sign, bp.sign)
                    } else {
                        c.and(ap.sign, bp.sign)
                    };
                    let v = c.select(zeros, &zero, &v);
                    let v = c.select(bp.nan, &b, &v);
                    c.select(ap.nan, &a, &v)
                }
                "add" | "subtract" | "multiply" | "divide" => {
                    let nan = c.or(ap.nan, bp.nan);
                    let propagated = nan_result(&mut c, fmt, &a, &b, &ap, &bp);
                    let sign = c.xor(ap.sign, bp.sign);
                    let finite = match op {
                        "add" | "subtract" => {
                            let bsign = if op == "subtract" {
                                c.not(bp.sign)
                            } else {
                                bp.sign
                            };
                            let swap = c.lt(&a[..fmt.width() - 1], &b[..fmt.width() - 1], false);
                            let hi = c.select(swap, &bp.sig, &ap.sig);
                            let lo = c.select(swap, &ap.sig, &bp.sig);
                            let he = c.select(swap, &bp.exponent, &ap.exponent);
                            let le = c.select(swap, &ap.exponent, &bp.exponent);
                            let hs = c.mux(swap, bsign, ap.sign);
                            let ls = c.mux(swap, ap.sign, bsign);
                            let different = c.xor(hs, ls);
                            let gap = c.sub(&he, &le).0;
                            let lo = std::iter::repeat_n(F, 3)
                                .chain(lo)
                                .chain(std::iter::once(F))
                                .collect::<Word>();
                            let lo = shift_jam(&mut c, &lo, &gap);
                            let hi = std::iter::repeat_n(F, 3)
                                .chain(hi)
                                .chain(std::iter::once(F))
                                .collect::<Word>();
                            let sum = c.add(&hi, &lo, F).0;
                            let difference = c.sub(&hi, &lo).0;
                            let mag = c.select(different, &difference, &sum);
                            let nonzero = c.nonzero(&mag);
                            let both = c.and(hs, ls);
                            let sign = c.mux(nonzero, hs, both);
                            let exponent = add_exp(&mut c, &he, -3);
                            let raw = pack(&mut c, fmt, sign, &mag, &exponent);
                            let b_inf = fmt.infinity(bsign);
                            let raw = c.select(bp.special, &b_inf, &raw);
                            let raw = c.select(ap.special, &a, &raw);
                            let both = c.and(ap.special, bp.special);
                            let different = c.xor(ap.sign, bsign);
                            let invalid = c.and(both, different);
                            c.select(invalid, &fmt.invalid(), &raw)
                        }
                        "multiply" => {
                            let mag = c.multiply(&ap.sig, &bp.sig, 2 * fmt.precision());
                            let exponent = c.add(&ap.exponent, &bp.exponent, F).0;
                            let raw = pack(&mut c, fmt, sign, &mag, &exponent);
                            let special = c.or(ap.special, bp.special);
                            let raw = c.select(special, &fmt.infinity(sign), &raw);
                            let az = c.and(ap.special, bp.zero);
                            let bz = c.and(bp.special, ap.zero);
                            let invalid = c.or(az, bz);
                            c.select(invalid, &fmt.invalid(), &raw)
                        }
                        _ => {
                            let (a, ashift) = normalize(&mut c, &ap.sig);
                            let (b, bshift) = normalize(&mut c, &bp.sig);
                            let scale = fmt.precision() + 3;
                            let numerator =
                                std::iter::repeat_n(F, scale).chain(a).collect::<Word>();
                            let (q, r) = divide(&mut c, &numerator, &b);
                            let mut mag = q[..fmt.precision() + 4].to_vec();
                            let sticky = c.nonzero(&r);
                            mag[0] = c.or(mag[0], sticky);
                            let e = c.sub(&ap.exponent, &bp.exponent).0;
                            let e = c.sub(&e, &ashift).0;
                            let e = c.add(&e, &bshift, F).0;
                            let e = add_exp(&mut c, &e, -(scale as i32));
                            let raw = pack(&mut c, fmt, sign, &mag, &e);
                            let mut zero = vec![F; fmt.width()];
                            zero[fmt.width() - 1] = sign;
                            let raw = c.select(bp.special, &zero, &raw);
                            let infinite = c.or(ap.special, bp.zero);
                            let raw = c.select(infinite, &fmt.infinity(sign), &raw);
                            let both = c.and(ap.special, bp.special);
                            let zeros = c.and(ap.zero, bp.zero);
                            let invalid = c.or(both, zeros);
                            c.select(invalid, &fmt.invalid(), &raw)
                        }
                    };
                    c.select(nan, &propagated, &finite)
                }
                _ => return Err(OrdinaryCarrierError::Shape),
            }
        }
    };
    Ok(IntegerCircuit {
        signature,
        circuit: c,
        output,
        failures: vec![],
    })
}

fn stage(id: &str, suffix: &str, circuit: Circuit, output: Word) -> IntegerCircuit {
    let cube = |w: usize| format!("{PREFIX}.Cube.D{}", address_bits(w as u32));
    let signature = ClosedOperationSignature {
        id: format!("{id}.{suffix}"),
        tag: ClosedOperationTag::Data,
        argument_type_ids: circuit.inputs.iter().map(|w| cube(w.len())).collect(),
        normal_result_type_id: cube(output.len()),
        ordered_checks: vec![],
    };
    IntegerCircuit {
        signature,
        circuit,
        output,
        failures: vec![],
    }
}
fn remainder_stages(id: &str) -> R<(Format, [IntegerCircuit; 3])> {
    let (fmt, _) = floating_signature(id)?;
    let p = fmt.precision();
    let state_width = 2 * p + 12;
    let mut c = Circuit::new(&[fmt.width(), fmt.width()]);
    let a = c.inputs[0].clone();
    let b = c.inputs[1].clone();
    let ap = parts(&mut c, &a, fmt);
    let bp = parts(&mut c, &b, fmt);
    let (a, al) = normalize(&mut c, &ap.sig);
    let (b, bl) = normalize(&mut c, &bp.sig);
    let ae = c.sub(&ap.exponent, &al).0;
    let be = c.sub(&bp.exponent, &bl).0;
    let less = c.lt(&ae, &be, true);
    let diff = c.sub(&ae, &be).0;
    let diff = c.select(less, &exp(0), &diff);
    let (delta, ge) = c.sub(&a, &b);
    let r = c.select(ge, &delta, &a);
    let initial = r
        .into_iter()
        .chain(b)
        .chain(diff[..12].iter().copied())
        .collect::<Word>();
    let initial = stage(id, "Init", c, initial);
    let mut c = Circuit::new(&[state_width]);
    let input = c.inputs[0].clone();
    let r = &input[..p];
    let d = &input[p..2 * p];
    let remaining = &input[2 * p..];
    let twice = std::iter::once(F)
        .chain(r.iter().copied())
        .collect::<Word>();
    let divisor = Circuit::extend(d, p + 1, false);
    let (delta, ge) = c.sub(&twice, &divisor);
    let next = c.select(ge, &delta, &twice);
    let enabled = c.nonzero(remaining);
    let r = c.select(enabled, &next[..p], r);
    let count = c.sub(remaining, &literal(1, 12)).0;
    let count = c.select(enabled, &count, remaining);
    let output = r
        .into_iter()
        .chain(d.iter().copied())
        .chain(count)
        .collect::<Word>();
    let step = stage(id, "Step", c, output);
    let mut c = Circuit::new(&[fmt.width(), fmt.width(), state_width]);
    let a = c.inputs[0].clone();
    let b = c.inputs[1].clone();
    let state = c.inputs[2].clone();
    let ap = parts(&mut c, &a, fmt);
    let bp = parts(&mut c, &b, fmt);
    let (_, bl) = normalize(&mut c, &bp.sig);
    let exponent = c.sub(&bp.exponent, &bl).0;
    let raw = pack(&mut c, fmt, ap.sign, &state[..p], &exponent);
    let small = c.lt(&a[..fmt.width() - 1], &b[..fmt.width() - 1], false);
    let raw = c.select(small, &a, &raw);
    let raw = c.select(bp.special, &a, &raw);
    let invalid = c.or(ap.special, bp.zero);
    let raw = c.select(invalid, &fmt.invalid(), &raw);
    let nan = c.or(ap.nan, bp.nan);
    let propagated = nan_result(&mut c, fmt, &a, &b, &ap, &bp);
    let raw = c.select(nan, &propagated, &raw);
    Ok((fmt, [initial, step, stage(id, "Finish", c, raw)]))
}
fn emit_floating(b: &mut Builder, id: &str) -> R<OrdinaryScalarDefinition> {
    if !id.ends_with(".remainder") {
        return emit_circuit(b, floating_circuit(id)?, "Floating");
    }
    let (fmt, stages) = remainder_stages(id)?;
    // Each helper is an ordinary definition over its concrete Boolean cubes.
    // Reuse one step term in the finite static composition; this introduces
    // neither a recursive theorem nor a checker-specific remainder rule.
    let definitions = stages
        .into_iter()
        .map(|p| emit_circuit(b, p, "FloatingSteps"))
        .collect::<R<Vec<_>>>()?;
    let depth = address_bits((2 * fmt.precision() + 12) as u32);
    if !b
        .globals
        .contains_key(&format!("{PREFIX}.Cube.D{depth}.Compose"))
    {
        b.helpers(depth)?;
    }
    let step = b.constant(&definitions[1].result_definition)?;
    let composed = b.compose(depth, &vec![step; fmt.bound()])?;
    let args = vec![b.var(1)?, b.var(0)?];
    let init = b.constant(&definitions[0].result_definition)?;
    let init = b.app(init, args.clone())?;
    let state = b.app(composed, vec![init])?;
    let finish = b.constant(&definitions[2].result_definition)?;
    let mut finish_args = args;
    finish_args.push(state);
    let value = b.app(finish, finish_args)?;
    let widths = [fmt.width(); 2];
    let body = bind_inputs(b, &widths, value)?;
    let result_type = b.cube(address_bits(fmt.width() as u32))?;
    let ty = input_type(b, &widths, result_type)?;
    let name = format!(
        "{PREFIX}.Floating.O{}",
        id.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    let result_definition = format!("{name}.Result");
    b.define(&result_definition, ty, body)?;
    let success_definition = format!("{name}.Success");
    let t = b.constant("Std.Bool.true")?;
    let body = bind_inputs(b, &widths, t)?;
    let ty = input_type(b, &widths, b.boolean)?;
    b.define(&success_definition, ty, body)?;
    Ok(OrdinaryScalarDefinition {
        operation: floating_signature(id)?.1,
        result_definition,
        success_definition,
        ordered_failure_definitions: vec![],
        boolean_gates: definitions.iter().map(|d| d.boolean_gates).sum(),
        state_depth: definitions.iter().map(|d| d.state_depth).max().unwrap(),
        static_transformers: definitions
            .iter()
            .map(|d| d.static_transformers)
            .sum::<usize>()
            + fmt.bound(),
    })
}
/// W09 floating operations over binary32/binary64 carriers. Numeric conversions,
/// whole-foundation expansion and application VC proofs remain separate work.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryFloatingProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryScalarDefinition>,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryFloatingProgram {
    pub fn definitions(&self) -> &[OrdinaryScalarDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed floating program")
    }
}
pub fn generate_csharp_practical_ordinary_floating(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryFloatingProgram> {
    let mut b = Builder::new()?;
    let mut definitions = vec![];
    let signatures = vir
        .operation_signatures()
        .iter()
        .filter(|s| s.id.starts_with("floating.single.") || s.id.starts_with("floating.double."))
        .map(|s| (s.id.clone(), s))
        .collect::<BTreeMap<_, _>>();
    for (id, signature) in signatures {
        if &floating_signature(&id)?.1 != signature {
            return Err(OrdinaryCarrierError::Linkage);
        }
        definitions.push(emit_floating(&mut b, &id)?);
    }
    let certificate = b.finish()?;
    let p = OrdinaryFloatingProgram {
        schema: "mpk.csharp.ordinary_floating.v1".into(),
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
pub fn import_csharp_practical_ordinary_floating(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryFloatingProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_floating(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;
    const OPS: &[&str] = &[
        "plus",
        "negate",
        "abs",
        "is_nan",
        "is_infinity",
        "is_finite",
        "equal",
        "not_equal",
        "less",
        "less_equal",
        "greater",
        "greater_equal",
        "min",
        "max",
        "add",
        "subtract",
        "multiply",
        "divide",
        "remainder",
    ];
    const FIXTURES: &[&str] = &[
        "floating.single.add",
        "floating.double.add",
        "floating.double.multiply",
        "floating.double.divide",
        "floating.single.remainder",
        "floating.double.remainder",
        "floating.double.min",
        "floating.single.is_nan",
    ];
    fn prune(mut p: IntegerCircuit) -> IntegerCircuit {
        p.circuit.prune(&mut p.output);
        p
    }
    fn observed(p: &IntegerCircuit, args: &[u128]) -> u128 {
        let values = p.circuit.evaluate(args);
        p.output
            .iter()
            .enumerate()
            .fold(0, |v, (i, &b)| v | ((values[b] as u128) << i))
    }
    fn cases(fmt: Format) -> Vec<u128> {
        let one = (fmt.bias() as u128) << fmt.f;
        let infinity = ((1u128 << fmt.e) - 1) << fmt.f;
        let sign = 1u128 << (fmt.width() - 1);
        let quiet = 1u128 << (fmt.f - 1);
        let mut values = vec![
            0,
            sign,
            1,
            sign | 1,
            (1 << fmt.f) - 1,
            1 << fmt.f,
            (1 << fmt.f) + 1,
            one - 1,
            one,
            one + 1,
            one | sign,
            one + (1 << (fmt.f - 1)),
            one - (1 << fmt.f),
            one + (1 << fmt.f),
            infinity - 1,
            infinity,
            infinity | sign,
            infinity | 1,
            infinity | quiet | 1,
            infinity | sign | 1,
            infinity | sign | quiet | 2,
            ((fmt.bias() - fmt.f as i32 - 1) as u128) << fmt.f,
        ];
        let mut random = 0x64b2f5d7d003e691u64;
        for _ in 0..8 {
            random ^= random << 13;
            random ^= random >> 7;
            random ^= random << 17;
            values.push((random as u128) & ((1u128 << fmt.width()) - 1));
        }
        values
    }
    #[test]
    fn floating_circuits_match_t03_numeric_oracle() {
        let bundle = validate_registered_foundation_bundle(
            registered_foundation_descriptor_transport(),
            registered_foundation_definitions_transport(),
        )
        .unwrap();
        let roots=serde_json::json!(["f32","f64","bool"].map(|id|serde_json::json!({"origin":"semantic_binding","provenance_id":format!("float.{id}"),"type":{"kind":"primitive","id":id}})));
        let bytes =
            canonical_closed_root_set_transport(&bundle, &roots, &serde_json::json!({})).unwrap();
        let roots = validate_closed_root_set(&bundle, &bytes).unwrap();
        let closed = derive_closed_instances(&bundle, &roots).unwrap();
        for kind in ["single", "double"] {
            for &op in OPS {
                let id = format!("floating.{kind}.{op}");
                let (fmt, sig) = floating_signature(&id).unwrap();
                let recipe = NumericOperation::new(
                    &id,
                    &sig.argument_type_ids,
                    &sig.normal_result_type_id,
                    None,
                )
                .unwrap();
                let raw = cases(fmt);
                let circuit = (op != "remainder").then(|| prune(floating_circuit(&id).unwrap()));
                let stages =
                    (op == "remainder").then(|| remainder_stages(&id).unwrap().1.map(prune));
                let mut pairs = raw
                    .iter()
                    .flat_map(|&a| raw.iter().map(move |&b| (a, b)))
                    .collect::<Vec<_>>();
                if matches!(op, "add" | "subtract" | "multiply" | "divide" | "remainder") {
                    let mut seed = 0x891aed14c7025be3u64;
                    let mask = (1u128 << fmt.width()) - 1;
                    let mantissa = (1u128 << fmt.f) - 1;
                    for i in 0..1024 {
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        let a = seed as u128 & mask;
                        seed ^= seed << 13;
                        seed ^= seed >> 7;
                        seed ^= seed << 17;
                        let b = match i % 3 {
                            0 => ((a.wrapping_add(1)) ^ (1u128 << (fmt.width() - 1))) & mask,
                            1 => (a & !mantissa) | (seed as u128 & mantissa),
                            _ => seed as u128 & mask,
                        };
                        pairs.push((a, b));
                    }
                }
                for (a, b) in pairs {
                    let args = [a, b];
                    let actual = if let Some(p) = &circuit {
                        observed(p, &args)
                    } else {
                        let [initial, step, finish] = stages.as_ref().unwrap();
                        let mut state = observed(initial, &args);
                        for _ in 0..fmt.bound() {
                            if state >> (2 * fmt.precision()) == 0 {
                                break;
                            }
                            state = observed(step, &[state]);
                        }
                        // Once the counter is zero, each remaining statically
                        // composed step must be an identity on the whole state.
                        if state >> (2 * fmt.precision()) == 0 {
                            assert_eq!(observed(step, &[state]), state);
                        }
                        observed(finish, &[a, b, state])
                    };
                    let values = args[..sig.argument_type_ids.len()]
                        .iter()
                        .map(|n| {
                            if kind == "single" {
                                MonomorphicValue::F32Bits {
                                    type_id: "mpk.csharp.value.f32.v1".into(),
                                    bits: format!("{n:08x}"),
                                }
                            } else {
                                MonomorphicValue::F64Bits {
                                    type_id: "mpk.csharp.value.f64.v1".into(),
                                    bits: format!("{n:016x}"),
                                }
                            }
                        })
                        .collect::<Vec<_>>();
                    let expected = match recipe.evaluate(&bundle, &roots, &closed, &values).unwrap()
                    {
                        MonomorphicValue::F32Bits { bits, .. }
                        | MonomorphicValue::F64Bits { bits, .. } => {
                            u128::from_str_radix(&bits, 16).unwrap()
                        }
                        MonomorphicValue::Bool { value, .. } => u128::from(value),
                        _ => panic!("unexpected float oracle result"),
                    };
                    assert_eq!(
                        actual, expected,
                        "{id}({a:x},{b:x}): actual {actual:x}, expected {expected:x}"
                    );
                }
            }
        }
    }
    #[test]
    fn floating_circuits_emit_all_signatures() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../develop/migrations/csharp-03/ordinary-foundation/floating-circuits");
        let out = std::env::var_os("MPK_W09_FLOAT_OUT").map(std::path::PathBuf::from);
        if let Some(dir) = &out {
            std::fs::create_dir_all(dir).unwrap();
        }
        let mut metrics = vec![];
        for kind in ["single", "double"] {
            for &op in OPS {
                let id = format!("floating.{kind}.{op}");
                let mut b = Builder::new().unwrap();
                let definition =
                    emit_floating(&mut b, &id).unwrap_or_else(|e| panic!("{id}: {e:?}"));
                let bytes = b.finish().unwrap_or_else(|e| panic!("{id}: {e:?}"));
                let cert = decode_canonical_certificate(&bytes).unwrap();
                metrics.push(serde_json::json!({"id":id,"definition":definition,"terms":cert.term_table.len(),"declarations":cert.declarations.len(),"hash":mpk_cert::hash_hex(&mpk_cert::certificate_hash(&bytes))}));
                if FIXTURES.contains(&id.as_str()) {
                    let hex = bytes.iter().map(|b| format!("{b:02x}")).collect::<String>() + "\n";
                    if let Some(dir) = &out {
                        std::fs::write(dir.join(format!("{id}.hex")), hex).unwrap();
                    } else {
                        assert_eq!(
                            std::fs::read_to_string(root.join(format!("{id}.hex"))).unwrap(),
                            hex,
                            "{id}"
                        );
                    }
                }
            }
        }
        if let Some(dir) = &out {
            std::fs::write(
                dir.join("metrics.json"),
                serde_json::to_vec_pretty(&metrics).unwrap(),
            )
            .unwrap();
        } else {
            let expected: Value =
                serde_json::from_slice(&std::fs::read(root.join("metrics.json")).unwrap()).unwrap();
            assert_eq!(serde_json::json!(metrics), expected);
        }
    }
    #[test]
    fn floating_circuits_core_evaluation_and_static_remainder() {
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                use super::super::super::tests::{apply, bit, run, V};
                for (id, cases) in [
                    (
                        "floating.single.add",
                        vec![
                            (vec![0x3f800000u128, 0x33800000], 0x3f800000u128),
                            (vec![0x3f800001, 0x33800000], 0x3f800002),
                            (vec![0x80000000, 0x80000000], 0x80000000),
                        ],
                    ),
                    (
                        "floating.single.remainder",
                        vec![
                            (vec![0x40b00000, 0x40000000], 0x3fc00000),
                            (vec![0xc0b00000, 0x40000000], 0xbfc00000),
                        ],
                    ),
                    (
                        "floating.double.min",
                        vec![
                            (
                                vec![0x7ff8000000000001, 0x7ff0000000000002],
                                0x7ff8000000000001,
                            ),
                            (vec![0, 0x8000000000000000], 0x8000000000000000),
                        ],
                    ),
                    (
                        "floating.single.is_nan",
                        vec![(vec![0x7f800001], 1), (vec![0x7f800000], 0)],
                    ),
                ] {
                    let (fmt, signature) = floating_signature(id).unwrap();
                    let mut builder = Builder::new().unwrap();
                    let definition = emit_floating(&mut builder, id).unwrap();
                    let cert = decode_canonical_certificate(&builder.finish().unwrap()).unwrap();
                    for (raw, expected) in cases {
                        let args = raw
                            .into_iter()
                            .map(|raw| {
                                V::Cube((0..fmt.width()).map(|i| raw & (1 << i) != 0).collect())
                            })
                            .collect::<Vec<_>>();
                        assert!(bit(run(
                            &cert,
                            &definition.success_definition,
                            args.clone()
                        )));
                        let value = run(&cert, &definition.result_definition, args);
                        let width = if signature.normal_result_type_id == "mpk.csharp.value.bool.v1"
                        {
                            1
                        } else {
                            fmt.width()
                        };
                        for i in 0..width {
                            let mut v = value.clone();
                            for j in 0..address_bits(width as u32) {
                                v = apply(&cert, v, V::Bit(i & (1 << j) != 0));
                            }
                            assert_eq!(bit(v), expected & (1 << i) != 0, "{id} bit {i}");
                        }
                    }
                }
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
