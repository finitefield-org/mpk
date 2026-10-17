"""Affected arithmetic/closed-wire interfaces and independent proof evidence."""
import copy
import json
import unittest
from collections import Counter
from arithmetic_terms import *
from arithmetic_cases import MUTATIONS,negative_plan,compiled_plan,expected,observations,lit,node,quantified,bound
from logical_elaboration import HEADS,ArithmeticEvaluator,ModelEvaluationPending,elaborate,validate
from go_obligations import OrdinaryElaborationPending,require_materialized
from run_ordinary_arithmetic import RECORD,documents,classify
from run_proof_pilot import sha


def constants(x):
    if isinstance(x,dict):
        if x.get('kind')=='const':yield x['name']
        for value in x.values():yield from constants(value)
    elif isinstance(x,list):
        for value in x:yield from constants(value)


class ArithmeticTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.docs=documents();cls.report=json.loads((RECORD/'observations.json').read_bytes())
        cls.native=json.loads((RECORD/'native.json').read_bytes());cls.model=json.loads((RECORD/'model.json').read_bytes())
        cls.closed=compiled_plan(cls.docs['expansion.json'],cls.docs['compilations.json'])
    def test_all10_paired_outcomes_and_exact_certificate_bytes(self):
        self.assertEqual(Counter(classify(r['case'],r['observations']) for r in self.report['cases']),{'accepted_zero_axiom':2,'ordinary_type_mismatch':8})
        for r in self.report['cases']:self.assertEqual(sha((RECORD/(r['case']['id']+'.mpcert')).read_bytes()),r['raw_sha256'])
    def test_negative_controls_change_one_member_or_add_false_proof(self):
        base=self.docs['expansion.json']
        for kind in MUTATIONS:
            old=self.closed if kind=='false-clause-proof' else base;new=negative_plan(base,self.closed,kind)
            changed=[a['name'] for a,b in zip(old['declarations'],new['declarations']) if a!=b]
            self.assertEqual(len(changed),0 if kind=='false-clause-proof' else 1)
            self.assertEqual(len(new['declarations'])-len(old['declarations']),int(kind=='false-clause-proof'))
    def test_all27_existing_logical_heads_are_exercised(self):
        def heads(x):
            if isinstance(x,dict):
                if 'op' in x:yield x['op']
                for value in x.values():yield from heads(value)
            elif isinstance(x,list):
                for value in x:yield from heads(value)
        self.assertEqual(set(heads(self.docs['expressions.json'])),HEADS);self.assertEqual(len(HEADS),27)
    def test_complete149_typed_components_reconstruct_without_caller_proofs(self):
        for row,c in zip(self.docs['expressions.json'],self.docs['compilations.json']):
            validate(row['expression'],c['program']);self.assertIsNone(c['program']['ordinary_proof'])
    def test22_component_cases_keep_rejection_unproved_and_pending_separate(self):
        from arithmetic_cases import classify_vectors
        self.assertEqual(classify_vectors(self.docs['vectors.json']),self.docs['outcomes.json'])
        self.assertEqual(Counter(x['classification'] for x in self.docs['outcomes.json']),{'typed_logical_unproved':1,'configuration_rejection':19,'ordinary_component_pending':2})
    def test_false_clause_remains_unproved_and_proof_override_rejects(self):
        expr=lit(False);program=elaborate(expr);self.assertIsNone(program['ordinary_proof']);self.assertIsNotNone(program['statement'])
        ev=ArithmeticEvaluator(self.closed);self.assertIs(ev.invoke(program['value_name']),False)
        program['ordinary_proof']=refl(boolean(),bit(True))
        with self.assertRaises(ValueError):validate(expr,program)
    def test_redundant_integer_representatives_preserve_operations_and_observations(self):
        ev=ArithmeticEvaluator(self.closed);x=ev.invoke(I+'Make',5,3);same=ev.invoke(I+'Make',2,0);y=ev.invoke(I+'Make',1,4)
        self.assertNotEqual((x(False),x(True)),(same(False),same(True)))
        self.assertIs(ev.invoke(I+'Equal',x,same),True)
        for op in ('Add','Sub','Mul'):
            a,b=ev.invoke(I+op,x,y),ev.invoke(I+op,same,y)
            self.assertEqual(ev.observe(a,'Int'),ev.observe(b,'Int'));self.assertIs(ev.invoke(I+'Equal',a,b),True)
        for op in ('Equal','Less','LessEqual'):self.assertEqual(ev.invoke(I+op,x,y),ev.invoke(I+op,same,y))
    def test_all_native_and_actual_AST_results_agree_with_independent_oracle(self):
        self.assertEqual(self.native,expected(self.docs['expressions.json']))
        self.assertEqual(observations(self.docs['expressions.json'],self.docs['compilations.json'],self.closed),self.model)
        for m,n in zip(self.model,self.native):
            if m['status']=='evaluated':self.assertEqual({k:v for k,v in m.items() if k!='status'},n)
    def test_half_open_empty_and_reversed_ranges_have_exact_identity(self):
        values={x['id']:x['value'] for x in self.native}
        for row in self.docs['expressions.json']:
            e=row['expression']
            if e['op'] in {'forall','exists','nat.sum','int.sum'} and e['payload']['lower']['op']=='nat.literal' and e['payload']['upper']['op']=='nat.literal':
                p=e['payload']
                if int(p['upper']['payload']['value'])<=int(p['lower']['payload']['value']):
                    self.assertEqual(values[row['id']],True if e['op']=='forall' else False if e['op']=='exists' else '0')
    def test_nested_lexical_binders_sum_actual_outer_and_inner_indices(self):
        e=quantified('nat.sum',lit(0),lit(4),quantified('nat.sum',lit(0),bound(),node('nat.add','Nat',bound(1),bound())))
        row=next(r for r in self.docs['expressions.json'] if r['expression']==e)
        self.assertEqual(next(n['value'] for n in self.native if n['id']==row['id']),'18')
    def test_fold_independence_requires_only_actual_bounded_equality(self):
        ds={d['name']:d for d in self.docs['expansion.json']['declarations']}
        for family in ('Forall','Exists','NatSum','IntSum'):
            d=ds[MODULE+'.Range.'+family+'BoundedRespects'];self.assertEqual(d['kind'],'theorem')
            self.assertIn(MODULE+'.Range.FoldForall',set(constants(d['ty'])))
            names=[];ty=d['ty']
            while ty['kind']=='pi':names.append(ty['name']);ty=ty['body']
            self.assertEqual(names,['lower','count','body','other','_bounded'])
    def test_large_integers_are_typed_while_private_unary_model_stays_pending(self):
        large=[r for r in self.docs['expressions.json'] if r['large']];self.assertEqual(len(large),10)
        self.assertTrue(any(abs(int(n['value']))>10**70 for n in self.native if n['type']!='Bool'))
        self.assertEqual(Counter(x['status'] for x in self.model),{'evaluated':139,'model_resource_pending':10})
        ev=ArithmeticEvaluator(self.closed);c=next(c for r,c in zip(self.docs['expressions.json'],self.docs['compilations.json']) if r['large'])
        with self.assertRaises(ModelEvaluationPending):ev.invoke(c['program']['value_name'])
    def test_literal_budget_is_pending_and_canonical_decimal_errors_reject(self):
        with self.assertRaises(OrdinaryElaborationPending):elaborate(lit(1024),2)
        for raw in ('01','-0','-1'):
            e=lit(0);e['payload']['value']=raw
            with self.assertRaises(ValueError):elaborate(e)
        self.assertTrue(elaborate(lit(10**70))['declarations'])
    def test_initial_producer_error_is_preserved_with_exact_inputs(self):
        files=list((RECORD/'verification').glob('initial-*/diagnostic.json'));self.assertEqual(len(files),1)
        for p in files:
            for x in json.loads(p.read_bytes())['inputs']:self.assertEqual(sha((p.parent/x['path']).read_bytes()),x['sha256'])
    def test_all169_retained_Go_requests_remain_unelaborated(self):
        bundles=json.loads((RECORD.parent/'proof-artifacts/artifacts.json').read_bytes());count=0
        for b in bundles:
            if b['id'] not in {'go-two-overlays','go-default-contracts'}:continue
            obligations=b['vc']['obligations'];count+=len(obligations['unelaborated_request_ids'])
            with self.assertRaises(OrdinaryElaborationPending):require_materialized(obligations)
        self.assertEqual(count,169)
    def test_component_evidence_does_not_promote_word_elaborator_or_source_proofs(self):
        for flag in ('word_library_connection_proved','elaborator_correctness_proved','source_evaluation_proved','complete_go_elaboration','profile_activated'):self.assertIs(self.report[flag],False)
        for c in self.docs['compilations.json']:
            for flag in ('source_evaluation_proved','complete_go_elaboration'):self.assertIs(c['program'][flag],False)


if __name__=='__main__':unittest.main()
