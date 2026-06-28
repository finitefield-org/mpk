use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use mpk_cert::{decode_certificate, validate_canonical_certificate};
use mpk_kernel::{verify_certificate_bytes, verify_certificate_bytes_json_output};

const CERT_BASIC_FIXTURE_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/cert-basic");
const CERT_CANONICAL_NONCANONICAL_FIXTURE_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/cert-canonical/non-canonical"
);
const CERT_DECODE_INVALID_FIXTURE_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/cert-decode/invalid"
);

#[test]
fn certificate_decoder_boundary_inputs_do_not_panic() {
    for (name, bytes) in malformed_boundary_cases() {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let _ = decode_certificate(&bytes);
            let _ = validate_canonical_certificate(&bytes);
            let _ = verify_certificate_bytes(&bytes);
            let _ = verify_certificate_bytes_json_output(&bytes);
        }));

        assert!(outcome.is_ok(), "boundary case `{name}` panicked");
    }
}

fn malformed_boundary_cases() -> Vec<(String, Vec<u8>)> {
    let mut cases = Vec::new();
    cases.extend(binary_edge_cases());
    cases.extend(fixture_cases(CERT_DECODE_INVALID_FIXTURE_DIR));
    cases.extend(fixture_cases(CERT_CANONICAL_NONCANONICAL_FIXTURE_DIR));

    let valid = read_hex_fixture(&Path::new(CERT_BASIC_FIXTURE_DIR).join("one-theorem.hex"));
    cases.extend(mutated_cases("one-theorem", &valid));
    cases
}

fn binary_edge_cases() -> Vec<(String, Vec<u8>)> {
    vec![
        ("empty".to_owned(), Vec::new()),
        ("single-zero".to_owned(), vec![0]),
        ("single-ff".to_owned(), vec![0xff]),
        ("varint-overflow-prefix".to_owned(), vec![0xff; 32]),
        ("magic-only".to_owned(), b"MPKCERT".to_vec()),
        (
            "magic-with-overlong-format-len".to_owned(),
            [b"MPKCERT".as_slice(), &[0xff; 16]].concat(),
        ),
        ("ascii-noise".to_owned(), b"not a certificate".to_vec()),
    ]
}

fn fixture_cases(directory: &str) -> Vec<(String, Vec<u8>)> {
    let mut entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("fixture directory {directory} is readable: {error}"))
        .map(|entry| entry.expect("fixture directory entry is readable").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "hex"))
        .collect::<Vec<_>>();
    entries.sort();

    entries
        .into_iter()
        .map(|path| {
            let name = path
                .file_name()
                .expect("fixture path has a file name")
                .to_string_lossy()
                .into_owned();
            (name, read_hex_fixture(&path))
        })
        .collect()
}

fn mutated_cases(prefix: &str, bytes: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut cases = Vec::new();

    for len in [0, 1, 2, 3, 4, 7, 8, 15, 16, 31, 32, bytes.len() / 2] {
        if len < bytes.len() {
            cases.push((format!("{prefix}-truncated-{len}"), bytes[..len].to_vec()));
        }
    }

    for index in selected_indices(bytes.len()) {
        let mut mutated = bytes.to_vec();
        mutated[index] ^= 0xff;
        cases.push((format!("{prefix}-flip-{index}"), mutated));
    }

    for index in selected_indices(bytes.len()) {
        let mut mutated = bytes.to_vec();
        mutated.insert(index, 0xff);
        cases.push((format!("{prefix}-insert-{index}"), mutated));
    }

    let mut trailing = bytes.to_vec();
    trailing.extend_from_slice(&[0xff, 0x80, 0x00]);
    cases.push((format!("{prefix}-trailing-noise"), trailing));

    cases
}

fn selected_indices(len: usize) -> Vec<usize> {
    let mut indices = [
        0,
        1,
        2,
        3,
        4,
        7,
        8,
        15,
        16,
        len / 4,
        len / 2,
        len.saturating_sub(1),
    ]
    .into_iter()
    .filter(|index| *index < len)
    .collect::<Vec<_>>();
    indices.sort_unstable();
    indices.dedup();
    indices
}

fn read_hex_fixture(path: &Path) -> Vec<u8> {
    let contents = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("hex fixture {} is readable: {error}", path.display()));
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
