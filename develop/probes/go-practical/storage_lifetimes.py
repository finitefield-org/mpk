"""Private complete-root lifetime probe layered on the retained view model.

Source scope/evaluation bindings and this algorithm still require independent
validation and ordinary proofs. No user-supplied live set is accepted here.
"""
import copy
from dataclasses import dataclass, field, replace
from storage_views import World, Layout, Owner, Location, Descriptor, ZeroAccess, INT, array, structure, slice_layout, SourcePanic, ShapeFailure, ProfileDomain, PermissionFailure, integer


@dataclass(frozen=True)
class Atom(Layout):
    width: int = 64
    signed: bool = True
    bound: int = 256


BOOL = Atom("bool", "bool")
BYTES = Atom("string", "bytes")
UINT64 = Atom("uint64", "word", signed=False)


@dataclass(frozen=True)
class ErrorValue:
    case: object = None
    payload: object = None


def error_layout(cases):
    return Layout("error", "error", fields=tuple(cases))


@dataclass(frozen=True)
class Value:
    layout: Layout
    data: object


@dataclass
class Packet:
    scope: str
    kind: str
    items: tuple = ()
    call_scope: object = None
    signature: tuple = ()
    ready: bool = True
    assigned: bool = True
    evidence: object = None


@dataclass
class Scope:
    parent: object
    variables: dict = field(default_factory=dict)
    entry: tuple = ()
    snapshot: object = None
    result_token: object = None


@dataclass(frozen=True)
class Snapshot:
    store: object
    inputs: tuple
    outputs: tuple = ()


@dataclass(frozen=True)
class CallEvidence:
    invocation: str
    entry: Snapshot
    post: Snapshot
    descriptor_translation: tuple


