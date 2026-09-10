//! W06 tag, payload, member and semantic-bound predicates.
//! These definitions do not supply reconstruction/native-body witnesses or proofs.
use super::super::super::binding_projections::conversion_name;
use super::*;

fn storage(r: &mut Relations<'_>, c: &OrdinaryCarrier) -> R<OrdinaryStructuralOperations> {
    let refs = r.carriers.iter().map(|(id, c)| (id.as_str(), c)).collect();
    Ok(r.storage
        .get(&mut r.b, c, &refs)?
        .ok_or(OrdinaryCarrierError::Shape)?
        .operations)
}
fn member(r: &mut Relations<'_>, c: &OrdinaryCarrier, id: &str) -> R<(String, String)> {
    let OrdinaryShape::Product { fields } = &c.shape else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let field = fields
        .iter()
        .find(|f| f.id == id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    let OrdinaryStructuralOperations::Product { operations } = storage(r, c)? else {
        return Err(OrdinaryCarrierError::Shape);
    };
    let get = operations
        .fields
        .iter()
        .find(|f| f.field_id == id)
        .ok_or(OrdinaryCarrierError::Linkage)?;
    Ok((get.definition.clone(), reference(&field.shape)?.into()))
}
fn mapped(
    r: &mut Relations<'_>,
    c: &OrdinaryCarrier,
    binding: &Value,
    role: &str,
) -> R<(String, String)> {
    member(r, c, text(&binding["member_map"], role)?)
}
fn emit_predicate(
    a: &mut BindingAssembly<'_>,
    symbol: String,
    args: &[&OrdinaryCarrier],
    body: u32,
) -> R<String> {
    let definition = name(&symbol);
    // Shared semantic predicates can be demanded by several source bindings.
    if !a.r.b.globals.contains_key(&definition) {
        define(
            &mut a.r.b,
            &definition,
            &args.iter().map(|c| c.depth).collect::<Vec<_>>(),
            0,
            body,
        )?;
    }
    insert(
        &mut a.predicates,
        symbol,
        definition.clone(),
        args.iter().map(|c| c.type_id.clone()).collect(),
    )?;
    Ok(definition)
}
fn converted_equal(r: &mut Relations<'_>, from: &str, to: &str, x: u32, y: u32) -> R<u32> {
    let x = if from == to {
        x
    } else {
        call(&mut r.b, &conversion_name(from, to), vec![x])?
    };
    let eq = r.source_observation(to)?.equal;
    call(&mut r.b, &eq, vec![x, y])
}
fn payload_role(role: &str, arm: &str) -> &'static str {
    match (role, arm) {
        ("result", "error") => "error",
        ("validation", "invalid") => "errors",
        _ => "value",
    }
}
fn tag(
    a: &mut BindingAssembly<'_>,
    source: &OrdinaryCarrier,
    binding: &Value,
    symbol: String,
    value: &str,
) -> R<String> {
    let (get, ty) = mapped(&mut a.r, source, binding, "tag")?;
    let c =
        a.r.carriers
            .get(&ty)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
    let OrdinaryShape::Bits { width } = c.shape else {
        return Err(OrdinaryCarrierError::Shape);
    };
    if width == 0 || width > 64 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let n = value
        .parse::<i128>()
        .map_err(|_| OrdinaryCarrierError::Shape)? as u128;
    let x = a.r.b.var(0)?;
    let x = call(&mut a.r.b, &get, vec![x])?;
    let mut body = bit(&mut a.r.b, true)?;
    for i in 0..width {
        let address = prefix(c.depth, i)
            .into_iter()
            .map(|v| bit(&mut a.r.b, v))
            .collect::<R<Vec<_>>>()?;
        let mut v = a.r.b.app(x, address)?;
        if n & (1u128 << i) == 0 {
            v = call(&mut a.r.b, "Std.Bool.not", vec![v])?;
        }
        body = and(&mut a.r.b, body, v)?;
    }
    emit_predicate(a, symbol, &[source], body)
}
fn length(r: &mut Relations<'_>, ty: &str, value: u32) -> R<u32> {
    let c = r
        .carriers
        .get(ty)
        .ok_or(OrdinaryCarrierError::Linkage)?
        .clone();
    if !matches!(c.shape, OrdinaryShape::Sequence { .. }) || c.depth < 5 {
        return Err(OrdinaryCarrierError::Shape);
    }
    let get = name(&format!("length.{ty}"));
    if !r.b.globals.contains_key(&get) {
        r.getter(&get, c.depth, 5, &vec![false; (c.depth - 5) as usize])?;
    }
    call(&mut r.b, &get, vec![value])
}
fn length_condition(r: &mut Relations<'_>, ty: &str, value: u32, max: Option<u32>) -> R<u32> {
    let len = length(r, ty, value)?;
    let limit = ordered_fold::word(&mut r.b, max.unwrap_or(0))?;
    let greater = ordered_fold::helper(&mut r.b, "Less", vec![limit, len])?;
    if max.is_some() {
        call(&mut r.b, "Std.Bool.not", vec![greater])
    } else {
        Ok(greater)
    }
}
fn bound_body(
    r: &mut Relations<'_>,
    target: &OrdinaryCarrier,
    role: &str,
    bound: &str,
    max: Option<u32>,
) -> R<u32> {
    let y = r.b.var(0)?;
    match (role, bound) {
        ("bounded_sequence" | "ordered_map" | "ordered_set", "length") => {
            length_condition(r, &target.type_id, y, max)
        }
        ("transition", "events") => {
            let (get, ty) = member(r, target, "events")?;
            let value = call(&mut r.b, &get, vec![y])?;
            length_condition(r, &ty, value, max)
        }
        ("validation", "errors") => {
            let OrdinaryShape::Sum { arms: shapes } = &target.shape else {
                return Err(OrdinaryCarrierError::Shape);
            };
            let OrdinaryStructuralOperations::Sum { arms, .. } = storage(r, target)? else {
                return Err(OrdinaryCarrierError::Shape);
            };
            let valid = arms
                .iter()
                .find(|a| a.arm_id == "valid")
                .ok_or(OrdinaryCarrierError::Shape)?;
            let invalid = arms
                .iter()
                .find(|a| a.arm_id == "invalid")
                .ok_or(OrdinaryCarrierError::Shape)?;
            let shape = shapes
                .iter()
                .find(|a| a.id == "invalid")
                .ok_or(OrdinaryCarrierError::Shape)?;
            if shape.fields.len() != 1 || invalid.fields.len() != 1 {
                return Err(OrdinaryCarrierError::Shape);
            }
            let value = call(&mut r.b, &invalid.fields[0].definition, vec![y])?;
            let check = length_condition(r, reference(&shape.fields[0].shape)?, value, max)?;
            let active = call(&mut r.b, &invalid.is_active_definition, vec![y])?;
            let invalid_ok = and(&mut r.b, active, check)?;
            let valid_ok = call(&mut r.b, &valid.is_active_definition, vec![y])?;
            call(&mut r.b, "Std.Bool.or", vec![valid_ok, invalid_ok])
        }
        _ => Err(OrdinaryCarrierError::Shape),
    }
}
fn representation(a: &mut BindingAssembly<'_>, rep: &BindingRepresentationVc) -> R<()> {
    let source =
        a.r.carriers
            .get(&rep.projection.source_type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
    let target =
        a.r.carriers
            .get(&rep.projection.semantic_type_id)
            .ok_or(OrdinaryCarrierError::Linkage)?
            .clone();
    let id = &rep.projection.id;
    let mt = &target.type_id;
    let binding = &rep.binding;
    let role = text(binding, "role")?;
    let mut arm_guards = BTreeMap::new();
    if let OrdinaryShape::Sum { arms: shapes } = &target.shape {
        let OrdinaryStructuralOperations::Sum { arms, .. } = storage(&mut a.r, &target)? else {
            return Err(OrdinaryCarrierError::Shape);
        };
        for shape in shapes {
            let ops = arms
                .iter()
                .find(|a| a.arm_id == shape.id && a.tag == shape.tag)
                .ok_or(OrdinaryCarrierError::Linkage)?;
            let value = text(&binding["tag_arms"], &shape.id)?;
            let source_tag = tag(
                a,
                &source,
                binding,
                format!("Mpk.CSharp.Binding.SourceTag.{id}.{value}"),
                value,
            )?;
            insert(
                &mut a.predicates,
                format!("Mpk.CSharp.Binding.SemanticArm.{mt}.{}", shape.id),
                ops.is_active_definition.clone(),
                vec![mt.clone()],
            )?;
            let x = a.r.b.var(1)?;
            let y = a.r.b.var(0)?;
            let sx = call(&mut a.r.b, &source_tag, vec![x])?;
            let sy = call(&mut a.r.b, &ops.is_active_definition, vec![y])?;
            let tag_match = and(&mut a.r.b, sx, sy)?;
            let mut body = tag_match;
            if shape.fields.len() != ops.fields.len() || shape.fields.len() > 1 {
                return Err(OrdinaryCarrierError::Shape);
            }
            for (field, get) in shape.fields.iter().zip(&ops.fields) {
                if field.id != get.field_id {
                    return Err(OrdinaryCarrierError::Linkage);
                }
                let (read, from) =
                    mapped(&mut a.r, &source, binding, payload_role(role, &shape.id))?;
                let left = call(&mut a.r.b, &read, vec![x])?;
                let right = call(&mut a.r.b, &get.definition, vec![y])?;
                let eq = converted_equal(&mut a.r, &from, reference(&field.shape)?, left, right)?;
                body = and(&mut a.r.b, body, eq)?;
            }
            let payload = emit_predicate(
                a,
                format!("Mpk.CSharp.Binding.Payload.{id}.{}", shape.id),
                &[&source, &target],
                body,
            )?;
            arm_guards.insert(
                shape.id.clone(),
                (source_tag, ops.is_active_definition.clone(), payload),
            );
        }
    }
    for (field_role, field_id) in binding["member_map"]
        .as_object()
        .ok_or(OrdinaryCarrierError::Shape)?
    {
        let field_id = field_id.as_str().ok_or(OrdinaryCarrierError::Shape)?;
        let x = a.r.b.var(1)?;
        let y = a.r.b.var(0)?;
        let body = if !arm_guards.is_empty() {
            // Exactly one recognized source/semantic arm must agree. Inactive
            // payload members are left to the separate reconstruction relation.
            let mut body = bit(&mut a.r.b, false)?;
            let OrdinaryShape::Sum { arms } = &target.shape else {
                return Err(OrdinaryCarrierError::Shape);
            };
            for arm in arms {
                let (sg, tg, payload) = &arm_guards[&arm.id];
                let value = if field_role != "tag"
                    && !arm.fields.is_empty()
                    && payload_role(role, &arm.id) == field_role
                {
                    call(&mut a.r.b, payload, vec![x, y])?
                } else {
                    let sx = call(&mut a.r.b, sg, vec![x])?;
                    let sy = call(&mut a.r.b, tg, vec![y])?;
                    and(&mut a.r.b, sx, sy)?
                };
                body = call(&mut a.r.b, "Std.Bool.or", vec![body, value])?;
            }
            body
        } else {
            let (get, from) = member(&mut a.r, &source, field_id)?;
            let left = call(&mut a.r.b, &get, vec![x])?;
            match role {
                "money" | "ordered_entry" | "transition" => {
                    let (get, to) = member(&mut a.r, &target, field_role)?;
                    let right = call(&mut a.r.b, &get, vec![y])?;
                    converted_equal(&mut a.r, &from, &to, left, right)?
                }
                "bounded_sequence" | "ordered_map" | "ordered_set" => {
                    converted_equal(&mut a.r, &from, mt, left, y)?
                }
                "instant"
                    if field_role == "milliseconds"
                        && from == "mpk.csharp.value.i64.v1"
                        && mt == "mpk.csharp.value.instant.v1" =>
                {
                    let eq = a.r.source_observation(mt)?.equal;
                    call(&mut a.r.b, &eq, vec![left, y])?
                }
                _ => return Err(OrdinaryCarrierError::Shape),
            }
        };
        emit_predicate(
            a,
            format!("Mpk.CSharp.Binding.MemberProjection.{id}.{field_id}.{field_role}"),
            &[&source, &target],
            body,
        )?;
    }
    for (bound, maximum) in binding["bounds"]
        .as_object()
        .ok_or(OrdinaryCarrierError::Shape)?
    {
        let maximum = maximum
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or(OrdinaryCarrierError::Shape)?;
        let body = bound_body(&mut a.r, &target, role, bound, Some(maximum))?;
        emit_predicate(
            a,
            format!("Mpk.CSharp.Binding.Bound.{id}.{bound}.{maximum}"),
            &[&target],
            body,
        )?;
    }
    if role == "validation" {
        let body = bound_body(&mut a.r, &target, role, "errors", None)?;
        emit_predicate(
            a,
            format!("Mpk.CSharp.Binding.NonemptyInvalid.{mt}"),
            &[&target],
            body,
        )?;
    }
    Ok(())
}
pub(super) fn emit_guards(a: &mut BindingAssembly<'_>, vc: &BindingVcProgram) -> R<()> {
    for rep in vc.representations() {
        representation(a, rep)?;
    }
    Ok(())
}
pub fn generate_csharp_practical_ordinary_binding_guards(
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingRelationProgram> {
    let construction = generate_construction_vcs(vir).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let vc = generate_binding_vcs(vir, &construction).map_err(|_| OrdinaryCarrierError::Linkage)?;
    let layouts = generate_csharp_practical_ordinary_carriers(vir)?;
    let mut a = assemble(vir, &vc, &layouts)?;
    emit_guards(&mut a, &vc)?;
    finish(vir, &vc, a, "mpk.csharp.ordinary_binding_guards.v1")
}
pub fn import_csharp_practical_ordinary_binding_guards(
    input: &[u8],
    certificate: &[u8],
    vir: &ValidatedPracticalVir,
) -> R<OrdinaryBindingRelationProgram> {
    if input.len() > 16 * 1024 * 1024 || certificate.len() > 16 * 1024 * 1024 {
        return Err(OrdinaryCarrierError::Limit);
    }
    let p = generate_csharp_practical_ordinary_binding_guards(vir)?;
    if input != p.canonical_bytes() || certificate != p.certificate_bytes() {
        return Err(OrdinaryCarrierError::Linkage);
    }
    Ok(p)
}
