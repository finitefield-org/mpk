"""Scoped decoded-original content review; not the repository/T02 whole gate."""
from pathlib import Path
import hashlib,json,runpy,subprocess,sys,types
ROOT=Path(__file__).resolve().parent;REPO=Path.cwd()
BASE='cd427ec4e6d7a7b2e6a0628da1a20fed94475019'
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
assert git('rev-parse','HEAD').decode().strip()==BASE
out=ROOT/'archive-content-review';out.mkdir(exist_ok=False)
driver=out/'driver.py';driver.write_bytes(Path(__file__).read_bytes())
scanner=Path('scripts/check-no-active-gir.sh')
code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
m=types.ModuleType('source_error_representation_virtual_review');m.__file__=str(REPO/scanner);sys.modules[m.__name__]=m;sys.argv=[str(scanner),str(REPO),'strict']
exec(compile(code,str(scanner),'exec'),m.__dict__)
matchers=m.load_matchers();manifest,configured,fixtures,_=m.validate_manifest();m.run_fixture_self_test(manifest,matchers);exclusions=m.active_retained_record_exclusions(manifest)
assert (len(matchers),len(m.SCOPES),len(manifest['sealed_records']),len(exclusions))==(127,4,39,51)
for name in configured|fixtures:assert Path(name).read_bytes()==git('show',BASE+':'+name)
virtual=runpy.run_path(str(ROOT/'virtual-originals.py'));cache={};rows=[];findings=[]
path_rules=[r for r in matchers if r['scope']=='exact-path']
for path,(raw,mode,archive,member) in virtual['index']().items():
    name=str(path.relative_to(REPO)) if path.is_relative_to(REPO) else str(path)
    digest=hashlib.sha256(raw).hexdigest();key=(digest,tuple(m.path_matches(name,r['path']) for r in path_rules))
    if key not in cache:cache[key]=[vars(f) for f in m.detect(name,raw.decode(errors='ignore'),matchers)]
    findings.extend({**f,'path':name} for f in cache[key])
    rows.append({'original_path':name,'mode':mode,'bytes':len(raw),'sha256':digest,'archive':archive,'member':member})
record={'status':'passed' if not findings else 'failed','actual_driver_prelaunch_sha256':hashlib.sha256(driver.read_bytes()).hexdigest(),'basis_commit':BASE,'matcher_count':127,'scopes':4,'seals':39,'exclusions':51,'scanner_rules_unchanged':True,'originals':rows,'findings':findings,'transport_was_decoded_before_scanning':True,'no_original_source_text_excluded_or_renamed_for_review':True,'whole_repository_gate_passed':False,'scope':'Only the newly retained native outputs and changed prelaunch snapshots; their actual original text/paths/modes remain audited independently of archive transport.'}
(out/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':record['status'],'original_files':len(rows),'findings':len(findings)}));raise SystemExit(bool(findings))
