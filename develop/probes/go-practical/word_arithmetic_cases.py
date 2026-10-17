"""Checked-statement controls and selected full-word denotation observations."""
import copy
from arithmetic_terms import *
from word_arithmetic import MODULE,Bridge
from word_foundation import literal
from logical_elaboration import ArithmeticEvaluator
from numeric_bridge_cases import queries as old_queries,mathematical_observation

WIDTHS=(8,16,32,64)
MUTATIONS=('dropped-final-carry','unrelated-overflow','missing-result-sign','unsigned-as-signed')


def queries():return [q for q in old_queries() if q['initial_nat']=='0']
def negative_plan(p,kind):
    q=copy.deepcopy(p);q['module']+='.'+kind.replace('-','_');prefix=p['module']+'.'
    name={'dropped-final-carry':'UnsignedAddition','unrelated-overflow':'SignedAddition','missing-result-sign':'SignedCrossSums','unsigned-as-signed':'SignedValue'}[kind]
    d=next(d for d in q['declarations'] if d['name']==prefix+name)
    if kind=='unsigned-as-signed':d['value']['body']['arguments'][1]=zero();return q
    ty=d['ty']
    while ty['kind']=='pi':
        if kind=='unrelated-overflow' and ty['name']=='no_overflow':ty['domain']=eq(boolean(),bit(True),bit(True));return q
        ty=ty['body']
    if kind=='dropped-final-carry':ty['arguments'][2]=ty['arguments'][2]['arguments'][0]
    else:ty['arguments'][1]=ty['arguments'][1]['arguments'][0]
    return q


def expected_denotation(q):
    w=q['width'];mask=(1<<w)-1;x,y=int(q['x_hex'],16),int(q['y_hex'],16);result=(x+y+int(q['incoming']))&mask
    signed=lambda n:n-(1<<w) if n&(1<<(w-1)) else n
    return {'id':q['id'],'width':w,'x_hex':q['x_hex'],'y_hex':q['y_hex'],'incoming':q['incoming'],
        'unsigned_x':str(x),'unsigned_y':str(y),'unsigned_result':str(result),
        'signed_x':str(signed(x)),'signed_y':str(signed(y)),'signed_result':str(signed(result))}


def model_observations(qs,plans):
    """Evaluate every actual new denotation for the selected8-bit query subset.

    Universal paired proofs cover all four widths. This finite interpreter
    selection avoids treating very large unary arithmetic as a release gate.
    """
    from word_arithmetic import seed_rows
    out=[];p=plans['8'];ev=ArithmeticEvaluator({'declarations':seed_rows()+p['declarations']});prefix=MODULE+'.W8.'
    for q in qs:
        if q['width']!=8:continue
        x,y=ev.eval(literal(8,int(q['x_hex'],16))),ev.eval(literal(8,int(q['y_hex'],16)))
        result=ev.invoke('Probe.GoPractical.WordFoundation.W8.AddCarry',x,y,q['incoming'])
        us=[ev.invoke(prefix+'UnsignedValue',a) for a in (x,y,result)]
        ints=[ev.observe(ev.invoke(prefix+'SignedValue',a),'Int') for a in (x,y,result)]
        out.append({'id':q['id'],'width':8,'x_hex':q['x_hex'],'y_hex':q['y_hex'],'incoming':q['incoming'],
            **{n:str(v) for n,v in zip(('unsigned_x','unsigned_y','unsigned_result','signed_x','signed_y','signed_result'),us+ints)}})
    return out
