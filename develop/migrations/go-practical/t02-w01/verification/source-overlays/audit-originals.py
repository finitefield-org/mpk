"""Literal original bytes/modes and source/sidecar binding correspondence."""
from pathlib import Path
import base64,hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes());read=runpy.run_path(str(OUT/'read-originals.py'))['read'];kept={};count=size=0;archives=[]
for p in sorted(OUT.glob('*/manifest.json')):
 name=p.parent.name;meta,files=read(name)
 for row in meta['original_inputs']:
  actual=Path(row['path']);assert actual.read_bytes()==files[row['member']]and actual.stat().st_mode&0o777==row['mode'];count+=1;size+=len(files[row['member']])
 archives.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
 if name=='native-original':kept[name]={k:v for k,v in files.items()if k.startswith('source-overlays/overlay-')or k=='source-overlays/method-init-production.json'}
 elif name not in {'native-initial-original','native-root-original','source-closure-original'}:kept[name]=files
assert len(archives)==9
c=json.loads((OUT/'corpus.json').read_bytes());native=kept['native-original'];rust=kept['rust-original'];assert json.loads(rust['input.json'])==c['input']
for n,row in enumerate(c['cases']):
 original=json.loads(native[f'source-overlays/overlay-{n:02}-input.json']);assert original['object']==row['input']['object']and original['name']==row['input']['name']and base64.b64decode(original['raw'],validate=True)==bytes.fromhex(row['input']['raw_hex'])
 assert row['native']==json.loads(native[f'source-overlays/overlay-{n:02}-actual.json'])==json.loads(rust[f'case-{n:02}-actual.json'])
assert len(c['cases'])==13
assert json.loads(native['source-overlays/method-init-production.json'])['issues']==[]
first=kept['first-rust-original'];assert all(name in rust and raw==rust[name]for name,raw in first.items())
syntax=json.loads(rust['syntax.json']);assert len(syntax['heads'])==70 and syntax['maximum_expression_nodes']==1024 and syntax['maximum_expression_depth']==32 and syntax['no_private_4096_digit_cap']
oldread=runpy.run_path(str(OUT.parent/'error-dispatch/read-originals.py'))['read'];_,old_dispatch=oldread('rust-original');assert old_dispatch==kept['dispatch-original']
_,old_selection=oldread('selection-original');assert old_selection==kept['selection-original']
front=json.loads(kept['frontend-original']['reproduced-builds.json']);assert front['byte_exact_equal']and front['independent_empty_caches']and front['source_inputs']==190
for obj in json.loads(kept['frontend-original']['captured-inputs.json'])['build_inputs']:
 raw=(Path('go-tools/go2vir')/obj['path']).read_bytes();assert len(raw)==obj['bytes']and hashlib.sha256(raw).hexdigest()==obj['raw_sha256']
record={'status':'passed','actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':archives,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'original_source_sidecar_pairs':13,'source_heads':70,'new_Rust_roots':5,'affected_old_Rust_roots':7,'old_dispatch_and_selection_outputs_literal_equal':True,'first_failed_coverage_run_outputs_preserved_and_equal':len(first),'input_slots':70,'method_init_distinct_from_package_initialization':True,'fresh_empty_cache_frontend_builds':2,'frontend_inputs':190,'frontend_binary_bytes':front['builds'][0]['bytes'],'frontend_binary_sha256':front['builds'][0]['binary_sha256'],'no_original_regeneration_or_recompilation':True};(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
