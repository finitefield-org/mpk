#!/usr/bin/env python3
"""T09 deterministic differential/fuzz corpus; private and never an activation route."""

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
CORPUS = "develop/specs/vectors/java-t09-corpus.json"
MASK64 = (1 << 64) - 1


def states(seed, iterations):
    value = int(seed, 16)
    result = []
    for _ in range(iterations):
        value ^= (value << 13) & MASK64
        value ^= value >> 7
        value ^= (value << 17) & MASK64
        value &= MASK64
        result.append(value)
    return result


def sequence_sha256(seed, iterations):
    payload = b"".join(value.to_bytes(8, "big") for value in states(seed, iterations))
    return hashlib.sha256(payload).hexdigest()


def validate_corpus(build):
    corpus = build.load_json(ROOT / CORPUS)
    build.exact_keys(corpus, ("schema", "owner_test", "task", "candidate_upgrade",
                              "differential_supplements", "fuzz_profiles", "upgrade_case_ids"))
    build.require(corpus["schema"] == "mpk.java.t09.corpus.v0" and corpus["task"] == "JAVA-03-T09",
                  "JAVA_RELEASE_CORPUS_IDENTITY")
    build.require(corpus["owner_test"] == "crates/mpk-cli/tests/java_release_gate.rs",
                  "JAVA_RELEASE_CORPUS_OWNER")
    upgrade = corpus["candidate_upgrade"]
    build.exact_keys(upgrade, ("from_frontend_bundle_id", "from_jar_sha256",
                               "to_frontend_bundle_id", "to_jar_sha256",
                               "unchanged_toolchain_bundle_id", "parent_snapshot_boundary",
                               "removed_child_rehash_bytes", "unchanged_launcher_mode",
                               "unchanged_timeout_seconds"))
    build.require(upgrade == {
        "from_frontend_bundle_id": "frontend.java.java2vir.candidate.v1",
        "from_jar_sha256": "333a050128cddc206474c9bdcca244276c08b246f2a5ba11f55983537cf7cd75",
        "to_frontend_bundle_id": "frontend.java.java2vir.candidate.v2",
        "to_jar_sha256": "aeddb537d396bc7374390d5d01c4dc576c1975e2244dcf7a64de5757fd921558",
        "unchanged_toolchain_bundle_id": "toolchain.java.temurin-25_0_4_1_1.candidate.v1",
        "parent_snapshot_boundary": "complete_descriptor_inventory_before_source",
        "removed_child_rehash_bytes": 175883627,
        "unchanged_launcher_mode": "-Xint",
        "unchanged_timeout_seconds": 120,
    }, "JAVA_RELEASE_CORPUS_CANDIDATE_UPGRADE")
    build.require(len(corpus["differential_supplements"]) == 5 and len(corpus["fuzz_profiles"]) == 5,
                  "JAVA_RELEASE_CORPUS_SHAPE")
    supplements = corpus["differential_supplements"]
    build.require([row["id"] for row in supplements] == [
        "int.division.zero", "int.remainder.zero", "call.dead_branch.normal",
        "call.multiple_entrypoints.caller", "call.multiple_entrypoints.callee",
    ], "JAVA_RELEASE_CORPUS_DIFFERENTIAL_IDS")
    for row in supplements:
        outcome = ("trap" if "trap" in row else "result" if "result" in row else None)
        build.require(outcome is not None and set(row) == {
            "id", "case_id", "method", "arguments", outcome,
        }, "JAVA_RELEASE_CORPUS_DIFFERENTIAL_SHAPE")
        build.require(isinstance(row["arguments"], list)
                      and all(isinstance(value, str) for value in row["arguments"])
                      and isinstance(row[outcome], str), "JAVA_RELEASE_CORPUS_DIFFERENTIAL_VALUE")
    expected_fuzz = [
        ("source_decoder_parser", "pinned_jdk_private_adapter"),
        ("contract_parser", "pinned_jdk_private_adapter"),
        ("diagnostic_normalizer", "pinned_jdk_private_adapter"),
        ("frontend_protocol", "rust_parent_validator"),
        ("resource_capture", "pinned_jdk_private_adapter"),
    ]
    build.require([(row["id"], row["executor"]) for row in corpus["fuzz_profiles"]] == expected_fuzz,
                  "JAVA_RELEASE_CORPUS_FUZZ_IDS")
    build.require(len(set(corpus["upgrade_case_ids"])) == len(corpus["upgrade_case_ids"]) == 12,
                  "JAVA_RELEASE_CORPUS_UPGRADES")
    for profile in corpus["fuzz_profiles"]:
        build.exact_keys(profile, ("id", "seed", "iterations", "executor", "sequence_sha256"))
        build.require(profile["iterations"] == 32 and len(profile["seed"]) == 16,
                      "JAVA_RELEASE_CORPUS_FUZZ")
        build.require(profile["sequence_sha256"] == sequence_sha256(profile["seed"], profile["iterations"]),
                      "JAVA_RELEASE_CORPUS_FUZZ_HASH")
    return corpus


