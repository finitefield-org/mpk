"""Format only the new native oracle with the complete registered Go SDK."""
from pathlib import Path
import json
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
SDK = Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
IMAGE = 'docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, sdk_files, write


def main():
    out = ROOT / 'go-format-write'
    out.mkdir(exist_ok=False)
    source = REPO / 'go-tools/go2vir/practical_analysis_info_differential_test.go'
    before = seal(source)
    neighbors = [seal(p) for p in sorted(source.parent.rglob('*')) if p.is_file() and p != source]
    sdk = sdk_files()
    command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=bind,src={source.parent},dst=/source', '--mount', f'type=bind,src={SDK},dst=/sdk,readonly', IMAGE, '/sdk/go/bin/gofmt', '-w', '/source/practical_analysis_info_differential_test.go']
    write(out / 'inputs.json', {'source_before': before, 'registered_sdk': sdk, 'other_complete_package_files': neighbors, 'driver': seal(Path(__file__).resolve()), 'command': command, 'selection_reason': 'Format only the added native Go oracle using the original registered Go SDK; retain the pre-format bytes and all other original package files.'})
    (out / 'source-before.go').write_bytes(source.read_bytes())
    child = subprocess.run(command, capture_output=True, check=False)
    (out / 'stdout.log').write_bytes(child.stdout)
    (out / 'stderr.log').write_bytes(child.stderr)
    after = seal(source)
    unchanged = sdk == sdk_files()
    neighbors_unchanged = neighbors == [seal(p) for p in sorted(source.parent.rglob('*')) if p.is_file() and p != source]
    passed = child.returncode == 0 and unchanged and neighbors_unchanged and not child.stdout and not child.stderr
    result = {'status': 'passed' if passed else 'failed', 'exit_code': child.returncode, 'source_before': before, 'source_after': after, 'expected_formatter_mutation': before != after, 'registered_sdk_unchanged': unchanged, 'other_complete_package_files_unchanged': neighbors_unchanged, 'stdout': seal(out / 'stdout.log'), 'stderr': seal(out / 'stderr.log'), 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'exit_code': result['exit_code'], 'expected_formatter_mutation': result['expected_formatter_mutation'], 'registered_sdk_unchanged': unchanged}))
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
