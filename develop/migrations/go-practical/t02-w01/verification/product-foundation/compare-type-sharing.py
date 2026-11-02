"""Diagnostic exact goal-type equality under the single checked Product.Type definition.

The original source/certificate bytes are never normalized or replaced. This is
an additional statement-preservation audit, not either production core's verdict.
"""
from pathlib import Path
from functools import lru_cache
import datetime
import hashlib
import json


def seal(path):
    path=Path(path);raw=path.read_bytes()
    return {"path":str(path),"sha256":hashlib.sha256(raw).hexdigest(),"bytes":len(raw)}


root=Path(__file__).resolve().parent.relative_to(Path.cwd().resolve())
reader=root.parent/"word-signed-division/prerequisite-analysis/measure.py"
ns={"__file__":str(reader.resolve())}
exec(compile(reader.read_text().split("sources=[]",1)[0],str(reader),"exec"),ns)
Tables=ns["Tables"]
base_path=root/"explicit-result-types/construction/certificates/product-foundation.mpcert"
base=Tables(seal(base_path))
assert base.record["sha256"]=="06d60305b4616213d61ca3dae35decf76fb150e729e1f063c8c6266cc9ec709e"
base_fingerprints=base.fingerprints();assert len(base_fingerprints)==78


class ExactNodes:
    def __init__(self):
        self.nodes=[];self.index={}
    def node(self,key):
        if key not in self.index:
            self.index[key]=len(self.nodes);self.nodes.append(key)
        return self.index[key]
    @lru_cache(None)
    def lift(self,index,cutoff=0):
        node=self.nodes[index];tag=node[0]
        if tag==1:
            return self.node((1,node[1]+int(node[1]>=cutoff)))
        if tag in {0,2}:return index
        if tag==3:
            return self.node((3,self.lift(node[1],cutoff),tuple(self.lift(a,cutoff) for a in node[2])))
        if tag in {4,5}:
            return self.node((tag,self.lift(node[1],cutoff),self.lift(node[2],cutoff+1)))
        assert tag==6
        return self.node((6,self.lift(node[1],cutoff),self.lift(node[2],cutoff),self.lift(node[3],cutoff+1)))
    def load(self,t):
        levels=[]
        for tag,a in t.levels:
            value=() if tag==0 else (t.names[a[0]],) if tag==3 else tuple(levels[i] for i in a)
            levels.append((tag,value))
        boolean=self.node((2,"Std.Bool",()))
        rec_type=self.node((2,"Std.Bool.recType",()))
        type_constant=self.node((2,"Mpk.Go.Practical.Product.Type",()))
        @lru_cache(None)
        def term(index):
            tag,a=t.terms[index]
            if tag==0:return self.node((0,levels[a[0]]))
            if tag==1:return self.node((1,a[0]))
            if tag==2:return self.node((2,t.declarations[a[0]][0],tuple(levels[i] for i in a[1])))
            if tag==3:
                function=term(a[0]);arguments=tuple(term(i) for i in a[1])
                if function==type_constant:
                    assert len(arguments)==2
                    body=self.node((3,rec_type,(self.lift(arguments[0]),self.lift(arguments[1]),self.node((1,0)))))
                    return self.node((5,boolean,body))
                return self.node((3,function,arguments))
            return self.node((tag,*(term(i) for i in a)))
        # Validate the exact compiled body that this diagnostic alone expands.
        _,kind,args=t.declarations[t.index["Mpk.Go.Practical.Product.Type"]]
        assert kind==1
        zero=levels[t.terms[args[0]][1][0]] if t.terms[args[0]][0]==0 else (0,())
        sort=self.node((0,zero))
        body=self.node((5,boolean,self.node((3,rec_type,(self.node((1,2)),self.node((1,1)),self.node((1,0)))))))
        expected=self.node((4,sort,self.node((4,sort,body))))
        assert term(args[1])==expected
        return {name:term(a[0]) for name,_,a in t.declarations if name.startswith("Mpk.Go.Practical.Test.Product.")}


rows=[]
for arity in range(65):
    old=root/f"final/tests/certificates/balanced-shape-{arity:02}.mpcert"
    new=root/f"alias-consumers/initial/tests/certificates/alias-balanced-shape-{arity:02}.mpcert"
    reports=[p.with_suffix(".rust.json") for p in [old,new]]
    try:
        decoded=[json.loads(p.read_text()) for p in reports]
    except (FileNotFoundError,json.JSONDecodeError):continue
    if not all(r["verdict"]=="accepted" for r in decoded):continue
    records=[seal(p) for p in [old,new]]
    tables=[Tables(r) for r in records]
    for table in tables:
        actual=table.fingerprints()
        for name,original in base_fingerprints.items():assert actual[name]==original,name
    arena=ExactNodes()
    original,shared=[arena.load(t) for t in tables]
    names={f"Mpk.Go.Practical.Test.Product.Pack{arity}",f"Mpk.Go.Practical.Test.Product.Reconstruct{arity}"}
    names.update(f"Mpk.Go.Practical.Test.Product.Frame{arity}Target{i}" for i in range(arity))
    assert set(original)==set(shared)==names
    assert original==shared,arity
    for p,record in zip([old,new],records,strict=True):assert seal(p)==record
    rows.append({"arity":arity,"expanded_certificate":records[0],"shared_certificate":records[1],"expanded_Rust_report":seal(reports[0]),"shared_Rust_report":seal(reports[1]),"every_original_named_goal_type_preserved":len(names),"original_goal_names":sorted(names),"exact_interned_node_equality_not_only_hashes":True,"all_original78_basis_identities_unchanged":True,"expanded_terms":len(tables[0].terms),"shared_terms":len(tables[1].terms)})
    print(json.dumps({"arity":arity,"identical_goal_types":len(names),"terms":[len(t.terms) for t in tables]}),flush=True)
assert rows
out=root/"type-sharing-analysis";out.mkdir(exist_ok=False)
record={"status":"completed_pairs_match_pending_remaining_matrix" if len(rows)<65 else "all65_original_goal_type_sets_match","observed_at_utc":datetime.datetime.now(datetime.timezone.utc).isoformat(),"runner":seal(__file__),"reader":seal(reader),"checked_generic_basis":seal(base_path),"matched_arities":[r["arity"] for r in rows],"pending_arities":[i for i in range(65) if i not in {r["arity"] for r in rows}],"pairs":rows,"only_expanded_name":"Mpk.Go.Practical.Product.Type","expansion":"lambda A:Sort0. lambda B:Sort0. Pi tag:Bool. Bool.recType A B tag; exact compiled body checked, capture-avoiding de Bruijn lifting, exact shared node equality","scope":"Only completed full actual Rust-reported original pairs at this observation epoch. Every full original type/binder/claim is compared after expanding exactly the checked Product.Type body into an independent diagnostic graph, with all78 original basis identities exact. Original source/JSON/certificate bytes remain untouched. No beta/eta/funext/axiom normalization, source rewrite, dropped field/claim or missing/ongoing case is promoted. Both unchanged production cores separately verify complete original byte streams; this comparison supplies no proof/source/context/profile/application authority."}
(out/"result.json").write_text(json.dumps(record,indent=2)+"\n")
print(json.dumps({"status":record["status"],"completed_pairs":len(rows)}))
