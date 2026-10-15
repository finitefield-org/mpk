"""Audit ordinary source-use traces against captured source syntax.

This checks exact captured input/body bytes and the ordinary program's trace
metadata. It does not prove W06 DefaultUseForbidden, including implicit/native
publications and whole-body execution.
"""

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
MIGRATION = ROOT / "develop/migrations/csharp-03"
FOUNDATION = MIGRATION / "ordinary-foundation"
OUTPUT = Path(__file__).resolve().parent.parent / "source-use-inventory.json"
TRACE_RECEIPT = Path(__file__).resolve().parent.parent / "source-use-traces/verification.json"


def read(path):
    return json.loads(path.read_text())


def sha(data):
    return hashlib.sha256(data).hexdigest()


def raw_type_id(key):
    if key is None or key.startswith(("intrinsic_argument:", "source_exception:")):
        return None
    length, tail = key.split(":", 1)
    assert length.isdecimal() and str(int(length)) == length
    assert len(tail) >= int(length)
    return tail[: int(length)]


def source_fixture(case_id):
    if case_id.startswith("binding-vc-"):
        return MIGRATION / "binding-vc", case_id.removeprefix("binding-vc-")
    if case_id.startswith("boundary-"):
        return FOUNDATION / "domain-sources", case_id.removeprefix("boundary-")
    if case_id.startswith("extra-validation-"):
        return FOUNDATION / "outcome-sources", case_id.removeprefix("extra-")
    if case_id in {
        "extra-bool-float-map",
        "extra-bool-nullable-map",
        "extra-shared-lookup-maps",
    }:
        return FOUNDATION / "collection-sources", case_id.removeprefix("extra-")
    directory = {
        "bool-float-entry": "entry-sources",
        "remapped-boundary-sequence": "projection-sources",
        "float-make-commutation": "binding-result-sources",
    }.get(case_id)
    assert directory is not None, case_id
    return FOUNDATION / directory, case_id


manifest_path = FOUNDATION / "binding-defaults/certificates.json"
manifest = read(manifest_path)
previous = read(FOUNDATION / "verification-logs/binding-defaults/verification.json")
pins = {row["id"]: row for row in previous["pins"]}
assert len(pins) == len(manifest["sources"]) == 45
rows = []
for source in manifest["sources"]:
    pin = pins[source["id"]]
    cert = (FOUNDATION / "binding-defaults" / (source["id"] + ".hex")).read_bytes()
    assert sha(cert) == pin["raw_file_sha256"]
    assert len(bytes.fromhex(cert.decode().strip())) == pin["bytes"]
    assert source["metadata"]["certificate_sha256"] == pin["certificate_sha256"]
    pending = source["metadata"]["pending_defaults"]
    if not pending:
        continue
    case_id = source["id"]
    defaults = {
        row["projection_id"]: row
        for row in source["metadata"]["actual_defaults"]
    }
    for row in pending:
        sequent = row["sequent"]
        assert sequent["kind"] == "actual_default"
        assert sequent["owner_id"] in defaults
        assert defaults[sequent["owner_id"]]["declared_arm"] == "ineligible"
        assert row["reason"] == "source_use_proof_required"
        assert not sequent["subjects"] and not sequent["assumptions"]
        assert len(sequent["goals"]) == 1
        assert (
            "Mpk.CSharp.Binding.DefaultUseForbidden." + sequent["owner_id"]
            in json.dumps(sequent["goals"])
        )
    directory, request_id = source_fixture(case_id)
    requests = [x for x in read(directory / "requests.json") if x["id"] == request_id]
    responses = [x for x in read(directory / "responses.json") if x["id"] == request_id]
    assert len(requests) == len(responses) == 1, case_id
    request, response = requests[0], responses[0]
    assert "reject" not in response and "facts" in response, case_id
    facts = response["facts"]
    traces = source["metadata"]["source_use_traces"]
    assert len(traces) == len(pending), case_id
    assert {x["projection_id"] for x in traces} == {
        x["sequent"]["owner_id"] for x in pending
    }, case_id
    assert request["roots"] == facts["selected_root_ids"], case_id
    inputs = {(item["kind"], item["path"]): item for item in facts["input_files"]}
    assert len(inputs) == len(request["inputs"]), case_id
    source_hashes = set()
    for item in request["inputs"]:
        key = item["kind"], item["path"]
        pinned = inputs[key]
        raw = item["utf8"].encode()
        assert sha(raw) == pinned["raw_sha256"], (case_id, key)
        assert len(raw) == pinned["size_bytes"], (case_id, key)
        if item["kind"] == "source":
            source_hashes.add(sha(raw))
    captured_types = {row["id"]: row for row in facts["types"]}
    for default in defaults.values():
        assert default["source_type_id"] in captured_types, case_id
    operation_nodes = 0
    default_nodes = []
    initialization_plans = 0
    data_steps = 0
    expected_trace_nodes = {trace["projection_id"]: [] for trace in traces}
    for callable in facts["callables"]:
        assert callable["source_sha256"] in source_hashes, case_id
        body = callable["body_utf8"].encode()
        assert sha(body) == callable["body_sha256"], (case_id, callable["id"])
        operations = json.loads(body)
        assert isinstance(operations, list), case_id
        operation_nodes += len(operations)
        for trace in traces:
            for ordinal, operation in enumerate(operations):
                if raw_type_id(operation["type"]) == trace["source_type_id"]:
                    expected_trace_nodes[trace["projection_id"]].append(
                        {
                            "callable_id": callable["id"],
                            "body_sha256": callable["body_sha256"],
                            "node_ordinal": ordinal,
                            "kind": operation["kind"],
                            "implicit": operation["implicit"],
                            "initialization_plan": any(
                                p["node_ordinal"] == ordinal
                                for p in callable.get("initialization_plans", [])
                            ),
                        }
                    )
        default_nodes.extend(
            {"callable_id": callable["id"], "type": operation["type"]}
            for operation in operations
            if operation["kind"] == "DefaultValue"
        )
        initialization_plans += len(callable.get("initialization_plans", []))
        data_steps += len(callable.get("data_steps", []))
    for trace in traces:
        assert trace["source_ir_sha256"] == source["metadata"]["source_ir_sha256"]
        assert trace["captured_source_available"] is True
        assert trace["captured_callable_count"] == len(facts["callables"])
        assert trace["native_execution_proof_pending"] is True
        assert trace["nodes"] == expected_trace_nodes[trace["projection_id"]], case_id
    rows.append(
        {
            "id": case_id,
            "source_ir_sha256": source["metadata"]["source_ir_sha256"],
            "binding_vc_sha256": source["metadata"]["binding_vc_sha256"],
            "fixture": str(directory.relative_to(ROOT)),
            "captured_input_sha256": {x["path"]: x["raw_sha256"] for x in facts["input_files"]},
            "pending_condition_ids": [x["sequent"]["id"] for x in pending],
            "callables": len(facts["callables"]),
            "operation_nodes": operation_nodes,
            "explicit_default_nodes": default_nodes,
            "data_steps": data_steps,
            "initialization_plans": initialization_plans,
            "ordinary_trace_records": len(traces),
            "ordinary_trace_nodes": sum(len(x["nodes"]) for x in traces),
        }
    )
