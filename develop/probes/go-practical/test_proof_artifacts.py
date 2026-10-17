"""Regeneration,source correspondence demands and closed pending proof boundaries."""
import copy
import json
import unittest
from pathlib import Path
from collections import Counter
from proof_canonical import canonical,parse,clone,digest
from proof_artifact_vectors import bases,specs,native_queries,base_query,source_validator,policy_from_outputs,altered,reseal
from proof_artifacts import ProofArtifacts,vc,certificate_manifest,skeleton
from go_obligations import OrdinaryElaborationPending,local_requirements,body_regions
from proof_components import fragments
from run_proof_artifacts import RECORD,evaluate,preservation,baseline_queries,hashes,boundaries
from capture_artifact_vectors import bases as source_bases
from capture_artifacts import Artifacts,go_map,go_manifest,frontend_manifest
from run_capture_artifacts import module_policy
from shared_vir_vectors import go_module
from shared_project import Project
from identity_catalog import SHARED


class ProofTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.qs=json.loads((RECORD/'native-queries.json').read_bytes());cls.native=json.loads((RECORD/'retained-native.json').read_bytes());cls.policy=staticmethod(policy_from_outputs(cls.qs,cls.native))
        source=source_bases();cls.data=bases({b['id']:cls.policy(base_query(b)) for b in source if 'project_spec' not in b},source);cls.by_id={b['id']:b for b in cls.data};cls.rows=specs();cls.components=fragments()

    def validator(self,name):
        b=self.by_id[name];return ProofArtifacts(source_validator(b),self.policy,base_query(b))

    def test_all181_vc_certificate_and_skeleton_cases(self):
        out=evaluate(self.data,self.rows,self.policy)
        self.assertEqual(Counter(x['classification'] for x in out),{'retained_vc_generated':6,'configuration_rejection':157,'certificate_manifest_linked_unproved':8,'retained_skeleton_generated':6,'go_obligations_unelaborated':2,'ordinary_elaboration_pending':2})

    def test_five_full_old_vc_skeleton_certificate_values_are_preserved(self):
        old=json.loads((RECORD/'baseline-native.json').read_bytes());self.assertEqual(len(preservation(self.data,self.policy,baseline_queries(self.data),old)),5)

    def test_actual_native_import_regenerates_mutated_members_groups_and_statements(self):
        for row in self.rows:
            if row['mutation'] not in ('missing-member','altered-conclusion','altered-group','nested-extra','altered-statement'):continue
            b=self.by_id[row['base']];q=copy.deepcopy(base_query(b));changed=altered(row,b)
            if row['stage']=='vc':q['vc_payload']={k:changed[k] for k in ('functions','verification_limit_profile')}
            else:q['theorem_declarations']=changed['theorem_declarations']
            self.assertIs(self.policy(q)['accepted'],False,row['id'])

    def test_actual_native_profile_contract_policy_is_used_for_all_retained_targets(self):
        for b in self.data:
            if 'project_spec' in b:continue
            q=copy.deepcopy(base_query(b));q['profile_contract']['value']['required_check_profile_id']='caller.unregistered'
            self.assertIs(self.policy(q)['accepted'],False)

    def test_new_vc_and_skeleton_do_not_add_a_container_around_old_term_arrays(self):
        for b in self.data:
            if 'project_spec' in b:continue
            old=self.policy(base_query(b));self.assertEqual(b['vc']['functions'],old['vc']['functions']);self.assertEqual(b['skeleton']['theorem_declarations'],old['skeleton']['theorem_declarations'])
            self.assertNotIn('program',b['vc']);self.assertNotIn('program',b['skeleton'])

    def test_go85_and84_demands_include_whole_function_obligations(self):
        for name,count in (('go-two-overlays',85),('go-default-contracts',84)):
            b=self.by_id[name];p=b['vc']['obligations'];rs=[r for f in p['functions'] for r in f['requests']]
            self.assertEqual(len(rs),count);self.assertEqual(set(p['unelaborated_request_ids']),{r['request_hash'] for r in rs})
            for f in p['functions']:
                kinds={r['requirement']['kind'] for r in f['requests']};self.assertTrue({'function_source_binding','entry_representation','function_frame','partial_correctness','postcondition'}<=kinds)

    def test_zero_node_function_still_has_source_contract_and_control_demands(self):
        b=self.by_id['go-two-overlays'];empty=next(x['function_decl_id'] for x in b['module']['program']['function_bodies'] if not x['body']['nodes'])
        f=next(x for x in b['vc']['obligations']['functions'] if x['function_decl_id']==empty)
        self.assertTrue({'function_source_binding','entry_representation','region_control','postcondition','function_frame'}<={r['requirement']['kind'] for r in f['requests']})

    def test_all56_project_bound_fragments_retain416_local_demands(self):
        self.assertEqual(len(self.components),56);count=0
        for c in self.components:
            self.assertTrue(all(d['schema']=='mpk.go.operation_definition.v2' for d in c['definitions']))
            nodes={n['id']:n for r in body_regions(c['region']) for n in r['nodes']}
            for site,row in zip(c['sites'],c['local_requirements']):
                got=local_requirements(site,nodes[site['node_id']]);self.assertEqual(row['requirements'],got)
                self.assertEqual(len(got),len(site['members'])+len(site['access']));count+=len(got)
        self.assertEqual(count,416)

    def test_member_coordinates_bind_exact_body_and_concrete_definition(self):
        b=self.by_id['go-two-overlays'];bodies={x['function_decl_id']:x for x in b['module']['program']['function_bodies']}
        for f in b['vc']['obligations']['functions']:
            body=bodies[f['function_decl_id']];sites={s['node_id']:s for s in body['obligation_sites']}
            for r in f['requests']:
                self.assertEqual(r['body_hash'],body['body_hash']);self.assertEqual(r['contract_hash'],body['contract_hash'])
                p=r['requirement']
                if p['kind']=='node_member':self.assertTrue(any(m['category']==p['member_category'] and m['ordinal']==p['member_ordinal'] for m in sites[p['node_id']]['members']))

    def test_call_dependencies_include_unannotated_callee_source_and_contract_goals(self):
        b=self.by_id['go-two-overlays'];requests=[r for f in b['vc']['obligations']['functions'] for r in f['requests']];by_id={r['request_hash']:r for r in requests}
        calls=[r for r in requests if r['requirement']['kind']=='call_relation'];self.assertTrue(calls)
        for call in calls:
            kinds={by_id[d]['requirement']['kind'] for d in call['dependencies']};self.assertTrue({'function_source_binding','function_frame','partial_correctness','postcondition'}<=kinds)
            self.assertEqual(call['requirement']['dependency']['relation']['origin'],{'kind':'default'})

    def changed_go(self,total=False,false_post=False):
        b=self.by_id['go-two-overlays'];spec=copy.deepcopy(b['project_spec'])
        for row in spec['overlays']:
            raw=parse(bytes.fromhex(row['raw_hex']))
            if raw['function']['function']=='Quote':
                if total:raw['termination']='total'
                if false_post:raw['ensures']=[{'op':'bool.literal','value':False}]
                row['raw_hex']=canonical(raw).hex()
        p=Project(spec);v=go_module(spec);m=go_map(v,p);f=frontend_manifest(v,m,go_manifest(p,b['authority']['provenance']))
        source=Artifacts(module_policy(),source_validator(b).policy,b['authority'],spec);a=ProofArtifacts(source,self.policy,None)
        return a,parse(a.generated(v,m,f))

    def test_total_caller_has_conditional_call_totality_without_forging_default_callee_totality(self):
        a,v=self.changed_go(total=True);functions=v['obligations']['functions'];caller=next(f for f in functions if any(r['requirement']['kind']=='call_totality' for r in f['requests']))
        self.assertTrue(any(r['requirement']['kind']=='function_totality' for r in caller['requests']))
        callee=next(f for f in functions if f['function_decl_id']==caller['callees'][0]);self.assertFalse(any(r['requirement']['kind']=='function_totality' for r in callee['requests']))
        with self.assertRaises(OrdinaryElaborationPending):a.expected_skeleton(v)

    def test_false_postcondition_remains_an_unproved_demand(self):
        a,v=self.changed_go(false_post=True)
        self.assertTrue(any(r['requirement']['kind']=='postcondition' and r['requirement']['expression']['payload']=={'value':False} for f in v['obligations']['functions'] for r in f['requests']))
        with self.assertRaises(OrdinaryElaborationPending):a.expected_skeleton(v)

    def test_go_skeleton_request_is_pending_and_never_empty_success(self):
        for name in ('go-two-overlays','go-default-contracts'):
            b=self.by_id[name];a=self.validator(name);a.validate_vc(b['module'],b['source_map'],b['manifest'],b['vc'])
            with self.assertRaises(OrdinaryElaborationPending):a.expected_skeleton(b['vc'])

    def test_caller_cannot_claim_an_expanded_go_ordinary_context(self):
        row=next(r for r in self.rows if r['mutation']=='caller-expansion');b=self.by_id[row['base']]
        with self.assertRaises(ValueError):self.validator(b['id']).validate_vc(b['module'],b['source_map'],b['manifest'],altered(row,b))

    def test_rehashed_go_request_dependency_or_expression_corruption_rejects(self):
        for mutation in ('missing-request','stale-request','missing-dependency','wrong-clause'):
            row=next(r for r in self.rows if r['mutation']==mutation);b=self.by_id[row['base']]
            with self.assertRaises(ValueError):self.validator(b['id']).validate_vc(b['module'],b['source_map'],b['manifest'],altered(row,b))

    def test_certificate_manifest_binds_frontend_and_vc_without_reverse_cycle(self):
        b=self.by_id['go-two-overlays'];self.assertEqual(b['certificate_manifest']['vc_hash'],b['vc']['vc_hash']);self.assertEqual(b['vc']['source_manifest_hash'],b['manifest']['source_manifest_hash'])
        self.assertNotIn('certificate_manifest_hash',b['vc']);v=copy.deepcopy(b['certificate_manifest']);v['source_manifest_hash']='0'*64
        with self.assertRaises(ValueError):self.validator(b['id']).validate_certificate_manifest(b['module'],b['source_map'],b['manifest'],b['vc'],reseal(v,'certificate'))

    def test_generated_skeleton_mutation_cannot_change_the_native_authority(self):
        b=self.by_id['go_fixed'];a=self.validator(b['id']);a.validate_vc(b['module'],b['source_map'],b['manifest'],b['vc'])
        first=a.expected_skeleton(b['vc']);first['theorem_declarations'][0]['name']='Caller.Forged'
        self.assertNotEqual(first,a.expected_skeleton(b['vc']))
        with self.assertRaises(ValueError):a.validate_skeleton(b['module'],b['source_map'],b['manifest'],b['vc'],first)

    def test_native_generator_query_cannot_be_borrowed_from_different_map_authority(self):
        b=self.by_id['go_fixed'];other=self.by_id['go_fixed_synthetic'];a=ProofArtifacts(source_validator(b),self.policy,base_query(other))
        with self.assertRaises(ValueError):a.validate_vc(b['module'],b['source_map'],b['manifest'],b['vc'])

    def test_copied_native_query_survives_caller_mutation(self):
        b=self.by_id['go_fixed'];q=base_query(b);a=ProofArtifacts(source_validator(b),self.policy,q);q['units']=[]
        a.validate_vc(b['module'],b['source_map'],b['manifest'],b['vc'])

    def test_extra_proof_flags_old_schemas_and_crossed_profiles_reject(self):
        b=self.by_id['go_fixed'];a=self.validator(b['id'])
        for field,value in (('verified',True),('schema','mpk.vc.v2'),('profile_contract','mpk.profile.vc.java_scalar.v2')):
            v=copy.deepcopy(b['vc']);v[field]=value
            with self.assertRaises(ValueError):a.validate_vc(b['module'],b['source_map'],b['manifest'],reseal(v,'vc'))

    def test_unknown_native_query_is_never_assumed_rejected(self):
        with self.assertRaises(AssertionError):self.policy({'unobserved':True})

    def test_registered_canonical_boundaries_use768_without_old256_cap(self):
        for n in (256,257,767,768):
            raw=b'['*n+b'0'+b']'*n;self.assertEqual(canonical(parse(raw)),raw);self.assertEqual(canonical(clone(parse(raw))),raw)
        with self.assertRaises(ValueError):parse(b'['*769+b'0'+b']'*769)

    def test_iterative_parser_rejects_duplicate_trailing_and_nonnarrow_json(self):
        for raw in (b'{"a":0,"a":1}',b'[1,]',b'{"a":1,}',b'{"a" 1}',b'1.0',b'NaN',b'1e2',b'truefalse',b'9007199254740992',b'"\\ud800"'):
            with self.assertRaises(ValueError,msg=str(raw)):parse(raw)

    def test_canonical_unicode_narrow_integer_and_domain_rules_match(self):
        self.assertEqual(canonical({'\ue000':0,'\U0001f600':1}),b'{"\xf0\x9f\x98\x80":1,"\xee\x80\x80":0}')
        self.assertEqual(parse(b'-0'),0)
        for d in ('x','MPK.BAD-1.0','MPK-1.x'):
            with self.assertRaises(ValueError):digest(d,{})

    def test_each_go_request_is_uniquely_bound_to_full_source_and_contract(self):
        for b in self.data:
            if 'obligations' not in b['vc']:continue
            ids=[]
            for f in b['vc']['obligations']['functions']:
                for r in f['requests']:
                    self.assertEqual(r['source_ir_hash'],b['module']['vir_hash']);self.assertEqual(r['body_hash'],f['body_hash']);ids.append(r['request_hash'])
            self.assertEqual(len(ids),len(set(ids)))


if __name__=='__main__':unittest.main()
