"""Record the final changed Go test files' format check."""
from pathlib import Path
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write


def main():
    out = ROOT / 'go-format-final-check'
    out.mkdir(exist_ok=False)
    executable = Path('/usr/local/go/bin/gofmt')
    files = [REPO / 'go-tools/go2vir/practical_compiler_capture_test.go', REPO / 'go-tools/go2vir/practical_compiler_inputs_test.go']
    inputs = [seal(p) for p in [Path(__file__).resolve(), executable, *files]]
    command = [str(executable), '-d', *map(str, files)]
    write(out / 'inputs.json', {'inputs': inputs, 'selection_reason': 'Only the two modified Go test sources need a final format check; the previous pinned Go1.25 formatter wrote and checked these files, and only a comment changed since.'})
    report = {'status': 'running', 'command': command, 'inputs': seal(out / 'inputs.json'), 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', report)
    result = subprocess.run(command, cwd=REPO, capture_output=True, check=False)
    (out / 'stdout.log').write_bytes(result.stdout)
    (out / 'stderr.log').write_bytes(result.stderr)
    unchanged = inputs == [seal(p) for p in [Path(__file__).resolve(), executable, *files]]
    passed = result.returncode == 0 and not result.stdout and not result.stderr and unchanged
    report.update(status='passed' if passed else 'failed', exit_code=result.returncode, inputs_unchanged=unchanged, stdout=seal(out/'stdout.log'), stderr=seal(out/'stderr.log'))
    write(out / 'result.json', report)
    print(json.dumps({k:report[k] for k in ['status','exit_code','inputs_unchanged']}))
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
