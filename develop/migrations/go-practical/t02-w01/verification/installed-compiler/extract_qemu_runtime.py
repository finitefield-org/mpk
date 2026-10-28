"""Copy the actual amd64 runtime files from the already pinned local image."""
from pathlib import Path
import json
import subprocess
import sys

ROOT = Path(__file__).resolve().parent
VOLUME = Path('/private/tmp/mpk-go-installed-205-fs')
DESTINATION = VOLUME / 'qemu-amd64-runtime'
IMAGE = 'sha256:d617f1b5c2a18778ba357ba0528e973bd24992b0dd18000dfbca059f4a15ebda'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write


def main():
    out = ROOT / 'qemu-amd64-runtime'
    out.mkdir(exist_ok=False)
    assert not DESTINATION.exists()
    DESTINATION.mkdir()
    binary = ROOT / 'linux-test-diagnostic/test-executable.elf'
    inputs = [seal(path) for path in [Path(__file__).resolve(), ROOT / 'run_scoped.py', ROOT / 'linux-test-diagnostic/result.json', binary]]
    calls = []

    def invoke(name, args):
        command = ['/usr/local/bin/docker', *args]
        child = subprocess.run(command, capture_output=True, check=False)
        stdout, stderr = out / f'{name}.stdout', out / f'{name}.stderr'
        stdout.write_bytes(child.stdout)
        stderr.write_bytes(child.stderr)
        calls.append({'name': name, 'command': command, 'exit_code': child.returncode, 'stdout': seal(stdout), 'stderr': seal(stderr)})
        write(out / 'commands.json', calls)
        if child.returncode:
            raise RuntimeError(f'{name} failed with exit code {child.returncode}')
        return child.stdout.decode().strip()

    write(out / 'inputs.json', {'inputs': inputs, 'image': IMAGE, 'scope': 'Extract the dynamic loader and libraries needed by the exact existing x86-64 test executable from the same pinned local Linux image. No installed bundle is changed. Local user-mode emulation does not constitute native Linux release acceptance.'})
    report = {'status': 'running', 'native_Linux_release_acceptance': False, 'inputs': seal(out / 'inputs.json')}
    write(out / 'result.json', report)
    container = None
    try:
        ldd = invoke('ldd', ['run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--mount', f'type=bind,src={binary},dst=/probe,readonly', IMAGE, '/usr/bin/ldd', '/probe'])
        assert all(name in ldd for name in ['libgcc_s.so.1', 'libc.so.6', 'ld-linux-x86-64.so.2'])
        container = invoke('create', ['create', '--platform', 'linux/amd64', '--network', 'none', IMAGE, '/bin/true'])
        assert len(container) == 64
        paths = ['lib64/ld-linux-x86-64.so.2', 'lib/x86_64-linux-gnu/libc.so.6', 'lib/x86_64-linux-gnu/libgcc_s.so.1']
        for index, path in enumerate(paths):
            destination = DESTINATION / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            invoke(f'copy-{index}', ['cp', '-L', f'{container}:/{path}', str(destination)])
        report.update(status='passed', runtime_files=[seal(DESTINATION / path) for path in paths])
    except BaseException as error:
        report.update(status='failed', error=repr(error))
        raise
    finally:
        if container:
            invoke('remove', ['rm', container])
        report.update(inputs_unchanged=inputs == [seal(path) for path in [Path(__file__).resolve(), ROOT / 'run_scoped.py', ROOT / 'linux-test-diagnostic/result.json', binary]], commands=calls)
        write(out / 'result.json', report)
    print(json.dumps({'status': report['status'], 'inputs_unchanged': report['inputs_unchanged'], 'runtime_files': len(report['runtime_files'])}))


if __name__ == '__main__':
    main()
