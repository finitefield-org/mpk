"""Unchanged-rule scan of full current inputs, exact source epochs and outputs."""
from pathlib import Path
import datetime,hashlib,json,re,runpy,subprocess,sys,types,urllib.parse
ROOT=Path.cwd();OUT=Path(__file__).resolve().parent;BASE='13edcaf13d267febe501e5b7b66135d20915fbc8';REPORT=OUT/'input-review.json'
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
def seal(p,raw=None):
 raw=Path(p).read_bytes()if raw is None else raw;return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
assert git('rev-parse','HEAD').decode().strip()==BASE and not Path('.github/workflows').exists()
scanner=Path('scripts/check-no-active-gir.sh');code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0];module=types.ModuleType('mpk_go_lex_input_review');module.__file__=str(ROOT/scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(ROOT),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
matchers=module.load_matchers();manifest,scanner_files,fixtures,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers);exclusions=module.active_retained_record_exclusions(manifest);assert(len(matchers),len(module.SCOPES),len(manifest['sealed_records']),len(exclusions))==(127,4,39,51)
for name in scanner_files|fixtures:assert Path(name).read_bytes()==git('show',BASE+':'+name)
configured=scanner_files|fixtures|exclusions;findings=[];inputs=[];links=[];cache={};path_rules=[m for m in matchers if m['scope']=='exact-path'];literals={}
def scan(name,raw):
 key=(hashlib.sha256(raw).hexdigest(),tuple(module.path_matches(name,m['path'])for m in path_rules))
 if key not in cache:cache[key]=[vars(f)for f in module.detect(name,raw.decode(errors='ignore'),matchers)]
 findings.extend({**f,'path':name}for f in cache[key])
def identity(obj,domain,field):
 body={k:v for k,v in obj.items()if k!=field};raw=json.dumps(body,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode();assert hashlib.sha256(domain.encode()+b'\0'+raw).hexdigest()==obj[field]
def source(obj,raw):
 assert obj['schema']=='mpk.go.source_object.v1'and len(raw)==obj['bytes']and hashlib.sha256(raw).hexdigest()==obj['raw_sha256'];identity(obj,'MPK-GO-SOURCE-OBJECT-1.0','id')
def token_literals(record,name):
 for offset,kind,encoded in record.get('tokens',[])if isinstance(record.get('tokens'),list)else[]:
  assert type(offset)is int and isinstance(kind,str)and isinstance(encoded,str);raw=bytes.fromhex(encoded);assert raw.hex()==encoded
  if raw:literals.setdefault(raw,name+f'#literal-at-{offset}')
if not REPORT.exists():REPORT.write_text(json.dumps({'status':'running','basis_commit':BASE})+'\n')
selected=git('diff','--name-only','-z','HEAD','--')+git('ls-files','--others','--exclude-standard','-z');paths=sorted(set(selected.decode().split('\0'))-{'',str(REPORT.relative_to(ROOT))})
for name in paths:
 p=ROOT/name;assert p.is_file()and not p.is_symlink();raw=p.read_bytes();inputs.append(seal(name,raw))
 if name not in configured:scan(name,raw)
 if p.suffix=='.md':
  for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^\n)]+)\)',raw.decode()):
   target=target.strip().strip('<>')
   if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target)or target.startswith('#'):continue
   path=urllib.parse.unquote(target.split('#',1)[0]);links.append({'document':name,'target':target,'exists':(p.parent/path).resolve().exists()})
archives=[json.loads(p.read_bytes())for p in sorted(OUT.glob('*/inputs.json'))];oids=sorted({r['original_git_blob']for a in archives for r in a['sources']if'original_git_blob'in r});raw=subprocess.check_output(['/usr/bin/git','cat-file','--batch'],input=('\n'.join(oids)+'\n').encode());originals={};cursor=0
for oid in oids:
 end=raw.index(b'\n',cursor);actual,kind,size=raw[cursor:end].decode().split();size=int(size);assert(actual,kind)==(oid,'blob');originals[oid]=raw[end+1:end+1+size];cursor=end+size+2
