"""Private finite-shape/accounting design; units are not Go bytes or proofs."""
from dataclasses import dataclass

U64_MAX = (1 << 64) - 1
ARRAY_LIMIT, FIELD_LIMIT, DEPTH_LIMIT = 256, 64, 16
VALUE_UNITS, CURRENT_UNITS, ALL_UNITS = 1 << 21, 1 << 23, 1 << 25
STRING_BYTES, DESCRIPTOR_UNITS, LOCATION_UNITS = 256, 5 + 2 * DEPTH_LIMIT, 1 + 2 * DEPTH_LIMIT


class CounterError(ValueError):
    pass


class CounterLimit(CounterError):
    def __init__(self, counter, maximum, observed):
        self.counter, self.maximum, self.observed = counter, maximum, observed
        super().__init__(f"{counter}: {observed} exceeds {maximum}")


def natural(n):
    if type(n) is not int or not 0 <= n <= U64_MAX: raise CounterError("exact unsigned64 counter input")
    return n


def checked_charge(current, amount, maximum, counter):
    """Preflight before retaining the next entry; caller commits only on success."""
    natural(current); natural(amount); natural(maximum)
    if current > maximum: raise CounterError("invalid prior counter state")
    if amount > maximum-current: raise CounterLimit(counter, maximum, maximum+1)
    return current+amount


def sat_add(a, b, maximum=VALUE_UNITS):
    return maximum+1 if a > maximum or b > maximum or a > maximum-b else a+b


def sat_mul(a, b, maximum=VALUE_UNITS):
    if not a or not b: return 0
    return maximum+1 if a > maximum or b > maximum or a > maximum//b else a*b


@dataclass(frozen=True)
class Shape:
    key: str
    kind: str
    element: str = ""
    length: int = 0
    fields: tuple = ()
    width: int = 64
    signed: bool = True


class Shapes:
    def __init__(self, rows, ceiling=VALUE_UNITS):
        rows = tuple(rows)
        self.rows = {r.key: r for r in rows}
        if len(self.rows) != len(rows): raise CounterError("unique exact type/layout key")
        self.cache = {};self.ceiling=natural(ceiling)

    @classmethod
    def from_layouts(cls, layouts, ceiling=VALUE_UNITS):
        rows = []
        for key, t in layouts.items():
            kind = {"scalar":"word", "bytes":"string"}.get(t.kind,t.kind)
            rows.append(Shape(key,kind,t.element.key if t.element else "",t.length,tuple((k,c.key) for k,c in t.fields),getattr(t,"width",64),getattr(t,"signed",True)))
        return cls(rows,ceiling)

    @classmethod
    def from_source_facts(cls, facts, universe):
        aliases = facts.get("aliases", {})
        def resolve(key):
            visited = set()
            while key in aliases:
                if key in visited: raise CounterError("acyclic source alias graph")
                visited.add(key); key = aliases[key]
            return key
        rows = []
        for r in facts["types"]:
            kind = r["kind"]
            fields = tuple((f["key"],resolve(f["type"])) for f in r.get("fields",[]))
            if kind == "error": fields = tuple((resolve(t),resolve(t)) for t in universe)
            rows.append(Shape(r["id"],kind,resolve(r.get("underlying",r.get("element",""))),r.get("length",0),fields,r.get("width",64),r.get("signed",True)))
        return cls(rows)

    def inspect(self, key):
        """Validate complete graph first, including zero arrays/inactive errors.

        Alias edges add no value depth. Aggregate edges add one. Memoized
        heights are absolute shape facts, so reuse cannot hide a deep path.
        """
        if key not in self.rows: raise CounterError("known exact type/layout")
        active = set(); done = {}; stack = [(key,False)]
        while stack:
            current, finish = stack.pop()
            if current in done: continue
            if current not in self.rows: raise CounterError("complete child layout graph")
            r = self.rows[current]
            if not finish:
                if current in active: raise CounterError("acyclic expanded type/error graph")
                if r.kind not in {"named","bool","word","string","array","struct","slice","error"}: raise CounterError("closed Root type kind")
                if r.kind == "word" and (r.width not in {8,16,32,64} or type(r.signed) is not bool): raise CounterError("captured complete word representation")
                natural(r.length)
                if r.kind == "array" and r.length > ARRAY_LIMIT: raise CounterLimit("array_elements",ARRAY_LIMIT,r.length)
                if r.kind == "struct" and len(r.fields) > FIELD_LIMIT: raise CounterLimit("struct_fields",FIELD_LIMIT,len(r.fields))
                if len({name for name,_ in r.fields}) != len(r.fields): raise CounterError("unique direct field/error case keys")
                active.add(current); stack.append((current,True))
                children = [r.element] if r.kind in {"named","array","slice"} else [t for _,t in r.fields] if r.kind in {"struct","error"} else []
                stack.extend((child,False) for child in reversed(children))
                continue
            active.remove(current)
            if r.kind == "named":
                height,plain,weight = done[r.element]
            elif r.kind in {"bool","word"}: height,plain,weight = 0,True,1
            elif r.kind == "string": height,plain,weight = 0,True,STRING_BYTES+1
            elif r.kind == "array":
                h,plain,w = done[r.element];height=h+1;weight=sat_add(1,sat_mul(r.length,w,self.ceiling),self.ceiling)
            elif r.kind == "slice":
                h,plain,_ = done[r.element]
                if not plain: raise CounterError("complete global Plain backing-cell predicate")
                height,plain,weight = h+1,False,DESCRIPTOR_UNITS
            else:
                children = [done[t] for _,t in r.fields]
                if r.kind=="error":
                    for _,child in r.fields:
                        while self.rows[child].kind=="named":child=self.rows[child].element
                        if self.rows[child].kind not in {"bool","word","struct"}:raise CounterError("v1 concrete error payload kind")
                height=1+max((h for h,_,_ in children),default=0)
                plain=all(p for _,p,_ in children);weight=1
                for _,_,w in children: weight=sat_add(weight,w,self.ceiling)
            if height > DEPTH_LIMIT: raise CounterLimit("aggregate_type_nesting",DEPTH_LIMIT,height)
            done[current]=(height,plain,weight)
        self.cache.update(done)
        return done[key]

    def weight(self,key):
        return self.inspect(key)[2]


