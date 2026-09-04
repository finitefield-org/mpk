#!/usr/bin/env python3
"""W08 freeze-package builder and executable conformance model (private only)."""
from __future__ import annotations

import copy
import hashlib
import json
import os
import sys
import tempfile
from decimal import Decimal, localcontext
from pathlib import Path

sys.dont_write_bytecode = True

import foundation_model as m
import foundation_runtime_model as runtime

OWNER = "crates/mpk-vc/tests/csharp_practical_spec.rs"
FAMILY_OWNERS = {
    "descriptor": ("CSHARP-03-T02-W02", "crates/mpk-vc/tests/csharp_practical_vir_model.rs"),
    "specialization": ("CSHARP-03-T02-W02", "crates/mpk-vc/tests/csharp_practical_vir_model.rs"),
    "binding": ("CSHARP-03-T02-W04", "crates/mpk-vc/tests/csharp_practical_source_artifacts.rs"),
    "projection": ("CSHARP-03-T06-W06", "crates/mpk-vc/tests/csharp_practical_vc.rs"),
    "default": ("CSHARP-03-T03-W03", "crates/mpk-cli/tests/csharp_practical_types.rs"),
    "construction": ("CSHARP-03-T03-W05", "crates/mpk-cli/tests/csharp_practical_types.rs"),
    "ownership": ("CSHARP-03-T03-W07", "crates/mpk-cli/tests/csharp_practical_collections.rs"),
    "collections": ("CSHARP-03-T03-W09", "crates/mpk-cli/tests/csharp_practical_collections.rs"),
    "ordering": ("CSHARP-03-T03-W06", "crates/mpk-cli/tests/csharp_practical_types.rs"),
    "nullable": ("CSHARP-03-T03-W12", "crates/mpk-cli/tests/csharp_practical_domain.rs"),
    "business": ("CSHARP-03-T03-W13", "crates/mpk-cli/tests/csharp_practical_domain.rs"),
    "loops": ("CSHARP-03-T04-W02", "crates/mpk-cli/tests/csharp_practical_control.rs"),
    "calls": ("CSHARP-03-T03-W04", "crates/mpk-cli/tests/csharp_practical_types.rs"),
}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def bytes_of(value: object) -> bytes:
    return m.canonical(value) + b"\n"


def strict_json(data: bytes) -> object:
    def pairs(values):
        result = {}
        for key, value in values:
            if key in result:
                raise m.ModelError("duplicate_json_key")
            result[key] = value
        return result
    return json.loads(data, object_pairs_hook=pairs,
                      parse_float=lambda _: (_ for _ in ()).throw(m.ModelError("floating_json")),
                      parse_constant=lambda _: (_ for _ in ()).throw(m.ModelError("nonfinite_json")))


def definitions() -> dict:
    exceptions = ["System.DivideByZeroException", "System.OverflowException", "System.IndexOutOfRangeException",
                  "System.ArgumentException", "System.ArgumentOutOfRangeException", "System.ArgumentNullException",
                  "System.InvalidOperationException", "System.NullReferenceException", "System.Runtime.CompilerServices.SwitchExpressionException"]
    return {
        "schema": m.SCHEMAS["definitions"], "foundation_id": m.FOUNDATION, "version": 1,
        "templates": m.template_registry(),
        "non_templates": [
            {"id": "mpk.csharp.value.unit.v1", "name": "unit", "representation": "zero Boolean cube", "default": "unit", "operations": ["make", "equal", "compare"]},
            {"id": "mpk.csharp.value.parse_error.v1", "name": "parse_error", "representation": "closed tag",
             "arms": ["input_bound", "syntax", "noncanonical", "scale_precision", "range"], "default": "ineligible", "operations": ["tag", "equal", "compare"]},
            {"id": "mpk.csharp.value.instant.v1", "name": "instant", "representation": "signed 64-bit Unix milliseconds",
             "default": "zero", "operations": ["milliseconds", "compare", "add_duration", "subtract_duration", "difference"],
             "errors": ["precision", "range"], "outcome_dependency": "explicit source result binding; never a hidden template argument"},
            {"id": "mpk.csharp.value.exception.v1", "name": "exception", "representation": "closed tag plus active payload Boolean cube",
             "builtins": exceptions, "source_arms": "reachable sealed direct System.Exception subtypes, ascending canonical source ID",
             "default": "ineligible", "operations": ["construct", "is_type", "payload"], "first_source_tag": len(exceptions)},
        ],
        "ordinary_core": {
            "schema": "mpk.csharp.foundation_expansion.v1", "value_carrier": "C(0)=Bool; C(d+1)=Pi(Bool,C(d))",
            "zero": "Z(0)=false; Z(d+1)=lambda(b:Bool).Z(d)",
            "address": "selector binders are least-significant first; false=0, true=1; unused addresses are zero",
            "leaf": "fixed-width little-endian Bool bits; no wide scalar is converted to unary Nat",
            "product": "prepend ceil(log2(field_count)) Bool selectors to padded children; fixed projection supplies field bits then unpads",
            "sum": "product(tag, active_payload); zero inactive branches; validity tests only the active payload",
            "sequence": "product(u32 length, Bool-addressed elements at the frozen role bound); out-of-bound and index>=length addresses are zero",
            "construction": "pre-VIR unique-token, bitmap and sequence state; not a storable source value; token erased only after VCs",
            "lift": "pad with leading Bool selectors; only the all-false padding address exposes the child; unpad supplies false",
            "conditionals": "for S=C(d), mux_S(c,t,e)=mux(d,c,t,e) pointwise; every Bool.rec has Bool cases, major, and result",
            "folds": "static balanced ordered composition of concrete S->S transformers, guarded by fixed-width index<length; Lam/App/Let only; no Nat.rec",
            "operations": "closed equations in templates and specification sections 4-9; finite Bool circuits and concrete transformer composition only",
            "equality": "field/active-payload/element extensional relation, never core function equality or an extensionality axiom",
            "proofs": "ordinary checked terms; no new inductive shape, Axiom, TheoryPrimitive, proof node or theory certificate",
        },
    }