assert cursor==len(raw);preimages=0
for path,a in zip(sorted(OUT.glob('*/inputs.json')),archives):
 assert seal(path.parent/'run-checks.py')==a['actual_driver_prelaunch']
 for row in a['sources']:
  buf=originals[row['original_git_blob']]if'original_git_blob'in row else Path(row['complete_changed_input']).read_bytes();assert len(buf)==row['bytes']and hashlib.sha256(buf).hexdigest()==row['sha256'];scan(row['path'],buf);preimages+=1
verification=json.loads((OUT/'verification.json').read_bytes());executions=[]
for p in sorted(OUT.glob('*/*/result.json')):
 d=json.loads(p.read_bytes());assert d['status']=='passed'and d['sources']==json.loads((p.parent.parent/'inputs.json').read_bytes())['sources']
 if not d['formatter_mutation_expected']:assert d['whole_source_inputs_unchanged']
 assert d['whole_external_inputs_unchanged']
 for key in ['stdout','stderr']:assert seal(d[key]['path'])==d[key]
 executions.append({'result':seal(p),'command':d['command'],'status':d['status']})
assert len(executions)==15 and len(archives)==8 and preimages==5662
for name,row in verification['current_checks'].items():
 assert seal(row['path'])==row;d=json.loads(Path(row['path']).read_bytes());assert d['status']=='passed'
 prefix='go-tools/go2vir/'if name in {'native-Go-original','native-byte-bound','vet','Go-format'}else'crates/'
 for r in d['sources']:
  if not r['path'].startswith(prefix):continue
  if name=='native-Go-original'and r['path'].endswith('practical_lex_reference_test.go'):
   assert Path(r['complete_changed_input']).read_bytes().split(b'func TestPracticalLexOriginalGoReference(t *testing.T) {',1)[1]==Path(r['path']).read_bytes().split(b'func TestPracticalLexOriginalGoReference(t *testing.T) {',1)[1]
  else:assert seal(r['path'])=={k:r[k]for k in ['path','bytes','sha256']}
reader=runpy.run_path(str(OUT/'read-native.py'))['read'];native_meta,native=reader('native-original');bound_meta,bound=reader('native-byte-bound');rust_meta,rust=runpy.run_path(str(OUT/'read-rust.py'))['read']();source_objects=0;decoded_sources=0
for name,files in [('native-original',native),('native-byte-bound',bound),('rust-originals',rust)]:
 for member,raw in files.items():
  proxy=str(OUT/name/member);scan(proxy,raw)
  if member.endswith('.json'):
   data=json.loads(raw);token_literals(data,proxy)
   if 'raw_hex'in data:
    buf=bytes.fromhex(data['raw_hex']);source(data['object'],buf);scan(proxy+'#whole-source',buf);decoded_sources+=1
 for member,raw in files.items():
  if member.endswith('/source.json'):
   obj=json.loads(raw);source(obj,files[member[:-4]+'raw']);source_objects+=1
assert source_objects==5853
for n in range(5853):
 key=f'case-{n:05d}';expected=json.loads(native[key+'/native.json']);actual=json.loads(rust['byte-bound-paired/'+key+'.json']);assert actual['accepted']==expected['accepted']
 if actual['accepted']:assert actual['tokens']==expected['tokens']
 assert rust['original-paired/'+key+'.json']==rust['byte-bound-paired/'+key+'.json']
 assert actual['source_typing_or_program_authority']is False
smoke=json.loads((OUT/'smoke.json').read_bytes());assert len(smoke)==2919
for n,case in enumerate(smoke):
 raw=bytes.fromhex(case['raw_hex']);source(case['object'],raw);scan(str(OUT/'smoke.json')+f'#source-{n}',raw);source_objects+=1
 expected=case['native'];actual=json.loads(rust[f'byte-bound-smoke/smoke-{n:05d}.json']);assert expected['accepted']==actual['accepted']
 if actual['accepted']:assert expected['tokens']==actual['tokens']
 assert rust[f'original-smoke/smoke-{n:05d}.json']==rust[f'byte-bound-smoke/smoke-{n:05d}.json']
 token_literals(expected,str(OUT/'smoke.json')+f'#case-{n}')
