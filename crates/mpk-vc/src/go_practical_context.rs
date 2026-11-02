//! Closed, inactive Go/Rust candidate configuration. The pinned foundation
//! descriptor is a design candidate, not a complete checked foundation or a
//! registered runtime profile. These values cannot establish a source theorem.

use crate::canonical_json::{
    canonical_json_bytes, parse_strict_json, StrictJsonLimits, StrictJsonValue,
};
use crate::go_practical_selection::{import_go_practical_selection, GoPracticalSelection};
use crate::hash::{hash_canonical_json, HashDomain};
use crate::semantic_profile_registry::{
    validate_registry_selection_envelope, validate_registry_semantic_context,
    validate_semantic_profile_registry, RegistryRevision, SelectionEnvelope, SemanticContext,
    SemanticProfileEntry, ValidatedSemanticProfileRegistry,
};
use serde_json::{json, Value};
use std::fmt;

const REGISTRY_SCHEMA: &str = "mpk.semantic_profile.registry.v3";
const ENTRY_SCHEMA: &str = "mpk.semantic_profile.entry.v3";
const CONTEXT_SCHEMA: &str = "mpk.semantic_context.v3";
const REQUEST_SCHEMA: &str = "mpk.validated_semantic_request.v3";
const GO_PROFILE: &str = "mpk.go.practical.v1";
const GO_PARAMETERS: &str = "mpk.semantic_parameters.go_practical.v1";
const GO_SELECTION: &str = "mpk.selection.go_functions.v1";
pub(crate) const PRACTICAL_EFFECTIVE_LANGUAGES: [&str; 4] =
    ["go1.22", "go1.23", "go1.24", "go1.25"];
const LIMITS_HASH: &str = "49ab79bb15643fa34b5b0d9b15ce4c833d098e38b70e89f0af3206bdac570551";
const FOUNDATION_HASH: &str = "7b038ab4a8fcac4079bdb2228dd6345c8489224233fec81544e2aa98283a225d";
const CANDIDATE_REGISTRY_HASH: &str =
    "6c300215b21893014f393940416cebc7aa83ac74443918db5fa33a7c7615a7bc";
const GO_ENTRY_HASH: &str = "0c15131de1279800ee5ee58c4c9a45f9a423e09b1721d9ba198868aa79fe5f9a";
const FIXED_ENTRY_HASH: &str = "ccff8b793a2ed6cbdfd1fbb534216d5f75c87a172b931d0397c426f5e3bee70c";
const RUST_ENTRY_HASH: &str = "05dfce894254c6e0b4b72aceb98ab0da8869eafcf45fa3331e3de5f67f4e4508";
const ENTRY_DOMAIN: HashDomain = HashDomain::new("MPK-SEMANTIC-PROFILE-ENTRY-3.0");
const REGISTRY_DOMAIN: HashDomain = HashDomain::new("MPK-SEMANTIC-PROFILE-REGISTRY-3.0");
const REQUEST_DOMAIN: HashDomain = HashDomain::new("MPK-VALIDATED-SEMANTIC-REQUEST-3.0");
const INTERNAL_LIMITS: StrictJsonLimits = StrictJsonLimits::new(524_288, 524_288, 32, 65_536);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoPrivateContextError {
    Json,
    Registry,
    Shape,
    Context,
    Parameters,
    Selection,
    RequestHash,
    Linkage,
}

impl fmt::Display for GoPrivateContextError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "inactive Go/Rust candidate context: {self:?}")
    }
}
impl std::error::Error for GoPrivateContextError {}
type Result<T> = std::result::Result<T, GoPrivateContextError>;

fn strict_value(value: &Value) -> Result<StrictJsonValue> {
    let bytes = serde_json::to_vec(value).map_err(|_| GoPrivateContextError::Json)?;
    parse_strict_json(&bytes, INTERNAL_LIMITS).map_err(|_| GoPrivateContextError::Json)
}

fn seal(value: &mut Value, field: &str, domain: HashDomain) -> Result<()> {
    let digest = hash_canonical_json(domain, &strict_value(value)?)
        .map_err(|_| GoPrivateContextError::Json)?
        .to_hex();
    value
        .as_object_mut()
        .ok_or(GoPrivateContextError::Shape)?
        .insert(field.to_owned(), Value::String(digest));
    Ok(())
}

fn contracts(suffix: &str) -> Value {
    let mut contracts = serde_json::Map::new();
    for name in [
        "ai",
        "evidence",
        "frontend",
        "manifest",
        "policy",
        "release",
        "source_map",
        "vc",
        "vir",
    ] {
        contracts.insert(
            name.to_owned(),
            json!(format!("mpk.profile.{name}.{suffix}.v2")),
        );
    }
    Value::Object(contracts)
}

