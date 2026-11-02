"""Independently inspect complete source relations and decoded leaf observations."""
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
    wire_module=module
    module=source['operations_module']
    assert module==source['boxing_module']+'.Operations'
    error=source['source_error_type_id'];assert keys[error]['kind']=='builtin' and keys[error]['name']=='error'
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
    operation_definition_count=len(definitions);operation_statement_count=len(laws)
    # Complete closed source-carrier observations, independently reconstructed.
    relation_module=source['relation_module']
    assert relation_module==module+'.ValueRelations'
    underlying=dict(source['underlying'])
    def sr(id,name,*xs):return f.app(f.const(relation_module+'.T'+id+'.'+name),*xs)
    def vector(bits,part):return ('vector',bits,part)
    def atom(t):return ('equal',t)
    def reference(id):return ('source',id)
    def word_view(width):return vector((width-1).bit_length(),atom(f.boolean()))
    def view(id):
        t=keys[id];kind=t['kind']
        if kind=='nominal':return reference(underlying[id])
        if kind=='builtin':
            if t['name']=='bool':return atom(f.boolean())
            if t['name']=='string':return ('product',[word_view(64),vector(8,word_view(8))])
            if t['name']=='error':return ('product',[word_view(64),*[reference(c) for c in cases]])
            assert t['name'] in ['int','int8','int16','int32','int64','uint','uint8','uint16','uint32','uint64']
            return word_view(t['width'])
        if kind=='array':return atom(f.boolean()) if t['length']==0 else vector((t['length']-1).bit_length(),reference(t['element']))
        if kind=='struct':return ('product',[reference(field['type_id']) for field in t['fields']])
        assert kind=='slice'
        return ('product',[atom(f.boolean()),atom(f.nat()),vector(4,('product',[word_view(8),word_view(64)])),word_view(64),word_view(64),word_view(64)])
    def view_carrier(v):
        if v[0]=='equal':return v[1]
        if v[0]=='source':return pair(v[1])[0]
        if v[0]=='vector':return f.bind('pi',[(f'_a{i}',f.boolean()) for i in range(v[1])],view_carrier(v[2]))
        return group([view_carrier(p) for p in v[1]])
    def fields(v,x):return [projection([view_carrier(p) for p in v],x,i) for i in range(len(v))]
    def relation(v,x,y,depth=0):
        if v[0]=='equal':return f.eq(v[1],x,y)
        if v[0]=='source':return sr(v[1],'Related',x,y)
        if v[0]=='vector':
            args=[(f'_observation{depth}_{i}',f.boolean()) for i in range(v[1])]
            indices=[f.var(n) for n,_ in args]
            return f.bind('pi',args,relation(v[2],f.app(x,*indices),f.app(y,*indices),depth+1))
        if not v[1]:return f.eq(f.boolean(),x,y)
        xs,ys=fields(v[1],x),fields(v[1],y)
        return group([relation(p,xs[i],ys[i],depth) for i,p in enumerate(v[1])])
    definition_count=statement_count=0
    for id in sorted(keys):
        v=view(id);carrier=pair(id)[0];x,y,z=map(f.var,['x','y','z'])
        observed=lambda a:sr(id,'View',a)
        raw=relation(v,observed(x),observed(y))
        definitions=[('View',f.pi('x',carrier,view_carrier(v)),f.lam('x',carrier,x)),('Related',f.bind('pi',[('x',carrier),('y',carrier)],{'kind':'sort0'}),f.bind('lam',[('x',carrier),('y',carrier)],raw))]
        for name,t,b in definitions:
            d=table[relation_module+'.T'+id+'.'+name];assert d[1]==1
            assert term(d[2])==m.frozen_ast(t),(id,name,'closed type')
            assert term(d[3])==m.frozen_ast(b),(id,name,'closed body')
        args=[('x',carrier),('y',carrier)]
        laws=[('Observations',f.bind('pi',args+[('same',sr(id,'Related',x,y))],raw)),('Reflexive',f.pi('x',carrier,sr(id,'Related',x,x))),('Symmetric',f.bind('pi',args+[('same',sr(id,'Related',x,y))],sr(id,'Related',y,x))),('Transitive',f.bind('pi',args+[('z',carrier),('xy',sr(id,'Related',x,y)),('yz',sr(id,'Related',y,z))],sr(id,'Related',x,z))),('OfEquality',f.bind('pi',args+[('equal',f.eq(carrier,x,y))],sr(id,'Related',x,y))),('ZeroRelated',sr(id,'Related',pair(id)[1],pair(id)[1]))]
        for name,t in laws:
            d=table[relation_module+'.T'+id+'.'+name];assert d[1]==2
            assert term(d[2])==m.frozen_ast(t),(id,name,'complete closed statement')
        definition_count+=len(definitions);statement_count+=len(laws)
    erel=lambda a,b:sr(error,'Related',a,b)
    row=lambda n,*xs:f.app(f.const(wire_module+'.'+n),*xs)
    assert wire_module==relation_module+('.C'+source['boxed_source_type_id'] if source['boxed_source_type_id'] is not None else '.Nil')
    error_carrier=pair(error)[0]
    row_laws=[('CopyTagRelated',f.pi('e',error_carrier,relation(word_view(64),observe('Tag',copied),observe('Tag',e))))]
    for i,id in enumerate(cases):row_laws.append(('CopyFieldRelated.C'+id,f.pi('e',error_carrier,sr(id,'Related',projection(types,copied,i+1),projection(types,e,i+1)))))
    row_laws.extend([('CopyRelated',f.pi('e',error_carrier,erel(copied,e))),('CopyTwiceRelated',f.pi('e',error_carrier,erel(named('Copy',copied),e))),('NilRelated',erel(nil,nil))])
    boxed_id=source['boxed_source_type_id']
    if boxed_id is not None:
        x,y=f.var('x'),f.var('y');bx=f.app(f.const(boxing+'.C'+boxed_id+'.Box'),x);by=f.app(f.const(boxing+'.C'+boxed_id+'.Box'),y)
        args=[('x',pair(boxed_id)[0]),('y',pair(boxed_id)[0]),('same',sr(boxed_id,'Related',x,y))]
        all_views=[word_view(64),*[reference(c) for c in cases]]
        for i,v in enumerate(all_views):row_laws.append(('BoxFieldRelated'+str(i),f.bind('pi',args,relation(v,projection(types,bx,i),projection(types,by,i)))))
        row_laws.append(('BoxRelated',f.bind('pi',args,erel(bx,by))))
        row_laws.append(('ActiveRelated',f.bind('pi',[('x',pair(boxed_id)[0]),('y',pair(boxed_id)[0]),('same',erel(bx,by))],sr(boxed_id,'Related',x,y))))
    for name,t in row_laws:
        d=table[wire_module+'.'+name];assert d[1]==2
        assert term(d[2])==m.frozen_ast(t),(name,'whole source relation row statement')
    leaf_rows=observe_leaves(terms,declarations,names,table,source,view) if boxed_id is None else []
    return {'decoded_leaf_observations':leaf_rows,'decoded_leaf_changes_observed':sum(r['every_encoded_leaf_change_is_observed'] for r in leaf_rows),'certificate':str(path),'raw_sha256':hashlib.sha256(raw).hexdigest(),'closed_relation_definitions':definition_count,'closed_relation_statements':statement_count,'whole_row_statements':len(row_laws),'all_source_types':len(keys),'cases':results,'empty_U_has_no_constructor':not bool(tags),'full_source_U_preserved':True,'declarations':len(declarations),'terms':len(terms),'bytes':len(raw),'actual_tag_bit_observations':64*len(cases),'complete_operation_definitions_compared':operation_definition_count,'complete_operation_theorem_types_compared':operation_statement_count,'all_copy_payload_fields_preserved':len(cases),'actual_nil_tag_bit_observations':64}


