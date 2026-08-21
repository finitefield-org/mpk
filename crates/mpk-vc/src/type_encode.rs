//! VIR and legacy GIR type to unresolved MPK type-term encoding.
//!
//! The production mapping is language-neutral and emits stable
//! `Std.Program.Base.*` names instead of certificate global ids. The GIR entry
//! point is retained as an internal migration wrapper and emits the same names.

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::gir::{GirFieldType, GirType, GirTypeKind};
use crate::semantic_profile::{
    validate_semantic_parameters, SemanticParameters, SemanticProfile, SemanticProfileError,
};
use crate::vir::{VirStructDecl, VirType};

pub const STD_PROGRAM_BASE_BOOL: &str = "Std.Program.Base.Bool";
pub const STD_PROGRAM_BASE_INT8: &str = "Std.Program.Base.Int8";
pub const STD_PROGRAM_BASE_INT16: &str = "Std.Program.Base.Int16";
pub const STD_PROGRAM_BASE_INT32: &str = "Std.Program.Base.Int32";
pub const STD_PROGRAM_BASE_INT64: &str = "Std.Program.Base.Int64";
pub const STD_PROGRAM_BASE_UINT8: &str = "Std.Program.Base.Uint8";
pub const STD_PROGRAM_BASE_UINT16: &str = "Std.Program.Base.Uint16";
pub const STD_PROGRAM_BASE_UINT32: &str = "Std.Program.Base.Uint32";
pub const STD_PROGRAM_BASE_UINT64: &str = "Std.Program.Base.Uint64";
pub const STD_PROGRAM_BASE_ARRAY_LENGTH: &str = "Std.Program.Base.Array.Length";
pub const STD_PROGRAM_BASE_ARRAY: &str = "Std.Program.Base.Array";
pub const STD_PROGRAM_BASE_STRUCT_SHAPE: &str = "Std.Program.Base.Struct.Shape";
pub const STD_PROGRAM_BASE_STRUCT_FIELD: &str = "Std.Program.Base.Struct.Field";
pub const STD_PROGRAM_BASE_STRUCT_FIELD_TYPE: &str = "Std.Program.Base.Struct.FieldType";
pub const STD_PROGRAM_BASE_STRUCT_VALUE: &str = "Std.Program.Base.Struct.Value";

// Keep the legacy public identifiers source-compatible during the GIR-to-VIR
// migration. Their values intentionally point at the language-neutral
// namespace, so retaining these Rust identifiers cannot reactivate the
// retired language-specific certificate namespace.
#[doc(hidden)]
pub const STD_GO_BASE_BOOL: &str = STD_PROGRAM_BASE_BOOL;
#[doc(hidden)]
pub const STD_GO_BASE_INT8: &str = STD_PROGRAM_BASE_INT8;
#[doc(hidden)]
pub const STD_GO_BASE_INT16: &str = STD_PROGRAM_BASE_INT16;
#[doc(hidden)]
pub const STD_GO_BASE_INT32: &str = STD_PROGRAM_BASE_INT32;
#[doc(hidden)]
pub const STD_GO_BASE_INT64: &str = STD_PROGRAM_BASE_INT64;
#[doc(hidden)]
pub const STD_GO_BASE_UINT8: &str = STD_PROGRAM_BASE_UINT8;
#[doc(hidden)]
pub const STD_GO_BASE_UINT16: &str = STD_PROGRAM_BASE_UINT16;
#[doc(hidden)]
pub const STD_GO_BASE_UINT32: &str = STD_PROGRAM_BASE_UINT32;
#[doc(hidden)]
pub const STD_GO_BASE_UINT64: &str = STD_PROGRAM_BASE_UINT64;
#[doc(hidden)]
pub const STD_GO_BASE_ARRAY_LENGTH: &str = STD_PROGRAM_BASE_ARRAY_LENGTH;
#[doc(hidden)]
pub const STD_GO_BASE_ARRAY: &str = STD_PROGRAM_BASE_ARRAY;
#[doc(hidden)]
pub const STD_GO_BASE_STRUCT_SHAPE: &str = STD_PROGRAM_BASE_STRUCT_SHAPE;
#[doc(hidden)]
pub const STD_GO_BASE_STRUCT_FIELD: &str = STD_PROGRAM_BASE_STRUCT_FIELD;
#[doc(hidden)]
pub const STD_GO_BASE_STRUCT_FIELD_TYPE: &str = STD_PROGRAM_BASE_STRUCT_FIELD_TYPE;
#[doc(hidden)]
pub const STD_GO_BASE_STRUCT_VALUE: &str = STD_PROGRAM_BASE_STRUCT_VALUE;

