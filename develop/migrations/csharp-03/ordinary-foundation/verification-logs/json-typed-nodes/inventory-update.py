from pathlib import Path
import json,re,hashlib,os
p=Path('develop/migrations/csharp-03/artifact-consumer-inventory.json');v=json.loads(p.read_text());policy=v['search_policy'];t=Path('crates/mpk-vc/tests/csharp_practical_inventory.rs').read_text()
xs=t[t.index('const POST_FREEZE_IMPLEMENTATION_PATHS:'):t.index('\n];')];exclude=set(re.findall(r'"([^"]+)"',xs))|{'develop/specs/CSHARP_PRACTICAL_PROFILE_V1.md','develop/specs/CSHARP_PRACTICAL_SHARED_ARTIFACTS_V1.md','develop/specs/vectors/csharp-practical-profile-v1.json'}
paths=[]
def visit(path):
 if path.is_symlink():return
 if path.is_file():
  rel=path.as_posix()
  if rel not in exclude and b'Std.' in path.read_bytes():paths.append(rel)
  return
 if path.name in set(policy['ignored_directory_names'])|{'__pycache__'}:return
 for child in path.iterdir():visit(child)
for root in policy['roots']:visit(Path(root))
paths=sorted(set(paths));new='crates/mpk-vc/src/csharp_practical_ordinary_json_typed_nodes.rs';assert new in paths
old=[x for x in paths if x!=new];hashpaths=lambda xs:hashlib.sha256(('\n'.join(xs)+'\n').encode()).hexdigest()
f=next(f for f in policy['fixtures'] if f['id']=='foundation.std_namespace')
assert len(old)==f['expected_count'] and hashpaths(old)==f['expected_paths_sha256'],(len(old),hashpaths(old),f)
f['expected_count']=len(paths);f['expected_paths_sha256']=hashpaths(paths);p.write_text(json.dumps(v,indent=2)+'\n')
c=Path('develop/migrations/csharp-03/data-phase/historical-inventory-cache-correction.json');value=json.loads(c.read_text());value['inventory_sha256']=hashlib.sha256(p.read_bytes()).hexdigest();c.write_text(json.dumps(value,indent=2)+'\n')
p=Path('crates/mpk-vc/tests/csharp_practical_inventory.rs');assert t.count('4_946,')==1;p.write_text(t.replace('4_946,','4_947,'))
print('Exact previous123-path preimage preserved;added only',new,';namespace paths',len(paths),'total4947')
