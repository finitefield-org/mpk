//! VIR type to unresolved MPK type-term encoding.

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};

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
    UnsupportedBitVectorWidth {
        width: u32,
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
            Self::UnsupportedBitVectorWidth { width } => {
                write!(formatter, "VIR bitvector width {width} is not supported")
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
