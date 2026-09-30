import concurrent.futures
import json
import os
import subprocess
import time
from datetime import datetime, timezone
from hashlib import sha256
from pathlib import Path

source = Path('/tmp/mpk-w09-construction-storage-types/attempt-1/certificates')
out = Path('/tmp/mpk-w09-construction-storage-types/checks')
out.mkdir(parents=True, exist_ok=True)
binaries = {'go': Path('/tmp/mpk-w09-default-closed-proofs/go-checker'),
            'rust': Path('/Users/kazuyoshitoshiya/mpk/target/release/mpk')}
digests = {'go': 'd631f01eb0beaab7b83bebd53e9f66b02f3a6a8c04e173b3af0acb48b9b305a2',
           'rust': 'b5207abd1d92acc1fd8b92a995ed700e1a2d929e6254ffe20b6f9f5d868bac91'}
for backend, path in binaries.items():
    assert sha256(path.read_bytes()).hexdigest() == digests[backend]
cases = sorted(p for p in source.glob('*.hex') if not p.stem.endswith('-wrong'))
assert len(cases) == 45
proofs = sum(len(json.loads(p.with_suffix('.json').read_text())['proofs']) for p in cases)
assert proofs == 87
state = dict(status='running', supervisor_pid=os.getpid(), proofs_supplied=proofs,
             source_contexts=45, launches={}, stages=[], binaries=digests, new_stages=[], retained_stages=[],
             application_proofs_pending=987, concrete_type_instances_pending=0, private_storage_domains=6, source_ownership_pending=True,
             full_t_gate='deferred to T06-W12',
             selection_reason='Replay the six changed original-source private construction type proof certificates with both unchanged binaries. Require zero axioms, matching hashes/counts and hash-corruption rejection; reject a well-typed constant-true concrete type definition at core checking when proof candidates are supplied. Retain the other 39 certificate checks and earlier wrong-proof negative only after exact current input, binary, raw report and stderr verification. Ownership, native execution and all 987 application proof IDs remain pending; the full T06 gate is deferred to W12.')

previous_root = Path('/private/tmp/mpk-w09-packed-pattern-proofs/develop/migrations/csharp-03/ordinary-foundation/verification-logs/concrete-type-proofs/checks')
previous = json.loads((previous_root / 'verification.json').read_bytes())
assert previous['status'] == 'passed' and len(previous['stages']) == 182
old_rows = {(r['backend'], r['case'], r['kind']): r for r in previous['stages']}
state['previous_receipt_sha256'] = sha256((previous_root / 'verification.json').read_bytes()).hexdigest()
manifest = json.loads(Path('/tmp/mpk-w09-construction-storage-types/source-manifest.json').read_bytes())
repo = Path('/private/tmp/mpk-w09-packed-pattern-proofs')

def verify_sources():
    for group in ('source_hashes', 'fixture_hashes'):
        for name, digest in manifest[group].items():
            assert sha256((repo / name).read_bytes()).hexdigest() == digest, name

verify_sources()
def persist():
    path = out / 'status.tmp'
    path.write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
    os.replace(path, out / 'status.json')

def run(backend, stem, data):
    path = out / (stem + '.mpcert')
    command = [str(binaries[backend])] + (['check'] if backend == 'rust' else []) + [str(path)]
    started = datetime.now(timezone.utc).isoformat()
    start = time.monotonic()
    with (out / (stem + '-' + backend + '.json')).open('wb') as stdout, (out / (stem + '-' + backend + '.stderr')).open('wb') as stderr:
        process = subprocess.Popen(command, stdout=stdout, stderr=stderr)
        return process, dict(backend=backend, command=command, process_pid=process.pid,
                             started_at=started, input_file_sha256=sha256(data).hexdigest(),
                             binary_sha256=digests[backend]), start

