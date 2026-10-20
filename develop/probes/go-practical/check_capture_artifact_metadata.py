#!/usr/bin/env python3
"""Validate capture proposals, prior module evidence and unfinished full scope."""
import re
from urllib.parse import unquote
from run_capture_artifacts import ROOT,RECORD,check_retained
from run_shared_vir import check_retained as check_modules
from check_scope_report import strict_json,require,sha
from identity_catalog import generate as identities
from shared_project import SCOPE_SCHEMA,SET_SCHEMA,CONTRACT_SCHEMA,OP_SCHEMA,BODY_SCHEMA,SCOPE_DOMAIN,SET_DOMAIN,CONTRACT_DOMAIN,OP_DOMAIN,BODY_DOMAIN
from capture_artifacts import INVENTORY_SCHEMA,INVENTORY_DOMAIN


def main():
    w=RECORD.parent;r=check_retained();old=check_modules()
    require(len(old['inputs'])==625 and old['hash_vectors']==62 and old['outcomes']=={'retained_native':5,'go_typed_unproved':2,'configuration_rejection':66},'preceding complete module evidence retained')
    require(r['outcomes']=={'map_linked_unproved':8,'frontend_linked_unproved':8,'configuration_rejection':171} and r['hash_vectors']==19 and len(r['preservation'])==5,'complete capture outcomes/hash/baseline coverage')
    cat=identities();ids={x['identity'] for x in cat['identities']}|set(cat['reserved_retired']['identities'])|{SCOPE_SCHEMA,SET_SCHEMA,CONTRACT_SCHEMA,OP_SCHEMA,BODY_SCHEMA}
    domains={x['domain'] for x in cat['identities']}|set(cat['reserved_retired']['domains'])|{SCOPE_DOMAIN,SET_DOMAIN,CONTRACT_DOMAIN,OP_DOMAIN,BODY_DOMAIN}
    require(INVENTORY_SCHEMA not in ids and INVENTORY_DOMAIN not in domains,'new unreserved Go inventory family')
    data=strict_json((RECORD/'artifacts.json').read_bytes());schema=strict_json((RECORD/'schema.json').read_bytes());go=next(x for x in data if 'project_spec' in x)
    require([set(x['fields']) for x in schema['roots']]==[set(go['source_map']),set(go['manifest']),set(go['manifest']['manifest']['input_inventory'])],'exact required root field sets')
    work=strict_json((w/'freeze-worklist.json').read_bytes());d=work['capture_artifact_draft']
    require((d['artifact_triples'],d['vectors'],d['hash_vectors'],d['native_queries'],d['source_inputs'],d['targeted_tests'])==(8,187,19,r['native_queries'],len(r['inputs']),24),'capture worklist/receipt closure')
    require(not d['shared_family_frozen'] and not d['source_proofs_checked'] and not d['registered_provenance_checked'] and work['status']=='in progress; not frozen or published' and work['open_decisions'],'full goal remains open')
    inventory=strict_json((w/'surface-inventory.json').read_bytes());pins=inventory['discovered_files'];require(len(pins)==len({x['path'] for x in pins})==143,'complete predecessor coverage')
    for p in pins+inventory['baseline']+[x for o in inventory['component_owners'] for x in o['existing_anchors']]:require(sha((ROOT/p['path']).read_bytes())==p['sha256'],'unchanged predecessor/baseline')
    source_pins={x['path']:x['sha256'] for x in r['inputs']};require(all(source_pins.get(x['path'])==x['sha256'] for x in r['public_source_audit']['sources']),'every unchanged linked public source file pinned')
    diagnostic=RECORD/'verification/initial-missing-manifest-hash';dr=strict_json((diagnostic/'diagnostic.json').read_bytes())
    for row in dr['raw_inputs_and_outputs']:require(sha((diagnostic/row['path']).read_bytes())==row['sha256'],'retained initial native setup failure')
    files=sorted(p for p in w.rglob('*.json') if 'verification' not in p.relative_to(w).parts)
    for p in files:strict_json(p.read_bytes())
    docs=[w/'README.md',RECORD/'README.md']+[w/n/'README.md' for n in ('shared-vir','vir-body','operation-signatures','contract-wire','closed-carriers','foundation-limits','context-protocol','identity-dag','structural-limits','source-admission','storage-lifetimes','storage-views','overlay-contracts','variant-termination','nat-termination')]
    docs += [w/'foundation-limits/semantic-rules.md',w/'type-error-zero-decisions.md',w/'operation-loop-decisions.md',ROOT/'develop/docs/09_go_practical_subset_design-todo.md',ROOT/'develop/docs/go-practical-implementation-traceability-ledger.md']
    links=0
    for p in docs:
        for target in re.findall(r'\[[^\]\n]*\]\(([^)\n]+)\)',p.read_text()):
            target=target.strip('<>')
            if re.match(r'^[a-z]+://',target) or target.startswith('#'):continue
            require((p.parent/unquote(target.split('#',1)[0])).is_file(),'document link: '+target);links+=1
    require(list(map(int,re.findall(r'^## (\d+)\.',docs[-1].read_text(),re.M)))==list(range(1,48)),'ordered ledger1–47')
    print(f"PASS:{len(files)} strict JSONs,{len(r['inputs'])} source pins,8 artifact triples/187 cases/19 hashes,five retained target contexts,143 predecessors,{len(docs)} documents/{links} links; W04 open")


if __name__=='__main__':main()
