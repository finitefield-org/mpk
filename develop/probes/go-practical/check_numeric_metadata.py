#!/usr/bin/env python3
"""Check current numeric bridge evidence and preserve the continuing full plan."""
import re
from urllib.parse import unquote
from run_numeric_bridge import ROOT,RECORD,check_retained
from run_word_foundation import check_retained as previous
from check_scope_report import require,strict_json,sha


def main():
    w=RECORD.parent;r=check_retained();old=previous();require(len(old['inputs'])==685 and old['source_operations']==787,'preceding word definition evidence retained')
    work=strict_json((w/'freeze-worklist.json').read_bytes());d=work['word_numeric_bridge_draft']
    require((d['widths'],d['source_values'],d['ordinary_cases'],d['native_observations'],d['source_inputs'],d['targeted_tests'])==([8,16,32,64],13,20,288,len(r['inputs']),12),'numeric worklist/evidence closure')
    require(work['status']=='in progress; not frozen or published' and work['open_decisions'] and not d['source_evaluation_proved'] and not d['complete_go_elaboration'],'no full-word/source/profile promotion')
    inv=strict_json((w/'surface-inventory.json').read_bytes());pins=inv['discovered_files'];require(len(pins)==len({x['path'] for x in pins})==143,'all predecessor paths')
    for x in pins+inv['baseline']+[x for o in inv['component_owners'] for x in o['existing_anchors']]:require(sha((ROOT/x['path']).read_bytes())==x['sha256'],'unchanged predecessor/baseline')
    files=sorted(p for p in w.rglob('*.json') if 'verification' not in p.relative_to(w).parts)
    for p in files:strict_json(p.read_bytes())
    names=('word-foundation','proof-artifacts','capture-artifacts','shared-vir','vir-body','operation-signatures','contract-wire','closed-carriers','foundation-limits','context-protocol','identity-dag','structural-limits','source-admission','storage-lifetimes','storage-views','overlay-contracts','variant-termination','nat-termination')
    docs=[w/'README.md',RECORD/'README.md']+[w/n/'README.md' for n in names]+[w/'foundation-limits/semantic-rules.md',w/'type-error-zero-decisions.md',w/'operation-loop-decisions.md',ROOT/'develop/docs/09_go_practical_subset_design-todo.md',ROOT/'develop/docs/go-practical-implementation-traceability-ledger.md'];count=0
    for p in docs:
        for t in re.findall(r'\[[^\]\n]*\]\(([^)\n]+)\)',p.read_text()):
            t=t.strip('<>')
            if re.match(r'^[a-z]+://',t) or t.startswith('#'):continue
            require((p.parent/unquote(t.split('#',1)[0])).is_file(),'document link: '+t);count+=1
    require(list(map(int,re.findall(r'^## (\d+)\.',docs[-1].read_text(),re.M)))==list(range(1,51)),'ordered ledger1–50')
    print(f"PASS:{len(files)} strict JSONs,{len(r['inputs'])} source pins,20 paired numeric cases/13 source Values/288 observations,143 predecessors,{len(docs)} documents/{count} links; W04 open")


if __name__=='__main__':main()
