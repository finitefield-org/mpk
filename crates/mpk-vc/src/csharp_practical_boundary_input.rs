//! W02: byte-only, immutable input handoff. No object-taking constructor exists.
use super::*;
use crate::csharp_practical_source_artifacts::{self as a, PracticalJsonValue as J};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryInputError {
    Limit,
    Json,
    Members,
    Order,
    Value,
    Linkage,
}
/// Complete typed argument and its already imported source reconstruction.
/// A reconstruction is an ordinary VIR operation with pending source invariant
/// obligations, not permission to execute an unproved application constructor.
#[derive(Clone, Debug)]
pub struct DecodedBoundaryArgument {
    field_id: String,
    source_type_id: String,
    value: MonomorphicValue,
    reconstruction: Option<ClosedOperationSignature>,
}
impl DecodedBoundaryArgument {
    pub fn field_id(&self) -> &str {
        &self.field_id
    }
    pub fn source_type_id(&self) -> &str {
        &self.source_type_id
    }
    pub fn value(&self) -> &MonomorphicValue {
        &self.value
    }
    pub fn reconstruction(&self) -> Option<&ClosedOperationSignature> {
        self.reconstruction.as_ref()
    }
}
/// One run retains one document. Both artifact chains are built only after
/// byte parsing, full typed conversion and source reconstruction linkage.
#[derive(Clone, Debug)]
pub struct CapturedBoundaryRun {
    capture: a::BoundaryInputCapture,
    arguments: Vec<DecodedBoundaryArgument>,
    manifest: a::ValidatedPracticalArtifact,
    artifacts: a::ValidatedPracticalArtifact,
    obligations: Vec<OutcomeObligation>,
}
impl CapturedBoundaryRun {
    pub fn capture(&self) -> &a::BoundaryInputCapture {
        &self.capture
    }
    pub fn arguments(&self) -> &[DecodedBoundaryArgument] {
        &self.arguments
    }
    pub fn manifest(&self) -> &a::ValidatedPracticalArtifact {
        &self.manifest
    }
    pub fn artifacts(&self) -> &a::ValidatedPracticalArtifact {
        &self.artifacts
    }
    pub fn obligations(&self) -> &[OutcomeObligation] {
        &self.obligations
    }
}
/// Borrowed bytes are copied into the resulting immutable run. An adapter's
/// original media remain separate from the required canonical UTF-8 document.
pub struct BoundaryInputBytes<'a> {
    pub boundary_id: &'a str,
    pub provenance_id: &'a str,
    pub raw_bytes: &'a [u8],
    pub canonical_document: &'a [u8],
}
pub struct BoundaryInputEvidence<'a> {
    pub capture: &'a [u8],
    pub manifest: &'a [u8],
    pub artifacts: &'a [u8],
}
impl EmittedDataPhase {
    pub fn capture_boundary_input(
        &self,
        b: &ValidatedFoundationBundle,
        context: &a::PracticalArtifactContext,
        captures: &a::CapturedInputSet,
        input: BoundaryInputBytes<'_>,
    ) -> Result<CapturedBoundaryRun, BoundaryInputError> {
        let boundary = self
            .boundaries()
            .iter()
            .find(|p| {
                p.artifact().value().get("boundary_id").and_then(J::as_str)
                    == Some(input.boundary_id)
            })
            .ok_or(BoundaryInputError::Linkage)?;
        // This bound precedes every parse/allocation of untrusted document data.
        preflight(input.canonical_document)?;
        if u32::try_from(input.raw_bytes.len()).is_err() {
            return Err(BoundaryInputError::Limit);
        }
        let document = a::parse_canonical_practical_json(
            a::PracticalArtifactKind::BoundaryInput,
            input.canonical_document,
        )
        .map_err(|_| BoundaryInputError::Json)?;
        value_limits(&document, 0, &mut 0)?;
        let members = document
            .utf16_members()
            .ok_or(BoundaryInputError::Members)?;
        let fields = boundary.input_fields();
        // Identity/unknown checks do not depend on input member order.
        if members
            .iter()
            .any(|(name, _)| !fields.iter().any(|f| f.json_name() == name))
        {
            return Err(BoundaryInputError::Members);
        }
        let ordered = fields.iter().filter_map(|f| {
            members
                .iter()
                .find(|(n, _)| n == f.json_name())
                .map(|(n, _)| n)
        });
        if ordered.ne(members.iter().map(|(n, _)| n)) {
            return Err(BoundaryInputError::Order);
        }
        let r = self.closure().roots();
        let c = self.closure().closed();
        let mut arguments = vec![];
        let mut canonical_values = vec![];
        let mut total_cells = 1;
        for f in fields {
            let supplied = members
                .iter()
                .find(|(name, _)| name == f.json_name())
                .map(|(_, v)| *v);
            let projected = self
                .closure()
                .projections()
                .get(f.source_type_id())
                .map(String::as_str)
                .unwrap_or(f.source_type_id());
            let meta = c.metadata.get(projected);
            let presence = f.presence_binding().is_some();
            let option = meta.is_some_and(|m| template_name(&m.template_id) == Some("option"));
            let state = match supplied {
                None => BoundaryValueState::Missing,
                Some(J::Null) => BoundaryValueState::Null,
                Some(v) => BoundaryValueState::Value(
                    data_phase::decode_boundary_default(b, r, c, f.payload_type_id(), v, f.codec())
                        .map_err(|_| BoundaryInputError::Value)?,
                ),
            };
            f.check_state(b, r, c, &state)
                .map_err(|_| BoundaryInputError::Value)?;
            let value = match state {
                BoundaryValueState::Missing => match f.missing_rule() {
                    BoundaryMissingRule::FrozenDefault(value) => value.clone(),
                    BoundaryMissingRule::ExposeMissing => MonomorphicValue::BoundaryPresence {
                        type_id: projected.into(),
                        arm: BoundaryArm::Missing,
                        value: None,
                    },
                    _ => return Err(BoundaryInputError::Value),
                },
                BoundaryValueState::Null if presence => MonomorphicValue::BoundaryPresence {
                    type_id: projected.into(),
                    arm: BoundaryArm::Null,
                    value: None,
                },
                BoundaryValueState::Null if option => MonomorphicValue::Option {
                    type_id: projected.into(),
                    arm: OptionArm::None,
                    value: None,
                },
                BoundaryValueState::Null => return Err(BoundaryInputError::Value),
                BoundaryValueState::Value(value) if presence => {
                    MonomorphicValue::BoundaryPresence {
                        type_id: projected.into(),
                        arm: BoundaryArm::Value,
                        value: Some(Box::new(value)),
                    }
                }
                BoundaryValueState::Value(value) if option => MonomorphicValue::Option {
                    type_id: projected.into(),
                    arm: OptionArm::Some,
                    value: Some(Box::new(value)),
                },
                BoundaryValueState::Value(value) => value,
            };
            total_cells += validate_value_inner(b, r, c, &value, false)
                .map_err(|_| BoundaryInputError::Value)?;
            if total_cells > TOTAL_VALUE_CELLS_MAX {
                return Err(BoundaryInputError::Limit);
            }
            let reconstruction = if value.type_id() == f.source_type_id() {
                None
            } else {
                Some(
                    self.vir()
                        .binding_projections()
                        .iter()
                        .find(|p| {
                            p.source_type_id == f.source_type_id()
                                && p.semantic_type_id == value.type_id()
                        })
                        .ok_or(BoundaryInputError::Linkage)?
                        .reconstruct
                        .clone(),
                )
            };
            let canonical_value = typed_json(b, r, c, &value, f.codec())?;
            value_limits(&canonical_value, 1, &mut 0)?;
            canonical_values.push((f.json_name().to_vec(), canonical_value));
            arguments.push(DecodedBoundaryArgument {
                field_id: f.id().into(),
                source_type_id: f.source_type_id().into(),
                value,
                reconstruction,
            });
        }
        let canonical_value = J::from_utf16_members(canonical_values);
        // Canonical document bytes are bounded before parsing. The typed
        // artifact may be larger because semantic tags/defaults are explicit;
        // its own frozen artifact transport bound still applies.
        let capture = a::build_typed_boundary_input_capture(
            context,
            &boundary.artifact().artifact_ref(),
            input.provenance_id,
            input.raw_bytes,
            input.canonical_document,
            canonical_value,
        )
        .map_err(|_| BoundaryInputError::Linkage)?;
        let (manifest, artifacts) =
            self.boundary_input_artifacts(b, context, captures, capture.artifact())?;
        Ok(CapturedBoundaryRun {
            capture,
            arguments,
            manifest,
            artifacts,
            obligations: boundary.obligations().to_vec(),
        })
    }
    pub fn import_boundary_input_run(
        &self,
        b: &ValidatedFoundationBundle,
        context: &a::PracticalArtifactContext,
        captures: &a::CapturedInputSet,
        input: BoundaryInputBytes<'_>,
        evidence: BoundaryInputEvidence<'_>,
    ) -> Result<CapturedBoundaryRun, BoundaryInputError> {
        let run = self.capture_boundary_input(b, context, captures, input)?;
        for (expected, transport) in [
            (run.capture.artifact(), evidence.capture),
            (&run.manifest, evidence.manifest),
            (&run.artifacts, evidence.artifacts),
        ] {
            a::validate_expected_artifact(expected, transport)
                .map_err(|_| BoundaryInputError::Linkage)?;
        }
        Ok(run)
    }
    /// Reproduction always reparses retained bytes; caller-supplied typed objects
    /// and hash-only equivalence cannot substitute for this path.
    pub fn validate_boundary_input_run(
        &self,
        b: &ValidatedFoundationBundle,
        context: &a::PracticalArtifactContext,
        captures: &a::CapturedInputSet,
        input: BoundaryInputBytes<'_>,
        expected: &CapturedBoundaryRun,
    ) -> Result<CapturedBoundaryRun, BoundaryInputError> {
        let run = self.capture_boundary_input(b, context, captures, input)?;
        if run
            .arguments
            .iter()
            .map(|a| (&a.source_type_id, &a.value, &a.reconstruction))
            .ne(expected
                .arguments
                .iter()
                .map(|a| (&a.source_type_id, &a.value, &a.reconstruction)))
            || run.capture.raw_bytes() != expected.capture.raw_bytes()
            || run.capture.canonical_document() != expected.capture.canonical_document()
        {
            return Err(BoundaryInputError::Linkage);
        }
        for (actual, expected) in [
            (run.capture.artifact(), expected.capture.artifact()),
            (&run.manifest, &expected.manifest),
            (&run.artifacts, &expected.artifacts),
        ] {
            a::validate_expected_artifact(actual, expected.canonical_bytes())
                .map_err(|_| BoundaryInputError::Linkage)?;
        }
        Ok(run)
    }
}

