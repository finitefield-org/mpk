//! Typed compositional reads/updates always consult the current root value.
//! These ordinary functions grant no source, path, owner or permission authority.
use super::syntax::*;
use super::{Builder, Result};
mod algebra;
mod arrays;
mod frames;
mod products;
const PREFIX: &str = "Mpk.Go.Practical.Storage.Lens";
fn lc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{PREFIX}.{name}"), args)
}
fn pc(name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("Mpk.Go.Practical.Product.{name}"), args)
}
fn function(a: Expr, b: Expr) -> Expr {
    pi("_arg", a, b)
}
fn product(a: Expr, b: Expr) -> Expr {
    pi(
        "_tag",
        boolean(),
        call("Std.Bool.recType", vec![a, b, var("_tag")]),
    )
}
fn lens(a: Expr, b: Expr) -> Expr {
    product(
        function(a.clone(), b.clone()),
        function(a.clone(), function(b, a)),
    )
}
fn make(a: Expr, b: Expr, get: Expr, put: Expr) -> Expr {
    lc("Make", vec![a, b, get, put])
}
fn get(a: Expr, b: Expr, l: Expr, s: Expr) -> Expr {
    lc("Get", vec![a, b, l, s])
}
fn put(a: Expr, b: Expr, l: Expr, s: Expr, x: Expr) -> Expr {
    lc("Put", vec![a, b, l, s, x])
}
fn compose(a: Expr, b: Expr, c: Expr, l: Expr, m: Expr) -> Expr {
    lc("Compose", vec![a, b, c, l, m])
}
fn predicate(a: Expr) -> Expr {
    function(a, Expr::Sort0)
}
fn relation(a: Expr) -> Expr {
    function(a.clone(), function(a, Expr::Sort0))
}
fn holds(r: Expr, x: Expr, y: Expr) -> Expr {
    app(r, vec![x, y])
}
fn get_put(a: Expr, b: Expr, l: Expr) -> Expr {
    pi(
        "_s",
        a.clone(),
        pi(
            "_x",
            b.clone(),
            eq(
                b.clone(),
                get(
                    a.clone(),
                    b.clone(),
                    l.clone(),
                    put(a, b, l, var("_s"), var("_x")),
                ),
                var("_x"),
            ),
        ),
    )
}
fn put_get(a: Expr, b: Expr, r: Expr, l: Expr) -> Expr {
    pi(
        "_s",
        a.clone(),
        holds(
            r,
            put(
                a.clone(),
                b.clone(),
                l.clone(),
                var("_s"),
                get(a, b, l, var("_s")),
            ),
            var("_s"),
        ),
    )
}
fn put_put(a: Expr, b: Expr, r: Expr, l: Expr) -> Expr {
    pi(
        "_s",
        a.clone(),
        pi(
            "_x",
            b.clone(),
            pi(
                "_y",
                b.clone(),
                holds(
                    r,
                    put(
                        a.clone(),
                        b.clone(),
                        l.clone(),
                        put(a.clone(), b.clone(), l.clone(), var("_s"), var("_x")),
                        var("_y"),
                    ),
                    put(a, b, l, var("_s"), var("_y")),
                ),
            ),
        ),
    )
}
fn get_respect(a: Expr, b: Expr, r: Expr, s: Expr, l: Expr) -> Expr {
    pi(
        "_s",
        a.clone(),
        pi(
            "_t",
            a.clone(),
            pi(
                "_h",
                holds(r, var("_s"), var("_t")),
                holds(
                    s,
                    get(a.clone(), b.clone(), l.clone(), var("_s")),
                    get(a, b, l, var("_t")),
                ),
            ),
        ),
    )
}
fn put_respect(a: Expr, b: Expr, r: Expr, s: Expr, l: Expr) -> Expr {
    pi(
        "_s",
        a.clone(),
        pi(
            "_x",
            b.clone(),
            pi(
                "_y",
                b.clone(),
                pi(
                    "_h",
                    holds(s, var("_x"), var("_y")),
                    holds(
                        r,
                        put(a.clone(), b.clone(), l.clone(), var("_s"), var("_x")),
                        put(a, b, l, var("_s"), var("_y")),
                    ),
                ),
            ),
        ),
    )
}
fn get_valid(a: Expr, b: Expr, p: Expr, q: Expr, l: Expr) -> Expr {
    pi(
        "_s",
        a.clone(),
        pi(
            "_valid",
            app(p, vec![var("_s")]),
            app(q, vec![get(a, b, l, var("_s"))]),
        ),
    )
}
fn put_valid(a: Expr, b: Expr, p: Expr, q: Expr, l: Expr) -> Expr {
    pi(
        "_s",
        a.clone(),
        pi(
            "_x",
            b.clone(),
            pi(
                "_valid",
                app(p.clone(), vec![var("_s")]),
                pi(
                    "_value_valid",
                    app(q, vec![var("_x")]),
                    app(p, vec![put(a, b, l, var("_s"), var("_x"))]),
                ),
            ),
        ),
    )
}
fn transitive(a: Expr, r: Expr) -> Expr {
    pi(
        "_x",
        a.clone(),
        pi(
            "_y",
            a.clone(),
            pi(
                "_z",
                a,
                pi(
                    "_xy",
                    holds(r.clone(), var("_x"), var("_y")),
                    pi(
                        "_yz",
                        holds(r.clone(), var("_y"), var("_z")),
                        holds(r, var("_x"), var("_z")),
                    ),
                ),
            ),
        ),
    )
}
fn retype(a: Expr, p: Expr, e: Equality, h: Expr) -> Expr {
    call("Std.Eq.rewrite", vec![a, e.left, e.right, p, e.proof, h])
}
fn transfer(a: Expr, r: Expr, x: Equality, y: Equality, h: Expr) -> Expr {
    let first = retype(
        a.clone(),
        lam(
            "_transport_left",
            a.clone(),
            holds(r.clone(), var("_transport_left"), y.right.clone()),
        ),
        x.clone().reverse(),
        h,
    );
    retype(
        a.clone(),
        lam(
            "_transport_right",
            a,
            holds(r, x.left, var("_transport_right")),
        ),
        y.reverse(),
        first,
    )
}
struct Lenses<'a>(&'a mut Builder);
impl Lenses<'_> {
    fn emit(&mut self, name: &str, args: &[Binder], ty: Expr, body: Expr, law: bool) -> Result<()> {
        emit(self.0, &format!("{PREFIX}.{name}"), args, ty, body, law)
    }
    fn theorem(&mut self, name: &str, args: &[Binder], e: Equality) -> Result<()> {
        self.emit(name, args, eq(e.ty, e.left, e.right), e.proof, true)
    }
    fn definitions(&mut self) -> Result<()> {
        let (a, b) = (var("A"), var("B"));
        let types = [("A", Expr::Sort0), ("B", Expr::Sort0)];
        self.emit(
            "Type",
            &types,
            Expr::Sort0,
            lens(a.clone(), b.clone()),
            false,
        )?;
        let mut args = types.to_vec();
        args.extend([
            ("get", function(a.clone(), b.clone())),
            ("put", function(a.clone(), function(b.clone(), a.clone()))),
        ]);
        self.emit(
            "Make",
            &args,
            lens(a.clone(), b.clone()),
            pc(
                "Make",
                vec![
                    function(a.clone(), b.clone()),
                    function(a.clone(), function(b.clone(), a.clone())),
                    var("get"),
                    var("put"),
                ],
            ),
            false,
        )?;
        let mut read = types.to_vec();
        read.extend([("l", lens(a.clone(), b.clone())), ("s", a.clone())]);
        self.emit(
            "Get",
            &read,
            b.clone(),
            app(
                pc(
                    "Left",
                    vec![
                        function(a.clone(), b.clone()),
                        function(a.clone(), function(b.clone(), a.clone())),
                        var("l"),
                    ],
                ),
                vec![var("s")],
            ),
            false,
        )?;
        let mut write = read;
        write.push(("x", b.clone()));
        self.emit(
            "Put",
            &write,
            a.clone(),
            app(
                pc(
                    "Right",
                    vec![
                        function(a.clone(), b.clone()),
                        function(a.clone(), function(b.clone(), a.clone())),
                        var("l"),
                    ],
                ),
                vec![var("s"), var("x")],
            ),
            false,
        )?;
        let mut read = args.clone();
        read.push(("s", a.clone()));
        let l = make(a.clone(), b.clone(), var("get"), var("put"));
        self.theorem(
            "GetOfMake",
            &read,
            Equality {
                ty: b.clone(),
                left: get(a.clone(), b.clone(), l.clone(), var("s")),
                right: app(var("get"), vec![var("s")]),
                proof: refl(b.clone(), app(var("get"), vec![var("s")])),
            },
        )?;
        let mut write = read;
        write.push(("x", b.clone()));
        self.theorem(
            "PutOfMake",
            &write,
            Equality {
                ty: a.clone(),
                left: put(a.clone(), b.clone(), l, var("s"), var("x")),
                right: app(var("put"), vec![var("s"), var("x")]),
                proof: refl(a.clone(), app(var("put"), vec![var("s"), var("x")])),
            },
        )?;
        self.emit(
            "Identity",
            &types[..1],
            lens(a.clone(), a.clone()),
            make(
                a.clone(),
                a.clone(),
                lam("_s", a.clone(), var("_s")),
                lam("_s", a.clone(), lam("_x", a.clone(), var("_x"))),
            ),
            false,
        )?;
        let c = var("C");
        let args = [
            ("A", Expr::Sort0),
            ("B", Expr::Sort0),
            ("C", Expr::Sort0),
            ("l", lens(a.clone(), b.clone())),
            ("m", lens(b.clone(), c.clone())),
        ];
        let g = get(
            b.clone(),
            c.clone(),
            var("m"),
            get(a.clone(), b.clone(), var("l"), var("_s")),
        );
        let p = put(
            a.clone(),
            b.clone(),
            var("l"),
            var("_s"),
            put(
                b.clone(),
                c.clone(),
                var("m"),
                get(a.clone(), b.clone(), var("l"), var("_s")),
                var("_x"),
            ),
        );
        self.emit(
            "Compose",
            &args,
            lens(a.clone(), c.clone()),
            make(
                a.clone(),
                c.clone(),
                lam("_s", a.clone(), g),
                lam("_s", a.clone(), lam("_x", c.clone(), p)),
            ),
            false,
        )?;
        let composite = compose(a.clone(), b.clone(), c.clone(), var("l"), var("m"));
        let mut read = args.to_vec();
        read.push(("s", a.clone()));
        let g = get(
            b.clone(),
            c.clone(),
            var("m"),
            get(a.clone(), b.clone(), var("l"), var("s")),
        );
        self.theorem(
            "ComposeGet",
            &read,
            Equality {
                ty: c.clone(),
                left: get(a.clone(), c.clone(), composite.clone(), var("s")),
                right: g.clone(),
                proof: refl(c.clone(), g),
            },
        )?;
        let mut write = read;
        write.push(("x", c.clone()));
        let p = put(
            a.clone(),
            b.clone(),
            var("l"),
            var("s"),
            put(
                b,
                c.clone(),
                var("m"),
                get(a.clone(), var("B"), var("l"), var("s")),
                var("x"),
            ),
        );
        self.theorem(
            "ComposePut",
            &write,
            Equality {
                ty: a.clone(),
                left: put(a.clone(), c, composite, var("s"), var("x")),
                right: p.clone(),
                proof: refl(a, p),
            },
        )
    }
}
pub(super) fn append(builder: &mut Builder) -> Result<()> {
    let mut l = Lenses(builder);
    l.definitions()?;
    l.algebra()?;
    l.frames()?;
    l.products()?;
    l.arrays()
}
