"""Complete named metadata observation, independent of numeric table positions."""
from pathlib import Path
import hashlib,json,sys
root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve());attempt=sys.argv[1]
def seal(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
reader=root.parent/'word-signed-division/prerequisite-analysis/measure.py';ns={'__file__':str(reader.resolve())};exec(compile(reader.read_text().split('sources=[]',1)[0],str(reader),'exec'),ns);Tables=ns['Tables']
p=root/attempt/'rust-construction/certificates/word-shift-counts.mpcert';c=Tables(seal(p));all=c.fingerprints();rows=[]
for rel,expected in [('../word-shifts/metadata-corrected/rust-tests/certificates/word-shifts.mpcert',3105),('full-count-boundaries/rust-construction/certificates/word-shift-counts.mpcert',3240),('actual-variable-left/rust-construction/certificates/word-shift-counts.mpcert',3352),('explicit-selected-top/rust-construction/certificates/word-shift-counts.mpcert',3542),('private-saturation-entry/rust-construction/certificates/word-shift-counts.mpcert',3685)]:
 old=Tables(seal(root/rel));fp=old.fingerprints();assert len(fp)==expected
 for n,v in fp.items():assert all[n]==v,n
 rows.append({'original_certificate':old.record,'all_named_types_bodies_proofs_levels_families_equal':expected})
assert len(c.index)==4053
out=root/'retained-identities.json';assert not out.exists();out.write_text(json.dumps({'status':'all_prior_named_identities_equal_not_proof_authority','current_certificate':c.record,'current_declarations':4053,'comparisons':rows,'reader_source':seal(reader),'scope':'Only original reader definitions before sources=[] are loaded. All complete prior accepted component names/typed proof metadata match without rename/substitution; both unchanged cores still independently inspect full current proof terms. No metadata/parser receipt gives authority.'},indent=2)+'\n');print(json.dumps({'status':'matched','prior_complete_prefixes':[3105,3240,3352,3542,3685],'current':4053}))
