"""Review full changed inputs and original archives with the unchanged scanner."""
from pathlib import Path
import base64,datetime,hashlib,json,re,runpy,subprocess,sys,types,urllib.parse
ROOT=Path.cwd();OUT=Path(__file__).resolve().parent;BASE='747e208ae9dc2d4735baa58747d2f98f42a8ad5a';REPORT=OUT/'input-review.json'
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
historical_manifest=json.loads((OUT/'historical-preimages/manifest.json').read_bytes())
transports={row['original_path']:row for row in historical_manifest['originals']}
def read_bytes(path):
 path=Path(path)
 if path.exists():return path.read_bytes()
 row=transports[str(path)];raw=bytes.fromhex(Path(row['transport']).read_text());assert len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['sha256'];return raw
def seal(path,raw=None):
 raw=read_bytes(path)if raw is None else raw;return {'path':str(path),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
assert git('rev-parse','HEAD').decode().strip()==BASE and not Path('.github/workflows').exists()
scanner=Path('scripts/check-no-active-gir.sh');code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0];module=types.ModuleType('mpk_type_key_review');module.__file__=str(ROOT/scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(ROOT),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
matchers=module.load_matchers();manifest,scanner_files,fixtures,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers);exclusions=module.active_retained_record_exclusions(manifest)
assert(len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(exclusions))==(127,4,39,51)
for name in scanner_files|fixtures:assert Path(name).read_bytes()==git('show',BASE+':'+name)
configured=scanner_files|fixtures|exclusions;findings=[];historical_findings=[];cache={};path_rules=[m for m in matchers if m['scope']=='exact-path'];links=[];inputs=[];payloads=[]
def scan(name,raw,historical=False):
 key=(hashlib.sha256(raw).hexdigest(),tuple(module.path_matches(name,m['path'])for m in path_rules))
 if key not in cache:cache[key]=[vars(f)for f in module.detect(name,raw.decode(errors='ignore'),matchers)]
 target=historical_findings if historical else findings
 target.extend({**f,'path':name}for f in cache[key])
def payload(name,data):
 assert data is None or isinstance(data,str);raw=b''if data is None else base64.b64decode(data,validate=True);scan(name,raw,True);payloads.append(seal(name,raw));return raw
def object_id(obj,raw):
 assert obj['schema']=='mpk.go.source_object.v1'and len(raw)==obj['bytes']and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
 identity(obj,'MPK-GO-SOURCE-OBJECT-1.0','id')
def identity(obj,domain,field):
 body={k:v for k,v in obj.items()if k!=field};pre=json.dumps(body,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode();assert hashlib.sha256(domain.encode()+b'\0'+pre).hexdigest()==obj[field]
def nested(data,name):
 if isinstance(data,dict):
  for key,value in data.items():
   if key in {'stdout','stderr','raw','original_expression','whole_original_source','original_source','canonical_components'}and(isinstance(value,str)or value is None):payload(name+'#'+key,value)
   elif key=='whole_original_bytes':
    assert isinstance(value,list)and all(type(v)is int and 0<=v<=255 for v in value);buf=bytes(value);scan(name+'#'+key,buf,True);payloads.append(seal(name+'#'+key,buf))
   elif key=='whole_original_sources':
    for file,raw in value.items():payload(name+'#'+file,raw)
   else:nested(value,name+'/'+key)
 elif isinstance(data,list):
  for n,value in enumerate(data):nested(value,name+f'/{n}')
 elif isinstance(data,str):scan(name,data.encode(),True)
if not REPORT.exists():REPORT.write_text(json.dumps({'status':'running','basis_commit':BASE})+'\n')
selected=git('diff','--name-only','-z','HEAD','--')+git('ls-files','--others','--exclude-standard','-z');paths=sorted(set(selected.decode().split('\0'))-{'',str(REPORT.relative_to(ROOT))})
for name in paths:
 path=ROOT/name;assert path.is_file()and not path.is_symlink();raw=path.read_bytes();inputs.append(seal(name,raw))
 if name not in configured:scan(name,raw)
 if path.suffix=='.md':
  for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^\n)]+)\)',raw.decode()):
   target=target.strip().strip('<>')
   if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target)or target.startswith('#'):continue
   clean=urllib.parse.unquote(target.split('#',1)[0]);links.append({'document':name,'target':target,'exists':(path.parent/clean).resolve().exists()})
