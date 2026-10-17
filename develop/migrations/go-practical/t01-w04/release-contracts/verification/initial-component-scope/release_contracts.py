"""Inactive release v3 graph; metadata validation cannot grant runtime readiness."""
from functools import lru_cache
import json
from proof_canonical import canonical, parse, clone, digest, seal
from identity_catalog import SHARED
from policy_sources import source_rows
from capture_artifact_vectors import release_fixture
from operation_signatures import exact, need
from run_proof_pilot import ROOT, sha

ACTIVE=ROOT/'release/bundles/bundle-registry.json'
CONTENT_DOMAIN='MPK-BUNDLE-CONTENT-1.0'
GATES=('source_artifact_migration','build_materialization_twice','paired_checker_corpus',
    'four_profile_preservation','practical_go_application_corpus','installed_replay_twice',
    'native_linux_isolation_resources','atomic_activation_rollback')
LIMITS={'canonical_bytes':67108864,'transport_bytes':67108865,'depth':256,'string_bytes':1048576,
    'bundle_descriptors':1024,'tuples':4096,'hosts':256,'runtime_layouts':256,'components':8192,
    'serialized_file_records':262144,'unique_files_per_bundle':262144,'path_bytes':1024,
    'file_bytes':4294967296,'declared_bytes_per_bundle':34359738368}
FIELDS={
 'frontend_bundle':('schema','bundle_id','name','version','profile_contracts','main','subordinate_binaries','inventory','file_modes','build_origin','bundle_sha256'),
 'toolchain_bundle':('schema','bundle_id','execution_host_profile_id','profile_contracts','components','inventory','file_modes','build_origin','distribution_sha256'),
 'bundle_registry':('schema','id','profile_registry','execution_host_profiles','native_runtime_layout_profiles','frontend_bundles','toolchain_bundles','tuples','registry_sha256'),
 'release_registry':('schema','id','profile_registry','bundle_registry','schema_set','consumer','image_inventory','release_sha256'),
 'bundle_candidate':('schema','release_registry','bundle_registry','required_contexts','missing','state'),
 'release_evidence':('schema','candidate','release_registry','checks','environment','state'),
 'release_receipt':('schema','candidate','release_registry','evidence','status','proof_authority','receipt_hash')}
HASH_FIELDS={'frontend_bundle':'bundle_sha256','toolchain_bundle':'distribution_sha256',
 'bundle_registry':'registry_sha256','release_registry':'release_sha256','release_receipt':'receipt_hash'}


def domain(f):return CONTENT_DOMAIN if f in ('frontend_bundle','toolchain_bundle') else SHARED[f][1]
def sealed(f,body):
    return seal(SHARED[f][0],domain(f),body,HASH_FIELDS[f]) if f in HASH_FIELDS else {'schema':SHARED[f][0],**body}
def ref(value):return {'schema':value['schema'],'canonical_sha256':sha(canonical(value))}
def release_ref(value):return {k:value[k] for k in ('schema','id','registry_sha256')}
def bounds(raw):
    need(type(raw)is bytes and len(raw)<=LIMITS['canonical_bytes'],'release canonical byte budget')
    v=parse(raw);stack=[(v,0)]
    while stack:
        node,d=stack.pop()
        if type(node)in(dict,list):
            need(d<LIMITS['depth'],'release container depth');stack.extend((x,d+1)for x in(node.values()if type(node)is dict else node))
    return v
def wire(value):raw=canonical(value);bounds(raw);return raw+b'\n'
def read_wire(raw):
    need(type(raw)is bytes and len(raw)<=LIMITS['transport_bytes']and raw.endswith(b'\n'),'release transport framing')
    v=bounds(raw[:-1]);need(wire(v)==raw,'canonical release transport plus one LF');return v


@lru_cache(None)
def _active_bytes():return ACTIVE.read_bytes()
def active():return json.loads(_active_bytes())
def contexts():
    rows={canonical(r['base']['module']['semantic_context']):r['base']['module']['semantic_context']for r in source_rows()}
    return [clone(rows[k])for k in sorted(rows)]
def key(context):return sha(canonical(context))
def new_id(old):return old+'.go-practical-wire.v1'


def descriptor(f,old,ctxs):
    b={k:clone(v)for k,v in old.items()if k not in ('schema',HASH_FIELDS[f],'inventory','profile_contracts')}
    b['bundle_id']=new_id(old['bundle_id'])
    old_refs={p['profile_entry_sha256']:p for p in old['profile_contracts']}
    from context_protocol import legacy_projection
    contracts=[]
    for c in ctxs:
        old_entry=legacy_projection(c)['context']['profile_entry_sha256']
        if old_entry not in old_refs:continue
        kind=next(r['base']['module']['program']['kind']for r in source_rows()if r['base']['module']['semantic_context']==c)
        role='frontend'if f=='frontend_bundle'else'release'
        p=old_refs[old_entry];envelope={'profile_entry_sha256':c['profile_entry_sha256'],
            'contract_id':'mpk.profile.'+role+'.'+kind+'.v2','value':clone(p['value'])}
        if envelope not in contracts:contracts.append(envelope)
    contracts.sort(key=lambda p:p['profile_entry_sha256']);b['profile_contracts']=contracts
    files=old['inventory']['files']
    b['inventory']={'schema':'mpk.release.bundle_inventory.v1','members':[
        {'path':p['path'],'raw_sha256':p['sha256'],'size_bytes':p['size_bytes']}for p in files]}
    b['file_modes']=[{'path':p['path'],'executable':p['executable']}for p in files]
    b['build_origin']={'kind':'legacy_metadata_only','source_schema':old['schema'],'source_descriptor_sha256':sha(canonical(old))}
    return sealed(f,b)


