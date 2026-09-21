"""Check changed pins individually, enforcing the user's sub-minute agent limit."""
import json, os, re, signal, subprocess, time, hashlib
from pathlib import Path
repo = Path(__file__).resolve().parents[6]
out = Path(__file__).resolve().parent
families = json.loads((out / 'planned-families.json').read_text())
tests = {'aggregate-folds': 'AggregateFolds', 'domains': 'RecursiveDomains', 'structural-foundations': 'StructuralFoundations', 'sequence-operations':'SequenceOperations', 'construction-operations':'ConstructionOperations', 'outcome-operations':'OutcomeOperations', 'entry-operations':'EntryOperations', 'collection-operations':'CollectionOperations', 'source-observations':'SourceObservations', 'money-operations':'MoneyOperations'}
results = json.loads((out/'checker-results.json').read_text()) if (out/'checker-results.json').exists() else []
completed = {(r['family'],r['file']):r for r in results}
for family in families:
    for filename in family['changed']:
        previous = completed.get((family['family'],filename))
        if previous is not None:
            assert previous['exit_code'] in (0, 124), 'Failed check requires review before resuming'
            pin=repo/'develop/migrations/csharp-03/ordinary-foundation'/family['family']/filename
            assert hashlib.sha256(pin.read_bytes()).hexdigest()==previous['certificate_file_sha256'], 'Changed pin needs a new receipt'
            assert hashlib.sha256((out/previous['log']).read_bytes()).hexdigest()==previous['log_sha256'], 'Changed log needs review'
            continue
        label = family['family'] + '__' + filename[:-4]
        command = ['/tmp/mpk-source-value-checker-tests', '-test.run', '^TestCheckerAgreementWithRustCLI' + tests[family['family']] + '$/^' + re.escape(filename) + '$', '-test.v', '-test.timeout', '25m']
        print('START', label, flush=True)
        start = time.monotonic()
        with (out / (label + '.log')).open('w') as log:
            process = subprocess.Popen(command, cwd=repo/'go-tools/mpk-checker-ref', stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
            try:
                code = process.wait(timeout=55)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait()
                code = 124
        path = repo/'develop/migrations/csharp-03/ordinary-foundation'/family['family']/filename
        result = {'family':family['family'], 'file':filename, 'command':command, 'seconds':time.monotonic()-start, 'exit_code':code, 'status':'passed' if code==0 else 'user_long_check_pending' if code==124 else 'failed', 'certificate_file_sha256':hashlib.sha256(path.read_bytes()).hexdigest(), 'log':label+'.log', 'log_sha256':hashlib.sha256((out/(label+'.log')).read_bytes()).hexdigest()}
        if code==0:
            text = (out/(label+'.log')).read_text()
            assert ('/'+filename+' (') in text and '\nPASS\n' in text, 'No matching checker subtest'
        results.append(result)
        (out/'checker-results.json').write_text(json.dumps(results,indent=2)+'\n')
        print('END', label, result['status'], round(result['seconds'],2), flush=True)
        if code not in [0,124]:
            raise SystemExit(code)
print('DONE', len(results), 'passed', sum(r['exit_code']==0 for r in results), 'pending', sum(r['exit_code']==124 for r in results), flush=True)
