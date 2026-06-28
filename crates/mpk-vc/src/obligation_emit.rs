//! VC obligation to theorem-declaration skeleton emission.
//!
//! This module stays on the untrusted VC side of the boundary: it does not
//! invent proofs or produce a checked `.mpcert`. It deterministically turns
//! generated VC obligations into core-shaped theorem declarations that later
//! certificate-emission code can resolve into term and proof tables.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::expr_encode::MpkExprTerm;
use crate::vc::{VcModule, VcObligation, VcObligationKind};

pub const VC_CERT_SKELETON_SCHEMA_VERSION: &str = "mpk.vc.cert_skeleton.v0";
pub const VC_DECLARATION_PREFIX: &str = "VC.Obligation";
pub const STD_LOGIC_IMP: &str = "Std.Logic.Imp";

pub fn emit_theorem_obligations(
    input: &VcModule,
) -> Result<VcCertificateSkeleton, ObligationEmitError> {
    ObligationEmitter::new().emit_module(input)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ObligationEmitter;

impl ObligationEmitter {
    pub fn new() -> Self {
        Self
    }

    pub fn emit_module(
        self,
        input: &VcModule,
    ) -> Result<VcCertificateSkeleton, ObligationEmitError> {
        let mut seen_obligation_ids = BTreeSet::new();
        let mut declaration_names_by_obligation = BTreeMap::new();
        let mut theorem_declarations = Vec::with_capacity(input.obligations.len());

        for (obligation_index, obligation) in input.obligations.iter().enumerate() {
            validate_obligation_shape(obligation_index, obligation)?;
            if !seen_obligation_ids.insert(obligation.id.clone()) {
                return Err(ObligationEmitError::DuplicateObligationId {
                    obligation_id: obligation.id.clone(),
                });
            }

            let declaration_name = core_declaration_name(&obligation.id);
            if let Some(first_obligation_id) = declaration_names_by_obligation
                .insert(declaration_name.clone(), obligation.id.clone())
            {
                return Err(ObligationEmitError::DuplicateDeclarationName {
                    declaration_name,
                    first_obligation_id,
                    duplicate_obligation_id: obligation.id.clone(),
                });
            }

            theorem_declarations.push(CoreTheoremDeclarationSkeleton {
                name: declaration_name,
                obligation_id: obligation.id.clone(),
                function_id: obligation.function_id.clone(),
                obligation_kind: obligation.kind,
                ty: theorem_type_for_obligation(obligation),
            });
        }

        Ok(VcCertificateSkeleton {
            schema_version: VC_CERT_SKELETON_SCHEMA_VERSION.to_owned(),
            source_gir_hash: input.source_gir_hash.clone(),
            theorem_declarations,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcCertificateSkeleton {
    pub schema_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_gir_hash: Option<String>,
    #[serde(default)]
    pub theorem_declarations: Vec<CoreTheoremDeclarationSkeleton>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CoreTheoremDeclarationSkeleton {
    pub name: String,
    pub obligation_id: String,
    pub function_id: String,
    pub obligation_kind: VcObligationKind,
    pub ty: MpkExprTerm,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ObligationEmitError {
    EmptyObligationId {
        obligation_index: usize,
    },
    EmptyFunctionId {
        obligation_id: String,
    },
    DuplicateObligationId {
        obligation_id: String,
    },
    DuplicateDeclarationName {
        declaration_name: String,
        first_obligation_id: String,
        duplicate_obligation_id: String,
    },
}

impl fmt::Display for ObligationEmitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyObligationId { obligation_index } => write!(
                formatter,
                "VC obligation at index {obligation_index} has an empty id"
            ),
            Self::EmptyFunctionId { obligation_id } => write!(
                formatter,
                "VC obligation {obligation_id:?} has an empty function id"
            ),
            Self::DuplicateObligationId { obligation_id } => {
                write!(formatter, "duplicate VC obligation id {obligation_id:?}")
            }
            Self::DuplicateDeclarationName {
                declaration_name,
                first_obligation_id,
                duplicate_obligation_id,
            } => write!(
                formatter,
                "VC obligations {first_obligation_id:?} and {duplicate_obligation_id:?} both emit core declaration {declaration_name:?}"
            ),
        }
    }
}

impl std::error::Error for ObligationEmitError {}

pub fn theorem_type_for_obligation(obligation: &VcObligation) -> MpkExprTerm {
    obligation
        .assumptions
        .iter()
        .rev()
        .fold(obligation.conclusion.clone(), |body, assumption| {
            MpkExprTerm::apply(STD_LOGIC_IMP, [assumption.clone(), body])
        })
}

pub fn core_declaration_name(obligation_id: &str) -> String {
    let mut components = VC_DECLARATION_PREFIX
        .split('.')
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut current = String::new();

    for byte in obligation_id.bytes() {
        match byte {
            b'.' | b'/' => push_component(&mut components, &mut current),
            byte if is_component_continue(byte) => {
                if current.is_empty() && !is_component_start(byte) {
                    current.push('_');
                }
                current.push(char::from(byte));
            }
            byte if byte.is_ascii() => {
                current.push('_');
            }
            byte => {
                current.push_str("_x");
                current.push(hex_digit(byte >> 4));
                current.push(hex_digit(byte & 0x0f));
            }
        }
    }
    push_component(&mut components, &mut current);

    components.join(".")
}

fn validate_obligation_shape(
    obligation_index: usize,
    obligation: &VcObligation,
) -> Result<(), ObligationEmitError> {
    if obligation.id.is_empty() {
        return Err(ObligationEmitError::EmptyObligationId { obligation_index });
    }
    if obligation.function_id.is_empty() {
        return Err(ObligationEmitError::EmptyFunctionId {
            obligation_id: obligation.id.clone(),
        });
    }
    Ok(())
}

fn push_component(components: &mut Vec<String>, current: &mut String) {
    if !current.is_empty() {
        components.push(std::mem::take(current));
    }
}

fn is_component_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_component_continue(byte: u8) -> bool {
    is_component_start(byte) || byte.is_ascii_digit() || byte == b'\''
}

fn hex_digit(nibble: u8) -> char {
    match nibble {
        0..=9 => char::from(b'0' + nibble),
        10..=15 => char::from(b'a' + nibble - 10),
        _ => unreachable!("nibble is masked to 4 bits"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr_encode::{MpkExprTerm, STD_EQ};

    fn var(name: &str) -> MpkExprTerm {
        MpkExprTerm::Var {
            name: name.to_owned(),
        }
    }

    fn eq(lhs: MpkExprTerm, rhs: MpkExprTerm) -> MpkExprTerm {
        MpkExprTerm::apply(STD_EQ, [lhs, rhs])
    }

    fn obligation(
        id: &str,
        kind: VcObligationKind,
        assumptions: Vec<MpkExprTerm>,
        conclusion: MpkExprTerm,
    ) -> VcObligation {
        VcObligation {
            id: id.to_owned(),
            function_id: "example/pkg.Identity".to_owned(),
            kind,
            assumptions,
            conclusion,
        }
    }

    #[test]
    fn emits_theorem_declaration_skeletons_for_vc_module() {
        let assumption = var("ok");
        let conclusion = eq(var("result"), var("value"));
        let module = VcModule {
            source_gir_hash: Some("abc123".to_owned()),
            obligations: vec![obligation(
                "example/pkg.Identity.post0",
                VcObligationKind::Postcondition,
                vec![assumption.clone()],
                conclusion.clone(),
            )],
        };

        let skeleton = emit_theorem_obligations(&module).expect("obligation skeletons emit");

        assert_eq!(skeleton.schema_version, VC_CERT_SKELETON_SCHEMA_VERSION);
        assert_eq!(skeleton.source_gir_hash.as_deref(), Some("abc123"));
        assert_eq!(skeleton.theorem_declarations.len(), 1);
        assert_eq!(
            skeleton.theorem_declarations[0].name,
            "VC.Obligation.example.pkg.Identity.post0"
        );
        assert_eq!(
            skeleton.theorem_declarations[0].obligation_id,
            "example/pkg.Identity.post0"
        );
        assert_eq!(
            skeleton.theorem_declarations[0].obligation_kind,
            VcObligationKind::Postcondition
        );
        assert_eq!(
            skeleton.theorem_declarations[0].ty,
            MpkExprTerm::apply(STD_LOGIC_IMP, [assumption, conclusion])
        );
    }

    #[test]
    fn folds_multiple_assumptions_into_right_associated_implications() {
        let first = var("a");
        let second = var("b");
        let conclusion = var("c");
        let obligation = obligation(
            "example/pkg.Path.post0",
            VcObligationKind::Postcondition,
            vec![first.clone(), second.clone()],
            conclusion.clone(),
        );

        assert_eq!(
            theorem_type_for_obligation(&obligation),
            MpkExprTerm::apply(
                STD_LOGIC_IMP,
                [
                    first,
                    MpkExprTerm::apply(STD_LOGIC_IMP, [second, conclusion])
                ]
            )
        );
    }

    #[test]
    fn leaves_conclusion_as_theorem_type_without_assumptions() {
        let conclusion = var("goal");
        let obligation = obligation(
            "example/pkg.Path.post0",
            VcObligationKind::Postcondition,
            Vec::new(),
            conclusion.clone(),
        );

        assert_eq!(theorem_type_for_obligation(&obligation), conclusion);
    }

    #[test]
    fn core_declaration_names_are_dotted_ascii_components() {
        assert_eq!(
            core_declaration_name("example/pkg.Max64.then.post0"),
            "VC.Obligation.example.pkg.Max64.then.post0"
        );
        assert_eq!(
            core_declaration_name("9pkg/bad-name.δ"),
            "VC.Obligation._9pkg.bad_name._xce_xb4"
        );
    }

    #[test]
    fn rejects_duplicate_obligation_ids() {
        let first = obligation(
            "example/pkg.Identity.post0",
            VcObligationKind::Postcondition,
            Vec::new(),
            var("a"),
        );
        let second = obligation(
            "example/pkg.Identity.post0",
            VcObligationKind::Postcondition,
            Vec::new(),
            var("b"),
        );
        let module = VcModule {
            source_gir_hash: None,
            obligations: vec![first, second],
        };

        assert_eq!(
            emit_theorem_obligations(&module).unwrap_err(),
            ObligationEmitError::DuplicateObligationId {
                obligation_id: "example/pkg.Identity.post0".to_owned()
            }
        );
    }

    #[test]
    fn rejects_empty_obligation_and_function_ids() {
        let empty_obligation_id = VcModule {
            source_gir_hash: None,
            obligations: vec![obligation(
                "",
                VcObligationKind::Postcondition,
                Vec::new(),
                var("a"),
            )],
        };
        assert_eq!(
            emit_theorem_obligations(&empty_obligation_id).unwrap_err(),
            ObligationEmitError::EmptyObligationId {
                obligation_index: 0
            }
        );

        let mut empty_function_id = obligation(
            "example/pkg.Identity.post0",
            VcObligationKind::Postcondition,
            Vec::new(),
            var("a"),
        );
        empty_function_id.function_id.clear();
        let empty_function_id = VcModule {
            source_gir_hash: None,
            obligations: vec![empty_function_id],
        };
        assert_eq!(
            emit_theorem_obligations(&empty_function_id).unwrap_err(),
            ObligationEmitError::EmptyFunctionId {
                obligation_id: "example/pkg.Identity.post0".to_owned()
            }
        );
    }

    #[test]
    fn rejects_normalized_declaration_name_collisions() {
        let first = obligation(
            "example/pkg.Identity.post0",
            VcObligationKind::Postcondition,
            Vec::new(),
            var("a"),
        );
        let second = obligation(
            "example.pkg.Identity.post0",
            VcObligationKind::Postcondition,
            Vec::new(),
            var("b"),
        );
        let module = VcModule {
            source_gir_hash: None,
            obligations: vec![first, second],
        };

        assert_eq!(
            emit_theorem_obligations(&module).unwrap_err(),
            ObligationEmitError::DuplicateDeclarationName {
                declaration_name: "VC.Obligation.example.pkg.Identity.post0".to_owned(),
                first_obligation_id: "example/pkg.Identity.post0".to_owned(),
                duplicate_obligation_id: "example.pkg.Identity.post0".to_owned(),
            }
        );
    }

    #[test]
    fn certificate_skeleton_serializes_stably() {
        let module = VcModule {
            source_gir_hash: Some("abc123".to_owned()),
            obligations: vec![obligation(
                "example/pkg.Identity.post0",
                VcObligationKind::Postcondition,
                Vec::new(),
                var("goal"),
            )],
        };
        let skeleton = emit_theorem_obligations(&module).expect("obligation skeletons emit");

        let encoded = serde_json::to_string_pretty(&skeleton).expect("skeleton serializes");

        assert_eq!(
            encoded,
            r#"{
  "schema_version": "mpk.vc.cert_skeleton.v0",
  "source_gir_hash": "abc123",
  "theorem_declarations": [
    {
      "name": "VC.Obligation.example.pkg.Identity.post0",
      "obligation_id": "example/pkg.Identity.post0",
      "function_id": "example/pkg.Identity",
      "obligation_kind": "postcondition",
      "ty": {
        "kind": "var",
        "name": "goal"
      }
    }
  ]
}"#
        );
    }
}
