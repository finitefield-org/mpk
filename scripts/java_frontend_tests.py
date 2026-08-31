#!/usr/bin/env python3
"""Private T04-T07 JDK conformance executor; no public source CLI entrypoint."""

import importlib.util
import json
import os
from pathlib import Path
import re
import sys
import tempfile
import uuid


ROOT = Path(__file__).resolve().parent.parent
TEST_ROOT = ROOT / "java-tools/tests"
HOST_ENVIRONMENT = {"PATH": "/usr/bin:/bin", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8", "TZ": "UTC"}
SPEC = importlib.util.spec_from_file_location("java_build_owner", Path(__file__).with_name("java_build_inputs.py"))
BUILD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BUILD)
ADMISSION_SPEC = importlib.util.spec_from_file_location("java_admission_owner", Path(__file__).with_name("java_admission_tests.py"))
ADMISSION = importlib.util.module_from_spec(ADMISSION_SPEC)
ADMISSION_SPEC.loader.exec_module(ADMISSION)
LOWERING_SPEC = importlib.util.spec_from_file_location("java_lowering_owner", Path(__file__).with_name("java_lowering_tests.py"))
LOWERING = importlib.util.module_from_spec(LOWERING_SPEC)
LOWERING_SPEC.loader.exec_module(LOWERING)

OBSERVATIONS = (
    "negative-literals", "constant-trees", "implicit-public", "utf16-tab-bmp-nonbmp", "syntax-eof",
    "attribution-unknown-name", "attribution-type", "attribution-uninitialized-local",
    "positive-min-magnitude", "parenthesized-min-magnitude", "positive-long-min-magnitude",
    "planted-source", "planted-class", "planted-processor-service", "jdk-reference-view",
    "excluded-class-default-constructor", "excluded-var-inferred-type", "parameter-slots-254",
    "parameter-slots-256", "diagnostic-listener-abort-1025",
)
OWNED_REJECTIONS = (
    "source.unicode_escape", "source.bom", "source.crlf", "source.missing_lf", "source.nul",
    "source.utf8", "source.noncharacter", "capture.symlink", "capture.hardlink", "capture.unlisted",
    "capture.case_collision", "compiler.unknown_tree", "compiler.error_type", "compiler.unknown_diagnostic",
    "compiler.external_source", "compiler.external_lookup", "compiler.unexpected_output",
)
PRECEDENCE = (
    "capture_before_encoding", "encoding_before_parse", "parse_before_attribution",
    "attribution_before_subset", "diagnostic_overflow_beats_source", "compiler_exception_beats_source",
)
# The T04 harness owns this family; T05/T06 contribute executable cases.
ADMISSION_PRECEDENCE = dict.fromkeys(ADMISSION.PRECEDENCE, "T05: --run-admission")
LOWERING_PRECEDENCE = dict.fromkeys(LOWERING.PRECEDENCE, "T06: --run-lowering")
RUNNER_PRECEDENCE = {"release_before_source": "T07: --run-runner"}
FOLLOW_ON_PRECEDENCE = {}


def fixture_ownership(vector):
    BUILD.require(tuple(case["id"] for case in vector["adapter_observations"]["cases"]) == OBSERVATIONS,
                  "JAVA_FRONTEND_TEST_OBSERVATION_OWNERS")
    for case in vector["adapter_observations"]["cases"]:
        utf16 = case["source"].encode("utf-16-le")
        for stage in ("before_analysis", "after_analysis"):
            for node in case.get(stage, []):
                first, last = node["start_utf16"], node["end_utf16"]
                if 0 <= first <= last <= len(utf16) // 2:
                    spelling = utf16[first * 2:last * 2].decode("utf-16-le")
                    start = len(utf16[:first * 2].decode("utf-16-le").encode("utf-8"))
                    end = len(utf16[:last * 2].decode("utf-16-le").encode("utf-8"))
                    BUILD.require(node["spelling"] == spelling and node["start_utf8"] == start and node["end_utf8"] == end,
                                  "JAVA_FRONTEND_TEST_FIXTURE_SOURCE_COORDINATES")
    actual = tuple(case["id"] for case in vector["rejected_cases"]
                   if case["id"].startswith(("source.", "capture.", "compiler.")))
    BUILD.require(actual == OWNED_REJECTIONS, "JAVA_FRONTEND_TEST_REJECTION_OWNERS")
    BUILD.require({case["id"] for case in vector["precedence_cases"]} == set(PRECEDENCE) | set(ADMISSION_PRECEDENCE) | set(LOWERING_PRECEDENCE) | set(RUNNER_PRECEDENCE) | set(FOLLOW_ON_PRECEDENCE),
                  "JAVA_FRONTEND_TEST_PRECEDENCE_OWNERS")
    for case in vector["limit_cases"]:
        BUILD.require(case["boundary"] == case["limit"] and case["overflow"] == case["limit"] + 1)
    BUILD.require(len(vector["limit_cases"]) == 32)


