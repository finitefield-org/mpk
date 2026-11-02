"""Static SSA/state/control/capture/provenance boundaries,not source semantics."""
import copy
import unittest
from vir_body import BodyValidator,validate_body,STATE,source
from vir_body_build import Builder
from vir_body_vectors import model,components,functions,vector_specs,alter,nodes,reseal
from body_canonical import canonical,parse


class BodyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.m=model();cls.cs=components(cls.m);cls.fs=functions(cls.m)

    def call_body(self,mode='valid'):
        m=self.m;b=Builder(m);code=m.id('fixture/policy.RejectCode')
        callee=next(d for d,sig in m.callees.items() if sig['receiver']==code)
        def make(r):
            receiver=r.zero(code);one,state_one=r.apply(19,{'callee':callee},[receiver,r.state]);two,state_two=r.apply(19,{'callee':callee},[receiver,r.state])
            if mode=='pack':
                text=r.zero(m.id('string'));one=r.apply(17,{'types':[m.id('string')]},[text])[0]
                r.apply(18,{'callee':callee,'index':0},[one,state_one])
            else:r.project(one,0,state_two if mode=='cross' else state_one)
            if mode=='rollback':r.state=state_one
            r.return_zero()
        return b.function(make)

    def test_all56_components_are_reconstructed(self):
        self.assertEqual(len(self.cs),56)
        for c in self.cs:
            v=BodyValidator(self.m,c['definitions']);v.region(c['region'],c['inputs'],c['outputs'],stack=tuple(c['entry_control_stack']))
            self.assertEqual(v.used_defs,set(v.defs))

    def test_three_functions_and20_body_outcomes(self):
        from operation_signatures import Signatures
        from contract_scope import Scope
        for spec in vector_specs():
            row=self.fs[spec['base']];m=self.m if row['model']=='scopes' else Signatures(Scope(row['raw'],row['universe']))
            if spec['expected']=='typed_unproved':validate_body(m,alter(spec,self.fs,self.m))
            else:
                with self.assertRaises(ValueError,msg=spec['id']):validate_body(m,alter(spec,self.fs,self.m))

    def test_static_call_projection_can_read_its_own_historical_post(self):
        r=self.call_body();validate_body(self.m,r)
        ns=list(nodes(r['body']));calls=[n for n in ns if n['kind']=='apply' and len(n['outputs'])==2]
        projection=next(n for n in ns if n['kind']=='project')
        self.assertEqual(projection['post_state'],calls[0]['outputs'][1]);self.assertNotEqual(projection['post_state'],calls[1]['outputs'][1])

    def test_cross_invocation_equal_type_post_is_rejected(self):
        with self.assertRaises(ValueError):self.call_body('cross')

    def test_packed_same_type_product_cannot_impersonate_call(self):
        with self.assertRaises(ValueError):self.call_body('pack')

    def test_historical_call_state_cannot_become_current(self):
        with self.assertRaises(ValueError):self.call_body('rollback')

    def test_closed_error_joint_projection_is_supported(self):
        b=Builder(self.m)
        def make(r):
            err=r.zero(self.m.id('error'));product,post=r.apply(26,{},[err,r.state]);r.project(product,0,post);r.return_zero()
        # Nil safety is an unproved ordinary obligation,never inferred from typing.
        record=b.function(make);validate_body(self.m,record)
        self.assertTrue(any(s['dependencies'] for s in record['obligation_sites']))

    def branch_capture(self,mode):
        b=Builder(self.m)
        def make(r):
            flag=r.r['parameters'][2]['id'];saved=[]
            def children(role,c,outputs):
                if role=='then':
                    if mode in {'unused','unlisted'}:c.r['captures']=[flag] if mode=='unused' else []
                    else:c.r['captures']=[flag];c.apply(8,{'type':self.m.id('bool'),'operator':'not'},[flag])
                    saved.append(c.zero(self.m.id('int')))
                    if mode=='unlisted':c.apply(8,{'type':self.m.id('bool'),'operator':'not'},[flag])
                elif mode=='sibling':c.r['captures']=[saved[0]]
                c.normal()
            control=r.apply(15,{'condition_type':self.m.id('bool')},[flag,r.state],children)[0]
            r.resume(control,lambda c:c.return_zero())
        return b.function(make)

    def test_lexical_capture_is_explicit_and_exact(self):validate_body(self.m,self.branch_capture('valid'))
    def test_unused_capture_rejects(self):
        with self.assertRaises(ValueError):self.branch_capture('unused')
    def test_unlisted_outer_value_rejects(self):
        with self.assertRaises(ValueError):self.branch_capture('unlisted')
    def test_sibling_value_cannot_escape_its_region(self):
        with self.assertRaises(ValueError):self.branch_capture('sibling')

    def test_location_freezes_descriptor_and_defers_access_to_store(self):
        m=self.m;b=Builder(m)
        def make(r):
            descriptor=r.load(r.address(m.scope.inputs[0]));index=r.zero(m.id('uint64'));loc=r.slice_address(descriptor,index)
            code=r.zero(m.id('fixture/policy.RejectCode'));callee=next(d for d,s in m.callees.items() if s['receiver']==m.id('fixture/policy.RejectCode'))
            r.apply(19,{'callee':callee},[code,r.state]);value=r.zero(m.id('int64'));r.apply(12,{'type':m.id('int64')},[loc,value,r.state]);r.return_zero()
        record=b.function(make);sites=record['obligation_sites'];store=next(s for s in sites if s['access'] and s['access'][0]['mode']=='write')
        access=store['access'][0];self.assertNotEqual(access['state_ref']['value'],access['location']['captured_state'])
        self.assertEqual(access['location']['root']['kind'],'slice');self.assertIn('full_index_bounds_at_access',access['checks'])
        addresses={n['id'] for n in nodes(record['body']) if n['kind']=='address'}
        self.assertTrue(all(not s['access'] for s in sites if s['node_id'] in addresses))

    def test_slice_crossing_cannot_be_hidden_in_array_path(self):
        b=Builder(self.m)
        def make(r):
            index=r.zero(self.m.id('uint64'));r.address(self.m.scope.inputs[0],[{'kind':'array','owner_type_id':self.m.id('[]int64'),'index':index}]);r.return_zero()
        with self.assertRaises(ValueError):b.function(make)

    def test_assignment_values32_do_not_become_joint_product32(self):
        b=Builder(self.m);t=self.m.id('int64')
        def make(r):
            value=r.zero(t);values=r.values([value]*32);r.apply(21,{'targets':[None]*32,'types':[t]*32},[values,r.state]);r.return_zero()
        record=b.function(make);validate_body(self.m,record)
        b=Builder(self.m);binding=next(x for x in self.m.scope.value['bindings'] if x['name']=='index' and x['id'] in self.m.scope.header_ids[4]);t=binding['source_type_id']
        def ordered(r):
            v=r.zero(t);r.declare(binding['id'],v);loc=r.address(binding['id']);vs=r.values([v,v,v])
            r.apply(21,{'targets':[t,None,t],'types':[t,t,t]},[loc,loc,vs,r.state]);r.return_zero()
        record=b.function(ordered);points=[a['state_ref'] for s in record['obligation_sites'] for a in s['access'] if a['mode']=='write']
        self.assertEqual([p['target_ordinal'] for p in points],[0,2]);self.assertTrue(all(p['kind']=='assignment_prefix' for p in points))

    def test_control_barrier_and_region_role_order_reject(self):
        for name in ('instruction-after-control','swapped-regions','forged-target-stack'):
            spec=next(s for s in vector_specs() if s['id']==name)
            with self.assertRaises(ValueError):validate_body(self.m,alter(spec,self.fs,self.m))
        binding=next(x for x in self.m.scope.value['bindings'] if x['name']=='index' and x['id'] in self.m.scope.header_ids[4])
        b=Builder(self.m)
        def bad(r):
            def child(c):c.declare(binding['id'],c.zero(binding['source_type_id']));c.normal()
            control=r.scope(child)
            r.resume(control,lambda c:(c.address(binding['id']),c.return_zero()))
        with self.assertRaises(ValueError):b.function(bad)
        b=Builder(self.m)
        def valid(r):
            def child(c):c.declare(binding['id'],c.zero(binding['source_type_id']));c.load(c.address(binding['id']));c.normal()
            control=r.scope(child);r.resume(control,lambda c:c.return_zero())
        validate_body(self.m,b.function(valid))

    def test_missing_test_without_break_has_no_normal_exit(self):
        b=Builder(self.m)
        def make(r):
            c=r.apply(51,{'loop':0,'condition_type':self.m.id('bool'),'has_test':False},[r.state])[0];r.yield_([c])
        record=b.function(make);v=BodyValidator(self.m,record['operation_definitions']);self.assertEqual(v.function(record['body'])['controls'],[])

    def test_actual_root_inputs_and_result_shape_are_required(self):
        record=copy.deepcopy(self.fs[0]['record']);record['body']['parameters'][0]['type']=source(self.m.id('[]uint8'))
        with self.assertRaises(ValueError):validate_body(self.m,reseal(record))

    def test_mutable_envelope_cannot_override_captured_source_context(self):
        m=model();record=copy.deepcopy(self.fs[0]['record']);m.scope.context['source_language']='rust';record['semantic_context']['source_language']='rust'
        with self.assertRaises(ValueError):validate_body(m,reseal(record))

    def test_duplicate_json_and_later_body_fields_reject(self):
        with self.assertRaises(ValueError):parse(b'{"body":0,"body":1}')
        record=copy.deepcopy(self.fs[0]['record']);record['proof_checked']=True
        with self.assertRaises(ValueError):validate_body(self.m,reseal(record))
        for bad in (None,{},[None],[{}]):
            record=copy.deepcopy(self.fs[0]['record']);record['operation_definitions']=bad
            with self.assertRaises(ValueError):validate_body(self.m,reseal(record))

    def test_registered256_container_limit_has_no_pilot64_cap(self):
        for depth in (64,65,255,256):
            raw=b'['*depth+b'0'+b']'*depth;self.assertEqual(canonical(parse(raw)),raw)
        with self.assertRaises(ValueError):parse(b'['*257+b'0'+b']'*257)

    def test_canonical_safe_words_and_unicode_match_existing_rules(self):
        self.assertEqual(canonical({'\U0001f600':1,'\ue000':'\x00ff'}),b'{"\xf0\x9f\x98\x80":1,"\xee\x80\x80":"\\u0000ff"}')
        for raw in (b'1.0',b'NaN',b'9007199254740992',b'"\\ud800"'):
            with self.assertRaises(ValueError):parse(raw)


if __name__=='__main__':unittest.main()
