"""Run the original complete-owner projection and coherent negative cases."""
from pathlib import Path
import json
import os
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
CARGO = Path('/private/tmp/mpk-go-source-root-matching/cargo')
TARGET = Path('/private/tmp/mpk-go-source-root-matching/target')
REGISTRY = Path('/Users/kazuyoshitoshiya/.cargo/registry')
IMAGE = 'sha256:7cf1e580ef5539f03b58560753e8ab84c8c360960d99dff714004aa98f203977'
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent / 'installed-compiler'))
from run_scoped import seal, write


def main():
    epoch = sys.argv[1]
    assert epoch.replace('-', '').isalnum() and Path.cwd() == REPO
    previous = json.loads((ROOT / 'projection-preflight-original/result.json').read_bytes())
    preflight = json.loads((ROOT / 'projection-preflight-original/inputs.json').read_bytes())
    live = ROOT / 'live-original'
    storage = json.loads((live / 'evidence-storage.json').read_bytes())
    assert previous['status'] == 'passed' and previous['original_inputs_unchanged'] and storage['status'] == 'passed'
    source = preflight['inputs']
    assert source == [seal(Path(r['path'])) for r in source]
    assert seal(Path(preflight['archive']['path'])) == preflight['archive']
    paths = [Path(__file__).resolve(), ROOT / 'restore_evidence.py', live / 'evidence-storage.json', live / 'original-evidence-manifest.json', live / 'result.json', live / 'coord/snapshot-ready.json']
    paths += [live / row['path'] for row in storage['chunks']]
    before = [seal(p) for p in paths]
    assert all(before[6 + i] == row['encoded'] for i, row in enumerate(storage['chunks']))
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    work = Path('/private/tmp') / ('mpk-go-live-export-projection-' + epoch)
    assert not work.exists()
    work.mkdir(mode=0o700)
    tmp = work / 'tmp'
    tmp.mkdir(mode=0o700)
    env = {'CARGO_HOME': '/cargo', 'CARGO_TARGET_DIR': '/target', 'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_DEV_DEBUG': '0', 'HOME': '/tmp', 'MPK_GO_LIVE_EXPORT_BINDING_OUTPUT': '/out'}
    command = ['/usr/local/bin/docker', 'run', '--pull=never', '--rm', '--platform', 'linux/arm64', '--network', 'none', '--read-only', '--cap-drop', 'ALL', '--security-opt', 'no-new-privileges', '--user', f'{os.getuid()}:{os.getgid()}', '--mount', f'type=bind,src={REPO},dst=/workspace,readonly', '--mount', f'type=bind,src={tmp},dst=/tmp', '--mount', f'type=bind,src={out},dst=/out', '--mount', f'type=bind,src={CARGO},dst=/cargo', '--mount', f'type=bind,src={REGISTRY},dst=/cargo/registry,readonly', '--mount', f'type=bind,src={TARGET},dst=/target', '--workdir', '/workspace']
    for key, value in env.items():
        command += ['--env', f'{key}={value}']
    command += [IMAGE, 'cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_live_export_projection', '--', '--nocapture']
    write(out / 'inputs.json', {'selection_reason': 'Restore all original evidence and bind the live export capability to the complete independently imported compiler owner; retain coherent export-byte and unread-namespace negative cases.', 'inputs': before, 'source_preflight_inputs': seal(ROOT / 'projection-preflight-original/inputs.json'), 'source_preflight_archive': preflight['archive'], 'command': command, 'environment': env})
    result = {'status': 'running', 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    started = time.monotonic()
    with (out / 'test.stdout').open('xb') as stdout, (out / 'test.stderr').open('xb') as stderr:
        proc = subprocess.Popen(command, cwd=REPO, stdout=stdout, stderr=stderr)
        last = started
        while proc.poll() is None:
            time.sleep(1)
            if time.monotonic() - last >= 30:
                print(json.dumps({'phase': 'restoring_and_matching_original_evidence', 'seconds': round(time.monotonic() - started)}), flush=True)
                last = time.monotonic()
        code = proc.wait()
    found = []
    for line in (out / 'test.stdout').read_text().splitlines():
        try:
            found.append(json.loads(line))
        except json.JSONDecodeError:
            pass
    observation = next((v for v in found if isinstance(v, dict) and v.get('complete_original_objects') == 5697), {})
    unchanged = before == [seal(p) for p in paths] and source == [seal(Path(r['path'])) for r in source]
    expected = observation.get('status') == 'passed' and observation.get('live_export_objects') == 181 and observation.get('compiler_files') == 5577 and observation.get('original_packages') == 182 and observation.get('coherent_changed_export_rejected') is True and observation.get('coherent_changed_namespace_rejected') is True
    artifacts = [out / 'changed-export.json', out / 'changed-namespace.json']
    passed = code == 0 and unchanged and expected and all(p.is_file() for p in artifacts)
    result.update(status='passed' if passed else 'failed', exit_code=code, seconds=time.monotonic() - started, original_inputs_unchanged=unchanged, observation=observation, stdout=seal(out / 'test.stdout'), stderr=seal(out / 'test.stderr'), negative_inputs=[seal(p) for p in artifacts if p.is_file()])
    write(out / 'result.json', result)
    print(json.dumps({'status': result['status'], 'exit_code': code, 'original_inputs_unchanged': unchanged}), flush=True)
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