def measure_world(world):
    """Charge every represented current/past record, not a supplied live set.

    This is accounting instrumentation of the private lifetime model. It does
    not establish source binding, finite-ID normalization or allocator bounds.
    """
    snapshot_stores = {}; snapshot_objects = set(); violations = []; values = [];exact_weights=[]
    def value(shapes,t):
        w=shapes.weight(t.key);values.append(w);exact_weights.append(w<=shapes.ceiling)
        if w > VALUE_UNITS: violations.append("value_units")
        return w
    def store(w):
        shapes=Shapes.from_layouts(w.layouts,ALL_UNITS);owner_units=0
        for owner in w.owners.values():
            if owner.kind=="heap":
                t=owner.layout;shapes.inspect(t.key)
                if t.kind!="array" or not shapes.inspect(t.element.key)[1]:raise CounterError("Plain typed heap storage array")
                # The backing collection is a store, not one source value.
                # Charge each materialized Plain cell, then the array/header.
                cell=value(shapes,t.element) if t.length else 0
                owner_units+=2+t.length*cell
            else:owner_units+=1+value(shapes,owner.layout)
        inline_units=0;role_units=len(w.scopes)+len(w.packets)+LOCATION_UNITS*len(w.targets)
        for scope in w.scopes.values():
            role_units+=LOCATION_UNITS*len(scope.variables)
            inline_units+=sum(value(shapes,v.layout) for v in scope.entry)
            if scope.snapshot is not None: snapshot(scope.snapshot)
        for packet in w.packets.values():
            inline_units+=sum(value(shapes,v.layout) for v in packet.items)
            if packet.evidence is not None:
                snapshot(packet.evidence.entry);snapshot(packet.evidence.post)
        return {"owners":len(w.owners),"owner_units":owner_units,"inline_units":inline_units,"role_units":role_units,"units":owner_units+inline_units+role_units}
    def snapshot(s):
        identity=id(s.store)
        if not getattr(s.store,"_readonly",False): raise CounterError("immutable past store namespace")
        if identity in snapshot_stores and snapshot_stores[identity] is None: raise CounterError("acyclic snapshot namespace graph")
        if id(s) in snapshot_objects:return
        snapshot_objects.add(id(s))
        # Namespace identity is observer bookkeeping only, never a serialized
        # source address. Equal contents in different snapshots still count.
        if identity not in snapshot_stores:
            snapshot_stores[identity]=None;row=store(s.store);snapshot_stores[identity]=row
        else:row=snapshot_stores[identity]
        shapes=Shapes.from_layouts(s.store.layouts,ALL_UNITS)
        observations=1+sum(value(shapes,v.layout) for v in s.inputs+s.outputs)
        row["observation_units"]=row.get("observation_units",0)+observations;row["units"]+=observations
    current=store(world)
    past=list(snapshot_stores.values())
    if any(row is None for row in past): raise CounterError("acyclic snapshot namespace graph")
    if current["units"] > CURRENT_UNITS: violations.append("current_store_units")
    if any(row["units"] > CURRENT_UNITS for row in past): violations.append("past_store_units")
    total=current["units"]+sum(row["units"] for row in past)
    if total > ALL_UNITS: violations.append("all_store_units")
    return {"current":current,"past":past,"past_store_count":len(past),"total_units":total,"largest_value_units":max(values,default=0),"units_exact":all(exact_weights),"violations":sorted(set(violations)),
            "unit_scope":"fixed abstract carrier units; not Go bytes/RSS or a finite canonical-ID/source/allocator proof"}