/// Encodes one VIR type under a validated semantic context.
pub fn encode_vir_type(
    profile: SemanticProfile,
    parameters: &SemanticParameters,
    declarations: &[VirStructDecl],
    input: &VirType,
) -> Result<MpkTypeTerm, TypeEncodeError> {
    ProgramTypeEncoder::new(profile, parameters, declarations)?.encode(input)
}

/// Language-neutral VIR type encoder.
///
/// Profiles share the value carriers. Carrying and validating the semantic
/// context here prevents a caller from selecting a target-sized type without
/// first fixing the profile's pointer width.
#[derive(Clone, Debug)]
pub struct ProgramTypeEncoder<'a> {
    parameters: &'a SemanticParameters,
    declarations: std::collections::BTreeMap<&'a str, &'a VirStructDecl>,
}

impl<'a> ProgramTypeEncoder<'a> {
    pub fn new(
        profile: SemanticProfile,
        parameters: &'a SemanticParameters,
        declarations: &'a [VirStructDecl],
    ) -> Result<Self, TypeEncodeError> {
        validate_semantic_parameters(profile, parameters)
            .map_err(TypeEncodeError::SemanticProfile)?;
        let mut by_id = std::collections::BTreeMap::new();
        let mut available = BTreeSet::new();
        for declaration in declarations {
            if declaration.id.is_empty() {
                return Err(TypeEncodeError::EmptyStructDeclarationId);
            }
            if by_id.insert(declaration.id.as_str(), declaration).is_some() {
                return Err(TypeEncodeError::DuplicateStructDeclaration {
                    id: declaration.id.clone(),
                });
            }
            let mut field_names = BTreeSet::new();
            for (field_index, field) in declaration.fields.iter().enumerate() {
                if field.name.is_empty() {
                    return Err(TypeEncodeError::EmptyVirStructFieldName {
                        declaration_id: declaration.id.clone(),
                        field_index,
                    });
                }
                if !field_names.insert(field.name.as_str()) {
                    return Err(TypeEncodeError::DuplicateVirStructFieldName {
                        declaration_id: declaration.id.clone(),
                        field_name: field.name.clone(),
                    });
                }
                validate_struct_references(&field.r#type, &declaration.id, &available)?;
            }
            available.insert(declaration.id.as_str());
        }
        Ok(Self {
            parameters,
            declarations: by_id,
        })
    }

    pub fn encode(&self, input: &VirType) -> Result<MpkTypeTerm, TypeEncodeError> {
        match input {
            VirType::Bool {} => Ok(MpkTypeTerm::constant(STD_PROGRAM_BASE_BOOL)),
            VirType::Bv { width, signed } => Ok(MpkTypeTerm::constant(bitvector_alias(
                width.bits(),
                *signed,
            )?)),
            VirType::Array { length, element } => Ok(MpkTypeTerm::apply(
                STD_PROGRAM_BASE_ARRAY,
                [
                    self.encode(element)?,
                    MpkTypeTerm::apply(
                        STD_PROGRAM_BASE_ARRAY_LENGTH,
                        [MpkTypeTerm::nat_literal(u64::from(length.get()))],
                    ),
                ],
            )),
            VirType::Struct { id } => self.encode_struct(id),
        }
    }

    pub fn encode_target_sized_integer(
        &self,
        signed: bool,
    ) -> Result<MpkTypeTerm, TypeEncodeError> {
        Ok(MpkTypeTerm::constant(bitvector_alias(
            self.parameters.pointer_width().bits(),
            signed,
        )?))
    }

    fn encode_struct(&self, id: &str) -> Result<MpkTypeTerm, TypeEncodeError> {
        let declaration = self
            .declarations
            .get(id)
            .copied()
            .ok_or_else(|| TypeEncodeError::UnknownStructDeclaration { id: id.to_owned() })?;
        let mut shape_args = Vec::with_capacity(declaration.fields.len() + 1);
        shape_args.push(MpkTypeTerm::string_literal(&declaration.id));
        for field in &declaration.fields {
            shape_args.push(MpkTypeTerm::apply(
                STD_PROGRAM_BASE_STRUCT_FIELD,
                [
                    MpkTypeTerm::string_literal(&field.name),
                    MpkTypeTerm::apply(
                        STD_PROGRAM_BASE_STRUCT_FIELD_TYPE,
                        [self.encode(&field.r#type)?],
                    ),
                ],
            ));
        }
        Ok(MpkTypeTerm::apply(
            STD_PROGRAM_BASE_STRUCT_VALUE,
            [MpkTypeTerm::apply(
                STD_PROGRAM_BASE_STRUCT_SHAPE,
                shape_args,
            )],
        ))
    }
}

fn validate_struct_references(
    input: &VirType,
    declaration_id: &str,
    available: &BTreeSet<&str>,
) -> Result<(), TypeEncodeError> {
    match input {
        VirType::Bool {} | VirType::Bv { .. } => Ok(()),
        VirType::Array { element, .. } => {
            validate_struct_references(element, declaration_id, available)
        }
        VirType::Struct { id } if available.contains(id.as_str()) => Ok(()),
        VirType::Struct { id } => Err(TypeEncodeError::StructDeclarationOrder {
            declaration_id: declaration_id.to_owned(),
            referenced_id: id.clone(),
        }),
    }
}

pub fn encode_gir_type(input: &GirType) -> Result<MpkTypeTerm, TypeEncodeError> {
    TypeEncoder::new().encode(input)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TypeEncoder;

impl TypeEncoder {
    pub fn new() -> Self {
        Self
    }

    pub fn encode(self, input: &GirType) -> Result<MpkTypeTerm, TypeEncodeError> {
        match input.kind {
            GirTypeKind::Bool => self.encode_bool(input),
            GirTypeKind::BitVector => self.encode_bitvector(input),
            GirTypeKind::Array => self.encode_array(input),
            GirTypeKind::Struct => self.encode_struct(input),
        }
    }

    fn encode_bool(self, input: &GirType) -> Result<MpkTypeTerm, TypeEncodeError> {
        if input.name.is_some()
            || input.width.is_some()
            || input.signed.is_some()
            || input.length.is_some()
            || input.element.is_some()
            || !input.fields.is_empty()
        {
            return Err(TypeEncodeError::BoolContainsUnsupportedFields);
        }
        Ok(MpkTypeTerm::constant(STD_PROGRAM_BASE_BOOL))
    }

    fn encode_bitvector(self, input: &GirType) -> Result<MpkTypeTerm, TypeEncodeError> {
        if input.name.is_some()
            || input.length.is_some()
            || input.element.is_some()
            || !input.fields.is_empty()
        {
            return Err(TypeEncodeError::BitVectorContainsUnsupportedFields);
        }
        let width = input.width.ok_or(TypeEncodeError::BitVectorMissingWidth)?;
        let signed = input
            .signed
            .ok_or(TypeEncodeError::BitVectorMissingSigned)?;
        Ok(MpkTypeTerm::constant(bitvector_alias(width, signed)?))
    }

    fn encode_array(self, input: &GirType) -> Result<MpkTypeTerm, TypeEncodeError> {
        if input.width.is_some() || input.signed.is_some() || !input.fields.is_empty() {
            return Err(TypeEncodeError::ArrayContainsUnsupportedFields);
        }
        let length = input.length.ok_or(TypeEncodeError::ArrayMissingLength)?;
        let element = input
            .element
            .as_deref()
            .ok_or(TypeEncodeError::ArrayMissingElement)?;

        let element_term = self.encode(element)?;
        let length_term = MpkTypeTerm::apply(
            STD_PROGRAM_BASE_ARRAY_LENGTH,
            [MpkTypeTerm::nat_literal(length)],
        );
        Ok(MpkTypeTerm::apply(
            STD_PROGRAM_BASE_ARRAY,
            [element_term, length_term],
        ))
    }

    fn encode_struct(self, input: &GirType) -> Result<MpkTypeTerm, TypeEncodeError> {
        if input.width.is_some()
            || input.signed.is_some()
            || input.length.is_some()
            || input.element.is_some()
        {
            return Err(TypeEncodeError::StructContainsUnsupportedFields);
        }

        let mut shape_args = Vec::with_capacity(input.fields.len() + 1);
        shape_args.push(MpkTypeTerm::string_literal(
            input.name.as_deref().unwrap_or_default(),
        ));
        let mut field_names = BTreeSet::new();
        for (field_index, field) in input.fields.iter().enumerate() {
            if field.name.is_empty() {
                return Err(TypeEncodeError::EmptyStructFieldName { field_index });
            }
            if !field_names.insert(field.name.clone()) {
                return Err(TypeEncodeError::DuplicateStructFieldName {
                    field_name: field.name.clone(),
                });
            }
            shape_args.push(self.encode_struct_field(field_index, field)?);
        }
        let shape = MpkTypeTerm::apply(STD_PROGRAM_BASE_STRUCT_SHAPE, shape_args);
        Ok(MpkTypeTerm::apply(STD_PROGRAM_BASE_STRUCT_VALUE, [shape]))
    }

    fn encode_struct_field(
        self,
        field_index: usize,
        field: &GirFieldType,
    ) -> Result<MpkTypeTerm, TypeEncodeError> {
        if field.name.is_empty() {
            return Err(TypeEncodeError::EmptyStructFieldName { field_index });
        }

        let field_type =
            self.encode(&field.r#type)
                .map_err(|source| TypeEncodeError::StructField {
                    field_name: field.name.clone(),
                    source: Box::new(source),
                })?;
        let field_type = MpkTypeTerm::apply(STD_PROGRAM_BASE_STRUCT_FIELD_TYPE, [field_type]);
        Ok(MpkTypeTerm::apply(
            STD_PROGRAM_BASE_STRUCT_FIELD,
            [MpkTypeTerm::string_literal(&field.name), field_type],
        ))
    }
}

fn bitvector_alias(width: u32, signed: bool) -> Result<&'static str, TypeEncodeError> {
    match (width, signed) {
        (8, true) => Ok(STD_PROGRAM_BASE_INT8),
        (16, true) => Ok(STD_PROGRAM_BASE_INT16),
        (32, true) => Ok(STD_PROGRAM_BASE_INT32),
        (64, true) => Ok(STD_PROGRAM_BASE_INT64),
        (8, false) => Ok(STD_PROGRAM_BASE_UINT8),
        (16, false) => Ok(STD_PROGRAM_BASE_UINT16),
        (32, false) => Ok(STD_PROGRAM_BASE_UINT32),
        (64, false) => Ok(STD_PROGRAM_BASE_UINT64),
        _ => Err(TypeEncodeError::UnsupportedBitVectorWidth { width }),
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MpkTypeTerm {
    Constant {
        name: String,
    },
    Apply {
        function: String,
        args: Vec<MpkTypeTerm>,
    },
    NatLiteral {
        value: u64,
    },
    StringLiteral {
        value: String,
    },
}

impl MpkTypeTerm {
    pub fn constant(name: impl Into<String>) -> Self {
        Self::Constant { name: name.into() }
    }

    pub fn apply<I>(function: impl Into<String>, args: I) -> Self
    where
        I: IntoIterator<Item = MpkTypeTerm>,
    {
        Self::Apply {
            function: function.into(),
            args: args.into_iter().collect(),
        }
    }

    pub fn nat_literal(value: u64) -> Self {
        Self::NatLiteral { value }
    }

    pub fn string_literal(value: impl Into<String>) -> Self {
        Self::StringLiteral {
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeEncodeError {
    SemanticProfile(SemanticProfileError),
    EmptyStructDeclarationId,
    DuplicateStructDeclaration {
        id: String,
    },
    EmptyVirStructFieldName {
        declaration_id: String,
        field_index: usize,
    },
    DuplicateVirStructFieldName {
        declaration_id: String,
        field_name: String,
    },
    StructDeclarationOrder {
        declaration_id: String,
        referenced_id: String,
    },
    UnknownStructDeclaration {
        id: String,
    },
    BoolContainsUnsupportedFields,
    BitVectorContainsUnsupportedFields,
    BitVectorMissingWidth,
    BitVectorMissingSigned,
    UnsupportedBitVectorWidth {
        width: u32,
    },
    ArrayContainsUnsupportedFields,
    ArrayMissingLength,
    ArrayMissingElement,
    StructContainsUnsupportedFields,
    EmptyStructFieldName {
        field_index: usize,
    },
    DuplicateStructFieldName {
        field_name: String,
    },
    StructField {
        field_name: String,
        source: Box<TypeEncodeError>,
    },
}

impl fmt::Display for TypeEncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SemanticProfile(error) => write!(formatter, "invalid semantic profile: {error}"),
            Self::EmptyStructDeclarationId => {
                write!(formatter, "VIR struct declaration ID is empty")
            }
            Self::DuplicateStructDeclaration { id } => {
                write!(formatter, "VIR struct declaration {id:?} is duplicated")
            }
            Self::EmptyVirStructFieldName {
                declaration_id,
                field_index,
            } => write!(
                formatter,
                "VIR struct {declaration_id:?} field {field_index} has an empty name"
            ),
            Self::DuplicateVirStructFieldName {
                declaration_id,
                field_name,
            } => write!(
                formatter,
                "VIR struct {declaration_id:?} field name {field_name:?} is duplicated"
            ),
            Self::StructDeclarationOrder {
                declaration_id,
                referenced_id,
            } => write!(
                formatter,
                "VIR struct {declaration_id:?} references {referenced_id:?} before its declaration"
            ),
            Self::UnknownStructDeclaration { id } => {
                write!(formatter, "VIR struct declaration {id:?} does not exist")
            }
            Self::BoolContainsUnsupportedFields => {
                write!(formatter, "GIR bool type contains unsupported fields")
            }
            Self::BitVectorContainsUnsupportedFields => {
                write!(formatter, "GIR bitvector type contains unsupported fields")
            }
            Self::BitVectorMissingWidth => write!(formatter, "GIR bitvector type is missing width"),
            Self::BitVectorMissingSigned => {
                write!(formatter, "GIR bitvector type is missing signed")
            }
            Self::UnsupportedBitVectorWidth { width } => {
                write!(formatter, "GIR bitvector width {width} is not supported")
            }
            Self::ArrayContainsUnsupportedFields => {
                write!(formatter, "GIR array type contains unsupported fields")
            }
            Self::ArrayMissingLength => write!(formatter, "GIR array type is missing length"),
            Self::ArrayMissingElement => write!(formatter, "GIR array type is missing element"),
            Self::StructContainsUnsupportedFields => {
                write!(formatter, "GIR struct type contains unsupported fields")
            }
            Self::EmptyStructFieldName { field_index } => {
                write!(formatter, "GIR struct field {field_index} has empty name")
            }
            Self::DuplicateStructFieldName { field_name } => {
                write!(
                    formatter,
                    "GIR struct field name {field_name:?} is duplicated"
                )
            }
            Self::StructField { field_name, source } => {
                write!(formatter, "GIR struct field {field_name:?}: {source}")
            }
        }
    }
}

impl std::error::Error for TypeEncodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::SemanticProfile(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bool_type() -> GirType {
        GirType {
            kind: GirTypeKind::Bool,
            name: None,
            width: None,
            signed: None,
            length: None,
            element: None,
            fields: Vec::new(),
        }
    }

    fn bv_type(width: u32, signed: bool) -> GirType {
        GirType {
            kind: GirTypeKind::BitVector,
            name: None,
            width: Some(width),
            signed: Some(signed),
            length: None,
            element: None,
            fields: Vec::new(),
        }
    }

    fn array_type(length: u64, element: GirType) -> GirType {
        GirType {
            kind: GirTypeKind::Array,
            name: None,
            width: None,
            signed: None,
            length: Some(length),
            element: Some(Box::new(element)),
            fields: Vec::new(),
        }
    }

    fn struct_type(name: &str, fields: Vec<GirFieldType>) -> GirType {
        GirType {
            kind: GirTypeKind::Struct,
            name: Some(name.to_owned()),
            width: None,
            signed: None,
            length: None,
            element: None,
            fields,
        }
    }

    fn field(name: &str, r#type: GirType) -> GirFieldType {
        GirFieldType {
            name: name.to_owned(),
            r#type,
        }
    }

    fn snapshot(term: &MpkTypeTerm) -> String {
        serde_json::to_string_pretty(term).expect("type term serializes")
    }

    #[test]
    fn encodes_go_bool_snapshot() {
        let term = encode_gir_type(&bool_type()).expect("bool type encodes");

        assert_eq!(
            snapshot(&term),
            r#"{
  "kind": "constant",
  "name": "Std.Program.Base.Bool"
}"#
        );
    }

    #[test]
    fn encodes_signed_and_unsigned_integer_snapshots() {
        let signed = encode_gir_type(&bv_type(8, true)).expect("signed type encodes");
        let unsigned = encode_gir_type(&bv_type(64, false)).expect("unsigned type encodes");

        assert_eq!(
            snapshot(&signed),
            r#"{
  "kind": "constant",
  "name": "Std.Program.Base.Int8"
}"#
        );
        assert_eq!(
            snapshot(&unsigned),
            r#"{
  "kind": "constant",
  "name": "Std.Program.Base.Uint64"
}"#
        );
    }

    #[test]
    fn encodes_fixed_array_snapshot() {
        let term =
            encode_gir_type(&array_type(3, bv_type(16, false))).expect("fixed array type encodes");

        assert_eq!(
            snapshot(&term),
            r#"{
  "kind": "apply",
  "function": "Std.Program.Base.Array",
  "args": [
    {
      "kind": "constant",
      "name": "Std.Program.Base.Uint16"
    },
    {
      "kind": "apply",
      "function": "Std.Program.Base.Array.Length",
      "args": [
        {
          "kind": "nat_literal",
          "value": 3
        }
      ]
    }
  ]
}"#
        );
    }

    #[test]
    fn encodes_struct_snapshot() {
        let term = encode_gir_type(&struct_type(
            "example.Pair",
            vec![
                field("Left", bv_type(64, true)),
                field("Flags", array_type(2, bool_type())),
            ],
        ))
        .expect("struct type encodes");

        assert_eq!(
            snapshot(&term),
            r#"{
  "kind": "apply",
  "function": "Std.Program.Base.Struct.Value",
  "args": [
    {
      "kind": "apply",
      "function": "Std.Program.Base.Struct.Shape",
      "args": [
        {
          "kind": "string_literal",
          "value": "example.Pair"
        },
        {
          "kind": "apply",
          "function": "Std.Program.Base.Struct.Field",
          "args": [
            {
              "kind": "string_literal",
              "value": "Left"
            },
            {
              "kind": "apply",
              "function": "Std.Program.Base.Struct.FieldType",
              "args": [
                {
                  "kind": "constant",
                  "name": "Std.Program.Base.Int64"
                }
              ]
            }
          ]
        },
        {
          "kind": "apply",
          "function": "Std.Program.Base.Struct.Field",
          "args": [
            {
              "kind": "string_literal",
              "value": "Flags"
            },
            {
              "kind": "apply",
              "function": "Std.Program.Base.Struct.FieldType",
              "args": [
                {
                  "kind": "apply",
                  "function": "Std.Program.Base.Array",
                  "args": [
                    {
                      "kind": "constant",
                      "name": "Std.Program.Base.Bool"
                    },
                    {
                      "kind": "apply",
                      "function": "Std.Program.Base.Array.Length",
                      "args": [
                        {
                          "kind": "nat_literal",
                          "value": 2
                        }
                      ]
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }
  ]
}"#
        );
    }

    #[test]
    fn rejects_unsupported_bitvector_width() {
        let error = encode_gir_type(&bv_type(128, false)).expect_err("BV128 rejects");

        assert_eq!(
            error,
            TypeEncodeError::UnsupportedBitVectorWidth { width: 128 }
        );
    }

    #[test]
    fn rejects_bitvector_without_signed_flag() {
        let input = GirType {
            signed: None,
            ..bv_type(64, true)
        };

        let error = encode_gir_type(&input).expect_err("missing signed rejects");

        assert_eq!(error, TypeEncodeError::BitVectorMissingSigned);
    }

    #[test]
    fn rejects_array_without_element() {
        let input = GirType {
            element: None,
            ..array_type(2, bool_type())
        };

        let error = encode_gir_type(&input).expect_err("missing element rejects");

        assert_eq!(error, TypeEncodeError::ArrayMissingElement);
    }

    #[test]
    fn rejects_empty_struct_field_names() {
        let input = struct_type("", vec![field("", bool_type())]);

        let error = encode_gir_type(&input).expect_err("empty field name rejects");

        assert_eq!(
            error,
            TypeEncodeError::EmptyStructFieldName { field_index: 0 }
        );
    }

    #[test]
    fn rejects_duplicate_struct_field_names() {
        let input = struct_type(
            "example.Bad",
            vec![
                field("Value", bool_type()),
                field("Value", bv_type(8, false)),
            ],
        );

        let error = encode_gir_type(&input).expect_err("duplicate field name rejects");

        assert_eq!(
            error,
            TypeEncodeError::DuplicateStructFieldName {
                field_name: "Value".to_owned()
            }
        );
    }
}