def fixtures(destination, build):
    vector = build.load_json(ROOT / build.VECTOR)
    corpus = validate_corpus(build)
    expected_upgrades = [case["id"] for case in vector["upgrade_cases"]]
    build.require(corpus["upgrade_case_ids"] == expected_upgrades, "JAVA_RELEASE_CORPUS_UPGRADE_OWNERSHIP")
    build.require(vector["case_harness"]["ownership"].startswith("T01 validates strict vector data"),
                  "JAVA_RELEASE_CORPUS_HARNESS")
    release = destination / "release"
    differential = release / "differential"
    differential.mkdir(parents=True)
    records = []
    cases = {case["id"]: case for case in vector["accepted_cases"]}
    for index, case in enumerate(vector["accepted_cases"]):
        root = differential / f"{index:03}"
        for name, source in case["sources"].items():
            target = root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(source, encoding="utf-8")
        for evaluation_index, evaluation in enumerate(case["evaluation_cases"]):
            method = evaluation.get("method", case["methods"][0])
            records.append({
                "id": f"{case['id']}.vector.{evaluation_index}", "case_id": case["id"],
                "case_index": index, "method": method, "arguments": evaluation["arguments"],
                "result": str(evaluation["result"]).lower() if isinstance(evaluation["result"], bool) else evaluation["result"],
            })
    for supplement in corpus["differential_supplements"]:
        build.require(supplement["case_id"] in cases and supplement["method"] in cases[supplement["case_id"]]["methods"],
                      "JAVA_RELEASE_CORPUS_DIFFERENTIAL")
        record = dict(supplement)
        record["case_index"] = next(index for index, case in enumerate(vector["accepted_cases"])
                                    if case["id"] == supplement["case_id"])
        records.append(record)
    lines = []
    for record in records:
        outcome = "trap" if "trap" in record else "result"
        expected = record[outcome]
        lines.append("\t".join((record["id"], str(record["case_index"]), record["case_id"], record["method"],
                                ",".join(str(value).lower() if isinstance(value, bool) else str(value)
                                         for value in record["arguments"]), outcome, str(expected))) + "\n")
    (release / "differential.tsv").write_text("".join(lines), encoding="ascii")
    (release / "fuzz.tsv").write_text("".join(
        "\t".join((profile["id"], profile["seed"], str(profile["iterations"]), profile["executor"],
                    profile["sequence_sha256"])) + "\n" for profile in corpus["fuzz_profiles"]), encoding="ascii")
    (release / "upgrade-ids.txt").write_text("\n".join(corpus["upgrade_case_ids"]) + "\n", encoding="ascii")
    baseline = cases["int.identity"]
    fuzz = release / "fuzz-base"
    fuzz.mkdir()
    (fuzz / "Case.java").write_text(baseline["sources"]["src/vector/Case.java"], encoding="utf-8")
    (fuzz / "contract.json").write_bytes(build.canonical(baseline["contracts"][0]) + b"\n")
    return {"corpus": corpus, "records": records}


def validate_report(report, lowering, fixture, build, lowering_module):
    corpus, records = fixture["corpus"], fixture["records"]
    build.exact_keys(report, ("schema", "differential", "fuzz", "upgrade_case_ids", "network_access",
                              "production_source_execution", "assertions"))
    build.require(report["schema"] == "mpk.java.release_gate_tests.v0" and report["network_access"] is False
                  and report["production_source_execution"] is False, "JAVA_RELEASE_REPORT_IDENTITY")
    build.require(report["upgrade_case_ids"] == corpus["upgrade_case_ids"], "JAVA_RELEASE_REPORT_UPGRADES")
    actual = report["differential"]
    build.require([row["id"] for row in actual] == [row["id"] for row in records],
                  "JAVA_RELEASE_REPORT_DIFFERENTIAL_ORDER")
    lowered = {row["id"]: json.loads(row["envelope"])["ir"]["value"]["units"][0]["functions"]
               for row in lowering["cases"] if row["id"].startswith("accepted/")}
    for wanted, observed in zip(records, actual):
        build.require(observed["case_id"] == wanted["case_id"] and observed["method"] == wanted["method"],
                      "JAVA_RELEASE_REPORT_DIFFERENTIAL_IDENTITY")
        functions = lowered["accepted/" + wanted["case_id"]]
        arguments = [value if isinstance(value, bool) else int(value) for value in wanted["arguments"]]
        vir = lowering_module.evaluate(functions, wanted["method"], arguments)
        if "trap" in wanted:
            build.require(observed.get("trap") == wanted["trap"] and vir.get("trap") is True,
                          "JAVA_RELEASE_REPORT_DIFFERENTIAL_TRAP")
        else:
            vir_result = str(vir["result"]).lower() if isinstance(vir["result"], bool) else str(vir["result"])
            build.require(observed.get("result") == str(wanted["result"]) == vir_result,
                          "JAVA_RELEASE_REPORT_DIFFERENTIAL_RESULT")
    build.require(len(actual) == 102, "JAVA_RELEASE_REPORT_DIFFERENTIAL_COUNT")
    fuzz = report["fuzz"]
    build.require([row["id"] for row in fuzz] == [row["id"] for row in corpus["fuzz_profiles"]],
                  "JAVA_RELEASE_REPORT_FUZZ_ORDER")
    for expected, observed in zip(corpus["fuzz_profiles"], fuzz):
        build.require(observed["seed"] == expected["seed"] and observed["iterations"] == expected["iterations"]
                      and observed["sequence_sha256"] == expected["sequence_sha256"]
                      and observed["executor"] == expected["executor"], "JAVA_RELEASE_REPORT_FUZZ_IDENTITY")
        wanted_cases = 0 if expected["executor"] == "rust_parent_validator" else expected["iterations"]
        build.require(observed["cases"] == wanted_cases and observed["rejections"] == wanted_cases,
                      "JAVA_RELEASE_REPORT_FUZZ_CASES")
    build.require(report["assertions"] >= 300, "JAVA_RELEASE_REPORT_ASSERTIONS")
