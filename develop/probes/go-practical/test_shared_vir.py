"""Shared-family dispatch, native preservation and whole-capture Go boundaries."""
import copy
import unittest
from unittest.mock import patch
from body_canonical import canonical,parse,digest
from shared_project import Project,ProjectModel,normalize_contract,SCOPE_SCHEMA,CONTRACT_SCHEMA,OP_SCHEMA,BODY_SCHEMA
from shared_vir import ModuleValidator,legacy_query,go_contract,CAPS
from shared_vir_vectors import bases,specs,altered,project_spec,queries,baseline_queries,rehash
from run_shared_vir import RECORD,policy_from_outputs,check_preservation,hashes
from check_scope_report import strict_json


class SharedTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data=bases();cls.by_id={x['id']:x for x in cls.data};cls.rows=specs();cls.qs=queries(cls.rows,cls.data)
        cls.native=strict_json((RECORD/'retained-native.json').read_bytes());cls.policy=staticmethod(policy_from_outputs(cls.qs,cls.native))

    def validate(self,name,value=None):
        b=self.by_id[name];return ModuleValidator(self.policy,b.get('project_spec')).validate(b['module'] if value is None else value)

    def test_all73_module_cases(self):
        for row in self.rows:
            base=self.by_id[row['base']];value=altered(row,base)
            if row['expected_accepted']:self.validate(row['base'],value)
            else:
                with self.assertRaises(ValueError,msg=row['id']):self.validate(row['base'],value)

    def test_all_five_target_contexts_match_original_typed_projection_and_vcs(self):
        old=strict_json((RECORD/'baseline-native.json').read_bytes())
        self.assertEqual(len(check_preservation(self.data,self.qs,self.native,baseline_queries(),old)),5)

    def test_native_type_validation_and_wp_reject_both_corruptions_for_every_profile(self):
        for name in ('go_fixed','rust64','csharp','java'):
            for mutation in ('unknown-instruction','wrong-operand-type'):
                row=next(r for r in self.rows if r['base']==name and r['mutation']==mutation);m=altered(row,self.by_id[name])
                answer=self.policy(legacy_query(m['semantic_context'],m['program']['units']))
                self.assertIs(answer['accepted'],False);self.assertTrue(answer['error'])

    def test_oracle_never_assumes_unobserved_query_is_rejected(self):
        with self.assertRaises(AssertionError):self.policy({'unobserved':True})

    def test_full_context_and_compiled_profile_contract_control_dispatch(self):
        for name in ('go_fixed','rust64','rust32','csharp','java','go-two-overlays'):
            v=copy.deepcopy(self.by_id[name]['module']);v['semantic_context']['semantic_parameters']['value']['features']=[]
            with self.assertRaises(ValueError):self.validate(name,rehash(v))
            v=copy.deepcopy(self.by_id[name]['module']);v['profile_contract']='mpk.profile.vir.go_fixed.v2' if name!='go_fixed' else 'mpk.profile.vir.rust_checked.v2'
            with self.assertRaises(ValueError):self.validate(name,rehash(v))

    def test_old_and_future_modules_and_unrecognized_fields_are_rejected(self):
        for schema in ('mpk.vir.v1','mpk.vir.v2','mpk.vir.v4'):
            v=copy.deepcopy(self.by_id['go_fixed']['module']);v['schema']=schema
            with self.assertRaises(ValueError):self.validate('go_fixed',rehash(v))
        v=copy.deepcopy(self.by_id['go_fixed']['module']);v['program']['producer_verified']=True
        with self.assertRaises(ValueError):self.validate('go_fixed',rehash(v))

    def test_shared_callee_hash_checked_before_legacy_rehash(self):
        v=copy.deepcopy(self.by_id['rust64']['module'])
        op=next(op for u in v['program']['units'] for f in u['functions'] for b in f['blocks'] for op in b['instructions'] if op['kind']=='CallStatic')
        op['contract_hash']='0'*64
        with self.assertRaises(ValueError):self.validate('rust64',rehash(v))

    def test_one_capture_binds_every_scope_binding_loop_operation_and_body(self):
        for name in ('go-two-overlays','go-default-contracts'):
            p=self.by_id[name]['module']['program'];cap=p['capture_context']['id'];scopes=p['source_scopes']['functions']
            self.assertEqual(len(scopes),3)
            for s in scopes:
                self.assertEqual(s['schema'],SCOPE_SCHEMA);self.assertEqual(s['capture_context_id'],cap)
                self.assertTrue(all(b['capture_context_id']==cap for b in s['bindings']))
                self.assertTrue(all(n['capture_context_id']==cap for n in s['identity_nodes'] if n['schema']=='mpk.go.loop_ref.v1'))
            for b in p['function_bodies']:
                self.assertEqual(b['schema'],BODY_SCHEMA);self.assertEqual(b['capture_context_id'],cap)
                self.assertTrue(all(d['schema']==OP_SCHEMA for d in b['operation_definitions']))

    def test_only_actual_raw_overlays_become_capture_objects(self):
        for enabled,count in ((True,2),(False,0)):
            p=Project(project_spec(enabled));s=p.scope_set(sorted(p.templates))
            self.assertEqual(len(s['overlay_objects']),count)
            self.assertEqual(sum(n['kind']=='overlay' for n in next(iter(p.scopes.values())).graph if n['schema']=='mpk.go.source_object.v1'),count)

    def test_unannotated_error_inputs_admit_all_same_universe_cases_without_raw_sidecar(self):
        p=Project(project_spec(False));decl=p.resolve({'package':'fixture/policy','function':'InputErrors','receiver':None});s=p.scope(decl);c=normalize_contract(s)
        self.assertIsNone(s.raw_bytes);self.assertIsNone(s.overlay_id);self.assertEqual(c['origin'],{'kind':'default'})
        self.assertEqual(c['schema'],CONTRACT_SCHEMA);self.assertEqual(c['termination'],'partial');self.assertEqual(c['requires'],[])
        self.assertTrue(c['error_inputs']);code=s.ids['fixture/policy.RejectCode']
        self.assertTrue(all(r['case_type_ids']==[code] for r in c['error_inputs']))

    def test_default_callee_contract_still_requires_source_body_relation(self):
        v=self.by_id['go-two-overlays']['module'];deps=[d for b in v['program']['function_bodies'] for s in b['obligation_sites'] for d in s['dependencies']]
        self.assertTrue(deps)
        self.assertTrue(all(d['relation']['kind']=='source_body_and_contract' and d['relation']['contract_schema']=='mpk.contract.v3' and d['relation']['origin']=={'kind':'default'} for d in deps))

    def test_raw_overlay_whitespace_changes_all_capture_dependent_ids(self):
        a=Project(project_spec());spec=project_spec();spec['overlays'][0]['raw_hex']=(bytes.fromhex(spec['overlays'][0]['raw_hex'])+b'\n').hex();b=Project(spec)
        self.assertNotEqual(a.capture['id'],b.capture['id'])
        for decl in a.templates:
            x,y=a.scope(decl),b.scope(decl)
            self.assertEqual(x.declaration,y.declaration);self.assertEqual(x.signature,y.signature)
            self.assertNotEqual(x.value['content_sha256'],y.value['content_sha256']);self.assertNotEqual(go_contract(x)['contract_hash'],go_contract(y)['contract_hash'])
            self.assertTrue(set(x.inputs).isdisjoint(y.inputs))

    def test_duplicate_and_unsorted_overlay_paths_reject(self):
        for mutation in ('duplicate','reverse'):
            s=project_spec()
            if mutation=='duplicate':s['overlays'].append(copy.deepcopy(s['overlays'][0]))
            else:s['overlays'].reverse()
            with self.assertRaises(ValueError):Project(s)

    def test_alias_resolved_duplicate_roots_and_overlays_reject(self):
        p=Project(project_spec(False));alias=next(a for a,t in p.facts['aliases'].items() if t=='fixture/policy.RejectCode')
        root={'package':'fixture/policy','function':'Error','receiver':{'package':'fixture/policy','type_name':'RejectCode'}}
        other=copy.deepcopy(root);other['receiver']['type_name']=alias.split('.')[-1]
        self.assertEqual(p.resolve(root),p.resolve(other))
        s=project_spec(False);s['roots']=sorted([root,other],key=canonical)
        with self.assertRaises(ValueError):Project(s)
        d=p.resolve(root);raw=copy.deepcopy(p.scope(d).raw);second=copy.deepcopy(raw);second['function']=other
        s=project_spec(False);s['overlays']=[{'path':f'contracts/{i}.json','raw_hex':canonical(r).hex()} for i,r in enumerate((raw,second))]
        with self.assertRaises(ValueError):Project(s)

    def test_deep_contract_uses_shared_limits_without_old64_depth_hash_cap(self):
        s=project_spec();r=s['overlays'][1];raw=parse(bytes.fromhex(r['raw_hex']));e={'op':'bool.literal','value':True}
        for _ in range(32):e={'op':'bool.not','args':[e]}
        raw['ensures']=[e];r['raw_hex']=canonical(raw).hex();p=Project(s);decl=p.resolve(raw['function'])
        self.assertEqual(normalize_contract(p.scope(decl))['ensures'][0]['op'],'bool.not')

    def test_selected_roots_need_bodies_and_go_sets_have_no_unreachable_extras(self):
        for mutation in ('missing-callee','extra-unreachable','cycle'):
            row=next(r for r in self.rows if r['mutation']==mutation)
            with self.assertRaisesRegex(ValueError,'closure|unreachable'):self.validate(row['base'],altered(row,self.by_id[row['base']]))

    def test_all_module_nodes_and_regions_count_across_functions(self):
        self.assertEqual(self.validate('go-two-overlays'),{'kind':'go_typed_unproved','functions':3,'instructions':9,'regions':3})
        for key,value in (('shared.instructions_per_module',8),('shared.blocks_per_module',2)):
            with patch.dict(CAPS,{key:value}):
                with self.assertRaisesRegex(ValueError,'whole module'):self.validate('go-two-overlays')

    def test_mutable_producer_scope_cannot_override_consumer_raw_authority(self):
        spec=project_spec();p=Project(spec);s=p.scope(p.root_ids[0]);s.context['source_language']='caller'
        with self.assertRaises(ValueError):ProjectModel(p,s.declaration)
        # The consumer accepts only the captured spec, not this modified producer.
        self.validate('go-two-overlays')

    def test_caller_mutation_after_validator_construction_has_no_authority(self):
        spec=project_spec();v=ModuleValidator(self.policy,spec);spec['overlays']=[]
        self.assertEqual(v.validate(self.by_id['go-two-overlays']['module'])['kind'],'go_typed_unproved')

    def test_duplicate_body_and_crossed_contract_sets_reject(self):
        for field in ('function_bodies','contracts'):
            v=copy.deepcopy(self.by_id['go-two-overlays']['module']);v['program'][field].append(copy.deepcopy(v['program'][field][0]))
            with self.assertRaises(ValueError):self.validate('go-two-overlays',rehash(v))

    def test_malformed_go_collections_reject_as_configuration_errors(self):
        for field in ('function_bodies','contracts','source_scopes'):
            for bad in (None,{},[None],[{}]):
                v=copy.deepcopy(self.by_id['go-two-overlays']['module']);v['program'][field]=bad
                with self.assertRaises(ValueError,msg=field):self.validate('go-two-overlays',rehash(v))
        for bad in (None,{},[None],[{}]):
            v=copy.deepcopy(self.by_id['go-two-overlays']['module']);v['program']['function_bodies'][0]['operation_definitions']=bad
            with self.assertRaises(ValueError):self.validate('go-two-overlays',rehash(v))
        for field,bad in (('loops',[None]),('loops',[{}]),('error_inputs',[None]),
                          ('error_inputs',[{'input':0,'path':[0],'cases':[]}]),('termination',{})):
            spec=project_spec();raw=parse(bytes.fromhex(spec['overlays'][0]['raw_hex']));raw[field]=bad
            spec['overlays'][0]['raw_hex']=canonical(raw).hex()
            with self.assertRaises(ValueError):Project(spec)

    def test_new_identity_hashes_cover_every_family_without_schema_substitution(self):
        rows=hashes(self.data);self.assertEqual(len(rows),len({r['id'] for r in rows}))
        domains={r['domain'] for r in rows};self.assertTrue({'MPK-VIR-3.0','MPK-CONTRACT-3.0','MPK-GO-SOURCE-SCOPE-1.0','MPK-GO-SOURCE-SCOPE-SET-1.0','MPK-GO-PRACTICAL-CONTRACT-2.0','MPK-GO-OPERATION-DEFINITION-2.0','MPK-GO-FUNCTION-BODY-2.0'}<=domains)


if __name__=='__main__':unittest.main()
