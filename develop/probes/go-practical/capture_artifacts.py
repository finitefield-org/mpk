"""Closed common maps/frontend manifests and complete Go fixture input inventories."""
import copy
import re
from body_canonical import canonical,digest,seal,parse,CAPS
from shared_project import Project
from shared_vir import ModuleValidator,KINDS,legacy_query
from context_protocol import GO,go_selection
from identity_catalog import SHARED
from identity_wire import path
from run_proof_pilot import HERE,sha
from operation_signatures import exact,need

MAP_FIELDS=('schema','semantic_context','profile_contract','source_ir_schema','source_ir_hash','map','source_map_hash')
MAN_FIELDS=('schema','semantic_context','profile_contract','vir_hash','source_map_hash','manifest','source_manifest_hash')
RETAINED_MAN_FIELDS=('kind','selection','limit_profile','release_registry','toolchain','frontend','units','target','inputs','input_set_hash')
GO_MAN_FIELDS=('kind','selection','limit_profile','release_registry','toolchain','frontend','capture_context','input_inventory','build_config','modules','packages')
PROVENANCE=('release_registry','toolchain','frontend')
INVENTORY_SCHEMA='mpk.go.input_inventory.v1';INVENTORY_DOMAIN='MPK-GO-INPUT-INVENTORY-1.0'
MAP_CANONICAL_MAX=33554432;MAP_ENTRIES_MAX=323728;MAN_CANONICAL_MAX=4194304


def profile_contract(context,kind):
    need(kind in {'source_map','manifest'},'closed artifact profile contract')
    return 'mpk.profile.'+kind+'.'+KINDS[context['semantic_profile']]+'.v2'


def source_map(vir,payload):
    return seal(SHARED['source_map'][0],SHARED['source_map'][1],{'semantic_context':vir['semantic_context'],
        'profile_contract':profile_contract(vir['semantic_context'],'source_map'),'source_ir_schema':SHARED['vir'][0],
        'source_ir_hash':vir['vir_hash'],'map':payload},'source_map_hash')


def frontend_manifest(vir,map_,payload):
    return seal(SHARED['frontend_manifest'][0],SHARED['frontend_manifest'][1],{'semantic_context':vir['semantic_context'],
        'profile_contract':profile_contract(vir['semantic_context'],'manifest'),'vir_hash':vir['vir_hash'],
        'source_map_hash':map_['source_map_hash'],'manifest':payload},'source_manifest_hash')


def provenance_shape(provenance):
    exact(provenance,PROVENANCE);release=provenance['release_registry'];exact(release,{'schema','id','registry_sha256'})
    need(release['schema']==SHARED['bundle_registry'][0] and release['id']==SHARED['release_registry'][0],'successor release reference family')
    def hash_(s):need(type(s) is str and re.fullmatch('[0-9a-f]{64}',s) is not None,'lowercase SHA256 identity')
    hash_(release['registry_sha256']);f=provenance['frontend'];t=provenance['toolchain']
    exact(f,{'bundle_id','name','version','binary_sha256','subordinate_binaries'});exact(t,{'bundle_id','distribution_sha256','components'})
    for row,keys in ((f,('bundle_id','name','version')),(t,('bundle_id',))):
        need(all(type(row[k]) is str and row[k] for k in keys),'literal provenance identifiers')
    hash_(f['binary_sha256']);hash_(t['distribution_sha256'])
    need(type(f['subordinate_binaries']) is list and len(f['subordinate_binaries'])<=CAPS['manifest.frontend_subordinates'],'Go frontend subordinate bound')
    need(type(t['components']) is list and len(t['components'])<=CAPS['manifest.toolchain_components'],'toolchain component bound')
    names=[]
    for c in t['components']:
        need(type(c) is dict and c.get('kind') in ('executable','content'),'closed component identity kind')
        hash_field='binary_sha256' if c['kind']=='executable' else 'content_sha256'
        fields={'kind','name','release',hash_field}
        if c['kind']=='executable' and 'commit_hash' in c:fields.add('commit_hash');need(type(c['commit_hash']) is str,'literal optional commit')
        exact(c,fields);need(type(c['name']) is str and c['name'] and type(c['release']) is str,'literal component name/release');hash_(c[hash_field]);names.append(c['name'])
    need(names==sorted(set(names),key=lambda s:s.encode()),'unique byte-ordered component identities')


def project_nodes(project):
    scope=project.scope(next(iter(project.templates)))
    return {n['id']:n for n in scope.graph}


def go_buffers(project):
    """Original byte authority for the pinned fixture, not a filesystem importer."""
    nodes=project_nodes(project);rows={}
    overlay={v['object']['id']:v['raw'] for v in project.overlays.values()}
    for id_ in project.capture['objects']:
        n=nodes[id_]
        if n['kind']=='go_source':raw=(HERE/'overlay-fixture.go.txt').read_bytes()
        elif n['kind']=='go_mod':raw=b'module fixture/policy\n\ngo 1.25.0\n'
        else:need(id_ in overlay,'every actual overlay has original bytes');raw=overlay[id_]
        need(len(raw)==n['bytes'] and sha(raw)==n['raw_sha256'],'capture object matches immutable original bytes')
        rows[id_]=raw
    return rows