fn retained_entry(entry: &SemanticProfileEntry, suffix: &str) -> Result<Value> {
    let mut value = json!({
        "schema": ENTRY_SCHEMA,
        "source_language": entry.source_language(),
        "semantic_profile": entry.semantic_profile(),
        "semantic_parameters_schema": entry.semantic_parameters_schema(),
        "selection_schema": entry.selection_schema(),
        "foundation_descriptor": null,
        "contracts": contracts(suffix),
    });
    seal(&mut value, "entry_sha256", ENTRY_DOMAIN)?;
    Ok(value)
}

fn candidate_foundation() -> Value {
    json!({
        "schema": "mpk.go.foundation_descriptor.v1",
        "id": "mpk.go.practical.foundation.v1",
        "content_sha256": FOUNDATION_HASH,
    })
}

fn practical_entry() -> Result<Value> {
    let mut value = json!({
        "schema": ENTRY_SCHEMA,
        "source_language": "go",
        "semantic_profile": GO_PROFILE,
        "semantic_parameters_schema": GO_PARAMETERS,
        "selection_schema": GO_SELECTION,
        "foundation_descriptor": candidate_foundation(),
        "contracts": contracts("go_practical"),
    });
    seal(&mut value, "entry_sha256", ENTRY_DOMAIN)?;
    Ok(value)
}

fn practical_parameters() -> Value {
    json!({
        "target_id": "linux/amd64",
        "pointer_width": 64,
        "goos": "linux",
        "goarch": "amd64",
        "goamd64": "v1",
        "compiler_release": "go1.25.0",
        "effective_languages": PRACTICAL_EFFECTIVE_LANGUAGES,
        "limits_id": "mpk.go.practical.limits.v1",
        "limits_sha256": LIMITS_HASH,
    })
}

fn exact(value: &Value, fields: &[&str]) -> bool {
    value.as_object().is_some_and(|object| {
        object.len() == fields.len() && fields.iter().all(|field| object.contains_key(*field))
    })
}

fn value_and_canonical(value: &StrictJsonValue) -> Result<(Value, Vec<u8>)> {
    let canonical = canonical_json_bytes(value).map_err(|_| GoPrivateContextError::Json)?;
    let parsed = serde_json::from_slice(&canonical).map_err(|_| GoPrivateContextError::Json)?;
    Ok((parsed, canonical))
}

/// The registry is generated only from the compiled Go/Rust revision-4
/// registry and the frozen candidate descriptor. It cannot be constructed
/// from a caller's registry, foundation, policy adapter or hash alone.
pub struct GoPrivateCandidateRegistry {
    retained: ValidatedSemanticProfileRegistry,
    registry: Value,
    reference: Value,
    canonical: Vec<u8>,
}

impl GoPrivateCandidateRegistry {
    pub fn new() -> Result<Self> {
        use GoPrivateContextError as E;
        let retained = validate_semantic_profile_registry(
            include_bytes!("../../../release/bundles/semantic-profile-registry.json"),
            RegistryRevision::Revision4,
        )
        .map_err(|_| E::Registry)?;
        if retained.entries().len() != 2 {
            return Err(E::Registry);
        }
        let fixed = retained
            .lookup("go", "mpk.go.fixed.v0")
            .ok_or(E::Registry)?;
        let rust = retained
            .lookup("rust", "mpk.rust.checked.v0")
            .ok_or(E::Registry)?;
        let entries = vec![
            retained_entry(fixed, "go_fixed")?,
            practical_entry()?,
            retained_entry(rust, "rust_checked")?,
        ];
        if entries[0]["entry_sha256"] != FIXED_ENTRY_HASH
            || entries[1]["entry_sha256"] != GO_ENTRY_HASH
            || entries[2]["entry_sha256"] != RUST_ENTRY_HASH
        {
            return Err(E::Registry);
        }
        let mut registry = json!({
            "schema": REGISTRY_SCHEMA,
            "id": REGISTRY_SCHEMA,
            "revision": 1,
            "profiles": entries,
        });
        seal(&mut registry, "registry_sha256", REGISTRY_DOMAIN)?;
        if registry["registry_sha256"] != CANDIDATE_REGISTRY_HASH {
            return Err(E::Registry);
        }
        let reference = json!({
            "schema": REGISTRY_SCHEMA,
            "id": REGISTRY_SCHEMA,
            "revision": 1,
            "registry_sha256": CANDIDATE_REGISTRY_HASH,
        });
        let canonical = canonical_json_bytes(&strict_value(&registry)?).map_err(|_| E::Json)?;
        Ok(Self {
            retained,
            registry,
            reference,
            canonical,
        })
    }

    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    /// Even a correctly rehashed caller-supplied registry is not authority.
    pub fn import_registry(&self, value: &StrictJsonValue) -> Result<()> {
        let (_, canonical) = value_and_canonical(value)?;
        if canonical != self.canonical {
            return Err(GoPrivateContextError::Registry);
        }
        Ok(())
    }

