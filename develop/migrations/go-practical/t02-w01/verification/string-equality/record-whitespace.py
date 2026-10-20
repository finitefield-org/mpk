"""Keep original Cargo output bytes while checking all other staged changes."""
from pathlib import Path
import datetime
import hashlib
import json
import subprocess

OUT = Path(__file__).resolve().parent
names = ["initial/foundation/stdout.log", "semantic-controls/selected/stdout.log"]
paths = [str((OUT/"epochs"/name).relative_to(Path.cwd())) for name in names]
commands = [["/usr/bin/git", "diff", "--cached", "--check"],
            ["/usr/bin/git", "diff", "--cached", "--check", "--", ".", *(":(exclude)"+path for path in paths)]]
results = []
for command in commands:
    result = subprocess.run(command, capture_output=True, check=False)
    results.append({"command": command, "exit_code": result.returncode,
                    "stdout": result.stdout.decode(), "stderr": result.stderr.decode()})
assert results[0]["exit_code"] == 2 and not results[0]["stderr"]
lines = results[0]["stdout"].splitlines()
assert len(lines) == 2 and {line.rsplit(":", 2)[0] for line in lines} == set(paths)
assert all(line.endswith(": new blank line at EOF.") for line in lines)
assert results[1]["exit_code"] == 0 and not results[1]["stdout"] and not results[1]["stderr"]
record = {"status": "passed_with_original_output_retained", "observed_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
          "actual_checks": results, "exact_raw_output_warnings": [{"path":path,"sha256":hashlib.sha256(Path(path).read_bytes()).hexdigest(),"bytes":Path(path).stat().st_size} for path in paths],
          "all_other_staged_files_pass_whitespace_check": True,
          "original_Cargo_output_bytes_modified": False, "obsolete_IR_matcher_scope_seal_or_exclusion_changes": False}
(OUT/"git-whitespace-review.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({key:record[key] for key in ["status","all_other_staged_files_pass_whitespace_check","original_Cargo_output_bytes_modified"]}))
