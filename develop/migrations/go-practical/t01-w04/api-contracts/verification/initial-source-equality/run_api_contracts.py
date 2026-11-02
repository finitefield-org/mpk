#!/usr/bin/env python3
"""Record/check API v4 design vectors and fresh retained API/hash observations."""
import argparse
from collections import Counter
import os
from pathlib import Path
import subprocess
import tempfile
import traceback
from api_json import canonical
from api_contracts import catalogue
from api_vectors import documents, hash_cases, PREVIOUS
from context_protocol import active_registry
from run_policy_roots import inputs as previous_inputs, check_retained as previous_check
from policy_native_build import build, TARGET
from policy_sources import source_rows
from run_proof_pilot import ROOT, HERE, DEFAULT_GOROOT, sha, json_bytes, verify_host_toolchain
from run_structural_limits import timed_input
from check_scope_report import require, strict_json

RECORD = PREVIOUS.parent/'api-contracts'
NEW = ('api_json.py', 'api_contracts.py', 'api_protocol.py', 'api_vectors.py',
       'run_api_contracts.py', 'test_api_contracts.py', 'check_api_metadata.py')
DOCS = ('schema.json', 'bundles.json', 'vectors.json', 'native-queries.json', 'hash-vectors.json')
SCOPE = 'inactive API v4 request/response/session design,all33 declared routes,atomic source-state transactions and exact local response/snapshot binding; six fresh retained APIv2 traces and independent small-document canonical hashes'
PENDING = ['compiled APIv4 dispatch,all ordinary/candidate executors and native full API envelope budgets remain T07 implementation work; the design model never simulates their success',
           'AI/release roots,complete Go ordinary program/skeleton signatures and reviewed producer/consumer migration closure remain W04 design work',
           'source/ordinary elaborator and operation/definedness/control/frame proofs belong to T02–T06; W05 inactive publication/T01 gate and the full T02–T11 objective remain required']


def source_inputs():
    rows = {r['path']: r['sha256'] for r in previous_inputs()}
    paths = [PREVIOUS/n for n in ('observations.json', 'api-trials.json', 'native.json')]
    paths += [HERE/n for n in NEW]
    for p in paths: rows[p.relative_to(ROOT).as_posix()] = sha(p.read_bytes())
    return [{'path': p, 'sha256': h} for p, h in sorted(rows.items())]


def inputs():
    return source_inputs() + [{'path': (RECORD/n).relative_to(ROOT).as_posix(),
                              'sha256': sha((RECORD/n).read_bytes())} for n in DOCS]


def design_documents():
    bundles, vectors = documents()
    trials = strict_json((PREVIOUS/'api-trials.json').read_bytes())
    return dict(zip(DOCS, (catalogue(), bundles, vectors, [r['query'] for r in trials], hash_cases(bundles))))


def expected_hashes(hs):
    return [{k: h[k] for k in ('id', 'accepted', 'canonical_hex', 'canonical_sha256')} for h in hs]


def check_retained():
    report = strict_json((RECORD/'observations.json').read_bytes()); ds = design_documents()
    for name, value in ds.items(): require((RECORD/name).read_bytes() == canonical(value) + b'\n', 'exact API design reconstruction: ' + name)
    require(report['scope'] == SCOPE and report['pending'] == PENDING and report['inputs'] == inputs(), 'API scope/input closure')
    require(report['cases'] == len(ds['vectors.json']) and report['outcomes'] == dict(Counter(v['classification'] for v in ds['vectors.json'])), 'complete API vector outcomes')
    trials = strict_json((PREVIOUS/'api-trials.json').read_bytes()); expected = [r['result'] for r in trials]
    require(strict_json((RECORD/'native.json').read_bytes()) == expected, 'fresh retained API observations preserve exact previous responses/errors/state')
    require(strict_json((RECORD/'rust-hashes.json').read_bytes()) == expected_hashes(ds['hash-vectors.json']), 'actual Rust API canonical/raw SHA identities')
    require(strict_json((RECORD/'go-hashes.json').read_bytes()) == [{'id': h['id'], 'canonical_sha256': h['canonical_sha256']} for h in ds['hash-vectors.json']], 'actual independent Go canonical/raw SHA identities')
    for name in ('native', 'rust-hashes', 'go-hashes'):
        require(len(report['runs'][name]) == 2 and all(r['exit_code'] == 0 and r['stdout_sha256'] == sha((RECORD/(name+'.json')).read_bytes()) for r in report['runs'][name]), 'two complete deterministic observations')
    for flag in ('compiled_APIv4_executed', 'Go_ordinary_elaborated', 'profile_activated', 'W04_frozen'): require(report[flag] is False, 'design/implementation authority boundary')
    return report


