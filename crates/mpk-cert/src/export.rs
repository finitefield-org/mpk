//! Export block builder for public declaration interfaces.

use crate::{
    encode::{
        Certificate, Declaration, DeclarationId, DeclarationKind, DefinitionReducibility,
        ExportEntry, HashBytes, NameId,
    },
    hash::{export_hash, hash_with_domain, HashDomain},
    DeclarationTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportBuildError {
    kind: ExportBuildErrorKind,
    detail: String,
}

impl ExportBuildError {
    pub fn kind(&self) -> ExportBuildErrorKind {
        self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    fn new(kind: ExportBuildErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum ExportBuildErrorKind {
    MissingDeclarationName,
    TooManyDeclarations,
}

pub fn build_export_block(certificate: &Certificate) -> Result<Vec<ExportEntry>, ExportBuildError> {
    build_export_block_for_declarations(&certificate.name_table, &certificate.declarations)
}

pub fn build_export_block_for_declarations(
    name_table: &[String],
    declarations: &[Declaration],
) -> Result<Vec<ExportEntry>, ExportBuildError> {
    let mut entries = Vec::with_capacity(declarations.len());

    for (index, declaration) in declarations.iter().enumerate() {
        let declaration_id = DeclarationId::try_from(index).map_err(|_| {
            ExportBuildError::new(
                ExportBuildErrorKind::TooManyDeclarations,
                "declaration count exceeds u32 ids",
            )
        })?;
        entries.push(ExportEntry {
            name: declaration.name,
            declaration: declaration_id,
            declaration_hash: declaration_interface_hash(name_table, declaration)?,
        });
    }

    Ok(entries)
}

pub fn declaration_interface_hash(
    name_table: &[String],
    declaration: &Declaration,
) -> Result<HashBytes, ExportBuildError> {
    let payload = encode_declaration_interface(name_table, declaration)?;
    Ok(hash_with_domain(HashDomain::Declaration, &payload))
}

pub fn encode_export_block(export_block: &[ExportEntry]) -> Vec<u8> {
    let mut encoder = ExportEncoder::new();
    encoder.write_vec(export_block, ExportEncoder::write_export_entry);
    encoder.finish()
}

pub fn export_block_hash(export_block: &[ExportEntry]) -> HashBytes {
    export_hash(&encode_export_block(export_block))
}

fn encode_declaration_interface(
    name_table: &[String],
    declaration: &Declaration,
) -> Result<Vec<u8>, ExportBuildError> {
    let name = declaration_name(name_table, declaration.name)?;
    let mut encoder = ExportEncoder::new();
    encoder.write_str_slice(name);
    encoder.write_declaration_interface_kind(&declaration.kind);
    Ok(encoder.finish())
}

fn declaration_name(name_table: &[String], name: NameId) -> Result<&str, ExportBuildError> {
    let index = usize::try_from(name).expect("u32 name id fits in usize");
    name_table.get(index).map(String::as_str).ok_or_else(|| {
        ExportBuildError::new(
            ExportBuildErrorKind::MissingDeclarationName,
            format!("declaration references missing name id {name}"),
        )
    })
}

struct ExportEncoder {
    bytes: Vec<u8>,
}

impl ExportEncoder {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn finish(self) -> Vec<u8> {
        self.bytes
    }

    fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    fn write_u32(&mut self, value: u32) {
        self.write_u64(u64::from(value));
    }

    fn write_u64(&mut self, value: u64) {
        Self::write_unsigned_varint_to(value, &mut self.bytes);
    }

    fn write_len(&mut self, len: usize) {
        let len = u64::try_from(len).expect("export block section length exceeds u64");
        self.write_u64(len);
    }

    fn write_unsigned_varint_to(mut value: u64, out: &mut Vec<u8>) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn write_bytes_with_len(&mut self, bytes: &[u8]) {
        self.write_len(bytes.len());
        self.write_bytes(bytes);
    }

    fn write_str_slice(&mut self, value: &str) {
        self.write_bytes_with_len(value.as_bytes());
    }

    fn write_hash(&mut self, hash: &HashBytes) {
        self.write_bytes(hash);
    }

    fn write_vec<T>(&mut self, values: &[T], mut write_item: impl FnMut(&mut Self, &T)) {
        self.write_len(values.len());
        for value in values {
            write_item(self, value);
        }
    }

    fn write_export_entry(&mut self, entry: &ExportEntry) {
        self.write_u32(entry.name);
        self.write_u32(entry.declaration);
        self.write_hash(&entry.declaration_hash);
    }

    fn write_reducibility(&mut self, reducibility: DefinitionReducibility) {
        match reducibility {
            DefinitionReducibility::Reducible => self.write_u8(0x00),
            DefinitionReducibility::Opaque => self.write_u8(0x01),
        }
    }

    fn write_declaration_interface_kind(&mut self, kind: &DeclarationKind) {
        match kind {
            DeclarationKind::Axiom { ty } => {
                self.write_u8(DeclarationTag::Axiom.as_u8());
                self.write_u32(*ty);
            }
            DeclarationKind::Def {
                ty,
                value,
                reducibility,
            } => {
                self.write_u8(DeclarationTag::Def.as_u8());
                self.write_u32(*ty);
                self.write_reducibility(*reducibility);
                if *reducibility == DefinitionReducibility::Reducible {
                    self.write_u32(*value);
                }
            }
            DeclarationKind::Theorem { ty, proof: _ } => {
                self.write_u8(DeclarationTag::Theorem.as_u8());
                self.write_u32(*ty);
            }
            DeclarationKind::Inductive { ty } => {
                self.write_u8(DeclarationTag::Inductive.as_u8());
                self.write_u32(*ty);
            }
            DeclarationKind::Constructor {
                ty,
                inductive,
                generated,
            } => {
                self.write_u8(DeclarationTag::Constructor.as_u8());
                self.write_u32(*ty);
                self.write_u32(*inductive);
                self.write_bool(*generated);
            }
            DeclarationKind::Recursor {
                ty,
                inductive,
                generated,
            } => {
                self.write_u8(DeclarationTag::Recursor.as_u8());
                self.write_u32(*ty);
                self.write_u32(*inductive);
                self.write_bool(*generated);
            }
            DeclarationKind::TheoryPrimitive { ty } => {
                self.write_u8(DeclarationTag::TheoryPrimitive.as_u8());
                self.write_u32(*ty);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        encode::{
            AxiomReport, Certificate, CertificateHashes, Declaration, DeclarationKind,
            DefinitionReducibility,
        },
        export_hash,
    };

    use super::{
        build_export_block, build_export_block_for_declarations, declaration_interface_hash,
        encode_export_block, export_block_hash, ExportBuildErrorKind,
    };

    fn certificate_with_declarations(
        name_table: Vec<&str>,
        declarations: Vec<Declaration>,
    ) -> Certificate {
        Certificate {
            module: "Example.Export".to_owned(),
            imports: Vec::new(),
            name_table: name_table.into_iter().map(str::to_owned).collect(),
            level_table: Vec::new(),
            term_table: Vec::new(),
            proof_node_table: Vec::new(),
            declarations,
            theory_certificates: Vec::new(),
            export_block: Vec::new(),
            axiom_report: AxiomReport::default(),
            source_manifest: None,
            hashes: CertificateHashes::default(),
        }
    }

    fn declaration(name: u32, kind: DeclarationKind) -> Declaration {
        Declaration { name, kind }
    }

    #[test]
    fn export_block_derives_entries_for_public_declarations() {
        let certificate = certificate_with_declarations(
            vec![
                "Example.Export.ax",
                "Example.Export.def",
                "Example.Export.thm",
                "Example.Export.ind",
                "Example.Export.ctor",
                "Example.Export.rec",
                "Example.Export.theory",
            ],
            vec![
                declaration(0, DeclarationKind::Axiom { ty: 0 }),
                declaration(
                    1,
                    DeclarationKind::Def {
                        ty: 0,
                        value: 1,
                        reducibility: DefinitionReducibility::Reducible,
                    },
                ),
                declaration(2, DeclarationKind::Theorem { ty: 0, proof: 2 }),
                declaration(3, DeclarationKind::Inductive { ty: 0 }),
                declaration(
                    4,
                    DeclarationKind::Constructor {
                        ty: 0,
                        inductive: 3,
                        generated: true,
                    },
                ),
                declaration(
                    5,
                    DeclarationKind::Recursor {
                        ty: 0,
                        inductive: 3,
                        generated: true,
                    },
                ),
                declaration(6, DeclarationKind::TheoryPrimitive { ty: 0 }),
            ],
        );

        let export_block = build_export_block(&certificate).expect("export block builds");

        assert_eq!(export_block.len(), certificate.declarations.len());
        for (index, entry) in export_block.iter().enumerate() {
            assert_eq!(entry.name, certificate.declarations[index].name);
            assert_eq!(entry.declaration, index as u32);
            assert_eq!(
                entry.declaration_hash,
                declaration_interface_hash(
                    &certificate.name_table,
                    &certificate.declarations[index]
                )
                .expect("interface hash builds")
            );
        }
    }

    #[test]
    fn theorem_proof_body_is_excluded_from_export_hash() {
        let name_table = vec!["Example.Export.thm"];
        let first = certificate_with_declarations(
            name_table.clone(),
            vec![declaration(0, DeclarationKind::Theorem { ty: 0, proof: 1 })],
        );
        let second = certificate_with_declarations(
            name_table,
            vec![declaration(0, DeclarationKind::Theorem { ty: 0, proof: 2 })],
        );

        let first_export = build_export_block(&first).expect("first export block builds");
        let second_export = build_export_block(&second).expect("second export block builds");

        assert_eq!(
            first_export[0].declaration_hash,
            second_export[0].declaration_hash
        );
        assert_eq!(
            export_block_hash(&first_export),
            export_block_hash(&second_export)
        );
    }

    #[test]
    fn theorem_type_is_included_in_export_hash() {
        let name_table = vec!["Example.Export.thm"];
        let first = certificate_with_declarations(
            name_table.clone(),
            vec![declaration(0, DeclarationKind::Theorem { ty: 0, proof: 1 })],
        );
        let second = certificate_with_declarations(
            name_table,
            vec![declaration(0, DeclarationKind::Theorem { ty: 1, proof: 1 })],
        );

        let first_export = build_export_block(&first).expect("first export block builds");
        let second_export = build_export_block(&second).expect("second export block builds");

        assert_ne!(
            first_export[0].declaration_hash,
            second_export[0].declaration_hash
        );
        assert_ne!(
            export_block_hash(&first_export),
            export_block_hash(&second_export)
        );
    }

    #[test]
    fn declaration_name_is_included_in_interface_hash() {
        let declaration = declaration(0, DeclarationKind::Axiom { ty: 0 });
        let first_name_table = vec!["Example.Export.first".to_owned()];
        let second_name_table = vec!["Example.Export.second".to_owned()];

        assert_ne!(
            declaration_interface_hash(&first_name_table, &declaration).expect("hash builds"),
            declaration_interface_hash(&second_name_table, &declaration).expect("hash builds")
        );
    }

    #[test]
    fn definition_body_visibility_matches_reducibility() {
        let name_table = vec!["Example.Export.def".to_owned()];
        let reducible_first = declaration(
            0,
            DeclarationKind::Def {
                ty: 0,
                value: 1,
                reducibility: DefinitionReducibility::Reducible,
            },
        );
        let reducible_second = declaration(
            0,
            DeclarationKind::Def {
                ty: 0,
                value: 2,
                reducibility: DefinitionReducibility::Reducible,
            },
        );
        let opaque_first = declaration(
            0,
            DeclarationKind::Def {
                ty: 0,
                value: 1,
                reducibility: DefinitionReducibility::Opaque,
            },
        );
        let opaque_second = declaration(
            0,
            DeclarationKind::Def {
                ty: 0,
                value: 2,
                reducibility: DefinitionReducibility::Opaque,
            },
        );

        assert_ne!(
            declaration_interface_hash(&name_table, &reducible_first).expect("hash builds"),
            declaration_interface_hash(&name_table, &reducible_second).expect("hash builds")
        );
        assert_eq!(
            declaration_interface_hash(&name_table, &opaque_first).expect("hash builds"),
            declaration_interface_hash(&name_table, &opaque_second).expect("hash builds")
        );
    }

    #[test]
    fn export_block_hash_uses_export_domain() {
        let certificate = certificate_with_declarations(
            vec!["Example.Export.ax"],
            vec![declaration(0, DeclarationKind::Axiom { ty: 0 })],
        );
        let export_block = build_export_block(&certificate).expect("export block builds");

        assert_eq!(
            export_block_hash(&export_block),
            export_hash(&encode_export_block(&export_block))
        );
    }

    #[test]
    fn missing_declaration_name_rejects() {
        let declarations = vec![declaration(1, DeclarationKind::Axiom { ty: 0 })];

        let error =
            build_export_block_for_declarations(&["Example.Export.ax".to_owned()], &declarations)
                .unwrap_err();

        assert_eq!(error.kind(), ExportBuildErrorKind::MissingDeclarationName);
        assert!(error.detail().contains("missing name id 1"));
    }
}
