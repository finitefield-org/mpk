"""Check complete original Go owners through the private test-only sealer."""
from pathlib import Path
import datetime
import json
import os
import subprocess
import sys
import tarfile
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT))
from run_scoped import seal, write
sys.path.insert(0, str(REPO / 'develop/probes/go-practical'))
from frontend_native_build import linked_inputs


def main():
    out = ROOT / 'whole-snapshot-default'
    out.mkdir(exist_ok=False)
    original_roots = [ROOT / 'linux-owner-serial/native-originals/versioned-owned', ROOT.parent / 'compiler-inputs/native/native-originals/versioned-owned']
    originals = {p for root in original_roots for p in root.rglob('*') if p.is_file()}
    sources = {Path(__file__).resolve(), ROOT / 'run_scoped.py', REPO / 'crates/mpk-cli/tests/go_practical_installed_compiler.rs', REPO / 'develop/probes/go-practical/frontend_native_build.py'}
    sources.update(REPO / row['path'] for row in linked_inputs())
    before_sources = [seal(p) for p in sorted(sources)]
    before_originals = [seal(p) for p in sorted(originals)]
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for path in sorted(sources):
            archive.add(path, arcname=str(path.relative_to(REPO)), recursive=False)
    write(out / 'inputs.json', {'sources': before_sources, 'source_archive': seal(out / 'inputs.tar.gz'), 'complete_originals': before_originals, 'selection_reason': 'Exercise the real shared binding logic against all5577 registered Linux SDK file bytes and modes plus the complete182-package/5697-object original source/export owner. Include a coherently rehashed hostile host and a whole original Darwin owner cross-check. This test-only sealed snapshot establishes no installed-root authority and does not activate the practical profile. Original files are retained in place and individually hashed before and after, rather than copied or narrowed.'})
    command = ['cargo', 'test', '--offline', '--locked', '-p', 'mpk-cli', '--test', 'go_practical_installed_compiler']
    env = {'CARGO_NET_OFFLINE': 'true', 'CARGO_INCREMENTAL': '0', 'CARGO_PROFILE_TEST_OPT_LEVEL': '1', 'CARGO_PROFILE_TEST_DEBUG': '0', 'CARGO_PROFILE_TEST_DEBUG_ASSERTIONS': 'true', 'CARGO_PROFILE_TEST_OVERFLOW_CHECKS': 'true', 'RUST_TEST_THREADS': '1'}
    report = {'status': 'running', 'command': command, 'environment': env, 'inputs': seal(out / 'inputs.json'), 'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'installed_root_exercised': False, 'native_Linux_release_acceptance': False, 'full_T_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', report)
    start = time.monotonic()
    with (out / 'stdout.log').open('xb') as stdout, (out / 'stderr.log').open('xb') as stderr:
        code = subprocess.run(command, cwd=REPO, env={**os.environ, **env}, stdout=stdout, stderr=stderr, check=False).returncode
    unchanged_sources = before_sources == [seal(p) for p in sorted(sources)]
    unchanged_originals = before_originals == [seal(p) for p in sorted(originals)]
    output = None
    if code == 0:
        output = json.loads((out / 'stdout.log').read_bytes())
    passed = code == 0 and unchanged_sources and unchanged_originals and output is not None and output['status'] == 'passed' and not output['installed_root_exercised']
    report.update(status='passed' if passed else 'failed', exit_code=code, seconds=time.monotonic() - start, consumed_sources_unchanged=unchanged_sources, whole_originals_unchanged=unchanged_originals, observation=output, stdout=seal(out / 'stdout.log'), stderr=seal(out / 'stderr.log'))
    write(out / 'result.json', report)
    print(json.dumps({k: report[k] for k in ['status', 'exit_code', 'seconds', 'consumed_sources_unchanged', 'whole_originals_unchanged', 'observation']}))
    raise SystemExit(not passed)


if __name__ == '__main__':
    main()
