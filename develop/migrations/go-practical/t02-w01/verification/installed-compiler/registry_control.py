"""Validate a coherent alternate registry independently of installed authority."""
from pathlib import Path
import hashlib
import json
import os
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
GO = Path('/private/tmp/mpk-installed-compiler-205-original-go-output/toolchain/go/bin/go')
IMAGE = 'sha256:d617f1b5c2a18778ba357ba0528e973bd24992b0dd18000dfbca059f4a15ebda'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'scripts'))
import successor_release_bundles as release


def main():
    out = ROOT / 'alternate-registry-validation'
    out.mkdir(exist_ok=False)
    build = json.loads((ROOT / 'linux-test-bound-final/result.json').read_bytes())
    assert build['status'] == 'passed' and build['consumed_inputs_unchanged']
    executable = Path(build['executable']['path'])
    assert seal(executable) == build['executable']
    registry, candidates = release.validate_release_models()
    inputs = [Path(__file__).resolve(), ROOT / 'run_scoped.py', REPO / 'scripts/successor_release_bundles.py', REPO / 'scripts/release_bundles.py', REPO / 'scripts/rust_build_inputs.py', release.REGISTRY_PATH, release.SEMANTIC_REGISTRY_PATH, *release.CANDIDATE_PATHS.values(), executable, GO]
    before = [seal(p) for p in inputs]
    write(out / 'inputs.json', {'inputs': before, 'scope': 'Starting from the full fixed Go/Rust registry and actual registered Go executable, coherently change one executable byte, its inventory and component digest, the full Go distribution digest and the complete registry digest. Check that the resulting full registry validates independently, without invoking installed authority.'})
    original = GO.read_bytes()
    changed = original[:-1] + bytes([original[-1] ^ 1])
    binary_path = out / 'changed-go-binary'
    binary_path.write_bytes(changed)
    alternate = json.loads(release.REGISTRY_PATH.read_bytes())
    go_id = candidates['go']['toolchain_bundles'][0]['bundle_id']
    toolchain = next(t for t in alternate['toolchain_bundles'] if t['bundle_id'] == go_id)
    digest = hashlib.sha256(changed).hexdigest()
    next(f for f in toolchain['inventory']['files'] if f['path'] == 'go/bin/go')['sha256'] = digest
    next(c for c in toolchain['components'] if c['name'] == 'go')['binary_sha256'] = digest
    toolchain['distribution_sha256'] = hashlib.sha256(release.CONTENT_DOMAIN + release.canonical(toolchain['inventory'])).hexdigest()
    alternate.pop('registry_sha256')
    alternate['registry_sha256'] = hashlib.sha256(release.REGISTRY_DOMAIN + release.canonical(alternate)).hexdigest()
    path = out / 'alternate-registry.json'
    path.write_bytes(release.canonical_line(alternate))
    command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/amd64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=bind,src={executable},dst=/observer,readonly', '--mount', f'type=bind,src={out},dst=/controls,readonly', IMAGE, '/observer', '--validate-registry', '/controls/alternate-registry.json']
    report = {'status': 'running', 'command': command, 'inputs': seal(out / 'inputs.json'), 'alternate_registry': seal(path), 'changed_binary': seal(binary_path), 'native_Linux_release_acceptance': False}
    write(out / 'result.json', report)
    start = time.monotonic()
    result = subprocess.run(command, cwd=REPO, capture_output=True, check=False)
    (out / 'stdout.log').write_bytes(result.stdout)
    (out / 'stderr.log').write_bytes(result.stderr)
    observed = json.loads(result.stdout) if result.returncode == 0 else None
    passed = result.returncode == 0 and not result.stderr and observed == {'status': 'accepted', 'phase': 'registry-validation', 'registry_sha256': alternate['registry_sha256']} and before == [seal(p) for p in inputs]
    report.update(status='passed' if passed else 'failed', exit_code=result.returncode, seconds=time.monotonic()-start, inputs_unchanged=before == [seal(p) for p in inputs], observation=observed, stdout=seal(out/'stdout.log'), stderr=seal(out/'stderr.log'))
    write(out / 'result.json', report)
    print(json.dumps({k:report[k] for k in ['status', 'exit_code', 'seconds', 'inputs_unchanged', 'observation']}))
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
