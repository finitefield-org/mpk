"""Observe accepted bytes and compare source recipes to the frozen Python terms.

The section reader observes an already kernel-accepted ordinary certificate;
it does not replace full format validation or either proof checker.
"""
from pathlib import Path
import functools
import hashlib
import importlib.util
import json

REPO = Path.cwd()
FROZEN = REPO/'develop/probes/go-practical/carrier_terms.py'
spec = importlib.util.spec_from_file_location('frozen_source_error_carriers', FROZEN)
f = importlib.util.module_from_spec(spec)
spec.loader.exec_module(f)


class Sections:
    def __init__(self, raw):
        self.raw, self.at = raw, 0

    def byte(self):
        b = self.raw[self.at]
        self.at += 1
        return b

    def number(self):
        value = 0
        for i in range(10):
            b = self.byte()
            value |= (b & 127) << (7*i)
            if b < 128:
                assert i == 0 or b != 0
                assert value < 2**64
                return value
        raise AssertionError('invalid observed integer')

    def vec(self, read):
        return [read() for _ in range(self.number())]

    def string(self):
        n = self.number()
        b = self.raw[self.at:self.at+n]
        assert len(b) == n
        self.at += n
        return b.decode()

    def level(self):
        tag = self.byte()
        assert tag in range(4)
        return (tag, *[self.number() for _ in range([0,1,2,1][tag])])

    def term(self):
        tag = self.byte()
        if tag in [0,1]: return (tag,self.number())
        if tag in [2,3]: return (tag,self.number(),tuple(self.vec(self.number)))
        assert tag in [4,5,6]
        return (tag,*[self.number() for _ in range(3 if tag == 6 else 2)])

    def declaration(self):
        name, tag = self.number(), self.byte()
        assert tag in range(7)
        ty = self.number()
        if tag == 1:
            value, reducible = self.number(), self.byte()
            assert reducible in [0,1]
            return (name,tag,ty,value,reducible)
        if tag == 2: return (name,tag,ty,self.number())
        if tag in [4,5]:
            parent, generated = self.number(), self.byte()
            assert generated in [0,1]
            return (name,tag,ty,parent,generated)
        return (name,tag,ty)


def frozen_ast(expr, scope=()):
    k = expr['kind']
    if k == 'sort0': return ('sort',('level',0))
    if k == 'const': return ('const',expr['name'],())
    if k == 'var': return ('var',scope[::-1].index(expr['name']))
    if k == 'app': return ('app',frozen_ast(expr['function'],scope),tuple(frozen_ast(x,scope) for x in expr['arguments']))
    assert k in ['lam','pi']
    return (k,frozen_ast(expr['domain'],scope),frozen_ast(expr['body'],scope+(expr['name'],)))


def inspect(certificate, metadata, report_path):
    raw = certificate.read_bytes()
    report = json.loads(report_path.read_bytes())
    assert report['verdict'] == 'accepted' and report['axiom_count'] == 0
    assert report['hashes']['certificate'] == hashlib.sha256(b'MPK-MODULE-CERT-0.1\0'+raw).hexdigest()
    source = json.loads(metadata.read_bytes())
    r = Sections(raw)
    assert raw[:7] == b'MPKCERT'
    r.at = 7
    assert r.string() == 'MPK-CERT-0.1' and r.string() == 'MPK-Core-0.1'
    module = r.string()
    assert module == source['module'] == report['module']
    assert r.number() == 0  # imports
    names = r.vec(r.string)
    levels = r.vec(r.level)
    terms = r.vec(r.term)
    assert r.number() == 0  # ordinary term proofs, no proof-node table
    declarations = r.vec(r.declaration)
    assert len(declarations) == report['declaration_count']
    table = {names[d[0]]:d for d in declarations}

    @functools.lru_cache(None)
    def level(i):
        tag,*args = levels[i]
        return ('level',0) if tag == 0 else ('level-node',tag,tuple(level(j) for j in args))

    @functools.lru_cache(None)
    def term(i):
        tag,*args = terms[i]
        if tag == 0: return ('sort',level(args[0]))
        if tag == 1: return ('var',args[0])
        if tag == 2: return ('const',names[declarations[args[0]][0]],tuple(level(j) for j in args[1]))
        if tag == 3: return ('app',term(args[0]),tuple(term(j) for j in args[1]))
        if tag in [4,5]: return ('lam' if tag == 4 else 'pi',term(args[0]),term(args[1]))
        return ('let',*(term(j) for j in args))

    rows = {t['id']:t for t in source['types']}
    underlying = dict(source['underlying'])
    cases = source['cases']
    assert cases == sorted(set(cases))
    def pair(id):
        return f.const(f'{module}.T{id}.Carrier'),f.const(f'{module}.T{id}.Zero')
    def product(parts): return f.product(parts,[0])
    observed = []
    for id,t in rows.items():
        kind = t['kind']
        if kind == 'nominal': expected = pair(underlying[id])
        elif kind == 'builtin':
            name = t['name']
            if name == 'bool': expected = f.boolean(),f.bit(False)
            elif name == 'string': expected = product([f.word(64),f.vector(8,*f.word(8))])
            elif name == 'error': expected = product([f.word(64),*[pair(c) for c in cases]])
            else:
                assert name in ['int','int8','int16','int32','int64','uint','uint8','uint16','uint32','uint64']
                expected = f.word(t['width'])
        elif kind == 'array':
            assert t['element'] in rows
            expected = (f.boolean(),f.bit(False)) if t['length'] == 0 else f.vector((t['length']-1).bit_length(),*pair(t['element']))
        elif kind == 'slice':
            assert t['element'] in rows
            expected = product([(f.boolean(),f.bit(True)),(f.nat(),f.const('Std.Nat.zero')),f.vector(4,*product([f.word(8),f.word(64)])),f.word(64),f.word(64),f.word(64)])
        else:
            assert kind == 'struct'
            expected = product([pair(v['type_id']) for v in t['fields']])
        ty,zero = expected
        for member,expected_type,expected_value,tag in [
            ('Carrier',{'kind':'sort0'},ty,1),('Zero',ty,zero,1),
            ('ZeroRecipe',f.eq(ty,pair(id)[1],zero),f.refl(ty,zero),2)]:
            d = table[f'{module}.T{id}.{member}']
            assert d[1] == tag
            assert term(d[2]) == frozen_ast(expected_type),(id,member,'type')
            assert term(d[3]) == frozen_ast(expected_value),(id,member,'value')
        observed.append({'type_id':id,'kind':kind,'complete_carrier_zero_and_recipe_match':True})
    assert len([name for name in table if name.startswith(module+'.T')]) == 3*len(rows)
    return {'certificate':str(certificate),'sha256':hashlib.sha256(raw).hexdigest(),'module':module,'cases':len(cases),'types':observed,'all_source_rows_and_case_payloads_retained':True,'scope':'Exact ordinary source-key carrier/zero recipe structure, not arbitrary-input validity or capture/program correspondence.'}