// Count depth and values before serde allocates a tree. Syntax and escapes are
// still checked by the sole canonical parser. Strings used as names are not
// value cells; names and string lengths receive the exact UTF-16 check below.
pub(super) fn preflight(bytes: &[u8]) -> Result<(), BoundaryInputError> {
    if bytes.len() > 1_048_576 {
        return Err(BoundaryInputError::Limit);
    }
    let mut at = 0;
    let mut depth: usize = 0;
    let mut cells = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        match byte {
            b'"' => {
                at += 1;
                while at < bytes.len() && bytes[at] != b'"' {
                    if bytes[at] == b'\\' {
                        at += 1;
                    }
                    at += 1;
                }
                at += 1;
                if bytes.get(at) != Some(&b':') {
                    cells += 1;
                    if depth > 32 {
                        return Err(BoundaryInputError::Limit);
                    }
                }
                continue;
            }
            b'{' | b'[' => {
                cells += 1;
                if depth > 32 {
                    return Err(BoundaryInputError::Limit);
                }
                depth += 1;
            }
            b'}' | b']' => {
                depth = depth.checked_sub(1).ok_or(BoundaryInputError::Json)?;
            }
            b'-' | b'0'..=b'9' | b't' | b'f' | b'n' => {
                cells += 1;
                if depth > 32 {
                    return Err(BoundaryInputError::Limit);
                }
                while at < bytes.len() && !b",]} \t\r\n".contains(&bytes[at]) {
                    at += 1;
                }
                continue;
            }
            _ => (),
        }
        if cells > TOTAL_VALUE_CELLS_MAX * 4 {
            return Err(BoundaryInputError::Limit);
        }
        at += 1;
    }
    if cells > TOTAL_VALUE_CELLS_MAX * 4 {
        return Err(BoundaryInputError::Limit);
    }
    Ok(())
}
pub(super) fn value_limits(v: &J, depth: usize, cells: &mut u64) -> Result<(), BoundaryInputError> {
    *cells += 1;
    if depth > 32 || *cells > TOTAL_VALUE_CELLS_MAX * 4 {
        return Err(BoundaryInputError::Limit);
    }
    match v {
        J::String(s) if s.encode_utf16().count() > 16_384 => return Err(BoundaryInputError::Limit),
        J::Utf16String(s) if s.len() > 16_384 => return Err(BoundaryInputError::Limit),
        J::Array(xs) => {
            for x in xs {
                value_limits(x, depth + 1, cells)?;
            }
        }
        _ => {
            if let Some(members) = v.utf16_members() {
                for (key, value) in members {
                    if key.len() > 16_384 {
                        return Err(BoundaryInputError::Limit);
                    }
                    value_limits(value, depth + 1, cells)?;
                }
            }
        }
    }
    Ok(())
}

