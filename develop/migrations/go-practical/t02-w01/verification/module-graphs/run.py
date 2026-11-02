"""Preserve complete consumed original inputs before each affected local worker."""
from pathlib import Path
import datetime, hashlib, json, os, subprocess, sys, tarfile, time

ROOT = Path(__file__).resolve().parent
REPO = Path.cwd()
SDK = Path('/private/tmp/mpk-go125-20261025-download/go')
CHANGED = [
    'go-tools/go2vir/practical_observation.go',
    'go-tools/go2vir/practical_resolution.go',
    'go-tools/go2vir/practical_module_capture.go',
    'go-tools/go2vir/practical_module_graph.go',
    'go-tools/go2vir/practical_module_graph_inputs.go',
    'go-tools/go2vir/practical_image_inputs.go',
    'go-tools/go2vir/practical_original_packages.go',
    'go-tools/go2vir/practical_module_observation_inputs.go',
    'go-tools/go2vir/practical_observation_test.go',
    'go-tools/go2vir/practical_storage_test.go',
    'go-tools/go2vir/practical_module_graph_test.go',
]

def seal(path):
    path = Path(path)
    raw = path.read_bytes()
    return {'path': str(path.resolve()), 'bytes': len(raw),
            'sha256': hashlib.sha256(raw).hexdigest(), 'mode': path.stat().st_mode & 0o777}

def write(path, value):
    path.write_text(json.dumps(value, indent=2) + '\n')

def preserve_sdk(sdk_root=SDK, name='original-sdk'):
    out = ROOT / name
    out.mkdir(exist_ok=False)
    sources = [seal(p) for p in sorted(sdk_root.rglob('*')) if p.is_file()]
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for row in sources:
            archive.add(row['path'], arcname=str(Path(row['path']).relative_to(sdk_root)), recursive=False)
    assert sources == [seal(row['path']) for row in sources]
    write(out / 'inputs.json', {'root': str(sdk_root), 'sources': sources,
          'complete_archive': seal(out / 'inputs.tar.gz')})
    print(json.dumps({'sdk_files': len(sources), 'original_bytes': sum(row['bytes'] for row in sources)}))

