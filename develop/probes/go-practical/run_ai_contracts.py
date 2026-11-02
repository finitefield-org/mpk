#!/usr/bin/env python3
"""Record/check AI v4 design and independently observed retained consumers."""
import argparse
from collections import Counter
import os
from pathlib import Path
import subprocess
import tempfile
from proof_canonical import canonical
from ai_vectors import documents
from ai_native_checks import inputs as native_inputs, TESTS
from run_api_contracts import inputs as prior_inputs, RECORD as PREVIOUS
from policy_native_build import build, TARGET
from context_protocol import active_registry
from run_proof_pilot import ROOT, HERE, DEFAULT_GOROOT, sha, json_bytes, verify_host_toolchain
from run_structural_limits import timed_input
from check_scope_report import strict_json, require

RECORD = PREVIOUS.parent/'ai-explanation'
NEW = ('ai_contracts.py', 'ai_vectors.py', 'ai_native_checks.py', 'run_ai_contracts.py',
       'test_ai_contracts.py', 'check_ai_metadata.py')
DOCS = ('schema.json', 'bundles.json', 'vectors.json', 'hash-vectors.json')
SCOPE = 'inactive AI v4 sanitized request and local explanation roots,complete deterministic32-property batching,source-owned status/alias restoration and inert provider fixtures; unchanged native GoFixed/Rust/CSharp AI consumers and independent raw canonical hashes'
PENDING = ['release roots,ordinary Go program/source-binding signatures and all reviewed migration dispositions remain W04 work; materialized Go policy integration closes with that program/migration batch',
    'compiled AIv4/real-provider integration and all four retained-profile activation/release checks remain T07 work; no external provider or native Java AI execution here',
    'W05 inactive publication/cumulative T01 gate and full T02–T11 source/compiler/ordinary/application proofs and native local Linux release gates remain required']


def native_retained():
    p = RECORD/'verification'; r = strict_json((p/'native-ai-checks.json').read_bytes())
    require(r['inputs'] == native_inputs() and r['network_used'] is False and r['public_source_changed'] is False, 'exact unchanged native AI test/fixture authority')
    require(len(r['runs']) == 2, 'two actual native AI test runs')
    for i, run in enumerate(r['runs'], 1):
        require(run['exit_code'] == 0 and run['passed'] == 2 and run['tests'] == list(TESTS), 'actual Linux test bodies,not cfg-skip')
        for stream in ('stdout', 'stderr'): require(run[stream+'_sha256'] == sha((p/f'native-ai-{i}-{stream}.log').read_bytes()), 'complete native AI logs')
    return r


def source_inputs():
    rows = {r['path']: r['sha256'] for r in prior_inputs()+native_inputs()}
    for p in [HERE/n for n in NEW]+[PREVIOUS/'observations.json', RECORD/'verification/native-ai-checks.json']:
        rows[p.relative_to(ROOT).as_posix()] = sha(p.read_bytes())
    return [{'path': p, 'sha256': h} for p, h in sorted(rows.items())]


def inputs():
    return source_inputs()+[{'path': (RECORD/n).relative_to(ROOT).as_posix(), 'sha256': sha((RECORD/n).read_bytes())} for n in DOCS]


def expected(hs): return [{k: h[k] for k in ('id', 'accepted', 'canonical_hex', 'canonical_sha256')} for h in hs]


def check_retained():
    r = strict_json((RECORD/'observations.json').read_bytes()); native_retained(); ds = documents()
    require(r['scope'] == SCOPE and r['pending'] == PENDING and r['inputs'] == inputs(), 'AI design scope/source closure')
    for n, value in ds.items(): require((RECORD/n).read_bytes() == canonical(value)+b'\n', 'complete exact AI document/vector reconstruction: '+n)
    require(r['cases'] == len(ds['vectors.json']) and r['outcomes'] == dict(Counter(v['classification'] for v in ds['vectors.json'])), 'all AI conformance outcomes')
    hs = ds['hash-vectors.json']; require(strict_json((RECORD/'rust-hashes.json').read_bytes()) == expected(hs), 'actual Rust AI raw canonical identities')
    require(strict_json((RECORD/'go-hashes.json').read_bytes()) == [{k: h[k] for k in ('id', 'canonical_sha256')} for h in hs], 'independent Go AI canonical/raw identities')
    for name in ('rust-hashes', 'go-hashes'):
        require(len(r['runs'][name]) == 2 and all(x['exit_code'] == 0 and x['stdout_sha256'] == sha((RECORD/(name+'.json')).read_bytes()) for x in r['runs'][name]), 'two deterministic complete hash observations')
    for p in (RECORD/'verification').glob('initial-*/diagnostic.json'):
        for row in strict_json(p.read_bytes())['inputs']: require(sha((p.parent/row['path']).read_bytes()) == row['sha256'], 'initial AI model finding preserved')
    for flag in ('compiled_AI_v4_executed', 'provider_contacted', 'Go_ordinary_elaborated', 'profile_activated', 'W04_frozen'): require(r[flag] is False, 'AI helper/design/proof authority boundary')
    return r