source(json.loads(bound['byte-bound-source.json']),bound['byte-bound.raw']);assert json.loads(bound['byte-bound-native.json'])==json.loads(rust['byte-bound-smoke/byte-bound-result.json'])==json.loads((OUT/'byte-bound-native.json').read_bytes());source_objects+=1
for raw,name in literals.items():scan(name,raw)
# Pin original input corpus and the unchanged whole compiler/stdlib image.
corpus=json.loads((OUT/'corpus-inputs.json').read_bytes());assert corpus['source_objects']==2934
for row in corpus['complete_inputs']:assert seal(row['original']['path'])==row['original']and seal(row['actual']['path'])==row['actual']
sdk=json.loads((OUT/'sdk-check/result.json').read_bytes());assert sdk['status']=='passed'and sdk['complete_files_unchanged']==14496
trace=sdk['actual_inline_driver_original_pre_execution_record'];assert seal(trace['path'])=={k:trace[k]for k in ['path','bytes','sha256']};assert 'input'in json.loads(Path(trace['path']).read_bytes())['payload']
frozen=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json').read_bytes())['inputs'];assert len(frozen)==255 and all(seal(r['path'])==r for r in frozen)
frontend=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/type-keys/corrected/analysis-inputs/captured-inputs.json').read_bytes())['build_inputs'];assert len(frontend)==189
for obj in frontend:source(obj,(Path('go-tools/go2vir')/obj['path']).read_bytes())
for name in ['crates/mpk-vc/src/go_practical_ordinary','crates/mpk-vc/src/go_practical_ordinary.rs','crates/mpk-core','crates/mpk-kernel','go-tools/mpk-checker-ref','proofs/std']:
 assert not git('diff','--name-only','HEAD','--',name)
for name in ['develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md']:assert Path(name).read_bytes().startswith(git('show',BASE+':'+name))
old=json.loads(git('show',BASE+':develop/migrations/go-practical/t02-w01/requirements.json'));latest=json.loads(Path('develop/migrations/go-practical/t02-w01/requirements.json').read_bytes())
for row in latest['requirements']:
 if row['id']in {'T02-W01-R01','T02-W01-R06'}:assert row.pop('preparatory_independent_lex_checkpoint')['status']=='scoped_checks_passed'
assert latest==old
missing=[r for r in links if not r['exists']];unchanged=all(seal(r['path'])==r for r in inputs)
record={'status':'passed'if not findings and not missing and unchanged else'failed','basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':inputs,'source_inputs_unchanged':unchanged,'findings':findings,'missing_links':missing,'links_checked':len(links),'source_epochs':8,'source_preimages':preimages,'actual_executions':executions,'original_files':native_meta['complete_original_files']+bound_meta['complete_original_files']+rust_meta['complete_original_files'],'original_bytes':native_meta['complete_original_bytes']+bound_meta['complete_original_bytes']+rust_meta['complete_original_bytes'],'unique_Rust_original_byte_objects':rust_meta['distinct_original_byte_blobs'],'verified_source_objects':source_objects,'decoded_Rust_source_inputs':decoded_sources,'distinct_decoded_token_literal_values':len(literals),'all_full_native_accepted_streams_exact':True,'all_native_rejections_agree':True,'full_byte_boundary_exact':True,'frozen_inputs_unchanged':255,'frontend_build_inputs_unchanged':189,'matcher_scope_seal_exclusion_counts':[127,4,39,51],'new_exclusions':False,'source_parsing_typing_or_program_authority':False,'public_profile_activated':False,'full_T_gate_passed':False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode());record['findings']=findings;record['status']='passed'if not findings and not missing and unchanged else'failed';REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','source_inputs_unchanged','source_preimages','original_files','original_bytes','verified_source_objects','distinct_decoded_token_literal_values']}));print(json.dumps({'findings':findings[:10],'missing_links':missing[:10]}));raise SystemExit(record['status']!='passed')
