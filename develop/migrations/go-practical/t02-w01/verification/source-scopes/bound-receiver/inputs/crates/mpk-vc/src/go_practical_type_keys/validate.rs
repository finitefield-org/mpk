use super::*;

pub(super) mod unicode;
pub(super) fn exported(c: char) -> bool {
    unicode::upper(c)
}

fn need(ok: bool, error: GoTypeKeyError) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(error)
    }
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
fn logical_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && !value.starts_with('/')
        && !value.contains('\\')
        && !value.chars().any(|c| c < ' ')
        && value.split('/').all(|p| !matches!(p, "" | "." | ".."))
}
fn identifier(value: &str) -> bool {
    if matches!(
        value,
        "break"
            | "default"
            | "func"
            | "interface"
            | "select"
            | "case"
            | "defer"
            | "go"
            | "map"
            | "struct"
            | "chan"
            | "else"
            | "goto"
            | "package"
            | "switch"
            | "const"
            | "fallthrough"
            | "if"
            | "range"
            | "type"
            | "continue"
            | "for"
            | "import"
            | "return"
            | "var"
    ) {
        return false;
    }
    let mut chars = value.chars();
    chars.next().is_some_and(|c| c == '_' || unicode::letter(c))
        && chars.all(|c| c == '_' || unicode::letter(c) || unicode::digit(c))
}
fn sorted<'a>(values: impl Iterator<Item = &'a str>) -> Result<()> {
    let mut previous = None;
    for value in values {
        need(digest(value), GoTypeKeyError::Identity)?;
        need(previous.is_none_or(|p| p < value), GoTypeKeyError::Order)?;
        previous = Some(value);
    }
    Ok(())
}
pub(super) fn source_shape(source: &GoTypeSourceObject) -> Result<()> {
    need(
        source.schema == SOURCE_SCHEMA
            && logical_path(&source.capability)
            && logical_path(&source.path)
            && digest(&source.raw_sha256)
            && matches!(
                source.kind.as_str(),
                "go_file"
                    | "go_mod"
                    | "go_sum"
                    | "overlay"
                    | "metadata"
                    | "export_data"
                    | "excluded_source"
                    | "auxiliary"
            ),
        GoTypeKeyError::Source,
    )?;
    need(
        source.bytes <= 16_777_216 && (source.kind != "overlay" || source.bytes <= 1_048_576),
        GoTypeKeyError::Limit,
    )
}
fn builtin(name: &str, width: Option<u8>, signed: Option<bool>) -> bool {
    let expected = match name {
        "bool" | "string" | "error" => return width.is_none() && signed.is_none(),
        "int8" => (8, true),
        "int16" => (16, true),
        "int32" => (32, true),
        "int64" | "int" => (64, true),
        "uint8" => (8, false),
        "uint16" => (16, false),
        "uint32" => (32, false),
        "uint64" | "uint" => (64, false),
        _ => return false,
    };
    width == Some(expected.0) && signed == Some(expected.1)
}
pub(super) fn components(w: &WireComponents, buffers: &[GoTypeSourceBuffer]) -> Result<()> {
    need(w.schema == ROOT_SCHEMA, GoTypeKeyError::Shape)?;
    need(
        digest(&w.content_sha256)
            && identity(w, ROOT_DOMAIN, "content_sha256")? == w.content_sha256,
        GoTypeKeyError::Identity,
    )?;
    sorted(w.source_objects.iter().map(|s| s.id.as_str()))?;
    sorted(w.packages.iter().map(|s| s.id.as_str()))?;
    sorted(w.declarations.iter().map(|s| s.id.as_str()))?;
    sorted(w.types.iter().map(GoSourceTypeKey::id))?;
    sorted(w.underlying.iter().map(|s| s.type_id.as_str()))?;
    sorted(w.aliases.iter().map(|s| s.declaration_id.as_str()))?;
    sorted(w.package_roots.iter().map(String::as_str))?;
    sorted(w.type_roots.iter().map(String::as_str))?;
    need(
        !w.package_roots.is_empty() && !w.type_roots.is_empty(),
        GoTypeKeyError::Reference,
    )?;
    // Construction budget only: these components are not the final artifact's
    // 4096 declarations or a full global-error-universe expansion.
    need(
        w.types.len() <= 4_194_304 && w.declarations.len() <= 4_194_304,
        GoTypeKeyError::Limit,
    )?;
    let sources: BTreeMap<_, _> = w
        .source_objects
        .iter()
        .map(|s| (s.id.as_str(), s))
        .collect();
    let owned: BTreeMap<_, _> = buffers.iter().map(|s| (s.object.id.as_str(), s)).collect();
    need(
        buffers.len() == owned.len() && owned.keys().eq(sources.keys()),
        GoTypeKeyError::Source,
    )?;
    let mut logical = BTreeSet::new();
    for source in &w.source_objects {
        source_shape(source)?;
        let buffer = owned[source.id.as_str()];
        need(
            source == &buffer.object
                && source.bytes == buffer.bytes.len() as u64
                && source.raw_sha256 == sha256_raw_file_bytes(&buffer.bytes).to_hex()
                && source.id == identity(source, SOURCE_DOMAIN, "id")?,
            GoTypeKeyError::Source,
        )?;
        need(
            logical.insert((&source.capability, &source.path)),
            GoTypeKeyError::Source,
        )?;
    }
    let packages: BTreeMap<_, _> = w.packages.iter().map(|p| (p.id.as_str(), p)).collect();
    let declarations: BTreeMap<_, _> = w.declarations.iter().map(|d| (d.id.as_str(), d)).collect();
    let types: BTreeMap<_, _> = w.types.iter().map(|t| (t.id(), t)).collect();
    let mut used_sources = BTreeSet::new();
    let mut package_names = BTreeSet::new();
    for p in &w.packages {
        need(
            p.schema == PACKAGE_SCHEMA && p.id == identity(p, PACKAGE_DOMAIN, "id")?,
            GoTypeKeyError::Identity,
        )?;
        need(
            logical_path(&p.import_path)
                && identifier(&p.package_name)
                && p.package_name != "_"
                && !p.source_objects.is_empty(),
            GoTypeKeyError::Shape,
        )?;
        need(
            package_names.insert(&p.import_path),
            GoTypeKeyError::Reference,
        )?;
        sorted(p.source_objects.iter().map(String::as_str))?;
        let manifest = sources
            .get(p.module_source_object_id.as_str())
            .ok_or(GoTypeKeyError::Reference)?;
        need(manifest.kind == "go_mod", GoTypeKeyError::Reference)?;
        used_sources.insert(p.module_source_object_id.as_str());
        for id in &p.source_objects {
            need(
                sources
                    .get(id.as_str())
                    .is_some_and(|s| s.kind == "go_file"),
                GoTypeKeyError::Reference,
            )?;
            used_sources.insert(id.as_str());
        }
    }
    need(
        used_sources.into_iter().eq(sources.keys().copied()),
        GoTypeKeyError::Reference,
    )?;
    let mut declaration_spans = BTreeSet::new();
    for d in &w.declarations {
        need(
            d.schema == DECL_SCHEMA && d.id == identity(d, DECL_DOMAIN, "id")?,
            GoTypeKeyError::Identity,
        )?;
        let package = packages
            .get(d.package_id.as_str())
            .ok_or(GoTypeKeyError::Reference)?;
        let source = sources
            .get(d.source_object_id.as_str())
            .ok_or(GoTypeKeyError::Reference)?;
        need(
            matches!(d.kind.as_str(), "type" | "alias_type")
                && identifier(&d.name)
                && d.name != "_"
                && package
                    .source_objects
                    .binary_search(&d.source_object_id)
                    .is_ok()
                && d.start_byte < d.end_byte
                && d.end_byte <= source.bytes,
            GoTypeKeyError::Reference,
        )?;
        need(
            declaration_spans.insert((
                &d.package_id,
                &d.source_object_id,
                d.start_byte,
                d.end_byte,
            )),
            GoTypeKeyError::Reference,
        )?;
    }
    let mut nominal = BTreeSet::new();
    let mut used_declarations = BTreeSet::new();
    let mut used_packages: BTreeSet<_> = w.package_roots.iter().map(String::as_str).collect();
    for d in &w.declarations {
        used_packages.insert(d.package_id.as_str());
    }
    for t in &w.types {
        need(
            t.schema() == TYPE_SCHEMA && t.id() == identity(t, TYPE_DOMAIN, "id")?,
            GoTypeKeyError::Identity,
        )?;
        match t {
            GoSourceTypeKey::Builtin {
                name,
                width,
                signed,
                ..
            } => need(builtin(name, *width, *signed), GoTypeKeyError::Shape)?,
            GoSourceTypeKey::Nominal { declaration_id, .. } => {
                need(
                    declarations
                        .get(declaration_id.as_str())
                        .is_some_and(|d| d.kind == "type"),
                    GoTypeKeyError::Reference,
                )?;
                nominal.insert(t.id());
                used_declarations.insert(declaration_id.as_str());
            }
            GoSourceTypeKey::Array { length, .. } => need(*length <= 256, GoTypeKeyError::Limit)?,
            GoSourceTypeKey::Slice { .. } => (),
            GoSourceTypeKey::Struct { fields, .. } => {
                need(fields.len() <= 64, GoTypeKeyError::Limit)?;
                let mut names = BTreeSet::new();
                for field in fields {
                    need(
                        identifier(&field.name)
                            && (field.name == "_" || names.insert(&field.name))
                            && field.tag_bytes_hex.len() % 2 == 0
                            && field
                                .tag_bytes_hex
                                .bytes()
                                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
                        GoTypeKeyError::Shape,
                    )?;
                    let exported = field.name.chars().next().is_some_and(unicode::upper);
                    need(
                        exported == field.unexported_package_id.is_none(),
                        GoTypeKeyError::Reference,
                    )?;
                    if let Some(package) = &field.unexported_package_id {
                        used_packages.insert(package.as_str());
                    }
                    if field.embedded {
                        // Alias spelling may differ from the target's name;
                        // source typing must establish that spelling separately.
                        need(
                            field.name != "_"
                                && matches!(
                                    types.get(field.type_id.as_str()),
                                    Some(
                                        GoSourceTypeKey::Nominal { .. }
                                            | GoSourceTypeKey::Builtin { .. }
                                    )
                                ),
                            GoTypeKeyError::Reference,
                        )?;
                    }
                }
            }
        }
        need(
            t.edges().iter().all(|id| types.contains_key(id)),
            GoTypeKeyError::Reference,
        )?;
    }
    need(
        used_packages.into_iter().eq(packages.keys().copied()),
        GoTypeKeyError::Reference,
    )?;
    let underlying: BTreeMap<_, _> = w
        .underlying
        .iter()
        .map(|u| (u.type_id.as_str(), u.underlying_type_id.as_str()))
        .collect();
    need(
        underlying.keys().copied().eq(nominal),
        GoTypeKeyError::Reference,
    )?;
    for id in underlying.values() {
        need(
            types
                .get(id)
                .is_some_and(|t| !matches!(t, GoSourceTypeKey::Nominal { .. })),
            GoTypeKeyError::Reference,
        )?;
    }
    for alias in &w.aliases {
        need(
            declarations
                .get(alias.declaration_id.as_str())
                .is_some_and(|d| d.kind == "alias_type")
                && types.contains_key(alias.target_type_id.as_str()),
            GoTypeKeyError::Reference,
        )?;
        used_declarations.insert(alias.declaration_id.as_str());
    }
    need(
        used_declarations
            .into_iter()
            .eq(declarations.keys().copied()),
        GoTypeKeyError::Reference,
    )?;
    let edges = |id: &str| -> Vec<&str> {
        if let Some(child) = underlying.get(id) {
            vec![*child]
        } else {
            types[id].edges()
        }
    };
    let mut pending = BTreeMap::new();
    let mut parents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut ready = BTreeSet::new();
    for id in types.keys().copied() {
        let children = edges(id);
        pending.insert(id, children.len());
        if children.is_empty() {
            ready.insert(id);
        }
        for child in children {
            parents.entry(child).or_default().push(id);
        }
    }
    let mut depth = BTreeMap::new();
    while let Some(id) = ready.pop_first() {
        let child_depth = edges(id)
            .iter()
            .map(|child| depth[child])
            .max()
            .unwrap_or(0u32);
        let d = child_depth
            + u32::from(matches!(
                types[id],
                GoSourceTypeKey::Array { .. }
                    | GoSourceTypeKey::Slice { .. }
                    | GoSourceTypeKey::Struct { .. }
            ));
        need(d <= 16, GoTypeKeyError::Limit)?;
        depth.insert(id, d);
        for parent in parents.get(id).into_iter().flatten() {
            let count = pending.get_mut(parent).ok_or(GoTypeKeyError::Reference)?;
            *count -= 1;
            if *count == 0 {
                ready.insert(*parent);
            }
        }
    }
    need(depth.len() == types.len(), GoTypeKeyError::Cycle)?;
    let mut seen = BTreeSet::new();
    let mut todo: Vec<_> = w
        .type_roots
        .iter()
        .map(String::as_str)
        .chain(w.aliases.iter().map(|a| a.target_type_id.as_str()))
        .collect();
    while let Some(id) = todo.pop() {
        need(types.contains_key(id), GoTypeKeyError::Reference)?;
        if seen.insert(id) {
            todo.extend(edges(id));
        }
    }
    need(
        seen.into_iter().eq(types.keys().copied()),
        GoTypeKeyError::Reference,
    )
}
