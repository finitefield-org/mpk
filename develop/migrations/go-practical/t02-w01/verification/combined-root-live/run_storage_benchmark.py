"""Record a complete-SDK storage comparison after the failed live attempt."""
from pathlib import Path
import json
import os
import subprocess
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
SDK = Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain')
IMAGE = 'sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
DOCKER = '/usr/local/bin/docker'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, sdk_files, write


def main():
    epoch = sys.argv[1]
    assert epoch.replace('-', '').isalnum() and Path.cwd() == REPO
    old = json.loads((ROOT / 'live-original/result.json').read_bytes())
    stored = json.loads((ROOT / 'live-original/failure-storage.json').read_bytes())
    assert old['status'] == 'failed' and stored['status'] == 'passed'
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    work = Path('/private/tmp') / ('mpk-sdk-storage-benchmark-' + epoch)
    assert not work.exists()
    work.mkdir(mode=0o700)
    host = work / 'host'
    host.mkdir(mode=0o700)
    volume = 'mpk-combined-sdk-storage-' + uuid.uuid4().hex
    before = sdk_files()
    code = ROOT / 'benchmark_sdk_storage.py'
    inputs = [seal(Path(__file__).resolve()), seal(code), seal(REPO / 'release/bundles/candidates/go.json')]
    create = [DOCKER, 'volume', 'create', volume]
    created = subprocess.run(create, cwd=REPO, capture_output=True, text=True, check=False)
    assert created.returncode == 0 and created.stdout.strip() == volume and not created.stderr
    init = [DOCKER, 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--user', '0:0', '--cap-drop', 'ALL', '--cap-add', 'CHOWN', '--mount', f'type=volume,src={volume},dst=/volume', IMAGE, '/bin/chown', f'{os.getuid()}:{os.getgid()}', '/volume']
    initialized = subprocess.run(init, cwd=REPO, capture_output=True, text=True, check=False)
    assert initialized.returncode == 0 and not initialized.stdout and not initialized.stderr
    command = [DOCKER, 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,nosuid,nodev,size=512m,mode=1777', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={SDK},dst=/sdk,readonly', '--mount', f'type=bind,src={host},dst=/host', '--mount', f'type=volume,src={volume},dst=/volume', '--mount', f'type=bind,src={out},dst=/out', '--workdir', '/workspace', IMAGE, 'python3', '-B', '/workspace/develop/migrations/go-practical/t02-w01/verification/combined-root-live/benchmark_sdk_storage.py']
    write(out / 'inputs.json', {'selection_reason': 'Investigate the completed Go package-export timeout by measuring original SDK copy/read overhead on the shared host directory and Linux volume, preserving all original bytes and limits.', 'inputs': inputs, 'registered_sdk': before, 'previous_failed_result': seal(ROOT / 'live-original/result.json'), 'previous_failure_storage': seal(ROOT / 'live-original/failure-storage.json'), 'volume': volume, 'volume_create': create, 'volume_init': init, 'command': command, 'host_temporary_directory': str(work)})
    report = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', report)
    started = time.monotonic()
    with (out / 'worker.stdout').open('xb') as stdout, (out / 'worker.stderr').open('xb') as stderr:
        proc = subprocess.Popen(command, cwd=REPO, stdout=stdout, stderr=stderr)
        last = started
        while proc.poll() is None:
            time.sleep(1)
            if time.monotonic() - last >= 30:
                print(json.dumps({'phase': 'copying_and_verifying_complete_sdk', 'seconds': round(time.monotonic() - started)}), flush=True)
                last = time.monotonic()
        code_result = proc.wait()
    worker = json.loads((out / 'worker-result.json').read_bytes()) if (out / 'worker-result.json').exists() else {}
    unchanged = before == sdk_files() and inputs == [seal(Path(row['path'])) for row in inputs]
    okay = code_result == 0 and unchanged and worker.get('status') == 'passed' and len(worker.get('results', [])) == 2 and all(row['original_files'] == 5577 and row['original_bytes'] == 166354960 and row['original_file_contents_and_modes_match'] for row in worker['results'])
    report.update(status='passed' if okay else 'failed', exit_code=code_result, seconds=time.monotonic() - started, original_inputs_unchanged=unchanged, worker=worker, volume=volume, stdout=seal(out / 'worker.stdout'), stderr=seal(out / 'worker.stderr'))
    write(out / 'result.json', report)
    print(json.dumps({'status': report['status'], 'original_inputs_unchanged': unchanged, 'volume': volume, 'results': worker.get('results', [])}), flush=True)
    raise SystemExit(not okay)


if __name__ == '__main__':
    main()
