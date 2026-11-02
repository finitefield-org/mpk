#!/usr/bin/env python3
"""Preserve prior operation evidence while checking the body graph proposal."""
import re
from urllib.parse import unquote
from run_vir_body import ROOT,RECORD,check_retained
from run_operation_signatures import check_retained as check_operations
from check_scope_report import strict_json,require,sha
from identity_catalog import generate as identities
from closed_carriers import TYPE_SCHEMA,TYPE_DOMAIN,INSTANCE_SCHEMA,INSTANCE_DOMAIN
from contract_scope import SCOPE_SCHEMA,SCOPE_DOMAIN,BINDING_SCHEMA,BINDING_DOMAIN
from operation_signatures import SCHEMA as OP_SCHEMA,DOMAIN as OP_DOMAIN
from vir_body import SCHEMA,DOMAIN


def main():
    w=RECORD.parent;r=check_retained();old=check_operations()
    require(len(r['inputs'])==604 and len(old['inputs'])==587,'complete source/data closure')
    require(r['outcomes']=={'typed_unproved':3,'configuration_rejection':17} and r['hash_vectors']==69,'body outcomes')
    require(old['outcomes']=={'typed_unproved':56,'configuration_rejection':168} and old['hash_vectors']==56,'retained operation outcomes')
    cat=identities();ids={x['identity'] for x in cat['identities']}|set(cat['reserved_csharp']['identities'])|{TYPE_SCHEMA,INSTANCE_SCHEMA,SCOPE_SCHEMA,BINDING_SCHEMA,OP_SCHEMA}
    domains={x['domain'] for x in cat['identities']}|set(cat['reserved_csharp']['domains'])|{TYPE_DOMAIN,INSTANCE_DOMAIN,SCOPE_DOMAIN,BINDING_DOMAIN,OP_DOMAIN}
    require(SCHEMA not in ids and DOMAIN not in domains,'unique unreserved body identity/domain')
    work=strict_json((w/'freeze-worklist.json').read_bytes());d=work['vir_body_draft']
    require((d['operation_components'],d['function_candidates'],d['graph_vectors'],d['hash_vectors'],d['targeted_tests'])==(56,3,20,69,21),'body coverage/worklist')
    require(d['shared_family_frozen'] is False and d['source_proofs_checked'] is False and d['dynamic_invocation_proofs_checked'] is False and work['status']=='in progress; not frozen or published' and work['open_decisions'],'whole objective remains open')
    inventory=strict_json((w/'surface-inventory.json').read_bytes());pins=inventory['discovered_files'];require(len(pins)==len({x['path'] for x in pins})==143,'predecessor coverage')
    for p in pins+inventory['baseline']+[x for o in inventory['component_owners'] for x in o['existing_anchors']]:require(sha((ROOT/p['path']).read_bytes())==p['sha256'],'unchanged predecessor/baseline')
    files=sorted(p for p in w.rglob('*.json') if 'verification' not in p.relative_to(w).parts)
    for p in files:strict_json(p.read_bytes())
    docs=[w/'README.md',RECORD/'README.md']+[w/name/'README.md' for name in ('operation-signatures','contract-wire','closed-carriers','foundation-limits','context-protocol','identity-dag','structural-limits','source-admission','storage-lifetimes','storage-views','overlay-contracts','variant-termination','nat-termination')]
    docs += [w/'foundation-limits/semantic-rules.md',w/'type-error-zero-decisions.md',w/'operation-loop-decisions.md',ROOT/'develop/docs/09_go_practical_subset_design-todo.md',ROOT/'develop/docs/go-practical-implementation-traceability-ledger.md']
    links=0
    for p in docs:
        for target in re.findall(r'\[[^\]\n]*\]\(([^)\n]+)\)',p.read_text()):
            target=target.strip('<>')
            if re.match(r'^[a-z]+://',target) or target.startswith('#'):continue
            require((p.parent/unquote(target.split('#',1)[0])).is_file(),'document link: '+target);links+=1
    require(list(map(int,re.findall(r'^## (\d+)\.',docs[-1].read_text(),re.M)))==list(range(1,46)),'ordered ledger1–45')
    print(f'PASS:{len(files)} strict JSONs,56 components/3 body candidates/20 graph cases/69 hashes,retained56 operation hashes/587 pins,143 predecessors,{len(docs)} documents/{links} links; W04 open')


if __name__=='__main__':main()