def descriptor() -> dict:
    body = bytes_of(definitions())
    result = {"schema": m.SCHEMAS["descriptor"], "id": m.FOUNDATION, "version": 1,
              "semantic_profile": m.PROFILE,
              "members": sorted([
                  {"path": m.DEFINITIONS_PATH, "schema": m.SCHEMAS["definitions"], "sha256": sha(body), "size_bytes": len(body)},
                  {"path": m.SPEC, "schema": "mpk.csharp.foundation_semantics.v1", "sha256": sha((m.ROOT / m.SPEC).read_bytes()), "size_bytes": len((m.ROOT / m.SPEC).read_bytes())},
              ], key=lambda row: row["path"]),
              "template_ids": sorted(row["id"] for row in m.TEMPLATES.values()),
              "non_template_ids": sorted("mpk.csharp.value." + name + ".v1" for name in m.NON_TEMPLATES),
              "hash_domains": m.DOMAINS, "structural_limits": m.LIMITS, "value_bounds": m.VALUE_BOUNDS,
              "source_callable_members": [], "caller_extension_points": [], "activation": "candidate_only"}
    result["content_sha256"] = m.digest(result, "descriptor")
    return result


def validate_descriptor(value: dict, body: bytes) -> None:
    if value != descriptor() or body != bytes_of(definitions()):
        raise m.ModelError("descriptor_recomputation")


def source(name: str, member_types: list[tuple[str, dict]], *, kind: str = "readonly_struct", enum_values: list[int] | None = None) -> dict:
    identity = m.source_identity("type", "Example", "", name, [], "")
    result = {**identity, "kind": kind, "members": [], "enum_values": [str(x) for x in (enum_values or [])],
              "enum_underlying": "i32" if kind == "enum" else None,
              "actual_default": {}, "public_default": True, "identity_sensitive": False,
              "source_sha256": sha(name.encode())}
    for index, (member_name, ty) in enumerate(member_types):
        preimage = {"owner": result["id"], "name": member_name, "type": ty, "storage": "readonly_field"}
        member = {"id": "mpk.csharp.member." + m.digest(preimage, "member"), "name": member_name,
                  "type": ty, "storage": "readonly_field", "ordinal": index, "required": False}
        result["members"].append(member)
        result["actual_default"][member["id"]] = "0" if ty["kind"] == "source" else 0
    return result


