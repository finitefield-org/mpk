"""Run Linux-target Clippy locally using the complete retained input epoch."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
TARGET = Path('/private/tmp/mpk-go-source-root-clippy-target')


def seal(path):
    data = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest(), 'mode': path.stat().st_mode & 0o777}


def main():
    out = ROOT / 'clippy-linux-target'; out.mkdir(exist_ok=False)
    prior = json.loads((ROOT / 'final/inputs.json').read_bytes())
    original = prior['inputs']
    assert original == [seal(Path(row['path'])) for row in original]
    assert seal(Path(prior['archive']['path'])) == prior['archive']
    command = ['cargo', 'clippy', '--offline', '--locked', '-p', 'mpk-cli', '--lib', '--test', 'go_practical_source_roots', '--target', 'x86_64-unknown-linux-gnu', '--', '-D', 'warnings']
    env = {'CARGO_TARGET_DIR': str(TARGET), 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0'}
    versions = {}
    for name, args in [('cargo', ['cargo', '--version']), ('rustc', ['rustc', '--version']), ('clippy', ['cargo', 'clippy', '--version'])]:
        result = subprocess.run(args, cwd=REPO, capture_output=True, check=True)
        versions[name] = result.stdout.decode().strip()
    (out / 'inputs.json').write_text(json.dumps({'selection_reason': 'Run strict Clippy on the changed CLI library and Linux root-matching test with the installed x86_64 Linux target. The prior epoch retains every original frontend and linked source input. Linux execution evidence is recorded separately; no whole T gate runs.', 'driver': seal(Path(__file__)), 'prior_inputs': seal(ROOT/'final/inputs.json'), 'prior_archive': prior['archive'], 'original_files': len(original), 'command': command, 'environment': env, 'versions': versions}, indent=2) + '\n')
    report = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (out / 'result.json').write_text(json.dumps(report, indent=2)+'\n')
    start = time.monotonic()
    with (out/'stdout.log').open('xb') as stdout, (out/'stderr.log').open('xb') as stderr:
        code = subprocess.run(command, cwd=REPO, env={**os.environ, **env}, stdout=stdout, stderr=stderr, check=False).returncode
    unchanged = original == [seal(Path(row['path'])) for row in original]
    report.update(status='passed' if code == 0 and unchanged else 'failed', exit_code=code, seconds=time.monotonic()-start, original_inputs_unchanged=unchanged, stdout=seal(out/'stdout.log'), stderr=seal(out/'stderr.log'))
    (out/'result.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k: report[k] for k in ['status', 'exit_code', 'seconds', 'original_inputs_unchanged']}))
    raise SystemExit(report['status'] != 'passed')


if __name__ == '__main__':
    main()