    pub fn import_context(&self, value: &StrictJsonValue) -> Result<GoPrivateCandidateContext> {
        use GoPrivateContextError as E;
        let (wire, canonical) = value_and_canonical(value)?;
        if !exact(
            &wire,
            &[
                "schema",
                "profile_registry",
                "profile_entry_sha256",
                "source_language",
                "semantic_profile",
                "semantic_parameters",
                "foundation_descriptor",
            ],
        ) || wire["schema"] != CONTEXT_SCHEMA
            || wire["profile_registry"] != self.reference
        {
            return Err(E::Shape);
        }
        let source_language = wire["source_language"].as_str().ok_or(E::Context)?;
        let profile = wire["semantic_profile"].as_str().ok_or(E::Context)?;
        let entry = self.registry["profiles"]
            .as_array()
            .and_then(|entries| {
                entries.iter().find(|entry| {
                    entry["source_language"] == source_language
                        && entry["semantic_profile"] == profile
                })
            })
            .ok_or(E::Context)?;
        if wire["profile_entry_sha256"] != entry["entry_sha256"]
            || wire["foundation_descriptor"] != entry["foundation_descriptor"]
            || !exact(&wire["semantic_parameters"], &["schema", "value"])
            || wire["semantic_parameters"]["schema"] != entry["semantic_parameters_schema"]
        {
            return Err(E::Context);
        }
        let retained_context = if profile == GO_PROFILE {
            if source_language != "go"
                || wire["semantic_parameters"]["value"] != practical_parameters()
            {
                return Err(E::Parameters);
            }
            None
        } else {
            let old = self
                .retained
                .lookup(source_language, profile)
                .ok_or(E::Context)?;
            let projection = json!({
                "profile_registry": self.retained.identity(),
                "profile_entry_sha256": old.entry_sha256(),
                "source_language": source_language,
                "semantic_profile": profile,
                "semantic_parameters": wire["semantic_parameters"],
            });
            Some(
                validate_registry_semantic_context(&self.retained, &projection)
                    .map_err(|_| E::Parameters)?,
            )
        };
        Ok(GoPrivateCandidateContext {
            wire,
            canonical,
            retained_context,
        })
    }

    pub fn import_request(&self, value: &StrictJsonValue) -> Result<GoPrivateCandidateRequest> {
        use GoPrivateContextError as E;
        let (wire, canonical) = value_and_canonical(value)?;
        if !exact(
            &wire,
            &["schema", "semantic_context", "selection", "request_sha256"],
        ) || wire["schema"] != REQUEST_SCHEMA
        {
            return Err(E::Shape);
        }
        let raw_context = value.get("semantic_context").ok_or(E::Shape)?;
        let context = self.import_context(raw_context)?;
        let raw_selection = value.get("selection").ok_or(E::Shape)?;
        let selection = match &context.retained_context {
            None => GoPrivateCandidateSelection::Practical(
                import_go_practical_selection(raw_selection).map_err(|_| E::Selection)?,
            ),
            Some(retained) => {
                let (wire_selection, _) = value_and_canonical(raw_selection)?;
                GoPrivateCandidateSelection::Retained(
                    validate_registry_selection_envelope(&self.retained, retained, &wire_selection)
                        .map_err(|_| E::Selection)?,
                )
            }
        };
        let preimage = value
            .clone_without_fields(&["request_sha256"])
            .map_err(|_| E::Shape)?;
        let digest = hash_canonical_json(REQUEST_DOMAIN, &preimage)
            .map_err(|_| E::Json)?
            .to_hex();
        if wire["request_sha256"].as_str() != Some(digest.as_str()) {
            return Err(E::RequestHash);
        }
        Ok(GoPrivateCandidateRequest {
            context,
            selection,
            canonical,
            sha256: digest,
        })
    }
}

/// Configuration only. The candidate foundation reference is not a checked
/// foundation, installed profile, source capture or program proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoPrivateCandidateContext {
    wire: Value,
    canonical: Vec<u8>,
    retained_context: Option<SemanticContext>,
}

impl GoPrivateCandidateContext {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub fn semantic_profile(&self) -> &str {
        self.wire["semantic_profile"]
            .as_str()
            .expect("validated candidate profile")
    }
    pub fn require_same(&self, other: &Self) -> Result<()> {
        if self.canonical != other.canonical {
            return Err(GoPrivateContextError::Linkage);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoPrivateCandidateSelection {
    Practical(GoPracticalSelection),
    Retained(SelectionEnvelope),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoPrivateCandidateRequest {
    context: GoPrivateCandidateContext,
    selection: GoPrivateCandidateSelection,
    canonical: Vec<u8>,
    sha256: String,
}
impl GoPrivateCandidateRequest {
    pub fn context(&self) -> &GoPrivateCandidateContext {
        &self.context
    }
    pub fn selection(&self) -> &GoPrivateCandidateSelection {
        &self.selection
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }
    pub fn request_sha256(&self) -> &str {
        &self.sha256
    }
}