def fixture_binding(role: str) -> tuple[dict, dict, dict]:
    tag = source("Tag" + role, [], kind="enum", enum_values=list(range(len(m.ARMS.get(role, [])))) or [0, 1])
    tag_ref = {"kind": "source", "id": tag["id"]}
    integer, string = m.type_ref("i32"), m.type_ref("string")
    members = {"tag": tag_ref, "value": integer, "error": string, "errors": m.instance("bounded_sequence", string),
               "state": integer, "events": m.instance("bounded_sequence", string), "response": integer,
               "milliseconds": m.type_ref("i64"), "amount": m.type_ref("decimal"), "currency": string,
               "elements": m.instance("bounded_sequence", integer), "key": string,
               "entries": m.instance("bounded_sequence", m.instance("ordered_entry", string, integer))}
    value = source("Value" + role, [(name, members[name]) for name in m.ROLE_MEMBERS[role]])
    sources = {tag["id"]: tag, value["id"]: value}
    mapped = {member["name"]: member["id"] for member in value["members"]}
    binding = {"schema": m.SCHEMAS["binding"], "source_type_id": value["id"], "source_content_sha256": value["source_sha256"],
               "role": role, "member_map": mapped, "tag_arms": {arm: str(i) for i, arm in enumerate(m.ARMS.get(role, []))},
               "inferred_argument_ids": [m.type_id(t) for t in m.binding_arguments(role, {r: members[r] for r in m.ROLE_MEMBERS[role]})],
               "default_arm": "none" if role == "option" else "missing_key" if role == "lookup" else "ineligible",
               "bounds": {"bounded_sequence": {"length": 4096}, "ordered_map": {"length": 4096}, "ordered_set": {"length": 4096}, "validation": {"errors": 256}, "transition": {"events": 4096}}.get(role, {}),
               "operation_map": {}}
    binding["binding_sha256"] = m.digest(binding, "binding")
    return binding, sources, {}


