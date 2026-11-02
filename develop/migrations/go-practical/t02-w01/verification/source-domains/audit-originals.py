"""Literal original bytes/modes plus current source-domain comparisons."""
from pathlib import Path
import hashlib,json,runpy
OUT=Path(__file__).resolve().parent;RUN=OUT/'original-byte-audit';RUN.mkdir(exist_ok=False);(RUN/'driver.py').write_bytes(Path(__file__).read_bytes());read=runpy.run_path(str(OUT/'read-originals.py'))['read'];native_rows=json.loads((OUT/'derived/inputs.json').read_bytes());producer={str(Path(r['copy']['path']).relative_to(OUT/'native-original')):r['original']for r in native_rows};count=size=0;kept={};archives=[]
for p in sorted(OUT.glob('*/manifest.json')):
 name=p.parent.name;manifest,files=read(name)
 for row in manifest['original_inputs']:
  actual=Path(producer[row['member']]['path'])if name=='native-archive'else Path(row['path']);assert actual.read_bytes()==files[row['member']]and actual.stat().st_mode&0o777==row['mode'];count+=1;size+=len(files[row['member']])
 archives.append({'name':name,'files':len(files),'bytes':sum(map(len,files.values()))})
 if name=='native-archive':kept[name]={k:v for k,v in files.items()if k.startswith('source-domains/domains-')}
 else:kept[name]=files
assert len(archives)==6
c=json.loads((OUT/'corpus.json').read_bytes());native=kept['native-archive'];assert c['observations']==json.loads(native['source-domains/domains-observations.json']);assert c['roots']==json.loads(native['source-domains/domains-roots.json']);assert c['declarations']==json.loads(native['source-domains/domains-declarations.json'])
rust=kept['corrected-rust-domains-original'];assert json.loads(rust['domain-input.json'])==c['input'];rows=json.loads(rust['domain-observations.json']);assert len(rows)==len(c['observations'])==15
for actual,expected in zip(rows,c['observations']):
 assert actual['root']==expected['root']and actual['cases']==(expected['cases']or[])
 errors=[p['error']for p in expected['inputs']if p['error']]
 if errors:assert actual['result']['error']==errors[0]
 else:assert actual['result']['inputs']==[p['properties']for p in expected['inputs']]
selected=kept['affected-selection-rust-selection-original'];assert json.loads(selected['current-summary.json'])=={'positive':24,'negative':27,'failures':[]}and json.loads(selected['baseline-summary.json'])=={'positive':15,'negative':18,'failures':[]}
# Every old boxing result must remain literally identical to its previous output.
previous=runpy.run_path(str(OUT.parent/'error-boxings/read-originals.py'))['read'];_,old=previous('rust-original');assert kept['affected-boxings-rust-boxings-original']==old
assert json.loads(rust['domain-union.json'])['explicit_entry_cases']==70
record={'status':'passed','actual_driver_prelaunch_sha256':hashlib.sha256((RUN/'driver.py').read_bytes()).hexdigest(),'archives':archives,'actual_original_files_compared_literally':count,'actual_original_bytes_compared_literally':size,'all_original_modes_compared':True,'compiler_property_comparisons':15,'new_Rust_roots':5,'affected_old_Rust_roots':6,'old_selection_cases':84,'old_selection_positive_graphs':39,'old_selection_rejections':45,'old_boxing_outputs_literal_equal':True,'case_count_control':70,'no_original_regeneration_or_recompilation':True};(RUN/'result.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record))
