"""Observe whether an alternate local emulator preserves the Linux boundary."""
from pathlib import Path
import json
import os
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
VOLUME = Path('/private/tmp/mpk-go-installed-205-fs')
IMAGE = 'rust:1.95-bookworm-arm64-mpk'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write


def main():
    label, executable = sys.argv[1:]
    emulator = Path(executable)
    probe = VOLUME / 'qemu-probe/probe'
    installed = VOLUME / 'installed-diagnostic'
    out = ROOT / f'qemu-{label}-openat2'
    out.mkdir(exist_ok=False)
    paths = [Path(__file__).resolve(), ROOT / 'run_scoped.py', ROOT / 'qemu_openat2.go', ROOT / 'qemu-probe-build/result.json', emulator, probe]
    inputs = [seal(path) for path in paths]
    base = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=bind,src={emulator},dst=/qemu,readonly', '--mount', f'type=bind,src={probe},dst=/probe,readonly', '--mount', f'type=bind,src={installed},dst=/installed,readonly', IMAGE, '/qemu']
    commands = [base + ['--version'], base + ['/probe']]
    write(out / 'inputs.json', {'inputs': inputs, 'commands': commands, 'scope': 'Observe the same openat2 flags and /proc/self/exe from a separate user-mode emulator against the complete copied installed root and /tmp. This is not native Linux release acceptance.'})
    results = []
    for name, command in zip(['version', 'probe'], commands):
        child = subprocess.run(command, capture_output=True, check=False)
        stdout, stderr = out / f'{name}.stdout', out / f'{name}.stderr'
        stdout.write_bytes(child.stdout)
        stderr.write_bytes(child.stderr)
        results.append({'name': name, 'exit_code': child.returncode, 'stdout': seal(stdout), 'stderr': seal(stderr)})
    report = {'status': 'observed', 'native_Linux_release_acceptance': False, 'inputs_unchanged': inputs == [seal(path) for path in paths], 'results': results}
    write(out / 'result.json', report)
    print(json.dumps({'status': report['status'], 'inputs_unchanged': report['inputs_unchanged'], 'exit_codes': [item['exit_code'] for item in results]}))


if __name__ == '__main__':
    main()
