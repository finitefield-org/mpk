#![no_main]

use libfuzzer_sys::fuzz_target;

use mpk_cert::{decode_certificate, validate_canonical_certificate};
use mpk_kernel::{verify_certificate_bytes, verify_certificate_bytes_json_output};

fuzz_target!(|data: &[u8]| {
    let _ = decode_certificate(data);
    let _ = validate_canonical_certificate(data);
    let _ = verify_certificate_bytes(data);
    let _ = verify_certificate_bytes_json_output(data);
});
