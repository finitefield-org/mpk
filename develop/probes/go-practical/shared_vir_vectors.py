"""Reconstructible five-profile modules, complete Go closures and corruptions."""
import copy
import json
from body_canonical import canonical,digest
from context_protocol import legacy_projection
from run_proof_pilot import ROOT
from foundation_limits_vectors import linked_contexts
from contract_wire_vectors import source_cases
from shared_project import Project,ProjectModel
from shared_vir import contract,go_contract,body,go_program,module,KINDS,legacy_query,ModuleValidator
from vir_body_build import Builder

GO_FIXTURE=ROOT/'fixtures/vir-go/frontend/basic-arith/vir.json'
RUST_FIXTURE=ROOT/'fixtures/rust-basic/positive/module-calls/artifacts/vir.json'


def retained_modules():
    cs=linked_contexts()['contexts'];go=json.loads(GO_FIXTURE.read_bytes());rust=json.loads(RUST_FIXTURE.read_bytes());rows=[]
    for name in ('go_fixed','rust64','rust32','csharp','java'):
        context=cs[name];units=copy.deepcopy((rust if name.startswith('rust') else go)['units'])
        if name in {'csharp','java'}:
            fn=next(f for f in units[0]['functions'] if f['name']=='BoolAnd');unit='policy'
            fn['id']='Example.Policy::And(bool,bool)->bool' if name=='csharp' else 'example.Policy::and(boolean,boolean)->boolean'
            fn['name']='And' if name=='csharp' else 'and';fn['unit_id']=unit
            units=[{'id':unit,'name':unit,'type_decls':[],'const_decls':[],'functions':[fn]}]
        hashes={}
        for u in units:
            for f in u['functions']:
                old=f['contracts'];value={k:copy.deepcopy(old[k]) for k in ('requires','ensures','modifies','panic','termination','loops')}
                value.update(kind=KINDS[context['semantic_profile']],unit_id=u['id'],function_id=f['id'])
                f['contracts']=contract(context,value);hashes[f['id']]=f['contracts']['contract_hash']
        for u in units:
            for f in u['functions']:
                for b in f['blocks']:
                    for op in b['instructions']:
                        if op['kind']=='CallStatic':op['contract_hash']=hashes[op['function']]
        rows.append({'id':name,'module':module(context,{'kind':KINDS[context['semantic_profile']],'units':units})})
    return rows


def baseline_queries():
    """Original Go/Rust64 bytes; independently formed old-family fixtures otherwise."""
    cs=linked_contexts()['contexts'];go=json.loads(GO_FIXTURE.read_bytes());rust=json.loads(RUST_FIXTURE.read_bytes());rows=[]
    for name in ('go_fixed','rust64','rust32','csharp','java'):
        if name in {'go_fixed','rust64'}:
            rows.append({'id':name,'query':{'original_vir':go if name=='go_fixed' else rust}});continue
        units=copy.deepcopy((rust if name=='rust32' else go)['units'])
        if name in {'csharp','java'}:
            f=next(f for f in units[0]['functions'] if f['name']=='BoolAnd')
            f['id']='Example.Policy::And(bool,bool)->bool' if name=='csharp' else 'example.Policy::and(boolean,boolean)->boolean'
            f['name']='And' if name=='csharp' else 'and';f['unit_id']='policy'
            units=[{'id':'policy','name':'policy','type_decls':[],'const_decls':[],'functions':[f]}]
        for unit in units:
            for f in unit['functions']:
                c=f['contracts'];f['contracts']={k:copy.deepcopy(c[k]) for k in ('requires','ensures','modifies','panic','termination','loops')}
                f['contracts'].update(unit_id=unit['id'],function_id=f['id'])
        rows.append({'id':name,'query':{'context':legacy_projection(cs[name])['context'],'units':units}})
    return rows


def project_spec(overlays=True):
    roots=sorted([{'package':'fixture/policy','function':n,'receiver':None} for n in ('Quote','InputErrors')],key=canonical)
    records=[]
    if overlays:
        for id_,name in (('missing-invariants-partial','Scopes'),('nested-and-tail-error-entry-sets','InputErrors')):
            raw=copy.deepcopy(next(r['raw'] for r in source_cases() if r['id']==id_))
            if name=='Scopes':
                raw['function']['function']='Quote';raw['loops']=[];raw['error_inputs']=[];raw['requires']=[];raw['ensures']=[{'op':'bool.literal','value':True}]
                name='Quote'
            records.append({'path':'contracts/'+name+'.json','raw_hex':(canonical(raw)+b'\n').hex()})
    return {'schema':'mpk.go.module_model_input.v0','roots':roots,'universe':['fixture/policy.RejectCode'],'overlays':sorted(records,key=lambda r:r['path'])}


def go_module(spec):
    p=Project(spec);quote=p.resolve({'package':'fixture/policy','function':'Quote','receiver':None})
    empty=p.resolve({'package':'fixture/policy','function':'InputErrors','receiver':None})
    code=p.resolve({'package':'fixture/policy','function':'Error','receiver':{'package':'fixture/policy','type_name':'RejectCode'}})
    bodies=[]
    for decl in (quote,empty,code):
        m=ProjectModel(p,decl);b=Builder(m)
        if decl==quote:
            inputs=[__import__('operation_signatures').source(m.id(x['type'])) for x in m.scope.function['inputs']]+[__import__('operation_signatures').STATE]
            r=b.region(inputs);receiver=r.zero(m.id('fixture/policy.RejectCode'));r.apply(19,{'callee':code},[receiver,r.state]);r.return_zero()
        elif decl==empty:
            inputs=[__import__('operation_signatures').source(m.id(x['type'])) for x in m.scope.function['inputs']]+[__import__('operation_signatures').STATE]
            r=b.region(inputs);r.normal()
        else:
            inputs=[__import__('operation_signatures').source(m.id(x['type'])) for x in m.scope.function['inputs']]+[__import__('operation_signatures').STATE]
            r=b.region(inputs);r.return_zero()
        bodies.append(body(p,decl,r.r,b.definitions()))
    program=go_program(p,bodies);return module(p.scope(quote).context,program)


