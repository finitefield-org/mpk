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
go_formatter=json.loads((out/'raw-formatter-observation.json').read_text())
assert go_formatter['path']=='develop/migrations/go-practical/t02-w01/verification/analysis-inputs/independent-build/binary-build-info.log'
assert go_formatter['sha256']=='e557c35a9ff43bf28d718e6c7f7e685a1b1acc150bff6b183e730f79c391b695' and go_formatter['bytes']==645
assert go_formatter['allowed_line_numbers']==[3,4,5,6]
assert go_formatter['exact_lines_hex']==['096d6f64096769746875622e636f6d2f66696e6974656669656c642d6f72672f6d706b2f676f2d746f6f6c732f676f327669720928646576656c29090a', '0964657009676f6c616e672e6f72672f782f6d6f640976302e32372e30090a', '0964657009676f6c616e672e6f72672f782f73796e630976302e31362e30090a', '0964657009676f6c616e672e6f72672f782f746f6f6c730976302e33362e30090a']
go_formatters=[]
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
  elif s==go_formatter['path'] and record['sha256']==go_formatter['sha256'] and record['bytes']==go_formatter['bytes'] and message==go_formatter['git_whitespace_output_raw_utf8'] and [line.hex() for line in p.read_bytes().splitlines(keepends=True)[2:6]]==go_formatter['exact_lines_hex']:go_formatters.append({**record,'original_go_build_info_formatter_bytes_preserved':True,'allowed_line_numbers':[3,4,5,6],'exact_git_whitespace_output':message})
  else:whitespace_errors.append({'path':s,'returncode':r.returncode,'message':message[:1800]})
 elif r.returncode not in [0,1]:git_errors.append({'path':s,'returncode':r.returncode})
 if p.suffix=='.md':
  for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^\n)]+)\)',text):
   target=target.strip().strip('<>')
   if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target) or target.startswith('#'):continue
   clean=urllib.parse.unquote(target.split('#',1)[0]);dest=(repo/p.parent/clean).resolve()
   links.append({'document':s,'target':target,'exists':dest.exists()})
missing=[l for l in links if not l['exists']]
record={'status':'passed' if not(findings or whitespace_errors or missing or git_errors) else 'failed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_base_commit':base_commit,'selection_reason':'T02-W01 exact original analysis-library snapshots/vendor ownership and complete owned frontend build closure, linked-library validation and shared rooted binary reader. Preserve original uncompiled TODO discrepancy and initial temporary-path-alias failure with exact inputs/logs. Two independently cached owned-source builds and an independently reconstructed third build match full binaries; affected compiler/current/legacy tests and current vet/format pass. All changed source, metadata, fixture binaries and raw diagnostics are inventoried without source/JSON/directory or blanket whitespace exclusions. One exact hash-bound645-byte Go CLI output retains only four observed empty-checksum formatter tabs. Actual complete ordinary foundation/context binding and application immutable replay remain open. Full T02 gate is deferred to W04; CSharp tests stay deferred by explicit user instruction.','scanner':{**seal(script),'matchers':len(matchers),'scope_count':len(module.SCOPES),'definitions_changed':False},'inputs':inputs,'selected_files':len(inputs),'findings':findings,'whitespace_errors':whitespace_errors,'git_errors':git_errors,'raw_stdout_EOF_observations':eofs,'raw_help_formatter_observations':formatters,'raw_go_build_info_formatter_observations':go_formatters,'local_markdown_links_checked':len(links),'links':links,'missing_links':missing,'excluded_live_outputs':sorted(live),'source_JSON_directory_exclusions':False,'blanket_whitespace_exclusions':False,'validated_exact_historical_seals':len(manifest['sealed_records']),'changed_exact_historical_record_observations':sealed_observations,'production_or_directory_exclusions_added':False,'full_gate_passed':False}
review.write_text(json.dumps(record,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({k:record[k] for k in ['status','selected_files','findings','whitespace_errors','git_errors','local_markdown_links_checked','missing_links']}))
if record['status']!='passed':raise SystemExit(1)