def fixtures(destination):
    vector = BUILD.load_json(ROOT / BUILD.VECTOR)
    fixture_ownership(vector)
    observations = destination / "observations"
    observations.mkdir()
    for case in vector["adapter_observations"]["cases"]:
        (observations / (case["id"] + ".java.txt")).write_text(case["source"], encoding="utf-8")
    (destination / "observation-ids.txt").write_text(
        "\n".join(case["id"] for case in vector["adapter_observations"]["cases"]) + "\n")
    baseline = next(case for case in vector["accepted_cases"] if case["id"] == "int.identity")
    source = baseline["sources"]["src/vector/Case.java"].encode("utf-8")
    (destination / "baseline.java.txt").write_bytes(source)
    (destination / "baseline-contract.json.txt").write_bytes(BUILD.canonical(baseline["contracts"][0]) + b"\n")
    encodings = {
        "source.unicode_escape": b"// " + b"\\" + b"u0041\n" + source,
        "source.bom": b"\xef\xbb\xbf" + source,
        "source.crlf": source.replace(b"\n", b"\r\n"),
        "source.missing_lf": source[:-1], "source.nul": b"\0" + source,
        "source.utf8": b"\xc0\xaf" + source, "source.noncharacter": "// \uffff\n".encode("utf-8") + source,
    }
    (destination / "encoding").mkdir()
    for name, data in encodings.items():
        (destination / "encoding" / (name + ".bin")).write_bytes(data)
    (destination / "encoding-ids.txt").write_text("\n".join(encodings) + "\n")
    (destination / "limits.tsv").write_text("".join(
        "\t".join((case["id"], str(case["limit"]), case["expected_overflow_phase"], case["expected_overflow_code"])) + "\n"
        for case in vector["limit_cases"]))
    poison = {
        "poison-source/demo/Hidden.java": "package demo; public interface Hidden { static int value() { return 7; } }\n",
        "poison-source/poison/Injected.java": "package poison; public final class Injected { private Injected() {} public static int value() { return 7; } }\n",
        "poison-source/poison/PoisonProcessor.java": """package poison;
public final class PoisonProcessor extends javax.annotation.processing.AbstractProcessor {
    public PoisonProcessor() { System.setProperty("mpk.java.test.processor", "loaded"); }
    @Override public java.util.Set<String> getSupportedAnnotationTypes() { return java.util.Set.of("*"); }
    @Override public javax.lang.model.SourceVersion getSupportedSourceVersion() { return javax.lang.model.SourceVersion.RELEASE_25; }
    @Override public boolean process(java.util.Set<? extends javax.lang.model.element.TypeElement> annotations, javax.annotation.processing.RoundEnvironment round) { return false; }
}
""",
        "poison-module/module-info.java": "module external.poison { exports poisonmodule; }\n",
        "poison-module/poisonmodule/Api.java": "package poisonmodule; public final class Api { private Api() {} public static int value() { return 7; } }\n",
    }
    for name, source in poison.items():
        path = destination / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(source, encoding="utf-8")
    return vector


