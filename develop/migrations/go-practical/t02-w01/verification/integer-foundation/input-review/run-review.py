"""Review all changed bytes with the unchanged official strict scanner."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,sys,types,urllib.parse

repo=Path.cwd();out=Path(sys.argv[1]);base=sys.argv[2]
review=out/'changed-inputs-review.json'
def git(*args):
 return subprocess.run(['/usr/bin/git',*args],stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
def seal(p):
 p=Path(p);raw=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
assert git('rev-parse','HEAD').stdout.decode().strip()==base
changed=git('diff','--name-only','-z','HEAD','--');untracked=git('ls-files','--others','--exclude-standard','-z')
assert changed.returncode==0 and untracked.returncode==0
paths=sorted(set(p for p in (changed.stdout+untracked.stdout).decode().split('\0') if p)-{str(review)})
script=Path('scripts/check-no-active-gir.sh');raw=script.read_text()
code=raw.split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0]
assert code.rstrip().endswith('raise SystemExit(main())')
code=code.rsplit('raise SystemExit(main())',1)[0]
module=types.ModuleType('mpk_integer_input_scanner');module.__file__=str(repo/script);sys.modules[module.__name__]=module
argv=sys.argv;sys.argv=[str(script),str(repo),'strict']
try:exec(compile(code,str(script),'exec'),module.__dict__);matchers=module.load_matchers()
finally:sys.argv=argv
manifest,scanner_files,fixtures,_=module.validate_manifest()
module.run_fixture_self_test(manifest,matchers)
retained=module.active_retained_record_exclusions(manifest)
configured=scanner_files|fixtures|retained
exemptions=json.loads((out/'raw-formatter-observations.json').read_text())['observations']
assert len(exemptions)==2
inputs=[];findings=[];whitespace=[];git_errors=[];eofs=[];links=[];historical=[]
for name in paths:
 p=Path(name)
 if not p.is_file():continue
 record=seal(p);inputs.append(record);text=p.read_text(errors='ignore')
 if name in configured:historical.append(record)
 else:findings.extend(vars(f) for f in module.detect(name,text,matchers))
 run=git('diff','--no-index','--check','--','/dev/null',name)
 message=(run.stdout+run.stderr).decode(errors='replace')
 if message:
  exact=exemptions.get(name)
  if exact and exact['sha256']==record['sha256'] and exact['bytes']==record['bytes'] and exact['kind']=='exact_original_libtest_stdout_EOF' and all(line.endswith(': new blank line at EOF.') for line in message.splitlines() if line):
   eofs.append({**record,'exact_raw_stdout_EOF_preserved':True,'git_whitespace_output':message})
  else:whitespace.append({'path':name,'exit_code':run.returncode,'message':message})
 elif run.returncode not in [0,1]:git_errors.append({'path':name,'exit_code':run.returncode})
 if p.suffix=='.md':
  for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^\n)]+)\)',text):
   target=target.strip().strip('<>')
   if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target) or target.startswith('#'):continue
   clean=urllib.parse.unquote(target.split('#',1)[0])
   links.append({'document':name,'target':target,'exists':(repo/p.parent/clean).resolve().exists()})
missing=[r for r in links if not r['exists']]
assert {r['path'] for r in eofs}==set(exemptions)
record={'status':'passed' if not(findings or whitespace or git_errors or missing) else 'failed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_base_commit':base,'selection_reason':'Actual balanced integer ordinary representation, ring/equivalence/congruence/observation laws, exact importer and complete retained Nat basis. Polynomial rewriting emits only proofs using previously checked Nat laws, with actual premise-bearing checkpoints. Current independent checkers verify each original full candidate; initial embedding-injection proof failure and all exact input/diagnostic bytes remain. Every changed byte is scanned with unchanged127-rule/four-scope policy and all39 historical seals/fixtures. Only two exact original libtest EOF observations; no source/JSON/directory or blanket whitespace exclusions. Full T02 gate remains deferred to W04 and CSharp tests skipped.','scanner':{**seal(script),'matchers':len(matchers),'scope_count':len(module.SCOPES),'definitions_changed':False},'inputs':inputs,'selected_files':len(inputs),'findings':findings,'whitespace_errors':whitespace,'git_errors':git_errors,'raw_stdout_EOF_observations':eofs,'local_markdown_links_checked':len(links),'links':links,'missing_links':missing,'validated_exact_historical_seals':len(manifest['sealed_records']),'changed_exact_historical_record_observations':historical,'source_JSON_directory_exclusions':False,'blanket_whitespace_exclusions':False,'production_or_directory_exclusions_added':False,'full_gate_passed':False}
review.write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({k:record[k] for k in ['status','selected_files','findings','whitespace_errors','git_errors','local_markdown_links_checked','missing_links']}))
if record['status']!='passed':raise SystemExit(1)
