//! Masked observations of reused slots and separate past-state namespaces.
use super::*;
fn at(slot: Expr) -> Vec<Expr> {
    let mut a = vec![slot];
    a.extend(addresses(7, "i"));
    a
}
fn updated(f: Expr) -> Expr {
    let q = addresses(8, "__liveq");
    cube_lambda(
        8,
        "__liveq",
        choose_t(
            value_type(),
            bool_eq(q[0].clone(), literal(false)),
            leaf(v("new"), &q[1..]),
            leaf(f, &q),
        ),
    )
}
fn observed(f: Expr, slot: Expr) -> Expr {
    choose_t(
        value_type(),
        apply(v("mask"), vec![slot.clone()]),
        leaf(f, &at(slot)),
        v("default"),
    )
}
fn observe_value(value: Expr, slot: Expr) -> Expr {
    choose_t(
        value_type(),
        apply(v("mask"), vec![slot]),
        value,
        v("default"),
    )
}
fn dead_proof() -> Expr {
    let zero = literal(false);
    let one = literal(true);
    let left0 = observed(updated(v("store")), zero.clone());
    let right0 = observed(v("store"), zero.clone());
    let erase = |read: Expr| {
        apply(
            c("Std.Eq.congr"),
            vec![
                bool_ty(),
                value_type(),
                lambda(
                    "flag",
                    bool_ty(),
                    choose_t(value_type(), v("flag"), read, v("default")),
                ),
                apply(v("mask"), vec![zero.clone()]),
                zero.clone(),
                v("free"),
            ],
        )
    };
    let pl = erase(leaf(updated(v("store")), &at(zero.clone())));
    let pr = erase(leaf(v("store"), &at(zero.clone())));
    let reverse = apply(
        c("Std.Eq.symm"),
        vec![value_type(), right0.clone(), v("default"), pr],
    );
    let false_case = trans(left0, v("default"), right0, pl, reverse);
    let l = leaf(updated(v("store")), &at(one.clone()));
    let r = leaf(v("store"), &at(one.clone()));
    let read_equal = prove(&same(l.clone(), r.clone()), &[], &mut 0);
    let true_case = apply(
        c("Std.Eq.congr"),
        vec![
            value_type(),
            value_type(),
            lambda("cell", value_type(), observe_value(v("cell"), one)),
            l,
            r,
            read_equal,
        ],
    );
    let motive = lambda(
        "slot",
        bool_ty(),
        same(
            observed(updated(v("store")), v("slot")),
            observed(v("store"), v("slot")),
        ),
    );
    apply(
        c("Std.Bool.cases"),
        vec![motive, false_case, true_case, v("slot")],
    )
}
pub(super) fn generate(root: &Path, output: &Path) {
    fs::create_dir_all(output).unwrap();
    let mut rows = vec![];
    for family in [
        "dead-slot-guard",
        "masked-replacement-frame",
        "past-store-namespace",
    ] {
        for altered in [false, true] {
            // The same safe-frame theorem supports both rejection boundaries;
            // retain one positive certificate rather than count it twice.
            if family == "masked-replacement-frame" && !altered {
                continue;
            }
            let mut args = vec![
                ("A".into(), c("__Sort0")),
                ("store".into(), cube(8)),
                ("new".into(), cube(7)),
            ];
            let (goal, proof) = if family == "past-store-namespace" {
                args.push(("old".into(), cube(8)));
                args.push(("slot".into(), bool_ty()));
                args.extend((0..7).map(|i| (format!("i{i}"), bool_ty())));
                let old_read = leaf(v("old"), &at(v("slot")));
                let after = apply(
                    lambda("current", cube(8), old_read.clone()),
                    vec![updated(v("store"))],
                );
                let valid = same(after.clone(), old_read.clone());
                (
                    if altered {
                        same(after, leaf(updated(v("store")), &at(v("slot"))))
                    } else {
                        valid.clone()
                    },
                    prove(&valid, &[], &mut 0),
                )
            } else {
                args.extend([
                    ("mask".into(), forall("_slot", bool_ty(), bool_ty())),
                    ("default".into(), value_type()),
                    (
                        "free".into(),
                        eq(
                            apply(v("mask"), vec![literal(false)]),
                            literal(altered && family == "dead-slot-guard"),
                        ),
                    ),
                    ("slot".into(), bool_ty()),
                ]);
                args.extend((0..7).map(|i| (format!("i{i}"), bool_ty())));
                let right = if altered && family == "masked-replacement-frame" {
                    leaf(v("store"), &at(v("slot")))
                } else {
                    observed(v("store"), v("slot"))
                };
                (
                    same(observed(updated(v("store")), v("slot")), right),
                    dead_proof(),
                )
            };
            let mut b = Builder::from(natural::foundation(root));
            b.c.module = MODULE.into();
            let (ty, value) = fold_binders(&args, goal, proof);
            let ty = compile(&ty, &mut vec![], &mut b);
            let value = compile(&value, &mut vec![], &mut b);
            let name = b.c.name_table.len() as u32;
            b.c.name_table.push(format!("{MODULE}.Claim"));
            b.c.declarations.push(Declaration {
                name,
                kind: DeclarationKind::Theorem { ty, proof: value },
            });
            let terms = b.c.term_table.len();
            let declarations = b.c.declarations.len();
            let raw = b.finish();
            let id = format!("{family}-{}", if altered { "altered" } else { "valid" });
            fs::write(output.join(format!("{id}.mpcert")), &raw).unwrap();
            rows.push(json!({"id":id,"family":family,"altered":altered,"expected":if altered {"rejected"}else{"accepted"},"bytes":raw.len(),"terms":terms,"declarations":declarations,"theorem":format!("{MODULE}.Claim"),"scope":"ordinary arbitrary-Sort0 masked frame for two 128-cell slots and separate past store; reference-closure/source/allocator binding remains required","value_enumeration":false,"address_enumeration":false}));
        }
    }
    fs::write(
        output.join("index.json"),
        serde_json::to_vec_pretty(
            &json!({"schema":"mpk.go_practical.lifetime_pilot.v0","cases":rows}),
        )
        .unwrap(),
    )
    .unwrap();
}
