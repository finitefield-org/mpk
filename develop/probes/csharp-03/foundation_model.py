#!/usr/bin/env python3
"""Private, executable T01-W08 specification model. Never imported by production."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import itertools
from dataclasses import dataclass, field
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]
WORK_ITEM = "CSHARP-03-T01-W08"
SPEC = "develop/specs/CSHARP_PRACTICAL_FOUNDATION_V1.md"
DESCRIPTOR_PATH = "develop/migrations/csharp-03/foundation/foundation-descriptor.json"
DEFINITIONS_PATH = "develop/migrations/csharp-03/foundation/foundation-definitions.json"
VECTOR_PATH = "develop/specs/vectors/csharp-practical-foundation-v1.json"
PROFILE = "mpk.csharp.practical.v1"
FOUNDATION = "mpk.csharp.practical.foundation.v1"
DOMAINS = {
    "binding": "MPK-CSHARP-SEMANTIC-BINDING-1.0",
    "closed_set": "MPK-CSHARP-CLOSED-INSTANCES-1.0",
    "declaration": "MPK-CSHARP-DECLARATION-1.0",
    "descriptor": "MPK-CSHARP-PRACTICAL-FOUNDATION-1.0",
    "instance": "MPK-CSHARP-SEMANTIC-INSTANCE-1.0",
    "member": "MPK-CSHARP-FOUNDATION-MEMBER-1.0",
    "provenance": "MPK-CSHARP-DECLARATION-PROVENANCE-1.0",
}
SCHEMAS = {
    "binding": "mpk.csharp.semantic_binding.v1",
    "closed_set": "mpk.csharp.closed_instances.v1",
    "descriptor": "mpk.csharp.foundation_descriptor.v1",
    "definitions": "mpk.csharp.foundation_definitions.v1",
    "roots": "mpk.csharp.closed_roots.v1",
    "vectors": "mpk.csharp.practical.foundation.conformance.v1",
}
LIMITS = {
    "binding_count": 128,
    "closed_instance_count": 256,
    "closed_instance_depth": 16,
    "expanded_declarations": 1024,
    "expanded_operations": 4096,
    "expanded_recipe_nodes": 262144,
    "projection_obligations_per_binding": 64,
}
VALUE_BOUNDS = {
    "array": 4096,
    "sequence": 4096,
    "string": 16384,
    "construction": 16384,
    "map": 4096,
    "set": 4096,
    "validation_errors": 256,
    "events": 4096,
    "total_cells": 65536,
}
PRIMITIVES = {
    "bool", "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "char",
    "f32", "f64", "decimal", "string", "date", "time", "duration", "guid",
    "day_of_week", "unit", "parse_error", "instant", "exception",
}
NON_TEMPLATES = ("unit", "parse_error", "instant", "exception")
ROOT_ORIGINS = {
    "source_array", "source_nullable", "source_string", "source_construction",
    "semantic_binding", "contract", "boundary", "transition", "codec_result",
}
DERIVATION_SOURCES = {
    "bounded_sequence": ["source_array", "source_string", "semantic_binding", "contract", "boundary", "transition", "dependency"],
    "sequence_construction": ["source_construction"],
    "ordered_entry": ["semantic_binding", "dependency"],
    "ordered_map": ["semantic_binding"], "ordered_set": ["semantic_binding"],
    "option": ["source_nullable", "semantic_binding", "contract", "boundary", "dependency"],
    "lookup": ["semantic_binding", "dependency"],
    "result": ["semantic_binding", "codec_result"], "validation": ["semantic_binding"],
    "boundary_field": ["semantic_binding", "boundary"], "transition": ["semantic_binding", "transition"],
    "money": ["semantic_binding"],
}
ROLE_MEMBERS = {
    "option": ("tag", "value"),
    "lookup": ("tag", "value"),
    "result": ("tag", "value", "error"),
    "validation": ("tag", "value", "errors"),
    "boundary_field": ("tag", "value"),
    "transition": ("state", "events", "response"),
    "instant": ("milliseconds",),
    "money": ("amount", "currency"),
    "bounded_sequence": ("elements",),
    "ordered_entry": ("key", "value"),
    "ordered_map": ("entries",),
    "ordered_set": ("elements",),
}
ARMS = {
    "option": ("none", "some"),
    "lookup": ("missing_key", "found"),
    "result": ("ok", "error"),
    "validation": ("valid", "invalid"),
    "boundary_field": ("missing", "null", "value"),
}


class ModelError(Exception):
    def __init__(self, code: str):
        self.code = code
        super().__init__(code)


def canonical(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":"), allow_nan=False).encode("utf-8")


def digest(value: object, domain: str) -> str:
    return hashlib.sha256(DOMAINS[domain].encode("ascii") + b"\0" + canonical(value)).hexdigest()


def exact(value: object, keys: set[str], code: str) -> dict:
    if not isinstance(value, dict) or set(value) != keys:
        raise ModelError(code)
    return value


def bounded_count(value: int, limit: int, code: str) -> None:
    if type(value) is not int or value < 0 or value > limit:
        raise ModelError(code)


def type_ref(name: str) -> dict:
    return {"kind": "primitive", "id": name}


def instance(template: str, *arguments: dict) -> dict:
    return {"kind": "instance", "template": template, "arguments": list(arguments)}


def parameter(index: int) -> dict:
    return {"kind": "parameter", "index": index}


def op(name: str, arguments: list[str], result: str, equation: str, errors: tuple[str, ...] = ()) -> dict:
    return {"name": name, "arguments": arguments, "result": result,
            "equation": equation, "error_precedence": list(errors)}


def template_registry() -> list[dict]:
    """All equations are interpreted by the closed expansion calculus in spec §4."""
    a, b, c = parameter(0), parameter(1), parameter(2)
    t = {
        "bounded_sequence": (1, [], {"kind": "sequence", "element": a}, [
            op("length", ["self"], "u32", "seq_length(x)"),
            op("read", ["self", "i32"], "arg0", "seq_read(x,i)", ("index_range",)),
            op("equal", ["self", "self"], "bool", "same_length_and_all_equal(x,y)"),
            op("compare", ["self", "self"], "i32", "lexicographic_compare(x,y)"),
        ]),
        "sequence_construction": (1, [instance("bounded_sequence", a)],
            {"kind": "construction", "element": a}, [
            op("allocate", ["i32", "bool"], "self", "allocate_cells_and_init_bitmap(n,default_eligible)", ("negative_length", "construction_bound")),
            op("read", ["self", "i32"], "arg0", "read_initialized_owned_cell(x,i)", ("ownership", "index_range", "uninitialized")),
            op("fill", ["self", "i32", "arg0"], "self", "first_write_then_mark_initialized(x,i,v)", ("ownership", "index_range", "already_initialized")),
            op("rewrite", ["self", "i32", "arg0"], "self", "functional_update_complete_unique_cells(x,i,v)", ("ownership", "index_range", "incomplete")),
            op("freeze", ["self"], "dependency0", "publish_all_initialized_at_role_bound(x)", ("ownership", "incomplete", "publication_bound")),
        ]),
        "ordered_entry": (2, [], {"kind": "product", "fields": [["key", a], ["value", b]]}, [
            op("make", ["arg0", "arg1"], "self", "product(k,v)"),
            op("key", ["self"], "arg0", "field(x,0)"),
            op("value", ["self"], "arg1", "field(x,1)"),
            op("equal", ["self", "self"], "bool", "fieldwise_equal(x,y)"),
            op("compare", ["self", "self"], "i32", "fieldwise_lexicographic_compare(x,y)"),
        ]),
        "ordered_map": (2, [instance("ordered_entry", a, b), instance("bounded_sequence", instance("ordered_entry", a, b)), instance("lookup", b)],
            {"kind": "ordered_map", "key": a, "value": b}, [
            op("validate", ["self"], "bool", "bounded_and_strictly_increasing_keys(x)"),
            op("count", ["self"], "u32", "seq_length(entries(x))"),
            op("contains", ["self", "arg0"], "bool", "exists_equal_key_in_order(x,k)"),
            op("lookup", ["self", "arg0"], "dependency2", "missing_or_found_without_collapsing_nullable_value(x,k)"),
            op("add", ["self", "arg0", "arg1"], "self", "insert_at_lower_bound(x,k,v)", ("invalid_representation", "duplicate_key", "capacity")),
            op("replace", ["self", "arg0", "arg1"], "self", "replace_existing_value_preserving_order(x,k,v)", ("invalid_representation", "missing_key")),
            op("equal", ["self", "self"], "bool", "entrywise_equal(x,y)"),
            op("compare", ["self", "self"], "i32", "lexicographic_key_then_value(x,y)"),
        ]),
        "ordered_set": (1, [instance("bounded_sequence", a)], {"kind": "ordered_set", "element": a}, [
            op("validate", ["self"], "bool", "bounded_and_strictly_increasing(x)"),
            op("count", ["self"], "u32", "seq_length(x)"),
            op("contains", ["self", "arg0"], "bool", "exists_equal_element_in_order(x,v)"),
            op("add", ["self", "arg0"], "self", "insert_at_lower_bound(x,v)", ("invalid_representation", "duplicate_element", "capacity")),
            op("equal", ["self", "self"], "bool", "elementwise_equal(x,y)"),
            op("compare", ["self", "self"], "i32", "lexicographic_compare(x,y)"),
        ]),
        "option": (1, [], {"kind": "sum", "arms": [["none", []], ["some", [a]]]}, [
            op("none", [], "self", "sum(0,unit)"),
            op("some", ["arg0"], "self", "sum(1,v)"),
            op("has_value", ["self"], "bool", "tag(x)==1"),
            op("value", ["self"], "arg0", "active_payload(x,1)", ("invalid_operation",)),
            op("value_or", ["self", "arg0"], "arg0", "if_tag_1_then_payload_else_fallback(x,v)"),
            op("equal", ["self", "self"], "bool", "tag_and_active_payload_equal(x,y)"),
            op("compare", ["self", "self"], "i32", "null_first_then_payload_compare(x,y)"),
        ]),
        "lookup": (1, [], {"kind": "sum", "arms": [["missing_key", []], ["found", [a]]]}, [
            op("missing", [], "self", "sum(0,unit)"),
            op("found", ["arg0"], "self", "sum(1,v)"),
            op("is_found", ["self"], "bool", "tag(x)==1"),
            op("value", ["self"], "arg0", "active_payload(x,1)", ("invalid_operation",)),
            op("equal", ["self", "self"], "bool", "tag_and_active_payload_equal(x,y)"),
            op("compare", ["self", "self"], "i32", "tag_then_active_payload_compare(x,y)"),
        ]),
        "result": (2, [], {"kind": "sum", "arms": [["ok", [a]], ["error", [b]]]}, [
            op("ok", ["arg0"], "self", "sum(0,v)"),
            op("error", ["arg1"], "self", "sum(1,e)"),
            op("is_ok", ["self"], "bool", "tag(x)==0"),
            op("value", ["self"], "arg0", "active_payload(x,0)", ("invalid_operation",)),
            op("error_value", ["self"], "arg1", "active_payload(x,1)", ("invalid_operation",)),
            op("equal", ["self", "self"], "bool", "tag_and_active_payload_equal(x,y)"),
            op("compare", ["self", "self"], "i32", "tag_then_active_payload_compare(x,y)"),
        ]),
        "validation": (2, [instance("bounded_sequence", b)],
            {"kind": "sum", "arms": [["valid", [a]], ["invalid", [instance("bounded_sequence", b)]]]}, [
            op("valid", ["arg0"], "self", "sum(0,v)"),
            op("invalid", ["dependency0"], "self", "sum(1,nonempty_errors)", ("empty_errors", "validation_bound")),
            op("is_valid", ["self"], "bool", "tag(x)==0"),
            op("value", ["self"], "arg0", "active_payload(x,0)", ("invalid_operation",)),
            op("errors", ["self"], "dependency0", "active_payload(x,1)", ("invalid_operation",)),
            op("append_errors", ["dependency0", "dependency0"], "dependency0", "left_errors_then_right_errors(x,y)", ("validation_bound",)),
            op("equal", ["self", "self"], "bool", "tag_and_active_payload_equal(x,y)"),
            op("compare", ["self", "self"], "i32", "tag_then_active_payload_compare(x,y)"),
        ]),
        "boundary_field": (1, [], {"kind": "sum", "arms": [["missing", []], ["null", []], ["value", [a]]]}, [
            op("missing", [], "self", "sum(0,unit)"),
            op("null", [], "self", "sum(1,unit)"),
            op("value", ["arg0"], "self", "sum(2,v)"),
            op("tag", ["self"], "u32", "tag(x)"),
            op("payload", ["self"], "arg0", "active_payload(x,2)", ("invalid_operation",)),
            op("equal", ["self", "self"], "bool", "tag_and_active_payload_equal(x,y)"),
            op("compare", ["self", "self"], "i32", "tag_then_active_payload_compare(x,y)"),
        ]),
        "transition": (3, [instance("bounded_sequence", b)],
            {"kind": "product", "fields": [["state", a], ["events", instance("bounded_sequence", b)], ["response", c]]}, [
            op("make", ["arg0", "dependency0", "arg2"], "self", "product(state,events,response)", ("event_bound",)),
            op("state", ["self"], "arg0", "field(x,0)"),
            op("events", ["self"], "dependency0", "field(x,1)"),
            op("response", ["self"], "arg2", "field(x,2)"),
            op("equal", ["self", "self"], "bool", "fieldwise_equal(x,y)"),
            op("compare", ["self", "self"], "i32", "fieldwise_lexicographic_compare(x,y)"),
        ]),
        "money": (1, [], {"kind": "product", "fields": [["amount", type_ref("decimal")], ["currency", a]]}, [
            op("create", ["decimal", "arg0", "i32"], "self", "validate_currency_scale_and_exact_amount(amount,currency,scale)", ("invalid_currency", "invalid_scale", "invalid_precision")),
            op("amount", ["self"], "decimal", "field(x,0)"),
            op("currency", ["self"], "arg0", "field(x,1)"),
            op("add", ["self", "self"], "self", "same_currency_checked_decimal_add(x,y)", ("currency_mismatch", "decimal_overflow")),
            op("subtract", ["self", "self"], "self", "same_currency_checked_decimal_subtract(x,y)", ("currency_mismatch", "decimal_overflow")),
            op("multiply", ["self", "decimal", "i32", "u32"], "self", "checked_decimal_product_then_explicit_round(x,q,scale,mode)", ("invalid_scale", "invalid_rounding", "decimal_overflow")),
            op("divide", ["self", "decimal", "i32", "u32"], "self", "checked_decimal_quotient_then_explicit_round(x,q,scale,mode)", ("invalid_scale", "invalid_rounding", "division_by_zero", "decimal_overflow")),
            op("amount_compare", ["self", "self"], "i32", "same_currency_decimal_compare(x,y)", ("currency_mismatch",)),
            op("equal", ["self", "self"], "bool", "currency_equal_and_decimal_value_equal(x,y)"),
            op("compare", ["self", "self"], "i32", "currency_first_then_decimal_value_compare(x,y)"),
        ]),
    }
    rows = []
    for name, (arity, dependencies, representation, operations) in sorted(t.items()):
        rows.append({"id": f"mpk.csharp.semantic.{name}.v1", "name": name, "version": 1,
                     "arity": arity, "dependencies": dependencies, "representation": representation,
                     "operations": operations,
                     "default": "none" if name == "option" else "missing_key" if name == "lookup" else "empty" if name in {"bounded_sequence", "ordered_map", "ordered_set"} else "recursive" if name == "ordered_entry" else "ineligible",
                     "derivation_sources": DERIVATION_SOURCES[name], "source_callable": False})
    return rows


TEMPLATES = {row["name"]: row for row in template_registry()}


def replace_parameters(value: object, arguments: list[dict]) -> object:
    if isinstance(value, dict):
        if value.get("kind") == "parameter":
            exact(value, {"kind", "index"}, "parameter_shape")
            index = value["index"]
            if type(index) is not int or not 0 <= index < len(arguments):
                raise ModelError("parameter_arity")
            return copy.deepcopy(arguments[index])
        return {key: replace_parameters(item, arguments) for key, item in value.items()}
    if isinstance(value, list):
        return [replace_parameters(item, arguments) for item in value]
    return value


def validate_type(value: object, source_types: dict[str, dict], depth: int = 0) -> dict:
    if depth > LIMITS["closed_instance_depth"]:
        raise ModelError("instance_depth")
    if not isinstance(value, dict):
        raise ModelError("type_shape")
    kind = value.get("kind")
    if kind == "primitive":
        exact(value, {"kind", "id"}, "type_shape")
        if value["id"] not in PRIMITIVES:
            raise ModelError("unknown_type")
    elif kind == "source":
        exact(value, {"kind", "id"}, "type_shape")
        if value["id"] not in source_types:
            raise ModelError("unknown_source_type")
    elif kind == "instance":
        exact(value, {"kind", "template", "arguments"}, "type_shape")
        template = TEMPLATES.get(value["template"])
        if template is None:
            raise ModelError("unknown_template")
        arguments = value["arguments"]
        if not isinstance(arguments, list) or len(arguments) != template["arity"]:
            raise ModelError("template_arity")
        for argument in arguments:
            validate_type(argument, source_types, depth + 1)
            if argument.get("template") == "sequence_construction" or argument.get("id") == "exception":
                raise ModelError("nonvalue_argument")
        if value["template"] == "option" and arguments[0].get("template") == "option":
            raise ModelError("nested_option")
        if value["template"] in {"ordered_map", "ordered_set"} and not orderable(arguments[0], source_types):
            raise ModelError("non_total_key")
        if value["template"] == "money":
            currency = arguments[0]
            source = source_types.get(currency.get("id"), {})
            if currency != type_ref("string") and not (currency.get("kind") == "source" and source.get("kind") == "enum"):
                raise ModelError("currency_type")
    else:
        raise ModelError("generic_or_unknown_type")
    return value


def orderable(value: dict, source_types: dict[str, dict], active: frozenset[str] = frozenset()) -> bool:
    kind = value.get("kind")
    if kind == "primitive":
        return value["id"] not in {"f32", "f64", "exception"}
    if kind == "instance":
        return value["template"] != "sequence_construction" and all(orderable(a, source_types, active) for a in value["arguments"])
    if kind == "source":
        name = value["id"]
        if name in active or name not in source_types:
            raise ModelError("source_cycle")
        source = source_types[name]
        return source["kind"] == "enum" or all(orderable(m["type"], source_types, active | {name}) for m in source["members"])
    raise ModelError("unknown_type")


def type_id(value: dict) -> str:
    if value["kind"] in {"primitive", "source"}:
        return ("mpk.csharp.value." + value["id"] + ".v1") if value["kind"] == "primitive" else value["id"]
    preimage = {"template": TEMPLATES[value["template"]]["id"], "version": 1,
                "arguments": [type_id(a) for a in value["arguments"]]}
    return "mpk.csharp.instance." + digest(preimage, "instance")


def all_instances(value: dict, source_types: dict[str, dict]) -> list[dict]:
    if value["kind"] == "source":
        return [item for member in source_types[value["id"]]["members"] for item in all_instances(member["type"], source_types)]
    if value["kind"] != "instance":
        return []
    return [value] + [item for arg in value["arguments"] for item in all_instances(arg, source_types)]


def concrete_representation(value: object) -> object:
    if isinstance(value, dict):
        if value.get("kind") in {"primitive", "source", "instance"}:
            return {"kind": "concrete", "type_id": type_id(value)}
        if value.get("kind") == "parameter":
            raise ModelError("residual_generic")
        return {key: concrete_representation(item) for key, item in value.items()}
    if isinstance(value, list):
        return [concrete_representation(item) for item in value]
    return value


def node_count(value: object) -> int:
    if isinstance(value, dict):
        return 1 + sum(node_count(v) for v in value.values())
    if isinstance(value, list):
        return 1 + sum(node_count(v) for v in value)
    return 1


def derive(roots: list[dict], source_types: dict[str, dict], foundation_hash: str) -> dict:
    validate_sources(source_types)
    if not re.fullmatch(r"[0-9a-f]{64}", foundation_hash):
        raise ModelError("foundation_hash")
    pending: list[tuple[dict, str]] = []
    seen_roots = set()
    for root in roots:
        exact(root, {"origin", "provenance_id", "type"}, "root_shape")
        if root["origin"] not in ROOT_ORIGINS or not re.fullmatch(r"[a-z][a-z0-9_.:-]{0,255}", root["provenance_id"]):
            raise ModelError("root_provenance")
        validate_type(root["type"], source_types)
        if root["type"]["kind"] == "instance" and root["origin"] not in DERIVATION_SOURCES[root["type"]["template"]]:
            raise ModelError("root_derivation_source")
        key = canonical(root)
        if key in seen_roots:
            raise ModelError("duplicate_root")
        seen_roots.add(key)
        for concrete in all_instances(root["type"], source_types):
            pending.append((concrete, root["provenance_id"]))
    instances: dict[str, dict] = {}
    provenance: dict[str, set[str]] = {}
    while pending:
        value, origin = pending.pop()
        validate_type(value, source_types)
        identity = type_id(value)
        if identity in instances and instances[identity] != value:
            raise ModelError("instance_collision")
        if identity not in instances:
            bounded_count(len(instances) + 1, LIMITS["closed_instance_count"], "instance_count")
            instances[identity] = value
            provenance[identity] = set()
        if origin in provenance[identity]:
            continue
        provenance[identity].add(origin)
        row = TEMPLATES[value["template"]]
        dependencies = replace_parameters(row["dependencies"], value["arguments"])
        for dependency in dependencies:
            for concrete in all_instances(dependency, source_types):
                pending.append((concrete, origin))
    entries = []
    declarations = operations = recipe_nodes = 0
    for identity, value in sorted(instances.items()):
        template = TEMPLATES[value["template"]]
        dependencies = replace_parameters(template["dependencies"], value["arguments"])
        declared_dependency_ids = [type_id(d) for d in dependencies]
        representation = concrete_representation(replace_parameters(template["representation"], value["arguments"]))
        def resolve_type(reference: str) -> str:
            if reference == "self":
                return identity
            if reference.startswith("arg"):
                return type_id(value["arguments"][int(reference[3:])])
            if reference.startswith("dependency"):
                return declared_dependency_ids[int(reference[10:])]
            return type_id(type_ref(reference))
        expanded_operations = []
        for operation in template["operations"]:
            if operation["name"] == "compare" and not orderable(value, source_types):
                continue
            expanded_operations.append({
                "id": identity + "." + operation["name"],
                "argument_type_ids": [resolve_type(t) for t in operation["arguments"]],
                "normal_result_type_id": resolve_type(operation["result"]),
                "equation": operation["equation"],
                "error_precedence": operation["error_precedence"],
            })
        entry = {"instance_id": identity, "template_id": template["id"], "version": 1,
                 "semantic_profile": PROFILE, "arity": template["arity"],
                 "argument_ids": [type_id(a) for a in value["arguments"]],
                 "dependency_ids": sorted(set(declared_dependency_ids)),
                 "provenance_ids": sorted(provenance[identity]),
                 "type_definition": {"id": identity, "representation": representation},
                 "operation_definitions": expanded_operations}
        entry["counters"] = {"declarations": 1, "operations": len(expanded_operations), "recipe_nodes": node_count(entry["type_definition"]) + node_count(expanded_operations)}
        declarations += entry["counters"]["declarations"]
        operations += entry["counters"]["operations"]
        recipe_nodes += entry["counters"]["recipe_nodes"]
        bounded_count(declarations, LIMITS["expanded_declarations"], "expanded_declarations")
        bounded_count(operations, LIMITS["expanded_operations"], "expanded_operations")
        bounded_count(recipe_nodes, LIMITS["expanded_recipe_nodes"], "expanded_recipe_nodes")
        entries.append(entry)
    result = {"schema": SCHEMAS["closed_set"], "semantic_profile": PROFILE,
              "foundation_id": FOUNDATION, "foundation_sha256": foundation_hash,
              "entries": entries, "counters": {"declarations": declarations, "operations": operations, "recipe_nodes": recipe_nodes}}
    result["closed_set_sha256"] = digest(result, "closed_set")
    return result


def validate_closed_set(value: dict, roots: list[dict], source_types: dict[str, dict], foundation_hash: str) -> None:
    if value != derive(roots, source_types, foundation_hash):
        raise ModelError("closed_set_recomputation")


@dataclass
class Construction:
    length: int
    default_eligible: bool
    cells: dict[int, object]
    state: str = "unique"
    borrowed: bool = False
    owner: object = field(default_factory=object, repr=False)

    def fork(self) -> "Construction":
        """Two control-flow states for one allocation retain the same owner."""
        return Construction(self.length, self.default_eligible, copy.deepcopy(self.cells),
                            self.state, self.borrowed, self.owner)

    @classmethod
    def allocate(cls, length: int, default_eligible: bool, default: object = 0) -> "Construction":
        if length < 0:
            raise ModelError("negative_length")
        bounded_count(length, VALUE_BOUNDS["construction"], "construction_bound")
        return cls(length, default_eligible, {i: copy.deepcopy(default) for i in range(length)} if default_eligible else {})

    def index(self, index: int) -> None:
        if type(index) is not int or index < 0 or index >= self.length:
            raise ModelError("index_range")

    def write(self, index: int, value: object, *, first: bool) -> None:
        if self.state != "unique" or self.borrowed:
            raise ModelError("ownership")
        self.index(index)
        if first and index in self.cells:
            raise ModelError("already_initialized")
        if not first and len(self.cells) != self.length:
            raise ModelError("incomplete")
        self.cells[index] = value

    def read(self, index: int) -> object:
        if self.state == "transferred":
            raise ModelError("ownership")
        self.index(index)
        if index not in self.cells:
            raise ModelError("uninitialized")
        return self.cells[index]

    def publish(self, role: str = "array", transfer: bool = False) -> list[object]:
        if self.state != "unique" or self.borrowed:
            raise ModelError("ownership")
        if len(self.cells) != self.length:
            raise ModelError("incomplete")
        if role not in {"array", "sequence", "string", "map", "set", "validation_errors", "events"}:
            raise ModelError("publication_role")
        bounded_count(self.length, VALUE_BOUNDS[role], "publication_bound")
        self.state = "transferred" if transfer else "frozen"
        return [self.cells[i] for i in range(self.length)]


def ordered_update(entries: list[list], key: object, value: object, *, replace: bool, capacity: int = 4096) -> list[list]:
    if len(entries) > capacity or any(entries[i - 1][0] >= entries[i][0] for i in range(1, len(entries))):
        raise ModelError("invalid_representation")
    found = next((i for i, row in enumerate(entries) if row[0] == key), None)
    if replace:
        if found is None:
            raise ModelError("missing_key")
        return [[k, value if i == found else v] for i, (k, v) in enumerate(entries)]
    if found is not None:
        raise ModelError("duplicate_key")
    if len(entries) == capacity:
        raise ModelError("capacity")
    return sorted([*entries, [key, value]], key=lambda row: row[0])


def source_identity(kind: str, namespace: str, owner: str, name: str,
                    parameter_ids: list[str], result_id: str) -> dict:
    identity = {"kind": kind, "namespace": namespace, "owner": owner, "name": name,
                "parameter_type_ids": parameter_ids, "result_type_id": result_id}
    return {"identity": identity, "id": "mpk.csharp.source." + digest(identity, "declaration")}


def validate_sources(sources: dict[str, dict]) -> None:
    if not isinstance(sources, dict):
        raise ModelError("source_table")
    for identity, source in sources.items():
        exact(source, {"id", "identity", "kind", "members", "enum_values", "enum_underlying", "actual_default",
                       "public_default", "identity_sensitive", "source_sha256"}, "source_shape")
        exact(source["identity"], {"kind", "namespace", "owner", "name", "parameter_type_ids", "result_type_id"}, "identity_shape")
        if source["identity"]["kind"] != "type" or source["identity"]["parameter_type_ids"] != [] or source["identity"]["result_type_id"] != "":
            raise ModelError("source_identity")
        if identity != source["id"] or identity != "mpk.csharp.source." + digest(source["identity"], "declaration"):
            raise ModelError("source_identity")
        if source["kind"] not in {"readonly_struct", "sealed_class", "enum"}:
            raise ModelError("source_kind")
        if not re.fullmatch(r"[0-9a-f]{64}", source["source_sha256"]):
            raise ModelError("source_hash")
        if type(source["public_default"]) is not bool or type(source["identity_sensitive"]) is not bool:
            raise ModelError("source_flags")
        ids = set()
        for ordinal, member in enumerate(source["members"]):
            exact(member, {"id", "name", "type", "storage", "ordinal", "required"}, "stored_member_shape")
            preimage = {"owner": identity, "name": member["name"], "type": member["type"], "storage": member["storage"]}
            if member["id"] != "mpk.csharp.member." + digest(preimage, "member") or member["id"] in ids:
                raise ModelError("stored_member_identity")
            if member["ordinal"] != ordinal or member["storage"] not in {"readonly_field", "get_auto", "init_auto"}:
                raise ModelError("stored_member_order_or_storage")
            if type(member["required"]) is not bool or (member["required"] and member["storage"] != "init_auto"):
                raise ModelError("required_storage")
            validate_type(member["type"], sources)
            if member["type"].get("template") == "sequence_construction" or member["type"].get("id") == "exception":
                raise ModelError("nonvalue_member")
            ids.add(member["id"])
        if not isinstance(source["actual_default"], dict) or set(source["actual_default"]) != ids:
            raise ModelError("source_default_shape")
        if source["kind"] == "enum":
            underlying = source["enum_underlying"]
            if underlying not in {"i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64"}:
                raise ModelError("enum_shape")
            bits = int(underlying[1:])
            lower, upper = (-(1 << (bits - 1)), (1 << (bits - 1)) - 1) if underlying[0] == "i" else (0, (1 << bits) - 1)
            if source["members"] or not source["enum_values"] or any(not isinstance(x, str) or not re.fullmatch(r"0|-?[1-9][0-9]*", x) or not lower <= int(x) <= upper for x in source["enum_values"]):
                raise ModelError("enum_shape")
        elif source["enum_values"] or source["enum_underlying"] is not None:
            raise ModelError("enum_shape")
    def visit(identity: str, active: frozenset[str]) -> None:
        if identity in active:
            raise ModelError("source_cycle")
        def walk(value: dict) -> None:
            if value["kind"] == "source":
                visit(value["id"], active | {identity})
            elif value["kind"] == "instance":
                for argument in value["arguments"]:
                    walk(argument)
        for member in sources[identity]["members"]:
            walk(member["type"])
    for identity in sources:
        visit(identity, frozenset())


def default_eligible(value: dict, sources: dict[str, dict], bound_roles: dict[str, dict] | None = None) -> bool:
    bound_roles = bound_roles or {}
    if value["kind"] == "primitive":
        return value["id"] not in {"string", "exception", "parse_error"}
    if value["kind"] == "instance":
        template = value["template"]
        if template in {"option", "lookup", "bounded_sequence", "ordered_map", "ordered_set"}:
            return True
        return template == "ordered_entry" and all(default_eligible(a, sources, bound_roles) for a in value["arguments"])
    source = sources[value["id"]]
    if source["kind"] == "sealed_class" or not source["public_default"]:
        return False
    if source["kind"] == "enum":
        return "0" in source["enum_values"]
    binding = bound_roles.get(value["id"], {})
    role = binding.get("role")
    if role in {"result", "validation", "money", "boundary_field", "transition"}:
        return False
    if role in {"option", "lookup"}:
        return (not any(m["required"] for m in source["members"])
                and source["actual_default"][binding["member_map"]["tag"]] == binding["tag_arms"][binding["default_arm"]])
    return all(not m["required"] and default_eligible(m["type"], sources, bound_roles) for m in source["members"])


def binding_arguments(role: str, member_types: dict[str, dict]) -> list[dict]:
    def sequence_element(name: str) -> dict:
        value = member_types[name]
        if value.get("template") != "bounded_sequence" or len(value.get("arguments", [])) != 1:
            raise ModelError("binding_carrier")
        return value["arguments"][0]
    if role in {"option", "lookup", "boundary_field"}:
        return [member_types["value"]]
    if role == "result":
        return [member_types["value"], member_types["error"]]
    if role == "validation":
        return [member_types["value"], sequence_element("errors")]
    if role == "transition":
        return [member_types["state"], sequence_element("events"), member_types["response"]]
    if role == "money":
        if member_types["amount"] != type_ref("decimal"):
            raise ModelError("binding_carrier")
        return [member_types["currency"]]
    if role == "instant":
        if member_types["milliseconds"] != type_ref("i64"):
            raise ModelError("binding_carrier")
        return []
    if role in {"bounded_sequence", "ordered_set"}:
        return [sequence_element("elements")]
    if role == "ordered_entry":
        return [member_types["key"], member_types["value"]]
    if role == "ordered_map":
        entry = sequence_element("entries")
        if entry.get("template") != "ordered_entry" or len(entry.get("arguments", [])) != 2:
            raise ModelError("binding_carrier")
        return entry["arguments"]
    raise ModelError("binding_role")


def binding_obligations(binding: dict, sources: dict[str, dict]) -> list[dict]:
    """Generate obligations, NOT evidence that an arbitrary source satisfies them."""
    source = sources[binding["source_type_id"]]
    obligations = [{"kind": name, "subject": source["id"]} for name in (
        "source_invariant_implies_projection_defined", "semantic_invariant_implies_reconstruction_defined",
        "source_round_trip_all_observable_members", "semantic_round_trip_all_arms",
        "distinct_arms_disjoint", "public_invariants_preserved", "identity_unobservable")]
    if binding["role"] in {"option", "lookup"}:
        obligations.append({"kind": "actual_default_arm_and_public_invariant", "subject": source["id"]})
    for operation, source_operation in sorted(binding["operation_map"].items()):
        obligations.append({"kind": "normal_and_exceptional_operation_commutation", "subject": source_operation, "operation": operation})
    for member in source["members"]:
        obligations.append({"kind": "field_complete_reconstruction", "subject": member["id"]})
    bounded_count(len(obligations), LIMITS["projection_obligations_per_binding"], "projection_obligations")
    return obligations


def validate_binding(binding: dict, sources: dict[str, dict], declarations: dict[str, dict]) -> list[dict]:
    exact(binding, {"schema", "source_type_id", "source_content_sha256", "role", "member_map", "tag_arms",
                    "inferred_argument_ids", "default_arm", "bounds", "operation_map", "binding_sha256"}, "binding_shape")
    if binding["schema"] != SCHEMAS["binding"]:
        raise ModelError("binding_schema")
    raw = {k: v for k, v in binding.items() if k != "binding_sha256"}
    if binding["binding_sha256"] != digest(raw, "binding"):
        raise ModelError("binding_hash")
    source = sources.get(binding["source_type_id"])
    if source is None:
        raise ModelError("binding_source_missing")
    if source["source_sha256"] != binding["source_content_sha256"]:
        raise ModelError("binding_source_stale")
    if source["identity_sensitive"] or source["kind"] == "enum":
        raise ModelError("binding_identity_observable")
    role = binding["role"]
    if role not in ROLE_MEMBERS or (role == "money" and source["kind"] != "readonly_struct"):
        raise ModelError("binding_role")
    exact(binding["member_map"], set(ROLE_MEMBERS[role]), "binding_member_roles")
    member_ids = list(binding["member_map"].values())
    members = {m["id"]: m for m in source["members"]}
    if len(set(member_ids)) != len(member_ids) or any(m not in members for m in member_ids):
        raise ModelError("binding_member_identity")
    member_types = {r: members[m]["type"] for r, m in binding["member_map"].items()}
    arguments = binding_arguments(role, member_types)
    value_type = type_ref("instant") if role == "instant" else instance(role, *arguments)
    validate_type(value_type, sources)
    if binding["inferred_argument_ids"] != [type_id(a) for a in arguments]:
        raise ModelError("binding_argument_inference")
    arms = binding["tag_arms"]
    if role in ARMS:
        exact(arms, set(ARMS[role]), "binding_tag_arms")
        tag_type = member_types["tag"]
        carrier = sources.get(tag_type.get("id"), {})
        values = list(arms.values())
        if carrier.get("kind") != "enum" or any(not isinstance(v, str) or v not in carrier["enum_values"] for v in values) or len(values) != len(set(values)) or set(values) != set(carrier["enum_values"]):
            raise ModelError("binding_tag_carrier")
    elif arms != {}:
        raise ModelError("binding_tag_arms")
    expected_default = "none" if role == "option" else "missing_key" if role == "lookup" else "ineligible"
    if binding["default_arm"] != expected_default:
        raise ModelError("binding_default_arm")
    expected_bounds = {
        "bounded_sequence": {"length": 4096}, "ordered_map": {"length": 4096},
        "ordered_set": {"length": 4096}, "validation": {"errors": 256},
        "transition": {"events": 4096},
    }.get(role, {})
    if binding["bounds"] != expected_bounds:
        raise ModelError("binding_bounds")
    operations = {op["name"] for op in TEMPLATES[role]["operations"]} if role in TEMPLATES else {"milliseconds", "compare", "add_duration", "subtract_duration", "difference"}
    if not isinstance(binding["operation_map"], dict) or not set(binding["operation_map"]) <= operations:
        raise ModelError("binding_operation_set")
    if any(not isinstance(i, str) or i not in declarations for i in binding["operation_map"].values()):
        raise ModelError("binding_operation_identity")
    for operation, target in binding["operation_map"].items():
        signature = declarations[target]
        exact(signature, {"argument_type_ids", "normal_result_type_id"}, "binding_operation_signature")
        if role == "instant":
            instant_id = type_id(type_ref("instant"))
            duration_id = type_id(type_ref("duration"))
            signatures = {
                "milliseconds": ([instant_id], type_id(type_ref("i64"))),
                "compare": ([instant_id, instant_id], type_id(type_ref("i32"))),
                "add_duration": ([instant_id, duration_id], instant_id),
                "subtract_duration": ([instant_id, duration_id], instant_id),
                "difference": ([instant_id, instant_id], duration_id),
            }
            argument_ids, result_id = signatures[operation]
        else:
            if operation == "compare" and not orderable(value_type, sources):
                raise ModelError("binding_operation_signature")
            template = TEMPLATES[role]
            recipe = next(o for o in template["operations"] if o["name"] == operation)
            dependencies = replace_parameters(template["dependencies"], arguments)
            def resolve(name: str) -> str:
                if name == "self": return type_id(value_type)
                if name.startswith("arg"): return type_id(arguments[int(name[3:])])
                if name.startswith("dependency"): return type_id(dependencies[int(name[10:])])
                return type_id(type_ref(name))
            argument_ids, result_id = [resolve(n) for n in recipe["arguments"]], resolve(recipe["result"])
        if signature != {"argument_type_ids": argument_ids, "normal_result_type_id": result_id}:
            raise ModelError("binding_operation_signature")
    return binding_obligations(binding, sources)


def validate_binding_table(bindings: list[dict], sources: dict[str, dict], declarations: dict[str, dict], reachable: set[str]) -> None:
    validate_sources(sources)
    bounded_count(len(bindings), LIMITS["binding_count"], "binding_count")
    ids = [b["source_type_id"] for b in bindings]
    if ids != sorted(set(ids)):
        raise ModelError("binding_order_or_collision")
    if set(ids) != reachable:
        raise ModelError("binding_missing_or_unreachable")
    for binding in bindings:
        validate_binding(binding, sources, declarations)


def finite_projection_check(source_values: list, semantic_values: list, project, reconstruct, operations: list[tuple] = ()) -> None:
    """Finite falsification model only; production must prove the quantified VCs."""
    for value in source_values:
        projected = project(value)
        if projected not in semantic_values or reconstruct(projected) != value:
            raise ModelError("source_round_trip")
    for value in semantic_values:
        if reconstruct(value) not in source_values or project(reconstruct(value)) != value:
            raise ModelError("semantic_round_trip")
    for source_operation, semantic_operation in operations:
        for value in source_values:
            if project(source_operation(value)) != semantic_operation(project(value)):
                raise ModelError("operation_commutation")


def constructor_transaction(members: list[dict], constructor_writes: list[str], initializer_writes: list[str],
                            *, escaped: bool = False, construction_invariant: bool = True, public_invariant: bool = True,
                            eligible_defaults: frozenset[str] = frozenset()) -> list[str]:
    member_map = {m["id"]: m for m in members}
    if escaped:
        raise ModelError("construction_escape")
    seen = set()
    for phase, writes in (("constructor", constructor_writes), ("initializer", initializer_writes)):
        for identity in writes:
            if identity not in member_map:
                raise ModelError("write_target")
            member = member_map[identity]
            if identity in seen:
                raise ModelError("duplicate_initialization")
            if phase == "constructor" and member["required"]:
                raise ModelError("required_constructor_write")
            if phase == "initializer" and member["storage"] != "init_auto":
                raise ModelError("initializer_target")
            seen.add(identity)
        if phase == "constructor" and not construction_invariant:
            raise ModelError("construction_invariant")
    if any(m["required"] and m["id"] not in initializer_writes for m in members):
        raise ModelError("required_missing")
    if any(m["id"] not in seen and m["id"] not in eligible_defaults for m in members):
        raise ModelError("uninitialized_member")
    if not public_invariant:
        raise ModelError("public_invariant")
    return [*constructor_writes, *initializer_writes]


def merge_construction(left: Construction, right: Construction) -> Construction:
    if left.owner is not right.owner or (left.length, left.default_eligible, left.state, left.borrowed) != (right.length, right.default_eligible, right.state, right.borrowed):
        raise ModelError("ownership_join")
    common = left.cells.keys() & right.cells.keys()
    # Different values become an explicit SSA phi, not an arbitrary chosen arm.
    cells = {i: left.cells[i] if left.cells[i] == right.cells[i] else {"phi": [left.cells[i], right.cells[i]]} for i in common}
    return Construction(left.length, left.default_eligible, cells, left.state, left.borrowed, left.owner)
