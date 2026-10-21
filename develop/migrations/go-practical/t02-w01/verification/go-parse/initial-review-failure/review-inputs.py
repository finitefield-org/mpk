"""Unchanged-rule scan of full current inputs, exact source epochs and outputs."""
from pathlib import Path
import datetime,hashlib,json,re,runpy,subprocess,sys,types,urllib.parse
ROOT=Path.cwd();OUT=Path(__file__).resolve().parent;BASE='19bcd2a482eca7839f00a4ec474ca03346eff135';REPORT=OUT/'input-review.json'
def git(*args):return subprocess.check_output(['/usr/bin/git',*args])
def seal(p,raw=None):
 raw=Path(p).read_bytes()if raw is None else raw;return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
assert git('rev-parse','HEAD').decode().strip()==BASE and not Path('.github/workflows').exists()
scanner=Path('scripts/check-no-active-gir.sh');code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0];module=types.ModuleType('mpk_go_parse_input_review');module.__file__=str(ROOT/scanner);sys.modules[module.__name__]=module;sys.argv=[str(scanner),str(ROOT),'strict'];exec(compile(code,str(scanner),'exec'),module.__dict__)
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
def node_literals(record,name):
 for node in record.get('nodes',[]):
  attrs=node['attrs'];encoded=attrs.get('ValueHex')
  if encoded is not None:
   raw=bytes.fromhex(encoded);assert raw.hex()==encoded
   if raw:literals.setdefault(raw,name+f"#literal-at-{node['start']}")

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
 d=json.loads(p.read_bytes());assert d['status']in {'passed','failed'}and d['sources']==json.loads((p.parent.parent/'inputs.json').read_bytes())['sources']
 if not d['formatter_mutation_expected']:assert d['whole_source_inputs_unchanged']
 assert d['whole_external_inputs_unchanged']
 for key in ['stdout','stderr']:assert seal(d[key]['path'])==d[key]
 executions.append({'result':seal(p),'command':d['command'],'status':d['status']})
assert len(executions)==20 and len(archives)==12 and preimages==8577
for name,row in verification['current_checks'].items():
 assert seal(row['path'])==row;d=json.loads(Path(row['path']).read_bytes());assert d['status']=='passed'
 prefix='go-tools/go2vir/'if name in {'native-original','native-depth','vet','Go-format'}else'crates/'
 for r in d['sources']:
  if not r['path'].startswith(prefix):continue
  if name=='native-original'and r['path'].endswith('practical_parse_reference_test.go'):
   old=Path(r['complete_changed_input']).read_bytes().rstrip()+b'\n';current=Path(r['path']).read_bytes().split(b'// Additional native parser-depth oracle;',1)[0].rstrip()+b'\n';assert old==current
  elif name=='Rust-original'and r['path'].endswith('go_practical_parse.rs'):
   old=Path(r['complete_changed_input']).read_bytes().rstrip()+b'\n';current=Path(r['path']).read_bytes().split(b'#[test]\nfn deep_syntax_uses_the_pinned_parser_domain',1)[0].rstrip()+b'\n';assert old==current
  else:assert seal(r['path'])=={k:r[k]for k in ['path','bytes','sha256']}
reader=runpy.run_path(str(OUT/'read-native.py'))['read'];native_meta,native=reader('native-original');depth_meta,depth=reader('native-depth');rust_meta,rust=runpy.run_path(str(OUT/'read-rust.py'))['read']();source_objects=0;decoded_sources=0
for name,files in [('native-original',native),('native-depth',depth),('rust-originals',rust)]:
 for member,raw in files.items():
  proxy=str(OUT/name/member);scan(proxy,raw)
  if member.endswith('.json'):
   data=json.loads(raw);node_literals(data,proxy)
   if 'raw_hex'in data:
    buf=bytes.fromhex(data['raw_hex']);source(data['object'],buf);scan(proxy+'#whole-source',buf);decoded_sources+=1
 for member,raw in files.items():
  if member.endswith('/source.json'):
   obj=json.loads(raw);source(obj,files[member[:-4]+'raw']);source_objects+=1
assert source_objects==2996
for n in range(2996):
 key=f'case-{n:05d}';expected=json.loads(native[key+'/native.json']);actual=json.loads(rust['comm-colons-paired/'+key+'.json']);assert actual['accepted']==expected['accepted']
 if actual['accepted']:assert actual['nodes']==expected['nodes']and actual['root']==expected['root']
 assert actual['source_typing_or_program_authority']is False