def bases():
    rows=retained_modules()
    for name,enabled in (('go-two-overlays',True),('go-default-contracts',False)):
        spec=project_spec(enabled);rows.append({'id':name,'project_spec':spec,'module':go_module(spec)})
    return rows


def rehash(value):
    value['vir_hash']=digest('MPK-VIR-3.0',{k:v for k,v in value.items() if k!='vir_hash'});return value


def specs():
    rows=[]
    for name in ('go_fixed','rust64','rust32','csharp','java','go-two-overlays','go-default-contracts'):
        rows += [{'id':name+'-'+kind,'base':name,'mutation':kind,'expected_accepted':kind=='same'}
                 for kind in ('same','extra-root','old-schema','wrong-profile-contract','wrong-arm','wrong-context','bad-hash')]
    for name in ('missing-callee','extra-unreachable','cycle','stale-capture','reordered-contracts','forged-scope','stale-body-contract','wrong-default-origin'):
        rows.append({'id':'go-'+name,'base':'go-two-overlays','mutation':name,'expected_accepted':False})
    for profile in ('go_fixed','rust64','csharp','java'):
        for name in ('unknown-instruction','wrong-operand-type','stale-contract','cross-contract-context'):
            rows.append({'id':profile+'-'+name,'base':profile,'mutation':name,'expected_accepted':False})
    return rows


def altered(row,base):
    value=copy.deepcopy(base['module']);kind=row['mutation']
    if kind=='same':return value
    if kind=='extra-root':value['caller_verified']=True
    elif kind=='old-schema':value['schema']='mpk.vir.v1'
    elif kind=='wrong-profile-contract':value['profile_contract']='mpk.profile.vir.go_practical.v1'
    elif kind=='wrong-arm':value['program']['kind']='caller_extension'
    elif kind=='wrong-context':value['semantic_context']['source_language']='unknown'
    elif kind=='bad-hash':value['vir_hash']='0'*64;return value
    elif kind in {'missing-callee','extra-unreachable','cycle','stale-capture','reordered-contracts','forged-scope','stale-body-contract','wrong-default-origin'}:
        p=value['program'];bodies=p['function_bodies']
        if kind=='missing-callee':
            target=next(d['function_decl_id'] for b in bodies for s in b['obligation_sites'] for d in s['dependencies'] if 'function_decl_id' in d)
            p['function_bodies']=[b for b in bodies if b['function_decl_id']!=target]
            value['program']=go_program(Project(base['project_spec']),p['function_bodies'])
        elif kind=='extra-unreachable':
            project=Project(base['project_spec']);decl=project.resolve({'package':'fixture/policy','function':'Scopes','receiver':None});m=ProjectModel(project,decl);builder=Builder(m)
            ins=[__import__('operation_signatures').source(m.id(x['type'])) for x in m.scope.function['inputs']]+[__import__('operation_signatures').STATE];r=builder.region(ins);r.return_zero()
            p['function_bodies'].append(body(project,decl,r.r,builder.definitions()));p['function_bodies'].sort(key=lambda b:b['function_decl_id'])
            value['program']=go_program(project,p['function_bodies'])
        elif kind=='cycle':
            # The complete candidate body set is cyclic,not merely a stale hash.
            project=Project(base['project_spec']);target=next(b['function_decl_id'] for b in bodies if b['body']['nodes'] and len(b['entry_arguments'])==1)
            m=ProjectModel(project,target);builder=Builder(m);ins=[__import__('operation_signatures').source(m.id(x['type'])) for x in m.scope.function['inputs']]+[__import__('operation_signatures').STATE]
            r=builder.region(ins);r.apply(19,{'callee':target},[r.r['parameters'][0]['id'],r.state]);r.return_zero();replacement=body(project,target,r.r,builder.definitions())
            p['function_bodies']=[replacement if b['function_decl_id']==target else b for b in bodies]
        elif kind=='stale-capture':p['capture_context']['objects']=[]
        elif kind=='reordered-contracts':p['contracts'].reverse()
        elif kind=='forged-scope':p['source_scopes']['functions'][0]['signature_id']='0'*64
        elif kind=='stale-body-contract':bodies[0]['contract_hash']='0'*64
        else:
            c=next(c for c in p['contracts'] if c['contract']['value']['origin']['kind']=='default');c['contract']['value']['origin']={'kind':'source_overlay','object_id':'0'*64}
    else:
        f=value['program']['units'][0]['functions'][0]
        if kind=='unknown-instruction':f['blocks'][0]['instructions'].append({'kind':'CallerInstruction'})
        elif kind=='wrong-operand-type':f['params'][0]['type']={'kind':'array','length':1,'element':{'kind':'bool'}}
        elif kind=='stale-contract':f['contracts']['contract_hash']='0'*64
        elif kind=='cross-contract-context':f['contracts']['semantic_context']['source_language']='unknown'
        else:raise AssertionError(kind)
    return rehash(value)


def queries(rows,data):
    by_id={b['id']:b for b in data};out={}
    for row in rows:
        b=by_id[row['base']];value=altered(row,b)
        if value['program'].get('kind')=='go_practical':continue
        try:q=legacy_query(value['semantic_context'],value['program']['units'])
        except (ValueError,KeyError):continue
        out[canonical(q)]=q
    return [out[k] for k in sorted(out)]
