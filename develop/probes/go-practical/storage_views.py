"""Private canonical-owner/typed-path semantics probe, not a trusted adapter."""
import copy
from dataclasses import dataclass, replace


class SourcePanic(ValueError): pass
class ProfileDomain(ValueError): pass
class PermissionFailure(ValueError): pass
class ShapeFailure(ValueError): pass


@dataclass(frozen=True)
class Layout:
    key: str
    kind: str
    element: object = None
    length: int = 0
    fields: tuple = ()


INT = Layout("int", "scalar")


def array(element, length, key=None):
    return Layout(key or f"[{length}]{element.key}", "array", element, length)


def structure(key, fields):
    return Layout(key, "struct", fields=tuple(fields))


def slice_layout(element):
    return Layout("[]" + element.key, "slice", element)


def plain(layout):
    if layout.kind == "scalar": return True
    if layout.kind == "array": return plain(layout.element)
    if layout.kind == "struct": return all(plain(t) for _, t in layout.fields)
    return False


def zero(layout):
    if layout.kind == "scalar": return 0
    if layout.kind == "array": return [zero(layout.element) for _ in range(layout.length)]
    if layout.kind == "struct": return {n: zero(t) for n, t in layout.fields}
    if layout.kind == "slice": return Descriptor(layout.element.key, True, None, 0, 0, 0)
    raise ShapeFailure("unknown closed layout")


@dataclass(frozen=True)
class Location:
    owner: str
    path: tuple = ()


@dataclass(frozen=True)
class ZeroAccess:
    element_key: str


@dataclass(frozen=True)
class Descriptor:
    element_key: str
    nil: bool
    array: object
    offset: int
    length: int
    capacity: int


@dataclass
class Owner:
    layout: Layout
    value: object
    kind: str
    access: str
    scope: str


def integer(n):
    if type(n) is not int: raise ShapeFailure("mathematical integer operand required")
    if n < 0: raise SourcePanic("nonnegative full source index required")
    return n


