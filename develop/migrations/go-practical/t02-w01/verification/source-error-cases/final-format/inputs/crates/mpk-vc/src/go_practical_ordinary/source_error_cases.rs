//! Complete source-U case comparisons for one original boxed source type.
//! All source constructors/laws remain; only unused general math is packaged
//! separately. No dispatch body, arbitrary-input validity or context is assumed.
use super::syntax::*;
use super::{
    composition, error_layout, source_error_operations, word, Builder, GoOrdinaryError, Result,
    MAX_BYTES,
};
use crate::go_practical_type_keys::GoSourceValueDomain;

pub struct GoSourceErrorCaseCatalogue {
    builder: Builder,
    source_module: String,
    boxing_module: String,
    operations_module: String,
    cases: Vec<(String, u64)>,
    prior_operations: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceErrorCaseRow {
    source_module: String,
    boxing_module: String,
    operations_module: String,
    module: String,
    boxed_type: String,
    cases: Vec<(String, u64)>,
    certificate: Vec<u8>,
}
impl GoSourceErrorCaseRow {
    pub fn source_layout_module_name(&self) -> &str {
        &self.source_module
    }
    pub fn boxing_module_name(&self) -> &str {
        &self.boxing_module
    }
    pub fn operations_module_name(&self) -> &str {
        &self.operations_module
    }
    pub fn module_name(&self) -> &str {
        &self.module
    }
    pub fn boxed_source_type_id(&self) -> &str {
        &self.boxed_type
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
fn source(module: &str, id: &str) -> Expr {
    constant(&format!("{module}.T{id}.Carrier"))
}
fn truth(value: Expr, expected: bool) -> Expr {
    eq(boolean(), value, bit(expected))
}

fn comparison_proof(left: Expr, right: Expr, a: u64, b: u64) -> Result<Expr> {
    if a == b {
        return Ok(call(
            "Mpk.Go.Practical.ErrorValue.TagEqualReflexive",
            vec![left],
        ));
    }
    let position = (a ^ b).trailing_zeros() as usize;
    let av = bit(a & (1u64 << position) != 0);
    let bv = bit(b & (1u64 << position) != 0);
    let at = word::at(left.clone(), 64, position)?;
    let bt = word::at(right.clone(), 64, position)?;
    let start = bc("Equal", vec![at.clone(), bt.clone()]);
    let middle = bc("Equal", vec![av.clone(), bt.clone()]);
    let end = bc("Equal", vec![av.clone(), bv.clone()]);
    let different = Equality::boolean(at, av.clone(), refl(boolean(), av.clone()))
        .mapped(
            lam("_a", boolean(), bc("Equal", vec![var("_a"), bt.clone()])),
            start,
            middle.clone(),
            boolean(),
        )
        .then(
            Equality::boolean(bt, bv.clone(), refl(boolean(), bv)).mapped(
                lam("_b", boolean(), bc("Equal", vec![av, var("_b")])),
                middle,
                end.clone(),
                boolean(),
            ),
        )?
        .then(Equality::boolean(
            end,
            bit(false),
            refl(boolean(), bit(false)),
        ))?;
    Ok(call(
        &format!("Mpk.Go.Practical.ErrorValue.TagDifferentAt{position}"),
        vec![left, right, different.proof],
    ))
}

impl GoSourceErrorCaseCatalogue {
    pub fn source_layout_module_name(&self) -> &str {
        &self.source_module
    }
    pub fn boxing_module_name(&self) -> &str {
        &self.boxing_module
    }
    pub fn operations_module_name(&self) -> &str {
        &self.operations_module
    }
    pub fn case_tags(&self) -> &[(String, u64)] {
        &self.cases
    }
    pub fn prior_operations_certificate_bytes(&self) -> &[u8] {
        &self.prior_operations
    }

    /// A row selects one boxed source type, while testing every target in the
    /// same complete U. Selection never changes carriers, payloads or U.
    pub fn generate_row(&self, boxed_type: &str) -> Result<GoSourceErrorCaseRow> {
        let (_, ordinal) = self
            .cases
            .iter()
            .find(|(id, _)| id == boxed_type)
            .ok_or(GoOrdinaryError::Linkage)?;
        let mut builder = self.builder.clone();
        let module = format!("{}.Cases.C{boxed_type}", self.operations_module);
        let types = std::iter::once(word::word_type(64)?)
            .chain(
                self.cases
                    .iter()
                    .map(|(id, _)| source(&self.source_module, id)),
            )
            .collect::<Vec<_>>();
        let carrier = error_layout::product(&types);
        let tag = var("tag");
        let e = var("e");
        let same = eq(
            word::word_type(64)?,
            member(&self.boxing_module, "Tag", vec![e.clone()]),
            tag.clone(),
        );
        emit(
            &mut builder,
            &format!("{module}.OwnTagMatches"),
            &[
                ("e", carrier),
                ("tag", word::word_type(64)?),
                ("same_tag", same),
            ],
            truth(
                member(
                    &self.operations_module,
                    "HasCase",
                    vec![tag.clone(), e.clone()],
                ),
                true,
            ),
            member(
                &self.boxing_module,
                "SameTagHasOwnCase",
                vec![e, tag, var("same_tag")],
            ),
            true,
        )?;
        let prefix = format!("{}.C{boxed_type}", self.boxing_module);
        let boxed = member(&prefix, "Box", vec![var("v")]);
        let boxed_tag = member(&prefix, "Tag", vec![]);
        let same_tag = member(&prefix, "TagOfBox", vec![var("v")]);
        for (id, target_ordinal) in &self.cases {
            let target = member(&format!("{}.C{id}", self.boxing_module), "Tag", vec![]);
            let matches = id == boxed_type;
            let compared = word::wc(64, "Equal", vec![target.clone(), boxed_tag.clone()]);
            emit(
                &mut builder,
                &format!("{module}.TagEqual.C{id}"),
                &[],
                truth(compared, matches),
                comparison_proof(target.clone(), boxed_tag.clone(), *target_ordinal, *ordinal)?,
                true,
            )?;
            let proof = if matches {
                member(
                    &module,
                    "OwnTagMatches",
                    vec![boxed.clone(), boxed_tag.clone(), same_tag.clone()],
                )
            } else {
                member(
                    &self.operations_module,
                    "DifferentTagHasNoCase",
                    vec![
                        target.clone(),
                        boxed.clone(),
                        boxed_tag.clone(),
                        same_tag.clone(),
                        member(&module, &format!("TagEqual.C{id}"), vec![]),
                    ],
                )
            };
            emit(
                &mut builder,
                &format!("{module}.Matches.C{id}"),
                &[("v", source(&self.source_module, boxed_type))],
                truth(
                    member(
                        &self.operations_module,
                        "HasCase",
                        vec![target, boxed.clone()],
                    ),
                    matches,
                ),
                proof,
                true,
            )?;
        }
        builder.certificate.module = module.clone();
        Ok(GoSourceErrorCaseRow {
            source_module: self.source_module.clone(),
            boxing_module: self.boxing_module.clone(),
            operations_module: self.operations_module.clone(),
            module,
            boxed_type: boxed_type.into(),
            cases: self.cases.clone(),
            certificate: builder.finish()?,
        })
    }
    pub fn import_row(&self, candidate: &[u8], boxed_type: &str) -> Result<GoSourceErrorCaseRow> {
        if candidate.len() > MAX_BYTES {
            return Err(GoOrdinaryError::Limit);
        }
        let expected = self.generate_row(boxed_type)?;
        if candidate != expected.certificate_bytes() {
            return Err(GoOrdinaryError::Linkage);
        }
        Ok(expected)
    }
}

pub fn generate_go_practical_source_error_case_catalogue(
    domain: &GoSourceValueDomain,
) -> Result<GoSourceErrorCaseCatalogue> {
    let (builder, operations) = source_error_operations::operations_builder(domain)?;
    let prior_operations = builder.clone().finish()?;
    let source_module = operations.source_layout_module_name().to_string();
    // Every original source-specific definition AND theorem is a root. This
    // closes all needed math dependencies, without copying unrelated general
    // arithmetic into each row or discarding any source case or payload law.
    let roots = builder
        .certificate
        .declarations
        .iter()
        .filter_map(|d| {
            let n = &builder.certificate.name_table[d.name as usize];
            n.starts_with(&(source_module.clone() + "."))
                .then(|| n.clone())
        })
        .collect::<Vec<_>>();
    let builder = composition::retain_declarations(builder, &roots)?;
    Ok(GoSourceErrorCaseCatalogue {
        builder,
        source_module,
        boxing_module: operations.boxing_module_name().into(),
        operations_module: operations.module_name().into(),
        cases: operations.case_tags().to_vec(),
        prior_operations,
    })
}
