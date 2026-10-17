"""Capture/source-reference authority and actual retained importer boundaries."""
import copy
import json
import subprocess
import unittest
from unittest.mock import patch
from body_canonical import canonical,digest
from capture_artifacts import Artifacts,input_inventory,go_buffers,go_map,go_manifest,source_map,frontend_manifest,CAPS
from capture_artifact_vectors import bases,specs,queries,altered,reseal
from run_capture_artifacts import RECORD,module_policy,evaluate,baseline_queries,preservation,hashes
from run_shared_vir import policy_from_outputs
from capture_native_build import TARGET
from shared_project import Project,ProjectModel
from shared_vir import body,go_program,module,ModuleValidator
from shared_vir_vectors import project_spec,rehash
from vir_body_build import Builder
from operation_signatures import source,STATE
from context_protocol import active_registry
from identity_catalog import SHARED
from run_proof_pilot import json_bytes


class CaptureTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data=bases();cls.by_id={b['id']:b for b in cls.data};cls.rows=specs();cls.qs=queries(cls.data,cls.rows)
        cls.native=json.loads((RECORD/'retained-native.json').read_bytes());cls.old=json.loads((RECORD/'baseline-native.json').read_bytes())
        cls.policy=staticmethod(policy_from_outputs(cls.qs,cls.native));cls.mp=staticmethod(module_policy())

    def validator(self,name):
        b=self.by_id[name];return Artifacts(self.mp,self.policy,b['authority'],b.get('project_spec'))

    def test_all187_cases_and_exact_classifications(self):
        out=evaluate(self.data,self.rows,self.policy)
        self.assertEqual(len(out),187);self.assertEqual(sum(x['classification']=='configuration_rejection' for x in out),171)

    def test_five_original_or_independent_native_baselines_match_full_values(self):
        self.assertEqual(len(preservation(self.data,self.qs,self.native,baseline_queries(self.data),self.old)),5)

    def test_every_retained_input_and_selection_mutation_reaches_actual_native_rejection(self):
        for row in self.rows:
            if row['base'] not in ('go_fixed','rust64','rust32','csharp','java') or row['mutation'] not in ('input-sha','input-size','missing-input','selection','target'):continue
            b=self.by_id[row['base']];a=self.validator(b['id']);q=a.query(b['module'],b['source_map'],altered(row,b))
            answer=self.policy(q);self.assertIs(answer['accepted'],False);self.assertTrue(answer['error'])

    def test_new_root_schema_and_profile_contracts_cannot_cross(self):
        b=self.by_id['go-two-overlays'];a=self.validator(b['id'])
        for stage,value in (('map',b['source_map']),('manifest',b['manifest'])):
            for key,bad in (('schema','mpk.source_manifest.certificate.v3'),('profile_contract','mpk.profile.vir.go_practical.v2')):
                v=copy.deepcopy(value);v[key]=bad;v=reseal(v,stage)
                with self.assertRaises(ValueError):
                    a.validate_map(b['module'],v) if stage=='map' else a.validate_manifest(b['module'],b['source_map'],v)

    def test_full_map_context_vir_hash_and_manifest_map_hash_are_bound(self):
        b=self.by_id['go-two-overlays'];a=self.validator(b['id'])
        for field in ('source_ir_hash','source_ir_schema'):
            v=copy.deepcopy(b['source_map']);v[field]='0'*64
            with self.assertRaises(ValueError):a.validate_map(b['module'],reseal(v,'map'))
        v=copy.deepcopy(b['manifest']);v['source_map_hash']='0'*64
        with self.assertRaises(ValueError):a.validate_manifest(b['module'],b['source_map'],reseal(v,'manifest'))

    def test_old_or_certificate_roots_and_proof_status_members_reject(self):
        b=self.by_id['go_fixed'];a=self.validator(b['id'])
        for schema in ('mpk.source_manifest.v1',SHARED['certificate_manifest'][0]):
            v=copy.deepcopy(b['manifest']);v['schema']=schema
            with self.assertRaises(ValueError):a.validate_manifest(b['module'],b['source_map'],reseal(v,'manifest'))
        v=copy.deepcopy(b['manifest']);v['vc_hash']=None
        with self.assertRaises(ValueError):a.validate_manifest(b['module'],b['source_map'],reseal(v,'manifest'))
        v=copy.deepcopy(b['source_map']);v['verified']=True
        with self.assertRaises(ValueError):a.validate_map(b['module'],reseal(v,'map'))

    def test_go_three_functions_nine_nodes_three_terminators_are_all_mapped(self):
        for name in ('go-two-overlays','go-default-contracts'):
            b=self.by_id[name];entries=b['source_map']['map']['entries'];self.assertEqual(len(entries),15)
            self.assertEqual([sum(e['reference']['kind']==k for e in entries) for k in ('function','instruction','terminator')],[3,9,3])
            self.validator(name).validate_map(b['module'],b['source_map'])

    def test_nested_if_scope_and_resume_continuation_have_complete_go_map_coverage(self):
        spec=project_spec(False);spec['roots']=[{'package':'fixture/policy','function':'Scopes','receiver':None}]
        p=Project(spec);decl=p.root_ids[0];m=ProjectModel(p,decl);builder=Builder(m)
        r=builder.region([source(m.id(x['type'])) for x in m.scope.function['inputs']]+[STATE])
        def scoped(c):
            flag=c.zero(m.id('bool'));control=c.apply(15,{'condition_type':m.id('bool')},[flag,c.state])[0];c.yield_([control])
        control=r.scope(scoped);r.resume(control,lambda c:c.return_zero())
        v=module(m.scope.context,go_program(p,[body(p,decl,r.r,builder.definitions())]));count=ModuleValidator(self.mp,spec).validate(v)
        mapped=go_map(v,p);a=Artifacts(self.mp,self.policy,self.by_id['go-default-contracts']['authority'],spec)
        a.validate_map(v,mapped);self.assertEqual(len(mapped['map']['entries']),count['functions']+count['instructions']+count['regions'])
        self.assertGreater(count['regions'],3)

    def test_go_input_inventory_contains_only_actual_raw_objects(self):
        for enabled,count in ((True,4),(False,2)):
            p=Project(project_spec(enabled));inv=input_inventory(p);raw=go_buffers(p)
            self.assertEqual(len(inv['objects']),count);self.assertEqual(set(raw),{x['id'] for x in inv['objects']})
            self.assertEqual(sum(x['kind']=='overlay' for x in inv['objects']),2 if enabled else 0)

    def test_rehashed_go_inventory_and_capture_forgeries_are_rejected(self):
        for mutation in ('inventory-sha','missing-input','capture','build-config'):
            row=next(x for x in self.rows if x['base']=='go-two-overlays' and x['stage']=='manifest' and x['mutation']==mutation)
            b=self.by_id[row['base']]
            with self.assertRaises(ValueError):self.validator(b['id']).validate_manifest(b['module'],b['source_map'],altered(row,b))

    def test_same_bytes_different_overlay_path_stale_manifest_rejects(self):
        old=self.by_id['go-two-overlays'];spec=copy.deepcopy(old['project_spec']);spec['overlays'][0]['path']='contracts/00-renamed.json';spec['overlays'].sort(key=lambda r:r['path'])
        from shared_vir_vectors import go_module
        p=Project(spec);v=go_module(spec);mapped=go_map(v,p);a=Artifacts(self.mp,self.policy,old['authority'],spec)
        stale=frontend_manifest(v,mapped,old['manifest']['manifest'])
        with self.assertRaises(ValueError):a.validate_manifest(v,mapped,stale)

    def test_cached_vir_is_keyed_by_all_bytes_and_does_not_trust_reused_hash(self):
        b=self.by_id['go-two-overlays'];a=self.validator(b['id']);a.validate_map(b['module'],b['source_map'])
        v=copy.deepcopy(b['module']);v['program']['function_bodies'][0]['body']['nodes'][0]['id']=999;rehash(v)
        with self.assertRaises(ValueError):a.validate_map(v,source_map(v,b['source_map']['map']))

    def test_constructor_copies_authority_and_project_before_caller_mutation(self):
        b=copy.deepcopy(self.by_id['go-two-overlays']);a=Artifacts(self.mp,self.policy,b['authority'],b['project_spec']);b['authority']['provenance']['frontend']['name']='caller';b['project_spec']['overlays']=[]
        original=self.by_id[b['id']];a.validate_manifest(original['module'],original['source_map'],original['manifest'])

    def test_external_release_reference_cannot_be_rehashed_into_authority(self):
        for name in ('go_fixed','go-two-overlays'):
            b=self.by_id[name];v=copy.deepcopy(b['manifest']);v['manifest']['release_registry']['registry_sha256']='0'*64
            with self.assertRaises(ValueError):self.validator(name).validate_manifest(b['module'],b['source_map'],reseal(v,'manifest'))

    def test_native_synthetic_permission_is_independent_and_exact(self):
        b=self.by_id['go_fixed_synthetic'];a=self.validator(b['id']);a.validate_manifest(b['module'],b['source_map'],b['manifest'])
        row=next(x for x in self.rows if x['mutation']=='wrong-reason');v=altered(row,b)
        self.assertIs(self.policy(a.query(b['module'],v))['accepted'],False)

    def test_untrusted_map_cannot_grant_its_own_synthetic_permission(self):
        b=self.by_id['go_fixed'];v=copy.deepcopy(b['source_map']);v['map']['synthetic_permissions']=[]
        with self.assertRaises(ValueError):self.validator(b['id']).validate_map(b['module'],reseal(v,'map'))

    def test_native_java_rejects_even_external_synthetic_permission(self):
        b=self.by_id['java'];a=self.validator(b['id']);q=a.query(b['module'],b['source_map'])
        q['synthetic_permissions']=[{'reference':b['source_map']['map']['entries'][-1]['reference'],'reason':'profile.private.test'}]
        result=subprocess.run([TARGET/'debug/go-practical-capture-native'],input=json_bytes({'registry':active_registry(),'queries':[q]}),stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=30)
        self.assertEqual(result.returncode,0);self.assertIs(json.loads(result.stdout)[0]['accepted'],False)

    def test_native_utf8_split_ranges_reject_for_both_scalar_profiles(self):
        for name in ('csharp','java'):
            b=self.by_id[name];a=self.validator(name);row=next(x for x in self.rows if x['base']==name and x['mutation']=='split-utf8')
            self.assertIs(self.policy(a.query(b['module'],altered(row,b)))['accepted'],False)

    def test_case_colliding_paths_in_one_capture_capability_reject(self):
        spec=project_spec();spec['overlays'][0]['path']='contracts/A.json';spec['overlays'][1]['path']='contracts/a.json'
        with self.assertRaises(ValueError):input_inventory(Project(spec))

    def test_capture_input_contract_and_byte_budgets_are_enforced(self):
        for key,value in (('manifest.inputs',3),('capture.contract_candidates_explicit_contract_paths',1),('capture.bytes_per_captured_candidate',1789)):
            with patch.dict(CAPS,{key:value}):
                with self.assertRaises(ValueError):input_inventory(Project(project_spec()))

    def test_source_map_and_manifest_apply_their_own_complete_byte_and_entry_caps(self):
        b=self.by_id['go-two-overlays'];a=self.validator(b['id'])
        for name,n in (('MAP_ENTRIES_MAX',14),('MAP_CANONICAL_MAX',len(canonical(b['source_map']))-1)):
            with patch('capture_artifacts.'+name,n):
                with self.assertRaises(ValueError):a.validate_map(b['module'],b['source_map'])
        with patch('capture_artifacts.MAN_CANONICAL_MAX',len(canonical(b['manifest']))-1):
            with self.assertRaises(ValueError):a.validate_manifest(b['module'],b['source_map'],b['manifest'])

    def test_malformed_collections_and_provenance_reject_without_type_crashes(self):
        b=self.by_id['go-two-overlays'];a=self.validator(b['id'])
        for bad in (None,{},[None]):
            m=copy.deepcopy(b['source_map']);m['map']['entries']=bad
            with self.assertRaises(ValueError):a.validate_map(b['module'],reseal(m,'map'))
            m=copy.deepcopy(b['manifest']);m['manifest']['input_inventory']=bad
            with self.assertRaises(ValueError):a.validate_manifest(b['module'],b['source_map'],reseal(m,'manifest'))
        authority=copy.deepcopy(b['authority']);authority['provenance']['toolchain']['components']=[None]
        with self.assertRaises(ValueError):Artifacts(self.mp,self.policy,authority,b['project_spec'])

    def test_missing_native_query_is_an_infrastructure_error(self):
        b=self.by_id['go_fixed'];v=copy.deepcopy(b['source_map']);v['map']['entries'][0]['origin']['start']+=1
        with self.assertRaises(AssertionError):self.validator(b['id']).validate_map(b['module'],reseal(v,'map'))

    def test_canonical_hash_vectors_cover_both_roots_inventory_and_external_fixture(self):
        rows=hashes(self.data);self.assertEqual(len(rows),19);self.assertEqual(len({r['id'] for r in rows}),19)
        self.assertEqual(sum(r['domain']=='MPK-GO-INPUT-INVENTORY-1.0' for r in rows),2)


if __name__=='__main__':unittest.main()
