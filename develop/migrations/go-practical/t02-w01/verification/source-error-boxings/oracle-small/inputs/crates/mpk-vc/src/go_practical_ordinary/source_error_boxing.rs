//! Source-key-specific error boxing with every inactive payload zero retained.
//! Tag ordinals belong to this private complete U, not a public context wire.
use super::syntax::*;
use super::{
    error_layout, source_error_layout, word, word_proofs, Builder, GoOrdinaryError, Result,
    MAX_BYTES,
};
use crate::go_practical_type_keys::GoSourceValueDomain;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceErrorBoxingFoundation {
    source_module: String,
    module: String,
    cases: Vec<(String, u64)>,
    certificate: Vec<u8>,
}
impl GoSourceErrorBoxingFoundation {
    pub fn source_layout_module_name(&self) -> &str {
        &self.source_module
    }
    pub fn module_name(&self) -> &str {
        &self.module
    }
    pub fn case_tags(&self) -> &[(String, u64)] {
        &self.cases
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
}
fn source(module: &str, id: &str, member: &str) -> Expr {
    constant(&format!("{module}.T{id}.{member}"))
}
fn literal(value: u64) -> Result<Expr> {
    word::vector(64, (0..64).map(|i| bit(value & (1u64 << i) != 0)).collect())
}
fn truth(value: Expr, yes: bool) -> Expr {
    eq(boolean(), value, bit(yes))
}
fn own(prefix: &str, name: &str, args: Vec<Expr>) -> Expr {
    call(&format!("{prefix}.{name}"), args)
}

struct Boxing<'a> {
    builder: &'a mut Builder,
    module: String,
    source_module: String,
    cases: Vec<String>,
    types: Vec<Expr>,
    zeros: Vec<Expr>,
}
impl Boxing<'_> {
    fn emit(
        &mut self,
        prefix: &str,
        name: &str,
        args: &[Binder],
        ty: Expr,
        value: Expr,
        theorem: bool,
    ) -> Result<()> {
        emit(
            self.builder,
            &format!("{prefix}.{name}"),
            args,
            ty,
            value,
            theorem,
        )
    }
    fn tag(&self, value: Expr) -> Expr {
        own(&self.module, "Tag", vec![value])
    }
    fn is_nil(&self, value: Expr) -> Expr {
        own(&self.module, "IsNil", vec![value])
    }
    fn is_non_nil(&self, value: Expr) -> Expr {
        own(&self.module, "IsNonNil", vec![value])
    }
    fn shared(&mut self) -> Result<()> {
        let module = self.module.clone();
        let carrier = error_layout::product(&self.types);
        let args = [("e", carrier)];
        self.emit(
            &module,
            "Tag",
            &args,
            word::word_type(64)?,
            error_layout::get(&self.types, var("e"), 0),
            false,
        )?;
        let payload = error_layout::product(&self.types[1..]);
        let values = (1..self.types.len())
            .map(|i| error_layout::get(&self.types, var("e"), i))
            .collect::<Vec<_>>();
        self.emit(
            &module,
            "PayloadVector",
            &args,
            payload,
            error_layout::make(&self.types[1..], &values),
            false,
        )?;
        self.emit(
            &module,
            "IsNil",
            &args,
            boolean(),
            word::wc(64, "IsZero", vec![self.tag(var("e"))]),
            false,
        )?;
        self.emit(
            &module,
            "IsNonNil",
            &args,
            boolean(),
            bc("Not", vec![self.is_nil(var("e"))]),
            false,
        )
    }
    fn case(&mut self, index: usize, ordinal: u64, recipes: &BTreeMap<String, Expr>) -> Result<()> {
        let id = self.cases[index].clone();
        let prefix = format!("{}.C{id}", self.module);
        let tag = own(&prefix, "Tag", vec![]);
        let raw_tag = literal(ordinal)?;
        let word = word::word_type(64)?;
        let input = [("v", self.types[index + 1].clone())];
        let payload_type = error_layout::product(&self.types[1..]);
        let mut payloads = self.zeros.clone();
        payloads[index] = var("v");
        let raw_payload = error_layout::make(&self.types[1..], &payloads);
        let carrier = error_layout::product(&self.types);
        let values = std::iter::once(tag.clone())
            .chain(payloads)
            .collect::<Vec<_>>();
        let raw_box = error_layout::make(&self.types, &values);
        let boxed = own(&prefix, "Box", vec![var("v")]);
        self.emit(&prefix, "Tag", &[], word.clone(), raw_tag.clone(), false)?;
        self.emit(
            &prefix,
            "Payload",
            &input,
            payload_type.clone(),
            raw_payload,
            false,
        )?;
        self.emit(&prefix, "Box", &input, carrier.clone(), raw_box, false)?;
        self.emit(
            &prefix,
            "TagOfBox",
            &input,
            eq(word.clone(), self.tag(boxed.clone()), tag.clone()),
            refl(word.clone(), tag.clone()),
            true,
        )?;
        self.emit(
            &prefix,
            "PayloadOfBox",
            &input,
            eq(
                payload_type.clone(),
                own(&self.module, "PayloadVector", vec![boxed.clone()]),
                own(&prefix, "Payload", vec![var("v")]),
            ),
            refl(payload_type, own(&prefix, "Payload", vec![var("v")])),
            true,
        )?;
        self.emit(
            &prefix,
            "ActiveOfBox",
            &input,
            eq(
                input[0].1.clone(),
                error_layout::get(&self.types, boxed.clone(), index + 1),
                var("v"),
            ),
            refl(input[0].1.clone(), var("v")),
            true,
        )?;
        let zero = word::wc(64, "Zero", vec![]);
        let position = ordinal.trailing_zeros() as usize;
        let different = word_proofs::Evaluation::new(recipes).equation(
            bc(
                "Equal",
                vec![
                    word::at(raw_tag.clone(), 64, position)?,
                    word::at(zero.clone(), 64, position)?,
                ],
            ),
            bit(false),
        )?;
        self.emit(
            &prefix,
            "TagNonzero",
            &[],
            truth(
                word::wc(64, "Equal", vec![tag.clone(), zero.clone()]),
                false,
            ),
            call(
                &format!("Mpk.Go.Practical.ErrorValue.TagDifferentAt{position}"),
                vec![tag.clone(), zero.clone(), different.proof],
            ),
            true,
        )?;
        let tag_of_box = Equality {
            ty: word.clone(),
            left: self.tag(boxed.clone()),
            right: tag.clone(),
            proof: own(&prefix, "TagOfBox", vec![var("v")]),
        };
        let nil_comparison = word::wc(64, "Equal", vec![tag.clone(), zero.clone()]);
        let non_nil = tag_of_box
            .clone()
            .mapped(
                lam(
                    "_tag",
                    word.clone(),
                    word::wc(64, "Equal", vec![var("_tag"), zero]),
                ),
                self.is_nil(boxed.clone()),
                nil_comparison.clone(),
                boolean(),
            )
            .then(Equality::boolean(
                nil_comparison,
                bit(false),
                own(&prefix, "TagNonzero", vec![]),
            ))?;
        self.emit(
            &prefix,
            "IsNilOfBox",
            &input,
            truth(self.is_nil(boxed.clone()), false),
            non_nil.proof,
            true,
        )?;
        let non_nil = Equality::boolean(
            self.is_nil(boxed.clone()),
            bit(false),
            own(&prefix, "IsNilOfBox", vec![var("v")]),
        )
        .mapped(
            constant("Mpk.Go.Practical.Bool.Not"),
            self.is_non_nil(boxed.clone()),
            bc("Not", vec![bit(false)]),
            boolean(),
        );
        self.emit(
            &prefix,
            "IsNonNilOfBox",
            &input,
            truth(self.is_non_nil(boxed.clone()), true),
            non_nil.proof,
            true,
        )?;
        self.emit(
            &prefix,
            "ConcreteZeroIsNonNil",
            &[],
            truth(
                self.is_non_nil(own(&prefix, "Box", vec![self.zeros[index].clone()])),
                true,
            ),
            own(&prefix, "IsNonNilOfBox", vec![self.zeros[index].clone()]),
            true,
        )?;
        let own_case = word::wc(64, "Equal", vec![tag.clone(), self.tag(boxed.clone())]);
        let same_tag = word::wc(64, "Equal", vec![tag.clone(), tag.clone()]);
        let same = tag_of_box
            .mapped(
                lam(
                    "_tag",
                    word.clone(),
                    word::wc(64, "Equal", vec![tag.clone(), var("_tag")]),
                ),
                own_case.clone(),
                same_tag.clone(),
                boolean(),
            )
            .then(Equality::boolean(
                same_tag,
                bit(true),
                call("Mpk.Go.Practical.ErrorValue.TagEqualReflexive", vec![tag]),
            ))?;
        self.emit(
            &prefix,
            "HasOwnCase",
            &input,
            truth(own_case, true),
            same.proof,
            true,
        )?;
        // This names the exact original source payload type, separately from
        // the same carrier shape of another nominal error case.
        self.emit(
            &prefix,
            "SourcePayloadCarrier",
            &[],
            Expr::Sort0,
            source(&self.source_module, &id, "Carrier"),
            false,
        )
    }
}

