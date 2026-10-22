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
        prefix=source['boxing_module']+'.C'+id
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
    certificate_module=module
    module=source['operations_module']
    assert module==source['boxing_module']+'.Operations'
    errors=[id for id,key in keys.items() if key['kind']=='builtin' and key['name']=='error'];assert len(errors)==1
    error=errors[0]
    boxing=source['boxing_module']
    def named(name,*args):return f.app(f.const(module+'.'+name),*args)
    def observe(name,value):return f.app(f.const(boxing+'.'+name),value)
    def group(ts):return product([(ty,f.bit(False)) for ty in ts])[0]
    def projection(ts,value,index):
        if len(ts)==1:return value
        half=len(ts)//2
        part=f.app(f.const('Mpk.Go.Practical.Product.'+('Left' if index<half else 'Right')),group(ts[:half]),group(ts[half:]),value)
        return projection(ts[:half],part,index) if index<half else projection(ts[half:],part,index-half)
    e=f.var('e');nil=named('Nil');copied=named('Copy',e)
    zeros=[pair(id)[1] for id in cases];raw_payload=product(list(zip(types[1:],zeros)))[1]
    fields=[projection(types,e,i) for i in range(len(types))]
    copy_value=product(list(zip(types,fields)))[1]
    zero_word=named('NilTag')
    definitions=[('Nil',carrier,f.const(parent+'.T'+error+'.Zero')),
        ('NilTag',f.word(64)[0],f.word(64)[1]),
        ('NilPayload',group(types[1:]),raw_payload),
        ('Copy',f.pi('e',carrier,carrier),f.lam('e',carrier,copy_value)),
        ('HasCase',f.bind('pi',[('case',f.word(64)[0]),('e',carrier)],f.boolean()),f.bind('lam',[('case',f.word(64)[0]),('e',carrier)],f.app(f.const('Mpk.Go.Practical.Word.W64.Equal'),f.var('case'),observe('Tag',e))))]
    for name,ty,body in definitions:
        d=table[module+'.'+name];assert d[1]==1
        assert term(d[2])==m.frozen_ast(ty),(name,'type')
        assert term(d[3])==m.frozen_ast(body),(name,'body')
    for address in range(64):assert observed_bit(table[module+'.NilTag'][3],address) is False,('NilTag','bit',address)
    payload=group(types[1:])
    laws=[('NilRecipe',f.eq(carrier,nil,f.app(f.const(boxing+'.Make'),zero_word,*zeros))),
        ('TagOfNil',f.eq(f.word(64)[0],observe('Tag',nil),zero_word)),
        ('PayloadOfNil',f.eq(payload,observe('PayloadVector',nil),named('NilPayload'))),
        ('IsNilOfNil',f.eq(f.boolean(),observe('IsNil',nil),f.bit(True))),
        ('IsNonNilOfNil',f.eq(f.boolean(),observe('IsNonNil',nil),f.bit(False))),
        ('CopyRecipe',f.pi('e',carrier,f.eq(carrier,copied,f.app(f.const(boxing+'.Make'),*fields))))]
    for name,ty in [('Tag',f.word(64)[0]),('PayloadVector',payload),('IsNil',f.boolean()),('IsNonNil',f.boolean())]:
        law={'Tag':'TagOfCopy','PayloadVector':'PayloadOfCopy','IsNil':'IsNilOfCopy','IsNonNil':'IsNonNilOfCopy'}[name]
        laws.append((law,f.pi('e',carrier,f.eq(ty,observe(name,copied),observe(name,e)))))
    laws.append(('HasCaseOfCopy',f.bind('pi',[('case',f.word(64)[0]),('e',carrier)],f.eq(f.boolean(),named('HasCase',f.var('case'),copied),named('HasCase',f.var('case'),e)))))
    raw_nil=f.word(64)[1];word_type=f.word(64)[0]
    laws.append(('FrozenTagIsNil',f.bind('pi',[('e',carrier),('tag_is_zero',f.eq(word_type,observe('Tag',e),raw_nil))],f.eq(f.boolean(),observe('IsNil',e),f.bit(True)))))
    laws.append(('NilIsNotNonNil',f.bind('pi',[('e',carrier),('is_nil',f.eq(f.boolean(),observe('IsNil',e),f.bit(True)))],f.eq(f.boolean(),observe('IsNonNil',e),f.bit(False)))))
    comparison=f.app(f.const('Mpk.Go.Practical.Word.W64.Equal'),f.var('case'),f.var('tag'))
    laws.append(('DifferentTagHasNoCase',f.bind('pi',[('case',word_type),('e',carrier),('tag',word_type),('same_tag',f.eq(word_type,observe('Tag',e),f.var('tag'))),('different',f.eq(f.boolean(),comparison,f.bit(False)))],f.eq(f.boolean(),named('HasCase',f.var('case'),e),f.bit(False)))))
    for i,(id,_) in enumerate(tags):
        laws.append(('CopyPayload'+str(i),f.pi('e',carrier,f.eq(types[i+1],projection(types,copied,i+1),fields[i+1]))))
        laws.append(('C'+id+'.NilHasNoCase',f.eq(f.boolean(),named('HasCase',f.const(boxing+'.C'+id+'.Tag'),nil),f.bit(False))))
    for name,ty in laws:
        d=table[module+'.'+name];assert d[1]==2
        assert term(d[2])==m.frozen_ast(ty),(name,'complete statement')
    operation_definition_count=len(definitions)
    boxed_id=source['boxed_source_type_id']
    comparison_rows=[];row_laws=[]
    if boxed_id is not None:
        boxed_id=source['boxed_source_type_id'];assert boxed_id in cases
        assert source['case_row_module']==module+'.Cases.C'+boxed_id
        def row(name,*args):return f.app(f.const(source['case_row_module']+'.'+name),*args)
        selected_tag=f.const(boxing+'.C'+boxed_id+'.Tag')
        boxed=f.app(f.const(boxing+'.C'+boxed_id+'.Box'),f.var('v'))
        row_laws=[('OwnTagMatches',f.bind('pi',[('e',carrier),('tag',word_type),('same_tag',f.eq(word_type,observe('Tag',e),f.var('tag')))],f.eq(f.boolean(),named('HasCase',f.var('tag'),e),f.bit(True))))]
        comparison_rows=[]
        for id,ordinal in tags:
            target=f.const(boxing+'.C'+id+'.Tag');expected=id==boxed_id
            row_laws.append(('TagEqual.C'+id,f.eq(f.boolean(),f.app(f.const('Mpk.Go.Practical.Word.W64.Equal'),target,selected_tag),f.bit(expected))))
            row_laws.append(('Matches.C'+id,f.pi('v',pair(boxed_id)[0],f.eq(f.boolean(),named('HasCase',target,boxed),f.bit(expected)))))
            comparison_rows.append({'target_source_type_id':id,'target_tag':ordinal,'expected_match':expected})
        for name,ty in row_laws:
            d=table[source['case_row_module']+'.'+name];assert d[1]==2
            assert term(d[2])==m.frozen_ast(ty),(name,'complete row statement')
        assert sum(c['expected_match'] for c in comparison_rows)==1
    import sys
    sys.setrecursionlimit(100000)
    corpus_path=ROOT.parent/'error-dispatch/corpus.json'
    corpus=json.loads(corpus_path.read_bytes());unit=next(r for r in corpus['cases'] if r['input']['name']==source['unit'])
    original=source['original_dispatch'];assert original in unit['native']['dispatches']
    assert original['source']==source['call']['source'] and original['method_expression']==source['call']['method_expression']
    assert [t['type_id'] for t in original['targets']]==[t['type_id'] for t in source['targets']]
    assert all(a['callee']['source']==b['source'] for a,b in zip(original['targets'],source['targets']))
    targets=source['targets'];assert [t['type_id'] for t in targets]==cases
    assert [t['name'] for t in targets]==['Error']*len(cases)
    assert all(t['source'] and type(t['declaration_node']) is int and type(t['body_node']) is int and not t['parameter_type_ids'] and len(t['result_type_ids'])==1 for t in targets)
    callsite=source['call'];assert callsite in source['all_calls']
    def call_key(c):return [c['source'],c['caller_node'],c['call_node'],c['receiver_node'],c['method_expression']]
    def target_key(t):return [t['type_id'],t['source'],t['declaration_node'],t['body_node'],t['name'],t['receiver_type_id'],t['receiver_path'],t['parameter_type_ids'],t['result_type_ids']]
    canonical=lambda x:json.dumps(x,separators=(',',':'),ensure_ascii=False).encode()
    base=hashlib.sha256(b'MPK-GO-SOURCE-ERROR-DISPATCH-PREPARATION-1\0'+canonical([parent,[call_key(c) for c in source['all_calls']],[target_key(t) for t in targets]])).hexdigest()
    suffix=hashlib.sha256(b'MPK-GO-SOURCE-ERROR-DISPATCH-ROW-PREPARATION-1\0'+base.encode()+canonical([call_key(callsite),boxed_id])).hexdigest()
    assert certificate_module==module+'.Dispatch.S'+suffix
    dispatch=certificate_module
    def dc(name,*args):return f.app(f.const(dispatch+'.'+name),*args)
    def natural(i):
        value=f.const('Std.Nat.zero')
        for _ in range(i):value=f.app(f.const('Std.Nat.succ'),value)
        return value
    def target_name(t):return 'Target.C'+t['type_id']+'.S'+t['source']+'.N'+str(t['declaration_node'])
    def selected_target(i):return dc(target_name(targets[i]))
    def tail(i,value):return dc('SelectFrom'+str(i),value)
    def matched(i,value):return named('HasCase',f.const(boxing+'.C'+cases[i]+'.Tag'),value)
    def choose(condition,yes,no):return f.app(f.const('Std.Bool.cases'),f.lam('_case',f.boolean(),f.nat()),no,yes,condition)
    definitions=[('NoTarget',f.nat(),natural(0))]
    for i,t in enumerate(targets):definitions.append((target_name(t),f.nat(),natural(i+1)))
    definitions.append(('SelectFrom'+str(len(cases)),f.pi('e',carrier,f.nat()),f.lam('e',carrier,dc('NoTarget'))))
    for i in reversed(range(len(cases))):definitions.append(('SelectFrom'+str(i),f.pi('e',carrier,f.nat()),f.lam('e',carrier,choose(matched(i,e),selected_target(i),tail(i+1,e)))))
    definitions.append(('Select',f.pi('e',carrier,f.nat()),f.lam('e',carrier,tail(0,e))))
    selector_laws=[]
    for i in range(len(cases)):
        selector_laws.append(('SelectTrue'+str(i),f.bind('pi',[('e',carrier),('matched',f.eq(f.boolean(),matched(i,e),f.bit(True)))],f.eq(f.nat(),tail(i,e),selected_target(i)))))
        selector_laws.append(('SelectFalse'+str(i),f.bind('pi',[('e',carrier),('not_matched',f.eq(f.boolean(),matched(i,e),f.bit(False))),('result',f.nat()),('tail',f.eq(f.nat(),tail(i+1,e),f.var('result')))],f.eq(f.nat(),tail(i,e),f.var('result')))))
    value=f.app(f.const(boxing+'.C'+boxed_id+'.Box'),f.var('v')) if boxed_id is not None else named('Nil')
    args=[('v',pair(boxed_id)[0])] if boxed_id is not None else []
    for i in range(len(cases)+1):
        result=selected_target(cases.index(boxed_id)) if boxed_id is not None and i<=cases.index(boxed_id) else dc('NoTarget')
        selector_laws.append(('Suffix'+str(i)+'OfValue',f.bind('pi',args,f.eq(f.nat(),tail(i,value),result))))
    result=selected_target(cases.index(boxed_id)) if boxed_id is not None else dc('NoTarget')
    selector_laws.append(('SelectedOfBox' if boxed_id is not None else 'SelectedOfNil',f.bind('pi',args,f.eq(f.nat(),dc('Select',value),result))))
    for name,ty,body in definitions:
        d=table[dispatch+'.'+name];assert d[1]==1
        assert term(d[2])==m.frozen_ast(ty),(name,'selector type')
        assert term(d[3])==m.frozen_ast(body),(name,'selector body')
    for name,ty in selector_laws:
        d=table[dispatch+'.'+name];assert d[1]==2
        assert term(d[2])==m.frozen_ast(ty),(name,'complete selector statement')
    # A separate lazy observation of the ACTUAL decoded Select/Box/Zero DAG.
    # No expected function body or expected target table is used as evaluator.
    class Thunk:
        def __init__(self,run):self.run=run;self.done=False;self.value=None
        def get(self):
            if not self.done:self.value=self.run();self.done=True
            return self.value
    def application(function,arg):
        if function==('succ',):return arg.get()+1
        assert function[0]=='closure',function
        return lazy(function[1],function[2]+(arg,))
    def lazy(i,scope=()):
        tag,*args=terms[i]
        if tag==1:return scope[-1-args[0]].get()
        if tag==2:
            name=names[declarations[args[0]][0]]
            if name=='Std.Bool.false':return False
            if name=='Std.Bool.true':return True
            if name=='Std.Nat.zero':return 0
            if name=='Std.Nat.succ':return ('succ',)
            d=declarations[args[0]];assert d[1]==1,name
            return lazy(d[3])
        if tag==4:return ('closure',args[1],scope)
        assert tag==3
        head=terms[args[0]]
        if head[0]==2:
            name=names[declarations[head[1]][0]]
            if name=='Std.Bool.cases':
                assert len(args[1])==4;condition=lazy(args[1][3],scope);assert type(condition) is bool
                return lazy(args[1][2 if condition else 1],scope)
            if name=='Std.Bool.rec':
                assert len(args[1])==3;condition=lazy(args[1][2],scope);assert type(condition) is bool
                return lazy(args[1][1 if condition else 0],scope)
        function=lazy(args[0],scope)
        for a in args[1]:function=application(function,Thunk(lambda a=a:lazy(a,scope)))
        return function
    def definition(name):
        d=table[name];assert d[1]==1
        return lazy(d[3])
    if boxed_id is None:actual_value=Thunk(lambda:definition(module+'.Nil'))
    else:
        actual_value=Thunk(lambda:application(definition(boxing+'.C'+boxed_id+'.Box'),Thunk(lambda:definition(parent+'.T'+boxed_id+'.Zero'))))
    observed_target=application(definition(dispatch+'.Select'),actual_value)
    expected_target=0 if boxed_id is None else cases.index(boxed_id)+1
    assert type(observed_target) is int and observed_target==expected_target==source['expected_selected_ordinal']
    return {'certificate':str(path),'raw_sha256':hashlib.sha256(raw).hexdigest(),'cases':results,'empty_U_has_no_constructor':not bool(tags),'full_source_U_preserved':True,'declarations':len(declarations),'terms':len(terms),'bytes':len(raw),'actual_tag_bit_observations':64*len(cases),'complete_operation_definitions_compared':operation_definition_count,'complete_operation_theorem_types_compared':len(laws),'all_copy_payload_fields_preserved':len(cases),'actual_nil_tag_bit_observations':64,'boxed_source_type_id':boxed_id,'case_comparisons':comparison_rows,'complete_case_row_theorem_types_compared':len(row_laws),'selector_definitions_compared':len(definitions),'selector_statements_compared':len(selector_laws),'actual_selected_target_observation':observed_target,'original_call_site':callsite,'complete_source_targets':targets}