def input_inventory(project):
    nodes=project_nodes(project);buffers=go_buffers(project);objects=[nodes[i] for i in project.capture['objects']]
    fold=str.maketrans('ABCDEFGHIJKLMNOPQRSTUVWXYZ','abcdefghijklmnopqrstuvwxyz')
    paths=[(n['capability'],n['path'].translate(fold)) for n in objects]
    need(len(paths)==len(set(paths)),'one case-stable captured path per capability')
    need(len(objects)<=CAPS['manifest.inputs'],'manifest input count')
    contracts=[n for n in objects if n['kind']=='overlay'];other=[n for n in objects if n['kind']!='overlay']
    need(len(contracts)<=CAPS['capture.contract_candidates_explicit_contract_paths'] and sum(n['bytes'] for n in contracts)<=CAPS['capture.total_contract_candidate_bytes'],'contract capture count/total bytes')
    need(all(n['bytes']<=CAPS['capture.bytes_per_contract_candidate'] for n in contracts),'contract capture per-file bytes')
    need(sum(n['bytes'] for n in other)<=CAPS['capture.total_captured_candidate_bytes'] and all(n['bytes']<=CAPS['capture.bytes_per_captured_candidate'] for n in other),'source capture bytes')
    need(set(buffers)==set(project.capture['objects']),'no missing/fabricated raw input')
    context=next(iter(project.templates.values())).context
    return seal(INVENTORY_SCHEMA,INVENTORY_DOMAIN,{'semantic_context':context,'capture_context_id':project.capture['id'],'objects':objects},'content_sha256')


def go_entries(vir,project):
    """Exact complete function/node/terminator locators; no AST-equivalence theorem."""
    nodes=project_nodes(project);buffers=go_buffers(project);entries=[]
    def add(ref,anchor):
        exact(anchor,{'source_object_id','start_byte','end_byte'});id_=anchor['source_object_id'];need(id_ in buffers and nodes[id_]['kind']=='go_source','captured Go source origin')
        a,b=anchor['start_byte'],anchor['end_byte'];need(type(a) is int and type(b) is int and 0<=a<b<=len(buffers[id_]),'nonempty in-bounds captured source span')
        try:buffers[id_].decode('utf-8');buffers[id_][:a].decode('utf-8');buffers[id_][:b].decode('utf-8')
        except UnicodeError as e:raise ValueError('UTF8 source boundaries') from e
        entries.append({'reference':ref,'origin':{'kind':'captured_span',**anchor}})
    for body in vir['program']['function_bodies']:
        decl=body['function_decl_id'];f=nodes[decl];anchor={k:f[k] for k in ('source_object_id','start_byte','end_byte')}
        add({'kind':'function','function_decl_id':decl},anchor)
        def region(r,enclosing):
            for n in r['nodes']:
                a={k:n['source_ref'][k] for k in ('source_object_id','start_byte','end_byte')}
                add({'kind':'instruction','function_decl_id':decl,'node_id':n['id']},a)
                if n['kind']=='apply':
                    for child in n['regions']:region(child['region'],a)
                elif n['kind']=='scope':region(n['region'],a)
            add({'kind':'terminator','function_decl_id':decl,'region_id':r['id']},enclosing)
            if r['terminator']['kind']=='resume':region(r['terminator']['continuation'],enclosing)
        region(body['body'],anchor)
    def key(e):
        r=e['reference'];return r['function_decl_id'],{'function':0,'instruction':1,'terminator':2}[r['kind']],r.get('node_id',r.get('region_id',-1))
    entries.sort(key=key);need(len(entries)<=MAP_ENTRIES_MAX,'source map entry budget');return entries


def go_map(vir,project):
    return source_map(vir,{'kind':'go_practical','capture_context_id':project.capture['id'],'entries':go_entries(vir,project)})


def go_manifest(project,provenance):
    provenance_shape(provenance);nodes=project_nodes(project);context=next(iter(project.templates.values())).context
    selection={'schema':GO['selection'][0],'value':{'roots':project.spec['roots']}};go_selection(selection)
    packages=[nodes[i] for i in project.capture['packages']];need(len(packages)<=CAPS['manifest.units'],'captured package manifest bound')
    return {'kind':'go_practical','selection':selection,'limit_profile':{'id':GO['limits'][0],'content_sha256':context['semantic_parameters']['value']['limits_sha256']},
        **copy.deepcopy(provenance),'capture_context':project.capture,'input_inventory':input_inventory(project),
        'build_config':nodes[project.capture['build_config_id']],'modules':[nodes[i] for i in project.capture['modules']],'packages':packages}


