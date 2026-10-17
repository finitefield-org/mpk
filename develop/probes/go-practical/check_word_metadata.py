#!/usr/bin/env python3
"""Check complete ordinary word evidence and all unchanged preceding pins."""
import json
import re
from urllib.parse import unquote
from run_word_foundation import ROOT,RECORD,check_retained
from run_proof_artifacts import check_retained as check_previous
from check_scope_report import strict_json,require,sha


def main():
    w=RECORD.parent;r=check_retained();previous=check_previous()
    require(len(previous['inputs'])==673 and previous['go_requests']==169 and previous['go_ordinary_terms_materialized'] is False,'preceding proof-demand evidence retained without promotion')
    work=strict_json((w/'freeze-worklist.json').read_bytes());d=work['word_foundation_draft']
    require((d['source_word_types'],d['source_operations'],d['ordinary_cases'],d['native_observations'],d['source_inputs'],d['targeted_tests'])==(13,787,21,1640,len(r['inputs']),16),'word worklist/source evidence closure')
    require(work['status']=='in progress; not frozen or published' and work['open_decisions'] and d['complete_go_elaboration'] is False and d['source_evaluation_proved'] is False,'complete W04 scope remains open')
    pins=strict_json((w/'surface-inventory.json').read_bytes());paths=pins['discovered_files'];require(len(paths)==len({x['path'] for x in paths})==143,'all predecessor paths retained')
    for x in paths+pins['baseline']+[x for o in pins['component_owners'] for x in o['existing_anchors']]:require(sha((ROOT/x['path']).read_bytes())==x['sha256'],'unchanged predecessor/baseline')
    files=sorted(p for p in w.rglob('*.json') if 'verification' not in p.relative_to(w).parts)
    for p in files:strict_json(p.read_bytes())
    docs=[w/'README.md',RECORD/'README.md']+[w/n/'README.md' for n in ('proof-artifacts','capture-artifacts','shared-vir','vir-body','operation-signatures','contract-wire','closed-carriers','foundation-limits','context-protocol','identity-dag','structural-limits','source-admission','storage-lifetimes','storage-views','overlay-contracts','variant-termination','nat-termination')]
    docs += [w/'foundation-limits/semantic-rules.md',w/'type-error-zero-decisions.md',w/'operation-loop-decisions.md',ROOT/'develop/docs/09_go_practical_subset_design-todo.md',ROOT/'develop/docs/go-practical-implementation-traceability-ledger.md'];count=0
    for p in docs:
        for t in re.findall(r'\[[^\]\n]*\]\(([^)\n]+)\)',p.read_text()):
            t=t.strip('<>')
            if re.match(r'^[a-z]+://',t) or t.startswith('#'):continue
            require((p.parent/unquote(t.split('#',1)[0])).is_file(),'document link: '+t);count+=1
    require(list(map(int,re.findall(r'^## (\d+)\.',docs[-1].read_text(),re.M)))==list(range(1,50)),'ordered ledger1–49')
    print(f"PASS:{len(files)} strict JSONs,{len(r['inputs'])} source pins,21 paired ordinary cases,13 word types/787 bindings/1640 observations,143 predecessors,{len(docs)} documents/{count} links; W04 open")


if __name__=='__main__':main()
