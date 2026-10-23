"""Scoped unchanged-rule review, including every full original and failed epoch.

Run outside the sandbox: all Git reads use the authorized system Git.
"""
from pathlib import Path
import hashlib,json,re,runpy,subprocess,sys,types,urllib.parse
REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
BASE='7aa3006df1b86ed465c123236ef7b0dcfc974778'
REPORT=ROOT/'input-review.json'
def git(*args,data=None):return subprocess.check_output(['/usr/bin/git',*args],input=data)
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
def logical(p):
 p=Path(p);return str(p.relative_to(REPO)) if p.is_absolute() and p.is_relative_to(REPO) else str(p)
assert git('rev-parse','HEAD').decode().strip()==BASE
assert not Path('.github/workflows').exists()
scanner=Path('scripts/check-no-active-gir.sh')
code=scanner.read_text().split("<<'PY'\n",1)[1].rsplit('\nPY',1)[0].rsplit('raise SystemExit(main())',1)[0]
m=types.ModuleType('source_string_review');m.__file__=str(REPO/scanner);sys.modules[m.__name__]=m;sys.argv=[str(scanner),str(REPO),'strict']
exec(compile(code,str(scanner),'exec'),m.__dict__)
matchers=m.load_matchers();manifest,configured,fixtures,_=m.validate_manifest();m.run_fixture_self_test(manifest,matchers);exclusions=m.active_retained_record_exclusions(manifest)
assert (len(matchers),len(m.SCOPES),len(manifest['sealed_records']),len(exclusions))==(127,4,39,51)
for name in configured|fixtures:assert Path(name).read_bytes()==git('show',BASE+':'+name)
base_blobs={};base_modes={}
for item in git('ls-tree','-rz',BASE).split(b'\0'):
 if not item:continue
 meta,name=item.split(b'\t',1);mode,kind,oid=meta.decode().split()
 assert kind=='blob';base_blobs[name.decode()]=oid;base_modes[name.decode()]=int(mode,8)&0o777
# Validate original tracked bytes in one Git transaction.
epochs=[(p,json.loads(p.read_bytes())) for p in sorted(ROOT.glob('*/inputs.json'))]
oids=sorted({r['original_git_blob'] for _,e in epochs for r in e['sources'] if 'original_git_blob' in r})
raw=git('cat-file','--batch',data=('\n'.join(oids)+'\n').encode());at=0;original={}
for oid in oids:
 end=raw.index(b'\n',at);actual,kind,n=raw[at:end].decode().split();n=int(n);assert (actual,kind)==(oid,'blob');original[oid]=raw[end+1:end+1+n];at=end+n+2
assert at==len(raw)
# Binaries already retained in the basis remain visible as basis findings.
# This comparison is on their complete Git blob identity, never an exclusion.
basis_data={}
for old in ['source-string-operations','source-string-lengths','source-string-indices','source-strings','source-value-relations','source-error-representation','core-interfaces']:
 for receipt in sorted((ROOT.parent/old).glob('*/prelaunch-inputs.json')):
  for r in json.loads(receipt.read_bytes()):
   saved=r.get('complete_prelaunch_snapshot')
   if not saved:continue
   name=logical(saved['path']);p=Path(saved['path'])
   if name not in base_blobs or not p.is_file():continue
   b=p.read_bytes();oid=hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
   if oid==base_blobs[name]:basis_data[(len(b),hashlib.sha256(b).hexdigest())]=name
cache={};findings=[];baseline=[];origins={};source_rows=0;executions=[]
path_rules=[r for r in matchers if r['scope']=='exact-path']
def scan(name,b,basis=False):
 key=(hashlib.sha256(b).hexdigest(),tuple(m.path_matches(name,r['path']) for r in path_rules))
 if key not in cache:cache[key]=[vars(f) for f in m.detect(name,b.decode(errors='ignore'),matchers)]
 for row in cache[key]:
  target=baseline if basis else findings
  target.append({**{k:v for k,v in row.items() if k!='excerpt'},'path':name,'excerpt_hex':row['excerpt'].encode().hex()})
for p,e in epochs:
 assert seal(p.parent/'driver.py')==e['actual_driver_prelaunch']
 result=json.loads(p.with_name('result.json').read_bytes())
 assert result['status'] in ['passed','failed'] and result['source_receipt']==seal(p)
 assert all(seal(result[k]['path'])==result[k] for k in ['stdout','stderr'])
 if not result['formatter_mutation_expected']:assert result['whole_source_inputs_unchanged']
 for r in e['sources']:
  oid=r.get('original_git_blob')
  if oid:
   assert base_blobs[r['path']]==oid and base_modes[r['path']]==r['mode'] and r['original_commit']==BASE;b=original[oid]
  else:
   saved=Path(r['complete_changed_input']);b=saved.read_bytes();assert saved.stat().st_mode&0o777==r['mode'];origins[logical(saved)]=r['path']
  assert len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256'];scan(r['path'],b,basis=bool(oid));source_rows+=1
 executions.append({'result':seal(p.with_name('result.json')),'status':result['status'],'command':result['command']})
