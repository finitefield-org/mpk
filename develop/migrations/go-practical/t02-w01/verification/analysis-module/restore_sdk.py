"""Restore and verify the registered SDK from the existing pinned local image."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
OUT = Path('/private/tmp/mpk-installed-compiler-205-original-go-output')
IMAGE = 'docker.io/library/golang@sha256:ebd54034f076819b3054f155db53660ded951612bb4dfd277f933e62059e5d5a'


def main():
    out = ROOT / 'sdk-restore'
    out.mkdir(exist_ok=False)
    assert not OUT.exists()
    OUT.mkdir(mode=0o700)
    toolchain = OUT / 'toolchain'
    toolchain.mkdir(mode=0o700)
    command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=bind,src={toolchain},dst=/out', IMAGE, '/bin/sh', '-ceu', 'mkdir -p /out/go; cp -a /usr/local/go/. /out/go/']
    report = {'status': 'running', 'command': command, 'image': IMAGE, 'native_Linux_release_acceptance': False}
    (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    start = time.monotonic()
    with (out / 'stdout.log').open('xb') as stdout, (out / 'stderr.log').open('xb') as stderr:
        code = subprocess.run(command, stdout=stdout, stderr=stderr, check=False).returncode
    if code:
        report.update(status='failed', exit_code=code, seconds=time.monotonic()-start)
        (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
        raise SystemExit(code)
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(REPO / 'scripts'))
    import release_bundles
    release_bundles.prune_go_distribution(toolchain / 'go')
    release_bundles.prune_empty_directories(toolchain)
    release_bundles.normalize_tree(toolchain, frontend=False)
    candidate = json.loads((REPO / 'release/bundles/candidates/go.json').read_bytes())
    expected = candidate['toolchain_bundles'][0]['inventory']['files']
    actual = release_bundles.inventory(toolchain)
    assert actual == expected and len(actual) == 5577
    source = ROOT.parent / 'analysis-build-info/registered-go2vir.elf'
    data = source.read_bytes()
    expected_frontend = candidate['frontend_bundles'][0]['inventory']['files'][0]
    assert len(data) == expected_frontend['size_bytes'] and hashlib.sha256(data).hexdigest() == expected_frontend['sha256']
    frontend = OUT / 'frontend/bin/go2vir'
    frontend.parent.mkdir(parents=True)
    frontend.write_bytes(data)
    frontend.chmod(0o555)
    report.update(status='passed', exit_code=code, seconds=time.monotonic()-start, registered_sdk_files=len(actual), registered_sdk_bytes=sum(row['size_bytes'] for row in actual), registered_frontend_sha256=expected_frontend['sha256'], exact_registered_inventories=True)
    (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps({key: report[key] for key in ['status', 'registered_sdk_files', 'registered_sdk_bytes', 'exact_registered_inventories']}))


if __name__ == '__main__':
    main()