def validate_report(report, vector):
    BUILD.require(report["schema"] == "mpk.java.frontend_tests.v0")
    BUILD.require(report["compiler_options"] == vector["compiler_session"]["options"], "JAVA_FRONTEND_TEST_OPTIONS")
    BUILD.require(report["compiler_codes"] == vector["diagnostic_normalization"]["compiler_code_allowlist"])
    BUILD.require(report["diagnostic_registry"] == vector["diagnostic_registry"], "JAVA_FRONTEND_TEST_CODEBOOK")
    BUILD.require(report["file_manager_boundary_checks"] == vector["adapter_observations"]["file_manager_boundary_checks"],
                  "JAVA_FRONTEND_TEST_FILE_MANAGER")
    BUILD.require(report["limits"] == [dict(id=case["id"], maximum=case["limit"],
                  code=case["expected_overflow_code"], phase=case["expected_overflow_phase"]) for case in vector["limit_cases"]])
    definitions = {definition["code"]: definition for definition in vector["diagnostic_registry"]}
    failures = {failure["id"]: failure for failure in report["failures"]}
    BUILD.require(len(failures) == len(report["failures"]), "JAVA_FRONTEND_TEST_DUPLICATE_CASE")
    for failure in failures.values():
        encoded = failure["envelope"].encode("utf-8")
        envelope = BUILD.strict_json(encoded, maximum=BUILD.MAX_JSON, canonical_transport=True)
        BUILD.require(set(envelope) == {"schema", "status", "phase", "semantic_context", "selection", "diagnostics", "rejected_features"})
        BUILD.require(envelope["schema"] == "mpk.frontend.cli.v1" and envelope["semantic_context"] == vector["semantic_context_fixture"])
        definition = definitions[failure["code"]]
        BUILD.require(failure["status"] == definition["status"] and failure["exit"] == definition["exit"])
        BUILD.require(definition["phase"] == "started_phase" or failure["phase"] == definition["phase"])
        BUILD.require(envelope["status"] == failure["status"] and envelope["phase"] == failure["phase"])
        branch = "rejected_features" if failure["status"] == "rejected" else "diagnostics"
        BUILD.require(not envelope["diagnostics" if branch == "rejected_features" else "rejected_features"])
        BUILD.require(1 <= len(envelope[branch]) <= 1024 and len(envelope[branch]) == failure["issues"])
        keys = []
        for issue in envelope[branch]:
            BUILD.require(issue["code"] == failure["code"] and issue["message"] == definition["message"])
            BUILD.require(set(issue) <= {"code", "message", "span", "function_id"})
            span = issue.get("span", {})
            if span:
                BUILD.require(set(span) == {"normalized_path", "start", "end"} and 0 <= span["start"] < span["end"])
                BUILD.require(re.fullmatch(r"[A-Za-z0-9_./-]+", span["normalized_path"]) is not None)
            keys.append((span.get("normalized_path", ""), span.get("start", -1), issue["code"], issue["message"],
                         issue.get("function_id", ""), span.get("end", -1)))
        BUILD.require(keys == sorted(keys), "JAVA_FRONTEND_TEST_DIAGNOSTIC_ORDER")
    for case in vector["rejected_cases"]:
        if case["id"] in OWNED_REJECTIONS:
            actual = failures[case["id"]]
            BUILD.require(all(actual[key] == case["expected_" + key] for key in ("code", "status", "phase")),
                          "JAVA_FRONTEND_TEST_REJECTION")
    for case in vector["precedence_cases"]:
        if case["id"] in PRECEDENCE:
            actual = failures["precedence/" + case["id"]]
            BUILD.require(all(actual[key] == case["expected_" + key] for key in ("code", "status", "phase")
                              if "expected_" + key in case), "JAVA_FRONTEND_TEST_PRECEDENCE")
    BUILD.require(len(report["observations"]) == len(OBSERVATIONS))
    for expected, actual in zip(vector["adapter_observations"]["cases"], report["observations"]):
        case_id = expected["id"]
        BUILD.require(actual["id"] in (case_id, "observation/" + case_id))
        if expected["diagnostics_seen"]:
            phase = "source" if expected["final_phase"] == "parse" else "typecheck"
            code = ("JAVA_FRONTEND_DIAGNOSTIC_BUDGET" if expected["diagnostics_seen"] == 1025 else
                    "JAVA_SOURCE_PARSE" if phase == "source" else "JAVA_SOURCE_DIAGNOSTIC")
            BUILD.require(actual["code"] == code and actual["phase"] == phase, "JAVA_FRONTEND_TEST_OBSERVED_DIAGNOSTIC")
            if expected["diagnostics_seen"] == 1:
                diagnostic = expected["diagnostics"][0]
                first, last = diagnostic["start_utf16"], diagnostic["end_utf16"]
                envelope = json.loads(actual["envelope"])
                issue = envelope["diagnostics"][0]
                if first == last:
                    BUILD.require("span" not in issue)
                else:
                    def byte_offset(position):
                        return len(expected["source"].encode("utf-16-le")[:position * 2].decode("utf-16-le").encode("utf-8"))
                    BUILD.require(issue["span"] == {"normalized_path": "src/demo/Probe.java", "start": byte_offset(first), "end": byte_offset(last)})
        else:
            BUILD.require(actual["status"] == "analyzed" and actual["manager_closed"] is True, "JAVA_FRONTEND_TEST_ANALYSIS")
            BUILD.require(actual["system_files_returned"] == expected["system_files_returned"] == 0 and actual["output_attempts"] == 0)
            for stage in ("before_analysis", "after_analysis"):
                if stage in expected:
                    rows = [{key: value for key, value in row.items() if key not in ("line", "tab_expanded_column")} for row in expected[stage]]
                    if actual[stage] != rows:
                        # Bounded diagnostics for these public, frozen test fixtures only.
                        for index, (wanted, observed) in enumerate(zip(rows, actual[stage])):
                            if wanted != observed:
                                delta = {key: [wanted.get(key), observed.get(key)] for key in set(wanted) | set(observed)
                                         if wanted.get(key) != observed.get(key)}
                                print(f"{case_id} {stage} row {index}: {json.dumps(delta, sort_keys=True)[:1200]}", file=sys.stderr)
                                break
                    BUILD.require(actual[stage] == rows, "JAVA_FRONTEND_TEST_TREE_OBSERVATION")