class ReleaseModel:
    def __init__(self,native):
        old=active();need(native.get('accepted')is True and native['source_transport_sha256']==sha(_active_bytes())and native['registry_sha256']==old['registry_sha256'],'actual frozen old validator observation')
        need((native['frontends'],native['toolchains'],len(native['tuples']))==(4,4,5),'complete retained release')
        need(all(x['accepted']is False for x in native['controls']),'actual old-version/hash/shape controls')
        self.native=clone(native);self.old=old;self.contexts=contexts()
        need(len(self.contexts)==6 and len({canonical(c['profile_registry'])for c in self.contexts})==1,'five profiles/six complete target contexts')
        self.docs=self.build();self._expected={}
        for f,values in self.docs.items():
            for i,v in enumerate(values if type(values)is list else [values]):self._expected[f,i]=canonical(v)
    def build(self):
        ctxs=[c for c in self.contexts if c['semantic_profile']!='mpk.go.practical.v1'];old=self.old
        fs=[descriptor('frontend_bundle',b,ctxs)for b in old['frontend_bundles']]
        ts=[descriptor('toolchain_bundle',b,ctxs)for b in old['toolchain_bundles']]
        tuples=[]
        for c in ctxs:
            from context_protocol import legacy_projection
            old_c=legacy_projection(c)['context'];matches=[t for t in old['tuples']if canonical(t['semantic_context'])==canonical(old_c)]
            need(len(matches)==1,'one exact retained native target tuple');t=matches[0]
            tuples.append({'semantic_context':c,'limit_profile_id':t['limit_profile_id'],
                'frontend_bundle_id':new_id(t['frontend_bundle_id']),'toolchain_bundle_id':new_id(t['toolchain_bundle_id'])})
        tuples.sort(key=lambda t:canonical(t['semantic_context']))
        registry=sealed('bundle_registry',{'id':SHARED['release_registry'][0],'profile_registry':self.contexts[0]['profile_registry'],
            'execution_host_profiles':old['execution_host_profiles'],'native_runtime_layout_profiles':old['native_runtime_layout_profiles'],
            'frontend_bundles':fs,'toolchain_bundles':ts,'tuples':tuples})
        image=sealed('release_registry',{'id':SHARED['release_registry'][0],'profile_registry':registry['profile_registry'],
            'bundle_registry':release_ref(registry),'schema_set':{k:v[0]for k,v in SHARED.items()},'consumer':None,'image_inventory':None})
        missing=[{'kind':'consumer_build','id':'bin/mpk'}]+[
            {'kind':'bundle_materialization','id':b['bundle_id']}for b in fs+ts]+[
            {'kind':'tuple','id':key(c)}for c in self.contexts if not any(canonical(t['semantic_context'])==canonical(c)for t in tuples)]
        missing.sort(key=lambda r:(r['kind'],r['id']))
        candidate=sealed('bundle_candidate',{'release_registry':ref(image),'bundle_registry':release_ref(registry),
            'required_contexts':self.contexts,'missing':missing,'state':'candidate_incomplete'})
        evidence=sealed('release_evidence',{'candidate':ref(candidate),'release_registry':ref(image),
            'checks':[{'name':g,'status':'not_run','observations':[]}for g in GATES],
            'environment':{'kind':'design_metadata_only','native_release_observed':False},'state':'not_run'})
        receipt=sealed('release_receipt',{
            'candidate':ref(candidate),'release_registry':ref(image),'evidence':ref(evidence),'status':'candidate_incomplete','proof_authority':'none'})
        return {'frontend_bundle':fs,'toolchain_bundle':ts,'bundle_registry':registry,'release_registry':image,
            'bundle_candidate':candidate,'release_evidence':evidence,'release_receipt':receipt}
    def validate(self,f,value,index=0):
        exact(value,FIELDS[f])
        need(canonical(value)==self._expected[f,index],'whole release graph regenerated from independent frozen source/metadata authority')
        return {'status':'metadata_only','runtime_ready':False,'proof_authority':'none'}
    def import_json(self,f,raw,index=0):return self.validate(f,read_wire(raw),index)
    def resolve(self,context,frontend_id,toolchain_id):
        need(any(canonical(context)==canonical(c)for c in self.contexts),'registered complete context')
        rows=[t for t in self.docs['bundle_registry']['tuples']if canonical(t['semantic_context'])==canonical(context)and t['frontend_bundle_id']==frontend_id and t['toolchain_bundle_id']==toolchain_id]
        need(len(rows)==1,'tuple unavailable; no GoFixed fallback')
        raise ValueError('bundle_materialization_and_new_protocol_build_required')
    def authorize_activation(self,receipt):
        self.validate('release_receipt',receipt)
        raise ValueError('T07_native_release_evidence_and_atomic_install_required')
    def bind_frontend_reference(self,reference):
        need(canonical(reference)==canonical(release_ref(self.docs['bundle_registry'])),'actual full bundle registry reference,not the historical linkage fixture')