for n in range(20):assert rust[f'first-paired/case-{n:05d}.json']
smoke=json.loads((OUT/'smoke.json').read_bytes());assert len(smoke)==62
for n,case in enumerate(smoke):
 raw=bytes.fromhex(case['raw_hex']);source(case['object'],raw);scan(str(OUT/'smoke.json')+f'#source-{n}',raw);source_objects+=1
 expected=case['native'];actual=json.loads(rust[f'depth-smoke/smoke-{n:04d}.json']);assert expected['accepted']==actual['accepted']
 if actual['accepted']:assert expected['nodes']==actual['nodes']
 assert rust[f'comm-colons-smoke/smoke-{n:04d}.json']==rust[f'depth-smoke/smoke-{n:04d}.json']
 node_literals(expected,str(OUT/'smoke.json')+f'#case-{n}')
references=json.loads((OUT/'depth-native.json').read_bytes())
for n in [0,1000,20000,100000]:
 key=f'depth-{n:06d}';obj=json.loads(depth[key+'-source.json']);source(obj,depth[key+'.raw']);source_objects+=1;expected=json.loads(depth[key+'-native.json']);actual=json.loads(rust['depth-smoke/'+key+'-result.json']);assert expected==references[str(n)]and actual['accepted']==expected['accepted']and actual['source_id']==expected['source_id']
for raw,name in literals.items():scan(name,raw)
sdk=json.loads((OUT/'sdk-check/result.json').read_bytes());assert sdk['status']=='passed'and sdk['original_files_unchanged']==14496
assert hashlib.sha256((OUT/'sdk-check/driver.py').read_bytes()).hexdigest()==sdk['actual_driver_prelaunch_sha256']
audit=json.loads((OUT/'original-byte-audit/result.json').read_bytes());assert audit['status']=='passed'and audit['actual_original_files_compared_byte_for_byte']==12165 and audit['actual_original_bytes_compared']==1268912231
frozen=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/native-observation/frozen-design-preservation.json').read_bytes())['inputs'];assert len(frozen)==255 and all(seal(r['path'])==r for r in frozen)
frontend=json.loads(Path('develop/migrations/go-practical/t02-w01/verification/type-keys/corrected/analysis-inputs/captured-inputs.json').read_bytes())['build_inputs'];assert len(frontend)==189
for obj in frontend:source(obj,(Path('go-tools/go2vir')/obj['path']).read_bytes())
for name in ['crates/mpk-vc/src/go_practical_ordinary','crates/mpk-vc/src/go_practical_ordinary.rs','crates/mpk-core','crates/mpk-kernel','go-tools/mpk-checker-ref','proofs/std']:
 assert not git('diff','--name-only','HEAD','--',name)
for name in ['develop/docs/go-practical-implementation-traceability-ledger.md','develop/migrations/go-practical/t02-w01/README.md']:assert Path(name).read_bytes().startswith(git('show',BASE+':'+name))
old=json.loads(git('show',BASE+':develop/migrations/go-practical/t02-w01/requirements.json'));latest=json.loads(Path('develop/migrations/go-practical/t02-w01/requirements.json').read_bytes())
for row in latest['requirements']:
 if row['id']in {'T02-W01-R01','T02-W01-R06'}:assert row.pop('preparatory_independent_syntax_checkpoint')['status']=='scoped_checks_passed'
assert latest==old
missing=[r for r in links if not r['exists']];unchanged=all(seal(r['path'])==r for r in inputs)
record={'status':'passed'if not findings and not missing and unchanged else'failed','basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':inputs,'source_inputs_unchanged':unchanged,'findings':findings,'missing_links':missing,'links_checked':len(links),'source_epochs':12,'source_preimages':preimages,'actual_executions':executions,'original_files':native_meta['complete_original_files']+depth_meta['complete_original_files']+rust_meta['complete_original_files'],'original_bytes':native_meta['complete_original_bytes']+depth_meta['complete_original_bytes']+rust_meta['complete_original_bytes'],'unique_Rust_original_byte_objects':rust_meta['distinct_original_byte_blobs'],'verified_source_objects':source_objects,'decoded_Rust_source_inputs':decoded_sources,'distinct_decoded_AST_literal_values':len(literals),'all_full_native_accepted_AST_graphs_exact':True,'all_native_rejections_agree':True,'native_depth_domain_checked':True,'frozen_inputs_unchanged':255,'frontend_build_inputs_unchanged':189,'matcher_scope_seal_exclusion_counts':[127,4,39,51],'new_exclusions':False,'source_parsing_typing_or_program_authority':False,'public_profile_activated':False,'full_T_gate_passed':False}
scan(str(REPORT.relative_to(ROOT)),json.dumps(record,indent=2).encode());record['findings']=findings;record['status']='passed'if not findings and not missing and unchanged else'failed';REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','source_inputs_unchanged','source_preimages','original_files','original_bytes','verified_source_objects','distinct_decoded_AST_literal_values']}));print(json.dumps({'findings':findings[:10],'missing_links':missing[:10]}));raise SystemExit(record['status']!='passed')
