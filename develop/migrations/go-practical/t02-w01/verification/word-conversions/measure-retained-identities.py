"""Named original metadata observation, never proof/import authority."""
from pathlib import Path
import datetime
import hashlib
import json
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
reader=root.parent/'word-signed-division/prerequisite-analysis/measure.py'
namespace={'__file__':str(reader.resolve())}
exec(compile(reader.read_text().split('sources=[]',1)[0],str(reader),'exec'),namespace)
Tables=namespace['Tables']
current=Tables(seal(root/'actual-count-transport/rust-construction/certificates/word-conversions.mpcert'))
refs=[];union={}
for directory,expected in [('word-order','c12b72e2f331427e24465ef60ef228b469d6c54c6d414bf005b4ba3a1a9380fa'),('word-arithmetic','2c3261d71d9a7c8fcad98700f55f1ccc26865d7a11b57ab2f30b21d667ef0b1c')]:
 record=json.loads((root.parent/directory/'verification.json').read_text())['fresh_component']['certificate']
 assert record['sha256']==expected
 t=Tables(record);fp=t.fingerprints();refs.append((record,fp))
 for name,value in fp.items():
  if name in union:assert union[name]==value,name
  union[name]=value
retained={};new=[]
for name,value in current.fingerprints().items():
 if name.startswith('Mpk.Go.Practical.Word.Conversions.'):new.append(name)
 else:assert union[name]==value,name;retained[name]=value
assert len(current.index)==1882 and len(retained)==888 and len(new)==994
record={'schema':'mpk.go_practical.word_conversions_retained_identity_observation.v1','status':'matched_structural_identities_not_checker_authority','observed_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'certificate':current.record,'complete_original_reference_certificates':[r for r,_ in refs],'reader_source':seal(reader),'reader_extraction':'Only definitions before sources=[]; original planning main is not executed or edited.','parser_sources':[seal('crates/mpk-cert/src/decode.rs'),seal('crates/mpk-cert/src/binary_tags.rs')],'all_retained_original_named_identities':888,'new_conversion_declarations':994,'reference_memberships':[{'certificate':r,'matched_names':sum(n in fp for n in retained)}for r,fp in refs],'retained_names':sorted(retained),'new_conversion_names':sorted(new),'scope':'Every retained type/body/proof/level/family identity equals its original named-DAG reference without any rewriting or substitution. The complete old factories retain their exact raw bytes in the current Rust root. Structural metadata supplies diagnostic evidence only: both unchanged production cores check actual complete proof terms. No fingerprint, parser or receipt grants source/import/profile/application authority.'}
p=root/'retained-identities.json';assert not p.exists();p.write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps({k:record[k] for k in ['status','all_retained_original_named_identities','new_conversion_declarations']}))