def runner_fixtures(destination, vector):
    candidate = BUILD.load_json(ROOT / "release/build-inputs/java/bundle-candidate.json")
    registry = BUILD.load_json(ROOT / "release/build-inputs/java/bundle-registry.json")
    substitutions = {
        "{selection.compilation}": "vector", "{each selection.sources in stored order}": "src/vector/Case.java",
        "{each selection.contracts in stored order}": "contracts/selected.json",
        "{each selection.methods in stored order}": "vector.Case::f(int)->int",
        "{release.frontend_bundle_id}": candidate["frontend_bundles"][0]["bundle_id"],
        "{release.frontend_sha256}": candidate["frontend_bundles"][0]["main"]["binary_sha256"],
        "{release.registry_sha256}": registry["registry_sha256"],
        "{release.toolchain_bundle_id}": candidate["toolchain_bundles"][0]["bundle_id"],
        "{release.toolchain_distribution_sha256}": candidate["toolchain_bundles"][0]["distribution_sha256"],
    }
    arguments = [substitutions.get(token, token) for token in vector["launcher_contract"]["frontend_argv_template"]]
    BUILD.require(all("{" not in token and "\n" not in token for token in arguments), "JAVA_RUNNER_FIXTURE")
    (destination / "runner-arguments.txt").write_text("\n".join(arguments) + "\n", encoding="ascii")