archives=[json.loads(p.read_bytes())for p in sorted(OUT.glob('*/inputs.json'))];oids=sorted({r['original_git_blob']for a in archives for r in a['sources']if'original_git_blob'in r});raw=subprocess.check_output(['/usr/bin/git','cat-file','--batch'],input=('\n'.join(oids)+'\n').encode());originals={};cursor=0
for oid in oids:
 end=raw.index(b'\n',cursor);actual,kind,size=raw[cursor:end].decode().split();size=int(size);assert(actual,kind)==(oid,'blob');originals[oid]=raw[end+1:end+1+size];cursor=end+size+2
assert cursor==len(raw);preimages=0
for path,a in zip(sorted(OUT.glob('*/inputs.json')),archives):
 assert seal(path.parent/'run-checks.py')==a['actual_driver_prelaunch']
 for row in a['sources']:
  buf=originals[row['original_git_blob']]if'original_git_blob'in row else read_bytes(row['complete_changed_input']);assert len(buf)==row['bytes']and hashlib.sha256(buf).hexdigest()==row['sha256'];scan(row['path'],buf,True);preimages+=1
executions=[]
for path in sorted(OUT.glob('*/*/result.json')):
 d=json.loads(path.read_bytes());assert d['status']in {'passed','failed'}and d['sources']==json.loads((path.parent.parent/'inputs.json').read_bytes())['sources']
 for key in ['stdout','stderr']:assert seal(d[key]['path'])==d[key]
 if not d['formatter_mutation_expected']:assert d['whole_source_inputs_unchanged']
 executions.append({'result':seal(path),'status':d['status'],'command':d['command']})
for row in transports.values():
 buf=bytes.fromhex(Path(row['transport']).read_text());assert len(buf)==row['bytes']and hashlib.sha256(buf).hexdigest()==row['sha256'];scan(row['original_path'],buf,True)
verification=json.loads((OUT/'verification.json').read_bytes());assert len(executions)==len(verification['actual_check_executions'])and len(archives)==verification['source_epochs']and preimages==verification['source_preimages']
for name,row in verification['current_checks'].items():
 assert seal(row['path'])==row;d=json.loads(Path(row['path']).read_bytes());assert d['status']=='passed'and d['whole_source_inputs_unchanged']
 prefix='crates/'if name in {'rust-selected','clippy','rust-format'}else'go-tools/go2vir/'
 for r in d['sources']:
  if r['path'].startswith(prefix):assert seal(r['path'])=={k:r[k]for k in ['path','bytes','sha256']}
read_native=runpy.run_path(str(OUT/'read-native.py'))['read'];native_files=native_bytes=source_objects=0;native_counts={};retained_Rust_records={}
for archive in verification['complete_native_archives']:
 name=Path(archive['path']).parent.name;metadata,files=read_native(name);native_counts[name]=len(files);native_files+=len(files);native_bytes+=sum(map(len,files.values()))
 for member,buf in files.items():
  proxy=str(OUT/name/member);scan(proxy,buf,True)
  if member.endswith('.json'):nested(json.loads(buf),proxy)
  if member.startswith('original-source-objects/')and member.endswith('.json'):
   obj=json.loads(buf);object_id(obj,files[member[:-5]+'.raw']);source_objects+=1
 if name=='native':
  for n in range(5):
   record=json.loads(files[f'type-keys-{n:02}.json']);wire=base64.b64decode(record['canonical_components'],validate=True);case=OUT/'native-components'/f'case-{n:02}';assert wire==(case/'components.json').read_bytes();obj=json.loads(wire);identity(obj,'MPK-GO-TYPE-COMPONENTS-2.0','content_sha256');assert not record['source_typing_or_registered_context_proof']
   for source in record['sources']:
    s=source['object'];raw=base64.b64decode(source['raw'],validate=True);object_id(s,raw);assert(case/'sources'/(s['id']+'.raw')).read_bytes()==raw and json.loads((case/'sources'/(s['id']+'.json')).read_bytes())==s
   for family,domain in [('packages','MPK-GO-TYPE-PACKAGE-2.0'),('declarations','MPK-GO-TYPE-DECLARATION-2.0'),('types','MPK-GO-SOURCE-TYPE-2.0')]:
    for node in obj[family]:identity(node,domain,'id')
 if name.startswith('rust-'):
  retained_Rust_records[name]=len([p for p in files if p.endswith('.json')]);assert all(files[p].endswith(b'\n')for p in files if p.endswith('.outcome.txt'))
