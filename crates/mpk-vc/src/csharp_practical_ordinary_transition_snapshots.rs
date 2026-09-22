//! Complete source-value equality for W08 retained command/context snapshots.
//! Equality includes all stored source members, even fields omitted by semantic
//! bindings. Input/public domains remain obligations. This does not prove the
//! source equality helper or serialize canonical JSON or prove application/replay.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryTransitionSnapshotDefinition {
    pub source: crate::csharp_practical_vir_model::TransitionSnapshotNode,
    pub carrier: OrdinaryCarrier,
    /// W08 constant resolved by this ordinary definition.
    pub symbol: String,
    pub equality_definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryTransitionSnapshotEncodingEquality {
    /// Exact W08 predicate, scoped by the transition contract hash.
    pub symbol: String,
    /// command, context, retained command, retained context.
    pub argument_type_ids: Vec<String>,
    pub command_equality_definition: String,
    pub context_equality_definition: String,
    pub definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryTransitionHistoryCapacity {
    /// Exact W08 capacity predicate, scoped by the transition contract hash.
    pub symbol: String,
    pub history_type_id: String,
    pub history_carrier: OrdinaryCarrier,
    pub length_definition: String,
    pub definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryTransitionRetainedHistory {
    pub contract_sha256: String,
    pub state_type_id: String,
    pub command_type_id: String,
    pub context_type_id: String,
    pub history_type_id: String,
    pub record_type_id: String,
    pub key_type_id: String,
    pub state_history_member_id: String,
    pub state_history_member_ordinal: usize,
    pub command_key_member_id: String,
    pub command_key_member_ordinal: usize,
    pub record_key_member_id: String,
    pub record_key_member_ordinal: usize,
    pub record_command_member_id: String,
    pub record_command_member_ordinal: usize,
    pub record_context_member_id: String,
    pub record_context_member_ordinal: usize,
    pub record_response_member_id: String,
    pub record_response_member_ordinal: usize,
    pub response_type_id: String,
    pub retained_record_symbol: String,
    pub retained_record_definition: String,
    pub retained_key_present_symbol: String,
    pub retained_key_present_definition: String,
    pub retained_keys_unique_symbol: String,
    pub retained_keys_unique_definition: String,
    pub append_complete_snapshot_symbol: String,
    pub append_complete_snapshot_definition: String,
    pub preserve_retained_history_order_symbol: String,
    pub preserve_retained_history_order_definition: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryTransitionSnapshotProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    transition_vc_sha256: String,
    definitions: Vec<OrdinaryTransitionSnapshotDefinition>,
    encoding_equalities: Vec<OrdinaryTransitionSnapshotEncodingEquality>,
    history_capacities: Vec<OrdinaryTransitionHistoryCapacity>,
    retained_histories: Vec<OrdinaryTransitionRetainedHistory>,
    /// All other W08 constants remain obligations of later transition assembly.
    pending_definition_names: Vec<String>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryTransitionSnapshotProgram {
    pub fn definitions(&self) -> &[OrdinaryTransitionSnapshotDefinition] {
        &self.definitions
    }
    pub fn pending_definition_names(&self) -> &[String] {
        &self.pending_definition_names
    }
    pub fn encoding_equalities(&self) -> &[OrdinaryTransitionSnapshotEncodingEquality] {
        &self.encoding_equalities
    }
    pub fn history_capacities(&self) -> &[OrdinaryTransitionHistoryCapacity] {
        &self.history_capacities
    }
    pub fn retained_histories(&self) -> &[OrdinaryTransitionRetainedHistory] {
        &self.retained_histories
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed transition snapshot program")
    }
}

fn application(term: &ContractTerm) -> Option<(&str, Vec<&ContractTerm>)> {
    let mut term = term;
    let mut arguments = vec![];
    while let ContractTerm::App {
        function, argument, ..
    } = term
    {
        arguments.push(argument.as_ref());
        term = function.as_ref();
    }
    let ContractTerm::Const { name, .. } = term else {
        return None;
    };
    arguments.reverse();
    Some((name, arguments))
}

fn collect_applications<'a>(
    term: &'a ContractTerm,
    name: &str,
    found: &mut Vec<Vec<&'a ContractTerm>>,
) {
    if let Some((actual, arguments)) = application(term) {
        if actual == name {
            found.push(arguments);
        }
    }
    match term {
        ContractTerm::App {
            function, argument, ..
        } => {
            collect_applications(function, name, found);
            collect_applications(argument, name, found);
        }
        ContractTerm::Lam { body, .. } => collect_applications(body, name, found),
        ContractTerm::Let { value, body, .. } => {
            collect_applications(value, name, found);
            collect_applications(body, name, found);
        }
        ContractTerm::Var { .. } | ContractTerm::Const { .. } => {}
    }
}

fn member_id(term: &ContractTerm) -> R<&str> {
    let (name, arguments) = application(term).ok_or(OrdinaryCarrierError::Linkage)?;
    if arguments.len() != 1 {
        return Err(OrdinaryCarrierError::Linkage);
    }
    name.strip_prefix("Mpk.CSharp.Transition.Member.")
        .ok_or(OrdinaryCarrierError::Linkage)
}

fn index_word(b: &mut Builder, bits: u32) -> R<u32> {
    let mut body = bit(b, false)?;
    for i in 0..bits {
        let at = equal_address(b, 5, 0, 5, i)?;
        let value = b.var(5 + bits - 1 - i)?;
        body = mux(b, at, value, body)?;
    }
    b.wrap_selectors(5, body)
}

fn product_member_definition(
    relations: &mut Relations<'_>,
    type_id: &str,
    member_id: &str,
) -> R<String> {
    let carrier = relations
        .carriers
        .get(type_id)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .clone();
    let carriers = relations
        .carriers
        .iter()
        .map(|(id, carrier)| (id.as_str(), carrier))
        .collect::<BTreeMap<_, _>>();
    let generated = relations
        .storage
        .get(&mut relations.b, &carrier, &carriers)?
        .ok_or(OrdinaryCarrierError::Shape)?;
    let OrdinaryStructuralOperations::Product { operations } = generated.operations else {
        return Err(OrdinaryCarrierError::Shape);
    };
    operations
        .fields
        .iter()
        .find(|field| field.field_id == member_id)
        .map(|field| field.definition.clone())
        .ok_or(OrdinaryCarrierError::Linkage)
}

fn conjunction(b: &mut Builder, leaves: impl IntoIterator<Item = u32>) -> R<u32> {
    let mut result = bit(b, true)?;
    for leaf in leaves {
        result = call(b, "Std.Bool.and", vec![result, leaf])?;
    }
    Ok(result)
}

pub fn generate_csharp_practical_ordinary_transition_snapshots(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryTransitionSnapshotProgram> {
    let source = crate::csharp_practical_vir_model::generate_transition_vcs(vir)
        .map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut relations = Relations {
        vir,
        shared_folds: true,
        observations: false,
        carriers: layouts
            .carriers()
            .iter()
            .map(|c| (c.type_id.clone(), c.clone()))
            .collect(),
        b: Builder::new()?,
        nodes: BTreeMap::new(),
        active: BTreeSet::new(),
        raw: BTreeMap::new(),
        special: BTreeMap::new(),
        storage: StorageCache::default(),
    };
    relations.b.helpers(5)?;
    let mut definitions = vec![];
    let (bundle, roots, _) = vir.construction_context();
    for snapshot in source.snapshots() {
        let id = &snapshot.type_id;
        let dag = generate_structural_program(bundle, roots, vir.data_closed(), id)
            .map_err(|_| OrdinaryCarrierError::Linkage)?;
        if !dag.is_total() {
            return Err(OrdinaryCarrierError::Shape);
        }
        let recipe = dag.recipes().get(id).ok_or(OrdinaryCarrierError::Linkage)?;
        let members = roots
            .source_types
            .get(id)
            .map(|t| t.members.iter().map(|m| m.id.clone()).collect::<Vec<_>>())
            .unwrap_or_default();
        if snapshot.rule != recipe.rule
            || snapshot.children != recipe.children
            || snapshot.member_ids != members
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let carrier = relations
            .carriers
            .get(id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        let node = relations.source_observation(id)?;
        definitions.push(OrdinaryTransitionSnapshotDefinition {
            source: snapshot.clone(),
            carrier,
            symbol: format!("Mpk.CSharp.Transition.SourceEqual.{id}"),
            equality_definition: node.equal,
        });
    }
    let mut encoding_equalities = vec![];
    for sequent in source
        .sequents()
        .iter()
        .filter(|s| s.kind == "snapshot_helper_equivalence")
    {
        if sequent.subjects.len() != 4
            || sequent.subjects[0].id != "command"
            || sequent.subjects[1].id != "context"
            || sequent.subjects[2].id != "retained_command"
            || sequent.subjects[3].id != "retained_context"
            || sequent.subjects[0].type_id != sequent.subjects[2].type_id
            || sequent.subjects[1].type_id != sequent.subjects[3].type_id
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let suffix = sequent
            .id
            .strip_prefix("transition.snapshot_helper_equivalence.")
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let symbol = format!("Mpk.CSharp.Transition.CanonicalFieldEncodingsEqual.{suffix}");
        if !source.definition_names().contains(&symbol) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let command = relations.source_observation(&sequent.subjects[0].type_id)?;
        let context = relations.source_observation(&sequent.subjects[1].type_id)?;
        let vars = [
            relations.b.var(3)?,
            relations.b.var(2)?,
            relations.b.var(1)?,
            relations.b.var(0)?,
        ];
        let command_equal = call(&mut relations.b, &command.equal, vec![vars[0], vars[2]])?;
        let context_equal = call(&mut relations.b, &context.equal, vec![vars[1], vars[3]])?;
        let body = call(
            &mut relations.b,
            "Std.Bool.and",
            vec![command_equal, context_equal],
        )?;
        let argument_type_ids = sequent
            .subjects
            .iter()
            .map(|s| s.type_id.clone())
            .collect::<Vec<_>>();
        let depths = argument_type_ids
            .iter()
            .map(|id| {
                relations
                    .carriers
                    .get(id)
                    .map(|carrier| carrier.depth)
                    .ok_or(OrdinaryCarrierError::Linkage)
            })
            .collect::<R<Vec<_>>>()?;
        // The W08 symbol ends in a bare hexadecimal contract hash. Certificate
        // name components must start with a letter, so keep the exact symbol in
        // metadata and use an H-prefixed component for the ordinary definition.
        let definition =
            format!("Mpk.CSharp.Ordinary.Transition.CanonicalFieldEncodingsEqual.H{suffix}");
        define(&mut relations.b, &definition, &depths, 0, body)?;
        encoding_equalities.push(OrdinaryTransitionSnapshotEncodingEquality {
            symbol,
            argument_type_ids,
            command_equality_definition: command.equal,
            context_equality_definition: context.equal,
            definition,
        });
    }
    let mut history_capacities = vec![];
    for sequent in source
        .sequents()
        .iter()
        .filter(|s| s.kind == "retained_key_uniqueness")
    {
        let suffix = sequent
            .id
            .strip_prefix("transition.retained_key_uniqueness.")
            .ok_or(OrdinaryCarrierError::Linkage)?;
        if sequent.goals.len() != 1 {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let ContractTerm::App {
            function, argument, ..
        } = &sequent.goals[0]
        else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        let ContractTerm::Const { name, .. } = function.as_ref() else {
            return Err(OrdinaryCarrierError::Linkage);
        };
        if name != &format!("Mpk.CSharp.Transition.RetainedKeysUnique.{suffix}") {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let history_type_id = argument.type_id().to_owned();
        let history_carrier = relations
            .carriers
            .get(&history_type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        if !matches!(
            history_carrier.shape,
            OrdinaryShape::Sequence { capacity: 4096, .. }
        ) {
            return Err(OrdinaryCarrierError::Shape);
        }
        relations.ty(&history_type_id)?;
        let length_definition = format!("{}.Length", n(&history_type_id));
        let history = relations.b.var(0)?;
        let length = call(&mut relations.b, &length_definition, vec![history])?;
        let bound = ordered_fold::word(&mut relations.b, 4096)?;
        let body = relations.eq_word(length, bound)?;
        let definition = format!("Mpk.CSharp.Ordinary.Transition.HistoryCapacity4096.H{suffix}");
        define(
            &mut relations.b,
            &definition,
            &[history_carrier.depth],
            0,
            body,
        )?;
        let symbol = format!("Mpk.CSharp.Transition.HistoryCapacity4096.{suffix}");
        if !source.definition_names().contains(&symbol) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        history_capacities.push(OrdinaryTransitionHistoryCapacity {
            symbol,
            history_type_id,
            history_carrier,
            length_definition,
            definition,
        });
    }
    let mut retained_histories = vec![];
    for sequent in source
        .sequents()
        .iter()
        .filter(|s| s.kind == "retained_key_uniqueness")
    {
        let suffix = sequent
            .id
            .strip_prefix("transition.retained_key_uniqueness.")
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let record_symbol = format!("Mpk.CSharp.Transition.RetainedRecord.{suffix}");
        let mut record_calls = vec![];
        for candidate in source.sequents() {
            for term in candidate.assumptions.iter().chain(&candidate.goals) {
                collect_applications(term, &record_symbol, &mut record_calls);
            }
        }
        record_calls.retain(|arguments| arguments.len() == 2);
        if record_calls.is_empty() {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let append_complete_snapshot_symbol =
            format!("Mpk.CSharp.Transition.AppendCompleteSnapshot.{suffix}");
        let mut append_calls = vec![];
        for candidate in source.sequents() {
            for term in candidate.assumptions.iter().chain(&candidate.goals) {
                collect_applications(term, &append_complete_snapshot_symbol, &mut append_calls);
            }
        }
        append_calls.retain(|arguments| arguments.len() == 5);
        if append_calls.is_empty()
            || append_calls.iter().any(|args| {
                args[0].type_id() != append_calls[0][0].type_id()
                    || args[1].type_id() != append_calls[0][1].type_id()
                    || args[2].type_id() != append_calls[0][2].type_id()
                    || args[3].type_id() != append_calls[0][3].type_id()
                    || args[4].type_id() != append_calls[0][4].type_id()
            })
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let response_type_id = append_calls[0][4].type_id().to_owned();
        let history_term = record_calls[0][0];
        let key_term = record_calls[0][1];
        if record_calls.iter().any(|args| {
            args[0].type_id() != history_term.type_id() || args[1].type_id() != key_term.type_id()
        }) {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let history_type_id = history_term.type_id().to_owned();
        let key_type_id = key_term.type_id().to_owned();
        let state_history_member_id = member_id(history_term)?.to_owned();
        let command_key_member_id = member_id(key_term)?.to_owned();
        let (_, history_arguments) =
            application(history_term).ok_or(OrdinaryCarrierError::Linkage)?;
        let (_, key_arguments) = application(key_term).ok_or(OrdinaryCarrierError::Linkage)?;
        let state_type_id = history_arguments[0].type_id().to_owned();
        let command_type_id = key_arguments[0].type_id().to_owned();
        let context_type_id = append_calls[0][3].type_id().to_owned();
        if append_calls[0][0].type_id() != state_type_id
            || append_calls[0][1].type_id() != state_type_id
            || append_calls[0][2].type_id() != command_type_id
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let history_carrier = relations
            .carriers
            .get(&history_type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        let OrdinaryShape::Sequence {
            capacity: 4096,
            element,
        } = &history_carrier.shape
        else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let OrdinaryShape::Reference {
            type_id: record_type_id,
        } = element.as_ref()
        else {
            return Err(OrdinaryCarrierError::Shape);
        };
        let record_type_id = record_type_id.clone();
        let record_carrier = relations
            .carriers
            .get(&record_type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        let key_carrier = relations
            .carriers
            .get(&key_type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        let (_, roots, _) = relations.vir.construction_context();
        let state_members = &roots
            .source_types
            .get(&state_type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .members;
        let command_members = &roots
            .source_types
            .get(&command_type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .members;
        let record_members = &roots
            .source_types
            .get(&record_type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .members;
        let state_history_member_ordinal = state_members
            .iter()
            .position(|member| member.id == state_history_member_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let command_key_member_ordinal = command_members
            .iter()
            .position(|member| member.id == command_key_member_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let mut projected_record_members = BTreeMap::new();
        for candidate in source.sequents() {
            for term in candidate.assumptions.iter().chain(&candidate.goals) {
                fn projected(
                    term: &ContractTerm,
                    record_symbol: &str,
                    members: &mut BTreeMap<String, String>,
                ) {
                    if let Some((name, arguments)) = application(term) {
                        if let Some(id) = name.strip_prefix("Mpk.CSharp.Transition.Member.") {
                            if arguments.len() == 1
                                && application(arguments[0])
                                    .is_some_and(|(inner, _)| inner == record_symbol)
                            {
                                members.insert(id.to_owned(), term.type_id().to_owned());
                            }
                        }
                    }
                    match term {
                        ContractTerm::App {
                            function, argument, ..
                        } => {
                            projected(function, record_symbol, members);
                            projected(argument, record_symbol, members);
                        }
                        ContractTerm::Lam { body, .. } => projected(body, record_symbol, members),
                        ContractTerm::Let { value, body, .. } => {
                            projected(value, record_symbol, members);
                            projected(body, record_symbol, members);
                        }
                        ContractTerm::Var { .. } | ContractTerm::Const { .. } => {}
                    }
                }
                projected(term, &record_symbol, &mut projected_record_members);
            }
        }
        let key_members = record_members
            .iter()
            .filter(|member| !projected_record_members.contains_key(&member.id))
            .collect::<Vec<_>>();
        if projected_record_members.len() != 3
            || key_members.len() != 1
            || closed_type_id(relations.vir.construction_context().0, &key_members[0].ty)
                .map_err(|_| OrdinaryCarrierError::Linkage)?
                != key_type_id
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        let record_key_member_id = key_members[0].id.clone();
        let record_key_member_ordinal = record_members
            .iter()
            .position(|member| member.id == record_key_member_id)
            .ok_or(OrdinaryCarrierError::Linkage)?;
        let projected_member = |type_id: &str| -> R<(&String, usize)> {
            let matches = projected_record_members
                .iter()
                .filter(|(_, actual)| actual.as_str() == type_id)
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(OrdinaryCarrierError::Linkage);
            }
            let id = matches[0].0;
            let ordinal = record_members
                .iter()
                .position(|member| member.id == *id)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            Ok((id, ordinal))
        };
        let (record_command_member_id, record_command_member_ordinal) =
            projected_member(&command_type_id)?;
        let (record_context_member_id, record_context_member_ordinal) =
            projected_member(&context_type_id)?;
        let (record_response_member_id, record_response_member_ordinal) =
            projected_member(&response_type_id)?;
        let record_command_member_id = record_command_member_id.clone();
        let record_context_member_id = record_context_member_id.clone();
        let record_response_member_id = record_response_member_id.clone();
        let state_history =
            product_member_definition(&mut relations, &state_type_id, &state_history_member_id)?;
        let command_key =
            product_member_definition(&mut relations, &command_type_id, &command_key_member_id)?;
        let record_key =
            product_member_definition(&mut relations, &record_type_id, &record_key_member_id)?;
        let record_command =
            product_member_definition(&mut relations, &record_type_id, &record_command_member_id)?;
        let record_context =
            product_member_definition(&mut relations, &record_type_id, &record_context_member_id)?;
        let record_response =
            product_member_definition(&mut relations, &record_type_id, &record_response_member_id)?;
        let key_equal = relations.ty(&key_type_id)?.equal;
        relations.ty(&history_type_id)?;
        let length_definition = format!("{}.Length", n(&history_type_id));
        let read_definition = format!("{}.ReadAt", n(&history_type_id));
        let fold = aggregate_fold::emit_fold(&mut relations.b, 12)?;
        let history = relations.b.var(13)?;
        let query = relations.b.var(12)?;
        let index = index_word(&mut relations.b, 12)?;
        let record = call(&mut relations.b, &read_definition, vec![history, index])?;
        let stored_key = call(&mut relations.b, &record_key, vec![record])?;
        let equal = call(&mut relations.b, &key_equal, vec![stored_key, query])?;
        let next = ordered_fold::helper(&mut relations.b, "Add1", vec![index])?;
        let zero = ordered_fold::word(&mut relations.b, 0)?;
        let hit = wmux(&mut relations.b, equal, next, zero)?;
        let predicate = relations.b.wrap_selectors(12, hit)?;
        let history = relations.b.var(1)?;
        let length = call(&mut relations.b, &length_definition, vec![history])?;
        let first = call(
            &mut relations.b,
            &fold.first_definition,
            vec![predicate, length],
        )?;
        let find_definition = format!("Mpk.CSharp.Ordinary.Transition.FindRetainedKey.H{suffix}");
        define(
            &mut relations.b,
            &find_definition,
            &[history_carrier.depth, key_carrier.depth],
            5,
            first,
        )?;
        let history = relations.b.var(1)?;
        let query = relations.b.var(0)?;
        let first = call(&mut relations.b, &find_definition, vec![history, query])?;
        let empty = ordered_fold::helper(&mut relations.b, "Empty", vec![first])?;
        let present = call(&mut relations.b, "Std.Bool.not", vec![empty])?;
        let present_definition =
            format!("Mpk.CSharp.Ordinary.Transition.RetainedKeyPresent.H{suffix}");
        define(
            &mut relations.b,
            &present_definition,
            &[history_carrier.depth, key_carrier.depth],
            0,
            present,
        )?;
        let state = relations.b.var(1)?;
        let command = relations.b.var(0)?;
        let history = call(&mut relations.b, &state_history, vec![state])?;
        let key = call(&mut relations.b, &command_key, vec![command])?;
        let body = call(&mut relations.b, &present_definition, vec![history, key])?;
        let retained_key_present_definition =
            format!("Mpk.CSharp.Ordinary.Transition.RetainedKeyPresentForState.H{suffix}");
        define(
            &mut relations.b,
            &retained_key_present_definition,
            &[
                relations.carriers[&state_type_id].depth,
                relations.carriers[&command_type_id].depth,
            ],
            0,
            body,
        )?;
        let subtract = sequence_subtraction(&mut relations.b)?;
        let history = relations.b.var(1)?;
        let query = relations.b.var(0)?;
        let first = call(&mut relations.b, &find_definition, vec![history, query])?;
        let one = ordered_fold::word(&mut relations.b, 1)?;
        let index = call(&mut relations.b, &subtract, vec![first, one])?;
        let record = call(&mut relations.b, &read_definition, vec![history, index])?;
        let retained_record_definition =
            format!("Mpk.CSharp.Ordinary.Transition.RetainedRecord.H{suffix}");
        define(
            &mut relations.b,
            &retained_record_definition,
            &[history_carrier.depth, key_carrier.depth],
            record_carrier.depth,
            record,
        )?;
        let history = relations.b.var(12)?;
        let index = index_word(&mut relations.b, 12)?;
        let record = call(&mut relations.b, &read_definition, vec![history, index])?;
        let key = call(&mut relations.b, &record_key, vec![record])?;
        let first = call(&mut relations.b, &find_definition, vec![history, key])?;
        let expected = ordered_fold::helper(&mut relations.b, "Add1", vec![index])?;
        let first_is_current = relations.eq_word(first, expected)?;
        let predicate = relations.b.wrap_selectors(12, first_is_current)?;
        let history = relations.b.var(0)?;
        let length = call(&mut relations.b, &length_definition, vec![history])?;
        let unique = call(
            &mut relations.b,
            &fold.all_definition,
            vec![predicate, length],
        )?;
        let retained_keys_unique_definition =
            format!("Mpk.CSharp.Ordinary.Transition.RetainedKeysUnique.H{suffix}");
        define(
            &mut relations.b,
            &retained_keys_unique_definition,
            &[history_carrier.depth],
            0,
            unique,
        )?;
        let record_equal = relations.source_observation(&record_type_id)?.equal;
        let command_equal = relations.source_observation(&command_type_id)?.equal;
        let context_equal = relations.source_observation(&context_type_id)?.equal;
        let response_equal = relations.source_observation(&response_type_id)?.equal;
        let old_history = relations.b.var(13)?;
        let next_history = relations.b.var(12)?;
        let index = index_word(&mut relations.b, 12)?;
        let old_record = call(&mut relations.b, &read_definition, vec![old_history, index])?;
        let next_record = call(
            &mut relations.b,
            &read_definition,
            vec![next_history, index],
        )?;
        let same_record = call(
            &mut relations.b,
            &record_equal,
            vec![old_record, next_record],
        )?;
        let predicate = relations.b.wrap_selectors(12, same_record)?;
        let old_history = relations.b.var(1)?;
        let old_length = call(&mut relations.b, &length_definition, vec![old_history])?;
        let prefix_equal = call(
            &mut relations.b,
            &fold.all_definition,
            vec![predicate, old_length],
        )?;
        let history_prefix_definition =
            format!("Mpk.CSharp.Ordinary.Transition.RetainedHistoryPrefix.H{suffix}");
        define(
            &mut relations.b,
            &history_prefix_definition,
            &[history_carrier.depth, history_carrier.depth],
            0,
            prefix_equal,
        )?;
        let state = relations.b.var(1)?;
        let next = relations.b.var(0)?;
        let old_history = call(&mut relations.b, &state_history, vec![state])?;
        let next_history = call(&mut relations.b, &state_history, vec![next])?;
        let body = call(
            &mut relations.b,
            &history_prefix_definition,
            vec![old_history, next_history],
        )?;
        let preserve_retained_history_order_definition =
            format!("Mpk.CSharp.Ordinary.Transition.PreserveRetainedHistoryOrder.H{suffix}");
        define(
            &mut relations.b,
            &preserve_retained_history_order_definition,
            &[
                relations.carriers[&state_type_id].depth,
                relations.carriers[&state_type_id].depth,
            ],
            0,
            body,
        )?;
        let state = relations.b.var(4)?;
        let next = relations.b.var(3)?;
        let command = relations.b.var(2)?;
        let context = relations.b.var(1)?;
        let response = relations.b.var(0)?;
        let old_history = call(&mut relations.b, &state_history, vec![state])?;
        let next_history = call(&mut relations.b, &state_history, vec![next])?;
        let old_length = call(&mut relations.b, &length_definition, vec![old_history])?;
        let next_length = call(&mut relations.b, &length_definition, vec![next_history])?;
        let expected_length = ordered_fold::helper(&mut relations.b, "Add1", vec![old_length])?;
        let length_equal = relations.eq_word(next_length, expected_length)?;
        let appended = call(
            &mut relations.b,
            &read_definition,
            vec![next_history, old_length],
        )?;
        let appended_key = call(&mut relations.b, &record_key, vec![appended])?;
        let appended_command = call(&mut relations.b, &record_command, vec![appended])?;
        let appended_context = call(&mut relations.b, &record_context, vec![appended])?;
        let appended_response = call(&mut relations.b, &record_response, vec![appended])?;
        let key = call(&mut relations.b, &command_key, vec![command])?;
        let key_matches = call(&mut relations.b, &key_equal, vec![appended_key, key])?;
        let command_matches = call(
            &mut relations.b,
            &command_equal,
            vec![appended_command, command],
        )?;
        let context_matches = call(
            &mut relations.b,
            &context_equal,
            vec![appended_context, context],
        )?;
        let response_matches = call(
            &mut relations.b,
            &response_equal,
            vec![appended_response, response],
        )?;
        let body = conjunction(
            &mut relations.b,
            [
                length_equal,
                key_matches,
                command_matches,
                context_matches,
                response_matches,
            ],
        )?;
        let append_complete_snapshot_definition =
            format!("Mpk.CSharp.Ordinary.Transition.AppendCompleteSnapshot.H{suffix}");
        define(
            &mut relations.b,
            &append_complete_snapshot_definition,
            &[
                relations.carriers[&state_type_id].depth,
                relations.carriers[&state_type_id].depth,
                relations.carriers[&command_type_id].depth,
                relations.carriers[&context_type_id].depth,
                relations.carriers[&response_type_id].depth,
            ],
            0,
            body,
        )?;
        let retained_key_present_symbol =
            format!("Mpk.CSharp.Transition.RetainedKeyPresent.{suffix}");
        let retained_keys_unique_symbol =
            format!("Mpk.CSharp.Transition.RetainedKeysUnique.{suffix}");
        let preserve_retained_history_order_symbol =
            format!("Mpk.CSharp.Transition.PreserveRetainedHistoryOrder.{suffix}");
        if !source.definition_names().contains(&record_symbol)
            || !source
                .definition_names()
                .contains(&retained_key_present_symbol)
            || !source
                .definition_names()
                .contains(&retained_keys_unique_symbol)
            || !source
                .definition_names()
                .contains(&append_complete_snapshot_symbol)
            || !source
                .definition_names()
                .contains(&preserve_retained_history_order_symbol)
        {
            return Err(OrdinaryCarrierError::Linkage);
        }
        retained_histories.push(OrdinaryTransitionRetainedHistory {
            contract_sha256: suffix.to_owned(),
            state_type_id,
            command_type_id,
            context_type_id,
            history_type_id,
            record_type_id,
            key_type_id,
            state_history_member_id,
            state_history_member_ordinal,
            command_key_member_id,
            command_key_member_ordinal,
            record_key_member_id,
            record_key_member_ordinal,
            record_command_member_id,
            record_command_member_ordinal,
            record_context_member_id,
            record_context_member_ordinal,
            record_response_member_id,
            record_response_member_ordinal,
            response_type_id,
            retained_record_symbol: record_symbol,
            retained_record_definition,
            retained_key_present_symbol,
            retained_key_present_definition,
            retained_keys_unique_symbol,
            retained_keys_unique_definition,
            append_complete_snapshot_symbol,
            append_complete_snapshot_definition,
            preserve_retained_history_order_symbol,
            preserve_retained_history_order_definition,
        });
    }
    let resolved = definitions
        .iter()
        .map(|d| d.symbol.as_str())
        .chain(encoding_equalities.iter().map(|d| d.symbol.as_str()))
        .chain(history_capacities.iter().map(|d| d.symbol.as_str()))
        .chain(retained_histories.iter().flat_map(|d| {
            [
                d.retained_record_symbol.as_str(),
                d.retained_key_present_symbol.as_str(),
                d.retained_keys_unique_symbol.as_str(),
                d.append_complete_snapshot_symbol.as_str(),
                d.preserve_retained_history_order_symbol.as_str(),
            ]
        }))
        .collect::<BTreeSet<_>>();
    let pending_definition_names = source
        .definition_names()
        .iter()
        .filter(|name| !resolved.contains(name.as_str()))
        .cloned()
        .collect();
    let static_transformers = relations.b.static_transformers;
    let certificate = relations.b.finish()?;
    let p = OrdinaryTransitionSnapshotProgram {
        schema: "mpk.csharp.ordinary_transition_snapshots.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        transition_vc_sha256: source.hash(),
        definitions,
        encoding_equalities,
        history_capacities,
        retained_histories,
        pending_definition_names,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}

pub fn import_csharp_practical_ordinary_transition_snapshots(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryTransitionSnapshotProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_transition_snapshots(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
