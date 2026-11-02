#!/usr/bin/env python3
"""Run exact-Go offline T01-W02 probes against original application packages.

Provision the pinned host toolchain separately. This command never downloads
a toolchain or dependency. Its synthetic dependency is supplied through a
local file proxy during preparation, then all measurement commands run with
GOPROXY=off. Outputs are private observations, not production admission.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile


ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
REPORT = ROOT / "develop/migrations/go-practical/t01-w02/module-probes.json"
DEFAULT_GOROOT = Path("/private/tmp/mpk-go-practical-toolchains/go1.25.0-darwin-arm64/go")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def json_bytes(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def run(argv, *, cwd, env, stdin=None, timeout=180):
    completed = subprocess.run(argv, cwd=cwd, env=env, input=stdin, text=True,
                               capture_output=True, timeout=timeout)
    if completed.returncode:
        raise ValueError(f"command failed ({completed.returncode}): {argv[0]}\n{completed.stdout}\n{completed.stderr}")
    return completed.stdout


def environment(goroot, workspace, module_cache, *, target):
    env = {
        "PATH": str(goroot / "bin") + ":/usr/bin:/bin",
        "GOROOT": str(goroot), "GOENV": "off", "GOTOOLCHAIN": "local",
        "GOWORK": "off", "GOPROXY": "off", "GOSUMDB": "off",
        "GONOSUMDB": "", "GOPRIVATE": "", "GOFLAGS": "", "GOEXPERIMENT": "",
        "GOPATH": str(workspace / "gopath"), "GOMODCACHE": str(module_cache),
        "GOCACHE": str(workspace / "build-cache"), "GOTMPDIR": str(workspace / "go-tmp"),
        "CGO_ENABLED": "0", "GOMAXPROCS": "4",
        "GOTELEMETRY": "off",
    }
    if target:
        env.update(GOOS="linux", GOARCH="amd64", GOAMD64="v1")
    return env


def write_files(root, files):
    for name, contents in files.items():
        path = Path(name)
        if path.is_absolute() or ".." in path.parts:
            raise ValueError(f"fixture path escape: {name}")
        destination = root / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(contents)


def source_inventory(root):
    return [
        {"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size,
         "sha256": digest(p.read_bytes())}
        for p in sorted(root.rglob("*")) if p.is_file()
    ]


def readonly(root):
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def cleanup_readonly(root):
    if not root.exists():
        return
    root.chmod(0o700)
    for folder, directories, files in os.walk(root):
        for name in directories:
            (Path(folder) / name).chmod(0o700)
        for name in files:
            (Path(folder) / name).chmod(0o600)
    shutil.rmtree(root)


def normalize(value, replacements):
    if isinstance(value, str):
        for before, after in sorted(replacements, key=lambda x: len(x[0]), reverse=True):
            value = value.replace(before, after)
        return value
    if isinstance(value, list):
        return [normalize(item, replacements) for item in value]
    if isinstance(value, dict):
        return {key: normalize(item, replacements) for key, item in value.items()}
    return value


def prepare_dependency(workspace, matrix, goroot, module_cache):
    dependency = matrix["dependency"]
    module, version = dependency["module"], dependency["version"]
    proxy = workspace / "file-proxy"
    version_dir = proxy / module / "@v"
    version_dir.mkdir(parents=True)
    (version_dir / "list").write_text(version + "\n")
    (version_dir / (version + ".info")).write_bytes(json_bytes({"Version": version, "Time": "2020-01-01T00:00:00Z"}))
    (version_dir / (version + ".mod")).write_text(dependency["files"]["go.mod"])
    archive = version_dir / (version + ".zip")
    with zipfile.ZipFile(archive, "w") as bundle:
        for name, source in sorted(dependency["files"].items()):
            entry = zipfile.ZipInfo(f"{module}@{version}/{name}", (2020, 1, 1, 0, 0, 0))
            entry.external_attr = 0o100644 << 16
            bundle.writestr(entry, source.encode())
    seed = workspace / "seed"
    seed.mkdir()
    (seed / "go.mod").write_text(f"module example.com/probe-seed\n\ngo 1.23\nrequire {module} {version}\n")
    env = environment(goroot, workspace, module_cache, target=False)
    env["GOPROXY"] = proxy.as_uri()
    record = json.loads(run([str(goroot / "bin/go"), "mod", "download", "-json", module], cwd=seed, env=env))
    if "Error" in record:
        raise ValueError(record["Error"])
    sums = f"{module} {version} {record['Sum']}\n{module} {version}/go.mod {record['GoModSum']}\n"
    return sums, {"module": module, "version": version, "archive_sha256": digest(archive.read_bytes()),
                  "sum": record["Sum"], "go_mod_sum": record["GoModSum"],
                  "preparation_transport": "local file proxy; no network",
                  "verification_transport": "GOPROXY=off; sealed prepopulated cache"}


def verify_expectations(case, result):
    expect = case["expect"]
    if result["status"] != expect["status"]:
        raise ValueError(f"{case['id']}: status {result['status']}, expected {expect['status']}\n{json.dumps(result, indent=2)}")
    if "code" in expect and expect["code"] not in {x["code"] for x in result["findings"]}:
        raise ValueError(f"{case['id']}: missing rejection {expect['code']}")
    inside = {x["id"] for x in result["functions"] if x["disposition"] in {"selected_root", "body_dependency"}}
    outside = {x["id"] for x in result["functions"] if x["disposition"] == "type_check_only"}
    if "body_count" in expect and len(inside) != expect["body_count"]:
        raise ValueError(f"{case['id']}: closure size {len(inside)}, expected {expect['body_count']}")
    if not set(expect.get("inside", [])).issubset(inside) or not set(expect.get("outside", [])).issubset(outside):
        raise ValueError(f"{case['id']}: incorrect declaration disposition")
    if "init_declarations" in expect and sum(".init@" in x["id"] for x in result["functions"]) != expect["init_declarations"]:
        raise ValueError(f"{case['id']}: initializer accounting lost declarations")
    if "module_count" in expect and len(result["module_graph"]) != expect["module_count"]:
        raise ValueError(f"{case['id']}: resolved module graph differs")
    if "type_dependency" in expect and expect["type_dependency"] not in result["type_constant_dependencies"]:
        raise ValueError(f"{case['id']}: imported nominal type dependency lost")
    if "standard_package" in expect and expect["standard_package"] not in result["standard_packages"]:
        raise ValueError(f"{case['id']}: original standard-library import not loaded")
    if "root_language" in expect:
        selected = [x for x in result["functions"] if x["disposition"] == "selected_root"]
        if len(selected) != 1 or selected[0]["go_version"] != expect["root_language"]:
            raise ValueError(f"{case['id']}: effective source-language version differs")
    for key, role in (("included_file", "typecheck_source"), ("excluded_file", "build_excluded_source"),
                      ("auxiliary_file", "auxiliary_build_input")):
        if key in expect and not any(x["path"].endswith("/" + expect[key]) and x["source_role"] == role for x in result["observed_loaded_inputs"]):
            raise ValueError(f"{case['id']}: wrong target/tag file selection for {expect[key]}")
    if result["status"] == "closed" and result["pointer_width"] != 64:
        raise ValueError(f"{case['id']}: host size leaked into target sizes")


def execute_case(case, pass_root, matrix, goroot, workspace, module_cache, sums, probe):
    case_root = pass_root / case["id"]
    source = case_root / "app"
    source.mkdir(parents=True)
    write_files(source, case["files"])
    mod = f"module {matrix['module']}\n\ngo {case.get('go_version', '1.23')}\n"
    mode = case.get("dependency_mode", "none")
    extra_roots = []
    dep_root = case_root / "dep"
    if mode != "none":
        required = "v0.9.0" if mode == "versioned_replace" else matrix["dependency"]["version"]
        mod += f"require {matrix['dependency']['module']} {required}\n"
        if mode in {"local", "escape"}:
            write_files(dep_root, matrix["dependency"]["files"])
            mod += f"replace {matrix['dependency']['module']} => ../dep\n"
            if mode == "local":
                extra_roots.append(str(dep_root))
            readonly(dep_root)
        else:
            (source / "go.sum").write_text(sums)
        if mode == "versioned_replace":
            mod += f"replace {matrix['dependency']['module']} v0.9.0 => {matrix['dependency']['module']} v1.0.0\n"
    mod += case.get("extra_mod", "")
    (source / "go.mod").write_text(mod)
    before = source_inventory(source)
    readonly(source)
    request = {"schema": "mpk.go_practical.module_probe_request.v0", "root": str(source),
               "package": matrix["module"], "function": matrix["module"] + ".Selected",
               "capture_roots": extra_roots, "build_tags": case.get("build_tags", [])}
    raw = run([str(probe)], cwd=workspace,
              env=environment(goroot, workspace, module_cache, target=True), stdin=json.dumps(request))
    result = json.loads(raw)
    if source_inventory(source) != before or (source / "SHOULD_NOT_EXIST").exists():
        raise ValueError(f"{case['id']}: application source changed or generation ran")
    verify_expectations(case, result)
    replacements = [(str(source), "<module-root>"), (str(dep_root), "<dependency-root>"),
                    (str(module_cache), "<module-cache>"), (str(goroot), "<toolchain-root>"),
                    (str(workspace), "<probe-work>")]
    return {"id": case["id"], "fixture_inputs": before, "request": normalize(request, replacements),
            "observations": normalize(result, replacements), "source_snapshot_unchanged": True,
            "application_init_or_generate_executed": False}


def verify_host_toolchain(goroot, pin):
    archive = goroot.parents[1] / pin["archive"]
    if archive.stat().st_size != pin["archive_bytes"] or digest(archive.read_bytes()) != pin["archive_sha256"]:
        raise ValueError("host compiler archive identity differs from the official pin")
    expected = []
    with tarfile.open(archive) as bundle:
        for member in bundle:
            if member.isdir():
                continue
            if not member.isfile() or not member.name.startswith("go/") or ".." in Path(member.name).parts:
                raise ValueError("unexpected compiler archive member")
            relative = member.name.removeprefix("go/")
            actual = goroot / relative
            if actual.is_symlink() or not actual.is_file():
                raise ValueError(f"compiler member missing or aliased: {relative}")
            expected_digest = digest(bundle.extractfile(member).read())
            if digest(actual.read_bytes()) != expected_digest:
                raise ValueError(f"compiler member changed: {relative}")
            if bool(actual.stat().st_mode & 0o111) != bool(member.mode & 0o111):
                raise ValueError(f"compiler executable mode changed: {relative}")
            expected.append({"path": relative, "bytes": member.size, "sha256": expected_digest})
    actual_names = {p.relative_to(goroot).as_posix() for p in goroot.rglob("*") if p.is_file()}
    if actual_names != {x["path"] for x in expected}:
        raise ValueError("compiler tree has extra or missing files")
    return {"files": len(expected), "sha256": digest(json_bytes(sorted(expected, key=lambda x: x["path"])))}


def probe_all(goroot, case_ids=None):
    pin = json.loads((HERE / "host-toolchain.json").read_text())
    matrix = json.loads((HERE / "module-cases.json").read_text())
    if matrix["schema"] != "mpk.go_practical.module_cases.v0":
        raise ValueError("unknown case matrix")
    identifiers = [case["id"] for case in matrix["cases"]]
    if len(identifiers) != len(set(identifiers)):
        raise ValueError("duplicate case IDs")
    if case_ids:
        if not set(case_ids).issubset(set(identifiers)):
            raise ValueError("unknown requested case")
        matrix["cases"] = [case for case in matrix["cases"] if case["id"] in case_ids]
    expected_spec = pin["shared_source_witness"]["sha256"]
    if digest((goroot / "doc/go_spec.html").read_bytes()) != expected_spec:
        raise ValueError("host compiler specification differs from registered-source witness")
    host_inventory = verify_host_toolchain(goroot, pin)
    workspace = Path(tempfile.mkdtemp(prefix="mpk-go-practical-w02-", dir="/private/tmp")).resolve()
    try:
        module_cache = workspace / "module-cache"
        for name in ("module-cache", "go-tmp", "gopath", "build-cache"):
            (workspace / name).mkdir()
        host_env = environment(goroot, workspace, module_cache, target=False)
        compiler_version = run([str(goroot / "bin/go"), "version"], cwd=workspace, env=host_env).strip()
        if compiler_version != "go version go1.25.0 darwin/arm64":
            raise ValueError(f"wrong probe compiler: {compiler_version}")
        build = workspace / "tool-build"
        build.mkdir()
        for name in ("go.mod", "go.sum"):
            shutil.copyfile(ROOT / "go-tools/go2vir" / name, build / name)
        shutil.copytree(ROOT / "go-tools/go2vir/vendor", build / "vendor")
        shutil.copyfile(HERE / "module_probe.go", build / "main.go")
        probe = workspace / "module-probe"
        run([str(goroot / "bin/go"), "build", "-mod=vendor", "-trimpath", "-o", str(probe), "."], cwd=build, env=host_env)
        sums, dependency = prepare_dependency(workspace, matrix, goroot, module_cache)
        readonly(module_cache)
        cache_before = source_inventory(module_cache)
        passes = []
        for pass_number in (1, 2):
            observations = []
            for case in matrix["cases"]:
                item = execute_case(case, workspace / f"pass-{pass_number}", matrix,
                                    goroot, workspace, module_cache, sums, probe)
                observations.append(item)
                print(f"pass {pass_number}: {case['id']} -> {item['observations']['status']}", flush=True)
            passes.append(observations)
        if passes[0] != passes[1]:
            raise ValueError("normalized observations differ between fresh source roots")
        if source_inventory(module_cache) != cache_before:
            raise ValueError("verification changed the sealed dependency snapshot")
        vendor = source_inventory(ROOT / "go-tools/go2vir/vendor")
        return {"schema": "mpk.go_practical.module_probe_report.v0", "work_item": "GO-PRAC-T01-W02",
                "scope": "compiler/type/closure feasibility; not VIR, VC, theorem or native release acceptance",
                "compiler_version": compiler_version, "target": "linux/amd64", "host": pin["host"],
                "host_compiler_sha256": digest((goroot / "bin/go").read_bytes()),
                "specification_sha256": expected_spec,
                "host_archive_sha256": pin["archive_sha256"],
                "host_toolchain_inventory": host_inventory,
                "probe_source_sha256": digest((HERE / "module_probe.go").read_bytes()),
                "runner_sha256": digest(Path(__file__).read_bytes()),
                "case_matrix_sha256": digest((HERE / "module-cases.json").read_bytes()),
                "analysis_dependencies": {"vendor_files": len(vendor), "inventory_sha256": digest(json_bytes(vendor))},
                "dependency_snapshot": dependency,
                "deterministic_passes": 2, "case_count": len(matrix["cases"]),
                "dependency_snapshot_unchanged": True, "go_downloads_during_measurement": False,
                "network_scope": "Go proxy/checksum/toolchain/telemetry disabled; no OS-level egress-isolation receipt on macOS",
                "cases": passes[0]}
    finally:
        cleanup_readonly(workspace)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--goroot", type=Path, default=DEFAULT_GOROOT)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    mode.add_argument("--probe", action="store_true", help="debug selected cases without replacing evidence")
    parser.add_argument("--case", action="append", dest="case_ids")
    args = parser.parse_args()
    if args.case_ids and not args.probe:
        raise ValueError("selected cases are debug-only; retained evidence requires the entire matrix")
    result = probe_all(args.goroot.resolve(), args.case_ids)
    data = json_bytes(result)
    if args.write:
        REPORT.parent.mkdir(parents=True, exist_ok=True)
        REPORT.write_bytes(data)
    elif args.check and REPORT.read_bytes() != data:
        raise ValueError("retained module-probes.json differs from actual observations")
    print(f"PASS: {result['case_count']} cases, 2 fresh-source passes; original source and dependency snapshot unchanged")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        print(f"Go practical module probes failed: {error}", file=sys.stderr)
        raise SystemExit(1)