class LiveWorld(World):
    def __init__(self, storage_bound=128, slots_per_element_type=2):
        super().__init__(storage_bound, slots_per_element_type)
        self.scopes = {}
        self.packets = {}
        self.targets = {}
        self.slot_owners = {}
        self.owner_slots = {}
        self.slot_events = []  # observer history, not part of the semantic carrier
        self._suspend_collection = False
        self._readonly = False
        self._error_universe = None

    def mutable(self):
        if self._readonly: raise PermissionFailure("entry snapshot is immutable")

    def is_plain(self, t):
        if t.kind in {"scalar", "word", "bool", "bytes"}: return True
        if t.kind == "array": return self.is_plain(t.element)
        if t.kind in {"struct", "error"}: return all(self.is_plain(c) for _, c in t.fields)
        return False

    def register(self, t, trail=()):
        if not isinstance(t, Layout) or t.key in trail: raise ShapeFailure("acyclic captured layout")
        if t.key in self.layouts and self.layouts[t.key] != t: raise ShapeFailure("one exact layout per type key")
        if t.kind == "array":
            integer(t.length); self.register(t.element, trail+(t.key,))
        elif t.kind in {"struct", "error"}:
            if len({k for k, _ in t.fields}) != len(t.fields): raise ShapeFailure("unique direct fields/cases")
            if t.kind == "error":
                if self._error_universe is not None and self._error_universe != t.fields: raise ShapeFailure("one closed error universe per unit")
                self._error_universe = t.fields
            for _, child in t.fields: self.register(child, trail+(t.key,))
        elif t.kind == "slice":
            self.register(t.element, trail+(t.key,))
            if not self.is_plain(t.element): raise ShapeFailure("plain backing cells including complete error universe")
        elif t.kind == "word":
            if not isinstance(t, Atom) or t.width not in {8,16,32,64} or type(t.signed) is not bool: raise ShapeFailure("captured word representation")
        elif t.kind not in {"scalar", "bool", "bytes"}: raise ShapeFailure("closed v1 root layout")
        self.layouts[t.key] = t

    def zero_value(self, t):
        if t.kind in {"scalar", "word"}: return 0
        if t.kind == "bool": return False
        if t.kind == "bytes": return b""
        if t.kind == "error": return ErrorValue()
        if t.kind == "slice": return Descriptor(t.element.key, True, None, 0, 0, 0)
        if t.kind == "array": return [self.zero_value(t.element) for _ in range(t.length)]
        if t.kind == "struct": return {k:self.zero_value(c) for k,c in t.fields}
        raise ShapeFailure("closed zero value")

    def check_value(self, t, v):
        if t.kind == "word":
            lo, hi = (-(1<<(t.width-1)), (1<<(t.width-1))-1) if t.signed else (0,(1<<t.width)-1)
            if type(v) is not int or not lo <= v <= hi: raise ShapeFailure("full captured word value")
        elif t.kind == "bool":
            if type(v) is not bool: raise ShapeFailure("source Boolean value")
        elif t.kind == "bytes":
            if type(v) is not bytes or len(v) > t.bound: raise ShapeFailure("bounded raw bytes")
        elif t.kind == "error":
            if not isinstance(v, ErrorValue): raise ShapeFailure("closed error value")
            if v.case is None:
                if v.payload is not None: raise ShapeFailure("nil interface has no payload")
            elif v.case not in dict(t.fields): raise ShapeFailure("tag in complete error universe")
            else: self.check_value(dict(t.fields)[v.case], v.payload)
        else: super().check_value(t, v)

    def validate(self, d):
        if not isinstance(d, Descriptor) or type(d.nil) is not bool or d.element_key not in self.layouts: raise ShapeFailure("known typed descriptor")
        if not self.is_plain(self.layouts[d.element_key]): raise ShapeFailure("plain descriptor cell")
        for n in (d.offset,d.length,d.capacity): integer(n)
        if d.length > d.capacity: raise ShapeFailure("descriptor length <= capacity")
        if d.nil:
            if d.array is not None or (d.offset,d.length,d.capacity) != (0,0,0): raise ShapeFailure("nil has no array")
        elif d.capacity == 0:
            if d.array != ZeroAccess(d.element_key) or d.offset != 0: raise ShapeFailure("typed zero-access witness")
        else:
            t,_ = self.select(d.array)
            if t.kind != "array" or t.element.key != d.element_key or d.offset+d.capacity > t.length: raise ShapeFailure("typed array path/extent")
        if d.capacity > self.storage_bound: raise ProfileDomain("slice capacity bound")

    def view(self, loc, lo=0, hi=None, maximum=None):
        t,_ = self.select(loc)
        if t.kind != "array" or not self.is_plain(t.element): raise ShapeFailure("addressable plain-cell array")
        lo=integer(lo); hi=t.length if hi is None else integer(hi); maximum=t.length if maximum is None else integer(maximum)
        if not lo <= hi <= maximum <= t.length: raise SourcePanic("array slice bounds")
        cap=maximum-lo
        if cap > self.storage_bound: raise ProfileDomain("slice capacity bound")
        return Descriptor(t.element.key,False,loc if cap else ZeroAccess(t.element.key),lo if cap else 0,hi-lo,cap)

    def create(self, *args, **kwargs):
        raise ShapeFailure("use declared roots or allocated evaluation tokens")

    def export_views(self, *args, **kwargs):
        raise ShapeFailure("complete source-root exit transition required")

    def copy_local(self, *args, **kwargs):
        raise ShapeFailure("copy into an exact declared source binding")

    def _create(self, layout, value=None, *, kind="local", access="fresh", scope="current"):
        self.mutable(); self.register(layout)
        if kind not in {"local","heap"} or access not in {"fresh","borrowed"}: raise ShapeFailure("closed provenance/access")
        if kind == "local" and (scope not in self.scopes or scope in self.ended_scopes): raise ShapeFailure("active bound local scope")
        slot=None
        if kind == "heap":
            if layout.kind != "array" or not self.is_plain(layout.element): raise ShapeFailure("plain heap root")
            if layout.length > self.storage_bound: raise ProfileDomain("heap storage bound")
            if not self._suspend_collection: self.collect()
            if layout.length:
                slot=next((i for i in range(self.slots) if (layout.element.key,i) not in self.slot_owners),None)
                if slot is None: raise ProfileDomain("live canonical owner slot bound")
        value=self.zero_value(layout) if value is None else value; self.check_value(layout,value)
        identity="owner:"+str(self.serial); self.serial+=1
        self.owners[identity]=Owner(layout,copy.deepcopy(value),kind,access,scope)
        if slot is not None:
            key=(layout.element.key,slot); self.slot_owners[key]=identity; self.owner_slots[identity]=key
            self.slot_events.append((identity,key))
        return Location(identity)

    def write(self, loc, value):
        self.mutable(); super().write(loc,value)

    def store_snapshot(self):
        s=LiveWorld(self.storage_bound,self.slots)
        for attr in ("owners","layouts","ended_scopes","slot_owners","owner_slots","_error_universe"):
            setattr(s,attr,copy.deepcopy(getattr(self,attr)))
        s.serial=self.serial; s._readonly=True
        return s

    def checked_items(self, items):
        if not isinstance(items,(tuple,list)): raise ShapeFailure("ordered typed product")
        if len(items)>16: raise ShapeFailure("joint result arity")
        result=[]
        for x in items:
            if not isinstance(x,Value): raise ShapeFailure("typed value packet")
            self.register(x.layout); self.check_value(x.layout,x.data); result.append(copy.deepcopy(x))
        return tuple(result)

    def begin_scope(self, name, parent=None, *, entry=None, result_token=None, result_types=()):
        self.mutable()
        if type(name) is not str or not name or name in self.scopes or name in self.ended_scopes: raise ShapeFailure("new exact activation scope")
        if parent is not None and parent not in self.scopes: raise ShapeFailure("active source parent scope")
        values=() if entry is None else self.checked_items(entry)
        snapshot=None if entry is None else Snapshot(self.store_snapshot(),copy.deepcopy(values))
        if result_token is not None:
            if result_token in self.packets or result_token in self.targets or type(result_token) is not str: raise ShapeFailure("unique joint invocation token")
            if not isinstance(result_types,(tuple,list)) or len(result_types)>16: raise ShapeFailure("fixed joint result arity")
            if entry is None: raise ShapeFailure("function entry packet required")
            for t in result_types: self.register(t)
            self.packets[result_token]=Packet(parent or "$outside","joint",call_scope=name,signature=tuple(result_types),ready=False,assigned=False)
        self.scopes[name]=Scope(parent,entry=values,snapshot=snapshot,result_token=result_token)

    def declare(self, scope, binding, layout, value=None):
        self.mutable()
        if scope not in self.scopes or binding in self.scopes[scope].variables: raise ShapeFailure("exact live source binding")
        loc=self._create(layout,value,scope=scope); self.scopes[scope].variables[binding]=loc
        return loc

    def variable(self, scope, binding):
        if scope not in self.scopes or binding not in self.scopes[scope].variables: raise ShapeFailure("live lexical binding")
        return self.scopes[scope].variables[binding]

    def hold_values(self, token, scope, items):
        self.mutable()
        if scope not in self.scopes or token in self.packets or token in self.targets: raise ShapeFailure("unique source evaluation token")
        self.packets[token]=Packet(scope,"temporary",self.checked_items(items))

    def hold_target(self, token, scope, loc):
        self.mutable(); self.select(loc)
        if scope not in self.scopes or token in self.packets or token in self.targets: raise ShapeFailure("unique captured location token")
        self.targets[token]=(scope,loc)

    def release(self, token):
        self.mutable()
        if token in self.packets:
            if self.packets[token].kind=="joint" and not self.packets[token].ready: raise ShapeFailure("call result not returned")
            del self.packets[token]
        elif token in self.targets: del self.targets[token]
        else: raise ShapeFailure("known evaluation token")

    def allocate(self, token, scope, element, length, *, access="fresh"):
        self.mutable()
        if scope not in self.scopes or token in self.packets or token in self.targets: raise ShapeFailure("unique allocating expression token")
        integer(length); loc=self._create(array(element,length),kind="heap",access=access)
        d=self.view(loc); self.hold_values(token,scope,[Value(slice_layout(element),d)])
        return d

    def put_result(self, scope, items):
        self.mutable(); s=self.scopes[scope]
        if s.result_token is None: raise ShapeFailure("function return token")
        p=self.packets[s.result_token]
        if p.ready or p.assigned: raise ShapeFailure("one result for one invocation")
        values=self.checked_items(items)
        if tuple(v.layout for v in values)!=p.signature: raise ShapeFailure("exact positional result signature")
        p.items=values; p.assigned=True
        # Readiness is published only after complete escape translation.

    def project_result(self, token, index):
        p=self.packets.get(token)
        if p is None or p.kind!="joint" or not p.ready: raise ShapeFailure("same invocation completed result")
        if type(index) is not int or not 0<=index<len(p.items): raise ShapeFailure("result projection bounds")
        return copy.deepcopy(p.items[index])

    def materialize_result(self, token, index, evaluation_token):
        """Protect a projected operand before consuming its joint packet."""
        value=self.project_result(token,index);p=self.packets[token]
        self.hold_values(evaluation_token,p.scope,[value])
        return value

    def descriptors(self, t, value):
        self.check_value(t,value)
        if t.kind=="slice": return [value]
        if t.kind=="array": return [d for v in value for d in self.descriptors(t.element,v)]
        if t.kind=="struct": return [d for k,c in t.fields for d in self.descriptors(c,value[k])]
        if t.kind=="error" and value.case is not None: return self.descriptors(dict(t.fields)[value.case],value.payload)
        return []

    def root_rows(self, skip_scope=None):
        rows=[]
        for sid,s in self.scopes.items():
            if sid==skip_scope: continue
            for binding,loc in s.variables.items(): rows.append(("binding",(sid,binding),loc))
            for i,v in enumerate(s.entry): rows.append(("entry",(sid,i),v))
        for token,p in self.packets.items():
            if p.scope!=skip_scope:
                for i,v in enumerate(p.items): rows.append(("packet",(token,i),v))
        for token,(scope,loc) in self.targets.items():
            if scope!=skip_scope: rows.append(("target",token,loc))
        return rows

    def reachability(self, skip_scope=None):
        reachable=set(); occurrences=[]; seen=set()
        def visit_location(loc, role):
            t,v=self.select(loc); reachable.add(loc.owner)
            if loc in seen: return
            seen.add(loc)
            visit_value(t,v,role)
        def visit_value(t,v,role):
            for d in self.descriptors(t,v):
                if not d.nil and d.capacity:
                    occurrences.append((role,d)); visit_location(d.array,role)
        for kind,key,item in self.root_rows(skip_scope):
            role=(kind,key)
            if isinstance(item,Location): visit_location(item,role)
            else: visit_value(item.layout,item.data,role)
        return reachable,occurrences

    def collect(self):
        return self._collect()

    def _collect(self, skip_scope=None):
        self.mutable(); reachable,_=self.reachability(skip_scope); retired=[]
        for identity in list(self.owners):
            if identity not in reachable:
                retired.append(identity); del self.owners[identity]
                if identity in self.owner_slots:
                    slot=self.owner_slots.pop(identity); del self.slot_owners[slot]
        return tuple(retired)

    def live_mask(self, element_key):
        reachable,_=self.reachability()
        return tuple(self.slot_owners.get((element_key,i)) in reachable for i in range(self.slots))

    def rewrite_value(self, t, v, mapping):
        if t.kind=="slice": return mapping.get(v,v)
        if t.kind=="array": return [self.rewrite_value(t.element,x,mapping) for x in v]
        if t.kind=="struct": return {k:self.rewrite_value(c,v[k],mapping) for k,c in t.fields}
        if t.kind=="error" and v.case is not None: return ErrorValue(v.case,self.rewrite_value(dict(t.fields)[v.case],v.payload,mapping))
        return copy.deepcopy(v)

    def end_scope(self, scope):
        """Atomic candidate transition; complete escaping roots are derived."""
        self.mutable()
        if scope not in self.scopes or any(s.parent==scope for s in self.scopes.values()): raise ShapeFailure("exact innermost exiting scope")
        if any(p.scope==scope for p in self.packets.values()) or any(s==scope for s,_ in self.targets.values()): raise ShapeFailure("unconsumed evaluation at scope exit")
        tx=copy.deepcopy(self)
        closing=tx.scopes[scope];token=closing.result_token
        evidence=None
        if token is not None:
            p=tx.packets[token]
            if not p.assigned or tuple(v.layout for v in p.items)!=p.signature: raise ShapeFailure("missing ordered return product")
            # Contract/frame observations are at the return endpoint, before
            # dead-root retirement and canonical escape translation.
            evidence=CallEvidence(scope,closing.snapshot,Snapshot(tx.store_snapshot(),copy.deepcopy(closing.entry),copy.deepcopy(p.items)),())
        rows=tx.root_rows(skip_scope=scope); desc=[]
        for kind,key,item in rows:
            if isinstance(item,Location):
                if tx.owners[item.owner].kind=="local" and tx.owners[item.owner].scope==scope: raise ShapeFailure("captured location crosses its source lifetime")
                t,v=tx.select(item)
            else: t,v=item.layout,item.data
            desc.extend(tx.descriptors(t,v))
        tx._collect(skip_scope=scope)
        tx._suspend_collection=True
        # The retained canonical grouping/packing operates on the derived
        # complete descriptor list. Plainness here includes bool/bytes/errors.
        output=tx._promote(desc,scope)
        tx._suspend_collection=False
        mapping=dict(zip(desc,output))
        for sid,s in tx.scopes.items():
            if sid==scope: continue
            for loc in s.variables.values():
                t,v=tx.select(loc); tx.write(loc,tx.rewrite_value(t,v,mapping))
            s.entry=tuple(Value(v.layout,tx.rewrite_value(v.layout,v.data,mapping)) for v in s.entry)
        for p in tx.packets.values(): p.items=tuple(Value(v.layout,tx.rewrite_value(v.layout,v.data,mapping)) for v in p.items)
        del tx.scopes[scope]
        if token is not None:
            p=tx.packets[token]
            if not p.assigned or tuple(v.layout for v in p.items)!=p.signature: raise ShapeFailure("missing ordered return product")
            p.ready=True;p.evidence=replace(evidence,descriptor_translation=tuple(mapping.items()))
        tx.collect(); self.__dict__=tx.__dict__

    def _promote(self, descriptors, scope):
        # This copies the retained private algorithm, without changing its
        # pinned source or assigning its proof to this extended state machine.
        refs={d.array for d in descriptors if not d.nil and d.capacity and self.owners[d.array.owner].kind=="local" and self.owners[d.array.owner].scope==scope}
        if any(self.owners[r.owner].access!="fresh" for r in refs): raise PermissionFailure("no promotion of borrowed storage")
        windows={r:{i for d in descriptors if d.array==r for i in range(d.offset,d.offset+d.capacity)} for r in refs}
        def contains(a,b):
            if a.owner!=b.owner or len(a.path)>=len(b.path) or b.path[:len(a.path)]!=a.path: return False
            kind,index=b.path[len(a.path)]; return kind=="index" and index in windows[a]
        roots=sorted((r for r in refs if not any(contains(a,r) for a in refs)),key=lambda r:(r.owner,repr(r.path)))
        indices={r:sorted(windows[r]) for r in roots}; counts={}
        for r in roots:
            t,_=self.select(r)
            if len(indices[r])>self.storage_bound: raise ProfileDomain("accessible export storage bound")
            counts[t.element.key]=counts.get(t.element.key,0)+1
        for key,n in counts.items():
            if sum(k==key for k,_ in self.slot_owners)+n>self.slots: raise ProfileDomain("live promotion slot bound")
        maps={r:{old:new for new,old in enumerate(indices[r])} for r in roots}; promoted={}
        for r in roots:
            t,values=self.select(r); promoted[r]=self._create(array(t.element,len(indices[r])),[values[i] for i in indices[r]],kind="heap")
        result=[]
        for d in descriptors:
            if d.array in refs:
                r=next(r for r in roots if r==d.array or contains(r,d.array))
                if r==d.array:
                    offset=maps[r][d.offset]
                    if any(maps[r][d.offset+i]!=offset+i for i in range(d.capacity)): raise ShapeFailure("window isometry")
                    d=replace(d,array=promoted[r],offset=offset)
                else:
                    _,index=d.array.path[len(r.path)]; path=(("index",maps[r][index]),)+d.array.path[len(r.path)+1:]
                    d=replace(d,array=Location(promoted[r].owner,path))
            result.append(d)
        self.ended_scopes.add(scope)
        return result
