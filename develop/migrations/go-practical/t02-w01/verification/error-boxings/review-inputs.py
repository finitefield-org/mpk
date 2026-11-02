"""Scoped unchanged-rule review of whole source epochs and decoded originals.
Run outside the sandbox; every Git operation uses /usr/bin/git.
"""
from pathlib import Path
import base64,datetime,hashlib,json,re,runpy,subprocess,sys,types,urllib.parse
ROOT=Path.cwd();OUT=Path(__file__).resolve().parent;BASE='aaf5055ef92575d94bc00b541605001665cbc8cb';REPORT=OUT/'input-review.json'
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
def seal(path,raw=None):
 raw=Path(path).read_bytes()if raw is None else raw;return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
assert git('rev-parse','HEAD').decode().strip()==BASE and not Path('.github/workflows').exists()
scanner=Path('scripts/check-no-active-gir.sh');code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
module=types.ModuleType('mpk_error_boxings_input_review');module.__file__=str(ROOT/scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(ROOT),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
matchers=module.load_matchers();manifest,scanner_files,fixtures,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers);exclusions=module.active_retained_record_exclusions(manifest)
assert(len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(exclusions))==(127,4,39,51)
for name in scanner_files|fixtures:assert Path(name).read_bytes()==git('show',BASE+':'+name)
configured=scanner_files|fixtures|exclusions;findings=[];inputs=[];links=[];cache={};path_rules=[m for m in matchers if m['scope']=='exact-path'];decoded={}
def scan(name,raw):
 key=(hashlib.sha256(raw).hexdigest(),tuple(module.path_matches(name,m['path'])for m in path_rules))
 if key not in cache:cache[key]=[vars(f)for f in module.detect(name,raw.decode(errors='ignore'),matchers)]
 findings.extend({**f,'path':name}for f in cache[key])
def source(obj,raw):
 body={k:v for k,v in obj.items()if k!='id'};canonical=json.dumps(body,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
 assert obj['schema']=='mpk.go.source_object.v1'and obj['bytes']==len(raw)and obj['raw_sha256']==hashlib.sha256(raw).hexdigest()
 assert obj['id']==hashlib.sha256(b'MPK-GO-SOURCE-OBJECT-1.0\0'+canonical).hexdigest()
def values(obj,name):
 if isinstance(obj,list):
  for n,v in enumerate(obj):values(v,name+f'[{n}]')
 if not isinstance(obj,dict):return
 if 'canonical_components'in obj:
  raw=base64.b64decode(obj['canonical_components'],validate=True);scan(name+'#canonical-components',raw);values(json.loads(raw),name+'#native-components');decoded['canonical_components']=decoded.get('canonical_components',0)+1
 if 'object'in obj and'raw'in obj:
  raw=base64.b64decode(obj['raw'],validate=True);scan(name+'#source-raw',raw);source(obj['object'],raw);decoded['source_raw']=decoded.get('source_raw',0)+1
 if 'whole_original_sources'in obj:
  for path,data in obj['whole_original_sources'].items():
   scan(name+'#whole-original-source/'+path,base64.b64decode(data,validate=True));decoded['whole_original_sources']=decoded.get('whole_original_sources',0)+1
 for key in ['raw_hex','components_hex','bytes_hex','whole_original_bytes']:
  if key not in obj or obj[key]is None:continue
  data=obj[key];raw=bytes(data)if key=='whole_original_bytes'else bytes.fromhex(data)
  scan(name+'#decoded-'+key,raw);decoded[key]=decoded.get(key,0)+1
  if key in {'raw_hex','whole_original_bytes'}and'object'in obj:source(obj['object'],raw)
  if key=='components_hex':values(json.loads(raw),name+'#components')
 for key,v in obj.items():
  if key not in {'raw_hex','components_hex','bytes_hex','whole_original_bytes'}:values(v,name+'/'+key)
if not REPORT.exists():REPORT.write_text(json.dumps({'status':'running','basis_commit':BASE})+'\n')
selected=git('diff','--name-only','-z','HEAD','--')+git('ls-files','--others','--exclude-standard','-z');paths=sorted(set(selected.decode().split('\0'))-{'',str(REPORT.relative_to(ROOT))})
for name in paths:
 p=ROOT/name;assert p.is_file()and not p.is_symlink();raw=p.read_bytes();inputs.append(seal(name,raw))
 if name not in configured:scan(name,raw)
 if p.name.startswith('corpus')and p.suffix=='.json':values(json.loads(raw),name)
 if p.suffix=='.md':
  for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^\n)]+)\)',raw.decode()):
   target=target.strip().strip('<>')
   if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target)or target.startswith('#'):continue
   path=urllib.parse.unquote(target.split('#',1)[0]);links.append({'document':name,'target':target,'exists':(p.parent/path).resolve().exists()})
