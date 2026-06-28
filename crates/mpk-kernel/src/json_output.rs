//! Stable JSON rendering for kernel verifier outcomes.

use std::fmt::Write as _;

use mpk_cert::{
    certificate_hash,
    encode::{
        AxiomReport, AxiomReportEntry, AxiomReportSummary, DeclarationAxiomDependencies, HashBytes,
    },
    hash_hex,
};

use crate::verifier::{verify_certificate_bytes, VerificationError, VerificationReport};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationJsonOutput {
    pub accepted: bool,
    pub json: String,
}

pub fn verify_certificate_bytes_json(bytes: &[u8]) -> String {
    verify_certificate_bytes_json_output(bytes).json
}

pub fn verify_certificate_bytes_json_output(bytes: &[u8]) -> VerificationJsonOutput {
    let computed_certificate_hash = certificate_hash(bytes);
    match verify_certificate_bytes(bytes) {
        Ok(report) => VerificationJsonOutput {
            accepted: true,
            json: render_verification_report_json(&report),
        },
        Err(error) => VerificationJsonOutput {
            accepted: false,
            json: render_verification_error_json(&computed_certificate_hash, &error),
        },
    }
}

pub fn verify_certificate_bytes_axiom_report_json_output(bytes: &[u8]) -> VerificationJsonOutput {
    let computed_certificate_hash = certificate_hash(bytes);
    match verify_certificate_bytes(bytes) {
        Ok(report) => VerificationJsonOutput {
            accepted: true,
            json: render_axiom_report_json(&report),
        },
        Err(error) => VerificationJsonOutput {
            accepted: false,
            json: render_verification_error_json(&computed_certificate_hash, &error),
        },
    }
}

pub fn render_axiom_report_json(report: &VerificationReport) -> String {
    let mut output = String::new();
    output.push_str("{\"certificate_hash\":");
    write_hash(&mut output, &report.certificate_hash);
    output.push_str(",\"axiom_report_hash\":");
    write_hash(&mut output, &report.axiom_report_hash);
    output.push_str(",\"axiom_report\":");
    write_axiom_report(&mut output, &report.axiom_report);
    output.push('}');
    output
}

pub fn render_verification_report_json(report: &VerificationReport) -> String {
    let mut output = String::new();
    output.push_str("{\"verdict\":\"accepted\",\"module\":");
    write_json_string(&mut output, &report.module);
    write!(
        output,
        ",\"declaration_count\":{},\"axiom_count\":{},\"hashes\":",
        report.declaration_count, report.axiom_count
    )
    .expect("writing to string cannot fail");
    write_hashes_object(
        &mut output,
        Some(&report.export_hash),
        Some(&report.axiom_report_hash),
        &report.certificate_hash,
    );
    output.push_str(",\"axiom_report\":");
    write_axiom_report(&mut output, &report.axiom_report);
    output.push_str(",\"error_code\":null,\"error_detail\":null}");
    output
}

pub fn render_verification_error_json(
    computed_certificate_hash: &HashBytes,
    error: &VerificationError,
) -> String {
    let mut output = String::new();
    output.push_str(
        "{\"verdict\":\"rejected\",\"module\":null,\"declaration_count\":null,\"axiom_count\":null,\"hashes\":",
    );
    write_hashes_object(&mut output, None, None, computed_certificate_hash);
    output.push_str(",\"axiom_report\":null,\"error_code\":");
    write_json_string(&mut output, error.kind().code());
    output.push_str(",\"error_detail\":");
    write_json_string(&mut output, error.detail());
    output.push('}');
    output
}

fn write_hashes_object(
    output: &mut String,
    export_hash: Option<&HashBytes>,
    axiom_report_hash: Option<&HashBytes>,
    certificate_hash: &HashBytes,
) {
    output.push_str("{\"export\":");
    write_optional_hash(output, export_hash);
    output.push_str(",\"axiom_report\":");
    write_optional_hash(output, axiom_report_hash);
    output.push_str(",\"certificate\":");
    write_hash(output, certificate_hash);
    output.push('}');
}

fn write_axiom_report(output: &mut String, report: &AxiomReport) {
    output.push_str("{\"summary\":");
    write_axiom_report_summary(output, &report.summary);
    output.push_str(",\"entries\":[");
    for (index, entry) in report.entries.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        write_axiom_report_entry(output, entry);
    }
    output.push_str("],\"declaration_dependencies\":[");
    for (index, dependencies) in report.declaration_dependencies.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        write_declaration_axiom_dependencies(output, dependencies);
    }
    output.push_str("]}");
}