/// Derive one nonzero private tag per source-type-id-sorted case of the same U.
/// Every constructor retains the complete U, active payload, and source zeros.
pub fn generate_go_practical_source_error_boxing_foundation(
    domain: &GoSourceValueDomain,
) -> Result<GoSourceErrorBoxingFoundation> {
    let (mut builder, layout, recipes) = source_error_layout::source_builder(domain)?;
    let source_module = layout.module_name().to_string();
    let module = format!("{source_module}.Boxing");
    let cases = layout.case_type_ids().to_vec();
    let types = std::iter::once(word::word_type(64)?)
        .chain(cases.iter().map(|id| source(&source_module, id, "Carrier")))
        .collect();
    let zeros = cases
        .iter()
        .map(|id| source(&source_module, id, "Zero"))
        .collect();
    let mut generator = Boxing {
        builder: &mut builder,
        module: module.clone(),
        source_module: source_module.clone(),
        cases: cases.clone(),
        types,
        zeros,
    };
    generator.shared()?;
    let mut tags = vec![];
    for (index, id) in cases.into_iter().enumerate() {
        let ordinal = u64::try_from(index)
            .ok()
            .and_then(|i| i.checked_add(1))
            .ok_or(GoOrdinaryError::Limit)?;
        generator.case(index, ordinal, &recipes)?;
        tags.push((id, ordinal));
    }
    builder.certificate.module = module.clone();
    Ok(GoSourceErrorBoxingFoundation {
        source_module,
        module,
        cases: tags,
        certificate: builder.finish()?,
    })
}
pub fn import_go_practical_source_error_boxing_foundation(
    candidate: &[u8],
    domain: &GoSourceValueDomain,
) -> Result<GoSourceErrorBoxingFoundation> {
    if candidate.len() > MAX_BYTES {
        return Err(GoOrdinaryError::Limit);
    }
    let expected = generate_go_practical_source_error_boxing_foundation(domain)?;
    if candidate != expected.certificate_bytes() {
        return Err(GoOrdinaryError::Linkage);
    }
    Ok(expected)
}
