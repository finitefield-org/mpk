#!/usr/bin/env python3
"""Check current arithmetic evidence,predecessors and the still-open full plan."""
import re
from urllib.parse import unquote
from run_ordinary_arithmetic import ROOT,RECORD,check_retained
from run_numeric_bridge import check_retained as previous
from check_scope_report import require,strict_json,sha


def main():
    w=RECORD.parent;r=check_retained();old=previous();require(len(old['inputs'])==697 and old['source_values']==13,'preceding word numeric evidence retained')
    work=strict_json((w/'freeze-worklist.json').read_bytes());d=work['ordinary_arithmetic_draft']
    require((d['logical_heads'],d['ordinary_cases'],d['native_observations'],d['actual_AST_observations'],d['model_resource_pending'],d['source_inputs'],d['targeted_tests'])==(27,10,149,139,10,len(r['inputs']),16),'arithmetic worklist/evidence closure')
    require(work['status']=='in progress; not frozen or published' and work['open_decisions'] and not d['source_evaluation_proved'] and not d['complete_go_elaboration'],'no full source/profile promotion')
    inv=strict_json((w/'surface-inventory.json').read_bytes());pins=inv['discovered_files'];require(len(pins)==len({x['path'] for x in pins})==143,'all predecessor paths')
    for x in pins+inv['baseline']+[x for o in inv['component_owners'] for x in o['existing_anchors']]:require(sha((ROOT/x['path']).read_bytes())==x['sha256'],'unchanged predecessor/baseline')
    files=sorted(p for p in w.rglob('*.json') if 'verification' not in p.relative_to(w).parts)
    for p in files:strict_json(p.read_bytes())
    names=('word-numeric-bridge','word-foundation','proof-artifacts','capture-artifacts','shared-vir','vir-body','operation-signatures','contract-wire','closed-carriers','foundation-limits','context-protocol','identity-dag','structural-limits','source-admission','storage-lifetimes','storage-views','overlay-contracts','variant-termination','nat-termination')
    docs=[w/'README.md',RECORD/'README.md']+[w/n/'README.md' for n in names]+[w/'foundation-limits/semantic-rules.md',w/'type-error-zero-decisions.md',w/'operation-loop-decisions.md',ROOT/'develop/docs/09_go_practical_subset_design-todo.md',ROOT/'develop/docs/go-practical-implementation-traceability-ledger.md'];count=0
    for p in docs:
        for t in re.findall(r'\[[^\]\n]*\]\(([^)\n]+)\)',p.read_text()):
            t=t.strip('<>')
            if re.match(r'^[a-z]+://',t) or t.startswith('#'):continue
            require((p.parent/unquote(t.split('#',1)[0])).is_file(),'document link: '+t);count+=1
    require(list(map(int,re.findall(r'^## (\d+)\.',docs[-1].read_text(),re.M)))==list(range(1,52)),'ordered ledger1–51')
    print(f"PASS:{len(files)} strict JSONs,{len(r['inputs'])} source pins,10 paired arithmetic cases/27 logical heads/149 native observations/139 actual AST/10 model pending,143 predecessors,{len(docs)} documents/{count} links; W04 open")


if __name__=='__main__':main()