def main():
    p = argparse.ArgumentParser(description=__doc__); g = p.add_mutually_exclusive_group(required=True)
    g.add_argument('--write', action='store_true'); g.add_argument('--check', action='store_true'); args = p.parse_args()
    before = source_inputs(); ds = design_documents(); cmd, audit = build()
    qs = ds['native-queries.json']; hs = ds['hash-vectors.json']; results = {}; runs = {}
    for name, queries in [('native', qs), ('rust-hashes', [{'mode': 'hash', **{k: h[k] for k in ('id', 'raw_json_hex')}} for h in hs])]:
        measured = [timed_input(cmd, canonical({'registry': active_registry(), 'queries': queries}), os.environ.copy()) for _ in range(2)]
        require(measured[0][0] == measured[1][0], 'two exact local native API observations')
        results[name] = measured[0][0]; runs[name] = [r for _, r in measured]
    inventory = verify_host_toolchain(DEFAULT_GOROOT, strict_json((HERE/'host-toolchain.json').read_bytes()))
    env = {**os.environ, 'GOENV': 'off', 'GOWORK': 'off', 'GOTOOLCHAIN': 'local', 'GOPROXY': 'off',
           'GOTELEMETRY': 'off', 'CGO_ENABLED': '0', 'GOFLAGS': '', 'GOEXPERIMENT': '', 'GOCACHE': '/private/tmp/mpk-go-practical-checker-cache'}
    with tempfile.TemporaryDirectory(prefix='mpk-api-hashes-', dir='/private/tmp') as t:
        # Existing Go canonicalizer produces these bytes independently; the raw
        # SHA observer consumes those Go bytes,not Python or Rust canonical output.
        exes = {}
        for name, source in [('canonical', 'proof_hashes.go'), ('raw', 'policy_raw_hashes.go')]:
            exe = Path(t)/name; r = subprocess.run([str(DEFAULT_GOROOT/'bin/go'), 'build', '-trimpath', '-buildvcs=false', '-o', str(exe), str(HERE/source)], cwd=ROOT, env=env, capture_output=True, timeout=300)
            require(r.returncode == 0 and not r.stderr, 'independent Go API hash observer build'); exes[name] = exe
        payload = canonical([{k: h[k] for k in ('id', 'raw_json_hex')} | {'domain': 'MPK-GO-IDENTITY-PILOT-1.0'} for h in hs])
        observations = []
        for _ in range(2):
            raw, canonical_run = timed_input([exes['canonical']], payload, env)
            values = strict_json(raw); require(all(v['accepted'] is True for v in values), 'Go independently canonicalizes every API sample')
            raw, raw_run = timed_input([exes['raw']], canonical([{'id': v['id'], 'canonical_hex': v['canonical_hex']} for v in values]), env)
            observations.append((raw, {'canonicalizer': canonical_run, **raw_run}))
        require(observations[0][0] == observations[1][0], 'two independent Go API canonical/raw hashes')
        results['go-hashes'] = observations[0][0]; runs['go-hashes'] = [r for _, r in observations]
    require(before == source_inputs() and sha((TARGET/'debug/go-practical-policy-native').read_bytes()) == audit['binary_sha256'], 'all API/source/native identities fixed during execution')
    if args.write:
        RECORD.mkdir(exist_ok=True)
        for n, value in ds.items(): (RECORD/n).write_bytes(canonical(value) + b'\n')
        for n, raw in results.items(): (RECORD/(n+'.json')).write_bytes(raw)
    report = {'schema': 'mpk.go_practical.api_contract_probe.v0', 'scope': SCOPE, 'pending': PENDING,
              'inputs': inputs(), 'public_source_audit': audit, 'runs': runs, 'host_toolchain_inventory': inventory,
              'cases': len(ds['vectors.json']), 'outcomes': dict(Counter(v['classification'] for v in ds['vectors.json'])),
              'routes': 33, 'source_bundles': 8, 'model_transactions': 96, 'native_queries': 6,
              'retained_native_API_operations': 102, 'retained_native_atomic_failures': 72, 'hash_vectors': len(hs),
              'compiled_APIv4_executed': False, 'Go_ordinary_elaborated': False, 'profile_activated': False, 'W04_frozen': False,
              'hash_observer_scope': '24 small complete API documents; the old native hash observer is not an API772-depth/transport-budget implementation',
              'resource_scope': 'local Linux amd64 emulation on arm64; proxy child metrics only,not native Linux release/checker process-tree evidence'}
    if args.write: (RECORD/'observations.json').write_bytes(json_bytes(report))
    old = check_retained(); require(old['public_source_audit'] == audit, 'fresh native build identity')
    print('PASS:', report['cases'], 'API cases,33 routes,96 model transactions,102 fresh retained API operations/72 atomic failures,24 raw hashes; W04 open', flush=True)


if __name__ == '__main__': main()
