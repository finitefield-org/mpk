//! Ordinary ordered-map/set operations. Successful bodies require the ordered
//! failure predicates to be false and all input/output public-domain obligations.
use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCollectionFailure {
    pub label: String,
    pub definition: String,
    pub argument_indices: Vec<usize>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCollectionOperation {
    pub operation_id: String,
    pub argument_type_ids: Vec<String>,
    pub result_type_id: String,
    pub normal_definition: String,
    pub failures: Vec<OrdinaryCollectionFailure>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCollectionDefinition {
    pub carrier: OrdinaryCarrier,
    pub key_type_id: String,
    pub value_type_id: Option<String>,
    pub lower_bound_definition: String,
    pub operations: Vec<OrdinaryCollectionOperation>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct OrdinaryCollectionProgram {
    schema: String,
    source_ir_sha256: String,
    foundation_sha256: String,
    definitions: Vec<OrdinaryCollectionDefinition>,
    static_transformers: usize,
    certificate_sha256: String,
    #[serde(skip)]
    certificate: Vec<u8>,
}
impl OrdinaryCollectionProgram {
    pub fn definitions(&self) -> &[OrdinaryCollectionDefinition] {
        &self.definitions
    }
    pub fn certificate_bytes(&self) -> &[u8] {
        &self.certificate
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("typed ordered collection program")
    }
}

fn address_word(b: &mut Builder, outer: u32, start: u32, width: u32) -> R<u32> {
    if start + width > outer || width > 32 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let mut body = bit(b, false)?;
    for i in 0..width {
        let at = equal_address(b, 5, 0, 5, i)?;
        let selector = b.var(5 + outer - 1 - start - i)?;
        body = mux(b, at, selector, body)?;
    }
    b.wrap_selectors(5, body)
}
fn dependency(
    vir: &ValidatedPracticalVir,
    ids: &[String],
    template: &str,
    args: &[String],
) -> R<String> {
    let matches = vir
        .data_closed()
        .entries()
        .iter()
        .filter_map(|e| e["instance_id"].as_str().map(|id| (id, e)))
        .filter(|(id, e)| {
            ids.iter().any(|candidate| candidate == id)
                && e["template_id"] == format!("mpk.csharp.semantic.{template}.v1")
                && vir.data_closed().metadata[*id].argument_ids == args
        })
        .map(|(id, _)| id)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(matches[0].into())
}
fn storage(r: &mut Relations<'_>, carrier: &OrdinaryCarrier) -> R<OrdinaryStructuralOperations> {
    let carriers = r
        .carriers
        .iter()
        .map(|(id, c)| (id.as_str(), c))
        .collect::<BTreeMap<_, _>>();
    Ok(r.storage
        .get(&mut r.b, carrier, &carriers)?
        .ok_or(OrdinaryCarrierError::Shape)?
        .operations)
}
fn emit_search(
    b: &mut Builder,
    carrier: &OrdinaryCarrier,
    key: &Node,
    entry_key: Option<&str>,
) -> R<()> {
    let name = format!("{}.Ordered", n(&carrier.type_id));
    let length = format!("{}.Length", n(&carrier.type_id));
    let read = format!("{}.ReadAt", n(&carrier.type_id));
    let lower = format!("{name}.LowerBound");
    let contains = format!("{name}.Contains");
    let missing = format!("{name}.Missing");
    if b.globals.contains_key(&contains) {
        return Ok(());
    }
    // Each predicate returns index+1 at the first stored key >= the query.
    // Zero is reserved for no match; the outer result then uses the full length.
    let source = b.var(13)?;
    let query = b.var(12)?;
    let index = index_word(b, 12)?;
    let item = call(b, &read, vec![source, index])?;
    let candidate = match entry_key {
        Some(get) => call(b, get, vec![item])?,
        None => item,
    };
    let cmp = call(
        b,
        key.compare.as_ref().ok_or(OrdinaryCarrierError::Shape)?,
        vec![candidate, query],
    )?;
    let less = read_bit(b, cmp, 31)?;
    let next = fold_helper(b, "Add1", vec![index])?;
    let zero = word(b, 0)?;
    let result = wmux(b, less, zero, next)?;
    let predicate = b.wrap_selectors(12, result)?;
    let source = b.var(1)?;
    let len = call(b, &length, vec![source])?;
    let fold = aggregate_fold::emit_fold(b, 12)?;
    let first = call(b, &fold.first_definition, vec![predicate, len])?;
    let empty = fold_helper(b, "Empty", vec![first])?;
    let subtract = super::super::super::super::scalar_bits::sequence_subtraction(b)?;
    let one = word(b, 1)?;
    let offset = call(b, &subtract, vec![first, one])?;
    let body = wmux(b, empty, len, offset)?;
    define(b, &lower, &[carrier.depth, key.depth], 5, body)?;

    let query = b.var(0)?;
    let position = call(b, &lower, vec![source, query])?;
    let in_range = fold_helper(b, "Less", vec![position, len])?;
    let item = call(b, &read, vec![source, position])?;
    let candidate = match entry_key {
        Some(get) => call(b, get, vec![item])?,
        None => item,
    };
    let equal = call(b, &key.equal, vec![candidate, query])?;
    let body = and(b, in_range, equal)?;
    define(b, &contains, &[carrier.depth, key.depth], 0, body)?;
    let body = not(b, body)?;
    define(b, &missing, &[carrier.depth, key.depth], 0, body)?;

    Ok(())
}
fn emit_lookup(
    r: &mut Relations<'_>,
    carrier: &OrdinaryCarrier,
    key_depth: u32,
    lookup_id: &str,
    entry_value: &str,
) -> R<String> {
    let name = format!("{}.Ordered", n(&carrier.type_id));
    let contains = format!("{name}.Contains");
    let lower = format!("{name}.LowerBound");
    let read = format!("{}.ReadAt", n(&carrier.type_id));
    let definition = format!("{name}.Lookup");
    if r.b.globals.contains_key(&definition) {
        return Ok(definition);
    }
    let c = r.carriers[lookup_id].clone();
    let OrdinaryStructuralOperations::Sum { arms, .. } = storage(r, &c)? else {
        return Err(OrdinaryCarrierError::Shape);
    };
    if arms.len() != 2
        || arms[0].tag != 0
        || arms[0].arm_id != "missing_key"
        || arms[1].tag != 1
        || arms[1].arm_id != "found"
    {
        return Err(OrdinaryCarrierError::Shape);
    }
    let b = &mut r.b;
    let source = b.var(1)?;
    let query = b.var(0)?;
    let found = call(b, &contains, vec![source, query])?;
    let pos = call(b, &lower, vec![source, query])?;
    let item = call(b, &read, vec![source, pos])?;
    let value = call(b, entry_value, vec![item])?;
    let yes = call(b, &arms[1].make_definition, vec![value])?;
    let no = call(b, &arms[0].make_definition, vec![])?;
    if !b
        .globals
        .contains_key(&format!("{PREFIX}.Cube.D{}.Mux", c.depth))
    {
        b.helpers(c.depth)?;
    }
    let body = ordered_fold::cube_mux(b, c.depth, found, yes, no)?;
    let definition = format!("{name}.Lookup");
    define(b, &definition, &[carrier.depth, key_depth], c.depth, body)?;
    Ok(definition)
}

// Contract reads share the exact search/lookup bodies with the complete
// collection generator. Input public domains remain obligations of the caller.
pub(in super::super) fn emit_contract_read(
    r: &mut Relations<'_>,
    type_id: &str,
    lookup: bool,
) -> R<String> {
    let metadata = r
        .vir
        .data_closed()
        .metadata
        .get(type_id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let map = match template_name(&metadata.template_id) {
        Some("ordered_map") => true,
        Some("ordered_set") if !lookup => false,
        _ => return Err(OrdinaryCarrierError::Linkage),
    };
    let args = metadata.argument_ids.clone();
    let deps = metadata.dependency_ids.clone();
    if args.len() != if map { 2 } else { 1 } || deps.len() != if map { 3 } else { 1 } {
        return Err(OrdinaryCarrierError::Linkage);
    }
    let carrier = r
        .carriers
        .get(type_id)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .clone();
    let key = r.ty(&args[0])?;
    let (entry_depth, entry_key, entry_value, lookup_id) = if map {
        let entry = dependency(r.vir, &deps, "ordered_entry", &args)?;
        dependency(
            r.vir,
            &deps,
            "bounded_sequence",
            std::slice::from_ref(&entry),
        )?;
        let lookup_id = dependency(r.vir, &deps, "lookup", &args[1..2])?;
        let entry = r
            .carriers
            .get(&entry)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
        let shape = product(vec![
            field("key", reference(&args[0])),
            field("value", reference(&args[1])),
        ]);
        if entry.shape != shape
            || carrier.shape
                != (OrdinaryShape::Sequence {
                    capacity: 4096,
                    element: Box::new(shape),
                })
        {
            return Err(OrdinaryCarrierError::Shape);
        }
        let OrdinaryStructuralOperations::Product { operations } = storage(r, &entry)? else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if operations.fields.len() != 2
            || operations.fields[0].field_id != "key"
            || operations.fields[1].field_id != "value"
        {
            return Err(OrdinaryCarrierError::Shape);
        }
        (
            entry.depth,
            Some(operations.fields[0].definition.clone()),
            Some(operations.fields[1].definition.clone()),
            Some(lookup_id),
        )
    } else {
        dependency(r.vir, &deps, "bounded_sequence", &args)?;
        if carrier.shape
            != (OrdinaryShape::Sequence {
                capacity: 4096,
                element: Box::new(reference(&args[0])),
            })
        {
            return Err(OrdinaryCarrierError::Shape);
        }
        (key.depth, None, None, None)
    };
    if carrier.depth != 13 + entry_depth {
        return Err(OrdinaryCarrierError::Shape);
    }
    r.ty(type_id)?;
    emit_search(&mut r.b, &carrier, &key, entry_key.as_deref())?;
    if lookup {
        emit_lookup(
            r,
            &carrier,
            key.depth,
            lookup_id.as_deref().ok_or(OrdinaryCarrierError::Linkage)?,
            entry_value
                .as_deref()
                .ok_or(OrdinaryCarrierError::Linkage)?,
        )
    } else {
        Ok(format!("{}.Ordered.Contains", n(type_id)))
    }
}

pub(super) fn emit_collection(
    d: &mut Domains<'_>,
    entry: &Value,
) -> R<OrdinaryCollectionDefinition> {
    let id = text(entry, "instance_id")?;
    let map = entry["template_id"] == "mpk.csharp.semantic.ordered_map.v1";
    let metadata = &d.r.vir.data_closed().metadata[id];
    let args = metadata.argument_ids.clone();
    let dependencies = &metadata.dependency_ids;
    if args.len() != if map { 2 } else { 1 } || dependencies.len() != if map { 3 } else { 1 } {
        return Err(OrdinaryCarrierError::Linkage);
    }
    // VIR dependency_ids is a sorted set, not the template's recipe order.
    // Resolve each concrete dependency by its independently required role.
    let deps = if map {
        let entry = dependency(d.r.vir, dependencies, "ordered_entry", &args)?;
        let sequence = dependency(
            d.r.vir,
            dependencies,
            "bounded_sequence",
            std::slice::from_ref(&entry),
        )?;
        let lookup = dependency(d.r.vir, dependencies, "lookup", &args[1..2])?;
        vec![entry, sequence, lookup]
    } else {
        vec![dependency(
            d.r.vir,
            dependencies,
            "bounded_sequence",
            &args,
        )?]
    };
    let key_id = &args[0];
    let carrier =
        d.r.carriers
            .get(id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
    let element = if map {
        product(vec![
            field("key", reference(key_id)),
            field("value", reference(&args[1])),
        ])
    } else {
        reference(key_id)
    };
    if carrier.shape
        != (OrdinaryShape::Sequence {
            capacity: 4096,
            element: Box::new(element.clone()),
        })
    {
        return Err(OrdinaryCarrierError::Shape);
    }
    let key = d.r.ty(key_id)?;
    let relation = d.r.ty(id)?;
    let count = d.ty(id)?;
    let name = format!("{}.Ordered", n(id));
    let length = format!("{}.Length", n(id));
    let read = format!("{}.ReadAt", n(id));
    let valid = format!("{name}.Valid");
    let invalid = format!("{name}.InvalidRepresentation");
    let capacity = format!("{name}.Capacity");
    let lower = format!("{name}.LowerBound");
    let contains = format!("{name}.Contains");
    let missing = format!("{name}.Missing");

    let (entry_depth, entry_make, entry_key, entry_value) = if map {
        let c =
            d.r.carriers
                .get(&deps[0])
                .ok_or(OrdinaryCarrierError::Linkage)?
                .clone();
        if c.shape != element {
            return Err(OrdinaryCarrierError::Shape);
        }
        let OrdinaryStructuralOperations::Product { operations } = storage(&mut d.r, &c)? else {
            return Err(OrdinaryCarrierError::Shape);
        };
        if operations.fields.len() != 2
            || operations.fields[0].field_id != "key"
            || operations.fields[1].field_id != "value"
        {
            return Err(OrdinaryCarrierError::Shape);
        }
        (
            c.depth,
            Some(operations.make_definition),
            Some(operations.fields[0].definition.clone()),
            Some(operations.fields[1].definition.clone()),
        )
    } else {
        (key.depth, None, None, None)
    };
    if carrier.depth != 13 + entry_depth {
        return Err(OrdinaryCarrierError::Shape);
    }
    let stored_key = |b: &mut Builder, item: u32| -> R<u32> {
        match &entry_key {
            Some(get) => call(b, get, vec![item]),
            None => Ok(item),
        }
    };
    let b = &mut d.r.b;
    let source = b.var(0)?;
    let cells = call(b, &count.definition, vec![source])?;
    let body = valid_count(b, cells)?;
    define(b, &valid, &[carrier.depth], 0, body)?;
    let body = not(b, body)?;
    define(b, &invalid, &[carrier.depth], 0, body)?;
    let len = call(b, &length, vec![source])?;
    let cap = word(b, 4096)?;
    let within = fold_helper(b, "Less", vec![len, cap])?;
    let body = not(b, within)?;
    define(b, &capacity, &[carrier.depth], 0, body)?;

    emit_search(b, &carrier, &key, entry_key.as_deref())?;
    let subtract = super::super::super::super::scalar_bits::sequence_subtraction(b)?;

    let value_depth = if map {
        Some(d.r.carriers[&args[1]].depth)
    } else {
        None
    };
    let mut updates = vec![];
    for replace in [false, true] {
        if replace && !map {
            continue;
        }
        let definition = format!("{name}.{}", if replace { "Replace" } else { "Add" });
        let write_at = format!("{definition}.At");
        let depth = carrier.depth;
        // Keep three value binders, as required for a C253 result within
        // the 256-binder limit. Position/length share the existing C6 pair.
        let source = b.var(depth + 2)?;
        let state = b.var(depth + 1)?;
        let replacement = b.var(depth)?;
        let pos = fold_helper(b, "Index", vec![state])?;
        let len = fold_helper(b, "Value", vec![state])?;
        let at = address_word(b, depth, 1, 12)?;
        let equal = {
            let left = fold_helper(b, "Less", vec![at, pos])?;
            let right = fold_helper(b, "Less", vec![pos, at])?;
            let no_left = not(b, left)?;
            let no_right = not(b, right)?;
            and(b, no_left, no_right)?
        };
        let previous_at = if replace {
            at
        } else {
            let before = fold_helper(b, "Less", vec![at, pos])?;
            let one = word(b, 1)?;
            let previous = call(b, &subtract, vec![at, one])?;
            wmux(b, before, at, previous)?
        };
        let previous = call(b, &read, vec![source, previous_at])?;
        let selectors = b.selectors(entry_depth)?;
        let previous = b.app(previous, selectors.clone())?;
        let replacement = b.app(replacement, selectors)?;
        let leaf = mux(b, equal, replacement, previous)?;
        let active = fold_helper(b, "Less", vec![at, len])?;
        let zero = bit(b, false)?;
        let leaf = mux(b, active, leaf, zero)?;
        let selectors = b.selectors(5)?;
        let count_leaf = b.app(len, selectors)?;
        let count_leaf = zero_padding(b, depth, 1, depth - 6, count_leaf)?;
        let role = b.var(depth - 1)?;
        let body = mux(b, role, leaf, count_leaf)?;
        let body = b.wrap_selectors(depth, body)?;
        define(b, &write_at, &[depth, 6, entry_depth], depth, body)?;
        let source = b.var(if map { 2 } else { 1 })?;
        let query = b.var(u32::from(map))?;
        let pos = call(b, &lower, vec![source, query])?;
        let replacement = if let Some(make) = &entry_make {
            let value = b.var(0)?;
            // Replace only the value; retain a stored key's exact cohort.
            let replacement_key = if replace {
                let item = call(b, &read, vec![source, pos])?;
                stored_key(b, item)?
            } else {
                query
            };
            call(b, make, vec![replacement_key, value])?
        } else {
            query
        };
        let len = call(b, &length, vec![source])?;
        let len = if replace {
            len
        } else {
            fold_helper(b, "Add1", vec![len])?
        };
        let state = fold_helper(b, "Make", vec![pos, len])?;
        let body = call(b, &write_at, vec![source, state, replacement])?;
        let mut inputs = vec![depth, key.depth];
        if let Some(value) = value_depth {
            inputs.push(value);
        }
        define(b, &definition, &inputs, depth, body)?;
        updates.push(definition);
    }
    let lookup = if map {
        Some(emit_lookup(
            &mut d.r,
            &carrier,
            key.depth,
            &deps[2],
            entry_value.as_deref().ok_or(OrdinaryCarrierError::Shape)?,
        )?)
    } else {
        None
    };
    let gate = |label: &str, definition: &str, indices: &[usize]| OrdinaryCollectionFailure {
        label: label.into(),
        definition: definition.into(),
        argument_indices: indices.to_vec(),
    };
    let mut operations = vec![];
    let mut expected = vec![];
    let mut op = |suffix: &str,
                  inputs: Vec<String>,
                  output: &str,
                  equation: &str,
                  normal: String,
                  failures: Vec<OrdinaryCollectionFailure>| {
        let operation_id = format!("{id}.{suffix}");
        expected.push(json!({"id":operation_id,"argument_type_ids":inputs,"normal_result_type_id":output,"equation":equation,"error_precedence":failures.iter().map(|f|&f.label).collect::<Vec<_>>()}));
        operations.push(OrdinaryCollectionOperation {
            operation_id,
            argument_type_ids: inputs,
            result_type_id: output.into(),
            normal_definition: normal,
            failures,
        });
    };
    op(
        "validate",
        vec![id.into()],
        BOOL_TYPE_ID,
        if map {
            "bounded_and_strictly_increasing_keys(x)"
        } else {
            "bounded_and_strictly_increasing(x)"
        },
        valid,
        vec![],
    );
    op(
        "count",
        vec![id.into()],
        "mpk.csharp.value.u32.v1",
        if map {
            "seq_length(entries(x))"
        } else {
            "seq_length(x)"
        },
        length,
        vec![],
    );
    op(
        "contains",
        vec![id.into(), key_id.clone()],
        BOOL_TYPE_ID,
        if map {
            "exists_equal_key_in_order(x,k)"
        } else {
            "exists_equal_element_in_order(x,v)"
        },
        contains.clone(),
        vec![],
    );
    if let Some(lookup) = lookup {
        op(
            "lookup",
            vec![id.into(), key_id.clone()],
            &deps[2],
            "missing_or_found_without_collapsing_nullable_value(x,k)",
            lookup,
            vec![],
        );
    }
    let mut inputs = vec![id.into()];
    inputs.extend(args.clone());
    op(
        "add",
        inputs.clone(),
        id,
        if map {
            "insert_at_lower_bound(x,k,v)"
        } else {
            "insert_at_lower_bound(x,v)"
        },
        updates[0].clone(),
        vec![
            gate("invalid_representation", &invalid, &[0]),
            gate(
                if map {
                    "duplicate_key"
                } else {
                    "duplicate_element"
                },
                &contains,
                &[0, 1],
            ),
            gate("capacity", &capacity, &[0]),
        ],
    );
    if map {
        op(
            "replace",
            inputs,
            id,
            "replace_existing_value_preserving_order(x,k,v)",
            updates[1].clone(),
            vec![
                gate("invalid_representation", &invalid, &[0]),
                gate("missing_key", &missing, &[0, 1]),
            ],
        );
    }
    op(
        "equal",
        vec![id.into(), id.into()],
        BOOL_TYPE_ID,
        if map {
            "entrywise_equal(x,y)"
        } else {
            "elementwise_equal(x,y)"
        },
        relation.equal,
        vec![],
    );
    if let Some(compare) = relation.compare {
        op(
            "compare",
            vec![id.into(), id.into()],
            I32_TYPE_ID,
            if map {
                "lexicographic_key_then_value(x,y)"
            } else {
                "lexicographic_compare(x,y)"
            },
            compare,
            vec![],
        );
    }
    if entry["operation_definitions"] != json!(expected) {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(OrdinaryCollectionDefinition {
        carrier,
        key_type_id: key_id.clone(),
        value_type_id: args.get(1).cloned(),
        lower_bound_definition: lower,
        operations,
    })
}

pub fn generate_csharp_practical_ordinary_collections(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryCollectionProgram> {
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut d = Domains {
        public_clauses: None,
        r: Relations {
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
        },
        counts: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    d.r.b.helpers(5)?;
    ordered_fold::auxiliary(&mut d.r.b)?;
    let mut definitions = vec![];
    for entry in vir.data_closed().entries().iter().filter(|e| {
        matches!(
            e["template_id"].as_str(),
            Some("mpk.csharp.semantic.ordered_map.v1" | "mpk.csharp.semantic.ordered_set.v1")
        )
    }) {
        definitions.push(emit_collection(&mut d, entry)?);
    }
    let static_transformers = d.r.b.static_transformers;
    let certificate = d.r.b.finish()?;
    let p = OrdinaryCollectionProgram {
        schema: "mpk.csharp.ordinary_collections.v1".into(),
        source_ir_sha256: vir.hash().into(),
        foundation_sha256: vir.construction_context().0.content_sha256().into(),
        definitions,
        static_transformers,
        certificate_sha256: mpk_cert::hash_hex(&mpk_cert::certificate_hash(&certificate)),
        certificate,
    };
    if p.canonical_bytes().len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    Ok(p)
}
pub fn import_csharp_practical_ordinary_collections(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryCollectionProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_collections(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
