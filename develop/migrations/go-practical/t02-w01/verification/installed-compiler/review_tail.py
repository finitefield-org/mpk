"""Apply the existing scoped scanner to final stage-audit inputs."""
from pathlib import Path
import hashlib
import json
import sys
import types

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[5]


def main():
    scanner = REPO/'scripts/check-no-active-gir.sh'
    code = scanner.read_text().split("<<'PY'\n", 1)[1].rsplit('\nPY', 1)[0].rsplit('raise SystemExit(main())', 1)[0]
    module = types.ModuleType('installed_go_stage_audit_review')
    module.__file__ = str(scanner)
    sys.modules[module.__name__] = module
    sys.argv = [str(scanner), str(REPO), 'strict']
    exec(compile(code, str(scanner), 'exec'), module.__dict__)
    matchers = module.load_matchers()
    manifest, _, _, _ = module.validate_manifest()
    module.run_fixture_self_test(manifest, matchers)
    assert (len(matchers), len(module.SCOPES), len(manifest['sealed_records']), len(module.active_retained_record_exclusions(manifest))) == (127, 4, 39, 51)
    names = ['audit_partial_stage.py','audit-partial-stage.json','staged-whitespace-all.log','staged-whitespace-all.json','staged-whitespace-code.log','staged-whitespace-code.json','review-additions.json','review_tail.py']
    inputs, findings = [], []
    for name in names:
        path = ROOT/name
        data = path.read_bytes()
        inputs.append({'path':str(path.resolve()),'bytes':len(data),'sha256':hashlib.sha256(data).hexdigest()})
        findings.extend(vars(item) for item in module.detect(str(path.relative_to(REPO)),data.decode(errors='ignore'),matchers))
        if path.suffix == '.py':
            compile(data,str(path),'exec')
    result = {'status':'passed' if not findings else 'failed','inputs':inputs,'findings':findings,'matcher_count':len(matchers),'whole_repository_gate_run':False,'full_T_gate':'Deferred to final T02-W04: ./scripts/check-fast.sh'}
    (ROOT/'review-tail.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps({'status':result['status'],'inputs':len(inputs),'findings':len(findings)}))
    raise SystemExit(bool(findings))


if __name__ == '__main__':
    main()
