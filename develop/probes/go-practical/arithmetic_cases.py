"""Closed contract observations,component rejections and actual proof controls."""
import copy
from arithmetic_terms import *
from logical_elaboration import type_ref,elaborate,ArithmeticEvaluator,ModelEvaluationPending
from ordinary_arithmetic import plan


def lit(value,ty=None):
    ty=ty or ('Bool' if type(value) is bool else 'Nat' if value>=0 else 'Int')
    return {'op':{'Bool':'bool.literal','Nat':'nat.literal','Int':'int.literal'}[ty],'type':type_ref(ty),'payload':{'value':value if ty=='Bool' else str(value)}}
def node(op,ty,*args):return {'op':op,'type':type_ref(ty),'payload':{'args':list(args)}}
def bound(depth=0):return {'op':'bound','type':type_ref('Nat'),'payload':{'depth':depth}}
def quantified(op,lower,upper,body):return {'op':op,'type':type_ref('Nat' if op=='nat.sum' else 'Int' if op=='int.sum' else 'Bool'),'payload':{'lower':lower,'upper':upper,'body':body}}


def expressions():
    out=[]
    def add_(e,large=False):out.append({'id':'logical-'+str(len(out)).zfill(3),'expression':e,'large':large})
    for b in (False,True):add_(lit(b));add_(node('bool.not','Bool',lit(b)))
    for op in ('bool.and','bool.or','bool.implies','eq','ne'):
        for a in (False,True):
            for b in (False,True):add_(node(op,'Bool',lit(a),lit(b)))
    for n in range(7):add_(lit(n))
    for n in (-6,-1,0,1,6):add_(lit(n,'Int'));add_(node('int.neg','Int',lit(n,'Int')))
    for n in (0,1,6):add_(node('nat.to_int','Int',lit(n)))
    for ty,prefix,pairs in (('Nat','nat.',[(0,0),(0,5),(5,0),(2,3),(6,6)]),('Int','int.',[(-3,2),(3,-2),(-3,-2),(0,0),(5,-5)])):
        for op in ('add','sub','mul'):
            for a,b in pairs:add_(node(prefix+op,ty,lit(a,ty),lit(b,ty)))
    for ty,pairs in (('Nat',[(2,3),(3,2),(3,3)]),('Int',[(-3,2),(2,-3),(-3,-3)])):
        for op in ('eq','ne','lt','le','gt','ge'):
            for a,b in pairs:add_(node(op,'Bool',lit(a,ty),lit(b,ty)))
    redundant=node('int.add','Int',lit(3,'Int'),lit(-3,'Int'))
    for op in ('eq','ne','lt','le','gt','ge'):add_(node(op,'Bool',redundant,lit(0,'Int')))
    for ty,a,b in (('Bool',False,True),('Nat',1,5),('Int',-3,3)):
        for condition in (False,True):add_(node('if',ty,lit(condition),lit(a,ty),lit(b,ty)))
    for lower,upper in ((0,0),(3,1),(1,5),(0,4)):
        for op in ('forall','exists','nat.sum','int.sum'):
            body=node('lt','Bool',bound(),lit(upper)) if op=='forall' else node('eq','Bool',bound(),lit(1)) if op=='exists' else node('nat.mul','Nat',bound(),bound()) if op=='nat.sum' else node('int.sub','Int',node('nat.to_int','Int',bound()),lit(2,'Int'))
            add_(quantified(op,lit(lower),lit(upper),body))
    nested=quantified('nat.sum',lit(0),lit(4),quantified('nat.sum',lit(0),bound(),node('nat.add','Nat',bound(1),bound())))
    add_(nested)
    for n in ((1<<64)+7,10**70+19):
        for ty in ('Nat','Int'):add_(lit(n,ty),True)
        add_(lit(-n,'Int'),True);add_(node('nat.mul','Nat',lit(n),lit(3)),True)
        add_(node('int.sub','Int',lit(n,'Int'),lit(-n,'Int')),True)
    return out