prelaunch=[]
encoded_originals=[]
for receipt in sorted(ROOT.glob('*/prelaunch-inputs.json')):
 records=json.loads(receipt.read_bytes());result=json.loads(receipt.with_name('result.json').read_bytes());assert result['status'] in ['passed','failed'] and result['complete_prelaunch_inputs']==seal(receipt)
 if result['status']=='passed':assert result['whole_inputs_unchanged']
 for r in records:
  saved=r['complete_prelaunch_snapshot'];encoded=Path(saved['path']).read_bytes();assert seal(saved['path'])==saved
  if r.get('snapshot_encoding')=='hex':
   b=bytes.fromhex(encoded.decode());assert Path(saved['path']).stat().st_mode&0o777==r['snapshot_mode']==0o644
   encoded_originals.append((r,b))
  else:
   b=encoded;assert Path(saved['path']).stat().st_mode&0o777==r['mode']
  assert len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256']
  origins[logical(saved['path'])]=logical(r['path'])
  scan(logical(r['path']),b,basis=(len(b),r['sha256']) in basis_data)
 prelaunch.append({'receipt':seal(receipt),'rows':len(records)})
read=runpy.run_path(str(ROOT/'read-originals.py'))['read'];virtual=[];virtual_inputs={}
relocations=json.loads((ROOT/'native-original-relocations.json').read_bytes());assert relocations['status']=='passed'
for group in relocations['groups']:
 manifest,files=read(group['archive'])
 for r in manifest['original_inputs']:
  b=files[r['member']];scan(logical(r['path']),b);virtual_inputs[r['path']]=(b,r['mode'])
  retained=Path(group['retained_physical_directory'])/r['member'];assert retained.read_bytes()==b and retained.stat().st_mode&0o777==r['mode']
 virtual.append({'archive':group['archive'],'files':len(files),'bytes':sum(map(len,files.values())),'decoded_and_scanned_with_original_paths_and_modes':True})
for r,b in encoded_originals:
 p=Path(r['path'])
 if p.exists():assert p.read_bytes()==b and p.stat().st_mode&0o777==r['mode']
 else:assert virtual_inputs[str(p)]==(b,r['mode'])
# Every current changed file is reviewed; copied documents resolve links from
# their original logical locations, rather than the physical snapshot folder.
if not REPORT.exists():REPORT.write_text('{}\n')
paths=sorted(set((git('diff','--name-only','-z','HEAD')+git('ls-files','--others','--exclude-standard','-z')).decode().split('\0'))-{'',logical(REPORT)})
inputs=[];links=[]
for name in paths:
 p=Path(name);assert p.is_file() and not p.is_symlink(),name;b=p.read_bytes()
 inputs.append({**seal(p),'mode':p.stat().st_mode&0o777})
 basis=(len(b),hashlib.sha256(b).hexdigest()) in basis_data
 scan(origins.get(name,name),b,basis)
 if p.suffix=='.md' or origins.get(name,'').endswith('.md'):
  source=Path(origins.get(name,name))
  for target in re.findall(r'\]\(([^)]+)\)',b.decode(errors='strict')):
   target=target.strip('<>').split('#',1)[0]
   if not target or re.match(r'^[a-zA-Z][a-zA-Z0-9+.-]*:',target):continue
   target=urllib.parse.unquote(target);destination=(source.parent/target).resolve() if not target.startswith('/') else Path(target)
   destination=Path(re.sub(r':\d+$','',str(destination)))
   links.append({'source':str(source),'target':target,'exists':destination.exists()})
assert all(r['exists'] for r in links)
record={'status':'passed' if not findings else 'failed','basis_commit':BASE,'matcher_count':127,'scopes':4,'seals':39,'exclusions':51,'scanner_rules_unchanged':True,'strict_whole_repository_gate_passed':False,'actual_driver_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'source_epochs':len(epochs),'source_preimages':source_rows,'executions':executions,'prelaunch_receipts':prelaunch,'complete_virtual_originals_scanned':virtual,'inputs':inputs,'findings':findings,'preexisting_basis_findings':baseline,'links':links,'scope':'Affected production original-package source handoff, strict source-only Rust input and its direct consumers. Every successful/failed input epoch, all three complete decoded native archives and encoded original-mode600 input remains scanned with exact original logical paths and modes. All32 prior proof bytes are replayed exactly; unchanged kernel matrices are not repeated. No old finding, active source, whole certificate, original field or native cap is excluded. Cumulative strict/T02 gate remains final W04.'}
REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':record['status'],'source_epochs':len(epochs),'new_findings':len(findings),'basis_findings':len(baseline),'links':len(links),'missing_links':sum(not r['exists'] for r in links)}));raise SystemExit(bool(findings))
