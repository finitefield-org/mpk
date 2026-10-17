"""Boundary/seeded observations and deliberately incorrect ordinary claims."""
import copy
import random
from word_foundation import MODULE,WIDTHS,plan,Evaluator,const,var,app,boolean,bit,eq,refl,word,call,at,literal

BINARY=('Add','Sub','Mul','And','Or','Xor','AndNot','Equal','NotEqual','UnsignedLess','UnsignedLessEqual','SignedLess','SignedLessEqual')
DIVISION=('UnsignedDiv','UnsignedRem','SignedDiv','SignedRem')
SHIFTS=('ShiftLeft','ShiftRight','SignedShiftRight')
NEGATIVES=('false-carry','false-add-identity','false-zero-division','ill-typed-shift')


def queries():
    rows=[]
    def add(w,op,x=0,y=0,count=0,signed_count=False,target=0):
        rows.append({'id':'word-'+str(len(rows)).zfill(4),'width':w,'operation':op,'x_hex':format(x&((1<<w)-1),'0'+str(w//4)+'x'),
            'y_hex':format(y&((1<<w)-1),'0'+str(w//4)+'x'),'count_hex':format(count,'016x'),'signed_count':signed_count,'target_width':target})
    for w in WIDTHS:
        mask=(1<<w)-1;minimum=1<<(w-1);rng=random.Random(0x4d504b+w)
        pattern=mask//3;values=[0,1,mask,minimum,minimum-1,minimum+1,pattern,pattern*2]
        pairs=[(0,0),(0,1),(1,1),(mask,1),(mask,2),(minimum,mask),(minimum,3),(mask-1,minimum),(pattern,pattern*2)]
        pairs += [(rng.getrandbits(w),rng.getrandbits(w)) for _ in range(3)]
        for x,y in pairs:
            for op in BINARY+DIVISION:add(w,op,x,y)
        for x in values:
            for op in ('Neg','Not'):add(w,op,x)
            for target in WIDTHS:
                for signed in (False,True):add(w,('Signed' if signed else 'Unsigned')+'To'+str(target),x,target=target)
        for x in (0,minimum,mask,pattern):
            for count in sorted({0,1,w-1,w,w+1,63,64,65,1<<63,(1<<64)-1}):
                for op in SHIFTS:add(w,op,x,count=count)
        for count in (0,1,63,1<<63,(1<<64)-1):
            for op in SHIFTS:add(w,op,pattern,count=count,signed_count=True)
    return rows


def model_observations(rows,progress=None):
    out=[];p=None;ev=None;group=None;values={}
    for ordinal,q in enumerate(rows):
        w=q['width'];key=(w,q['x_hex'],q['y_hex'],q['count_hex'])
        if p is None or w!=p['width']:p={'width':w,'plan':plan(w,theorems=False)}
        if key!=group:
            ev=Evaluator(p['plan']);values={};group=key
        def value(width,raw):
            k=(width,raw)
            if k not in values:values[k]=ev.word(width,int(raw,16))
            return values[k]
        x=value(w,q['x_hex']);y=value(w,q['y_hex']);count=value(64,q['count_hex']);op=q['operation'];prefix=MODULE+'.W'+str(w)+'.'
        if (op in DIVISION and not ev.invoke(prefix+'DivideDefined',y)) or (op in SHIFTS and q['signed_count'] and not ev.invoke(prefix+'SignedCountDefined',count)):
            result={'id':q['id'],'kind':'runtime_safety_failure'}
        else:
            args=(x,y) if op in BINARY+DIVISION else (x,count) if op in SHIFTS else (x,)
            observed=ev.invoke(prefix+op,*args)
            if type(observed) is bool:result={'id':q['id'],'kind':'boolean','value':observed}
            else:
                target=q['target_width'] or w
                result={'id':q['id'],'kind':'word','value_hex':format(ev.integer(target,observed),'0'+str(target//4)+'x')}
        out.append(result)
        if progress and (ordinal+1)%100==0:progress(ordinal+1,len(rows))
    return out


def mathematical_observation(q):
    """Independent integer oracle,not the ordinary AST interpreter."""
    w=q['width'];mask=(1<<w)-1;x=int(q['x_hex'],16);y=int(q['y_hex'],16);n=int(q['count_hex'],16);op=q['operation']
    signed=lambda v:v-(1<<w) if v&(1<<(w-1)) else v
    a,b=signed(x),signed(y)
    if (op in DIVISION and y==0) or (op in SHIFTS and q['signed_count'] and n&(1<<63)):return {'id':q['id'],'kind':'runtime_safety_failure'}
    if op in ('Equal','NotEqual','UnsignedLess','UnsignedLessEqual','SignedLess','SignedLessEqual'):
        v={'Equal':x==y,'NotEqual':x!=y,'UnsignedLess':x<y,'UnsignedLessEqual':x<=y,'SignedLess':a<b,'SignedLessEqual':a<=b}[op]
        return {'id':q['id'],'kind':'boolean','value':v}
    if op=='Add':v=x+y
    elif op=='Sub':v=x-y
    elif op=='Mul':v=x*y
    elif op=='Neg':v=-x
    elif op=='Not':v=~x
    elif op=='And':v=x&y
    elif op=='Or':v=x|y
    elif op=='Xor':v=x^y
    elif op=='AndNot':v=x&~y
    elif op=='UnsignedDiv':v=x//y
    elif op=='UnsignedRem':v=x%y
    elif op in ('SignedDiv','SignedRem'):
        quotient=(abs(a)//abs(b))*(-1 if (a<0)!=(b<0) else 1)
        v=quotient if op=='SignedDiv' else a-quotient*b
    elif op=='ShiftLeft':v=0 if n>=w else x<<n
    elif op=='ShiftRight':v=0 if n>=w else x>>n
    elif op=='SignedShiftRight':v=(-1 if a<0 else 0) if n>=w else a>>n
    elif 'To' in op:
        v=a if op.startswith('Signed') else x;w=q['target_width'];mask=(1<<w)-1
    else:raise ValueError(op)
    return {'id':q['id'],'kind':'word','value_hex':format(v&mask,'0'+str(w//4)+'x')}


def negative_plan(width,kind):
    p=plan(width);prefix=MODULE+'.W'+str(width)+'.';name=prefix+'Negative.'+kind.replace('-','_')
    if kind=='false-add-identity':
        d=copy.deepcopy(next(d for d in p['declarations'] if d['name']==prefix+'AddZeroAt'));d['name']=name
        body=d['ty']
        while body['kind']=='pi':body=body['body']
        body['arguments'][2]=bit(False)
    else:
        if kind=='false-carry':ty=eq(boolean(),call(MODULE+'.Bit.Carry',bit(False),bit(False),bit(False)),bit(True));value=refl(boolean(),bit(False))
        elif kind=='false-zero-division':ty=eq(boolean(),call(prefix+'DivideDefined',const(prefix+'Zero')),bit(True));value=refl(boolean(),bit(False))
        elif kind=='ill-typed-shift':ty=word(width)[0];value=call(prefix+'ShiftLeft',literal(width,1),bit(False))
        else:raise ValueError(kind)
        d={'name':name,'kind':'theorem','ty':ty,'value':value}
    p['declarations'].append(d);return p
