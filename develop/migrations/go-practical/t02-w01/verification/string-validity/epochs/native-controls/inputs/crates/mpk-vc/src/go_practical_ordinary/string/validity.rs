//! Full carrier validity from actual high bits and the boundary of its prefix.
//! No source evaluation, nominal equality, callback or new core primitive.
use super::*;
use crate::go_practical_ordinary::GoOrdinaryError;

const LEFT: [&str; 8] = ["l0", "l1", "l2", "l3", "l4", "l5", "l6", "l7"];
const RIGHT: [&str; 8] = ["r0", "r1", "r2", "r3", "r4", "r5", "r6", "r7"];
fn vc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.String.Validity.{name}"), args)
}
fn fv(d: usize, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.FiniteVector.D{d}.{name}"), args)
}
fn idx(d: usize, address: Vec<Expr>) -> Expr {
    fv(d, "Index", address)
}
fn cap(d: usize) -> Expr {
    fv(d, "Capacity", vec![])
}
fn sh(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Word.Shifts.{name}"), args)
}
fn wi(name: &str, args: Vec<Expr>) -> Expr {
    ix(name, args)
}
fn coefficient(n: usize) -> Expr {
    numeric("PowerTwo", vec![(0..n).fold(zero(), |n, _| succ(n))])
}
fn function(a: Expr, b: Expr) -> Expr {
    pi("_arg", a, b)
}
fn less_left(e: Equality, b: Expr) -> Equality {
    e.clone().mapped(
        lam("_n", nat(), less(var("_n"), b.clone())),
        less(e.left, b.clone()),
        less(e.right, b),
        boolean(),
    )
}
fn less_right(a: Expr, e: Equality) -> Equality {
    e.clone().mapped(
        lam("_n", nat(), less(a.clone(), var("_n"))),
        less(a.clone(), e.left),
        less(a, e.right),
        boolean(),
    )
}
fn law(name: &str, args: Vec<Expr>, left: Expr, right: Expr) -> Equality {
    Equality::natural(left, right, nc(name, args))
}
fn high(word: Expr) -> Expr {
    sh("CountHigh8", vec![word])
}
fn high_value(word: Expr) -> Expr {
    sh("CountHighValue8", vec![word])
}
fn digits(word: Expr) -> Expr {
    numeric("W64.Digits8", vec![word])
}
fn low_bits(word: Expr) -> Result<Vec<Expr>> {
    (0..8)
        .rev()
        .map(|i| word::at(word.clone(), 64, i))
        .collect()
}
fn high_tree(word: Expr, a: usize, b: usize, weighted: bool) -> Result<Expr> {
    if a + 1 == b {
        let flag = word::at(word, 64, a)?;
        return Ok(if weighted {
            mul(coefficient(a - 8), numeric("Bit", vec![flag]))
        } else {
            flag
        });
    }
    let m = a + (b - a) / 2;
    let l = high_tree(word.clone(), a, m, weighted)?;
    let r = high_tree(word, m, b, weighted)?;
    Ok(if weighted {
        add(l, r)
    } else {
        bc("Or", vec![l, r])
    })
}
struct Validity<'a>(&'a mut Builder);
impl Validity<'_> {
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.String.Validity.{name}"),
            args,
            eq(e.ty, e.left, e.right),
            e.proof,
            true,
        )
    }
    fn proof(&mut self, name: &str, args: &[Binder], ty: Expr, proof: Expr) -> Result<()> {
        emit(
            self.0,
            &format!("Mpk.Go.Practical.String.Validity.{name}"),
            args,
            ty,
            proof,
            true,
        )
    }
    fn indices(&mut self) -> Result<()> {
        self.theorem(
            "IndexZero0",
            &[],
            Equality::natural(idx(0, vec![]), zero(), refl(nat(), zero())),
        )?;
        for d in 1..=8 {
            let tails = address_bits(&LEFT[..d - 1]);
            let args = LEFT[..d - 1]
                .iter()
                .map(|n| (*n, boolean()))
                .collect::<Vec<_>>();
            for high in [false, true] {
                let mut all = vec![bit(high)];
                all.extend(tails.clone());
                let target = if high {
                    add(cap(d - 1), idx(d - 1, tails.clone()))
                } else {
                    idx(d - 1, tails.clone())
                };
                self.theorem(
                    &format!("Index{}{d}", if high { "High" } else { "Low" }),
                    &args,
                    Equality::natural(idx(d, all), target.clone(), refl(nat(), target)),
                )?;
            }
            self.theorem(
                &format!("IndexZero{d}"),
                &[],
                Equality::natural(
                    idx(d, vec![bit(false); d]),
                    idx(d - 1, vec![bit(false); d - 1]),
                    vc(&format!("IndexLow{d}"), vec![bit(false); d - 1]),
                )
                .then(Equality::natural(
                    idx(d - 1, vec![bit(false); d - 1]),
                    zero(),
                    vc(&format!("IndexZero{}", d - 1), vec![]),
                ))?,
            )?;
            let (l, r) = (address_bits(&LEFT[..d - 1]), address_bits(&RIGHT[..d - 1]));
            let mut args = LEFT[..d - 1]
                .iter()
                .map(|n| (*n, boolean()))
                .collect::<Vec<_>>();
            args.extend(RIGHT[..d - 1].iter().map(|n| (*n, boolean())));
            for high in [false, true] {
                let (mut il, mut ir) = (vec![bit(high)], vec![bit(high)]);
                il.extend(l.clone());
                ir.extend(r.clone());
                let lp = if high {
                    add(cap(d - 1), idx(d - 1, l.clone()))
                } else {
                    idx(d - 1, l.clone())
                };
                let rp = if high {
                    add(cap(d - 1), idx(d - 1, r.clone()))
                } else {
                    idx(d - 1, r.clone())
                };
                let mut e = less_left(
                    Equality::natural(
                        idx(d, il),
                        lp.clone(),
                        vc(
                            &format!("Index{}{d}", if high { "High" } else { "Low" }),
                            l.clone(),
                        ),
                    ),
                    idx(d, ir.clone()),
                )
                .then(less_right(
                    lp.clone(),
                    Equality::natural(
                        idx(d, ir),
                        rp.clone(),
                        vc(
                            &format!("Index{}{d}", if high { "High" } else { "Low" }),
                            r.clone(),
                        ),
                    ),
                ))?;
                if high {
                    let h = cap(d - 1);
                    let (li, ri) = (idx(d - 1, l.clone()), idx(d - 1, r.clone()));
                    e = e
                        .then(less_left(
                            law(
                                "AddComm",
                                vec![h.clone(), li.clone()],
                                lp,
                                add(li.clone(), h.clone()),
                            ),
                            rp.clone(),
                        ))?
                        .then(less_right(
                            add(li.clone(), h.clone()),
                            law(
                                "AddComm",
                                vec![h.clone(), ri.clone()],
                                rp,
                                add(ri.clone(), h.clone()),
                            ),
                        ))?
                        .then(Equality::boolean(
                            less(add(li.clone(), h.clone()), add(ri.clone(), h.clone())),
                            less(li.clone(), ri.clone()),
                            nc("LessTranslate", vec![li, ri, h]),
                        ))?;
                }
                let lower = eq(
                    boolean(),
                    less(idx(d - 1, l.clone()), idx(d - 1, r.clone())),
                    bit(true),
                );
                let mut guarded = args.clone();
                guarded.push(("inside", lower));
                e = e.then(Equality::boolean(
                    less(idx(d - 1, l.clone()), idx(d - 1, r.clone())),
                    bit(true),
                    var("inside"),
                ))?;
                self.theorem(
                    &format!("Less{}{d}", if high { "High" } else { "Low" }),
                    &guarded,
                    e,
                )?;
            }
            let (mut il, mut ir) = (vec![bit(false)], vec![bit(true)]);
            il.extend(l.clone());
            ir.extend(r.clone());
            let mut bound = l.clone();
            bound.push(idx(d - 1, r.clone()));
            let e = less_left(
                Equality::natural(
                    idx(d, il),
                    idx(d - 1, l.clone()),
                    vc(&format!("IndexLow{d}"), l.clone()),
                ),
                idx(d, ir.clone()),
            )
            .then(less_right(
                idx(d - 1, l.clone()),
                Equality::natural(
                    idx(d, ir),
                    add(cap(d - 1), idx(d - 1, r.clone())),
                    vc(&format!("IndexHigh{d}"), r.clone()),
                ),
            ))?
            .then(Equality::boolean(
                less(idx(d - 1, l), add(cap(d - 1), idx(d - 1, r))),
                bit(true),
                fv(d - 1, "IndexPaddedBound", bound),
            ))?;
            self.theorem(&format!("LessCross{d}"), &args, e)?;
        }
        Ok(())
    }
    fn high_definitions(&mut self) -> Result<()> {
        for weighted in [false, true] {
            let body = lam("word", word64(), high_tree(var("word"), 8, 64, weighted)?);
            self.theorem(
                if weighted {
                    "HighValueDefinition"
                } else {
                    "HighDefinition"
                },
                &[],
                Equality {
                    ty: function(word64(), if weighted { nat() } else { boolean() }),
                    left: constant(if weighted {
                        "Mpk.Go.Practical.Word.Shifts.CountHighValue8"
                    } else {
                        "Mpk.Go.Practical.Word.Shifts.CountHigh8"
                    }),
                    right: body.clone(),
                    proof: refl(
                        function(word64(), if weighted { nat() } else { boolean() }),
                        body,
                    ),
                },
            )?;
        }
        Ok(())
    }
    fn bounds(&mut self) -> Result<()> {
        let s = var("s");
        let word = length(s.clone());
        let d = digits(word.clone());
        let clear = eq(boolean(), high(word.clone()), bit(false));
        let args = [("s", string_type()), ("clear", clear)];
        let small = Equality::natural(
            size(s.clone()),
            d.clone(),
            wi("SmallValue", vec![word.clone(), var("clear")]),
        );
        self.theorem(
            "SizeIndex",
            &args,
            small.clone().then(Equality::natural(
                d.clone(),
                idx(8, low_bits(word.clone())?),
                wi("DigitsIndex", vec![word.clone()]),
            ))?,
        )?;
        let complement = sh("CountLowComplement8", vec![word.clone()]);
        let balance = add(add(d.clone(), complement.clone()), one());
        let c = Equality::natural(
            cap(8),
            coefficient(8),
            call(
                "Std.Eq.symm",
                vec![nat(), coefficient(8), cap(8), wi("CapacityPower", vec![])],
            ),
        )
        .then(
            Equality::natural(
                balance.clone(),
                coefficient(8),
                sh("CountLowBalance8", vec![word]),
            )
            .reverse(),
        )?
        .then(law(
            "AddAssoc",
            vec![d.clone(), complement.clone(), one()],
            balance,
            add(d.clone(), add(complement.clone(), one())),
        ))?;
        let e = less_left(c, size(s.clone()))
            .then(less_right(
                add(d.clone(), add(complement.clone(), one())),
                small,
            ))?
            .then(Equality::boolean(
                less(add(d.clone(), add(complement.clone(), one())), d.clone()),
                bit(false),
                sh("CountNoLessExtension", vec![d, add(complement, one())]),
            ))?;
        self.theorem("ClearUpper", &args, e)?;
        let args = [
            ("s", string_type()),
            ("full", eq(nat(), size(s.clone()), cap(8))),
        ];
        self.theorem(
            "FullUpper",
            &args,
            less_right(
                cap(8),
                Equality::natural(size(s.clone()), cap(8), var("full")),
            )
            .then(Equality::boolean(
                less(cap(8), cap(8)),
                bit(false),
                nc("LessIrreflexive", vec![cap(8)]),
            ))?,
        )?;
        let addr = address_bits(&SLOTS);
        let i = idx(8, addr.clone());
        let inside = less_right(
            i.clone(),
            Equality::natural(size(s.clone()), cap(8), var("full")),
        )
        .then(Equality::boolean(
            less(i.clone(), cap(8)),
            bit(true),
            fv(8, "IndexBound", addr.clone()),
        ))?;
        let contradiction =
            Equality::boolean(less(i, size(s.clone())), bit(false), var("_outside"))
                .reverse()
                .then(inside)?;
        let goal = eq(byte(), app(data(s.clone()), addr), zero_word(8));
        let proof = cube(
            &SLOTS,
            lam(
                "_outside",
                eq(
                    boolean(),
                    less(idx(8, address_bits(&SLOTS)), size(s.clone())),
                    bit(false),
                ),
                bc("FalseElim", vec![goal, contradiction.proof]),
            ),
            true,
        );
        self.proof("FullPadding", &args, padding(s.clone()), proof)?;
        self.proof(
            "FullValid",
            &args,
            valid(s.clone()),
            pc(
                "Make",
                vec![
                    upper(s.clone()),
                    padding(s.clone()),
                    vc("FullUpper", vec![s.clone(), var("full")]),
                    vc("FullPadding", vec![s, var("full")]),
                ],
            ),
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut v = Validity(builder);
    v.indices()?;
    v.high_definitions()?;
    v.bounds()
}

struct Literal<'a> {
    builder: &'a mut Builder,
    serial: usize,
}
impl Literal<'_> {
    fn save(&mut self, e: Equality) -> Result<Equality> {
        let name = format!(
            "Mpk.Go.Practical.String.Value.Validity.Step{:03}",
            self.serial
        );
        self.serial += 1;
        emit(
            self.builder,
            &name,
            &[],
            eq(e.ty.clone(), e.left.clone(), e.right.clone()),
            e.proof,
            true,
        )?;
        Ok(Equality {
            proof: constant(&name),
            ..e
        })
    }
    fn bit(&self, at: usize, yes: bool) -> Result<Equality> {
        Ok(Equality::boolean(
            word::at(length(constant("Mpk.Go.Practical.String.Value")), 64, at)?,
            bit(yes),
            constant(&format!("Mpk.Go.Practical.String.Value.LengthBit{at}")),
        ))
    }
    fn high(&mut self, a: usize, b: usize, weighted: bool) -> Result<Equality> {
        if a + 1 == b {
            let e = self.bit(a, weighted && a == 8)?;
            if !weighted {
                return Ok(e);
            }
            let flag = e.left.clone();
            let literal = e.right.clone();
            let bit_e = e
                .mapped(
                    lam("_flag", boolean(), numeric("Bit", vec![var("_flag")])),
                    numeric("Bit", vec![flag]),
                    numeric("Bit", vec![literal]),
                    nat(),
                )
                .then(Equality::natural(
                    numeric("Bit", vec![bit(a == 8)]),
                    if a == 8 { one() } else { zero() },
                    refl(nat(), if a == 8 { one() } else { zero() }),
                ))?;
            let e = mul_right(coefficient(a - 8), bit_e);
            let e = if a == 8 {
                e.then(law(
                    "MulOne",
                    vec![coefficient(0)],
                    mul(coefficient(0), one()),
                    coefficient(0),
                ))?
                .then(Equality::natural(
                    coefficient(0),
                    one(),
                    numeric("PowerTwoZero", vec![]),
                ))?
            } else {
                e.then(law(
                    "MulZero",
                    vec![coefficient(a - 8)],
                    mul(coefficient(a - 8), zero()),
                    zero(),
                ))?
            };
            return self.save(e);
        }
        let m = a + (b - a) / 2;
        let l = self.high(a, m, weighted)?;
        let r = self.high(m, b, weighted)?;
        let e = if weighted {
            let left = add(l.left.clone(), r.left.clone());
            let right = add(l.right.clone(), r.right.clone());
            let both = l
                .clone()
                .on_left(r.left.clone())
                .then(r.clone().on_right(l.right.clone()))?;
            let target = if a == 8 { one() } else { zero() };
            let simplify = if a == 8 {
                law("AddZero", vec![one()], right.clone(), target.clone())
            } else {
                law("ZeroAdd", vec![zero()], right.clone(), target.clone())
            };
            let e = both.then(simplify)?;
            debug_assert_eq!(e.left, left);
            e
        } else {
            let from = bc("Or", vec![l.left.clone(), r.left.clone()]);
            let mid = bc("Or", vec![l.right.clone(), r.left.clone()]);
            let end = bc("Or", vec![l.right.clone(), r.right.clone()]);
            l.clone()
                .mapped(
                    lam(
                        "_flag",
                        boolean(),
                        bc("Or", vec![var("_flag"), r.left.clone()]),
                    ),
                    from,
                    mid.clone(),
                    boolean(),
                )
                .then(r.clone().mapped(
                    lam(
                        "_flag",
                        boolean(),
                        bc("Or", vec![l.right.clone(), var("_flag")]),
                    ),
                    mid,
                    end.clone(),
                    boolean(),
                ))?
                .then(Equality::boolean(
                    end,
                    bit(false),
                    refl(boolean(), bit(false)),
                ))?
        };
        self.save(e)
    }
    fn high_result(&mut self, weighted: bool) -> Result<Equality> {
        let word = length(constant("Mpk.Go.Practical.String.Value"));
        let ty = if weighted { nat() } else { boolean() };
        let fty = function(word64(), ty.clone());
        let fname = constant(if weighted {
            "Mpk.Go.Practical.Word.Shifts.CountHighValue8"
        } else {
            "Mpk.Go.Practical.Word.Shifts.CountHigh8"
        });
        let explicit = lam("word", word64(), high_tree(var("word"), 8, 64, weighted)?);
        let unroll = Equality {
            ty: fty.clone(),
            left: fname,
            right: explicit,
            proof: vc(
                if weighted {
                    "HighValueDefinition"
                } else {
                    "HighDefinition"
                },
                vec![],
            ),
        }
        .mapped(
            lam("_function", fty, app(var("_function"), vec![word.clone()])),
            if weighted {
                high_value(word.clone())
            } else {
                high(word.clone())
            },
            high_tree(word, 8, 64, weighted)?,
            ty,
        );
        let observation = self.high(8, 64, weighted)?;
        self.save(unroll.then(observation)?)
    }
    fn digits_index(&self, n: usize) -> Result<Equality> {
        let w = length(constant("Mpk.Go.Practical.String.Value"));
        let mut current = low_bits(w.clone())?;
        let mut e = Equality::natural(
            digits(w.clone()),
            idx(8, current.clone()),
            wi("DigitsIndex", vec![w]),
        );
        for position in 0..8 {
            let field = 7 - position;
            let observed = self.bit(field, (n >> field) & 1 != 0)?;
            let before = idx(8, current.clone());
            let mut body = current.clone();
            body[position] = var("_bit");
            current[position] = observed.right.clone();
            e = e.then(observed.mapped(
                lam("_bit", boolean(), idx(8, body)),
                before,
                idx(8, current.clone()),
                nat(),
            ))?;
        }
        Ok(e)
    }
    fn size_index(&mut self, n: usize) -> Result<(Equality, Expr)> {
        let s = constant("Mpk.Go.Practical.String.Value");
        let clear = self.high_result(false)?;
        let word = length(s.clone());
        let e = Equality::natural(
            size(s),
            digits(word.clone()),
            wi("SmallValue", vec![word, clear.proof.clone()]),
        )
        .then(self.digits_index(n)?)?;
        Ok((self.save(e)?, clear.proof))
    }
    fn size_full(&mut self) -> Result<Equality> {
        let s = constant("Mpk.Go.Practical.String.Value");
        let w = length(s.clone());
        let high = self.high_result(true)?;
        let low = self.digits_index(256)?.then(Equality::natural(
            idx(8, vec![bit(false); 8]),
            zero(),
            vc("IndexZero8", vec![]),
        ))?;
        let e = Equality::natural(
            size(s),
            add(
                digits(w.clone()),
                mul(coefficient(8), high_value(w.clone())),
            ),
            sh("CountBalance8", vec![w]),
        )
        .then(low.on_left(mul(
            coefficient(8),
            high_value(length(constant("Mpk.Go.Practical.String.Value"))),
        )))?
        .then(mul_right(coefficient(8), high).on_right(zero()))?
        .then(
            law(
                "MulOne",
                vec![coefficient(8)],
                mul(coefficient(8), one()),
                coefficient(8),
            )
            .on_right(zero()),
        )?
        .then(law(
            "ZeroAdd",
            vec![coefficient(8)],
            add(zero(), coefficient(8)),
            coefficient(8),
        ))?
        .then(Equality::natural(
            coefficient(8),
            cap(8),
            wi("CapacityPower", vec![]),
        ))?;
        self.save(e)
    }
}
fn less_indices(d: usize, left: &[Expr], right: &[Expr]) -> Result<Expr> {
    if d == 0 {
        return Err(GoOrdinaryError::Linkage);
    }
    let mut args = left[1..].to_vec();
    args.extend_from_slice(&right[1..]);
    if left[0] == bit(false) && right[0] == bit(true) {
        return Ok(vc(&format!("LessCross{d}"), args));
    }
    if left[0] != right[0] || (left[0] != bit(false) && left[0] != bit(true)) {
        return Err(GoOrdinaryError::Linkage);
    }
    args.push(less_indices(d - 1, &left[1..], &right[1..])?);
    Ok(vc(
        &format!(
            "Less{}{d}",
            if left[0] == bit(true) { "High" } else { "Low" }
        ),
        args,
    ))
}
fn padded(n: usize, prefix: &[bool], size_index: &Equality) -> Result<Expr> {
    let s = constant("Mpk.Go.Practical.String.Value");
    let mut address = prefix.iter().copied().map(bit).collect::<Vec<_>>();
    address.extend(SLOTS[prefix.len()..].iter().map(|n| var(n)));
    let start = prefix.iter().fold(0usize, |a, b| 2 * a + usize::from(*b)) << (8 - prefix.len());
    let end = start + (1 << (8 - prefix.len()));
    let outside = eq(
        boolean(),
        less(idx(8, address.clone()), size(s.clone())),
        bit(false),
    );
    let goal = eq(byte(), app(data(s.clone()), address.clone()), zero_word(8));
    let proof = if start >= n {
        lam("_outside", outside, refl(byte(), zero_word(8)))
    } else if end <= n {
        let count = (0..8)
            .rev()
            .map(|i| bit(n & (1 << i) != 0))
            .collect::<Vec<_>>();
        let inside =
            less_right(idx(8, address.clone()), size_index.clone()).then(Equality::boolean(
                less(idx(8, address), idx(8, count.clone())),
                bit(true),
                less_indices(
                    8,
                    &{
                        let mut a = prefix.iter().copied().map(bit).collect::<Vec<_>>();
                        a.extend(SLOTS[prefix.len()..].iter().map(|n| var(n)));
                        a
                    },
                    &count,
                )?,
            ))?;
        let contradiction = Equality::boolean(inside.left.clone(), bit(false), var("_outside"))
            .reverse()
            .then(inside)?;
        lam(
            "_outside",
            outside,
            bc("FalseElim", vec![goal, contradiction.proof]),
        )
    } else {
        let k = prefix.len();
        let mut no = prefix.to_vec();
        no.push(false);
        let mut yes = prefix.to_vec();
        yes.push(true);
        let mut dynamic = prefix.iter().copied().map(bit).collect::<Vec<_>>();
        dynamic.extend(SLOTS[k..].iter().map(|n| var(n)));
        let property = cube(
            &SLOTS[k + 1..],
            pi(
                "_outside",
                eq(
                    boolean(),
                    less(idx(8, dynamic.clone()), size(s.clone())),
                    bit(false),
                ),
                eq(byte(), app(data(s), dynamic), zero_word(8)),
            ),
            false,
        );
        return Ok(lam(
            SLOTS[k],
            boolean(),
            call(
                "Std.Bool.cases",
                vec![
                    lam(SLOTS[k], boolean(), property),
                    padded(n, &no, size_index)?,
                    padded(n, &yes, size_index)?,
                    var(SLOTS[k]),
                ],
            ),
        ));
    };
    Ok(cube(&SLOTS[prefix.len()..], proof, true))
}
pub(super) fn append_value(builder: &mut Builder, n: usize) -> Result<()> {
    let s = constant("Mpk.Go.Practical.String.Value");
    let mut literal = Literal { builder, serial: 0 };
    let (upper_proof, padding_proof) = if n == 256 {
        let full = literal.size_full()?;
        (
            vc("FullUpper", vec![s.clone(), full.proof.clone()]),
            vc("FullPadding", vec![s.clone(), full.proof]),
        )
    } else {
        let (size_index, clear) = literal.size_index(n)?;
        (
            vc("ClearUpper", vec![s.clone(), clear]),
            padded(n, &[], &size_index)?,
        )
    };
    for (name, ty, proof) in [
        ("Fits", upper(s.clone()), upper_proof),
        ("Padding", padding(s.clone()), padding_proof),
    ] {
        emit(
            literal.builder,
            &format!("Mpk.Go.Practical.String.Value.{name}"),
            &[],
            ty,
            proof,
            true,
        )?;
    }
    emit(
        literal.builder,
        "Mpk.Go.Practical.String.Value.Valid",
        &[],
        valid(s.clone()),
        pc(
            "Make",
            vec![
                upper(s.clone()),
                padding(s),
                constant("Mpk.Go.Practical.String.Value.Fits"),
                constant("Mpk.Go.Practical.String.Value.Padding"),
            ],
        ),
        true,
    )
}
