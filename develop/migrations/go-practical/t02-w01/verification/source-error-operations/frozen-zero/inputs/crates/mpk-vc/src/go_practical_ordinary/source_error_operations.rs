//! Complete source-U nil, copy and nil/case observations using ordinary proofs.
//! These value equations grant no source execution, validity or context authority.
use super::syntax::*;
use super::{error_layout, source_error_boxing, word, Builder, GoOrdinaryError, Result, MAX_BYTES};
use crate::go_practical_type_keys::{GoSourceTypeKey, GoSourceValueDomain};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceErrorOperationsFoundation {
    source_module: String,
    boxing_module: String,
    module: String,
    error_type: String,
    cases: Vec<(String, u64)>,
    certificate: Vec<u8>,
}
impl GoSourceErrorOperationsFoundation {
    pub fn source_layout_module_name(&self) -> &str {
        &self.source_module
    }
    pub fn boxing_module_name(&self) -> &str {
        &self.boxing_module
    }
    pub fn module_name(&self) -> &str {
        &self.module
    }
    pub fn source_error_type_id(&self) -> &str {
        &self.error_type
    }
    pub fn case_tags(&self) -> &[(String, u64)] {
        &self.cases
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
fn member(module: &str, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{module}.{name}"), args)
}
fn truth(value: Expr, expected: bool) -> Expr {
    eq(boolean(), value, bit(expected))
}
struct Operations<'a> {
    builder: &'a mut Builder,
    module: &'a str,
    boxing: &'a str,
    types: Vec<Expr>,
    zeros: Vec<Expr>,
}
impl Operations<'_> {
    fn own(&self, name: &str, args: Vec<Expr>) -> Expr {
        member(self.module, name, args)
    }
    fn observe(&self, name: &str, value: Expr) -> Expr {
        member(self.boxing, name, vec![value])
    }
    fn emit(&mut self, name: &str, args: &[Binder], ty: Expr, body: Expr, law: bool) -> Result<()> {
        emit(
            self.builder,
            &format!("{}.{name}", self.module),
            args,
            ty,
            body,
            law,
        )
    }
    fn payload_recipe(&self, value: Expr, values: Vec<Expr>, recipe: Expr) -> Result<Expr> {
        let carrier = error_layout::product(&self.types);
        let payload = error_layout::product(&self.types[1..]);
        let made = member(self.boxing, "Make", values.clone());
        let made_payload = self.observe("PayloadVector", made.clone());
        Ok(Equality {
            ty: carrier.clone(),
            left: value.clone(),
            right: made,
            proof: recipe,
        }
        .mapped(
            lam(
                "_error",
                carrier,
                self.observe("PayloadVector", var("_error")),
            ),
            self.observe("PayloadVector", value),
            made_payload.clone(),
            payload.clone(),
        )
        .then(Equality {
            ty: payload,
            left: made_payload,
            right: error_layout::make(&self.types[1..], &values[1..]),
            proof: member(self.boxing, "PayloadOfMake", values),
        })?
        .proof)
    }
    fn nil(&mut self, source_zero: Expr) -> Result<()> {
        let carrier = error_layout::product(&self.types);
        let payload = error_layout::product(&self.types[1..]);
        let nil = self.own("Nil", vec![]);
        // Frozen nil uses a constant-false function, while Word.Zero keeps a
        // decision tree. They agree at every address but whole-function
        // equality is not assumed. Preserve the exact frozen constructor.
        let mut raw_zero = bit(false);
        for name in word::ADDRESSES.iter().rev() {
            raw_zero = lam(name, boolean(), raw_zero);
        }
        self.emit("NilTag", &[], word::word_type(64)?, raw_zero, false)?;
        let zero = self.own("NilTag", vec![]);
        let values = std::iter::once(zero.clone())
            .chain(self.zeros.iter().cloned())
            .collect::<Vec<_>>();
        self.emit("Nil", &[], carrier.clone(), source_zero, false)?;
        let nil_payload = error_layout::make(&self.types[1..], &self.zeros);
        self.emit("NilPayload", &[], payload.clone(), nil_payload, false)?;
        self.emit(
            "NilRecipe",
            &[],
            eq(
                carrier.clone(),
                nil.clone(),
                member(self.boxing, "Make", values.clone()),
            ),
            refl(carrier, error_layout::make(&self.types, &values)),
            true,
        )?;
        self.emit(
            "TagOfNil",
            &[],
            eq(
                word::word_type(64)?,
                self.observe("Tag", nil.clone()),
                zero.clone(),
            ),
            refl(word::word_type(64)?, zero.clone()),
            true,
        )?;
        self.emit(
            "PayloadOfNil",
            &[],
            eq(
                payload,
                self.observe("PayloadVector", nil.clone()),
                self.own("NilPayload", vec![]),
            ),
            self.payload_recipe(nil.clone(), values, self.own("NilRecipe", vec![]))?,
            true,
        )?;
        let computed = word::wc(64, "IsZero", vec![zero.clone()]);
        let proof = Equality {
            ty: word::word_type(64)?,
            left: self.observe("Tag", nil.clone()),
            right: zero.clone(),
            proof: self.own("TagOfNil", vec![]),
        }
        .mapped(
            word::wc(64, "IsZero", vec![]),
            self.observe("IsNil", nil.clone()),
            computed.clone(),
            boolean(),
        )
        .then(Equality::boolean(
            computed,
            bit(true),
            call(
                &format!(
                    "Mpk.Go.Practical.ErrorLayout.N{}.FrozenZeroIsZero",
                    self.zeros.len()
                ),
                self.types[1..].to_vec(),
            ),
        ))?;
        self.emit(
            "IsNilOfNil",
            &[],
            truth(self.observe("IsNil", nil.clone()), true),
            proof.proof,
            true,
        )?;
        let proof = Equality::boolean(
            self.observe("IsNil", nil.clone()),
            bit(true),
            self.own("IsNilOfNil", vec![]),
        )
        .mapped(
            constant("Mpk.Go.Practical.Bool.Not"),
            self.observe("IsNonNil", nil.clone()),
            bc("Not", vec![bit(true)]),
            boolean(),
        );
        self.emit(
            "IsNonNilOfNil",
            &[],
            truth(self.observe("IsNonNil", nil), false),
            proof.proof,
            true,
        )
    }
    fn copy(&mut self) -> Result<()> {
        let carrier = error_layout::product(&self.types);
        let args = [("e", carrier.clone())];
        let copied = self.own("Copy", vec![var("e")]);
        let values = (0..self.types.len())
            .map(|i| error_layout::get(&self.types, var("e"), i))
            .collect::<Vec<_>>();
        let raw = error_layout::make(&self.types, &values);
        self.emit("Copy", &args, carrier.clone(), raw.clone(), false)?;
        self.emit(
            "CopyRecipe",
            &args,
            eq(
                carrier.clone(),
                copied.clone(),
                member(self.boxing, "Make", values.clone()),
            ),
            refl(carrier.clone(), raw),
            true,
        )?;
        self.emit(
            "TagOfCopy",
            &args,
            eq(
                word::word_type(64)?,
                self.observe("Tag", copied.clone()),
                self.observe("Tag", var("e")),
            ),
            refl(word::word_type(64)?, self.observe("Tag", var("e"))),
            true,
        )?;
        self.emit(
            "PayloadOfCopy",
            &args,
            eq(
                error_layout::product(&self.types[1..]),
                self.observe("PayloadVector", copied.clone()),
                self.observe("PayloadVector", var("e")),
            ),
            self.payload_recipe(
                copied.clone(),
                values,
                self.own("CopyRecipe", vec![var("e")]),
            )?,
            true,
        )?;
        for i in 1..self.types.len() {
            let original = error_layout::get(&self.types, var("e"), i);
            self.emit(
                &format!("CopyPayload{}", i - 1),
                &args,
                eq(
                    self.types[i].clone(),
                    error_layout::get(&self.types, copied.clone(), i),
                    original.clone(),
                ),
                refl(self.types[i].clone(), original),
                true,
            )?;
        }
        let tag = Equality {
            ty: word::word_type(64)?,
            left: self.observe("Tag", copied.clone()),
            right: self.observe("Tag", var("e")),
            proof: self.own("TagOfCopy", vec![var("e")]),
        };
        let nil_left = self.observe("IsNil", copied.clone());
        let nil_right = self.observe("IsNil", var("e"));
        let proof = tag.clone().mapped(
            word::wc(64, "IsZero", vec![]),
            nil_left.clone(),
            nil_right.clone(),
            boolean(),
        );
        self.emit(
            "IsNilOfCopy",
            &args,
            eq(boolean(), nil_left.clone(), nil_right.clone()),
            proof.proof,
            true,
        )?;
        let proof = Equality::boolean(nil_left, nil_right, self.own("IsNilOfCopy", vec![var("e")]))
            .mapped(
                constant("Mpk.Go.Practical.Bool.Not"),
                self.observe("IsNonNil", copied.clone()),
                self.observe("IsNonNil", var("e")),
                boolean(),
            );
        self.emit(
            "IsNonNilOfCopy",
            &args,
            eq(boolean(), proof.left, proof.right),
            proof.proof,
            true,
        )?;
        let case = var("case");
        self.emit(
            "HasCase",
            &[("case", word::word_type(64)?), ("e", carrier.clone())],
            boolean(),
            word::wc(
                64,
                "Equal",
                vec![case.clone(), self.observe("Tag", var("e"))],
            ),
            false,
        )?;
        let matched = |e| self.own("HasCase", vec![case.clone(), e]);
        let proof = tag.mapped(
            lam(
                "_tag",
                word::word_type(64)?,
                word::wc(64, "Equal", vec![case.clone(), var("_tag")]),
            ),
            matched(copied),
            matched(var("e")),
            boolean(),
        );
        self.emit(
            "HasCaseOfCopy",
            &[("case", word::word_type(64)?), ("e", carrier)],
            eq(boolean(), proof.left, proof.right),
            proof.proof,
            true,
        )
    }
    fn nil_has_no_case(&mut self, id: &str, ordinal: u64) -> Result<()> {
        let prefix = format!("{}.C{id}", self.boxing);
        let tag = member(&prefix, "Tag", vec![]);
        let nil = self.own("Nil", vec![]);
        let zero = self.own("NilTag", vec![]);
        let comparison = word::wc(64, "Equal", vec![tag.clone(), zero.clone()]);
        let observed = self.own("HasCase", vec![tag.clone(), nil.clone()]);
        let proof = Equality {
            ty: word::word_type(64)?,
            left: self.observe("Tag", nil),
            right: zero,
            proof: self.own("TagOfNil", vec![]),
        }
        .mapped(
            lam(
                "_tag",
                word::word_type(64)?,
                word::wc(64, "Equal", vec![tag, var("_tag")]),
            ),
            observed.clone(),
            comparison.clone(),
            boolean(),
        )
        .then(Equality::boolean(
            comparison,
            bit(false),
            call(
                &format!(
                    "Mpk.Go.Practical.ErrorValue.TagDifferentAt{}",
                    ordinal.trailing_zeros()
                ),
                vec![
                    member(&prefix, "Tag", vec![]),
                    self.own("NilTag", vec![]),
                    refl(boolean(), bit(false)),
                ],
            ),
        ))?;
        self.emit(
            &format!("C{id}.NilHasNoCase"),
            &[],
            truth(observed, false),
            proof.proof,
            true,
        )
    }
}
pub fn generate_go_practical_source_error_operations_foundation(
    domain: &GoSourceValueDomain,
) -> Result<GoSourceErrorOperationsFoundation> {
    let (mut builder, boxing) = source_error_boxing::nil_builder(domain)?;
    let source_module = boxing.source_layout_module_name().to_string();
    let boxing_module = boxing.module_name().to_string();
    let cases = boxing.case_tags().to_vec();
    let error_type = domain
        .types()
        .find(|t| matches!(t,GoSourceTypeKey::Builtin{name,..}if name=="error"))
        .ok_or(GoOrdinaryError::Linkage)?
        .id()
        .to_string();
    let module = format!("{boxing_module}.Operations");
    let source = |id, name| constant(&format!("{source_module}.T{id}.{name}"));
    let types = std::iter::once(word::word_type(64)?)
        .chain(cases.iter().map(|(id, _)| source(id, "Carrier")))
        .collect();
    let zeros = cases.iter().map(|(id, _)| source(id, "Zero")).collect();
    let mut generator = Operations {
        builder: &mut builder,
        module: &module,
        boxing: &boxing_module,
        types,
        zeros,
    };
    generator.nil(source(&error_type, "Zero"))?;
    generator.copy()?;
    for (id, ordinal) in &cases {
        generator.nil_has_no_case(id, *ordinal)?;
    }
    builder.certificate.module = module.clone();
    Ok(GoSourceErrorOperationsFoundation {
        source_module,
        boxing_module,
        module,
        error_type,
        cases,
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_source_error_operations_foundation(
    candidate: &[u8],
    domain: &GoSourceValueDomain,
) -> Result<GoSourceErrorOperationsFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_source_error_operations_foundation(domain)?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}