// Shared frozen value encoder. Callers must first validate the complete typed
// value. W02 supplies decoded input; W03 independently reparses returned output.
pub(super) fn typed_json(
    b: &ValidatedFoundationBundle,
    r: &ValidatedClosedRootSet,
    c: &ClosedInstanceSet,
    v: &MonomorphicValue,
    selected: Option<&BoundaryCodec>,
) -> Result<J, BoundaryInputError> {
    use MonomorphicValue as V;
    let child = |v: &V| typed_json(b, r, c, v, selected);
    let array = |xs: &[V]| {
        xs.iter()
            .map(child)
            .collect::<Result<Vec<_>, _>>()
            .map(J::Array)
    };
    let object = |fields| J::object(fields);
    let sum = |tag: &str, payload: Option<&V>| -> Result<J, BoundaryInputError> {
        let mut fields = vec![("tag", J::string(tag))];
        if let Some(v) = payload {
            fields.push(("payload", child(v)?));
        }
        Ok(object(fields))
    };
    Ok(match v {
        V::Bool { value, .. } => J::Bool(*value),
        V::Char { utf16, .. } => string_units(vec![*utf16]),
        V::String { utf16, .. } => string_units(utf16.clone()),
        V::Signed { type_id, value } | V::Unsigned { type_id, value }
            if !type_id.ends_with("64.v1") =>
        {
            if value.starts_with('-') {
                J::I64(value.parse().map_err(|_| BoundaryInputError::Value)?)
            } else {
                J::U64(value.parse().map_err(|_| BoundaryInputError::Value)?)
            }
        }
        V::Signed { value, .. } | V::Unsigned { value, .. } => J::string(value),
        V::Enum { carrier, .. } => J::string(carrier),
        V::Product { fields, .. } => J::Object(
            fields
                .iter()
                .map(|f| Ok((f.name.clone(), child(&f.value)?)))
                .collect::<Result<_, BoundaryInputError>>()?,
        ),
        V::Array { elements, .. }
        | V::Sequence { elements, .. }
        | V::OrderedSet { elements, .. } => array(elements)?,
        V::OrderedEntry { key, value, .. } => {
            object(vec![("key", child(key)?), ("value", child(value)?)])
        }
        V::OrderedMap { entries, .. } => J::Array(
            entries
                .iter()
                .map(|e| {
                    Ok(object(vec![
                        ("key", child(&e.key)?),
                        ("value", child(&e.value)?),
                    ]))
                })
                .collect::<Result<_, BoundaryInputError>>()?,
        ),
        V::Option { arm, value, .. } => sum(
            if *arm == OptionArm::None {
                "none"
            } else {
                "some"
            },
            value.as_deref(),
        )?,
        V::BoundaryPresence { arm, value, .. } => sum(
            match arm {
                BoundaryArm::Missing => "missing",
                BoundaryArm::Null => "null",
                BoundaryArm::Value => "value",
            },
            value.as_deref(),
        )?,
        V::TaggedSum { arm, payload, .. } => sum(arm, payload.first())?,
        V::Money {
            amount, currency, ..
        } => object(vec![
            ("amount", child(amount)?),
            ("currency", child(currency)?),
        ]),
        V::Transition {
            state,
            events,
            response,
            ..
        } => object(vec![
            ("state", child(state)?),
            ("events", array(events)?),
            ("response", child(response)?),
        ]),
        V::Unit { .. } | V::ClosedException { .. } => return Err(BoundaryInputError::Value),
        V::ParseError { arm, .. } => J::string(match arm {
            ParseErrorArm::InputBound => "input_bound",
            ParseErrorArm::Syntax => "syntax",
            ParseErrorArm::Noncanonical => "noncanonical",
            ParseErrorArm::ScalePrecision => "scale_precision",
            ParseErrorArm::Range => "range",
        }),
        _ => {
            let id = match v {
                V::F32Bits { .. } => "binary32",
                V::F64Bits { .. } => "binary64",
                V::DecimalBits { .. } => "decimal.normalized",
                V::Date { .. } => "date",
                V::Time { .. } => "time",
                V::Duration { .. } => "duration_ticks",
                V::Instant { .. } => "unix_milliseconds",
                V::Guid { .. } => "guid.n",
                _ => return Err(BoundaryInputError::Value),
            };
            let default = BoundaryCodec::new(id, v.type_id(), None, None)
                .map_err(|_| BoundaryInputError::Value)?;
            let units = selected
                .filter(|c| c.type_id() == v.type_id())
                .unwrap_or(&default)
                .format(b, r, c, v)
                .map_err(|_| BoundaryInputError::Value)?;
            string_units(units)
        }
    })
}

