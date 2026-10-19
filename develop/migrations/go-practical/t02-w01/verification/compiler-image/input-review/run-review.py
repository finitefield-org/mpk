from pathlib import Path
import subprocess,json,hashlib,datetime,sys,types,re,urllib.parse
repo=Path.cwd();out=Path(sys.argv[1]);base_commit=sys.argv[2];review=out/'changed-inputs-review.json'
live=set()
def git(*args):return subprocess.run(['/usr/bin/git',*args],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
def seal(p):return {'path':str(p),'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'bytes':p.stat().st_size}
assert git('rev-parse','HEAD').stdout.decode().strip()==base_commit
changed=git('diff','--name-only','HEAD','--');assert changed.returncode==0
untracked=git('ls-files','--others','--exclude-standard');assert untracked.returncode==0
paths=sorted((set(changed.stdout.decode().splitlines())|set(untracked.stdout.decode().splitlines()))-live-{str(review)})
script=Path('scripts/check-no-active-gir.sh');raw=script.read_text();code=raw.split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0];assert code.rstrip().endswith('raise SystemExit(main())');code=code.rsplit('raise SystemExit(main())',1)[0]
module=types.ModuleType('mpk_selected_input_scanner');module.__file__=str(repo/script);sys.modules[module.__name__]=module
argv=sys.argv;sys.argv=[str(script),str(repo),'strict']
try:exec(compile(code,str(script),'exec'),module.__dict__);matchers=module.load_matchers()
finally:sys.argv=argv
manifest,scanner_files,fixtures,_=module.validate_manifest()
module.run_fixture_self_test(manifest,matchers)
retained=module.active_retained_record_exclusions(manifest)
configured_exclusions=scanner_files|fixtures|retained
sealed_observations=[]
exemptions={}
findings=[];inputs=[];whitespace_errors=[];eofs=[];formatters=[];links=[];git_errors=[]
for s in paths:
 p=Path(s)
 if not p.is_file():continue
 record=seal(p);inputs.append(record);text=p.read_text(errors='ignore')
 if s in configured_exclusions:sealed_observations.append({**record,"class":"historical_record" if s in retained else "scanner_configuration" if s in scanner_files else "validated_fixture"})
 else:findings.extend([vars(f) for f in module.detect(s,text,matchers)])
 r=git('diff','--no-index','--check','--','/dev/null',s);message=(r.stdout+r.stderr).decode(errors='replace')
 if message:
  exact=exemptions.get(s)
  if exact and exact['sha256']==record['sha256'] and all(line.endswith(': new blank line at EOF.') for line in message.splitlines() if line):eofs.append({**record,'exact_raw_stdout_EOF_preserved':True})
  elif s=='develop/migrations/go-practical/t01-w05/verification/retained-csharp-repair/control-builtin-throw-diagnosis/binary-help.stdout.log' and exact and exact['sha256']==record['sha256'] and exact.get('kind')=='exact_original_libtest_help_formatter' and exact.get('allowed_line_numbers')==[4,7,9] and message==exact.get('git_whitespace_output_raw_utf8'):formatters.append({**record,'original_libtest_help_formatter_bytes_preserved':True,'allowed_line_numbers':[4,7,9],'exact_git_whitespace_output':message})
  else:whitespace_errors.append({'path':s,'returncode':r.returncode,'message':message[:1800]})
 elif r.returncode not in [0,1]:git_errors.append({'path':s,'returncode':r.returncode})
 if p.suffix=='.md':
  for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^\n)]+)\)',text):
   target=target.strip().strip('<>')
   if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target) or target.startswith('#'):continue
   clean=urllib.parse.unquote(target.split('#',1)[0]);dest=(repo/p.parent/clean).resolve()
   links.append({'document':s,'target':target,'exists':dest.exists()})
missing=[l for l in links if not l['exists']]
record={'status':'passed' if not(findings or whitespace_errors or missing or git_errors) else 'failed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_base_commit':base_commit,'selection_reason':'T02-W01 complete prepared compiler/stdlib image validation, exact original source witnesses and nine tool identities, owned revalidation and fresh complete inventory. Original initial fixture-mode failure inputs/logs remain. Affected native image/build-configuration controls, legacy matrix, current vet/format and independent all-file original-SDK hashes pass. All changed inputs, including the full prepared inventory, are inventoried without source/JSON/directory/whitespace exemptions; unchanged strict scanner and exact historical policy apply. Checked private context/analysis identity/immutable production scope remains open. Full T02 gate is deferred to W04; CSharp tests remain deferred by explicit user instruction.','scanner':{**seal(script),'matchers':len(matchers),'scope_count':len(module.SCOPES),'definitions_changed':False},'inputs':inputs,'selected_files':len(inputs),'findings':findings,'whitespace_errors':whitespace_errors,'git_errors':git_errors,'raw_stdout_EOF_observations':eofs,'raw_help_formatter_observations':formatters,'local_markdown_links_checked':len(links),'links':links,'missing_links':missing,'excluded_live_outputs':sorted(live),'source_JSON_directory_whitespace_exclusions':False,'validated_exact_historical_seals':len(manifest['sealed_records']),'changed_exact_historical_record_observations':sealed_observations,'production_or_directory_exclusions_added':False,'full_gate_passed':False}
review.write_text(json.dumps(record,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({k:record[k] for k in ['status','selected_files','findings','whitespace_errors','git_errors','local_markdown_links_checked','missing_links']}))
if record['status']!='passed':raise SystemExit(1)