def oracle(e,scope=()):
    op=e['op'];p=e['payload']
    if op.endswith('.literal'):return p['value'] if op=='bool.literal' else int(p['value'])
    if op=='bound':return scope[-1-p['depth']]
    if op in {'forall','exists','nat.sum','int.sum'}:
        values=[oracle(p['body'],scope+(n,)) for n in range(oracle(p['lower'],scope),oracle(p['upper'],scope))]
        return all(values) if op=='forall' else any(values) if op=='exists' else sum(values)
    a=[oracle(x,scope) for x in p['args']]
    if op=='if':return a[1] if a[0] else a[2]
    if op=='bool.not':return not a[0]
    if op=='bool.and':return a[0] and a[1]
    if op=='bool.or':return a[0] or a[1]
    if op=='bool.implies':return not a[0] or a[1]
    if op=='int.neg':return -a[0]
    if op=='nat.to_int':return a[0]
    if op in {'eq','ne','lt','le','gt','ge'}:return {'eq':a[0]==a[1],'ne':a[0]!=a[1],'lt':a[0]<a[1],'le':a[0]<=a[1],'gt':a[0]>a[1],'ge':a[0]>=a[1]}[op]
    return a[0]+a[1] if op.endswith('.add') else a[0]*a[1] if op.endswith('.mul') else max(0,a[0]-a[1]) if op=='nat.sub' else a[0]-a[1]


def compilations(rows):return [{'id':r['id'],'program':elaborate(r['expression'])} for r in rows]
def compiled_plan(base,compiled):
    out=copy.deepcopy(base);names={d['name']:d for d in out['declarations']}
    for row in compiled:
        for d in row['program']['declarations']:
            if d['name'] in names:
                assert names[d['name']]==d,'identical literal/value definition reuse'
            else:names[d['name']]=d;out['declarations'].append(d)
    out['module']=MODULE+'.ClosedLogical';return out


def expected(rows):
    return [{'id':r['id'],'type':r['expression']['type']['name'],'value':oracle(r['expression']) if r['expression']['type']['name']=='Bool' else str(oracle(r['expression']))} for r in rows]
def observations(rows,compiled,program):
    ev=ArithmeticEvaluator(program);out=[]
    for r,c in zip(rows,compiled):
        ty=c['program']['type']['name']
        try:value=ev.observe(ev.invoke(c['program']['value_name']),ty);result={'id':r['id'],'status':'evaluated','type':ty,'value':value if ty=='Bool' else str(value)}
        except ModelEvaluationPending:result={'id':r['id'],'status':'model_resource_pending'}
        assert (result['status']=='model_resource_pending')==r['large'],'exact private model evaluation boundary'
        out.append(result)
    return out