fn write_axiom_report_summary(output: &mut String, summary: &AxiomReportSummary) {
    write!(
        output,
        "{{\"core_axiom_count\":{},\"builtin_theory_axiom_count\":{},\"go_semantics_axiom_count\":{},\"external_axiom_count\":{},\"total_axiom_count\":{}}}",
        summary.core_axiom_count,
        summary.builtin_theory_axiom_count,
        summary.go_semantics_axiom_count,
        summary.external_axiom_count,
        summary.total_axiom_count
    )
    .expect("writing to string cannot fail");
}

fn write_axiom_report_entry(output: &mut String, entry: &AxiomReportEntry) {
    output.push_str("{\"category\":");
    write_json_string(output, entry.category.canonical_name());
    output.push_str(",\"name\":");
    write_json_string(output, &entry.name);
    output.push_str(",\"origin_module\":");
    write_json_string(output, &entry.origin_module);
    output.push_str(",\"type_hash\":");
    write_hash(output, &entry.type_hash);
    output.push_str(",\"declaration_hash\":");
    write_hash(output, &entry.declaration_hash);
    output.push_str(",\"source_certificate_hash\":");
    write_optional_hash(output, entry.source_certificate_hash.as_ref());
    output.push_str(",\"direct_dependent_declarations\":");
    write_u32_array(output, &entry.direct_dependent_declarations);
    output.push_str(",\"transitive_dependent_declarations\":");
    write_u32_array(output, &entry.transitive_dependent_declarations);
    output.push_str(",\"approval_profile\":");
    write_optional_string(output, entry.approval_profile.as_deref());
    output.push_str(",\"reviewer_note\":");
    write_optional_string(output, entry.reviewer_note.as_deref());
    output.push('}');
}

fn write_declaration_axiom_dependencies(
    output: &mut String,
    dependencies: &DeclarationAxiomDependencies,
) {
    output.push_str("{\"declaration_name\":");
    write_json_string(output, &dependencies.declaration_name);
    output.push_str(",\"declaration_hash\":");
    write_hash(output, &dependencies.declaration_hash);
    output.push_str(",\"direct_axiom_dependencies\":");
    write_u32_array(output, &dependencies.direct_axiom_dependencies);
    output.push_str(",\"transitive_axiom_dependencies\":");
    write_u32_array(output, &dependencies.transitive_axiom_dependencies);
    output.push('}');
}

fn write_optional_hash(output: &mut String, hash: Option<&HashBytes>) {
    match hash {
        Some(hash) => write_hash(output, hash),
        None => output.push_str("null"),
    }
}

fn write_hash(output: &mut String, hash: &HashBytes) {
    write_json_string(output, &hash_hex(hash));
}

fn write_optional_string(output: &mut String, value: Option<&str>) {
    match value {
        Some(value) => write_json_string(output, value),
        None => output.push_str("null"),
    }
}

fn write_u32_array(output: &mut String, values: &[u32]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        write!(output, "{value}").expect("writing to string cannot fail");
    }
    output.push(']');
}