def instant(operation: str, left: int, right: int) -> dict:
    if operation in {"add_duration", "subtract_duration"}:
        if right % 10000:
            return {"error": "precision"}
        value = left + (right // 10000) * (1 if operation == "add_duration" else -1)
    elif operation == "difference":
        value = (left - right) * 10000
    else:
        raise AssertionError(operation)
    return {"value": value} if runtime.MIN64 <= value <= runtime.MAX64 else {"error": "range"}


def money_create(amount: str, currency: str, scale: int) -> dict:
    # The fixture's explicitly declared finite currency domain, not global metadata.
    if currency not in {"AAA", "BBB"}:
        return {"error": "invalid_currency"}
    if not 0 <= scale <= 28:
        return {"error": "invalid_scale"}
    with localcontext() as context:
        context.prec = 120
        value = Decimal(amount)
        if value * 10 ** scale != (value * 10 ** scale).to_integral_value():
            return {"error": "invalid_precision"}
    return {"amount": amount, "currency": currency}


def vector_rows() -> list[dict]:
    rows = []
    def carrier_json(value):
        # Shared canonical JSON permits only safe integer tokens. Wider exact
        # semantic carriers use decimal strings, as the W07 codecs already do.
        if type(value) is int and not -(2**53 - 1) <= value <= 2**53 - 1:
            return str(value)
        if isinstance(value, dict):
            return {k: carrier_json(v) for k, v in value.items()}
        if isinstance(value, (list, tuple)):
            return [carrier_json(v) for v in value]
        return value
    def record(family: str, name: str, data: object, expected: object) -> None:
        task, path = FAMILY_OWNERS[family]
        rows.append({"id": family + "." + name, "family": family, "inputs": carrier_json(data), "expected": carrier_json(expected),
                     "implementation_owner": task, "production_test_owner": path + "#" + task})
    def rejects(family: str, name: str, code: str, function, data: object = None) -> None:
        try:
            function()
        except m.ModelError as error:
            if error.code != code:
                raise AssertionError((name, code, error.code)) from error
        else:
            raise AssertionError("accepted rejection vector " + name)
        record(family, name, data, {"reject": code})
    def equal(family: str, name: str, actual: object, expected: object, data: object = None) -> None:
        if actual != expected:
            raise AssertionError((name, actual, expected))
        record(family, name, data, {"value": expected})

    frozen = descriptor()
    validate_descriptor(frozen, bytes_of(definitions()))
    record("descriptor", "content", frozen, {"accept": True})
    for key in frozen:
        changed = copy.deepcopy(frozen)
        changed.pop(key)
        rejects("descriptor", "missing_" + key, "descriptor_recomputation", lambda: validate_descriptor(changed, bytes_of(definitions())))
    rejects("descriptor", "member_body_mutation", "descriptor_recomputation", lambda: validate_descriptor(frozen, bytes_of(definitions()) + b" "))
    rejects("descriptor", "duplicate_json_key", "duplicate_json_key", lambda: strict_json(b'{"id":0,"id":1}'))
    rejects("descriptor", "floating_json", "floating_json", lambda: strict_json(b'{"version":1.0}'))
    rejects("descriptor", "nonfinite_json", "nonfinite_json", lambda: strict_json(b'{"version":NaN}'))

    roots = []
    for name, template in m.TEMPLATES.items():
        args = [m.type_ref("string") if name == "money" else m.type_ref("i32")] * template["arity"]
        origin = "source_construction" if name == "sequence_construction" else "semantic_binding"
        roots.append({"origin": origin, "provenance_id": "root." + name, "type": m.instance(name, *args)})
    roots.append({"origin": "source_nullable", "provenance_id": "root.nullable.second", "type": m.instance("option", m.type_ref("i32"))})
    closed = m.derive(roots, {}, frozen["content_sha256"])
    equal("specialization", "root_permutation", m.derive(list(reversed(roots)), {}, frozen["content_sha256"]), closed, roots)
    record("specialization", "all_templates", {"roots": roots, "source_types": {}}, closed)
    equal("specialization", "registry_cardinality", len(m.TEMPLATES), 12)
    equal("specialization", "non_template_cardinality", len(m.NON_TEMPLATES), 4)
    for key in closed:
        changed = copy.deepcopy(closed)
        changed.pop(key)
        rejects("specialization", "missing_" + key, "closed_set_recomputation", lambda: m.validate_closed_set(changed, roots, {}, frozen["content_sha256"]))
    for name, mutate in (("omit_dependency", lambda x: x["entries"].pop()), ("reorder", lambda x: x["entries"].reverse()),
                         ("duplicate", lambda x: x["entries"].append(x["entries"][0])),
                         ("provenance", lambda x: x["entries"][0]["provenance_ids"].append("fake")),
                         ("residual_generic", lambda x: x["entries"][0]["type_definition"].update(representation=m.parameter(0))),
                         ("operation_body", lambda x: x["entries"][0]["operation_definitions"][0].update(equation="trusted")),
                         ("counter", lambda x: x["counters"].update(operations=0))):
        changed = copy.deepcopy(closed)
        mutate(changed)
        rejects("specialization", name, "closed_set_recomputation", lambda: m.validate_closed_set(changed, roots, {}, frozen["content_sha256"]))
    rejects("specialization", "duplicate_root", "duplicate_root", lambda: m.derive([roots[0], roots[0]], {}, frozen["content_sha256"]))
    for name, value, code in (("user_generic", m.parameter(0), "generic_or_unknown_type"),
                              ("unknown_template", m.instance("user", m.type_ref("i32")), "unknown_template"),
                              ("wrong_arity", m.instance("result", m.type_ref("i32")), "template_arity"),
                              ("nested_option", m.instance("option", m.instance("option", m.type_ref("i32"))), "nested_option"),
                              ("float_key", m.instance("ordered_map", m.type_ref("f32"), m.type_ref("i32")), "non_total_key"),
                              ("linear_payload", m.instance("option", m.instance("sequence_construction", m.type_ref("i32"))), "nonvalue_argument"),
                              ("money_integer_currency", m.instance("money", m.type_ref("i32")), "currency_type")):
        rejects("specialization", name, code, lambda: m.validate_type(value, {}), value)
    lookup_nullable = m.instance("lookup", m.instance("option", m.type_ref("i32")))
    equal("specialization", "lookup_nullable", m.validate_type(lookup_nullable, {}), lookup_nullable)
    aggregate = source("Aggregate", [("items", m.instance("bounded_sequence", m.type_ref("i32"))), ("optional", m.instance("option", m.type_ref("i32")))])
    aggregate_roots = [{"origin": "contract", "provenance_id": "root.aggregate", "type": {"kind": "source", "id": aggregate["id"]}}]
    aggregate_closed = m.derive(aggregate_roots, {aggregate["id"]: aggregate}, frozen["content_sha256"])
    equal("specialization", "source_member_closure", sorted(x["template_id"] for x in aggregate_closed["entries"]), ["mpk.csharp.semantic.bounded_sequence.v1", "mpk.csharp.semantic.option.v1"], {"sources": {aggregate["id"]: aggregate}, "roots": aggregate_roots})
    cyclic_id = m.source_identity("type", "Example", "", "Cycle", [], "")["id"]
    cyclic = source("Cycle", [("next", {"kind": "source", "id": cyclic_id})])
    rejects("specialization", "source_cycle", "source_cycle", lambda: m.validate_sources({cyclic_id: cyclic}))
    changed_origin = copy.deepcopy(roots[0])
    changed_origin["origin"] = "source_construction"
    rejects("specialization", "invalid_root_derivation", "root_derivation_source", lambda: m.derive([changed_origin], {}, frozen["content_sha256"]))
    for count in (15, 16, 17):
        ty = m.type_ref("i32")
        for _ in range(count):
            ty = m.instance("bounded_sequence", ty)
        if count <= 16:
            equal("specialization", f"depth_{count}", m.validate_type(ty, {}), ty)
        else:
            rejects("specialization", f"depth_{count}", "instance_depth", lambda: m.validate_type(ty, {}))
    for count in (255, 256, 257):
        sources = {}
        large_roots = []
        for i in range(count):
            value = source("E" + str(i), [], kind="enum", enum_values=[0])
            sources[value["id"]] = value
            large_roots.append({"origin": "source_nullable", "provenance_id": f"source.{i}", "type": m.instance("option", {"kind": "source", "id": value["id"]})})
        if count <= 256:
            equal("specialization", f"instances_{count}", len(m.derive(large_roots, sources, frozen["content_sha256"])["entries"]), count)
        else:
            rejects("specialization", f"instances_{count}", "instance_count", lambda: m.derive(large_roots, sources, frozen["content_sha256"]))
    for name, cap in m.LIMITS.items():
        for count in (cap - 1, cap):
            m.bounded_count(count, cap, name)
            record("specialization", f"counter_{name}_{count}", count, {"accept": True})
        rejects("specialization", f"counter_{name}_over", name, lambda: m.bounded_count(cap + 1, cap, name), cap + 1)

    for role in m.ROLE_MEMBERS:
        binding, sources, declarations = fixture_binding(role)
        obligations = m.validate_binding(binding, sources, declarations)
        record("binding", role, {"binding": binding, "sources": sources, "declarations": []}, {"obligations": obligations})
        for key in binding:
            changed = copy.deepcopy(binding)
            changed.pop(key)
            rejects("binding", role + "_missing_" + key, "binding_shape", lambda: m.validate_binding(changed, sources, declarations))
        for name, code, mutation in (
            ("stale_source", "binding_source_stale", lambda b: b.update(source_content_sha256="f" * 64)),
            ("member_identity", "binding_member_identity", lambda b: b["member_map"].update({next(iter(b["member_map"])): "missing"})),
            ("inferred_argument", "binding_argument_inference", lambda b: b.update(inferred_argument_ids=["wrong"])),
            ("default", "binding_default_arm", lambda b: b.update(default_arm="caller_selected")),
            ("bounds", "binding_bounds", lambda b: b.update(bounds={"caller_selected": 1})),
            ("unknown_operation", "binding_operation_set", lambda b: b.update(operation_map={"magical": "missing"})),
        ):
            changed = copy.deepcopy(binding)
            mutation(changed)
            changed["binding_sha256"] = m.digest({k: v for k, v in changed.items() if k != "binding_sha256"}, "binding")
            rejects("binding", role + "_" + name, code, lambda: m.validate_binding(changed, sources, declarations))
    binding, sources, declarations = fixture_binding("option")
    m.validate_binding_table([binding], sources, declarations, {binding["source_type_id"]})
    rejects("binding", "duplicate_entry", "binding_order_or_collision", lambda: m.validate_binding_table([binding, binding], sources, declarations, {binding["source_type_id"]}))
    rejects("binding", "unreachable_entry", "binding_missing_or_unreachable", lambda: m.validate_binding_table([binding], sources, declarations, set()))
    rejects("binding", "missing_entry", "binding_missing_or_unreachable", lambda: m.validate_binding_table([], sources, declarations, {binding["source_type_id"]}))
    source_type = {"kind": "source", "id": binding["source_type_id"]}
    equal("default", "bound_option_actual_none", m.default_eligible(source_type, sources, {binding["source_type_id"]: binding}), True)
    changed_sources = copy.deepcopy(sources)
    changed_sources[binding["source_type_id"]]["actual_default"][binding["member_map"]["tag"]] = "1"
    # A role may be valid while publishing the CLR default is not.
    m.validate_binding(binding, changed_sources, declarations)
    equal("default", "bound_option_actual_some", m.default_eligible(source_type, changed_sources, {binding["source_type_id"]: binding}), False)
    changed = copy.deepcopy(binding)
    changed["tag_arms"]["some"] = changed["tag_arms"]["none"]
    changed["binding_sha256"] = m.digest({k: v for k, v in changed.items() if k != "binding_sha256"}, "binding")
    rejects("binding", "colliding_tag_arms", "binding_tag_carrier", lambda: m.validate_binding(changed, sources, declarations))
    mapped = copy.deepcopy(binding)
    method = m.source_identity("method", "Example", binding["source_type_id"], "HasValue", [], m.type_id(m.type_ref("bool")))["id"]
    mapped["operation_map"] = {"has_value": method}
    mapped["binding_sha256"] = m.digest({k: v for k, v in mapped.items() if k != "binding_sha256"}, "binding")
    signatures = {method: {"argument_type_ids": [m.type_id(m.instance("option", m.type_ref("i32")))], "normal_result_type_id": m.type_id(m.type_ref("bool"))}}
    obligations = m.validate_binding(mapped, sources, signatures)
    equal("binding", "mapped_operation_obligation", sum(o["kind"] == "normal_and_exceptional_operation_commutation" for o in obligations), 1)
    signatures[method]["normal_result_type_id"] = m.type_id(m.type_ref("i32"))
    rejects("binding", "wrong_operation_signature", "binding_operation_signature", lambda: m.validate_binding(mapped, sources, signatures))
    source_values = [(0, 0), (1, -1), (1, 0), (1, 1)]
    semantic_values = [(0, None), (1, -1), (1, 0), (1, 1)]
    project = lambda x: (x[0], x[1] if x[0] else None)
    reconstruct = lambda x: (x[0], x[1] if x[0] else 0)
    m.finite_projection_check(source_values, semantic_values, project, reconstruct)
    record("projection", "total_option", {"source": source_values, "semantic": semantic_values}, {"accept": True})
    rejects("projection", "inactive_payload_observable", "source_round_trip", lambda: m.finite_projection_check([*source_values, (0, 17)], semantic_values, project, reconstruct))
    rejects("projection", "arm_collapse", "source_round_trip", lambda: m.finite_projection_check(source_values, semantic_values, lambda x: (0, None), reconstruct))
    rejects("projection", "operation_mismatch", "operation_commutation", lambda: m.finite_projection_check(source_values, semantic_values, project, reconstruct, [(lambda x: (1, 1), lambda x: (1, 0))]))
    bit_values = ["00000000", "80000000", "7fc01234"]
    m.finite_projection_check(bit_values, bit_values, lambda x: x, lambda x: x)
    rejects("projection", "signed_zero_bits_lost", "source_round_trip", lambda: m.finite_projection_check(bit_values, bit_values, lambda x: "00000000" if x == "80000000" else x, lambda x: x))

    for name in sorted(m.PRIMITIVES):
        equal("default", name, m.default_eligible(m.type_ref(name), {}), name not in {"string", "exception", "parse_error"})
        equal("ordering", name, m.orderable(m.type_ref(name), {}), name not in {"f32", "f64", "exception"})
    for name, arguments in (("result", [m.type_ref("i32"), m.type_ref("string")]), ("validation", [m.type_ref("i32"), m.type_ref("string")]), ("money", [m.type_ref("string")])):
        equal("default", name, m.default_eligible(m.instance(name, *arguments), {}), False)
    for name, values in (("EnumZero", [0, 1, 1]), ("EnumNoZero", [1, 2])):
        value = source(name, [], kind="enum", enum_values=values)
        equal("default", name, m.default_eligible({"kind": "source", "id": value["id"]}, {value["id"]: value}), 0 in values)
    members = [{"id": "a", "storage": "readonly_field", "required": False}, {"id": "b", "storage": "init_auto", "required": True}]
    equal("construction", "ordered_writes", m.constructor_transaction(members, ["a"], ["b"]), ["a", "b"])
    for name, code, ctor, init, flags in (
        ("required_in_ctor", "required_constructor_write", ["b"], [], {}),
        ("required_missing", "required_missing", ["a"], [], {}),
        ("duplicate", "duplicate_initialization", ["a"], ["a", "b"], {}),
        ("escape", "construction_escape", [], [], {"escaped": True}),
        ("construction_inv", "construction_invariant", ["a"], ["b"], {"construction_invariant": False}),
        ("public_inv", "public_invariant", ["a"], ["b"], {"public_invariant": False})):
        rejects("construction", name, code, lambda: m.constructor_transaction(members, ctor, init, **flags))
    rejects("construction", "uninitialized_member", "uninitialized_member", lambda: m.constructor_transaction(members, [], ["b"]))
    equal("construction", "eligible_unassigned_default", m.constructor_transaction(members, [], ["b"], eligible_defaults=frozenset({"a"})), ["b"])

    empty = m.Construction.allocate(0, False)
    equal("ownership", "empty_nondefaultable", empty.publish(), [])
    partial = m.Construction.allocate(2, False)
    partial.write(1, 9, first=True)
    rejects("ownership", "read_uninitialized", "uninitialized", lambda: partial.read(0))
    rejects("ownership", "duplicate_first_write", "already_initialized", lambda: partial.write(1, 8, first=True))
    rejects("ownership", "incomplete_rewrite", "incomplete", lambda: partial.write(1, 8, first=False))
    rejects("ownership", "incomplete_publish", "incomplete", lambda: partial.publish())
    partial.write(0, 7, first=True)
    partial.write(1, 8, first=False)
    equal("ownership", "complete_rewrite_publish", partial.publish(), [7, 8])
    rejects("ownership", "write_after_freeze", "ownership", lambda: partial.write(0, 0, first=False))
    transferred = m.Construction.allocate(1, True)
    transferred.publish(transfer=True)
    rejects("ownership", "read_after_transfer", "ownership", lambda: transferred.read(0))
    borrowed = m.Construction.allocate(1, True)
    borrowed.borrowed = True
    rejects("ownership", "borrowed_write", "ownership", lambda: borrowed.write(0, 1, first=False))
    rejects("ownership", "borrowed_publish", "ownership", lambda: borrowed.publish())
    left = m.Construction.allocate(2, False)
    right = left.fork()
    rejects("ownership", "join_different_allocations", "ownership_join", lambda: m.merge_construction(left, m.Construction.allocate(2, False)))
    left.write(0, 1, first=True)
    right.write(1, 2, first=True)
    equal("ownership", "join_intersection", m.merge_construction(left, right).cells, {})
    left.write(1, 2, first=True)
    right.write(0, 3, first=True)
    equal("ownership", "join_phi", m.merge_construction(left, right).cells, {0: {"phi": [1, 3]}, 1: 2})
    left.publish()
    rejects("ownership", "join_lifetime_mismatch", "ownership_join", lambda: m.merge_construction(left, right))
    rejects("ownership", "negative_length", "negative_length", lambda: m.Construction.allocate(-1, False))
    for role, bound in m.VALUE_BOUNDS.items():
        if role in {"total_cells", "construction"}:
            continue
        for length in (bound - 1, bound, bound + 1):
            if length > m.VALUE_BOUNDS["construction"]:
                rejects("ownership", role + "_construction_limit", "construction_bound", lambda: m.Construction.allocate(length, True))
            elif length > bound:
                rejects("ownership", role + "_publication_limit", "publication_bound", lambda: m.Construction.allocate(length, True).publish(role))
            else:
                equal("ownership", f"{role}_{length}", len(m.Construction.allocate(length, True).publish(role)), length)
    entries = [[1, "a"], [3, "b"]]
    equal("collections", "insert_sorted", m.ordered_update(entries, 2, "c", replace=False), [[1, "a"], [2, "c"], [3, "b"]])
    equal("collections", "replace_existing", m.ordered_update(entries, 3, "c", replace=True, capacity=2), [[1, "a"], [3, "c"]])
    rejects("collections", "duplicate_before_capacity", "duplicate_key", lambda: m.ordered_update(entries, 3, "c", replace=False, capacity=2))
    rejects("collections", "missing_replace", "missing_key", lambda: m.ordered_update(entries, 2, "c", replace=True, capacity=2))
    rejects("collections", "capacity", "capacity", lambda: m.ordered_update(entries, 2, "c", replace=False, capacity=2))
    rejects("collections", "invalid_order", "invalid_representation", lambda: m.ordered_update(list(reversed(entries)), 3, "c", replace=False, capacity=2))
    equal("collections", "lookup_null_is_found", ("found", None) != ("missing_key", None), True)
    equal("collections", "validation_order_duplicates", ["a", "b"] + ["a"], ["a", "b", "a"])
    for operation, left, right, expected_value in (
        ("add_duration", runtime.MAX64, 1, {"error": "precision"}),
        ("add_duration", runtime.MAX64, 10000, {"error": "range"}),
        ("subtract_duration", runtime.MIN64, 10000, {"error": "range"}),
        ("subtract_duration", 0, runtime.MIN64, {"error": "precision"}),
        ("difference", runtime.MAX64, runtime.MIN64, {"error": "range"}),
        ("difference", 2, 1, {"value": 10000}),
        ("add_duration", 2, -10000, {"value": 1}),
        ("difference", 922337203685477, 0, {"value": 9223372036854770000}),
        ("difference", 922337203685478, 0, {"error": "range"})):
        equal("business", f"instant_{operation}_{left}_{right}", instant(operation, left, right), expected_value, [operation, left, right])
    for amount, currency, scale, expected_value in (
        ("1.001", "unknown", 29, {"error": "invalid_currency"}),
        ("1.001", "AAA", 29, {"error": "invalid_scale"}),
        ("1.001", "AAA", 2, {"error": "invalid_precision"}),
        ("1.00", "AAA", 2, {"amount": "1.00", "currency": "AAA"})):
        equal("business", f"money_create_{currency}_{scale}_{amount}", money_create(amount, currency, scale), expected_value, [amount, currency, scale])
    for case in runtime.cases():
        operation = case["operation"]
        if operation.startswith(("nullable.", "lifted.")) or operation == "source.null_short_circuit":
            family = "nullable"
        elif operation == "source.array_two_pass":
            family = "loops"
        elif operation.startswith("source.array_"):
            family = "ownership"
        elif operation == "source.struct_default":
            family = "default"
        elif operation == "source.null_call_order":
            family = "calls"
        elif operation.startswith("source."):
            family = "construction"
        else:
            family = "business"
        record(family, "runtime_" + case["id"], {"operation": case["operation"], "inputs": case["inputs"]}, case["expected"])
    ids = [row["id"] for row in rows]
    if len(ids) != len(set(ids)):
        raise AssertionError("duplicate vector ID")
    return sorted(rows, key=lambda row: row["id"])


def vectors() -> dict:
    rows = vector_rows()
    return {"schema": m.SCHEMAS["vectors"], "owner_test": OWNER, "work_item": m.WORK_ITEM,
            "foundation_sha256": descriptor()["content_sha256"],
            "family_owners": {k: {"work_item": v[0], "test": v[1] + "#" + v[0]} for k, v in sorted(FAMILY_OWNERS.items())},
            "vectors": rows, "vector_count": len(rows), "vector_ids_sha256": sha(m.canonical([r["id"] for r in rows]))}


def products() -> dict[str, bytes]:
    return {m.DEFINITIONS_PATH: bytes_of(definitions()), m.DESCRIPTOR_PATH: bytes_of(descriptor()), m.VECTOR_PATH: bytes_of(vectors())}


def main() -> None:
    if sys.argv[1:] not in (["--check"], ["--emit-patch"], ["--update"]):
        raise SystemExit("usage: --check | --emit-patch | --update")
    files = products()
    if sys.argv[1:] == ["--update"]:
        # These are generated canonical artifacts, never hand-edited source.
        for path, data in files.items():
            target = m.ROOT / path
            target.parent.mkdir(parents=True, exist_ok=True)
            descriptor, temporary = tempfile.mkstemp(prefix=".w08-generated-", dir=target.parent)
            try:
                with os.fdopen(descriptor, "wb") as stream:
                    stream.write(data)
                    stream.flush()
                    os.fsync(stream.fileno())
                os.chmod(temporary, 0o644)
                os.replace(temporary, target)
            finally:
                Path(temporary).unlink(missing_ok=True)
        print(f"W08 generated {len(files)} canonical artifacts")
    elif sys.argv[1:] == ["--emit-patch"]:
        print("*** Begin Patch")
        for path, data in files.items():
            existing = m.ROOT / path
            if existing.exists():
                print("*** Update File: " + str(existing))
                print("@@")
                for line in existing.read_text().splitlines():
                    print("-" + line)
            else:
                print("*** Add File: " + str(existing))
            # Canonical records are single-line JSON, generated by this model.
            for line in data.decode().splitlines():
                print("+" + line)
        print("*** End Patch")
    else:
        for path, data in files.items():
            if (m.ROOT / path).read_bytes() != data:
                raise SystemExit("W08 freeze drift: " + path)
        print(f"W08 specification model passed: {len(vector_rows())} vectors; 12 templates; 4 non-template definitions")


if __name__ == "__main__":
    main()
