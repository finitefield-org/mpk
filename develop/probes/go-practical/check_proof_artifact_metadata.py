#!/usr/bin/env python3
"""Verify proof-stage data,retained source evidence and the continuing full plan."""
import re
from urllib.parse import unquote
from run_proof_artifacts import ROOT,RECORD,check_retained
from run_capture_artifacts import check_retained as check_capture
from go_obligations import SCHEMA,DOMAIN,REQUEST_SCHEMA,REQUEST_DOMAIN
from identity_catalog import generate as identities
from shared_project import SCOPE_SCHEMA,SET_SCHEMA,CONTRACT_SCHEMA,OP_SCHEMA,BODY_SCHEMA,SCOPE_DOMAIN,SET_DOMAIN,CONTRACT_DOMAIN,OP_DOMAIN,BODY_DOMAIN
from capture_artifacts import INVENTORY_SCHEMA,INVENTORY_DOMAIN
from check_scope_report import strict_json,require,sha


def main():
    w=RECORD.parent;r=check_retained();old=check_capture();require(len(old['inputs'])==652 and old['hash_vectors']==19,'preceding source/capture evidence retained')
    require(r['outcomes']=={'retained_vc_generated':6,'configuration_rejection':157,'certificate_manifest_linked_unproved':8,'retained_skeleton_generated':6,'go_obligations_unelaborated':2,'ordinary_elaboration_pending':2},'all181 proof outcomes')
    require((r['local_fragments'],r['local_demands'],r['go_requests'])==(56,416,169) and len(r['preservation'])==5,'complete Go interface/retained target coverage')
    cat=identities();ids={x['identity'] for x in cat['identities']}|set(cat['reserved_csharp']['identities'])|{SCOPE_SCHEMA,SET_SCHEMA,CONTRACT_SCHEMA,OP_SCHEMA,BODY_SCHEMA,INVENTORY_SCHEMA}
    domains={x['domain'] for x in cat['identities']}|set(cat['reserved_csharp']['domains'])|{SCOPE_DOMAIN,SET_DOMAIN,CONTRACT_DOMAIN,OP_DOMAIN,BODY_DOMAIN,INVENTORY_DOMAIN}
    require(SCHEMA!=REQUEST_SCHEMA and DOMAIN!=REQUEST_DOMAIN and all(s not in ids and d not in domains for s,d in ((SCHEMA,DOMAIN),(REQUEST_SCHEMA,REQUEST_DOMAIN))),'new unreserved Go demand identities')
    work=strict_json((w/'freeze-worklist.json').read_bytes());d=work['proof_artifact_draft']
    require((d['proof_bundles'],d['vectors'],d['local_fragments'],d['local_demands'],d['go_requests'],d['native_queries'],d['hash_vectors'],d['source_inputs'],d['targeted_tests'])==(8,181,56,416,169,r['native_queries'],r['hash_vectors'],len(r['inputs']),25),'proof worklist/source receipt closure')
    require(not d['shared_family_frozen'] and not d['go_ordinary_terms_materialized'] and not d['go_skeleton_emitted'] and work['status']=='in progress; not frozen or published' and work['open_decisions'],'no narrowed goal/freeze/readiness claim')
    inv=strict_json((w/'surface-inventory.json').read_bytes());pins=inv['discovered_files'];require(len(pins)==len({x['path'] for x in pins})==143,'all predecessor paths')
    for p in pins+inv['baseline']+[x for o in inv['component_owners'] for x in o['existing_anchors']]:require(sha((ROOT/p['path']).read_bytes())==p['sha256'],'unchanged predecessor/baseline')
    actual={x['path']:x['sha256'] for x in r['inputs']};require(all(actual.get(p['path'])==p['sha256'] for p in r['public_source_audit']['sources']),'all unchanged public source inputs pinned')
    diagnostic=RECORD/'verification/initial-canonical-limit';dr=strict_json((diagnostic/'diagnostic.json').read_bytes())
    for x in dr['raw_inputs_and_outputs']:require(sha((diagnostic/x['path']).read_bytes())==x['sha256'],'retained initial native canonical limit failure')
    files=sorted(p for p in w.rglob('*.json') if 'verification' not in p.relative_to(w).parts)
    for p in files:strict_json(p.read_bytes())
    docs=[w/'README.md',RECORD/'README.md']+[w/n/'README.md' for n in ('capture-artifacts','shared-vir','vir-body','operation-signatures','contract-wire','closed-carriers','foundation-limits','context-protocol','identity-dag','structural-limits','source-admission','storage-lifetimes','storage-views','overlay-contracts','variant-termination','nat-termination')]
    docs += [w/'foundation-limits/semantic-rules.md',w/'type-error-zero-decisions.md',w/'operation-loop-decisions.md',ROOT/'develop/docs/09_go_practical_subset_design-todo.md',ROOT/'develop/docs/go-practical-implementation-traceability-ledger.md']
    count=0
    for p in docs:
        for t in re.findall(r'\[[^\]\n]*\]\(([^)\n]+)\)',p.read_text()):
            t=t.strip('<>')
            if re.match(r'^[a-z]+://',t) or t.startswith('#'):continue
            require((p.parent/unquote(t.split('#',1)[0])).is_file(),'document link: '+t);count+=1
    require(list(map(int,re.findall(r'^## (\d+)\.',docs[-1].read_text(),re.M)))==list(range(1,49)),'ordered ledger1–48')
    print(f"PASS:{len(files)} strict JSONs,{len(r['inputs'])} source pins,8 bundles/181 cases/{r['hash_vectors']} hashes,56 fragments/416 local demands/169 Go requests,143 predecessors,{len(docs)} documents/{count} links; W04 open")


if __name__=='__main__':main()
