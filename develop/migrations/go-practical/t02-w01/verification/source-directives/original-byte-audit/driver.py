"""Compare complete actual outputs and independently recovered source diagnostics."""
from pathlib import Path
import base64,datetime,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False)
(RUN/'driver.py').write_bytes(Path(__file__).read_bytes());read=runpy.run_path(str(OUT/'read-originals.py'))['read']
def seal(p):
 raw=Path(p).read_bytes();return {'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
archives=[];kept={};count=size=0
for p in sorted(OUT.glob('*/manifest.json')):
 name=p.parent.name;manifest,files=read(name)
 for row in manifest['original_inputs']:
  path=Path(row['path']);assert path.read_bytes()==files[row['member']]and path.stat().st_mode&0o777==row['mode'];count+=1;size+=len(files[row['member']])
 archives.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
 if name=='native-original':kept[name]={n:r for n,r in files.items()if n.startswith('closure-')}
 if name=='rust-original':kept[name]=files
 if name=='failed-native-original':
  failure=json.loads(files['storage-original-exports.json']);stderr=base64.b64decode(failure['stderr'],validate=True);assert stderr.count(b'misplaced compiler directive')==2
 if name=='initial-rust-original':
  assert len(files)==114 and all(f'{label}-case-27-input.json'in files and f'{label}-case-27-actual.json'not in files for label in ['current','baseline'])
assert len(archives)==5 and count==22445 and size==239797673
rust=kept['rust-original'];counts={};norm=lambda v:sorted(v,key=lambda r:json.dumps(r,sort_keys=True,separators=(',',':')))
physical=lambda r:{k:r[k]for k in ['source','start','end']}
for label,path,total in [('current',OUT/'corpus.json',51),('baseline',OUT.parent/'source-selection/corpus.json',33)]:
 cases=json.loads(path.read_bytes());assert len(cases)==total;positive=negative=calls=deps=declarations=directives=0;paths=[]
 for n,row in enumerate(cases):
  if label=='current':assert row['native']==json.loads(kept['native-original'][f'closure-{n:02}-actual.json'])
  assert row['input']==json.loads(rust[f'{label}-case-{n:02}-input.json']);actual=json.loads(rust[f'{label}-case-{n:02}-actual.json'])
  if row['input']['expected_issue_codes']is None:
   assert'error'not in actual
   for key in ['calls','dependencies']:assert norm(actual[key])==norm(row['native'][key])
   expected=[{k:v for k,v in d.items()if k!='module_source_id'}for d in row['native']['declarations']];assert norm(actual['declarations'])==norm(expected)
   positive+=1;calls+=len(actual['calls']);deps+=len(actual['dependencies']);declarations+=len(actual['declarations'])
  else:assert'error'in actual;negative+=1
  if'compiler_directive'in actual:
   span=actual['compiler_directive'];issue=next(i for i in row['native']['issues']if i['code']=='compiler_directive'and i['source']==actual['source']and i['start']==span['start']and i['end']==span['end'])
   assert actual['path'][0]==physical(row['input']['root'])and actual['path'][-1]==physical(issue['declaration'])
   raw=next(bytes.fromhex(s['raw_hex'])for s in row['input']['sources']if s['object']['id']==actual['source']);comment=raw[span['start']:span['end']]
   assert comment.replace(b'\r',b'').startswith(b'//go:')and comment.endswith(b'\r')and raw[span['end']]==10
   for a,b in zip(actual['path'],actual['path'][1:]):assert any(physical(d['from'])==a and physical(d['to'])==b for d in row['native']['dependencies'])
   directives+=1;paths.append(len(actual['path']))
 assert json.loads(rust[f'{label}-summary.json'])=={'positive':positive,'negative':negative,'failures':[]}
 counts[label]={'cases':total,'positive_graphs':positive,'negative_rejections':negative,'calls':calls,'dependencies':deps,'declarations':declarations,'exact_directive_spans':directives,'directive_path_lengths':paths}
assert[(counts[k]['positive_graphs'],counts[k]['negative_rejections'],counts[k]['exact_directive_spans'])for k in ['current','baseline']]==[(24,27,10),(15,18,1)]
conversion=json.loads((OUT/'derived/result.json').read_bytes());assert conversion['status']=='passed'
for row in [conversion['actual_driver_prelaunch'],conversion['original_input_receipt'],conversion['output']]:assert all(seal(row['path'])[k]==row[k]for k in ['path','bytes','sha256'])
for row in json.loads(Path(conversion['original_input_receipt']['path']).read_bytes())['files']:
 p=Path(row['path']);assert all(seal(p)[k]==row[k]for k in ['path','bytes','sha256'])and p.stat().st_mode&0o777==row['mode']
parent_read=runpy.run_path(str(OUT.parent/'source-selection/read-originals.py'))['read'];_,front=parent_read('frontend-original');inputs=json.loads(front['captured-inputs.json'])['build_inputs'];assert len(inputs)==190
for obj in inputs:
 raw=(Path('go-tools/go2vir')/obj['path']).read_bytes();assert len(raw)==obj['bytes']and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
parser=Path('/private/tmp/mpk-go-practical-original-packages-sdk-20261021/go/src/go/parser/parser.go')
record={'status':'passed','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':archives,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'corpora':counts,'total_original_cases':84,'exact_positive_graphs':39,'independent_negative_rejections':45,'exact_directive_spans_and_paths':11,'full_original_comment_tokens_including_CR':True,'pinned_go_parser_source':seal(parser),'all_prior_frontend_inputs_unchanged':190,'fresh_frontend_rebuild_required':False,'initial_rust_observation_limit':'The first comparison asserted the directive span before writing case27 actual rows in both threads; those two actual rows were not retained. All114 actually available files, complete code/input epochs and actual stdout/stderr remain. The test now retains actual rows before assertions; no missing old row is recreated.','source_admission_capture_or_execution_program_proof':False,'not_recreated_or_recompiled':True}
(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({k:record[k]for k in ['status','actual_original_files_compared_literally','actual_original_bytes_compared_literally','total_original_cases','exact_directive_spans_and_paths']}))
