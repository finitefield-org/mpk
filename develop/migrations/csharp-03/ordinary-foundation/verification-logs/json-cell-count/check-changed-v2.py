from pathlib import Path
import subprocess, os, json, hashlib, re, sys, time
repo = Path('/Users/kazuyoshitoshiya/mpk')
base = repo / 'develop/migrations/csharp-03/ordinary-foundation'
plan = json.loads((base / 'unit-4-json-cell-count-replacements.json').read_text())
result_path = base / 'verification-logs/json-cell-count/changed-checker-results.json'
results = json.loads(result_path.read_text()) if result_path.exists() else []
def certificate_hash(path):
    return hashlib.sha256(b'MPK-MODULE-CERT-0.1\0' + bytes.fromhex(path.read_text())).hexdigest()
def passed_files(data, test):
    return set(re.findall(r'^\s*--- PASS: ' + re.escape(test) + r'/([^ /]+\.hex) \(', data, re.MULTILINE))
for group in plan['checker_groups']:
    family = group['family']
    expected = {r['file']: r['new_sha256'] for r in plan['rows'] if r['family'] == family and r['file'] in group['files']}
    actual = {name: certificate_hash(base / family / name) for name in group['files']}
    assert actual == expected, ('candidate changed', family)
    test = group['run_filter'].split('$/')[0].removeprefix('^')
    passed = set()
    for prior in results:
        if prior['family'] != family: continue
        log = (repo / prior['retained_log']).read_bytes()
        assert hashlib.sha256(log).hexdigest() == prior['log_sha256'], ('changed result log', family)
        for name in passed_files(log.decode(), test):
            if name in actual and prior['candidates'].get(name) == actual[name]: passed.add(name)
    remaining = {name: digest for name, digest in actual.items() if name not in passed}
    if not remaining:
        print(f'Skip {family}: all {len(actual)} exact candidates already passed', flush=True)
        continue
    run_filter = '^' + test + '$/^(' + '|'.join(re.escape(n) for n in remaining) + ')$'
    version = 2
    while (Path('/tmp') / f'mpk-w09-json-cell-count-checkers-{family}-v{version}.log').exists(): version += 1
    log = Path('/tmp') / f'mpk-w09-json-cell-count-checkers-{family}-v{version}.log'
    args = ['go', 'test', '-timeout', '0', '-tags', 'checkeragreement', '-run', run_filter, '-count=1', '-v']
    print(f'Starting {family}: {len(remaining)} candidates; retain {len(passed)} completed candidates', flush=True)
    started = time.time()
    with log.open('wb') as out:
        result = subprocess.run(args, cwd=repo / 'go-tools/mpk-checker-ref', env={**os.environ, 'GOCACHE': '/tmp/mpk-w09-go-cache'}, stdout=out, stderr=subprocess.STDOUT)
    data = log.read_bytes()
    retained = base / 'verification-logs/json-cell-count' / log.name
    retained.write_bytes(data)
    observed = passed_files(data.decode(), test)
    status = result.returncode if result.returncode else (0 if observed == set(remaining) else 1)
    results.append(dict(family=family, candidates=remaining, command=args, exit_code=status, process_exit_code=result.returncode, passed_files=sorted(observed), seconds=round(time.time()-started, 2), log=str(log), retained_log=str(retained.relative_to(repo)), log_sha256=hashlib.sha256(data).hexdigest()))
    temp = result_path.with_suffix('.tmp'); temp.write_text(json.dumps(results, indent=2) + '\n'); temp.replace(result_path)
    print(f'Finished {family}: exit{status}', flush=True)
    if status: sys.exit(status)
