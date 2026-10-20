"""Run outside sandbox: verify all reviewed bytes and actual staged Git blobs."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import types

repo = Path.cwd()
root = Path(__file__).resolve().parent.parent.relative_to(repo)
base = "11946e845d64e29f242ed8bc5d3477a4ac80dd02"


def git(*args):
    return subprocess.run(["/usr/bin/git", *args], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE).stdout


def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {"path": str(path), "sha256": hashlib.sha256(raw).hexdigest(), "bytes": len(raw)}


assert git("rev-parse", "HEAD").decode().strip() == base
assert git("branch", "--show-current").decode().strip() == "codex/go-practical"
review = root / "input-review/changed-inputs-review.json"
record = json.loads(review.read_text())
assert record["status"] == "passed" and record["selected_files"] == len(record["inputs"])
assert not (record["findings"] or record["whitespace_errors"] or record["git_errors"] or record["missing_links"])
expected = {r["path"]: r for r in record["inputs"]}
assert len(expected) == len(record["inputs"])
expected[str(review)] = seal(review)
for name, original in expected.items():
    assert seal(name) == original, name
assert not git("diff", "--name-only", "-z")
assert not git("ls-files", "--others", "--exclude-standard", "-z")
assert set(git("diff", "--cached", "--name-only", "-z", "HEAD").decode().strip("\0").split("\0")) == set(expected)
rows = git("diff", "--cached", "--raw", "--abbrev=40", "-z", "HEAD").split(b"\0")
assert rows.pop() == b"" and len(rows) == 2 * len(expected)
for index in range(0, len(rows), 2):
    header, name = rows[index].decode().split(), rows[index + 1].decode()
    assert header[4] in {"A", "M"}
    raw = Path(name).read_bytes()
    assert hashlib.sha1(b"blob " + str(len(raw)).encode() + b"\0" + raw).hexdigest() == header[3], name
scanner = Path("scripts/check-no-active-gir.sh")
code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit("\nPY", 1)[0].rsplit("raise SystemExit(main())", 1)[0]
module = types.ModuleType("mpk_source_syntax_review_self")
module.__file__ = str(repo / scanner)
sys.modules[module.__name__] = module
argv = sys.argv
sys.argv = [str(scanner), str(repo), "strict"]
try:
    exec(compile(code, str(scanner), "exec"), module.__dict__)
    matchers = module.load_matchers()
finally:
    sys.argv = argv
assert not list(module.detect(str(review), review.read_text(), matchers))
check = subprocess.run(["/usr/bin/git", "diff", "--no-index", "--check", "--", "/dev/null", str(review)], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
assert check.returncode in {0, 1} and not (check.stdout or check.stderr)
frozen = json.loads((root / "frozen-design-preservation.json").read_text())
assert frozen["frozen_inputs"] == 255
for original in frozen["inputs"]:
    assert seal(original["path"]) == original
assert json.loads((root / "verification.json").read_text())["status"] == "scoped_checks_passed"
output = {"status": "passed", "base_commit": base, "branch": "codex/go-practical",
          "staged_files": len(expected), "reviewed_changed_inputs": len(record["inputs"]),
          "review_self_checked": True, "all_staged_blob_bytes_equal_reviewed_sha256_and_length": True,
          "unstaged_or_untracked_files": 0, "review": seal(review), "verification": seal(root / "verification.json"),
          "frozen_inputs_unchanged": 255, "all_actual_validation_handles_terminal": True,
          "full_T02_gate_passed": False, "CSharp_tests_executed": False, "T02_W01_complete": False, "goal_status": "active"}
Path("/private/tmp/mpk-go-source-syntax-precommit-audit.json").write_text(json.dumps(output, indent=2) + "\n")
print(json.dumps(output))