fn string_units(units: Vec<u16>) -> J {
    match String::from_utf16(&units) {
        Ok(text) => J::String(text),
        Err(_) => J::Utf16String(units),
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    pub(in crate::csharp_practical_vir_model) fn fixture() -> (
        ValidatedFoundationBundle,
        ValidatedClosedRootSet,
        ClosedInstanceSet,
    ) {
        let b = validate_registered_foundation_bundle(
            registered_foundation_descriptor_transport(),
            registered_foundation_definitions_transport(),
        )
        .unwrap();
        let p = |id| json!({"kind":"primitive","id":id});
        let types = [
            json!({"kind":"instance","template":"bounded_sequence","arguments":[p("i32")]}),
            json!({"kind":"instance","template":"bounded_sequence","arguments":[p("string")]}),
            json!({"kind":"instance","template":"ordered_map","arguments":[p("i32"),p("string")]}),
            json!({"kind":"instance","template":"ordered_entry","arguments":[p("i32"),p("string")]}),
            json!({"kind":"instance","template":"ordered_set","arguments":[p("i32")]}),
            json!({"kind":"instance","template":"option","arguments":[p("i32")]}),
            json!({"kind":"instance","template":"lookup","arguments":[p("i32")]}),
            json!({"kind":"instance","template":"result","arguments":[p("i32"),p("string")]}),
            json!({"kind":"instance","template":"validation","arguments":[p("i32"),p("string")]}),
            json!({"kind":"instance","template":"boundary_field","arguments":[p("i32")]}),
            json!({"kind":"instance","template":"money","arguments":[p("string")]}),
            json!({"kind":"instance","template":"transition","arguments":[p("i32"),p("string"),p("i32")]}),
        ];
        let roots = types.iter().enumerate().map(|(i,t)| json!({"origin":"semantic_binding","provenance_id":format!("input.{i}"),"type":t})).collect::<Vec<_>>();
        let r = validate_closed_root_set(
            &b,
            &canonical_closed_root_set_transport(&b, &json!(roots), &json!({})).unwrap(),
        )
        .unwrap();
        let c = derive_closed_instances(&b, &r).unwrap();
        (b, r, c)
    }
    fn decode(
        b: &ValidatedFoundationBundle,
        r: &ValidatedClosedRootSet,
        c: &ClosedInstanceSet,
        id: &str,
        bytes: &[u8],
    ) -> Result<MonomorphicValue, BoundaryInputError> {
        preflight(bytes)?;
        let json =
            a::parse_canonical_practical_json(a::PracticalArtifactKind::BoundaryInput, bytes)
                .map_err(|_| BoundaryInputError::Json)?;
        value_limits(&json, 0, &mut 0)?;
        data_phase::decode_boundary_default(b, r, c, id, &json, None)
            .map_err(|_| BoundaryInputError::Value)
    }
    #[test]
    fn csharp_03_t05_w02_scalar_utf8_and_codec_corpus() {
        let (b, r, c) = fixture();
        let rows = [
            ("bool", vec!["true", "false"], vec!["0", "\"true\""]),
            ("i8", vec!["-128", "127"], vec!["128", "-129", "\"0\""]),
            ("u8", vec!["0", "255"], vec!["256", "-1"]),
            ("i16", vec!["-32768", "32767"], vec!["32768"]),
            ("u16", vec!["65535"], vec!["65536"]),
            (
                "i32",
                vec!["-2147483648", "2147483647"],
                vec!["2147483648", "1e0", "1.0", "+1", "-0", "01"],
            ),
            ("u32", vec!["4294967295"], vec!["4294967296"]),
            (
                "i64",
                vec![r#""-9223372036854775808""#],
                vec![r#""9223372036854775808""#, "0", r#""-0""#],
            ),
            (
                "u64",
                vec![r#""18446744073709551615""#],
                vec![r#""18446744073709551616""#, r#""-1""#],
            ),
            (
                "f32",
                vec![r#""00000000""#, r#""80000000""#, r#""7fc00000""#],
                vec![r#""7FC00000""#, r#""1.0""#, "0"],
            ),
            (
                "f64",
                vec![r#""8000000000000000""#, r#""7ff8000000000000""#],
                vec![r#""0000""#],
            ),
            (
                "decimal",
                vec![r#""0""#, r#""1.25""#],
                vec![r#""1.250""#, r#""-0""#, r#""1,25""#],
            ),
            (
                "date",
                vec![r#""0001-01-01""#, r#""2000-02-29""#, r#""9999-12-31""#],
                vec![r#""1900-02-29""#, r#""2026-9-08""#],
            ),
            (
                "time",
                vec![r#""00:00:00.0000000""#, r#""23:59:59.9999999""#],
                vec![r#""24:00:00.0000000""#, r#""12:00:00""#],
            ),
            (
                "duration",
                vec![r#""-9223372036854775808""#],
                vec![r#""1.0""#, "0"],
            ),
            (
                "instant",
                vec![r#""-1""#, r#""9223372036854775807""#],
                vec![r#""2026-09-08""#, "0"],
            ),
            (
                "guid",
                vec![r#""00112233445566778899aabbccddeeff""#],
                vec![
                    r#""00112233-4455-6677-8899-aabbccddeeff""#,
                    r#""00112233445566778899AABBCCDDEEFF""#,
                ],
            ),
            ("day_of_week", vec![r#""0""#, r#""6""#], vec![r#""7""#]),
            (
                "char",
                vec![r#""\ud800""#, r#""\u0000""#, r#""a""#],
                vec![r#""😀""#, r#""ab""#],
            ),
            (
                "string",
                vec![
                    r#""😀/é""#,
                    r#""\ud800""#,
                    r#""\udfff""#,
                    r#""\u0001sdc00""#,
                    r#""\"\\\u000a""#,
                ],
                vec![
                    r#""\uD800""#,
                    r#""\n""#,
                    r#""\/""#,
                    r#""\ud83d\ude00""#,
                    r#""\u0061""#,
                ],
            ),
        ];
        for (token, good, bad) in rows {
            let id = format!("mpk.csharp.value.{token}.v1");
            for text in good {
                let first = decode(&b, &r, &c, &id, text.as_bytes())
                    .unwrap_or_else(|e| panic!("{token}/{text}: {e:?}"));
                let second = decode(&b, &r, &c, &id, text.as_bytes()).unwrap();
                assert_eq!(first, second);
                assert_eq!(
                    a::canonical_practical_json_bytes(
                        &typed_json(&b, &r, &c, &first, None).unwrap()
                    )
                    .unwrap(),
                    text.as_bytes(),
                    "{token}/{text}"
                );
            }
            for text in bad {
                assert!(
                    decode(&b, &r, &c, &id, text.as_bytes()).is_err(),
                    "{token}/{text}"
                );
            }
        }
        for bytes in [
            b"\xff".as_slice(),
            b"\"\xc0\xaf\"",
            b"\"\xed\xa0\x80\"",
            b"\xef\xbb\xbf\"x\"",
        ] {
            assert!(decode(&b, &r, &c, "mpk.csharp.value.string.v1", bytes).is_err());
        }
    }
    #[test]
    fn csharp_03_t05_w02_entry_arrays_and_active_tags() {
        let (b, r, c) = fixture();
        for (role, good, bad) in [
            (
                "ordered_map",
                vec![
                    r#"[]"#,
                    r#"[{"key":1,"value":"a"},{"key":2,"value":"\ud800"}]"#,
                ],
                vec![
                    r#"{"1":"a"}"#,
                    r#"[{"key":1,"value":"a"},{"key":1,"value":"b"}]"#,
                    r#"[{"key":2,"value":"a"},{"key":1,"value":"b"}]"#,
                    r#"[{"value":"a","key":1}]"#,
                    r#"[{"key":1,"value":"a","other":0}]"#,
                ],
            ),
            ("ordered_set", vec!["[]", "[1,2]"], vec!["[1,1]", "[2,1]"]),
            (
                "option",
                vec![r#"{"tag":"none"}"#, r#"{"tag":"some","payload":7}"#],
                vec![
                    r#"{"tag":"none","payload":7}"#,
                    r#"{"tag":"some"}"#,
                    r#"{"payload":7,"tag":"some"}"#,
                ],
            ),
            (
                "lookup",
                vec![r#"{"tag":"missing_key"}"#, r#"{"tag":"found","payload":7}"#],
                vec![r#"{"tag":"null"}"#],
            ),
            (
                "result",
                vec![
                    r#"{"tag":"ok","payload":7}"#,
                    r#"{"tag":"error","payload":"bad"}"#,
                ],
                vec![
                    r#"{"tag":"error","payload":7}"#,
                    r#"{"tag":"ok","payload":7,"unused":0}"#,
                ],
            ),
            (
                "validation",
                vec![
                    r#"{"tag":"valid","payload":7}"#,
                    r#"{"tag":"invalid","payload":["bad"]}"#,
                ],
                vec![r#"{"tag":"invalid","payload":[]}"#],
            ),
            (
                "boundary_field",
                vec![
                    r#"{"tag":"missing"}"#,
                    r#"{"tag":"null"}"#,
                    r#"{"tag":"value","payload":7}"#,
                ],
                vec![r#"{"tag":"other"}"#, r#"{"tag":"null","payload":7}"#],
            ),
            (
                "money",
                vec![r#"{"amount":"1.25","currency":"USD"}"#],
                vec![r#"{"amount":"1.250","currency":"USD"}"#],
            ),
            (
                "transition",
                vec![r#"{"state":1,"events":["e"],"response":2}"#],
                vec![r#"{"state":1,"response":2,"events":["e"]}"#],
            ),
        ] {
            let id = c
                .metadata
                .iter()
                .find(|(_, m)| template_name(&m.template_id) == Some(role))
                .unwrap()
                .0;
            for text in good {
                let value = decode(&b, &r, &c, id, text.as_bytes())
                    .unwrap_or_else(|e| panic!("{role}/{text}: {e:?}"));
                assert_eq!(
                    a::canonical_practical_json_bytes(
                        &typed_json(&b, &r, &c, &value, None).unwrap()
                    )
                    .unwrap(),
                    text.as_bytes()
                );
            }
            for text in bad {
                assert!(
                    decode(&b, &r, &c, id, text.as_bytes()).is_err(),
                    "{role}/{text}"
                );
            }
        }
    }
    #[test]
    fn csharp_03_t05_w02_resource_boundaries_and_surrogate_member_identity() {
        let (b, r, c) = fixture();
        for n in [16_383, 16_384, 16_385] {
            let text = format!("\"{}\"", "x".repeat(n));
            assert_eq!(
                decode(&b, &r, &c, "mpk.csharp.value.string.v1", text.as_bytes()).is_ok(),
                n <= 16_384
            );
        }
        for n in [31, 32, 33] {
            let bytes = format!("{}0{}", "[".repeat(n), "]".repeat(n));
            assert_eq!(preflight(bytes.as_bytes()).is_ok(), n <= 32);
        }
        for n in [262_143, 262_144, 262_145] {
            let bytes = format!("[{}]", vec!["0"; n - 1].join(","));
            assert_eq!(preflight(bytes.as_bytes()).is_ok(), n <= 262_144);
        }
        for n in [1_048_575, 1_048_576, 1_048_577] {
            let bytes = vec![b' '; n];
            assert_eq!(preflight(&bytes).is_ok(), n <= 1_048_576);
        }
        let sequence = c
            .metadata
            .iter()
            .find(|(_, m)| {
                template_name(&m.template_id) == Some("bounded_sequence")
                    && m.argument_ids == ["mpk.csharp.value.i32.v1"]
            })
            .unwrap()
            .0;
        for n in [4095, 4096, 4097] {
            let text = format!("[{}]", vec!["0"; n].join(","));
            assert_eq!(
                decode(&b, &r, &c, sequence, text.as_bytes()).is_ok(),
                n <= 4096
            );
        }
        for raw in [
            r#"{"\ud800":0}"#,
            r#"{"\u0000sd800":0,"\ud800":1,"\u0000n":2}"#,
            r#"{"\u0000sd800":0}"#,
            r#"{"\u0001sdc00":0,"\udc00":1}"#,
            r#"{"😀":0,"\ud800":1}"#,
        ] {
            let j = a::parse_canonical_practical_json(
                a::PracticalArtifactKind::BoundaryInput,
                raw.as_bytes(),
            )
            .unwrap();
            assert_eq!(
                a::canonical_practical_json_bytes(&j).unwrap(),
                raw.as_bytes()
            );
        }
        for raw in [
            r#"{"\ud800":0,"\ud800":1}"#,
            r#"{"\uD800":0}"#,
            r#"{"\ud83d\ude00":0}"#,
        ] {
            assert!(a::parse_canonical_practical_json(
                a::PracticalArtifactKind::BoundaryInput,
                raw.as_bytes()
            )
            .is_err());
        }
    }
}
