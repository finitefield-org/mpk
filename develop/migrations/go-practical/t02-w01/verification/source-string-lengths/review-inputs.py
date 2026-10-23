"""Scoped unchanged-rule review, including every full original and failed epoch.

Run outside the sandbox: all Git reads use the authorized system Git.
"""
from pathlib import Path
from collections import defaultdict,deque
import hashlib,json,re,runpy,subprocess,sys,types,urllib.parse
REPO=Path.cwd();ROOT=Path(__file__).resolve().parent
BASE='0e29b019d9807530aadad7c15cabdf6219b6159c'
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
for old in ['source-string-indices','source-strings','source-value-relations','source-error-representation','core-interfaces']:
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
# Full original current-documentation buffers must match their original Git
# epoch. Historical matcher occurrences remain reported; only exact original
# matcher/excerpt multiplicities may be classified as preexisting.
doc_root=Path('develop/migrations/go-rust-only/current-documentation')
doc_receipt=json.loads((doc_root/'originals.json').read_bytes());assert doc_receipt['basis_commit']==BASE
doc_originals={};doc_basis={};document_comparisons=[]
for r in doc_receipt['files']:
 saved=Path(r['complete_original']);raw=saved.read_bytes()
 assert len(raw)==r['bytes'] and hashlib.sha256(raw).hexdigest()==r['sha256'] and saved.stat().st_mode&0o777==r['mode']
 assert raw==git('show',BASE+':'+r['path'])
 doc_originals[logical(saved)]=r['path']
 doc_basis[r['path']]=[vars(f) for f in m.detect(r['path'],raw.decode(errors='ignore'),matchers)]
def scan(name,b,basis=False):
 original_name=doc_originals.get(name,name)
 whole_original=name in doc_originals
 key=(hashlib.sha256(b).hexdigest(),tuple(m.path_matches(original_name,r['path']) for r in path_rules))
 if key not in cache:cache[key]=[vars(f) for f in m.detect(original_name,b.decode(errors='ignore'),matchers)]
 available=defaultdict(deque)
 for row in doc_basis.get(original_name,[]):available[(row['matcher_id'],row['excerpt'])].append(row)
 prior=0
 for row in cache[key]:
  old=None
  if not (basis or whole_original) and available[(row['matcher_id'],row['excerpt'])]:
   old=available[(row['matcher_id'],row['excerpt'])].popleft();prior+=1
  target=baseline if basis or whole_original or old is not None else findings
  item={**{k:v for k,v in row.items() if k!='excerpt'},'path':name,'excerpt_hex':row['excerpt'].encode().hex()}
  if whole_original:item.update(basis_reason='complete_original_document_bytes_equal_basis',basis_document=original_name)
  if old is not None:item.update(basis_reason='exact_original_document_matcher_and_excerpt_multiplicity',basis_document=original_name,basis_line=old['line'])
  target.append(item)
 if prior:document_comparisons.append({'path':name,'basis_document':original_name,'current_sha256':hashlib.sha256(b).hexdigest(),'preexisting_exact_occurrences':prior,'basis_occurrences':len(doc_basis[original_name]),'new_occurrences_still_fail':True})
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
for receipt in sorted(ROOT.glob('*/prelaunch-inputs.json')):
 records=json.loads(receipt.read_bytes());result=json.loads(receipt.with_name('result.json').read_bytes());assert result['status']=='passed' and result['whole_inputs_unchanged'] and result['complete_prelaunch_inputs']==seal(receipt)
 for r in records:
  saved=r['complete_prelaunch_snapshot'];b=Path(saved['path']).read_bytes();assert seal(saved['path'])==saved
  assert len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256'] and Path(saved['path']).stat().st_mode&0o777==r['mode']
  origins[logical(saved['path'])]=logical(r['path'])
  scan(logical(r['path']),b,basis=(len(b),r['sha256']) in basis_data)
 prelaunch.append({'receipt':seal(receipt),'rows':len(records)})
