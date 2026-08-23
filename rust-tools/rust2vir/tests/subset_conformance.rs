use rust2vir_internal::json::{self, JsonValue};
use rust2vir_internal::limits::{
    checked_add, validate_limit, RustLimitError, RustLimitId, RUST_LIMIT_PROFILE_ID,
};
use std::collections::{BTreeMap, BTreeSet};

// The locked launcher mounts only the packaged frontend snapshot. The repository
// coverage checker separately requires this hermetic mirror to be byte-identical
// to develop/specs/vectors/rust-subset-v0.json.
const NORMATIVE_VECTOR: &[u8] = include_bytes!("../testdata/rust-subset-v0.json");

#[test]
fn every_normative_limit_row_executes_the_registered_checked_boundary() {
    let vector = json::parse(NORMATIVE_VECTOR, NORMATIVE_VECTOR.len()).unwrap();
    let root = vector.as_object().unwrap();
    assert_eq!(text(root, "schema"), "mpk.rust.subset.conformance.v0");
    assert_eq!(text(root, "spec_schema"), "mpk.rust.checked.v0");
    assert_eq!(
        text(root, "owner_test"),
        "rust-tools/rust2vir/tests/subset_conformance.rs"
    );
    assert_eq!(
        text(root["profiles"].as_object().unwrap(), "limit_profile"),
        RUST_LIMIT_PROFILE_ID
    );

    let rows = root["limit_boundaries"].as_array().unwrap();
    assert_eq!(rows.len(), RustLimitId::ALL.len());
    let mut consumed = BTreeSet::new();

    for (index, row) in rows.iter().enumerate() {
        let row = row.as_object().unwrap();
        assert_eq!(
            row.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            BTreeSet::from(["above", "at", "id", "maximum", "unit"]),
            "limit row {index} is not closed"
        );
        let id = text(row, "id");
        assert!(consumed.insert(id), "duplicate vector limit {id}");
        let limit = RustLimitId::try_from(id).expect("vector limit must be registered");
        assert_eq!(
            limit,
            RustLimitId::ALL[index],
            "registry/vector order drift"
        );
        assert_eq!(text(row, "unit"), limit.unit(), "{id} unit drift");
        assert_eq!(text(row, "at"), "accept", "{id} at-boundary action");
        assert_eq!(
            text(row, "above"),
            limit.above_action(),
            "{id} above-boundary action"
        );

        let at = u64::try_from(row["maximum"].integer().unwrap()).unwrap();
        assert_eq!(at, limit.maximum(), "{id} maximum drift");
        let below = at
            .checked_sub(1)
            .expect("all normative maxima are positive");
        let above = at.checked_add(1).expect("normative maxima fit in u64");

        assert_eq!(validate_limit(limit, below), Ok(()), "{id} below");
        assert_eq!(validate_limit(limit, at), Ok(()), "{id} at");
        let exceeded = RustLimitError::Exceeded {
            limit,
            observed: above,
        };
        assert_eq!(
            validate_limit(limit, above),
            Err(exceeded.clone()),
            "{id} above"
        );
        assert_eq!(exceeded.action(), Some(text(row, "above")));
        assert_eq!(checked_add(limit, below, 1), Ok(at), "{id} checked at");
        assert_eq!(
            checked_add(limit, at, 1),
            Err(exceeded),
            "{id} checked above"
        );
        let overflow = RustLimitError::CounterOverflow { limit };
        assert_eq!(
            checked_add(limit, u64::MAX, 1),
            Err(overflow.clone()),
            "{id} checked overflow"
        );
        assert_eq!(overflow.action(), Some(text(row, "above")));
    }

    assert_eq!(consumed.len(), 35);
    assert_eq!(
        consumed,
        RustLimitId::ALL
            .into_iter()
            .map(RustLimitId::as_str)
            .collect::<BTreeSet<_>>()
    );
}

fn text<'a>(object: &'a BTreeMap<String, JsonValue>, field: &str) -> &'a str {
    object[field].as_str().unwrap()
}