def catalogue():
    return {'schema':'mpk.go_practical.release_contract_draft.v0','roots':{SHARED[f][0]:list(fields)for f,fields in FIELDS.items()},
        'hash_fields':HASH_FIELDS,'hash_domains':{f:domain(f)for f in HASH_FIELDS},'limits':LIMITS,
        'inventory_v1_exact_fields':{'root':['schema','members'],'member':['path','raw_sha256','size_bytes']},
        'file_modes':['path','executable'],'build_origin_arms':{
            'legacy_metadata_only':['kind','source_schema','source_descriptor_sha256'],
            'reproduced':['kind','source_inputs_ref','toolchain_inputs_ref','build_recipe_id','replay_observation_refs']},
        'inherited_execution_shapes':'exact ExecutableRecord/RuntimeLibrary/static|dynamic runtime,executable|content ToolchainComponent (content inventory.v0 unchanged),ExecutionHostProfile/NativeRuntimeLayoutProfile/InterpreterMount/LibraryMount from pinned release_bundle.rs; no fields are widened',
        'candidate_fields':list(FIELDS['bundle_candidate']),'tuple_fields':['semantic_context','limit_profile_id','frontend_bundle_id','toolchain_bundle_id'],
        'required_gates':list(GATES),'gate_record_fields':['name','status','observations'],
        'gate_statuses':['not_run','passed','failed','environment_error'],
        'observation_fields':['run_id','command_id','source_epoch_ref','release_registry_ref','input_inventory_ref','output_inventory_ref','exit_code','stdout_sha256','stderr_sha256','host_probe_ref'],
        'evidence_environment_arms':{'design_metadata_only':['kind','native_release_observed'],'native_linux':['kind','host_profile_ref','host_probe_ref','gate_driver_ref']},
        'consumer_fields':['binary_ref','build_inputs_ref','compiled_semantic_registry_ref','compiled_bundle_registry_ref','compiled_schema_set_ref','checker_build_refs'],
        'image_inventory':'nullable only in incomplete candidate; materialized={schema:mpk.release.bundle_inventory.v1,members:[path,raw_sha256,size_bytes]}; file modes in separately bound consumer/bundle layout; exact installed files are bin/mpk,share/mpk/{semantic-profile-registry,bundle-registry}.json,and registered bundle trees; own release manifest is excluded from its inventory to avoid self hashing',
        'receipt_statuses':['candidate_incomplete','release_verified'],'receipt_authority':'helper_release_only; no source theorem or package proof acceptance',
        'readiness_rule':'release_verified requires exact complete context/bundle/consumer coverage,actual independent byte/mode snapshots,proven reproducible origins,two matching complete native gate runs,and no missing/failed/environment-error checks. Import regenerates every result from separately captured local gate authority; JSON flags/hashes are never execution authority. No such authority exists in this model.',
        'hash_DAG':'semantic registry + built frontends/toolchains -> bundle registry -> consumer compilation embeds those refs -> outer release registry binds actual consumer/image bytes -> candidate -> evidence -> receipt -> external activation journal; consumer never embeds its own outer release hash',
        'activation':'prepare immutable owner-controlled image; open/validate descriptor-relative complete snapshot; verify exact candidate/receipt and compare expected previous image+generation; commit one atomic image pointer after fsync; old open handles retain old image; failure leaves pointer unchanged; rollback swaps whole image with new generation,no mixed families or mutable path/environment override',
        'historical_linkage_fixture':release_fixture()['reference'],
        'preservation':'keep old inventory.v0 content hashes and execution/runtime/component values byte-exact in metadata projections; new IDs and new schema3 descriptor content hashes include full inventory1,file modes,execution and origin. Projections cannot run as protocol3 binaries. Old observations stay attached to their source epoch; T02 regenerates frontend/source hashes against real new release refs.',
        'production_owners':['crates/mpk-vc/src/go_practical_release.rs (T02 private contracts,T07 activation)','crates/mpk-cli/src/go_practical_release.rs (T07)','scripts/build-release-bundles.sh (T07)','scripts/check-release-bundles.sh (T07)'],
        'remaining':['ordinary program/materialized policy and exact Go execution-host/resource contract closure; all reviewed consumer migrations and historical/new source epochs','W05 inactive publication/T01 gate; all T02–T11 implementation/proof/native release gates']}
