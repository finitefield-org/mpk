//! Source-key-specific error boxing with every inactive payload zero retained.
//! Tag ordinals belong to this private complete U, not a public context wire.
use super::syntax::*;
use super::{error_layout, source_error_layout, word, Builder, GoOrdinaryError, Result, MAX_BYTES};
use crate::go_practical_type_keys::GoSourceValueDomain;
use std::collections::BTreeSet;

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
    fn tree(value: u64, start: usize, size: usize, depth: usize) -> Expr {
        if size == 1 { return bit(value & (1u64 << start) != 0); }
        let m = size/2;
        let left = tree(value,start,m,depth+1);
        let right = tree(value,start+m,m,depth+1);
        if left == right { left } else {
            word::choose(var(word::ADDRESSES[depth]),right,left,boolean())
        }
    }
    let mut body = tree(value,0,64,0);
    for name in word::ADDRESSES.iter().rev() { body = lam(name,boolean(),body); }
    Ok(body)
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
    nonzero_bits: BTreeSet<usize>,
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
        )?;
        let word = word::word_type(64)?;
        let tag = var("tag");
        let tagged = self.tag(var("e"));
        let zero = word::wc(64, "Zero", vec![]);
        let same_tag = eq(word.clone(), tagged.clone(), tag.clone());
        let observed = Equality {
            ty: word.clone(),
            left: tagged,
            right: tag.clone(),
            proof: var("same_tag"),
        };
        let mut args = vec![
            ("e", args[0].1.clone()),
            ("tag", word.clone()),
            ("same_tag", same_tag),
        ];
        let comparison = word::wc(64, "Equal", vec![tag.clone(), zero.clone()]);
        let proof = observed
            .clone()
            .mapped(
                lam(
                    "_tag",
                    word.clone(),
                    word::wc(64, "Equal", vec![var("_tag"), zero]),
                ),
                self.is_nil(var("e")),
                comparison.clone(),
                boolean(),
            )
            .then(Equality::boolean(
                comparison.clone(),
                bit(false),
                var("nonzero"),
            ))?;
        args.push(("nonzero", truth(comparison, false)));
        self.emit(
            &module,
            "TagNonzeroIsNotNil",
            &args,
            truth(self.is_nil(var("e")), false),
            proof.proof,
            true,
        )?;
        args.pop();
        let own_case = word::wc(64, "Equal", vec![tag.clone(), self.tag(var("e"))]);
        let reflexive = word::wc(64, "Equal", vec![tag.clone(), tag.clone()]);
        let proof = observed
            .mapped(
                lam(
                    "_tag",
                    word,
                    word::wc(64, "Equal", vec![tag.clone(), var("_tag")]),
                ),
                own_case.clone(),
                reflexive.clone(),
                boolean(),
            )
            .then(Equality::boolean(
                reflexive,
                bit(true),
                call("Mpk.Go.Practical.ErrorValue.TagEqualReflexive", vec![tag]),
            ))?;
        self.emit(
            &module,
            "SameTagHasOwnCase",
            &args,
            truth(own_case, true),
            proof.proof,
            true,
        )?;
        let args = [
            ("e", args[0].1.clone()),
            ("not_nil", truth(self.is_nil(var("e")), false)),
        ];
        let proof = Equality::boolean(self.is_nil(var("e")), bit(false), var("not_nil")).mapped(
            constant("Mpk.Go.Practical.Bool.Not"),
            self.is_non_nil(var("e")),
            bc("Not", vec![bit(false)]),
            boolean(),
        );
        self.emit(
            &module,
            "NotNilIsNonNil",
            &args,
            truth(self.is_non_nil(var("e")), true),
            proof.proof,
            true,
        )?;
        let a = var("A");
        let b = var("B");
        let pair = error_layout::product(&[a.clone(),b.clone()]);
        let make = |left, right| error_layout::make(&[a.clone(),b.clone()],&[left,right]);
        let old = make(var("l"),var("r"));
        let middle = make(var("next_l"),var("r"));
        let next = make(var("next_l"),var("next_r"));
        let proof = Equality {ty:a.clone(),left:var("l"),right:var("next_l"),proof:var("left")}
            .mapped(lam("_left",a.clone(),make(var("_left"),var("r"))),old.clone(),middle.clone(),pair.clone())
            .then(Equality {ty:b.clone(),left:var("r"),right:var("next_r"),proof:var("right")}
                .mapped(lam("_right",b.clone(),make(var("next_l"),var("_right"))),middle,next.clone(),pair.clone()))?;
        self.emit(&module,"PairMakeCongruent",&[("A",Expr::Sort0),("B",Expr::Sort0),("l",a.clone()),("next_l",a.clone()),("r",b.clone()),("next_r",b.clone()),("left",eq(a,var("l"),var("next_l"))),("right",eq(b,var("r"),var("next_r")))],
            eq(pair,old,next),proof.proof,true)
    }
    fn nonzero_bit(&mut self, position: usize) -> Result<()> {
        if !self.nonzero_bits.insert(position) {
            return Ok(());
        }
        let module = self.module.clone();
        let word = word::word_type(64)?;
        let tag = var("tag");
        let zero = word::wc(64, "Zero", vec![]);
        let tag_bit = word::at(tag.clone(), 64, position)?;
        let zero_bit = word::at(zero.clone(), 64, position)?;
        let comparison = bc("Equal", vec![tag_bit.clone(), zero_bit.clone()]);
        let after_tag = bc("Equal", vec![bit(true), zero_bit.clone()]);
        let computed = bc("Equal", vec![bit(true), bit(false)]);
        let proof = Equality::boolean(tag_bit.clone(), bit(true), var("set_bit"))
            .mapped(
                lam(
                    "_tag_bit",
                    boolean(),
                    bc("Equal", vec![var("_tag_bit"), zero_bit.clone()]),
                ),
                comparison,
                after_tag.clone(),
                boolean(),
            )
            .then(
                Equality::boolean(zero_bit, bit(false), refl(boolean(), bit(false))).mapped(
                    lam(
                        "_zero_bit",
                        boolean(),
                        bc("Equal", vec![bit(true), var("_zero_bit")]),
                    ),
                    after_tag,
                    computed.clone(),
                    boolean(),
                ),
            )?
            .then(Equality::boolean(
                computed,
                bit(false),
                refl(boolean(), bit(false)),
            ))?;
        self.emit(
            &module,
            &format!("SetBit{position}IsNonzero"),
            &[("tag", word), ("set_bit", truth(tag_bit, true))],
            truth(
                word::wc(64, "Equal", vec![tag.clone(), zero.clone()]),
                false,
            ),
            call(
                &format!("Mpk.Go.Practical.ErrorValue.TagDifferentAt{position}"),
                vec![tag, zero, proof.proof],
            ),
            true,
        )
    }
    fn case(&mut self, index: usize, ordinal: u64) -> Result<()> {
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
        let payload = own(&prefix, "Payload", vec![var("v")]);
        let observed_payload = own(&self.module, "PayloadVector", vec![boxed.clone()]);
        let payload_proof = if self.cases.len() == 1 {
            refl(payload_type.clone(),payload.clone())
        } else {
            let middle = self.cases.len()/2;
            let a = error_layout::product(&self.types[1..middle+1]);
            let b = error_layout::product(&self.types[middle+1..]);
            let projection = |right: bool, value| call(
                if right {"Mpk.Go.Practical.Product.Right"}else{"Mpk.Go.Practical.Product.Left"},
                vec![a.clone(),b.clone(),value]);
            let left = projection(false,observed_payload.clone());
            let next_left = projection(false,payload.clone());
            let right = projection(true,observed_payload.clone());
            let next_right = projection(true,payload.clone());
            own(&self.module,"PairMakeCongruent",vec![a.clone(),b.clone(),left,next_left.clone(),right,next_right.clone(),refl(a,next_left),refl(b,next_right)])
        };
        self.emit(
            &prefix,
            "PayloadOfBox",
            &input,
            eq(
                payload_type.clone(),
                observed_payload,
                payload,
            ),
            payload_proof,
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
        self.nonzero_bit(position)?;
        self.emit(
            &prefix,
            "TagNonzero",
            &[],
            truth(
                word::wc(64, "Equal", vec![tag.clone(), zero.clone()]),
                false,
            ),
            own(
                &self.module,
                &format!("SetBit{position}IsNonzero"),
                vec![tag.clone(), refl(boolean(), bit(true))],
            ),
            true,
        )?;
        self.emit(
            &prefix,
            "IsNilOfBox",
            &input,
            truth(self.is_nil(boxed.clone()), false),
            own(
                &self.module,
                "TagNonzeroIsNotNil",
                vec![
                    boxed.clone(),
                    tag.clone(),
                    own(&prefix, "TagOfBox", vec![var("v")]),
                    own(&prefix, "TagNonzero", vec![]),
                ],
            ),
            true,
        )?;
        self.emit(
            &prefix,
            "IsNonNilOfBox",
            &input,
            truth(self.is_non_nil(boxed.clone()), true),
            own(
                &self.module,
                "NotNilIsNonNil",
                vec![boxed.clone(), own(&prefix, "IsNilOfBox", vec![var("v")])],
            ),
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
        self.emit(
            &prefix,
            "HasOwnCase",
            &input,
            truth(own_case, true),
            own(
                &self.module,
                "SameTagHasOwnCase",
                vec![boxed, tag, own(&prefix, "TagOfBox", vec![var("v")])],
            ),
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
    let (mut builder, layout, _) = source_error_layout::source_builder(domain)?;
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
        nonzero_bits: BTreeSet::new(),
    };
    generator.shared()?;
    let mut tags = vec![];
    for (index, id) in cases.into_iter().enumerate() {
        let ordinal = u64::try_from(index)
            .ok()
            .and_then(|i| i.checked_add(1))
            .ok_or(GoOrdinaryError::Limit)?;
        generator.case(index, ordinal)?;
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