def worker(admission=False, lowering=False, runner=False):
    # This diagnostic stream is for compilation of our test/project sources only.
    # Selected-source diagnostics are still normalized by the Java implementation.
    execute = BUILD.execute
    def compile_with_diagnostics(argv, **kwargs):
        result = execute(argv, **kwargs)
        if argv[0].endswith("/javac") and result[0] != 0:
            sys.stderr.buffer.write(result[2])
        return result
    BUILD.execute = compile_with_diagnostics
    descriptor, jar = BUILD.compile_project()
    # These planted dependencies live only on the private test classpath.
    poison_arguments = ["/mpk/toolchain/jdk/bin/javac", *BUILD.RECIPE["compiler_jvm_arguments"],
                        "--release", "25", "-encoding", "UTF-8", "-g:none", "-proc:none", "-implicit:none",
                        "-Xlint:all", "-Werror", "--class-path", "/work/empty"]
    for output, sources in (("/work/poison-classes", ["poison-source/poison/Injected.java", "poison-source/poison/PoisonProcessor.java"]),
                            ("/work/poison-modules/external.poison", ["poison-module/module-info.java", "poison-module/poisonmodule/Api.java"])):
        source_path = "/mpk/tests/poison-module" if "poison-modules" in output else "/work/empty"
        code, stdout, stderr = compile_with_diagnostics([*poison_arguments, "--source-path", source_path, "-d", output,
                                                        *["/mpk/tests/" + source for source in sources]], environment=BUILD.ENVIRONMENT)
        BUILD.require(code == 0 and not stdout and not stderr, "JAVA_FRONTEND_TEST_POISON")
    service = Path("/work/poison-classes/META-INF/services/javax.annotation.processing.Processor")
    service.parent.mkdir(parents=True)
    service.write_text("poison.PoisonProcessor\n")
    arguments = ["/mpk/toolchain/jdk/bin/javac", *BUILD.RECIPE["compiler_jvm_arguments"],
                 "--release", "25", "-encoding", "UTF-8", "-g:none", "-proc:none", "-implicit:none",
                 "-Xlint:all", "-Werror", "--class-path", "/work/java2vir.jar",
                 "--source-path", "/work/empty", "--processor-path", "/work/empty",
                 "--module-path", "/work/empty", "-d", "/work/test-classes",
                 "/mpk/tests/RunnerTests.java" if runner else "/mpk/tests/LoweringTests.java" if lowering else "/mpk/tests/AdmissionTests.java" if admission else "/mpk/tests/FrontendTests.java"]
    code, stdout, stderr = compile_with_diagnostics(arguments, environment=BUILD.ENVIRONMENT)
    BUILD.require(code == 0 and not stdout and not stderr, "JAVA_FRONTEND_TEST_COMPILE")
    code, stdout, stderr = execute(
        ["/mpk/toolchain/jdk/bin/java", *BUILD.JVM_ARGUMENTS, "-cp", "/work/java2vir.jar:/work/test-classes:/work/poison-classes",
         "mpk.java2vir.RunnerTests" if runner else "mpk.java2vir.LoweringTests" if lowering else "mpk.java2vir.AdmissionTests" if admission else "mpk.java2vir.FrontendTests"],
        environment=BUILD.ENVIRONMENT, timeout=300)
    if stderr:
        sys.stderr.buffer.write(stderr)
    BUILD.require(code == 0 and not stderr, "JAVA_FRONTEND_TEST_FAILED")
    report = BUILD.strict_json(stdout, maximum=BUILD.MAX_REPORT, canonical_transport=True)
    report["candidate_inventory"] = BUILD.candidate_inventory(jar, descriptor)
    sys.stdout.buffer.write(BUILD.canonical(report) + b"\n")