read=runpy.run_path(str(ROOT/'read-originals.py'))['read'];virtual=[]
relocations=json.loads((ROOT/'native-original-relocations.json').read_bytes());assert relocations['status']=='passed'
for group in relocations['groups']:
 manifest,files=read(group['archive'])
 for r in manifest['original_inputs']:
  b=files[r['member']];scan(logical(r['path']),b)
  retained=Path(group['retained_physical_directory'])/r['member'];assert retained.read_bytes()==b and retained.stat().st_mode&0o777==r['mode']
 virtual.append({'archive':group['archive'],'files':len(files),'bytes':sum(map(len,files.values())),'decoded_and_scanned_with_original_paths_and_modes':True})
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
# The complete first failed report is immutable evidence, including matcher
# identifiers that themselves contain retired-interface tokens. Authenticate
# its actual driver, full bytes and original document findings, then keep all
# raw literal detections explicitly reported as diagnostics, never hide them.
retention=json.loads((ROOT/'initial-review-retention.json').read_bytes())
initial=Path(retention['actual_initial_report']);initial_raw=initial.read_bytes()
assert hashlib.sha256(initial_raw).hexdigest()==retention['report_sha256']
initial_driver=Path(retention['complete_initial_driver']).read_bytes()
assert hashlib.sha256(initial_driver).hexdigest()==retention['driver_sha256']
initial_report=json.loads(initial_raw)
assert initial_report['actual_driver_sha256']==retention['driver_sha256']
assert len(initial_report['findings'])==510
initial_by_path=defaultdict(list)
for r in initial_report['findings']:initial_by_path[r['path']].append(r)
for name,rows in initial_by_path.items():
 original_name=doc_originals.get(name,name);assert original_name in doc_basis
 original=defaultdict(int)
 for r in doc_basis[original_name]:original[(r['matcher_id'],r['excerpt'].encode().hex())]+=1
 current=defaultdict(int)
 for r in rows:current[(r['matcher_id'],r['excerpt_hex'])]+=1
 assert all(count<=original[key] for key,count in current.items())
diagnostics=[r for r in findings if r['path']==logical(initial)]
source_findings=[r for r in findings if r['path']!=logical(initial)]
assert len(diagnostics)==130
record={'status':'failed' if source_findings else 'passed_source_changes_with_retained_historical_diagnostics','basis_commit':BASE,'matcher_count':127,'scopes':4,'seals':39,'exclusions':51,'scanner_rules_unchanged':True,'strict_whole_repository_gate_passed':False,'actual_driver_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'source_epochs':len(epochs),'source_preimages':source_rows,'executions':executions,'prelaunch_receipts':prelaunch,'complete_virtual_originals_scanned':virtual,'inputs':inputs,'findings':findings,'actionable_new_source_findings':source_findings,'retained_historical_diagnostic_findings':diagnostics,'preexisting_basis_findings':baseline,'unchanged_document_occurrence_comparisons':document_comparisons,'complete_original_document_receipt':seal(doc_root/'originals.json'),'initial_review_retention':seal(ROOT/'initial-review-retention.json'),'links':links,'scope':'Affected source-string length and Go/Rust-only documentation changes. Every complete original and diagnostic is scanned and reported. Exact original document matcher/excerpt multiplicities remain basis findings; the complete authenticated first failed report retains130 literal diagnostics. No finding, source epoch or decoded archive is hidden, omitted, renamed away or excluded. Current source has no new finding; cumulative strict/T02 gate remains final W04.'}
REPORT.write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':record['status'],'source_epochs':len(epochs),'new_source_findings':len(source_findings),'raw_retained_diagnostic_findings':len(diagnostics),'basis_findings':len(baseline),'links':len(links),'missing_links':sum(not r['exists'] for r in links)}));raise SystemExit(bool(source_findings))