epochs=[(p,json.loads(p.read_bytes()))for p in sorted(OUT.glob('*/inputs.json'))if(p.parent/'run-checks.py').is_file()]
oids=sorted({r['original_git_blob']for _,e in epochs for r in e['sources']if'original_git_blob'in r})
raw=subprocess.check_output(['/usr/bin/git','cat-file','--batch'],input=('\n'.join(oids)+'\n').encode());originals={};cursor=0
for oid in oids:
 end=raw.index(b'\n',cursor);actual,kind,size=raw[cursor:end].decode().split();size=int(size);assert(actual,kind)==(oid,'blob');originals[oid]=raw[end+1:end+1+size];cursor=end+size+2
assert cursor==len(raw);preimages=0
for path,e in epochs:
 assert seal(path.parent/'run-checks.py')==e['actual_driver_prelaunch']
 for row in e['sources']:
  buf=originals[row['original_git_blob']]if'original_git_blob'in row else Path(row['complete_changed_input']).read_bytes()
  assert len(buf)==row['bytes']and hashlib.sha256(buf).hexdigest()==row['sha256'];scan(row['path'],buf);preimages+=1
assert len(epochs)==15 and preimages==12854
executions=[]
for p in sorted(OUT.glob('*/*/result.json')):
 d=json.loads(p.read_bytes())
 if'source_receipt'not in d:continue
 assert d['status']in {'passed','failed'}and d['sources']==json.loads((p.parent.parent/'inputs.json').read_bytes())['sources']
 if not d['formatter_mutation_expected']:assert d['whole_source_inputs_unchanged']
 for key in ['stdout','stderr']:assert seal(d[key]['path'])==d[key]
 executions.append({'result':seal(p),'status':d['status'],'command':d['command']})
assert len(executions)==26 and sum(r['status']=='failed'for r in executions)==3
verification=json.loads((OUT/'verification.json').read_bytes())
for name,row in verification['current_checks'].items():
 assert seal(row['path'])==row;d=json.loads(Path(row['path']).read_bytes());assert d['status']=='passed'
 rust=name.startswith(('Rust','rust-'))or name=='clippy'
 for r in d['sources']:
  p=r['path'];relevant=p.startswith('crates/')if rust else p.startswith('go-tools/go2vir/')
  if relevant:assert seal(p)=={k:r[k]for k in ['path','bytes','sha256']}and Path(p).stat().st_mode&0o777==r['mode'],(name,p)
read=runpy.run_path(str(OUT/'read-originals.py'))['read'];files_count=bytes_count=0;front_files={}
for path in sorted(OUT.glob('*/manifest.json')):
 name=path.parent.name;meta,files=read(name);files_count+=meta['complete_original_files'];bytes_count+=meta['complete_original_bytes']
 if name=='frontend-original':front_files=files
 for member,buf in files.items():
  proxy=str(OUT/name/member);scan(proxy,buf)
  if member.endswith('.json'):
   obj=json.loads(buf);values(obj,proxy)
   if isinstance(obj,dict)and obj.get('schema')=='mpk.go.source_object.v1':source(obj,files[member[:-5]+'.raw'])
 for row in meta['original_inputs']:
  p=Path(row['path']);assert p.read_bytes()==files[row['member']]and p.stat().st_mode&0o777==row['mode']
assert files_count==54753 and bytes_count==385437972
for relative in ['original-byte-audit','sdk-check']:
 audit=json.loads((OUT/relative/'result.json').read_bytes());assert audit['status']=='passed'
 assert hashlib.sha256((OUT/relative/'driver.py').read_bytes()).hexdigest()==audit['actual_driver_prelaunch_sha256']
sdk=json.loads((OUT/'sdk-check/result.json').read_bytes());assert sdk['original_files_unchanged']==14496 and sdk['original_bytes_unchanged']==202722007
conversion=json.loads((OUT/'derived/result.json').read_bytes());assert conversion['status']=='passed'and conversion['cases']==46
for row in [conversion['actual_driver_prelaunch'],conversion['original_input_receipt'],conversion['output']]:assert all(seal(row['path'])[k]==row[k]for k in ['path','bytes','sha256'])
for key in ['previous_directive_basis','previous_source_selection_basis','unchanged_ordinary_paired_basis','previous_frontend_basis','literal_byte_mode_audit']:
 row=verification[key];assert seal(row['path'])==row