def run(admission=False, lowering=False, runner=False):
    inputs = BUILD.load_toolchain()
    BUILD.validate_active_boundary()
    descriptor = BUILD.load_descriptor(update=True)
    archive = BUILD.check_cache(inputs)
    with tempfile.TemporaryDirectory(prefix="mpk-java-t04-", dir="/tmp") as temporary:
        root = Path(temporary).resolve()
        root.chmod(0o755)
        archive_copy = root / BUILD.ARCHIVE_NAME
        BUILD.copy_verified(archive, archive_copy, BUILD.archive_record(inputs), inputs["archive_policy"]["max_archive_bytes"])
        BUILD.extract_jdk(archive_copy, root / "jdk", inputs)
        project = root / "project"
        project.mkdir()
        for record in descriptor["project_files"]:
            target = project / record["path"]
            target.parent.mkdir(parents=True, exist_ok=True)
            BUILD.copy_verified(ROOT / BUILD.PROJECT / record["path"], target, record, BUILD.MAX_SOURCE)
        frozen = root / "inputs"
        frozen.mkdir()
        (frozen / "toolchain.json").write_bytes(BUILD.canonical(inputs) + b"\n")
        (frozen / "build-inputs.json").write_bytes(BUILD.canonical(descriptor) + b"\n")
        scripts = root / "build"
        scripts.mkdir()
        for name in ("java_build_inputs.py", "java_frontend_tests.py", "java_admission_tests.py", "java_lowering_tests.py"):
            (scripts / name).write_bytes(BUILD.read_bytes(ROOT / "scripts" / name, BUILD.MAX_SOURCE))
        tests = root / "tests"
        tests.mkdir()
        (tests / "FrontendTests.java").write_bytes(BUILD.read_bytes(TEST_ROOT / "FrontendTests.java", BUILD.MAX_SOURCE))
        vector = fixtures(tests)
        if runner:
            (tests / "RunnerTests.java").write_bytes(BUILD.read_bytes(TEST_ROOT / "RunnerTests.java", BUILD.MAX_SOURCE))
            runner_fixtures(tests, vector)
        if admission:
            (tests / "AdmissionTests.java").write_bytes(BUILD.read_bytes(TEST_ROOT / "AdmissionTests.java", BUILD.MAX_SOURCE))
            admission_fixtures = ADMISSION.fixtures(tests, BUILD)
        if lowering:
            (tests / "LoweringTests.java").write_bytes(BUILD.read_bytes(TEST_ROOT / "LoweringTests.java", BUILD.MAX_SOURCE))
            lowering_fixtures = LOWERING.fixtures(tests, BUILD)
        config = root / "docker-config"
        config.mkdir(mode=0o700)
        docker = BUILD.docker_prefix(config)
        name = "mpk-java-t04-" + uuid.uuid4().hex
        argv = [*docker, "run", "--rm", "--pull=never", "--platform=linux/amd64", "--name", name,
                "--hostname=mpk-java-t04", "--network=none", "--ipc=none", "--read-only", "--user=65534:65534",
                "--cap-drop=ALL", "--security-opt=no-new-privileges", "--pids-limit=128",
                "--memory=1073741824", "--memory-swap=1073741824", "--ulimit=core=0:0", "--ulimit=nofile=1024:1024",
                "--tmpfs=/work:rw,nosuid,nodev,noexec,size=134217728,uid=65534,gid=65534,mode=0700", "--workdir=/work"]
        if runner:
            argv.append("--tmpfs=/mpk/tmp:rw,nosuid,nodev,noexec,size=67108864,uid=65534,gid=65534,mode=0700")
        for source, target in (("jdk", "/mpk/toolchain/jdk"), ("project", "/mpk/project"),
                               ("inputs", "/mpk/inputs"), ("build", "/mpk/build"), ("tests", "/mpk/tests")):
            BUILD.require("," not in str(root / source))
            argv.extend(["--mount", f"type=bind,src={root / source},dst={target},readonly"])
        argv.extend([BUILD.IMAGE, "/usr/bin/env", "-i"])
        argv.extend(key + "=" + value for key, value in BUILD.ENVIRONMENT.items())
        argv.extend(["/usr/local/bin/python3", "-I", "-S", "-B", "/mpk/build/java_frontend_tests.py",
                     "_worker-runner" if runner else "_worker-lowering" if lowering else "_worker-admission" if admission else "_worker"])
        try:
            code, stdout, stderr = BUILD.execute(argv, environment=HOST_ENVIRONMENT, timeout=600)
            if stderr:
                sys.stderr.buffer.write(stderr)
            BUILD.require(code == 0 and not stderr, "JAVA_FRONTEND_TEST_CONTAINER")
            report = BUILD.strict_json(stdout, maximum=BUILD.MAX_REPORT, canonical_transport=True)
            BUILD.require(report["candidate_inventory"]["project_files_sha256"] == BUILD.sha256(BUILD.canonical(descriptor["project_files"])))
            BUILD.require(BUILD.project_records(ROOT / BUILD.PROJECT) == descriptor["project_files"])
            BUILD.validate_active_boundary()
            if runner:
                BUILD.require(report["schema"] == "mpk.java.runner_tests.v0" and report["assertions"] >= 100
                              and report["precedence"] == "release_before_source", "JAVA_RUNNER_REPORT")
                envelope = BUILD.strict_json(report["envelope"].encode("utf-8"), canonical_transport=True)
                BUILD.require(envelope["status"] == "frontend-error" and envelope["phase"] == "metadata"
                              and "vir" not in envelope and "source_manifest" not in envelope, "JAVA_RUNNER_REPORT")
            elif lowering:
                LOWERING.validate_report(report, lowering_fixtures, BUILD)
            elif admission:
                ADMISSION.validate_report(report, admission_fixtures, BUILD)
            else:
                validate_report(report, vector)
                report["admission_precedence"] = ADMISSION_PRECEDENCE
                report["lowering_precedence"] = LOWERING_PRECEDENCE
                report["runner_precedence"] = RUNNER_PRECEDENCE
                report["follow_on_precedence"] = FOLLOW_ON_PRECEDENCE
            sys.stdout.buffer.write(BUILD.canonical(report) + b"\n")
        finally:
            BUILD.execute([*docker, "rm", "--force", name], environment=HOST_ENVIRONMENT, limit=BUILD.MAX_STDERR, timeout=30)
            code, remaining, stderr = BUILD.execute(
                [*docker, "container", "ls", "--all", "--filter", f"name=^/{name}$", "--format", "{{.ID}}"],
                environment=HOST_ENVIRONMENT, limit=BUILD.MAX_STDERR, timeout=30)
            BUILD.require(code == 0 and not remaining and not stderr, "JAVA_FRONTEND_TEST_CLEANUP")