def main():
    p = argparse.ArgumentParser(description=__doc__); g = p.add_mutually_exclusive_group(required=True)
    g.add_argument('--write', action='store_true'); g.add_argument('--check', action='store_true'); args = p.parse_args()
    before = source_inputs(); native = native_retained(); ds = documents(); cmd, audit = build(); hs = ds['hash-vectors.json']
    queries = [{'mode': 'hash', **{k: h[k] for k in ('id', 'raw_json_hex')}} for h in hs]
    observed = [timed_input(cmd, canonical({'registry': active_registry(), 'queries': queries}), os.environ.copy()) for _ in range(2)]
    require(observed[0][0] == observed[1][0], 'two exact actual Rust AI canonical/raw observations')
    results = {'rust-hashes': observed[0][0]}; runs = {'rust-hashes': [r for _, r in observed]}
    inventory = verify_host_toolchain(DEFAULT_GOROOT, strict_json((HERE/'host-toolchain.json').read_bytes()))
    env = {**os.environ, 'GOENV': 'off', 'GOWORK': 'off', 'GOTOOLCHAIN': 'local', 'GOPROXY': 'off', 'GOTELEMETRY': 'off',
           'CGO_ENABLED': '0', 'GOFLAGS': '', 'GOEXPERIMENT': '', 'GOCACHE': '/private/tmp/mpk-go-practical-checker-cache'}
    with tempfile.TemporaryDirectory(prefix='mpk-ai-hashes-', dir='/private/tmp') as t:
        executables = {}
        for name, source in [('canonical', 'proof_hashes.go'), ('raw', 'policy_raw_hashes.go')]:
            exe = Path(t)/name; b = subprocess.run([str(DEFAULT_GOROOT/'bin/go'), 'build', '-trimpath', '-buildvcs=false', '-o', str(exe), str(HERE/source)], cwd=ROOT, env=env, capture_output=True, timeout=300)
            require(b.returncode == 0 and not b.stderr, 'independent Go AI hash observer build'); executables[name] = exe
        payload = canonical([{k: h[k] for k in ('id', 'raw_json_hex')} | {'domain': 'MPK-GO-IDENTITY-PILOT-1.0'} for h in hs]); observations = []
        for _ in range(2):
            raw, first = timed_input([executables['canonical']], payload, env); values = strict_json(raw)
            require(all(v['accepted'] is True for v in values), 'Go independently canonicalizes every AI root')
            raw, second = timed_input([executables['raw']], canonical([{'id': v['id'], 'canonical_hex': v['canonical_hex']} for v in values]), env)
            observations.append((raw, {'canonicalizer': first, **second}))
        require(observations[0][0] == observations[1][0], 'two exact independent Go canonical/raw SHA observations')
        results['go-hashes'] = observations[0][0]; runs['go-hashes'] = [r for _, r in observations]
    require(before == source_inputs() and sha((TARGET/'debug/go-practical-policy-native').read_bytes()) == audit['binary_sha256'], 'all source/native identities fixed throughout hash execution')
    if args.write:
        for n, v in ds.items(): (RECORD/n).write_bytes(canonical(v)+b'\n')
        for n, raw in results.items(): (RECORD/(n+'.json')).write_bytes(raw)
    report = {'schema': 'mpk.go_practical.ai_contract_probe.v0', 'scope': SCOPE, 'pending': PENDING, 'inputs': inputs(),
        'native_AI_check_manifest_sha256': sha((RECORD/'verification/native-ai-checks.json').read_bytes()),
        'public_hash_observer_audit': audit, 'host_toolchain_inventory': inventory, 'runs': runs,
        'cases': len(ds['vectors.json']), 'outcomes': dict(Counter(v['classification'] for v in ds['vectors.json'])),
        'bundles': len(ds['bundles.json']), 'profiles': 5, 'languages': ['en', 'ja'], 'hash_vectors': len(hs),
        'request_batches': sum(len(b['requests']) for b in ds['bundles.json'].values()), 'native_AI_tests': 2, 'native_AI_runs': 2,
        'Go_unelaborated_demands': 169, 'compiled_AI_v4_executed': False, 'provider_contacted': False,
        'Go_ordinary_elaborated': False, 'profile_activated': False, 'W04_frozen': False,
        'execution_scope': 'local emulated Linux amd64 native consumer tests/hashes; no registered Go producer,external provider,native Java AI test or native installed-release gate'}
    if args.write: (RECORD/'observations.json').write_bytes(json_bytes(report))
    old = check_retained(); require(old['public_hash_observer_audit'] == audit, 'fresh actual hash observer build identity')
    print('PASS:', report['cases'], 'AI cases,16 bundles/24 batches,169 pending Go demands,40 raw hashes and two real native AI tests run twice; W04 open', flush=True)


if __name__ == '__main__': main()
