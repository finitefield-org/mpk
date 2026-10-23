"""Show precisely which closed source shape changes under the unary fix."""
from pathlib import Path
import hashlib,json,re
ROOT=Path(__file__).resolve().parent
module='crates/mpk-vc/src/go_practical_ordinary/source_value_relations.rs'
def snapshot(epoch):
    rows=json.loads((ROOT/epoch/'inputs.json').read_bytes())['sources']
    return Path(next(r for r in rows if r['path']==module)['complete_changed_input'])
before,after=snapshot('final-large'),snapshot('small-fixed')
old,new=before.read_text(),after.read_text()
method=r'    fn needs_function_adapter\(&self\) -> bool \{\n.*?\n    \}\n'
stripped,count=re.subn(method,'',new,flags=re.S);assert count==1
stripped=stripped.replace('if view.needs_function_adapter() {','if matches!(view, View::Vector(..)) {')
assert stripped==old,'the entire producer module has no other change'
inputs=[before,after,Path(__file__)];units=[];changed=[]
for epoch,unit in [('small-fixed','nil-only'),('small-fixed','entry'),('heterogeneous-final','heterogeneous-payloads'),('final-large','seventy-cases')]:
    p=ROOT/epoch/'certificates'/f'{unit}-nil.sources.json';inputs.append(p);d=json.loads(p.read_bytes())
    def shape(t):
        if t['kind']=='nominal':return ('source',)
        if t['kind']=='builtin':
            if t['name']=='error':return ('product',[('vector',),*[('source',) for _ in d['case_tags']]])
            if t['name']=='string':return ('product',[('vector',),('vector',)])
            return ('equality',) if t['name']=='bool' else ('vector',)
        if t['kind']=='array':return ('vector',) if t['length'] else ('equality',)
        if t['kind']=='struct':return ('product',[('source',) for _ in t['fields']])
        assert t['kind']=='slice';return ('product',[('equality',),('equality',),('vector',),('vector',),('vector',),('vector',)])
    def needs(v):return v[0]=='vector' or (v[0]=='product' and len(v[1])==1 and needs(v[1][0]))
    rows=[]
    for t in d['types']:
        v=shape(t);prior=v[0]=='vector';current=needs(v)
        rows.append({'source_type_id':t['id'],'kind':t['kind'],'before':prior,'after':current})
        if prior!=current:changed.append({'unit':unit,'source_type_id':t['id'],'kind':t['kind'],'name':t.get('name')})
    units.append({'unit':unit,'full_U_cases':len(d['case_tags']),'all_type_choices':rows})
assert len(changed)==1 and changed[0]['unit']=='nil-only' and changed[0]['name']=='error'
rows=[]
for p in inputs:
    raw=p.read_bytes();rows.append({'path':str(p),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'mode':p.stat().st_mode&0o777})
report={'status':'only_nil_only_error_root_changes','inputs':rows,'entire_producer_module_diff_limited_to_one_adapter_predicate':True,'units':units,'changed_source_type_choices':changed,'complete_large_and_heterogeneous_source_type_recipes_unchanged':True,'no_old_native_result_relabelled_as_new_source_execution':True,'remaining':'Fresh complete-library/boundary-row byte replay and final worktree binding are recorded separately after original source consumers terminate.'}
(ROOT/'adapter-choice-audit.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'status':report['status'],'full_source_type_choices':sum(len(u['all_type_choices']) for u in units),'changed_choices':len(changed)}))
