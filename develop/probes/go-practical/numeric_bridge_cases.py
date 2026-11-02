"""Proof mutations and genuine Go/big-integer versus actual ripple observations."""
import copy
import random
from word_numeric_bridge import (MODULE,Bridge,plan,cube,nat,old,tree,composition,maybe,function,trans,congr,symmetry)
from word_foundation import WIDTHS,Evaluator,MODULE as WORD,bit

MUTATIONS=('wrong-local-carry','wrong-leaf-bit','unrelated-overflow','dropped-final-carry')


def negative_plan(width,mutation,base=None):
    p=copy.deepcopy(plan(width) if base is None else base);prefix=MODULE+'.W'+str(width)+'.'
    suffix={'wrong-local-carry':'LocalCarry','wrong-leaf-bit':'Sum0At0','unrelated-overflow':'SignedAddition','dropped-final-carry':'NatConservation'}[mutation]
    row=next(d for d in p['declarations'] if d['name']==prefix+suffix);body=row['ty']
    if mutation=='unrelated-overflow':
        while body['kind']=='pi' and body['name']!='no_overflow':body=body['body']
        body['domain']={'kind':'app','function':{'kind':'const','name':'Std.Eq'},'arguments':[{'kind':'const','name':'Std.Bool'},bit(True),bit(True)]}
    else:
        while body['kind']=='pi':body=body['body']
        rhs=body['arguments'][2]
        if mutation=='wrong-local-carry':rhs['function']['name']=WORD+'.Bit.Sum'
        elif mutation=='wrong-leaf-bit':rhs['arguments'][-1]=bit(True)
        else:rhs['function']=rhs['function']['arguments'][0]
    p['module']+='.'+mutation.replace('-','_')
    return p


def queries():
    out=[]
    for w in WIDTHS:
        mask=(1<<w)-1;half=1<<(w-1);pattern=mask//3;rng=random.Random(0x4e4154+w)
        pairs=[(0,0),(0,mask),(1,1),(mask,1),(mask,mask),(half,half),(half,mask),(half-1,1),(pattern,pattern*2)]
        pairs += [(rng.getrandbits(w),rng.getrandbits(w)) for _ in range(3)]
        for x,y in pairs:
            for c in (False,True):
                for initial in ('0','17',str(10**50+37)):
                    out.append({'id':'numeric-'+str(len(out)).zfill(3),'width':w,'x_hex':format(x,'0'+str(w//4)+'x'),
                        'y_hex':format(y,'0'+str(w//4)+'x'),'incoming':c,'initial_nat':initial})
    return out


def observation(q,result,carry,incoming,guard):
    w=q['width'];power=1<<w;x=int(q['x_hex'],16);y=int(q['y_hex'],16);c=int(q['incoming']);initial=int(q['initial_nat'])
    sign=lambda v:power if v&(power//2) else 0
    unsigned=initial+x+y+c
    return {'id':q['id'],'word_hex':format(result,'0'+str(w//4)+'x'),'carry':carry,'sign_incoming':incoming,'no_overflow':guard,
        'go_signed_result':str(result-sign(result)),'unsigned_left':str(unsigned),'unsigned_right':str(initial+result+power*int(carry)),
        'signed_cross_left':str(unsigned+sign(result)),'signed_cross_right':str(initial+result+sign(x)+sign(y)),
        'integer_sum':str(x-sign(x)+y-sign(y)+c)}


def mathematical_observation(q):
    w=q['width'];x=int(q['x_hex'],16);y=int(q['y_hex'],16);c=int(q['incoming']);power=1<<w
    total=x+y+c;carry=total>=power;incoming=(x%(power//2)+y%(power//2)+c)>=power//2
    return observation(q,total%power,carry,incoming,carry==incoming)


def model_observations(rows,plans):
    out=[];key=None;ev=None;cache=None
    for q in rows:
        group=(q['width'],q['x_hex'],q['y_hex'],q['incoming'])
        if group!=key:
            w=q['width'];ev=Evaluator(plans[w]);prefix=WORD+'.W'+str(w)+'.'
            x,y=ev.word(w,int(q['x_hex'],16)),ev.word(w,int(q['y_hex'],16));c=q['incoming']
            result=ev.integer(w,ev.invoke(prefix+'AddCarry',x,y,c));carry=ev.invoke(prefix+'Carry'+str(w),x,y,c)
            incoming=ev.invoke(prefix+'Carry'+str(w-1),x,y,c)
            guard=ev.invoke(MODULE+'.W'+str(w)+'.NoOverflow',x,y,c)
            cache=(result,carry,incoming,guard);key=group
        out.append(observation(q,*cache))
    return out