def observe_leaves(terms,declarations,names,table,source,view):
    """Observe each actual decoded relation at every encoded scalar leaf.

    Values are full carrier functions, including array padding and inactive
    error fields. This evaluator observes ordinary definitions, never proofs.
    A changed leaf must yield a false Eq at that exact path; identical decoded
    source zeros must yield true there. Mathematical truth is checked by the
    two native cores separately.
    """
    def apply(value,*arguments):
        for a in arguments:
            if callable(value):value=value(a)
            elif value[0]=='closure':value=evaluate(value[1],value[2]+(a,))
            elif value[0]=='quantifier':value=evaluate(value[1],value[2]+(a,))
            else:raise AssertionError(('nonfunction observation',value))
        return value
    def curry(n,fn,args=()):return fn(*args) if n==0 else lambda a:curry(n-1,fn,args+(a,))
    @functools.lru_cache(None)
    def named(name):
        if name=='Std.Bool.false':return False
        if name=='Std.Bool.true':return True
        if name=='Std.Bool':return ('type','bool')
        if name=='Std.Nat':return ('type','nat')
        if name=='Std.Nat.zero':return ('natural',0)
        if name=='Std.Nat.succ':return lambda a:('natural',a[1]+1)
        if name=='Std.Eq':return curry(3,lambda _,x,y:('relation-eq',x,y))
        d=table[name];assert d[1]==1,('definition observation',name,d[1])
        return evaluate(d[3])
    def evaluate(i,scope=()):
        tag,*a=terms[i]
        if tag==0:return ('sort',)
        if tag==1:return scope[-1-a[0]]
        if tag==2:return named(names[declarations[a[0]][0]])
        if tag in [4,5]:return ('closure' if tag==4 else 'quantifier',a[1],scope)
        if tag==6:return evaluate(a[2],scope+(evaluate(a[1],scope),))
        assert tag==3
        head=terms[a[0]]
        if head[0]==2:
            n=names[declarations[head[1]][0]]
            if n in ['Std.Bool.cases','Std.Bool.rec','Std.Bool.recType']:
                xs=a[1];condition=evaluate(xs[-1],scope);assert type(condition) is bool
                offset=1 if n=='Std.Bool.cases' else 0
                return evaluate(xs[offset+int(condition)],scope)
        return apply(evaluate(a[0],scope),*[evaluate(j,scope) for j in a[1]])
    def bits(index,n):return [bool(index&(1<<j)) for j in range(n-1,-1,-1)]
    def get_product(value,count,index):
        if count==1:return value
        half=count//2
        selected=apply(value,index>=half)
        return get_product(selected,half,index) if index<half else get_product(selected,count-half,index-half)
    def make_product(values):
        if not values:return False
        if len(values)==1:return values[0]
        half=len(values)//2
        return lambda right:make_product(values[half:] if right else values[:half])
    def paths(v):
        if v[0]=='equal':yield ();return
        if v[0]=='source':yield from paths(view(v[1]));return
        if v[0]=='vector':
            for address in range(1<<v[1]):
                for rest in paths(v[2]):yield (address,)+rest
            return
        if not v[1]:yield ();return
        for i,p in enumerate(v[1]):
            for rest in paths(p):yield (i,)+rest
    def update(v,value,path):
        if v[0]=='source':return update(view(v[1]),value,path)
        if v[0]=='equal' or (v[0]=='product' and not v[1]):
            assert not path
            return not value if type(value) is bool else ('natural',value[1]+1)
        target,*rest=path
        if v[0]=='vector':
            def leaf(*indices):
                address=0
                for digit in indices:address=address*2+int(digit)
                current=apply(value,*indices)
                return update(v[2],current,tuple(rest)) if address==target else current
            return curry(v[1],leaf)
        values=[get_product(value,len(v[1]),i) for i in range(len(v[1]))]
        values[target]=update(v[1][target],values[target],tuple(rest))
        return make_product(values)
    def observation(v,value,path):
        if v[0]=='source':return observation(view(v[1]),value,path)
        if v[0]=='equal' or (v[0]=='product' and not v[1]):
            assert not path and value[0]=='relation-eq'
            return value[1]==value[2]
        i,*rest=path
        if v[0]=='vector':return observation(v[2],apply(value,*bits(i,v[1])),tuple(rest))
        return observation(v[1][i],get_product(value,len(v[1]),i),tuple(rest))
    rows=[]
    for t in source['types']:
        id=t['id'];v=view(id);zero=named(source['source_layout_module']+'.T'+id+'.Zero')
        relate=named(source['relation_module']+'.T'+id+'.Related')
        same=apply(relate,zero,zero);count=0
        for path in paths(v):
            assert observation(v,same,path) is True,(id,'identical zero',path)
            changed=update(v,zero,path)
            actual=apply(relate,zero,changed)
            assert observation(v,actual,path) is False,(id,'lost changed encoded leaf',path)
            count+=1
        rows.append({'source_type_id':id,'kind':t['kind'],'identical_original_zero_leaves_observed':count,'every_encoded_leaf_change_is_observed':count})
    return rows