def main():
    if sys.argv[1:] == ['preserve-sdk']:
        preserve_sdk()
        return
    if sys.argv[1:] == ['preserve-legacy-sdk']:
        preserve_sdk(Path('/usr/local/go'), 'original-legacy-sdk')
        return
    epoch, mode = sys.argv[1:]
    assert epoch.replace('-', '').isalnum()
    out = ROOT / epoch
    out.mkdir(exist_ok=False)
    formatting = mode in ['format-write', 'format']
    files = {Path(__file__).resolve()}
    files.update(ROOT.glob('*.py'))
    if formatting:
        files.update(REPO / name for name in CHANGED)
    else:
        # The Go compiler consumes every original package/test/vendor source;
        # retain the full existing fixture and proof universes as well.
        for name in ['go-tools/go2vir', 'go-tools/mpk-checker-ref', 'crates',
                     'proofs/std', 'fixtures', 'release/bundles', 'develop/specs/vectors', '.cargo']:
            files.update(p for p in (REPO / name).rglob('*') if p.is_file())
        files.update(REPO / name for name in ['Cargo.toml', 'Cargo.lock'])
    sdk_receipt = ROOT / ('original-legacy-sdk/inputs.json' if mode == 'legacy' else 'original-sdk/inputs.json')
    sdk = json.loads(sdk_receipt.read_bytes())
    sdk_before = [seal(row['path']) for row in sdk['sources']]
    assert sdk_before == sdk['sources']
    sources = [seal(p) for p in sorted(files)]
    with tarfile.open(out / 'inputs.tar.gz', 'x:gz', dereference=True) as archive:
        for p in sorted(files):
            archive.add(p, arcname=str(p.relative_to(REPO)), recursive=False)
    receipt = out / 'inputs.json'
    write(receipt, {'basis_commit': 'bf96736804ec439c16d124dca14e61e45855d455',
          'sources': sources, 'complete_archive': seal(out / 'inputs.tar.gz'),
          'whole_original_sdk': seal(sdk_receipt), 'sdk_archive': sdk['complete_archive'],
          'scope': 'Complete original source/fixture/proof universe; exact full pinned SDK archive reused without dropping inputs.'})
    commands = {
        'format-write': [str(SDK / 'bin/gofmt'), '-w', *CHANGED],
        'format': [str(SDK / 'bin/gofmt'), '-l', *CHANGED],
        'native': [str(SDK / 'bin/go'), 'test', '-json', '-count=1', '-timeout=40m', '-run', '^TestPracticalModuleGraph'],
        'edges': [str(SDK / 'bin/go'), 'test', '-json', '-count=1', '-timeout=40m', '-run', '^TestPracticalModuleGraph(PreservesUnselectedOldLeavesCyclesAndPrunedInputs|RejectsChangedWholeEdges)$'],
        'replacements': [str(SDK / 'bin/go'), 'test', '-json', '-count=1', '-timeout=40m', '-run', '^TestPracticalModuleGraphRetainsVersionedReplacementAndInitialRootUpgrade$'],
        'toolchains': [str(SDK / 'bin/go'), 'test', '-json', '-count=1', '-timeout=40m', '-run', '^TestPracticalModuleGraphKeepsOriginalVirtualToolchainInputs$'],
        'affected': [str(SDK / 'bin/go'), 'test', '-json', '-count=1', '-timeout=40m', '-run', '^TestPractical(Resolution|Observation|ImageInputsRebindWholeOriginalModuleGraphAfterRetirement|ModuleObservationsKeepOriginal)'],
        'vet': [str(SDK / 'bin/go'), 'vet', '.'],
        'legacy': ['/usr/local/go/bin/go', 'test', '-run', '^$', '.'],
    }
    changes = {'GOROOT': str(SDK) if mode != 'legacy' else '/usr/local/go',
               'GOTOOLCHAIN': 'local', 'GOENV': 'off', 'GOWORK': 'off', 'GOFLAGS': '-mod=vendor',
               'GOPROXY': 'off', 'GOSUMDB': 'off', 'CGO_ENABLED': '0', 'GOMAXPROCS': '1',
               'GOCACHE': '/private/tmp/mpk-go-export-inputs-190-cache',
               'GOMODCACHE': '/private/tmp/mpk-go-module-observations-191-mod'}
    if mode in ['native', 'affected', 'edges', 'replacements', 'toolchains']:
        native = out / 'native-originals'
        native.mkdir()
        changes['MPK_GO_PRACTICAL_SYNTAX_EVIDENCE'] = str(native)
    env = os.environ.copy()
    env.update(changes)
    for key in ['RUST_MIN_STACK', 'MPK_CORE_MAX_STEPS', 'MPK_CORE_MAX_SECONDS']:
        env.pop(key, None)
    command = commands[mode]
    result = {'status': 'running', 'command': command, 'environment': changes,
              'inputs': seal(receipt), 'driver': seal(__file__), 'go_tool': seal(command[0]),
              'started_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'selection_reason': 'Changed controlled Go observation, original replacement location, immutable module rebinding and owner graph handoff. Compare actual pruned/unpruned original graphs, unselected/empty old manifests, cycles, duplicates, exclusions, root upgrades, local/versioned replacements, original input retirement, whole native stdout/stderr and detached handoffs. Run new graph roots plus affected original resolution/observation/image/module-owner consumers, vet and explicit changed-file format. Rust consumers are unchanged and retain their original evidence; no removed-language tests or whole gate run.',
              'full_gate': 'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    write(out / 'result.json', result)
    start = time.monotonic()
    with (out / 'stdout.log').open('xb') as stdout, (out / 'stderr.log').open('xb') as stderr:
        code = subprocess.run(command, cwd=REPO if formatting else REPO / 'go-tools/go2vir',
                              env=env, stdout=stdout, stderr=stderr, check=False).returncode
    unchanged = sources == [seal(p) for p in sorted(files)]
    sdk_unchanged = sdk_before == [seal(row['path']) for row in sdk['sources']]
    formatted = not formatting or not (out / 'stdout.log').read_bytes()
    result.update(status='passed' if code == 0 and (unchanged or mode == 'format-write') and sdk_unchanged and formatted else 'failed',
                  exit_code=code, seconds=time.monotonic() - start,
                  consumed_inputs_unchanged=unchanged, whole_sdk_unchanged=sdk_unchanged,
                  expected_formatter_mutation=mode == 'format-write', stdout=seal(out / 'stdout.log'),
                  stderr=seal(out / 'stderr.log'), finished_at_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
    write(out / 'result.json', result)
    print(json.dumps({key: result[key] for key in ['status', 'exit_code', 'seconds', 'consumed_inputs_unchanged', 'whole_sdk_unchanged']}))
    raise SystemExit(result['status'] != 'passed')

if __name__ == '__main__':
    main()
