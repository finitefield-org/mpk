"""Observe the unmodified installed loader through the alternate emulator."""
from pathlib import Path
import json
import os
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
VOLUME = Path('/private/tmp/mpk-go-installed-205-fs')
INSTALLED = VOLUME / 'installed-diagnostic'
EMULATOR = VOLUME / 'qemu-backports/qemu-x86_64'
RUNTIME = VOLUME / 'qemu-amd64-runtime'
IMAGE = 'rust:1.95-bookworm-arm64-mpk'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write


def main():
    out = ROOT / 'installed-qemu-diagnostic'
    out.mkdir(exist_ok=False)
    paths = {Path(__file__).resolve(), ROOT / 'run_scoped.py', ROOT / 'qemu-backports-openat2/result.json', ROOT / 'qemu-amd64-runtime/result.json', EMULATOR}
    paths.update(p for root in [INSTALLED, RUNTIME] for p in root.rglob('*') if p.is_file())
    before = [seal(path) for path in sorted(paths)]
    command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--tmpfs', '/tmp:rw,nosuid,nodev,size=128m,mode=1777', '--mount', f'type=bind,src={INSTALLED},dst=/installed,readonly', '--mount', f'type=bind,src={EMULATOR},dst=/qemu,readonly', '--mount', f'type=bind,src={RUNTIME},dst=/x86root,readonly', IMAGE, '/qemu', '-L', '/x86root', '/installed/bin/mpk', '--diagnose-installed']
    write(out / 'inputs.json', {'inputs': before, 'command': command, 'scope': 'Observe the unchanged descriptor-relative installed loader against the complete four-bundle release root using an alternate local x86-64 user-mode emulator and libraries copied from the pinned image. This is not native Linux release acceptance.'})
    report = {'status': 'running', 'inputs': seal(out / 'inputs.json'), 'native_Linux_release_acceptance': False}
    write(out / 'result.json', report)
    start = time.monotonic()
    child = subprocess.run(command, capture_output=True, check=False)
    stdout, stderr = out / 'stdout.log', out / 'stderr.log'
    stdout.write_bytes(child.stdout)
    stderr.write_bytes(child.stderr)
    try:
        observation = json.loads(child.stdout)
    except (ValueError, UnicodeDecodeError):
        observation = None
    report.update(status='observed', exit_code=child.returncode, observation=observation, seconds=time.monotonic()-start, inputs_unchanged=before == [seal(path) for path in sorted(paths)], stdout=seal(stdout), stderr=seal(stderr))
    write(out / 'result.json', report)
    print(json.dumps({key:report[key] for key in ['status', 'exit_code', 'observation', 'seconds', 'inputs_unchanged']}))


if __name__ == '__main__':
    main()