frozen=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json').read_bytes())['inputs'];assert len(frozen)==255 and all(seal(r['path'])==r for r in frozen)
for path in ['crates/mpk-vc/src/go_practical_ordinary','go-tools/mpk-checker-ref','crates/mpk-kernel','crates/mpk-core']:
 assert not git('diff','--name-only','HEAD','--',path)
for name in ['develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md']:assert Path(name).read_bytes().startswith(git('show',BASE+':'+name))
old=json.loads(git('show',BASE+':develop/migrations/go-practical/t02-w01/requirements.json'));latest=json.loads(Path('develop/migrations/go-practical/t02-w01/requirements.json').read_bytes())
for row in latest['requirements']:
 if row['id']in {'T02-W01-R01','T02-W01-R06'}:assert row.pop('preparatory_original_type_key_checkpoint')['status']=='scoped_checks_passed'
assert latest==old
build=json.loads((OUT/'corrected/analysis-inputs/captured-inputs.json').read_bytes());assert len(build['build_inputs'])==189
for row in build['build_inputs']:object_id(row,(Path('go-tools/go2vir')/row['path']).read_bytes())
rebuild=json.loads((OUT/'corrected/analysis-inputs/reproduced-builds.json').read_bytes());assert rebuild['byte_exact_equal']and len(rebuild['builds'])==2
unicode=json.loads((OUT/'unicode-check/result.json').read_bytes());assert unicode['status']=='passed'and unicode['whole_inputs_unchanged']
for r in unicode['sources']:assert seal(r['complete_original'])['sha256']==r['sha256']
prelaunch=json.loads((OUT/'prelaunch-errors/manifest.json').read_bytes());assert prelaunch['observed_prelaunch_driver_failures']==4
for r in prelaunch['original_records']:
 buf=Path(r['path']).read_bytes();assert len(buf)==r['bytes']and hashlib.sha256(buf).hexdigest()==r['sha256'];record=json.loads(buf);assert 'output'in record['payload']and b'AssertionError'in buf
assert all(f['matcher_id']=='helper-kind-go-source'for f in historical_findings)
missing=[r for r in links if not r['exists']];unchanged=all(seal(r['path'])==r for r in inputs)
record={'status':'passed'if not findings and not missing and unchanged else'failed','basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':inputs,'source_inputs_unchanged':unchanged,'findings':findings,'missing_links':missing,'links_checked':len(links),'source_epochs':len(archives),'verified_source_preimages':preimages,'actual_executions':executions,'native_files':native_files,'native_bytes':native_bytes,'verified_original_source_objects':source_objects,'archive_counts':native_counts,'Rust_retained_JSON_input_records':retained_Rust_records,'decoded_data_payloads':payloads,'original_scan_findings':[dict((k,v)for k,v in f.items()if k!='excerpt')|{'excerpt_utf8_hex':f['excerpt'].encode().hex()}for f in historical_findings],'original_scan_findings_count':len(historical_findings),'original_findings_are_not_suppressed_or_added_to_scanner_exclusions':True,'frozen_inputs_unchanged':255,'frontend_build_inputs':189,'matcher_scope_seal_exclusion_counts':[127,4,39,51],'new_exclusions':False,'candidate_components_only':True,'source_typing_or_registered_context_or_program_proof':False,'public_profile_activated':False,'full_T_gate_passed':False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode());record['findings']=findings;record['status']=('completed_with_original_findings'if historical_findings else'passed')if not findings and not missing and unchanged else'failed';REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','source_inputs_unchanged','source_epochs','verified_source_preimages','native_files','native_bytes','verified_original_source_objects']}));print(json.dumps({'findings':findings[:10],'missing_links':missing[:10]}));raise SystemExit(record['status']not in {'passed','completed_with_original_findings'})
