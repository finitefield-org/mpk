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
        if size == 1 {
            return bit(value & (1u64 << start) != 0);
        }
        let m = size / 2;
        let left = tree(value, start, m, depth + 1);
        let right = tree(value, start + m, m, depth + 1);
        if left == right {
            left
        } else if left == bit(false) && right == bit(true) {
            var(word::ADDRESSES[depth])
        } else if left == bit(true) && right == bit(false) {
            bc("Not", vec![var(word::ADDRESSES[depth])])
        } else {
            word::choose(var(word::ADDRESSES[depth]), right, left, boolean())
        }
    }
    let mut body = tree(value, 0, 64, 0);
    for name in word::ADDRESSES.iter().rev() {
        body = lam(name, boolean(), body);
    }
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
        let pair = error_layout::product(&[a.clone(), b.clone()]);
        let make = |left, right| error_layout::make(&[a.clone(), b.clone()], &[left, right]);
        let old = make(var("l"), var("r"));
        let middle = make(var("next_l"), var("r"));
        let next = make(var("next_l"), var("next_r"));
        let proof = Equality {
            ty: a.clone(),
            left: var("l"),
            right: var("next_l"),
            proof: var("left"),
        }
        .mapped(
            lam("_left", a.clone(), make(var("_left"), var("r"))),
            old.clone(),
            middle.clone(),
            pair.clone(),
        )
        .then(
            Equality {
                ty: b.clone(),
                left: var("r"),
                right: var("next_r"),
                proof: var("right"),
            }
            .mapped(
                lam("_right", b.clone(), make(var("next_l"), var("_right"))),
                middle,
                next.clone(),
                pair.clone(),
            ),
        )?;
        self.emit(
            &module,
            "PairMakeCongruent",
            &[
                ("A", Expr::Sort0),
                ("B", Expr::Sort0),
                ("l", a.clone()),
                ("next_l", a.clone()),
                ("r", b.clone()),
                ("next_r", b.clone()),
                ("left", eq(a, var("l"), var("next_l"))),
                ("right", eq(b, var("r"), var("next_r"))),
            ],
            eq(pair, old, next),
            proof.proof,
            true,
        )
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
    fn payload_equality(&self, types: &[Expr], left: &[Expr], right: &[Expr]) -> Expr {
        if types.is_empty() {
            return refl(boolean(), bit(false));
        }
        if let [one] = types {
            return refl(one.clone(), right[0].clone());
        }
        let middle = types.len() / 2;
        own(
            &self.module,
            "PairMakeCongruent",
            vec![
                error_layout::product(&types[..middle]),
                error_layout::product(&types[middle..]),
                error_layout::make(&types[..middle], &left[..middle]),
                error_layout::make(&types[..middle], &right[..middle]),
                error_layout::make(&types[middle..], &left[middle..]),
                error_layout::make(&types[middle..], &right[middle..]),
                self.payload_equality(&types[..middle], &left[..middle], &right[..middle]),
                self.payload_equality(&types[middle..], &left[middle..], &right[middle..]),
            ],
        )
    }
    fn payload_make_law(&mut self) -> Result<()> {
        let parameters = self
            .types
            .iter()
            .enumerate()
            .map(|(i, ty)| (format!("value{i}"), ty.clone()))
            .collect::<Vec<_>>();
        let values = parameters.iter().map(|(n, _)| var(n)).collect::<Vec<_>>();
        let mut ty = error_layout::product(&self.types);
        let mut body = error_layout::make(&self.types, &values);
        for (name, domain) in parameters.iter().rev() {
            ty = pi(name, domain.clone(), ty);
            body = lam(name, domain.clone(), body);
        }
        let module = self.module.clone();
        self.emit(&module, "Make", &[], ty, body, false)?;
        let made = own(&module, "Make", values.clone());
        let observed_values = (1..self.types.len())
            .map(|i| error_layout::get(&self.types, made.clone(), i))
            .collect::<Vec<_>>();
        let mut ty = eq(
            error_layout::product(&self.types[1..]),
            own(&module, "PayloadVector", vec![made]),
            error_layout::make(&self.types[1..], &values[1..]),
        );
        let mut proof = self.payload_equality(&self.types[1..], &observed_values, &values[1..]);
        for (name, domain) in parameters.iter().rev() {
            ty = pi(name, domain.clone(), ty);
            proof = lam(name, domain.clone(), proof);
        }
        self.emit(&module, "PayloadOfMake", &[], ty, proof, true)
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
            .chain(payloads.iter().cloned())
            .collect::<Vec<_>>();
        let raw_box = error_layout::make(&self.types, &values);
        let boxed = own(&prefix, "Box", vec![var("v")]);
        self.emit(&prefix, "Tag", &[], word.clone(), raw_tag.clone(), false)?;
        self.emit(
            &prefix,
            "Payload",
            &input,
            payload_type.clone(),
            raw_payload.clone(),
            false,
        )?;
        self.emit(
            &prefix,
            "Box",
            &input,
            carrier.clone(),
            raw_box.clone(),
            false,
        )?;
        let made = own(&self.module, "Make", values.clone());
        self.emit(
            &prefix,
            "BoxRecipe",
            &input,
            eq(carrier.clone(), boxed.clone(), made.clone()),
            refl(carrier.clone(), raw_box),
            true,
        )?;
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
        // Rewrite the whole input through its checked recipe before applying
        // the shared payload law. This avoids reducing every case payload in
        // a single conversion while preserving the entire vector equality.
        let made_payload = own(&self.module, "PayloadVector", vec![made.clone()]);
        let payload_proof = Equality {
            ty: carrier.clone(),
            left: boxed.clone(),
            right: made,
            proof: own(&prefix, "BoxRecipe", vec![var("v")]),
        }
        .mapped(
            lam(
                "_boxed",
                carrier.clone(),
                own(&self.module, "PayloadVector", vec![var("_boxed")]),
            ),
            observed_payload.clone(),
            made_payload.clone(),
            payload_type.clone(),
        )
        .then(Equality {
            ty: payload_type.clone(),
            left: made_payload,
            right: raw_payload,
            proof: own(&self.module, "PayloadOfMake", values),
        })?;
        self.emit(
            &prefix,
            "PayloadOfBox",
            &input,
            eq(payload_type.clone(), observed_payload, payload),
            payload_proof.proof,
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
    let (builder, mut foundation) = boxing_builder(domain)?;
    foundation.certificate = builder.finish()?;
    Ok(foundation)
}
pub(super) fn boxing_builder(
    domain: &GoSourceValueDomain,
) -> Result<(Builder, GoSourceErrorBoxingFoundation)> {
    build(domain, false)
}
pub(super) fn nil_builder(
    domain: &GoSourceValueDomain,
) -> Result<(Builder, GoSourceErrorBoxingFoundation)> {
    build(domain, true)
}
fn build(
    domain: &GoSourceValueDomain,
    nil_zero_law: bool,
) -> Result<(Builder, GoSourceErrorBoxingFoundation)> {
    let (builder, layout, _) = source_error_layout::source_builder(domain)?;
    // Keep the entire existing error-value basis and all source type/zero
    // recipes. Generic layout adapters retain their own unchanged complete
    // certificates; this component needs their nil constructor and its exact
    // closed dependencies, rather than duplicating every unused adapter law.
    let nil = format!(
        "Mpk.Go.Practical.ErrorLayout.N{}.Nil",
        layout.case_type_ids().len()
    );
    let zero_law = format!(
        "Mpk.Go.Practical.ErrorLayout.N{}.FrozenZeroIsZero",
        layout.case_type_ids().len()
    );
    let roots = builder
        .certificate
        .declarations
        .iter()
        .filter_map(|d| {
            let name = &builder.certificate.name_table[d.name as usize];
            (!name.starts_with("Mpk.Go.Practical.ErrorLayout.")
                || name == &nil
                || (nil_zero_law && name == &zero_law))
            .then(|| name.clone())
        })
        .collect::<Vec<_>>();
    let mut builder = super::composition::retain_declarations(builder, &roots)?;
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
    generator.payload_make_law()?;
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
    let foundation = GoSourceErrorBoxingFoundation {
        source_module,
        module,
        cases: tags,
        certificate: vec![],
    };
    Ok((builder, foundation))
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
