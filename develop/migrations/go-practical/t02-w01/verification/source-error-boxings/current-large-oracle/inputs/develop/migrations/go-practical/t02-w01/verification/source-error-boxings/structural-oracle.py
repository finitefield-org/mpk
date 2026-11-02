"""Independent complete-box construction oracle for already accepted bytes."""
from pathlib import Path
import functools
import importlib.util
import hashlib
import json

ROOT=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('frozen_source_error_shapes',ROOT.parent/'source-error-layouts/structural-oracle.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
f=m.f


def literal(value):
    names=[f'_bit{i}' for i in range(6)]
    def tree(start,size,depth):
        if size==1:return f.bit(bool(value & (1<<start)))
        half=size//2
        left,right=tree(start,half,depth+1),tree(start+half,half,depth+1)
        if left==right:return left
        if left==f.bit(False) and right==f.bit(True):return f.var(names[depth])
        if left==f.bit(True) and right==f.bit(False):return f.app(f.const('Mpk.Go.Practical.Bool.Not'),f.var(names[depth]))
        return f.app(f.const('Std.Bool.cases'),f.lam('_case',f.boolean(),f.boolean()),left,right,f.var(names[depth]))
    return f.bind('lam',[(n,f.boolean()) for n in names],tree(0,64,0))


def inspect(path):
    raw=path.read_bytes();report=json.loads(path.with_suffix('.rust.json').read_bytes());source=json.loads(path.with_suffix('.sources.json').read_bytes())
    assert report['verdict']=='accepted' and report['axiom_count']==0
    assert report['hashes']['certificate']==hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()
    r=m.Sections(raw);assert raw[:7]==b'MPKCERT';r.at=7
    assert r.string()=='MPK-CERT-0.1' and r.string()=='MPK-Core-0.1'
    module=r.string();assert module==source['module']==report['module'];assert r.number()==0
    names=r.vec(r.string);levels=r.vec(r.level);terms=r.vec(r.term);assert r.number()==0
    declarations=r.vec(r.declaration);table={names[d[0]]:d for d in declarations}
    assert len(declarations)==report['declaration_count']
    @functools.lru_cache(None)
    def level(i):
        tag,*args=levels[i]
        assert tag==0
        return ('level',0)
    @functools.lru_cache(None)
    def term(i):
        tag,*args=terms[i]
        if tag==0:return ('sort',level(args[0]))
        if tag==1:return ('var',args[0])
        if tag==2:return ('const',names[declarations[args[0]][0]],tuple(level(j) for j in args[1]))
        if tag==3:return ('app',term(args[0]),tuple(term(j) for j in args[1]))
        if tag in [4,5]:return ('lam' if tag==4 else 'pi',term(args[0]),term(args[1]))
        return ('let',*(term(j) for j in args))
    def evaluate(i,scope=()):
        tag,*args=terms[i]
        if tag==1:return scope[-1-args[0]]
        if tag==2:
            name=names[declarations[args[0]][0]]
            if name=='Std.Bool.false':return False
            if name=='Std.Bool.true':return True
            d=declarations[args[0]];assert d[1]==1,name
            return evaluate(d[3])
        if tag==4:return ('closure',args[1],scope)
        assert tag==3
        head=terms[args[0]]
        if head[0]==2 and names[declarations[head[1]][0]]=='Std.Bool.cases':
            assert len(args[1])==4
            condition=evaluate(args[1][3],scope);assert type(condition) is bool
            return evaluate(args[1][2 if condition else 1],scope)
        if head[0]==2 and names[declarations[head[1]][0]]=='Std.Bool.rec':
            assert len(args[1])==3
            condition=evaluate(args[1][2],scope);assert type(condition) is bool
            return evaluate(args[1][1 if condition else 0],scope)
        function=evaluate(args[0],scope)
        for argument in args[1]:
            assert function[0]=='closure'
            function=evaluate(function[1],function[2]+(evaluate(argument,scope),))
        return function
    def observed_bit(value,address):
        function=evaluate(value)
        for digit in range(5,-1,-1):
            assert function[0]=='closure'
            function=evaluate(function[1],function[2]+(bool(address&(1<<digit)),))
        assert type(function) is bool
        return function

    parent=source['source_layout_module'];tags=source['case_tags'];cases=[row[0] for row in tags]
    assert cases==sorted(set(cases)) and [row[1] for row in tags]==list(range(1,len(cases)+1))
    keys={t['id']:t for t in source['types']}
    assert all(keys[id]['kind']=='nominal' for id in cases)
    def pair(id):return f.const(f'{parent}.T{id}.Carrier'),f.const(f'{parent}.T{id}.Zero')
    def product(parts):return f.product(parts,[0])
    types=[f.word(64)[0],*[pair(id)[0] for id in cases]]
    carrier=product([(ty,f.bit(False)) for ty in types])[0]
    results=[]
    for index,(id,ordinal) in enumerate(tags):
        prefix=module+'.C'+id
        payloads=[pair(other) for other in cases]
        payloads[index]=(pair(id)[0],f.var('v'))
        payload_type,payload=product(payloads)
        _,boxed=product([(f.word(64)[0],f.const(prefix+'.Tag')),*payloads])
        ty=pair(id)[0]
        expected=[('Tag',f.word(64)[0],literal(ordinal)),
            ('Payload',f.pi('v',ty,payload_type),f.lam('v',ty,payload)),
            ('Box',f.pi('v',ty,carrier),f.lam('v',ty,boxed)),
            ('SourcePayloadCarrier',{'kind':'sort0'},ty)]
        for name,t,v in expected:
            d=table[prefix+'.'+name];assert d[1]==1
            assert term(d[2])==m.frozen_ast(t),(id,name,'type')
            assert term(d[3])==m.frozen_ast(v),(id,name,'body')
        tag_decl=table[prefix+'.Tag']
        for address in range(64):
            assert observed_bit(tag_decl[3],address)==bool(ordinal&(1<<address)),(id,'full64-bit tag',address)
        for theorem in ['BoxRecipe','TagOfBox','PayloadOfBox','ActiveOfBox','TagNonzero','IsNilOfBox','IsNonNilOfBox','ConcreteZeroIsNonNil','HasOwnCase']:
            assert table[prefix+'.'+theorem][1]==2
        results.append({'source_type_id':id,'tag':ordinal,'all64_tag_bits_and_whole_active_inactive_payload_structure_equal':True})
    return {'certificate':str(path),'raw_sha256':hashlib.sha256(raw).hexdigest(),'cases':results,'empty_U_has_no_constructor':not bool(tags),'full_source_U_preserved':True,'declarations':len(declarations),'terms':len(terms),'bytes':len(raw),'actual_tag_bit_observations':64*len(cases)}