MUTATIONS=('bad-add-comm','unrelated-equality-premise','positive-only-int-equality','incomplete-int-multiply','false-empty-forall','inclusive-upper-bound','unrelated-int-order-premise','false-clause-proof')
def negative_plan(base,compiled,kind):
    p=copy.deepcopy(compiled if kind=='false-clause-proof' else base);p['module']+='.'+kind.replace('-','_')
    rows={d['name']:d for d in p['declarations']}
    if kind=='false-clause-proof':
        false=elaborate(lit(False));name=MODULE+'.FalseClaim'
        p['declarations'].append({'name':name,'kind':'theorem','ty':false['statement'],'value':refl(boolean(),bit(True))});return p
    name={'bad-add-comm':N+'AddComm','unrelated-equality-premise':N+'EqualReflect','positive-only-int-equality':I+'Equal','incomplete-int-multiply':I+'Mul','false-empty-forall':MODULE+'.Range.FoldForall','inclusive-upper-bound':MODULE+'.Range.Forall','unrelated-int-order-premise':I+'LessRespects'}[kind]
    d=rows[name];d['ty']=copy.deepcopy(d['ty']);d['value']=copy.deepcopy(d['value'])
    if kind=='bad-add-comm':
        ty=d['ty']
        while ty['kind']=='pi':ty=ty['body']
        ty['arguments'][2]=var('a')
    elif kind in {'unrelated-equality-premise','unrelated-int-order-premise'}:
        ty=d['ty']
        while ty['kind']=='pi' and ty['name'] not in {'_true','_xu'}:ty=ty['body']
        ty['domain']=eq(boolean(),bit(True),bit(True))
    elif kind=='positive-only-int-equality':d['value']=bind('lam',[('x',pi('_part',boolean(),nat())),('y',pi('_part',boolean(),nat()))],nc('Equal',ic('Positive',var('x')),ic('Positive',var('y'))))
    elif kind=='incomplete-int-multiply':d['value']=bind('lam',[('x',pi('_part',boolean(),nat())),('y',pi('_part',boolean(),nat()))],ic('Make',mul(ic('Positive',var('x')),ic('Positive',var('y'))),zero()))
    elif kind=='false-empty-forall':
        body=d['value']
        while body['kind']=='lam':body=body['body']
        body['arguments'][1]=bit(False)
    else:
        body=d['value']
        while body['kind']=='lam':body=body['body']
        body['arguments'][1]=succ(body['arguments'][1])
    return p


def vectors():
    rows=[]
    def add_(name,e,expected='configuration_rejection',**extra):rows.append({'id':name,'expression':e,'expected':expected,**extra})
    add_('typed-false-unproved',lit(False),'typed_logical_unproved')
    e=lit(True);e['trusted']=True;add_('extra-root',e)
    e=lit(True);e['type']=type_ref('Nat');add_('crossed-result-type',e)
    e=lit(0);e['op']='nat.divide';add_('unknown-head',e)
    e=lit(True);e['payload']['proved']=True;add_('extra-payload',e)
    for name,ty,value in (('json-number','Nat',3),('negative-nat','Nat','-1'),('negative-zero','Int','-0'),('leading-zero','Nat','01')):
        e=lit(0,ty);e['payload']['value']=value;add_(name,e)
    add_('free-bound',bound())
    add_('bool-bound-index',bound(False))
    add_('wrong-upper',quantified('nat.sum',lit(0),lit(-1,'Int'),bound()))
    add_('wrong-sum-body',quantified('nat.sum',lit(0),lit(3),lit(True)))
    add_('wrong-if-guard',node('if','Nat',lit(0),lit(1),lit(2)))
    add_('mixed-if-branches',node('if','Nat',lit(True),lit(1),lit(2,'Int')))
    add_('bool-order',node('lt','Bool',lit(False),lit(True)))
    add_('wrong-arity',node('nat.add','Nat',lit(1)))
    add_('proof-flag',lit(False),program_mutation='ordinary_proof')
    add_('source-flag',lit(True),program_mutation='source_evaluation_proved')
    add_('escaping-binder',quantified('forall',lit(0),lit(2),node('eq','Bool',bound(1),lit(0))))
    from closed_carriers import fixture
    source={'op':'source.zero','type':{'kind':'source','source_type_id':fixture()['ids']['int64']},'payload':{}}
    add_('source-component-pending',source,'ordinary_component_pending')
    add_('construction-budget-pending',lit(1024),'ordinary_component_pending',budget=2)
    return rows


def classify_vectors(rows):
    from logical_elaboration import validate
    from go_obligations import OrdinaryElaborationPending
    out=[]
    for row in rows:
        try:
            p=elaborate(row['expression'],row.get('budget',4096))
            if 'program_mutation' in row:p[row['program_mutation']]=True;validate(row['expression'],p)
            status='typed_logical_unproved'
        except OrdinaryElaborationPending:status='ordinary_component_pending'
        except ValueError:status='configuration_rejection'
        assert status==row['expected'],(row['id'],status)
        out.append({'id':row['id'],'classification':status})
    return out
