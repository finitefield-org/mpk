"""Actual whole-word numeric proofs,negative boundaries and source references."""
import json
import unittest
from collections import Counter
from numeric_bridge_cases import MUTATIONS,negative_plan,queries,model_observations,mathematical_observation
from word_numeric_bridge import MODULE,SEED,WIDTHS,WORD_BINDINGS
from run_numeric_bridge import RECORD,documents,classify
from run_proof_pilot import sha


class NumericTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.docs=documents();cls.report=json.loads((RECORD/'observations.json').read_bytes())
        cls.native=json.loads((RECORD/'native.json').read_bytes());cls.model=json.loads((RECORD/'model.json').read_bytes())
    def test_all20_actual_paired_proof_outcomes_and_exact_certificate_bytes(self):
        self.assertEqual(Counter(classify(r['case'],r['observations']) for r in self.report['cases']),{'accepted_zero_axiom':4,'ordinary_type_mismatch':14,'candidate_conversion_fuel_failure':2})
        for r in self.report['cases']:self.assertEqual(sha((RECORD/(r['case']['id']+'.mpcert')).read_bytes()),r['raw_sha256'])
    def test_each_negative_changes_only_its_intended_statement(self):
        suffixes={'wrong-local-carry':'LocalCarry','wrong-leaf-bit':'Sum0At0','unrelated-overflow':'SignedAddition','dropped-final-carry':'NatConservation'}
        for w in WIDTHS:
            p=self.docs['expansions.json'][str(w)]
            for kind in MUTATIONS:
                q=negative_plan(w,kind,p);changed=[(a['name'],a['value']==b['value']) for a,b in zip(p['declarations'],q['declarations']) if a!=b]
                self.assertEqual(changed,[(MODULE+'.W'+str(w)+'.'+suffixes[kind],True)])
    def test_thirteen_actual_typed_value_declarations_and_numeric_theorems(self):
        original={b['operation_hash']:b for b in json.loads(WORD_BINDINGS.read_bytes())['bindings']};rows=self.docs['source-values.json'];self.assertEqual(len(rows),13)
        for row in rows:
            p=self.docs['expansions.json'][str(row['width'])];declarations={d['name']:d for d in p['declarations']};old=original[row['operation_hash']]
            self.assertEqual(row['value_name'],old['value_name']);self.assertEqual(row['source_scope_sha256'],old['source_scope_sha256'])
            for d in old['declarations']:self.assertEqual(declarations[d['name']],d)
            self.assertEqual(declarations[row['theorem']]['kind'],'theorem');self.assertIs(row['source_evaluation_proved'],False)
    def test_source_nominal_ids_and_theorems_remain_distinct(self):
        rows=self.docs['source-values.json'];self.assertEqual(len({r['source_type_id'] for r in rows}),13)
        self.assertEqual(len({r['operation_hash'] for r in rows}),13);self.assertEqual(len({r['theorem'] for r in rows}),13)
        self.assertEqual(Counter(r['width'] for r in rows),{8:3,16:2,32:2,64:6})
    def test_every_binary_block_has_carry_sum_and_sign_stage_correspondence(self):
        for w in WIDTHS:
            names=[d['name'] for d in self.docs['expansions.json'][str(w)]['declarations']];prefix=MODULE+'.W'+str(w)+'.'
            for family in ('Carry','Sum','SignIncoming'):
                block=[n for n in names if n.startswith(prefix+family) and 'At' in n]
                self.assertEqual(len(block),2*w-1)
    def test_actual_current_ripple_carries_and_guard_reproduce_all288_cases(self):
        plans={int(k):v for k,v in self.docs['expansions.json'].items()}
        self.assertEqual(model_observations(queries(),plans),self.model)
    def test_go_big_integers_and_independent_oracle_agree(self):
        self.assertEqual(self.model,self.native);self.assertEqual(self.model,[mathematical_observation(q) for q in queries()])
        for x in self.native:self.assertEqual(x['unsigned_left'],x['unsigned_right'])
    def test_exact_signed_guard_tracks_genuine_overflow(self):
        self.assertEqual(sum(r['no_overflow'] for r in self.native),210)
        for q,r in zip(queries(),self.native):
            half=1<<(q['width']-1);integer=int(r['integer_sum'])
            self.assertEqual(r['no_overflow'],-half<=integer<half)
            self.assertEqual(r['no_overflow'],r['signed_cross_left']==r['signed_cross_right'])
            if r['no_overflow']:self.assertEqual(r['integer_sum'],r['go_signed_result'])
    def test_large_initial_naturals_remain_exact(self):
        qs=queries();self.assertEqual({q['initial_nat'] for q in qs},{'0','17',str(10**50+37)})
        for q,r in zip(qs,self.native):self.assertGreaterEqual(int(r['unsigned_left']),int(q['initial_nat']))
    def test_incoming_carry_is_quantified_and_observed(self):
        self.assertEqual(Counter(q['incoming'] for q in queries()),{False:144,True:144})
        for w in WIDTHS:
            row=next(d for d in self.docs['expansions.json'][str(w)]['declarations'] if d['name']==MODULE+'.W'+str(w)+'.NatConservation')
            binders=[];ty=row['ty']
            while ty['kind']=='pi':binders.append(ty['name']);ty=ty['body']
            self.assertEqual(binders,['initial','x','y','incoming'])
    def test_numeric_addition_does_not_promote_all_word_or_go_source_proofs(self):
        for field in ('all_word_Nat_Int_operations_proved','source_evaluation_proved','complete_go_elaboration','profile_activated'):self.assertIs(self.report[field],False)
        from go_obligations import OrdinaryElaborationPending,require_materialized
        with self.assertRaises(OrdinaryElaborationPending):require_materialized({'unelaborated_request_ids':['remaining-source-operation-statements'],'pending_candidates':[]})
    def test_initial_fuel_findings_preserve_both_checker_outcomes(self):
        files=list((RECORD/'verification').glob('initial-*/diagnostic.json'));self.assertEqual(len(files),2)
        for p in files:
            for x in json.loads(p.read_bytes())['inputs']:self.assertEqual(sha((p.parent/x['path']).read_bytes()),x['sha256'])
        p=RECORD/'verification/initial-source-value-conversion'
        self.assertEqual(json.loads((p/'rust.json').read_bytes())['exit_code'],1)
        self.assertEqual(json.loads((p/'go.json').read_bytes())['exit_code'],0)


if __name__=='__main__':unittest.main()