persist()
for case in cases:
    data = bytes.fromhex(case.read_text())
    jobs = [(case.stem + '-positive', data, 'positive'),
            (case.stem + '-hash', data[:-1] + bytes([data[-1] ^ 1]), 'hash')]
    wrong = source / (case.stem + '-wrong.hex')
    if wrong.exists():
        jobs.append((case.stem + '-wrong', bytes.fromhex(wrong.read_text()), 'wrong'))
    for stem, contents, kind in jobs:
        (out / (stem + '.mpcert')).write_bytes(contents)
        verify_sources()
        retained = {}
        active = {}
        for backend in binaries:
            old = old_rows.get((backend, case.stem, kind))
            if old and old['input_file_sha256'] == sha256(contents).hexdigest() and old['binary_sha256'] == digests[backend]:
                assert (previous_root / (stem + '.mpcert')).read_bytes() == contents
                stdout = (previous_root / (stem + '-' + backend + '.json')).read_bytes()
                stderr = (previous_root / (stem + '-' + backend + '.stderr.txt')).read_bytes()
                assert sha256(stdout).hexdigest() == old['report_sha256']
                assert sha256(stderr).hexdigest() == old['stderr_sha256']
                (out / (stem + '-' + backend + '.json')).write_bytes(stdout)
                (out / (stem + '-' + backend + '.stderr')).write_bytes(stderr)
                retained[backend] = old
            else:
                active[backend] = run(backend, stem, contents)
        state['launches'] = {b: v[1] for b, v in active.items()}
        state['stage'] = stem
        persist()
        exits = {b: r['exit_code'] for b, r in retained.items()}
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
            exits.update(dict(zip(active, pool.map(lambda b: active[b][0].wait(), active))))
        reports = {}
        for backend in binaries:
            stdout = (out / (stem + '-' + backend + '.json')).read_bytes()
            stderr = (out / (stem + '-' + backend + '.stderr')).read_bytes()
            report = json.loads(stdout)
            assert exits[backend] == (0 if kind == 'positive' else 1), (stem, backend, exits[backend])
            assert report['verdict'] == ('accepted' if kind == 'positive' else 'rejected'), (stem, backend)
            certificate_hash = sha256(b'MPK-MODULE-CERT-0.1\0' + contents).hexdigest()
            if kind == 'positive':
                if backend == 'go':
                    core = report['report']
                    assert core['AxiomCount'] == 0
                    assert all(value == 0 for value in core['AxiomReport']['Summary'].values())
                    hashes = {key: bytes(core[field]).hex() for key, field in [('export', 'ExportHash'), ('axiom_report', 'AxiomReportHash'), ('certificate', 'CertificateHash')]}
                    counts = (core['Module'], core['DeclarationCount'], core['AxiomCount'])
                else:
                    assert report['axiom_count'] == 0
                    assert all(value == 0 for value in report['axiom_report']['summary'].values())
                    hashes = report['hashes']
                    counts = (report['module'], report['declaration_count'], report['axiom_count'])
                assert hashes['certificate'] == certificate_hash
                reports[backend] = (hashes, counts)
            else:
                assert (report['certificate'] if backend == 'go' else report['hashes']['certificate']) == certificate_hash
                if kind == 'wrong':
                    assert (report.get('error_kind') == 'core_check' if backend == 'go' else report.get('error_code') == 'KERNEL_CORE_CHECK')
            if backend in retained:
                row = dict(retained[backend], retained_from='concrete-type-proofs/checks/verification.json', retained_input_and_binary_exact=True)
                state['retained_stages'].append(row)
            else:
                process, launch, start = active[backend]
                row = dict(launch, case=case.stem, kind=kind, exit_code=exits[backend],
                           finished_at=datetime.now(timezone.utc).isoformat(),
                           elapsed_seconds=round(time.monotonic() - start, 3),
                           certificate_sha256=certificate_hash,
                           report_sha256=sha256(stdout).hexdigest(), stderr_sha256=sha256(stderr).hexdigest())
                state['new_stages'].append(row)
            assert row['certificate_sha256'] == certificate_hash
            assert sha256(stdout).hexdigest() == row['report_sha256'] and sha256(stderr).hexdigest() == row['stderr_sha256']
            state['stages'].append(row)
            persist()
        if kind == 'positive':
            assert reports['go'] == reports['rust'], stem
        if active:
            print(stem, 'passed both checkers', flush=True)
state.update(status='passed', finished_at=datetime.now(timezone.utc).isoformat(), launches={})
assert len(state['stages']) == 184
assert len(state['retained_stages']) == 158 and len(state['new_stages']) == 26
verify_sources()
persist()
(out / 'verification.json').write_text(json.dumps(state, indent=2, sort_keys=True) + '\n')
