"""Observe genuine prior-math alternatives and exact case-row ownership."""
from pathlib import Path
import functools
import hashlib
import importlib.util
import json
import sys
root=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('case_row_owned_oracle',root/'structural-oracle.py')
o=importlib.util.module_from_spec(spec);spec.loader.exec_module(o)
f=o.f
controls,small,output=map(Path,sys.argv[1:])
prior=root.parent/'source-error-operations/explicit-bit/certificates/all-root-shapes.sources.json'
source=json.loads(prior.read_bytes());cases=source['case_tags'];boxing=source['boxing_module'];operations=source['module']
def decode(path):
    raw=path.read_bytes();report=json.loads(path.with_suffix('.rust.json').read_bytes())
    assert report['verdict']=='accepted' and report['axiom_count']==0
    assert report['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()
    r=o.m.Sections(raw);r.at=7;r.string();r.string();module=r.string();assert module==operations==report['module'];assert r.number()==0
    names=r.vec(r.string);levels=r.vec(r.level);terms=r.vec(r.term);assert r.number()==0;declarations=r.vec(r.declaration)
    table={names[d[0]]:d for d in declarations}
    @functools.lru_cache(None)
    def term(i):
        tag,*args=terms[i]
        if tag==0:assert levels[args[0]]==(0,);return ('sort',('level',0))
        if tag==1:return ('var',args[0])
        if tag==2:return ('const',names[declarations[args[0]][0]],())
        if tag==3:return ('app',term(args[0]),tuple(term(j) for j in args[1]))
        if tag in [4,5]:return ('lam' if tag==4 else 'pi',term(args[0]),term(args[1]))
        return ('let',*(term(j) for j in args))
    return table,term
p=controls/'duplicate-tags-prior-math.mpcert';table,term=decode(p)
d=table[boxing+'.C'+cases[2][0]+'.Tag'];assert term(d[3])!=o.m.frozen_ast(o.literal(3))
assert term(d[3])==('const',boxing+'.C'+cases[0][0]+'.Tag',())
p=controls/'constant-false-prior-math.mpcert';table,term=decode(p)
types=[f.word(64)[0],*[f.const(source['source_layout_module']+'.T'+id+'.Carrier') for id,_ in cases]]
carrier=f.product([(t,f.bit(False)) for t in types],[0])[0]
expected=f.bind('lam',[('case',f.word(64)[0]),('e',carrier)],f.app(f.const('Mpk.Go.Practical.Word.W64.Equal'),f.var('case'),f.app(f.const(boxing+'.Tag'),f.var('e'))))
d=table[operations+'.HasCase'];assert term(d[3])!=o.m.frozen_ast(expected)
assert term(d[3])==o.m.frozen_ast(f.bind('lam',[('case',f.word(64)[0]),('e',carrier)],f.bit(False)))
files=sorted(small.glob('all-root-shapes-C*.mpcert'));assert len(files)==4
actual,expected_owner=files[1],files[0]
for p in [actual,expected_owner]:assert json.loads(p.with_suffix('.rust.json').read_bytes())['verdict']=='accepted'
work=output.parent/'wrong-owner-copy';work.mkdir(exist_ok=False)
copy=work/'substituted.mpcert';copy.write_bytes(actual.read_bytes());copy.with_suffix('.rust.json').write_bytes(actual.with_suffix('.rust.json').read_bytes());copy.with_suffix('.sources.json').write_bytes(expected_owner.with_suffix('.sources.json').read_bytes())
a=json.loads(actual.with_suffix('.sources.json').read_bytes());e=json.loads(expected_owner.with_suffix('.sources.json').read_bytes());assert a['boxed_source_type_id']!=e['boxed_source_type_id'] and a['module']!=e['module']
try:o.inspect(copy)
except AssertionError as error:assert error.args==()
else:raise AssertionError('a different mathematically accepted source case row was substituted')
result={'status':'passed','prior_duplicate_tag_math_is_accepted_but_frozen_tag_body_mismatch':True,'prior_constant_false_math_is_accepted_but_frozen_comparison_body_mismatch':True,'different_accepted_case_row_owner_mismatch_detected':True,'actual_other_row':str(actual),'expected_owner_row':str(expected_owner),'wrong_owner_copy_is_literal_original_math_bytes':copy.read_bytes()==actual.read_bytes(),'scope':'Prior value equations are mathematically weaker than complete rows; exact source IDs and operation bodies remain required. No compiler/context/runtime/program authority is inferred.'}
output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