assert len(rows) == 22
assert sum(len(x["pending_condition_ids"]) for x in rows) == 32
assert sum(x["operation_nodes"] for x in rows) == 58
assert not any(x["explicit_default_nodes"] for x in rows)
record = {
    "status": "verified_ordinary_source_use_trace_only",
    "scope": "Original 32 ineligible-default W06 conditions in 22 binding contexts. Ordinary trace nodes are reconciled with exact captured input and callable-body hashes; no DefaultUseForbidden definition, source-use theorem or application proof is supplied.",
    "selection_reason": "These 32 conditions are the remaining actual_default obligations. The ordinary program now carries source-bound proof inputs; implicit/native publications and source execution still require ordinary reasoning.",
    "manifest_sha256": sha(manifest_path.read_bytes()),
    "contexts": len(rows),
    "pending_conditions": sum(len(x["pending_condition_ids"]) for x in rows),
    "captured_operation_nodes": sum(x["operation_nodes"] for x in rows),
    "explicit_default_nodes": 0,
    "captured_data_steps": sum(x["data_steps"] for x in rows),
    "captured_initialization_plans": sum(x["initialization_plans"] for x in rows),
    "ordinary_trace_records": sum(x["ordinary_trace_records"] for x in rows),
    "ordinary_trace_nodes": sum(x["ordinary_trace_nodes"] for x in rows),
    "sources": rows,
    "proofs_discharged": 0,
    "full_gate": "deferred_to_T06_W12",
}
OUTPUT.write_text(json.dumps(record, indent=2) + "\n")
TRACE_RECEIPT.parent.mkdir(parents=True, exist_ok=True)
TRACE_RECEIPT.write_text(
    json.dumps(
        {
            "status": "passed_scoped_source_use_trace_audit",
            "work_item": "CSHARP-03-T06-W09",
            "internal_unit": 4,
            "selection_reason": "The binding-default generator now serializes proof inputs for the 32 original ineligible source uses; original source facts, metadata links and all 45 unchanged certificate pins are the affected scope.",
            "original_contexts": 45,
            "ineligible_contexts": record["contexts"],
            "pending_default_conditions": record["pending_conditions"],
            "ordinary_trace_records": record["ordinary_trace_records"],
            "ordinary_trace_nodes": record["ordinary_trace_nodes"],
            "preserved_certificate_pins": len(pins),
            "manifest_sha256": record["manifest_sha256"],
            "source_inventory_sha256": sha(OUTPUT.read_bytes()),
            "prior_dual_checker_receipt": "../verification.json",
            "conditions_discharged": 0,
            "proofs_discharged": 0,
            "scope_limit": "Source-bound trace only. DefaultUseForbidden, implicit/native publication semantics, complete source execution and application proofs remain pending.",
            "full_gate": "deferred_to_T06_W12",
        },
        indent=2,
    )
    + "\n"
)
print({key: record[key] for key in ("status", "contexts", "pending_conditions", "captured_operation_nodes", "explicit_default_nodes")})
