"""Review changed files, deletions, links, and full original payloads; outside sandbox."""
from pathlib import Path
import base64,datetime,hashlib,json,re,subprocess,sys,types,urllib.parse
ROOT=Path.cwd();BASE='528b599aacfe439d4abd8f50e046a754c9a24435'
OUT=Path(__file__).resolve().parent
MODE=sys.argv[1] if len(sys.argv)>1 else 'prestage'
assert MODE in {'prestage','staged'}
REPORT=OUT/(MODE+'-review.json')
def git(*args):
 return subprocess.run(['/usr/bin/git',*args],capture_output=True,check=True).stdout
def seal(name,raw):return {'path':name,'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw)}
assert git('rev-parse','HEAD').decode().strip()==BASE
if MODE=='staged':
 selected=git('diff','--cached','--name-only','-z','HEAD','--')
 remaining=git('diff','--name-only','-z')+git('ls-files','--others','--exclude-standard','-z')
 assert set(remaining.decode().split('\0'))-{'',str(REPORT.relative_to(ROOT))}==set()
else:selected=git('diff','--name-only','-z','HEAD','--')+git('ls-files','--others','--exclude-standard','-z')
paths=sorted(set(selected.decode().split('\0'))-{'',str(REPORT.relative_to(ROOT))})
scanner=Path('scripts/check-no-active-gir.sh')
code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
module=types.ModuleType('mpk_go_rust_scope_review');module.__file__=str(ROOT/scanner);sys.modules[module.__name__]=module
argv=sys.argv;sys.argv=[str(scanner),str(ROOT),'strict']
try:exec(compile(code,str(scanner),'exec'),module.__dict__);matchers=module.load_matchers()
finally:sys.argv=argv
manifest,scanner_files,fixtures,_=module.validate_manifest();module.run_fixture_self_test(manifest,matchers)
configured=scanner_files|fixtures|module.active_retained_record_exclusions(manifest)
assert (len(matchers),len(module.SCOPES),len(manifest['sealed_records']))==(127,4,39)
inputs=[];deletions=[];findings=[];links=[];decoded=[];historical=[];cache={};decoded_seen=set()
path_rules=[m for m in matchers if m['scope']=='exact-path']
def scan(name,raw):
 key=(hashlib.sha256(raw).hexdigest(),tuple(module.path_matches(name,m['path']) for m in path_rules))
 if key not in cache:cache[key]=[vars(f) for f in module.detect(name,raw.decode(errors='ignore'),matchers)]
 findings.extend({**f,'path':name} for f in cache[key])
def decode_all(name,node):
 if isinstance(node,dict):
  if isinstance(node.get('base64'),str):
   expected=node.get('raw_sha256',node.get('sha256'));size=node.get('bytes',node.get('size_bytes',node.get('utf8_length')))
   if expected is not None and size is not None:
    raw=base64.b64decode(node['base64'],validate=True)
    assert hashlib.sha256(raw).hexdigest()==expected and len(raw)==size,name
    owner=name+'#decoded-'+str(len(decoded));scan(owner,raw);decoded.append({**seal(owner,raw),'source_path':node.get('path',node.get('source_id'))})
    if expected not in decoded_seen:
     decoded_seen.add(expected)
     try:inner=json.loads(raw)
     except (ValueError,UnicodeError):pass
     else:decode_all(owner,inner)
  for key,value in node.items():
   if key!='base64':decode_all(name,value)
 elif isinstance(node,list):
  for value in node:decode_all(name,value)
for name in paths:
 p=ROOT/name
 if not p.exists():
  raw=git('show',BASE+':'+name);object_id=git('rev-parse',BASE+':'+name).decode().strip()
  deletions.append({**seal(name,raw),'basis_commit':BASE,'git_blob':object_id,'active':False});continue
 assert p.is_file() and not p.is_symlink(),name
 raw=p.read_bytes();inputs.append(seal(name,raw))
 if name in configured:historical.append(seal(name,raw))
 else:scan(name,raw)
 if p.suffix=='.json':
  try:node=json.loads(raw)
  except (ValueError,UnicodeError):pass
  else:decode_all(name,node)
 if p.suffix=='.md':
  for target in re.findall(r'(?<!!)\[[^\]]*\]\(([^\n)]+)\)',raw.decode()):
   target=target.strip().strip('<>')
   if re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target) or target.startswith('#'):continue
   clean=urllib.parse.unquote(target.split('#',1)[0]);links.append({'document':name,'target':target,'exists':(p.parent/clean).resolve().exists()})
check=subprocess.run(['/usr/bin/git','diff',*(['--cached'] if MODE=='staged' else []),'--check','HEAD','--'],capture_output=True)
missing=[row for row in links if not row['exists']]
unchanged=all((ROOT/row['path']).is_file() and hashlib.sha256((ROOT/row['path']).read_bytes()).hexdigest()==row['sha256'] for row in inputs)
result={'status':'passed' if not (findings or missing or check.returncode) and unchanged else 'failed','mode':MODE,'basis_commit':BASE,'observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'inputs':inputs,'deletions':deletions,'findings':[{k:v for k,v in finding.items() if k!='excerpt'} for finding in findings],'missing_links':missing,'links_checked':len(links),'decoded_complete_payloads':decoded,'decoded_complete_payload_count':len(decoded),'source_inputs_unchanged_during_review':unchanged,'whitespace_exit_code':check.returncode,'whitespace_stdout':check.stdout.decode(errors='replace'),'whitespace_stderr':check.stderr.decode(errors='replace'),'original_matcher_count':127,'original_scope_count':4,'original_sealed_record_count':39,'validated_changed_historical_records':historical,'new_source_JSON_directory_or_blanket_exclusions':False,'whole_content_and_path_class_scan_cache_entries':len(cache),'full_T_gate_passed':False}
REPORT.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:result[k] for k in ['status','mode','decoded_complete_payload_count','source_inputs_unchanged_during_review','links_checked','findings','missing_links','whitespace_exit_code']}))
raise SystemExit(result['status']!='passed')