class Artifacts:
    def __init__(self,module_policy,artifact_policy,authority,project_spec=None):
        exact(authority,{'release_registry','provenance'} if project_spec is not None else {'captured_inputs','synthetic_permissions','expected_release_registry','release_registry'})
        if project_spec is not None:provenance_shape(authority['provenance'])
        else:
            need(type(authority['captured_inputs']) is list and type(authority['synthetic_permissions']) is list,'independent captured input/permission arrays')
            for row in authority['captured_inputs']:
                exact(row,{'kind','normalized_path','raw_hex'});need(row['kind'] in ('source','contract','build_manifest','lockfile'),'captured input kind');path(row['normalized_path'])
                need(type(row['raw_hex']) is str and re.fullmatch('(?:[0-9a-f]{2})*',row['raw_hex']) is not None,'original bytes in lowercase hex')
        self.modules=ModuleValidator(module_policy,project_spec);self.policy=artifact_policy
        self.authority_bytes=canonical(authority);self.project_bytes=None if project_spec is None else canonical(project_spec)
        self.checked_vir=None;self.expected_go_map=None;self.expected_go_manifest=None

    def linked_vir(self,vir):
        raw=canonical(vir)
        if self.checked_vir!=raw:
            self.modules.validate(vir);self.checked_vir=raw;self.expected_go_map=None;self.expected_go_manifest=None
        return vir['semantic_context'],vir['program']['kind']

    def validate_map(self,vir,value):
        exact(value,MAP_FIELDS);need(value['schema']==SHARED['source_map'][0],'one shared source map generation')
        context,kind=self.linked_vir(vir)
        need(canonical(value['semantic_context'])==canonical(context) and value['profile_contract']==profile_contract(context,'source_map'),'full source-map context/profile contract')
        need(value['source_ir_schema']==SHARED['vir'][0] and value['source_ir_hash']==vir['vir_hash'],'exact common VIR binding')
        need(len(canonical(value))<=MAP_CANONICAL_MAX and value['source_map_hash']==digest(SHARED['source_map'][1],{k:v for k,v in value.items() if k!='source_map_hash'}),'bounded complete source map hash')
        payload=value['map'];need(type(payload) is dict and payload.get('kind')==kind,'context-dispatched source map arm')
        exact(payload,{'kind','capture_context_id','entries'} if kind=='go_practical' else {'kind','entries'})
        need(type(payload['entries']) is list and len(payload['entries'])<=MAP_ENTRIES_MAX,'bounded complete source map entry array')
        if kind=='go_practical':
            if self.expected_go_map is None:self.expected_go_map=canonical(go_map(vir,Project(parse(self.project_bytes))))
            need(canonical(value)==self.expected_go_map,'exact complete Go source reference/origin/capture set')
        else:need(self.policy(self.query(vir,value))['accepted'] is True,'actual retained source map importer')
        return {'kind':'map_linked_unproved','entries':len(payload['entries'])}

    def query(self,vir,map_,manifest=None):
        authority=parse(self.authority_bytes);query=legacy_query(vir['semantic_context'],vir['program']['units'])
        query.update(stage='map' if manifest is None else 'manifest',entries=map_['map']['entries'],captured_inputs=authority['captured_inputs'],synthetic_permissions=authority['synthetic_permissions'])
        if manifest is not None:
            payload=manifest['manifest'];exact(payload,RETAINED_MAN_FIELDS)
            old={k:v for k,v in payload.items() if k!='kind'}
            old['release_registry']=authority['expected_release_registry']
            query.update(manifest=old,expected_release_registry=authority['expected_release_registry'])
        return query

    def validate_manifest(self,vir,map_,value):
        exact(value,MAN_FIELDS);need(value['schema']==SHARED['frontend_manifest'][0],'frontend stage only; no certificate/old/future root')
        context,kind=self.linked_vir(vir)
        need(canonical(value['semantic_context'])==canonical(context) and value['profile_contract']==profile_contract(context,'manifest'),'full manifest context/profile contract')
        need(value['vir_hash']==vir['vir_hash'] and value['source_map_hash']==map_['source_map_hash'],'exact VIR/source-map references')
        need(len(canonical(value))<=MAN_CANONICAL_MAX and value['source_manifest_hash']==digest(SHARED['frontend_manifest'][1],{k:v for k,v in value.items() if k!='source_manifest_hash'}),'bounded complete frontend manifest hash')
        payload=value['manifest'];need(type(payload) is dict and payload.get('kind')==kind,'context-dispatched frontend manifest arm')
        exact(payload,GO_MAN_FIELDS if kind=='go_practical' else RETAINED_MAN_FIELDS)
        authority=parse(self.authority_bytes)
        need(canonical(payload['release_registry'])==canonical(authority['release_registry']),'exact independent successor release-registry reference before projection')
        self.validate_map(vir,map_)
        if kind=='go_practical':
            if self.expected_go_manifest is None:
                authority=parse(self.authority_bytes);project=Project(parse(self.project_bytes))
                self.expected_go_manifest=canonical(go_manifest(project,authority['provenance']))
            need(canonical(payload)==self.expected_go_manifest,'complete Go capture/inventory/build/module/package/selection/provenance linkage')
        else:need(self.policy(self.query(vir,map_,value))['accepted'] is True,'actual retained frontend manifest importer')
        return {'kind':'frontend_linked_unproved'}
