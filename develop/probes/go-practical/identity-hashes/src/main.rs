//! Untrusted identity-hash observer using the unchanged Rust canonicalizer.
#[allow(dead_code)]
#[path = "../../../../../crates/mpk-vc/src/canonical_json.rs"]
mod canonical_json;
use canonical_json::{canonical_json_bytes, parse_strict_json, StrictJsonLimits};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Read};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    id: String,
    domain: String,
    raw_json_hex: String,
}
#[derive(Serialize)]
struct Output {
    id: String,
    accepted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<String>,
}
fn hex(raw: &[u8]) -> String {
    raw.iter().map(|b| format!("{b:02x}")).collect()
}
fn domain_ok(domain: &str) -> bool {
    let Some((prefix, version)) = domain.rsplit_once('-') else {
        return false;
    };
    let Some((major, minor)) = version.split_once('.') else {
        return false;
    };
    prefix
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_uppercase)
        && prefix
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'-')
        && !major.is_empty()
        && !minor.is_empty()
        && major.bytes().all(|b| b.is_ascii_digit())
        && minor.bytes().all(|b| b.is_ascii_digit())
}
fn main() {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).unwrap();
    let inputs: Vec<Input> = serde_json::from_str(&raw).unwrap();
    let output = inputs
        .into_iter()
        .map(|row| {
            let bytes = (0..row.raw_json_hex.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&row.raw_json_hex[i..i + 2], 16).unwrap())
                .collect::<Vec<_>>();
            let canonical = if domain_ok(&row.domain) {
                parse_strict_json(&bytes, StrictJsonLimits::new(1 << 20, 131072, 64, 1 << 20))
                    .ok()
                    .and_then(|value| canonical_json_bytes(&value).ok())
            } else {
                None
            };
            let digest = canonical.as_ref().map(|value| {
                let mut h = Sha256::new();
                h.update(row.domain.as_bytes());
                h.update([0]);
                h.update(value);
                hex(&h.finalize())
            });
            Output {
                id: row.id,
                accepted: canonical.is_some(),
                canonical_hex: canonical.as_ref().map(|v| hex(v)),
                digest,
            }
        })
        .collect::<Vec<_>>();
    println!("{}", serde_json::to_string(&output).unwrap());
}
