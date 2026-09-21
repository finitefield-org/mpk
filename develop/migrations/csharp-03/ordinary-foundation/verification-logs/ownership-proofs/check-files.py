"""Check explicit certificate files with unchanged Go and Rust cores, stage by stage.

Each stage has its own timeout. --hash-only resumes corruption checks after
separately recorded positive acceptance; it never claims positive acceptance.
Build target/release/mpk with cargo before use. The helper only adapts Go reports.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time

GO = r'''package main
import("encoding/json";"os"; checker "github.com/finitefield-org/mpk/go-tools/mpk-checker-ref")
func main(){b,e:=os.ReadFile(os.Args[1]);if e!=nil{panic(e)};r,e:=checker.VerifyCertificateBytes(b);if e!=nil{v,ok:=e.(*checker.VerifyError);if !ok{panic(e)};json.NewEncoder(os.Stdout).Encode(map[string]any{"verdict":"rejected","error_kind":v.Kind,"error_detail":v.Detail,"certificate":checker.HashHex(checker.CertificateHash(b))});os.Exit(1)};json.NewEncoder(os.Stdout).Encode(map[string]any{"verdict":"accepted","report":r})}
'''


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("files", type=Path, nargs="+")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--hash-only", action="store_true")
    parser.add_argument("--timeout-seconds", type=int, default=3600)
    args = parser.parse_args()
    assert args.timeout_seconds > 0
    repo = Path(__file__).resolve().parents[6]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    module = output / "helper"
    module.mkdir()
    (module / "main.go").write_text(GO)
    (module / "go.mod").write_text("module reportadapter\n\ngo 1.23\n\nrequire github.com/finitefield-org/mpk/go-tools/mpk-checker-ref v0.0.0\nreplace github.com/finitefield-org/mpk/go-tools/mpk-checker-ref => " + str(repo / "go-tools/mpk-checker-ref") + "\n")
    go = output / "go-checker"
    rust = repo / "target/release/mpk"
    subprocess.run(["go", "build", "-o", str(go), "."], cwd=module, check=True)
    sources = sorted((repo / "go-tools/mpk-checker-ref").glob("*.go"))
    for crate in ["mpk-core", "mpk-cert", "mpk-kernel", "mpk-theory", "mpk-cli"]:
        sources.extend(sorted((repo / "crates" / crate / "src").rglob("*.rs")))
    provenance = {str(p.relative_to(repo)): digest(p) for p in sources}
    (output / "provenance.json").write_text(json.dumps({"sources": provenance, "go_binary_sha256": digest(go), "rust_binary_sha256": digest(rust), "report_adapter_sha256": digest(module / "main.go")}, indent=2) + "\n")
    results = []
    cases = []
    binaries = {str(go): digest(go), str(rust): digest(rust)}

    def stage(label, command, data, corrupt):
        assert digest(Path(command[0])) == binaries[command[0]]
        print("START", label, flush=True)
        start = time.monotonic()
        timed_out = False
        stdout, stderr = output / (label + ".json"), output / (label + ".stderr")
        with stdout.open("w") as out, stderr.open("w") as err:
            p = subprocess.Popen(command, stdout=out, stderr=err, start_new_session=True)
            try:
                status = p.wait(timeout=args.timeout_seconds)
            except subprocess.TimeoutExpired:
                timed_out = True
                os.killpg(p.pid, signal.SIGKILL)
                status = p.wait()
        row = {"stage": label, "command": command, "wall_seconds": round(time.monotonic()-start, 3), "exit_code": status, "timed_out": timed_out, "input_file_sha256": hashlib.sha256(data).hexdigest(), "stdout_sha256": digest(stdout), "stderr_sha256": digest(stderr), "passed": False}
        results.append(row)
        try:
            assert not timed_out
            report = json.loads(stdout.read_text())
            expected_hash = hashlib.sha256(b"MPK-MODULE-CERT-0.1\x00" + data).hexdigest()
            assert status == int(corrupt)
            assert report["verdict"] == ("rejected" if corrupt else "accepted")
            if label.endswith("go"):
                if corrupt:
                    assert report["error_kind"] == "hash_mismatch"
                    assert report["certificate"] == expected_hash
                else:
                    g = report["report"]
                    assert bytes(g["CertificateHash"]).hex() == expected_hash
                    assert g["AxiomCount"] == 0
            else:
                assert report["hashes"]["certificate"] == expected_hash
                if corrupt:
                    assert report["error_code"] == "KERNEL_HASH_MISMATCH"
                else:
                    assert report["axiom_count"] == 0 and report["error_code"] is None
            assert all(digest(repo / path) == h for path, h in provenance.items())
            row["passed"] = True
            return report
        finally:
            (output / "results.json").write_text(json.dumps(results, indent=2) + "\n")
            print("END", label, "PASS" if row["passed"] else "FAIL", row["wall_seconds"], flush=True)

    for file in args.files:
        original = file.resolve()
        data = bytes.fromhex(original.read_text()) if original.suffix == ".hex" else original.read_bytes()
        initial = digest(original)
        for corrupt in ([True] if args.hash_only else [False, True]):
            submitted = data[:-1] + bytes([data[-1] ^ 1]) if corrupt else data
            label = original.stem + ("-hash" if corrupt else "-positive")
            candidate = output / (label + ".mpcert")
            candidate.write_bytes(submitted)
            g = stage(label + "-go", [str(go), str(candidate)], submitted, corrupt)
            r = stage(label + "-rust", [str(rust), "check", str(candidate)], submitted, corrupt)
            if not corrupt:
                g = g["report"]
                assert (r["module"], r["declaration_count"], r["axiom_count"]) == (g["Module"], g["DeclarationCount"], g["AxiomCount"])
                assert r["hashes"]["export"] == bytes(g["ExportHash"]).hex()
                assert r["hashes"]["axiom_report"] == bytes(g["AxiomReportHash"]).hex()
            assert digest(original) == initial
        cases.append({"file": str(original), "file_sha256": initial,
                      "hash_only": args.hash_only, "passed": True,
                      "accepted_reports_agree": not args.hash_only})
        (output / "cases.json").write_text(json.dumps(cases, indent=2) + "\n")
        print("DONE", original.name, "hash checks only" if args.hash_only else "both checkers and hash checks", flush=True)


if __name__ == "__main__":
    main()