def main():
    os.umask(0o022)
    if sys.argv[1:] == ["_worker"]:
        worker()
    elif sys.argv[1:] == ["_worker-admission"]:
        worker(admission=True)
    elif sys.argv[1:] == ["_worker-lowering"]:
        worker(lowering=True)
    elif sys.argv[1:] == ["_worker-runner"]:
        worker(runner=True)
    elif sys.argv[1:] == ["run"]:
        run()
    elif sys.argv[1:] == ["run-admission"]:
        run(admission=True)
    elif sys.argv[1:] == ["run-lowering"]:
        run(lowering=True)
    elif sys.argv[1:] == ["run-runner"]:
        run(runner=True)
    elif sys.argv[1:] == ["check-fixtures"]:
        with tempfile.TemporaryDirectory(prefix="mpk-java-t04-fixtures-", dir="/tmp") as directory:
            fixtures(Path(directory))
    elif sys.argv[1:] == ["check-admission-fixtures"]:
        with tempfile.TemporaryDirectory(prefix="mpk-java-t05-fixtures-", dir="/tmp") as directory:
            ADMISSION.fixtures(Path(directory), BUILD)
    elif sys.argv[1:] == ["check-lowering-fixtures"]:
        with tempfile.TemporaryDirectory(prefix="mpk-java-t06-fixtures-", dir="/tmp") as directory:
            LOWERING.fixtures(Path(directory), BUILD)
    else:
        raise BUILD.BuildFailure("JAVA_FRONTEND_TEST_USAGE", 64)


if __name__ == "__main__":
    try:
        main()
    except BUILD.BuildFailure as error:
        print(error.code, file=sys.stderr)
        raise SystemExit(error.exit_code)
