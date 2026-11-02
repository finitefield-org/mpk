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
module=types.ModuleType('mpk_word_shifts_input_scanner');module.__file__=str(repo/script);sys.modules[module.__name__]=module
argv=sys.argv;sys.argv=[str(script),str(repo),'strict']
try:exec(compile(code,str(script),'exec'),module.__dict__);matchers=module.load_matchers()
finally:sys.argv=argv
manifest,scanner_files,fixtures,_=module.validate_manifest()
module.run_fixture_self_test(manifest,matchers)
retained=module.active_retained_record_exclusions(manifest)
configured=scanner_files|fixtures|retained
exemptions=json.loads((out/'raw-formatter-observations.json').read_text())['observations']
assert len(exemptions)==7
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
record={'status':'passed' if not(findings or whitespace or git_errors or missing) else 'failed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_base_commit':base,'selection_reason':'Actual fixed-right and full64 high-count prerequisite component,3105decl/77415terms/1100109bytes, zero axioms.2669 exact original named identities include all1882 conversions and four existing Builder seed definitions omitted from the original2665-root count expectation. Production and old signed-division raw pins remain exact. Three unique affected roots complete across the combined101-control/504-model run and the one corrected metadata-root rerun. Only assertion/report count literals change; unaffected semantic roots are not repeated. Both unchanged cores inspect103 complete same-byte streams,2accepted101 genuine type rejections. All504 original full rows, full64 count signedness routes/source stages and4212 fixed-right Bool observations match typed Go and Python integers, with24 real negative-count panics480defined309large-defined. Current selected Clippy/Rust/Go format/ignored-helper listing pass. Preserve original compiler/complete type failure/prefix diagnostic/count-reader/count-assertion/observer-subtype/native-key/early-report-lookup observations and their exact snapshots/raw streams; tool transcriptions are labeled. Observer subtype matcher repair reuses the exact original first core pair, then inspects remaining100 once. Full variable-shift stage/saturation/signed-Int/source/context/application/profile/Linux release work remains due; W01/T02-T11 and169demands stay active. Frozen255inputs, official127-rule/four-scope scanner39seals unchanged. Only7 exact per-file hash/length original libtest EOF observations; no source/JSON/directory/blanket exclusions, CSharp tests, unaffected suite repetitions or intermediate whole gate. Cumulative T02 gateW04 --skip-csharp.','scanner':{**seal(script),'matchers':len(matchers),'scope_count':len(module.SCOPES),'definitions_changed':False},'inputs':inputs,'selected_files':len(inputs),'findings':findings,'whitespace_errors':whitespace,'git_errors':git_errors,'raw_stdout_EOF_observations':eofs,'local_markdown_links_checked':len(links),'links':links,'missing_links':missing,'validated_exact_historical_seals':len(manifest['sealed_records']),'changed_exact_historical_record_observations':historical,'source_JSON_directory_exclusions':False,'blanket_whitespace_exclusions':False,'production_or_directory_exclusions_added':False,'full_gate_passed':False}
review.write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({k:record[k] for k in ['status','selected_files','findings','whitespace_errors','git_errors','local_markdown_links_checked','missing_links']}))
if record['status']!='passed':raise SystemExit(1)
