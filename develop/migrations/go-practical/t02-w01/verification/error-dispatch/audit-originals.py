"""Literal byte/mode audit and independent native closure correspondence."""
from pathlib import Path
import hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes());read=runpy.run_path(str(OUT/'read-originals.py'))['read'];kept={};archives=[];count=size=0
for p in sorted(OUT.glob('*/manifest.json')):
 name=p.parent.name;meta,files=read(name)
 for row in meta['original_inputs']:
  actual=Path(row['path']);assert actual.read_bytes()==files[row['member']]and actual.stat().st_mode&0o777==row['mode'];count+=1;size+=len(files[row['member']])
 archives.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
 if name=='native-original':kept[name]={k:v for k,v in files.items()if k.startswith('error-dispatch/dispatch-')or k.startswith('error-dispatch/production-')}
 elif name not in {'native-initial-original','source-closure-original'}:kept[name]=files
assert len(archives)==14
c=json.loads((OUT/'corpus.json').read_bytes());native=kept['native-original'];rust=kept['rust-original'];assert json.loads(rust['dispatch-input.json'])==c['input'];positive=negative=0
for n,test in enumerate(c['cases']):
 assert test['input']==json.loads(native[f'error-dispatch/dispatch-{n:02}-input.json'])and test['native']==json.loads(native[f'error-dispatch/dispatch-{n:02}-actual.json'])
 if not test['input']['expected_rejection']:
  actual=json.loads(rust[f'case-{n:02}-actual.json']);assert actual==test['native'];positive+=1
 else:assert f'case-{n:02}-rejection.json'in rust;negative+=1
assert(positive,negative)==(19,4)
first=kept['first-rust-original'];assert all(name in rust and raw==rust[name]for name,raw in first.items());assert 'dispatch-summary.json'not in first
summary=json.loads(rust['dispatch-summary.json']);assert(summary['accepted'],summary['rejected'])==(19,4)
for name in ['selection','domains','boxings']:assert kept[name+'-before-cache']==kept[name+'-original']
selected=kept['selection-original'];assert json.loads(selected['current-summary.json'])=={'positive':24,'negative':27,'failures':[]}and json.loads(selected['baseline-summary.json'])=={'positive':15,'negative':18,'failures':[]}
previous=runpy.run_path(str(OUT.parent/'error-boxings/read-originals.py'))['read'];_,oldbox=previous('rust-original');assert kept['boxings-original']==oldbox
prior_domains=runpy.run_path(str(OUT.parent/'source-domains/read-originals.py'))['read'];_,olddomain=prior_domains('corrected-rust-domains-original');assert kept['domains-original']==olddomain
_,oldmethod=previous('method-original');newmethod=kept['method-original'];assert json.loads(newmethod['summary.json'])=={'accepted':146,'rejected':300,'outside':15,'failures':[]}
promoted=[];changed_negative_diagnostics=0
for i in range(461):
 key=f'case-{i:05}-input.json';assert newmethod[key]==oldmethod[key];key=f'case-{i:05}-actual.json';a=json.loads(oldmethod[key]);b=json.loads(newmethod[key])
 if a['accepted']:assert b==a
 elif b['accepted']:promoted.append(i)
 elif a!=b:changed_negative_diagnostics+=1
assert promoted==[154,300,301]
front=json.loads(kept['frontend-original']['reproduced-builds.json']);assert front['byte_exact_equal']and front['independent_empty_caches']and front['source_inputs']==190
for obj in json.loads(kept['frontend-original']['captured-inputs.json'])['build_inputs']:
 raw=(Path('go-tools/go2vir')/obj['path']).read_bytes();assert len(raw)==obj['bytes']and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
record={'status':'passed','actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':archives,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'positive_closure_comparisons':positive,'original_negative_target_rejections':negative,'new_Rust_roots':3,'affected_old_Rust_roots':18,'partial_profile_baseline_outputs_literal_equal':len(first),'partial_baseline_full_suite_passed':False,'old_selection_graphs':39,'old_selection_rejections':45,'old_boxing_and_domain_outputs_literal_equal':True,'old_method_cases':461,'old_method_newly_supported_direct_error_headers':promoted,'old_negative_diagnostic_changes':changed_negative_diagnostics,'explicit_case_control':70,'successive_growth_rounds':70,'fresh_empty_cache_frontend_builds':2,'frontend_inputs':190,'frontend_binary_bytes':front['builds'][0]['bytes'],'frontend_binary_sha256':front['builds'][0]['binary_sha256'],'no_original_regeneration_or_recompilation':True};(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