fn write_json_string(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            ch if ch <= '\u{1f}' => {
                write!(output, "\\u{:04x}", ch as u32).expect("writing to string cannot fail");
            }
            ch => output.push(ch),
        }
    }
    output.push('"');
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use mpk_cert::encode::{
        AxiomCategory, AxiomReport, AxiomReportEntry, AxiomReportSummary,
        DeclarationAxiomDependencies,
    };

    use crate::verifier::VerificationReport;

    use super::{
        render_axiom_report_json, render_verification_report_json,
        verify_certificate_bytes_axiom_report_json_output, verify_certificate_bytes_json,
        verify_certificate_bytes_json_output,
    };

    const CERT_BASIC_FIXTURE_DIR: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/cert-basic");
    const CERT_CANONICAL_NONCANONICAL_FIXTURE_DIR: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/cert-canonical/non-canonical"
    );

    fn decode_hex_fixture(path: &Path) -> Vec<u8> {
        let contents = fs::read_to_string(path).expect("hex fixture is readable");
        let hex = contents
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();
        assert_eq!(hex.len() % 2, 0, "fixture hex must use full bytes");

        hex.as_bytes()
            .chunks_exact(2)
            .map(|chunk| {
                let byte = std::str::from_utf8(chunk).expect("fixture hex is utf8");
                u8::from_str_radix(byte, 16).expect("fixture hex byte is valid")
            })
            .collect()
    }

    fn hash(byte: u8) -> [u8; 32] {
        [byte; 32]
    }

    #[test]
    fn accepted_output_matches_snapshot() {
        let bytes = decode_hex_fixture(&Path::new(CERT_BASIC_FIXTURE_DIR).join("zero-axiom.hex"));
        let output = verify_certificate_bytes_json_output(&bytes);

        assert!(output.accepted);
        assert_eq!(
            output.json,
            concat!(
                "{\"verdict\":\"accepted\",\"module\":\"Example.Basic.ZeroAxiom\",",
                "\"declaration_count\":0,\"axiom_count\":0,",
                "\"hashes\":{\"export\":\"0eeef32184a5c39018828814a4a293d89075e3dccfee21d4a502483ca8ed1db0\",",
                "\"axiom_report\":\"0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5\",",
                "\"certificate\":\"a5e2465d422837c4a90e3e068f2818fc5cf88f191997a3d23f4e4a4585450cdf\"},",
                "\"axiom_report\":{\"summary\":{\"core_axiom_count\":0,",
                "\"builtin_theory_axiom_count\":0,\"go_semantics_axiom_count\":0,",
                "\"external_axiom_count\":0,\"total_axiom_count\":0},",
                "\"entries\":[],\"declaration_dependencies\":[]},",
                "\"error_code\":null,\"error_detail\":null}"
            )
        );
    }

    #[test]
    fn rejected_output_matches_snapshot() {
        let bytes = decode_hex_fixture(
            &Path::new(CERT_CANONICAL_NONCANONICAL_FIXTURE_DIR).join("unsorted-name-table.hex"),
        );
        let output = verify_certificate_bytes_json_output(&bytes);

        assert!(!output.accepted);
        assert_eq!(
            output.json,
            concat!(
                "{\"verdict\":\"rejected\",\"module\":null,\"declaration_count\":null,",
                "\"axiom_count\":null,\"hashes\":{\"export\":null,\"axiom_report\":null,",
                "\"certificate\":\"a68ac215fb95b174b6c0a282032efca955208cfb2e086517fe4556ad8bcf13bd\"},",
                "\"axiom_report\":null,\"error_code\":\"KERNEL_CANONICAL_CERTIFICATE\",",
                "\"error_detail\":\"name_table: Z before A\"}"
            )
        );
        assert_eq!(verify_certificate_bytes_json(&bytes), output.json);
    }

    #[test]
    fn axiom_report_output_includes_entries_and_dependencies_snapshot() {
        let report = VerificationReport {
            module: "Example.Json".to_owned(),
            declaration_count: 2,
            axiom_count: 1,
            export_hash: hash(0x10),
            axiom_report_hash: hash(0x11),
            certificate_hash: hash(0x12),
            axiom_report: AxiomReport {
                entries: vec![AxiomReportEntry {
                    category: AxiomCategory::CoreAxiom,
                    name: "Example.Json.ax".to_owned(),
                    origin_module: "Example.Json".to_owned(),
                    type_hash: hash(0x13),
                    declaration_hash: hash(0x14),
                    source_certificate_hash: Some(hash(0x15)),
                    direct_dependent_declarations: vec![0, 2],
                    transitive_dependent_declarations: vec![0, 1, 2],
                    approval_profile: Some("bootstrap".to_owned()),
                    reviewer_note: Some("line \"one\"\nline two".to_owned()),
                }],
                declaration_dependencies: vec![DeclarationAxiomDependencies {
                    declaration_name: "Example.Json.thm".to_owned(),
                    declaration_hash: hash(0x16),
                    direct_axiom_dependencies: vec![0],
                    transitive_axiom_dependencies: vec![0, 3],
                }],
                summary: AxiomReportSummary {
                    core_axiom_count: 1,
                    builtin_theory_axiom_count: 0,
                    go_semantics_axiom_count: 0,
                    external_axiom_count: 0,
                    total_axiom_count: 1,
                },
            },
        };

        assert_eq!(
            render_verification_report_json(&report),
            concat!(
                "{\"verdict\":\"accepted\",\"module\":\"Example.Json\",",
                "\"declaration_count\":2,\"axiom_count\":1,",
                "\"hashes\":{\"export\":\"1010101010101010101010101010101010101010101010101010101010101010\",",
                "\"axiom_report\":\"1111111111111111111111111111111111111111111111111111111111111111\",",
                "\"certificate\":\"1212121212121212121212121212121212121212121212121212121212121212\"},",
                "\"axiom_report\":{\"summary\":{\"core_axiom_count\":1,",
                "\"builtin_theory_axiom_count\":0,\"go_semantics_axiom_count\":0,",
                "\"external_axiom_count\":0,\"total_axiom_count\":1},",
                "\"entries\":[{\"category\":\"CoreAxiom\",\"name\":\"Example.Json.ax\",",
                "\"origin_module\":\"Example.Json\",",
                "\"type_hash\":\"1313131313131313131313131313131313131313131313131313131313131313\",",
                "\"declaration_hash\":\"1414141414141414141414141414141414141414141414141414141414141414\",",
                "\"source_certificate_hash\":\"1515151515151515151515151515151515151515151515151515151515151515\",",
                "\"direct_dependent_declarations\":[0,2],",
                "\"transitive_dependent_declarations\":[0,1,2],",
                "\"approval_profile\":\"bootstrap\",",
                "\"reviewer_note\":\"line \\\"one\\\"\\nline two\"}],",
                "\"declaration_dependencies\":[{\"declaration_name\":\"Example.Json.thm\",",
                "\"declaration_hash\":\"1616161616161616161616161616161616161616161616161616161616161616\",",
                "\"direct_axiom_dependencies\":[0],",
                "\"transitive_axiom_dependencies\":[0,3]}]},",
                "\"error_code\":null,\"error_detail\":null}"
            )
        );
        assert_eq!(
            render_axiom_report_json(&report),
            concat!(
                "{\"certificate_hash\":\"1212121212121212121212121212121212121212121212121212121212121212\",",
                "\"axiom_report_hash\":\"1111111111111111111111111111111111111111111111111111111111111111\",",
                "\"axiom_report\":{\"summary\":{\"core_axiom_count\":1,",
                "\"builtin_theory_axiom_count\":0,\"go_semantics_axiom_count\":0,",
                "\"external_axiom_count\":0,\"total_axiom_count\":1},",
                "\"entries\":[{\"category\":\"CoreAxiom\",\"name\":\"Example.Json.ax\",",
                "\"origin_module\":\"Example.Json\",",
                "\"type_hash\":\"1313131313131313131313131313131313131313131313131313131313131313\",",
                "\"declaration_hash\":\"1414141414141414141414141414141414141414141414141414141414141414\",",
                "\"source_certificate_hash\":\"1515151515151515151515151515151515151515151515151515151515151515\",",
                "\"direct_dependent_declarations\":[0,2],",
                "\"transitive_dependent_declarations\":[0,1,2],",
                "\"approval_profile\":\"bootstrap\",",
                "\"reviewer_note\":\"line \\\"one\\\"\\nline two\"}],",
                "\"declaration_dependencies\":[{\"declaration_name\":\"Example.Json.thm\",",
                "\"declaration_hash\":\"1616161616161616161616161616161616161616161616161616161616161616\",",
                "\"direct_axiom_dependencies\":[0],",
                "\"transitive_axiom_dependencies\":[0,3]}]}}"
            )
        );
    }

    #[test]
    fn axiom_report_command_output_matches_snapshot() {
        let bytes = decode_hex_fixture(&Path::new(CERT_BASIC_FIXTURE_DIR).join("one-theorem.hex"));
        let output = verify_certificate_bytes_axiom_report_json_output(&bytes);

        assert!(output.accepted);
        assert_eq!(
            output.json,
            concat!(
                "{\"certificate_hash\":\"37744c27174b7637485f6c005902dbf72604641ba66e2ebec90795eaddde1e94\",",
                "\"axiom_report_hash\":\"0ebc281c3a8d37e2d1a9ce033773e2865f96a13186a6364cb3446204c6a990d5\",",
                "\"axiom_report\":{\"summary\":{\"core_axiom_count\":0,",
                "\"builtin_theory_axiom_count\":0,\"go_semantics_axiom_count\":0,",
                "\"external_axiom_count\":0,\"total_axiom_count\":0},",
                "\"entries\":[],\"declaration_dependencies\":[]}}"
            )
        );
    }
}
