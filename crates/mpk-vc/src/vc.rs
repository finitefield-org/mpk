//! Verification-condition obligation skeletons.

use serde::{Deserialize, Serialize};

use crate::expr_encode::MpkExprTerm;
use crate::gir::GirModule;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcModule {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_gir_hash: Option<String>,
    #[serde(default)]
    pub obligations: Vec<VcObligation>,
}

impl VcModule {
    pub fn empty_for_gir(gir: &GirModule) -> Self {
        Self {
            source_gir_hash: gir.gir_hash.clone(),
            obligations: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VcObligation {
    pub id: String,
    pub function_id: String,
    pub kind: VcObligationKind,
    #[serde(default)]
    pub assumptions: Vec<MpkExprTerm>,
    pub conclusion: MpkExprTerm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VcObligationKind {
    Precondition,
    Postcondition,
    RuntimeSafety,
    LoopInvariantInitial,
    LoopInvariantPreservation,
    LoopExit,
    Decreases,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr_encode::{MpkExprTerm, STD_EQ};
    use crate::gir::{import_gir_json, GIR_SCHEMA_VERSION};

    #[test]
    fn empty_vc_module_preserves_gir_hash() {
        let gir = import_gir_json(
            r#"{"schema_version":"mpk.gir.v0","packages":[],"gir_hash":"deadbeef"}"#,
        )
        .expect("GIR imports");

        let vc_module = VcModule::empty_for_gir(&gir);

        assert_eq!(vc_module.source_gir_hash.as_deref(), Some("deadbeef"));
        assert!(vc_module.obligations.is_empty());
    }

    #[test]
    fn vc_obligation_model_serializes_stably() {
        let obligation = VcObligation {
            id: "example.Identity.post0".to_owned(),
            function_id: "example.Identity".to_owned(),
            kind: VcObligationKind::Postcondition,
            assumptions: vec![MpkExprTerm::bool_literal(true)],
            conclusion: MpkExprTerm::Apply {
                function: STD_EQ.to_owned(),
                args: vec![
                    MpkExprTerm::Result { index: 0 },
                    MpkExprTerm::Var {
                        name: "value".to_owned(),
                    },
                ],
            },
        };

        let encoded = serde_json::to_string(&obligation).expect("serialize obligation");

        assert!(encoded.contains("\"kind\":\"postcondition\""));
        assert!(encoded.contains("\"function_id\":\"example.Identity\""));
        assert!(encoded.contains("\"function\":\"Std.Eq\""));
    }

    #[test]
    fn gir_schema_constant_tracks_importer() {
        assert_eq!(GIR_SCHEMA_VERSION, "mpk.gir.v0");
    }
}
