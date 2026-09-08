//! W03: encode the complete returned source value, independently decode it,
//! then retain both boundary receipts in the original source artifact chain.
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryOutputError {
    Linkage,
    Value,
    Limit,
    Json,
    Equality,
    Bytes,
}
/// This evidence is reproduction data, not evidence that an application ran or
/// that its pending postconditions/projection obligations have been discharged.
#[derive(Clone, Debug)]
pub struct CapturedBoundaryOutputRun {
    input: CapturedBoundaryRun,
    returned: MonomorphicValue,
    reparsed: MonomorphicValue,
    capture: a::BoundaryOutputCapture,
    manifest: a::ValidatedPracticalArtifact,
    artifacts: a::ValidatedPracticalArtifact,
}
impl CapturedBoundaryOutputRun {
    pub fn input(&self) -> &CapturedBoundaryRun {
        &self.input
    }
    pub fn returned_value(&self) -> &MonomorphicValue {
        &self.returned
    }
    pub fn reparsed_value(&self) -> &MonomorphicValue {
        &self.reparsed
    }
    pub fn capture(&self) -> &a::BoundaryOutputCapture {
        &self.capture
    }
    pub fn manifest(&self) -> &a::ValidatedPracticalArtifact {
        &self.manifest
    }
    pub fn artifacts(&self) -> &a::ValidatedPracticalArtifact {
        &self.artifacts
    }
}
pub struct BoundaryOutputEvidence<'a> {
    pub canonical_document: &'a [u8],
    pub capture: &'a [u8],
    pub manifest: &'a [u8],
    pub artifacts: &'a [u8],
}
impl EmittedDataPhase {
    pub fn capture_boundary_output(
        &self,
        b: &ValidatedFoundationBundle,
        context: &a::PracticalArtifactContext,
        captures: &a::CapturedInputSet,
        input: &CapturedBoundaryRun,
        returned: &MonomorphicValue,
    ) -> Result<CapturedBoundaryOutputRun, BoundaryOutputError> {
        let boundary = self
            .boundaries()
            .iter()
            .find(|boundary| {
                input
                    .capture()
                    .artifact()
                    .value()
                    .get("boundary_contract_sha256")
                    .and_then(J::as_str)
                    == Some(boundary.artifact().hash())
            })
            .ok_or(BoundaryOutputError::Linkage)?;
        // Even an immutable run from another emission/context must not be spliced.
        let input = self
            .validate_boundary_input_run(
                b,
                context,
                captures,
                BoundaryInputBytes {
                    boundary_id: boundary
                        .artifact()
                        .value()
                        .get("boundary_id")
                        .and_then(J::as_str)
                        .ok_or(BoundaryOutputError::Linkage)?,
                    provenance_id: input
                        .capture()
                        .artifact()
                        .value()
                        .get("raw_input")
                        .and_then(|v| v.get("provenance_id"))
                        .and_then(J::as_str)
                        .ok_or(BoundaryOutputError::Linkage)?,
                    raw_bytes: input.capture().raw_bytes(),
                    canonical_document: input.capture().canonical_document(),
                },
                input,
            )
            .map_err(|_| BoundaryOutputError::Linkage)?;
        let r = self.closure().roots();
        let c = self.closure().closed();
        let (document, reparsed) = match boundary.output_fields() {
            [] if matches!(returned, MonomorphicValue::Unit { type_id } if type_id == "mpk.csharp.value.unit.v1") => {
                (J::object(Vec::<(&str, J)>::new()), returned.clone())
            }
            [field] if returned.type_id() == field.source_type_id() => {
                // Preserve all source storage, including unmapped/inactive fields.
                // Binding project/reconstruct operations remain owned by the VIR.
                let (value, reparsed) = encode_returned_value(b, r, c, returned, field.codec())?;
                (
                    J::from_utf16_members(vec![(field.json_name().to_vec(), value)]),
                    reparsed,
                )
            }
            _ => return Err(BoundaryOutputError::Value),
        };
        boundary_input::value_limits(&document, 0, &mut 0).map_err(input_error)?;
        let bytes =
            a::canonical_practical_json_bytes(&document).map_err(|_| BoundaryOutputError::Json)?;
        boundary_input::preflight(&bytes).map_err(input_error)?;
        // Reparse the entire document, including the declared field name/order.
        let parsed =
            a::parse_canonical_practical_json(a::PracticalArtifactKind::BoundaryOutput, &bytes)
                .map_err(|_| BoundaryOutputError::Json)?;
        if parsed != document {
            return Err(BoundaryOutputError::Equality);
        }
        // Nothing publishable exists before the complete typed/byte checks above.
        let capture = a::build_boundary_output_capture(
            context,
            &boundary.artifact().artifact_ref(),
            document,
        )
        .map_err(|_| BoundaryOutputError::Linkage)?;
        let (manifest, artifacts) = self
            .boundary_run_artifacts(
                b,
                context,
                captures,
                input.capture().artifact(),
                Some(capture.artifact()),
            )
            .map_err(|_| BoundaryOutputError::Linkage)?;
        Ok(CapturedBoundaryOutputRun {
            input,
            returned: returned.clone(),
            reparsed,
            capture,
            manifest,
            artifacts,
        })
    }
    pub fn import_boundary_output_run(
        &self,
        b: &ValidatedFoundationBundle,
        context: &a::PracticalArtifactContext,
        captures: &a::CapturedInputSet,
        input: &CapturedBoundaryRun,
        returned: &MonomorphicValue,
        evidence: BoundaryOutputEvidence<'_>,
    ) -> Result<CapturedBoundaryOutputRun, BoundaryOutputError> {
        let run = self.capture_boundary_output(b, context, captures, input, returned)?;
        if run.capture.canonical_document() != evidence.canonical_document {
            return Err(BoundaryOutputError::Bytes);
        }
        for (expected, bytes) in [
            (run.capture.artifact(), evidence.capture),
            (&run.manifest, evidence.manifest),
            (&run.artifacts, evidence.artifacts),
        ] {
            a::validate_expected_artifact(expected, bytes)
                .map_err(|_| BoundaryOutputError::Linkage)?;
        }
        Ok(run)
    }
}
fn input_error(e: BoundaryInputError) -> BoundaryOutputError {
    match e {
        BoundaryInputError::Limit => BoundaryOutputError::Limit,
        _ => BoundaryOutputError::Json,
    }
}
/// Source arrays and their immutable sequence snapshots have the same declared
/// value representation. All fields, tags, order and floating-point bits remain
/// observable; only the frozen decimal value equivalence ignores decimal bits.
fn source_equal(original: &MonomorphicValue, reparsed: &MonomorphicValue) -> bool {
    fn arrays(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(fields) => {
                if fields.get("kind").and_then(serde_json::Value::as_str) == Some("array") {
                    fields.insert("kind".into(), json!("sequence"));
                }
                for value in fields.values_mut() {
                    arrays(value);
                }
            }
            serde_json::Value::Array(xs) => {
                for value in xs {
                    arrays(value);
                }
            }
            _ => (),
        }
    }
    let normalize = |v: &MonomorphicValue| {
        let mut json = serde_json::to_value(v).expect("typed value");
        arrays(&mut json);
        serde_json::from_value(json).expect("same typed sequence shape")
    };
    domain::source_observations_equal(&normalize(original), &normalize(reparsed))
}
fn encode_returned_value(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    returned: &MonomorphicValue,
    codec: Option<&BoundaryCodec>,
) -> Result<(J, MonomorphicValue), BoundaryOutputError> {
    let cells =
        validate_value_inner(b, r, c, returned, false).map_err(|_| BoundaryOutputError::Value)?;
    if cells >= TOTAL_VALUE_CELLS_MAX {
        return Err(BoundaryOutputError::Limit);
    } // root object counts too
    let value = boundary_input::typed_json(b, r, c, returned, codec)
        .map_err(|_| BoundaryOutputError::Value)?;
    boundary_input::value_limits(&value, 1, &mut 0).map_err(input_error)?;
    let bytes = a::canonical_practical_json_bytes(&value).map_err(|_| BoundaryOutputError::Json)?;
    boundary_input::preflight(&bytes).map_err(input_error)?;
    let parsed =
        a::parse_canonical_practical_json(a::PracticalArtifactKind::BoundaryOutput, &bytes)
            .map_err(|_| BoundaryOutputError::Json)?;
    let reparsed = data_phase::decode_boundary_default(b, r, c, returned.type_id(), &parsed, codec)
        .map_err(|_| BoundaryOutputError::Value)?;
    validate_monomorphic_value(b, r, c, &reparsed).map_err(|_| BoundaryOutputError::Value)?;
    if !source_equal(returned, &reparsed) {
        return Err(BoundaryOutputError::Equality);
    }
    Ok((value, reparsed))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csharp_03_t05_w03_frozen_output_goldens_and_typed_reparse() {
        let (b, r, c) = boundary_input::tests::fixture();
        let rows = [
            ("bool", "false"),
            ("i8", "-128"),
            ("u8", "255"),
            ("i16", "-32768"),
            ("u16", "65535"),
            ("i32", "-2147483648"),
            ("u32", "4294967295"),
            ("i64", r#""-9223372036854775808""#),
            ("u64", r#""18446744073709551615""#),
            ("f32", r#""80000000""#),
            ("f32", r#""7fc00000""#),
            ("f64", r#""7ff8000000000000""#),
            ("f64", r#""8000000000000000""#),
            ("decimal", r#""1.25""#),
            ("date", r#""2000-02-29""#),
            ("time", r#""23:59:59.9999999""#),
            ("duration", r#""-1""#),
            ("instant", r#""9223372036854775807""#),
            ("guid", r#""00112233445566778899aabbccddeeff""#),
            ("char", r#""\ud800""#),
            ("string", r#""😀\ud800\u000a\"\\/""#),
            ("day_of_week", r#""6""#),
            ("parse_error", r#""noncanonical""#),
            ("bounded_sequence", "[1,2]"),
            ("ordered_set", "[1,2]"),
            ("ordered_entry", r#"{"key":1,"value":"a"}"#),
            (
                "ordered_map",
                r#"[{"key":1,"value":"a"},{"key":2,"value":"\ud800"}]"#,
            ),
            ("option", r#"{"tag":"none"}"#),
            ("option", r#"{"tag":"some","payload":7}"#),
            ("lookup", r#"{"tag":"missing_key"}"#),
            ("lookup", r#"{"tag":"found","payload":7}"#),
            ("result", r#"{"tag":"ok","payload":7}"#),
            ("result", r#"{"tag":"error","payload":"bad"}"#),
            ("validation", r#"{"tag":"valid","payload":7}"#),
            ("validation", r#"{"tag":"invalid","payload":["bad"]}"#),
            ("boundary_field", r#"{"tag":"missing"}"#),
            ("boundary_field", r#"{"tag":"null"}"#),
            ("boundary_field", r#"{"tag":"value","payload":7}"#),
            ("money", r#"{"amount":"1.25","currency":"USD"}"#),
            ("transition", r#"{"state":1,"events":["e"],"response":2}"#),
        ];
        for (role, golden) in rows {
            let id = c
                .metadata
                .iter()
                .find(|(_, m)| {
                    template_name(&m.template_id) == Some(role)
                        && (role != "bounded_sequence"
                            || m.argument_ids == ["mpk.csharp.value.i32.v1"])
                })
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| format!("mpk.csharp.value.{role}.v1"));
            let json = a::parse_canonical_practical_json(
                a::PracticalArtifactKind::BoundaryOutput,
                golden.as_bytes(),
            )
            .unwrap();
            let original =
                data_phase::decode_boundary_default(&b, &r, &c, &id, &json, None).unwrap();
            let (encoded, reparsed) = encode_returned_value(&b, &r, &c, &original, None)
                .unwrap_or_else(|e| panic!("{role}: {e:?}"));
            assert_eq!(
                a::canonical_practical_json_bytes(&encoded).unwrap(),
                golden.as_bytes(),
                "{role}"
            );
            assert_eq!(original, reparsed, "{role}");
        }
        let decimal = MonomorphicValue::DecimalBits {
            type_id: "mpk.csharp.value.decimal.v1".into(),
            negative: false,
            coefficient: "1250".into(),
            scale: 3,
        };
        let (encoded, reparsed) = encode_returned_value(&b, &r, &c, &decimal, None).unwrap();
        assert_eq!(encoded, J::string("1.25"));
        assert_ne!(decimal, reparsed); // frozen decimal value equality ignores scale
        assert!(source_equal(&decimal, &reparsed));
        let fixed = BoundaryCodec::new("decimal.fixed", decimal.type_id(), Some(2), Some("ToEven"))
            .unwrap();
        assert_eq!(
            encode_returned_value(&b, &r, &c, &decimal, Some(&fixed))
                .unwrap()
                .0,
            J::string("1.25")
        );
        let lossy = BoundaryCodec::new("decimal.fixed", decimal.type_id(), Some(1), Some("ToEven"))
            .unwrap();
        assert!(encode_returned_value(&b, &r, &c, &decimal, Some(&lossy)).is_err());
        let guid = BoundaryCodec::new("guid.d", "mpk.csharp.value.guid.v1", None, None).unwrap();
        let original = guid
            .parse(
                &"00112233-4455-6677-8899-aabbccddeeff"
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )
            .unwrap();
        assert_eq!(
            encode_returned_value(&b, &r, &c, &original, Some(&guid))
                .unwrap()
                .0,
            J::string("00112233-4455-6677-8899-aabbccddeeff")
        );
    }
    #[test]
    fn csharp_03_t05_w03_output_limits_and_hostile_typed_values() {
        let (b, r, c) = boundary_input::tests::fixture();
        let id = c
            .metadata
            .iter()
            .find(|(_, m)| {
                template_name(&m.template_id) == Some("bounded_sequence")
                    && m.argument_ids == ["mpk.csharp.value.i32.v1"]
            })
            .unwrap()
            .0;
        for n in [4095, 4096, 4097] {
            let v = MonomorphicValue::Array {
                type_id: id.clone(),
                elements: vec![
                    MonomorphicValue::Signed {
                        type_id: "mpk.csharp.value.i32.v1".into(),
                        value: "0".into()
                    };
                    n
                ],
            };
            assert_eq!(
                encode_returned_value(&b, &r, &c, &v, None).is_ok(),
                n <= 4096
            );
        }
        for n in [16383, 16384, 16385] {
            let v = MonomorphicValue::String {
                type_id: "mpk.csharp.value.string.v1".into(),
                utf16: vec![0xd800; n],
            };
            assert_eq!(
                encode_returned_value(&b, &r, &c, &v, None).is_ok(),
                n <= 16384
            );
        }
        for value in ["00", "2147483648", "-0"] {
            let v = MonomorphicValue::Signed {
                type_id: "mpk.csharp.value.i32.v1".into(),
                value: value.into(),
            };
            assert!(encode_returned_value(&b, &r, &c, &v, None).is_err());
        }
        let id = c
            .metadata
            .iter()
            .find(|(_, m)| template_name(&m.template_id) == Some("option"))
            .unwrap()
            .0;
        let v = MonomorphicValue::Option {
            type_id: id.clone(),
            arm: OptionArm::None,
            value: Some(Box::new(MonomorphicValue::Signed {
                type_id: "mpk.csharp.value.i32.v1".into(),
                value: "7".into(),
            })),
        };
        assert!(encode_returned_value(&b, &r, &c, &v, None).is_err());
    }
}