class World:
    def __init__(self, storage_bound=128, slots_per_element_type=2):
        self.owners = {}
        self.storage_bound = storage_bound
        self.slots = slots_per_element_type
        self.serial = 0
        self.layouts = {}
        self.ended_scopes = set()

    def register(self, layout, trail=()):
        if not isinstance(layout, Layout) or layout.key in trail: raise ShapeFailure("acyclic typed layout")
        if layout.key in self.layouts and self.layouts[layout.key] != layout: raise ShapeFailure("one exact layout per type key")
        if layout.kind == "array":
            integer(layout.length); self.register(layout.element, trail+(layout.key,))
        elif layout.kind == "struct":
            if len({n for n, _ in layout.fields}) != len(layout.fields): raise ShapeFailure("unique direct fields")
            for _, t in layout.fields: self.register(t, trail+(layout.key,))
        elif layout.kind == "slice":
            self.register(layout.element, trail+(layout.key,))
            if not plain(layout.element): raise ShapeFailure("slice cells are plain")
        elif layout.kind != "scalar": raise ShapeFailure("closed layout kind")
        self.layouts[layout.key] = layout

    def create(self, layout, value=None, *, kind="local", access="fresh", scope="current"):
        self.register(layout)
        if type(scope) is not str or not scope: raise ShapeFailure("bound owner lifetime scope")
        if kind == "local" and scope in self.ended_scopes: raise ShapeFailure("new activation scope required")
        if kind not in {"local", "heap"} or access not in {"fresh", "borrowed"}: raise ShapeFailure("closed owner provenance")
        if kind == "heap":
            if layout.kind != "array" or not plain(layout.element): raise ShapeFailure("heap cells must be plain")
            if layout.length > self.storage_bound: raise ProfileDomain("heap storage element bound")
            count = sum(o.kind == "heap" and o.layout.length > 0 and o.layout.element.key == layout.element.key for o in self.owners.values())
            if layout.length > 0 and count >= self.slots: raise ProfileDomain("heap owner slots, not typed views")
        value = zero(layout) if value is None else value
        self.check_value(layout, value)
        identity = "owner:" + str(self.serial); self.serial += 1
        self.owners[identity] = Owner(layout, copy.deepcopy(value), kind, access, scope)
        return Location(identity)

    def check_value(self, layout, value):
        if layout.kind == "scalar":
            if type(value) is not int or not -(1 << 63) <= value < (1 << 63): raise ShapeFailure("target int64 value")
        elif layout.kind == "array":
            if type(value) is not list or len(value) != layout.length: raise ShapeFailure("exact value-array extent")
            for v in value: self.check_value(layout.element, v)
        elif layout.kind == "struct":
            if type(value) is not dict or set(value) != {n for n, _ in layout.fields}: raise ShapeFailure("exact value fields")
            for n, t in layout.fields: self.check_value(t, value[n])
        elif layout.kind == "slice":
            self.validate(value)
            if value.element_key != layout.element.key: raise ShapeFailure("exact descriptor element type")
        else: raise ShapeFailure("closed value layout")

    def select(self, loc):
        if not isinstance(loc, Location) or loc.owner not in self.owners: raise ShapeFailure("known canonical owner")
        o = self.owners[loc.owner]; t, v = o.layout, o.value
        if o.kind == "local" and o.scope in self.ended_scopes: raise ShapeFailure("local location outlived its scope")
        for kind, key in loc.path:
            if kind == "index" and t.kind == "array":
                integer(key)
                if key >= t.length: raise SourcePanic("array location bounds")
                t, v = t.element, v[key]
            elif kind == "field" and t.kind == "struct":
                fields = dict(t.fields)
                if key not in fields: raise ShapeFailure("direct typed field")
                t, v = fields[key], v[key]
            else: raise ShapeFailure("path cannot cross descriptor; pivot to its owner")
        return t, v

    def read(self, loc):
        return copy.deepcopy(self.select(loc)[1])

    def child(self, loc, kind, key):
        result = Location(loc.owner, loc.path + ((kind, key),)); self.select(result)
        return result

    def write(self, loc, value):
        layout, _ = self.select(loc); self.check_value(layout, value)
        owner = self.owners[loc.owner]
        if owner.access != "fresh": raise PermissionFailure("positive store requires derived fresh ownership")
        def update(old, path):
            if not path: return copy.deepcopy(value)
            _, key = path[0]; result = copy.deepcopy(old)
            result[key] = update(result[key], path[1:])
            return result
        owner.value = update(owner.value, loc.path)

    def view(self, loc, lo=0, hi=None, maximum=None):
        t, _ = self.select(loc)
        if t.kind != "array" or not plain(t.element): raise ShapeFailure("view of addressable plain-cell array")
        lo = integer(lo); hi = t.length if hi is None else integer(hi); maximum = t.length if maximum is None else integer(maximum)
        if not lo <= hi <= maximum <= t.length: raise SourcePanic("array slice bounds")
        cap = maximum-lo
        if cap > self.storage_bound: raise ProfileDomain("slice capacity bound; not source panic")
        return Descriptor(t.element.key, False, loc if cap else ZeroAccess(t.element.key), lo if cap else 0, hi-lo, cap)

    def validate(self, d):
        if not isinstance(d, Descriptor): raise ShapeFailure("typed descriptor")
        for n in (d.offset, d.length, d.capacity): integer(n)
        if d.length > d.capacity: raise ShapeFailure("descriptor length <= capacity")
        if d.nil:
            if d.array is not None or (d.offset, d.length, d.capacity) != (0, 0, 0): raise ShapeFailure("nil has no array")
        elif d.capacity == 0:
            if d.array != ZeroAccess(d.element_key) or d.offset != 0: raise ShapeFailure("canonical no-access typed witness")
        else:
            t, _ = self.select(d.array)
            if t.kind != "array" or not plain(t.element) or t.element.key != d.element_key or d.offset+d.capacity > t.length: raise ShapeFailure("typed array path/extent")
        if d.capacity > self.storage_bound: raise ProfileDomain("slice capacity bound")

    def reslice(self, d, lo, hi, maximum=None):
        self.validate(d); lo, hi = integer(lo), integer(hi)
        maximum = d.capacity if maximum is None else integer(maximum)
        if not lo <= hi <= maximum <= d.capacity: raise SourcePanic("slice bounds")
        cap = maximum-lo
        return Descriptor(d.element_key, d.nil, d.array if cap else None if d.nil else ZeroAccess(d.element_key), d.offset+lo if cap else 0, hi-lo, cap)

    def element_location(self, d, index):
        self.validate(d); index = integer(index)
        if index >= d.length: raise SourcePanic("slice index bounds")
        return self.child(d.array, "index", d.offset+index)

    def values(self, d):
        return [self.read(self.element_location(d, i)) for i in range(d.length)]

    def copy_slice(self, dest, source):
        self.validate(dest); self.validate(source)
        if dest.element_key != source.element_key: raise ShapeFailure("exact element type")
        count = min(dest.length, source.length)
        values = [self.read(self.element_location(source, i)) for i in range(count)]
        for i, value in enumerate(values): self.write(self.element_location(dest, i), value)
        return count

    def append(self, d, values):
        self.validate(d)
        if d.length+len(values) > d.capacity: raise ProfileDomain("no-growth append obligation; Go may grow")
        output = replace(d, length=d.length+len(values))
        for i, value in enumerate(values): self.write(self.element_location(output, d.length+i), value)
        return output

    def copy_local(self, loc, scope="current"):
        layout, value = self.select(loc)
        return self.create(layout, value, kind="local", scope=scope)

    def export_views(self, descriptors, ending_scope="current"):
        """Private scoped promotion of escaping views; no source allocation.

        Group actual array paths by containment, then promote only maximal
        exposed arrays. Preserve all child paths and common aliases together.
        A complete source/lifetime/counter proof remains required.
        """
        if type(ending_scope) is not str or not ending_scope or ending_scope in self.ended_scopes: raise ShapeFailure("one exact ending activation scope")
        for d in descriptors: self.validate(d)
        refs = {d.array for d in descriptors if not d.nil and d.capacity > 0 and self.owners[d.array.owner].kind == "local" and self.owners[d.array.owner].scope == ending_scope}
        if any(self.owners[r.owner].access != "fresh" for r in refs): raise PermissionFailure("cannot promote borrowed local storage into fresh ownership")
        windows = {r: {i for d in descriptors if d.array == r for i in range(d.offset, d.offset+d.capacity)} for r in refs}
        def contains(a, b):
            if a.owner != b.owner or len(a.path) >= len(b.path) or b.path[:len(a.path)] != a.path: return False
            kind, index = b.path[len(a.path)]
            return kind == "index" and index in windows[a]
        roots = [r for r in refs if not any(contains(a, r) for a in refs)]
        roots.sort(key=lambda r: (r.owner, repr(r.path)))
        counts = {}
        indices = {}
        for r in roots:
            layout, _ = self.select(r)
            indices[r] = sorted(windows[r])
            if len(indices[r]) > self.storage_bound: raise ProfileDomain("export accessible storage bound")
            counts[layout.element.key] = counts.get(layout.element.key, 0)+1
        for key, count in counts.items():
            existing = sum(o.kind == "heap" and o.layout.length > 0 and o.layout.element.key == key for o in self.owners.values())
            if existing+count > self.slots: raise ProfileDomain("promotion quota before any state update")
        maps = {r: {old: new for new, old in enumerate(indices[r])} for r in roots}
        promoted = {}
        for r in roots:
            t, values = self.select(r)
            promoted[r] = self.create(array(t.element, len(indices[r])), [values[i] for i in indices[r]], kind="heap")
        output = []
        for d in descriptors:
            if d.array in refs:
                r = next(r for r in roots if r == d.array or contains(r, d.array))
                if r == d.array:
                    # Every descriptor window is contiguous after deleting
                    # inaccessible gaps; no admitted reslice can cross a gap.
                    offset = maps[r][d.offset]
                    if any(maps[r][d.offset+i] != offset+i for i in range(d.capacity)): raise ShapeFailure("window isometry")
                    d = replace(d, array=promoted[r], offset=offset)
                else:
                    _, index = d.array.path[len(r.path)]
                    suffix = (("index", maps[r][index]),) + d.array.path[len(r.path)+1:]
                    d = replace(d, array=Location(promoted[r].owner, suffix))
            output.append(d)
        self.ended_scopes.add(ending_scope)
        return output
