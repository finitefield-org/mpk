#!/usr/bin/env python3
"""Verify shared draft closure, predecessor retention and the continuing worklist."""
import re
from urllib.parse import unquote
from run_shared_vir import ROOT,RECORD,check_retained
from run_vir_body import check_retained as check_bodies
from check_scope_report import strict_json,require,sha
from identity_catalog import generate as identities
from closed_carriers import TYPE_SCHEMA,TYPE_DOMAIN,INSTANCE_SCHEMA,INSTANCE_DOMAIN
from contract_scope import SCOPE_SCHEMA as OLD_SCOPE_SCHEMA,SCOPE_DOMAIN as OLD_SCOPE_DOMAIN,BINDING_SCHEMA,BINDING_DOMAIN
from operation_signatures import SCHEMA as OLD_OP_SCHEMA,DOMAIN as OLD_OP_DOMAIN
from vir_body import SCHEMA as OLD_BODY_SCHEMA,DOMAIN as OLD_BODY_DOMAIN
from shared_project import SCOPE_SCHEMA,SCOPE_DOMAIN,SET_SCHEMA,SET_DOMAIN,CONTRACT_SCHEMA,CONTRACT_DOMAIN,OP_SCHEMA,OP_DOMAIN,BODY_SCHEMA,BODY_DOMAIN


def main():
    w=RECORD.parent;r=check_retained();old=check_bodies()
    require(len(old['inputs'])==604 and old['hash_vectors']==69,'preceding typed-body evidence retained')
    require(r['outcomes']=={'retained_native':5,'go_typed_unproved':2,'configuration_rejection':66} and len(r['preservation'])==5,'whole shared case/preservation coverage')
    cat=identities();ids={x['identity'] for x in cat['identities']}|set(cat['reserved_csharp']['identities'])|{TYPE_SCHEMA,INSTANCE_SCHEMA,OLD_SCOPE_SCHEMA,BINDING_SCHEMA,OLD_OP_SCHEMA,OLD_BODY_SCHEMA}
    domains={x['domain'] for x in cat['identities']}|set(cat['reserved_csharp']['domains'])|{TYPE_DOMAIN,INSTANCE_DOMAIN,OLD_SCOPE_DOMAIN,BINDING_DOMAIN,OLD_OP_DOMAIN,OLD_BODY_DOMAIN}
    new=[(SCOPE_SCHEMA,SCOPE_DOMAIN),(SET_SCHEMA,SET_DOMAIN),(CONTRACT_SCHEMA,CONTRACT_DOMAIN),(OP_SCHEMA,OP_DOMAIN),(BODY_SCHEMA,BODY_DOMAIN)]
    require(len({x[0] for x in new})==len({x[1] for x in new})==5,'unique new Go families')
    require(all(s not in ids and d not in domains for s,d in new),'new identities avoid earlier Go and inactive C# reservations')
    data=strict_json((RECORD/'modules.json').read_bytes());program=next(x['module']['program'] for x in data if 'project_spec' in x)
    samples={SET_SCHEMA:program['source_scopes'],SCOPE_SCHEMA:program['source_scopes']['functions'][0],
             CONTRACT_SCHEMA:program['contracts'][0]['contract']['value'],BODY_SCHEMA:program['function_bodies'][0],
             OP_SCHEMA:next(d for b in program['function_bodies'] for d in b['operation_definitions'])}
    schema=strict_json((RECORD/'schema.json').read_bytes())
    require(all(set(row['fields'])==set(samples[row['schema']]) for row in schema['new_go_identities']),'exact new-root required field sets')
    work=strict_json((w/'freeze-worklist.json').read_bytes());d=work['shared_vir_draft']
    require((d['module_candidates'],d['module_vectors'],d['retained_target_contexts'],d['native_queries'],d['hash_vectors'],d['source_inputs'],d['targeted_tests'])==(7,73,5,r['native_queries'],r['hash_vectors'],len(r['inputs']),22),'shared worklist/receipt closure')
    require(d['shared_family_frozen'] is False and d['source_proofs_checked'] is False and d['ordinary_semantic_proofs_checked'] is False and work['status']=='in progress; not frozen or published' and work['open_decisions'],'remaining objective retained')
    inventory=strict_json((w/'surface-inventory.json').read_bytes());pins=inventory['discovered_files'];require(len(pins)==len({x['path'] for x in pins})==143,'all predecessor paths')
    for p in pins+inventory['baseline']+[x for o in inventory['component_owners'] for x in o['existing_anchors']]:require(sha((ROOT/p['path']).read_bytes())==p['sha256'],'unchanged predecessor/baseline')
    audit=r['retained_source_audit'];seen={p['path']:p['sha256'] for p in r['inputs']}
    require(all(seen.get(p['path'])==p['sha256'] for p in audit['sources']),'every unchanged mirrored semantic source is pinned')
    require(audit['original_prefix_sha256']==sha((ROOT/audit['modified_mirror_file']).read_bytes()) and audit['bridge_sha256']==sha((ROOT/'develop/probes/go-practical/shared-native/bridge.rs').read_bytes()),'only appended observation bridge')
    files=sorted(p for p in w.rglob('*.json') if 'verification' not in p.relative_to(w).parts)
    for p in files:strict_json(p.read_bytes())
    docs=[w/'README.md',RECORD/'README.md']+[w/name/'README.md' for name in ('vir-body','operation-signatures','contract-wire','closed-carriers','foundation-limits','context-protocol','identity-dag','structural-limits','source-admission','storage-lifetimes','storage-views','overlay-contracts','variant-termination','nat-termination')]
    docs += [w/'foundation-limits/semantic-rules.md',w/'type-error-zero-decisions.md',w/'operation-loop-decisions.md',ROOT/'develop/docs/09_go_practical_subset_design-todo.md',ROOT/'develop/docs/go-practical-implementation-traceability-ledger.md']
    links=0
    for p in docs:
        for target in re.findall(r'\[[^\]\n]*\]\(([^)\n]+)\)',p.read_text()):
            target=target.strip('<>')
            if re.match(r'^[a-z]+://',target) or target.startswith('#'):continue
            require((p.parent/unquote(target.split('#',1)[0])).is_file(),'document link: '+target);links+=1
    require(list(map(int,re.findall(r'^## (\d+)\.',docs[-1].read_text(),re.M)))==list(range(1,47)),'ordered ledger1–46')
    print(f"PASS:{len(files)} strict JSONs,{len(r['inputs'])} pinned inputs,7 modules/73 cases/{r['hash_vectors']} hashes,five retained target contexts,143 predecessors,{len(docs)} documents/{links} links; W04 open")


if __name__=='__main__':main()
