"""Affected ordinary word semantics,source specializations and proof boundaries."""
import copy
import json
import unittest
from collections import Counter
from word_foundation import MODULE,WIDTHS,Evaluator,EvaluationProof,free_variables,var,lam,boolean,bit
from word_cases import queries,mathematical_observation,NEGATIVES
from word_bindings import Bindings,fixture,combined_plan
from run_word_foundation import RECORD,classify
from proof_canonical import canonical
from run_proof_pilot import sha


class WordTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data=fixture();cls.compiler=Bindings(cls.data['project_input'],cls.data['function_decl_id']);cls.plan=combined_plan(cls.data)
        cls.by_hash={o['operation_hash']:o for o in cls.data['operations']};cls.bindings={b['operation_hash']:b for b in cls.data['bindings']}
        cls.report=json.loads((RECORD/'observations.json').read_bytes());cls.native=json.loads((RECORD/'native.json').read_bytes());cls.model=json.loads((RECORD/'model.json').read_bytes())
    def selected(self,n,**parameters):
        return next(o for o in self.data['operations'] if o['op']=='GP-OP-'+str(n).zfill(2) and all(o['parameters'].get(k)==v for k,v in parameters.items()))
    def evaluate(self,operation,arguments,member='Value'):
        ev=Evaluator(self.plan);binding=self.bindings[operation['operation_hash']];args=[ev.word(self.compiler.model.integer(sid)['width'],v) for sid,v in zip(binding['source_input_types'],arguments)]
        return ev,ev.invoke(binding['value_name'] if member=='Value' else binding['definedness_name'],*args)
    def test_all21_actual_paired_core_results_and_exact_certificate_bytes(self):
        self.assertEqual(Counter(classify(r['case'],r['observations']) for r in self.report['cases']),{'accepted_zero_axiom':5,'incorrect_ordinary_claim':12,'ill_typed_ordinary_term':4})
        for r in self.report['cases']:self.assertEqual(sha((RECORD/(r['case']['id']+'.mpcert')).read_bytes()),r['raw_sha256'])
    def test_all1640_actual_ast_go_and_integer_observations_match(self):
        self.assertEqual(self.model,[mathematical_observation(q) for q in queries()])
        self.assertEqual(self.model,[{k:v for k,v in r.items() if k!='detail'} for r in self.native])
        self.assertEqual(Counter(r['kind'] for r in self.model),{'word':1312,'boolean':288,'runtime_safety_failure':40})
    def test_every787_source_specialization_regenerates(self):
        self.assertEqual((len(self.data['source_word_types']),len(self.data['bindings'])),(13,787))
        for b in self.data['bindings']:self.compiler.validate(self.by_hash[b['operation_hash']],b)
    def test_nominal_same_width_types_cannot_borrow_a_binding(self):
        m=self.compiler.model;a=m.ids['fixture/policy.Amount'];b=m.ids['fixture/policy.OtherAmount']
        x=self.selected(5,type=a,operator='add');y=self.selected(5,type=b,operator='add')
        self.assertNotEqual(x['operation_hash'],y['operation_hash'])
        with self.assertRaises(ValueError):self.compiler.validate(y,self.bindings[x['operation_hash']])
    def test_rehashed_or_changed_operation_fields_reject(self):
        o=copy.deepcopy(self.data['operations'][0]);o['trusted']=True
        with self.assertRaises(ValueError):self.compiler.compile(o)
        o=copy.deepcopy(self.selected(5,operator='add'));o['parameters']['operator']='subtract'
        with self.assertRaises(ValueError):self.compiler.compile(o)
    def test_caller_formula_or_proof_flag_cannot_replace_definedness(self):
        o=self.selected(6,operator='divide');b=copy.deepcopy(self.bindings[o['operation_hash']]);b['declarations'][1]['value']=bit(True)
        with self.assertRaises(ValueError):self.compiler.validate(o,b)
        b=copy.deepcopy(self.bindings[o['operation_hash']]);b['source_evaluation_proved']=True
        with self.assertRaises(ValueError):self.compiler.validate(o,b)
    def test_negative_narrow_signed_counts_are_sign_extended_and_blocked(self):
        m=self.compiler.model;t=m.ids['int64']
        for name in ('int8','int16','int32','int64'):
            sid=m.ids[name];w=m.integer(sid)['width'];o=self.selected(7,type=t,count_type=sid,operator='left')
            _,defined=self.evaluate(o,[1,(1<<w)-1],'Defined');self.assertIs(defined,False)
            _,defined=self.evaluate(o,[1,0],'Defined');self.assertIs(defined,True)
    def test_full_unsigned64_shift_count_is_not_narrowed(self):
        m=self.compiler.model;o=self.selected(7,type=m.ids['uint8'],count_type=m.ids['uint64'],operator='left')
        ev,value=self.evaluate(o,[255,1<<63]);self.assertEqual(ev.integer(8,value),0)
        _,defined=self.evaluate(o,[255,1<<63],'Defined');self.assertIs(defined,True)
    def test_division_by_zero_is_explicitly_blocked_for_every_source_word_type(self):
        for sid in self.data['source_word_types']:
            for op in ('divide','remainder'):
                _,defined=self.evaluate(self.selected(6,type=sid,operator=op),[1,0],'Defined');self.assertIs(defined,False)
    def test_signed_minimum_divided_by_negative_one_wraps_and_remainder_is_zero(self):
        m=self.compiler.model
        for name in ('int8','int16'):
            sid=m.ids[name];w=m.integer(sid)['width'];minimum=1<<(w-1)
            ev,value=self.evaluate(self.selected(6,type=sid,operator='divide'),[minimum,(1<<w)-1]);self.assertEqual(ev.integer(w,value),minimum)
            ev,value=self.evaluate(self.selected(6,type=sid,operator='remainder'),[minimum,(1<<w)-1]);self.assertEqual(ev.integer(w,value),0)
    def test_constant_representability_is_checked_before_circuit_wrapping(self):
        m=self.compiler.model
        for name,value in (('int8','128'),('int8','-129'),('uint8','256'),('uint8','-1')):
            with self.assertRaises(ValueError):m.derive('GP-OP-02',{'type':m.ids[name],'value':value})
    def test_integer_conversions_preserve_source_signedness(self):
        m=self.compiler.model
        for source,expected in (('int8',65535),('uint8',255)):
            o=self.selected(3,**{'from':m.ids[source],'to':m.ids['uint16']});ev,value=self.evaluate(o,[255]);self.assertEqual(ev.integer(16,value),expected)
    def test_nonword_operations_are_not_silently_lowered(self):
        m=self.compiler.model;o=m.derive('GP-OP-01',{'type':m.ids['bool']})
        with self.assertRaises(ValueError):self.compiler.compile(o)
    def test_substitution_avoids_capturing_neutral_word_variables(self):
        e=EvaluationProof([]);result=e.substitute(lam('y',boolean(),var('x')),'x',var('y'))
        self.assertNotEqual(result['name'],'y');self.assertEqual(free_variables(result),{'y'})
    def test_word_library_cannot_promote_complete_go_vc_or_source_proof(self):
        for field in ('source_evaluation_proved','full_word_Nat_bridge_proved','complete_go_elaboration','profile_activated'):self.assertIs(self.report[field],False)
        from go_obligations import require_materialized,OrdinaryElaborationPending
        with self.assertRaises(OrdinaryElaborationPending):require_materialized({'unelaborated_request_ids':['source-and-operation-demands'],'pending_candidates':[]})
    def test_every_initial_failed_probe_keeps_exact_inputs_and_results(self):
        rows=list((RECORD/'verification').glob('initial-*/diagnostic.json'));self.assertEqual(len(rows),6)
        for file in rows:
            for x in json.loads(file.read_bytes())['inputs']:self.assertEqual(sha((file.parent/x['path']).read_bytes()),x['sha256'])


if __name__=='__main__':unittest.main()