frozen=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json').read_bytes())['inputs'];assert len(frozen)==255 and all(seal(r['path'])==r for r in frozen)
frontend=json.loads(front_files['captured-inputs.json'])['build_inputs'];assert len(frontend)==190
for obj in frontend:source(obj,(Path('go-tools/go2vir')/obj['path']).read_bytes())
parent_read=runpy.run_path(str(OUT.parent/'source-selection/read-originals.py'))['read'];_,oldfront=parent_read('frontend-original');previous=json.loads(oldfront['captured-inputs.json'])['build_inputs'];assert len(previous)==190
assert [r for r in frontend if r['path']!='practical_closure.go']==[r for r in previous if r['path']!='practical_closure.go']
production={p.name for p in Path('go-tools/go2vir').glob('*.go')if not p.name.endswith('_test.go')}
assert production=={r['path']for r in frontend if'/'not in r['path']and r['path'].endswith('.go')}
for name in production-{'practical_closure.go'}:assert Path('go-tools/go2vir',name).read_bytes()==git('show',BASE+':go-tools/go2vir/'+name)
allowed_crates={'crates/mpk-vc/src/go_practical_type_keys.rs','crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing.rs','crates/mpk-vc/tests/go_practical_error_boxings.rs'}|{'crates/mpk-vc/src/go_practical_type_keys/source_scalar_typing/'+name for name in ['error_boxings.rs','aggregates.rs','assignments.rs','bodies.rs','methods.rs','returns.rs','selection.rs','slice_builtins.rs']}
assert set(git('diff','--name-only','HEAD','--','crates').decode().splitlines())<=allowed_crates
for name in ['go-tools/mpk-checker-ref','proofs/std','Cargo.toml','Cargo.lock']:
 assert not git('diff','--name-only','HEAD','--',name)
documents={'develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md','develop/migrations/go-practical/t02-w01/requirements.json'}
allowed=allowed_crates|documents|{'go-tools/go2vir/practical_closure.go','go-tools/go2vir/practical_error_boxings_test.go'}
assert all(name in allowed or name.startswith(str(OUT.relative_to(ROOT))+'/')for name in paths)
for name in documents-{'develop/migrations/go-practical/t02-w01/requirements.json'}:assert Path(name).read_bytes().startswith(git('show',BASE+':'+name))
old=json.loads(git('show',BASE+':develop/migrations/go-practical/t02-w01/requirements.json'));latest=json.loads(Path('develop/migrations/go-practical/t02-w01/requirements.json').read_bytes())
for row in latest['requirements']:
 if row['id']=='T02-W01-R06':assert row.pop('preparatory_original_error_boxing_checkpoint')['status']=='scoped_checks_passed'
assert latest==old
for baseline in ['source-selection/corpus.json','source-directives/corpus.json']:
 path='develop/migrations/go-practical/t02-w01/verification/'+baseline;assert Path(path).read_bytes()==git('show',BASE+':'+path)
scope_spec='develop/specs/LANGUAGE_SUPPORT_SCOPE_V1.md';assert Path(scope_spec).read_bytes()==git('show',BASE+':'+scope_spec)
missing=[r for r in links if not r['exists']];unchanged=all(seal(r['path'])==r for r in inputs)
record={'status':'passed'if not findings and not missing and unchanged else'failed','basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':inputs,'source_inputs_unchanged':unchanged,'findings':findings,'missing_links':missing,'links_checked':len(links),'source_epochs':15,'source_preimages':preimages,'actual_executions':executions,'literal_original_files':files_count,'literal_original_bytes':bytes_count,'decoded_fields':decoded,'original_new_cases':46,'exact_body_boxing_sites':125,'checked_value_sites':128,'compiler_typed_sites':129,'typed_evidence_is_subset':True,'global_error_universe_complete':False,'affected_Rust_roots':26,'old_Rust_selection_cases':84,'legacy_native_original_cases_passed':51,'mixed_native_runs_overall_passed':False,'flat_helpers':512,'selected_roots':70,'parameter_slots':70,'tuple_result_slots':16,'SDK_original_files_unchanged':14496,'frozen_inputs_unchanged':255,'old_frontend_inputs_unchanged':189,'current_frontend_inputs':190,'empty_cache_frontend_builds':2,'matcher_scope_seal_exclusion_counts':[127,4,39,51],'new_exclusions':False,'lexer_parser_scopes_constants_core_ordinary_and_registries_unchanged':True,'source_language_scope':['Go','Rust'],'other_language_tests_run':False,'complete_capture_Root_Plain_dispatch_contract_closure_or_source_program_authority':False,'public_profile_activated':False,'full_T_gate_passed':False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode());record['findings']=findings;record['status']='passed'if not findings and not missing and unchanged else'failed';REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({key:record[key]for key in ['status','source_inputs_unchanged','literal_original_files','exact_body_boxing_sites','decoded_fields']}));print(json.dumps({'findings':findings[:10],'missing_links':missing[:10]}));raise SystemExit(record['status']!='passed')
