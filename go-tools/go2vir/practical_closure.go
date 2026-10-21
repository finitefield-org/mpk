package main

import (
	"fmt"
	"go/ast"
	"go/token"
	"go/types"
	"sort"
	"strings"
)

// Private compiler observations over intact owned packages. This graph does not
// certify source admission, contracts, closed error/Plain domains or execution.
// It is never a registered context or a source/program proof.
type practicalClosureRef struct {
	packagePath string
	span        practicalSourceSpan
}
type practicalClosureDeclaration struct {
	ref                  practicalClosureRef
	moduleSourceID, kind string
	names, roles         []string
}
type practicalClosureCall struct {
	caller, callee practicalClosureRef
	span           practicalSourceSpan
}
type practicalClosureDependency struct {
	from, to practicalClosureRef
	role     string
}
type practicalClosureIssue struct {
	code        string
	declaration practicalClosureRef
	span        practicalSourceSpan
	path        []practicalClosureRef
}
type practicalSourceClosure struct {
	imageDigest, compilerDigest, mainSourceID string
	roots                                     []practicalClosureRef
	declarations                              []practicalClosureDeclaration
	calls                                     []practicalClosureCall
	dependencies                              []practicalClosureDependency
	issues                                    []practicalClosureIssue
	sources                                   []practicalSourceBuffer
}
type practicalClosureOriginal struct {
	declaration practicalClosureDeclaration
	node        ast.Node
	loaded      *practicalTypedPackage
	values      []ast.Expr
	typ         ast.Expr
}
type practicalClosureWork struct {
	ref  practicalClosureRef
	role string
	path []practicalClosureRef
}
type practicalClosureBuilder struct {
	owner        *practicalOriginalPackages
	originals    map[practicalClosureRef]*practicalClosureOriginal
	objects      map[types.Object]practicalClosureRef
	roles        map[practicalClosureRef]map[string]bool
	sources      map[string]practicalSourceBuffer
	calls        []practicalClosureCall
	dependencies []practicalClosureDependency
	issues       []practicalClosureIssue
	work         []practicalClosureWork
	done         map[practicalClosureRef]map[string]bool
}

func practicalClosureLess(a, b practicalClosureRef) bool {
	if a.packagePath != b.packagePath {
		return a.packagePath < b.packagePath
	}
	if a.span.sourceID != b.span.sourceID {
		return a.span.sourceID < b.span.sourceID
	}
	if a.span.start != b.span.start {
		return a.span.start < b.span.start
	}
	return a.span.end < b.span.end
}
func practicalClosurePresent(n ast.Node) bool {
	if n == nil {
		return false
	}
	if fields, ok := n.(*ast.FieldList); ok {
		return fields != nil
	}
	return true
}
func practicalClosureChildren(n ast.Node) []ast.Node {
	children := []ast.Node{}
	ast.Inspect(n, func(child ast.Node) bool {
		if child == nil {
			return false
		}
		if child == n {
			return true
		}
		children = append(children, child)
		return false
	})
	return children
}
func (b *practicalClosureBuilder) span(n ast.Node) (practicalSourceSpan, error) {
	s, _, err := b.owner.loader.syntax.spanBytes(n)
	return s, err
}
func (b *practicalClosureBuilder) source(pkg string, span practicalSourceSpan) error {
	if s, ok := b.sources[span.sourceID]; ok {
		if span.start < 0 || span.end <= span.start || span.end > len(s.data) {
			return fmt.Errorf("closure source span is out of bounds")
		}
		return nil
	}
	s, err := b.owner.storageOriginalSpan(pkg, span)
	if err != nil {
		return err
	}
	b.sources[span.sourceID] = s
	return nil
}
func (b *practicalClosureBuilder) add(loaded *practicalTypedPackage, n ast.Node, kind string, names []*ast.Ident, values []ast.Expr, typ ast.Expr) (practicalClosureRef, error) {
	span, err := b.span(n)
	if err != nil {
		return practicalClosureRef{}, err
	}
	ref := practicalClosureRef{loaded.importPath, span}
	if err := b.source(ref.packagePath, span); err != nil {
		return practicalClosureRef{}, err
	}
	if prior := b.originals[ref]; prior != nil {
		if prior.node != n || prior.loaded != loaded {
			return practicalClosureRef{}, fmt.Errorf("ambiguous original closure declaration")
		}
		return ref, nil
	}
	d := practicalClosureDeclaration{ref: ref, moduleSourceID: loaded.moduleSourceID, kind: kind}
	for _, id := range names {
		d.names = append(d.names, id.Name)
		if obj := loaded.info.Defs[id]; obj != nil {
			if fn, ok := obj.(*types.Func); ok {
				obj = fn.Origin()
			}
			if prior, ok := b.objects[obj]; ok && prior != ref {
				return practicalClosureRef{}, fmt.Errorf("closure object has multiple original declarations")
			}
			b.objects[obj] = ref
		}
	}
	b.originals[ref] = &practicalClosureOriginal{declaration: d, node: n, loaded: loaded, values: values, typ: typ}
	return ref, nil
}
func (b *practicalClosureBuilder) specs(loaded *practicalTypedPackage, g *ast.GenDecl) error {
	var inherited []ast.Expr
	var inheritedType ast.Expr
	for _, s := range g.Specs {
		switch s := s.(type) {
		case *ast.TypeSpec:
			if _, err := b.add(loaded, s, "type", []*ast.Ident{s.Name}, nil, s.Type); err != nil {
				return err
			}
		case *ast.ValueSpec:
			values, typ := s.Values, s.Type
			if g.Tok == token.CONST {
				if len(values) == 0 {
					values, typ = inherited, inheritedType
				} else {
					inherited, inheritedType = values, typ
				}
			}
			if _, err := b.add(loaded, s, g.Tok.String(), s.Names, values, typ); err != nil {
				return err
			}
		}
	}
	return nil
}
func (b *practicalClosureBuilder) index() error {
	names := []string{}
	for name := range b.owner.packages {
		names = append(names, name)
	}
	sort.Strings(names)
	for _, name := range names {
		native := b.owner.loader.packages[name]
		row := b.owner.packages[name]
		if row.intrinsic {
			continue
		}
		if native == nil || native.info == nil || native.types == nil {
			return fmt.Errorf("closure lacks intact original native package")
		}
		// Keep every consulted source, including unselected bodies and initializers.
		for _, object := range row.sources {
			if err := b.source(name, practicalSourceSpan{object.ID, 0, int(object.Bytes)}); err != nil {
				return err
			}
		}
		for _, file := range native.files {
			// Inventory unselected nested declarations without entering their
			// bodies into verification. Native typechecking already saw them.
			pending := []ast.Node{file}
			seen := map[ast.Node]bool{}
			for len(pending) > 0 {
				node := pending[len(pending)-1]
				pending = pending[:len(pending)-1]
				if seen[node] {
					continue
				}
				seen[node] = true
				if declaration, ok := node.(*ast.GenDecl); ok {
					if err := b.specs(native, declaration); err != nil {
						return err
					}
				}
				pending = append(pending, practicalClosureChildren(node)...)
			}
			for _, d := range file.Decls {
				switch d := d.(type) {
				case *ast.FuncDecl:
					if _, err := b.add(native, d, "function", []*ast.Ident{d.Name}, nil, nil); err != nil {
						return err
					}
				case *ast.GenDecl:
					if err := b.specs(native, d); err != nil {
						return err
					}
				}
			}
		}
	}
	return nil
}
func (b *practicalClosureBuilder) need(ref practicalClosureRef, role string, path []practicalClosureRef) {
	if b.roles[ref] == nil {
		b.roles[ref] = map[string]bool{}
	}
	if role != "selected_root" && len(path) > 0 {
		b.dependencies = append(b.dependencies, practicalClosureDependency{path[len(path)-1], ref, role})
	}
	b.roles[ref][role] = true
	next := append([]practicalClosureRef(nil), path...)
	if len(next) == 0 || next[len(next)-1] != ref {
		next = append(next, ref)
	}
	b.work = append(b.work, practicalClosureWork{ref, role, next})
}
func (b *practicalClosureBuilder) issue(code string, ref practicalClosureRef, n ast.Node, path []practicalClosureRef) error {
	span, err := b.span(n)
	if err != nil {
		return err
	}
	b.issues = append(b.issues, practicalClosureIssue{code, ref, span, append([]practicalClosureRef(nil), path...)})
	return nil
}
func (b *practicalClosureBuilder) object(obj types.Object, role string, path []practicalClosureRef) error {
	if obj == nil || obj.Pkg() == nil {
		return nil
	}
	if name, ok := obj.(*types.TypeName); ok {
		if _, parameter := name.Type().(*types.TypeParam); parameter {
			return nil
		}
	}
	if fn, ok := obj.(*types.Func); ok {
		obj = fn.Origin()
	}
	ref, ok := b.objects[obj]
	if !ok {
		return fmt.Errorf("closure object has no captured original declaration: %s", obj.Name())
	}
	b.need(ref, role, path)
	return nil
}
func (b *practicalClosureBuilder) types(root types.Type, path []practicalClosureRef) error {
	seen := map[types.Type]bool{}
	work := []types.Type{root}
	for len(work) > 0 {
		typ := work[len(work)-1]
		work = work[:len(work)-1]
		if typ == nil || seen[typ] {
			continue
		}
		seen[typ] = true
		switch typ := typ.(type) {
		case *types.Alias:
			if err := b.object(typ.Obj(), "type_dependency", path); err != nil {
				return err
			}
			work = append(work, typ.Rhs())
		case *types.Named:
			if err := b.object(typ.Obj(), "type_dependency", path); err != nil {
				return err
			}
			work = append(work, typ.Underlying())
			// A concrete error's exact method signature is a type dependency. Its body
			// is required only by an actual static call or an explicit selected root.
			if types.Implements(typ, types.Universe.Lookup("error").Type().Underlying().(*types.Interface)) {
				method := types.NewMethodSet(typ).Lookup(nil, "Error")
				if method != nil {
					if err := b.object(method.Obj(), "signature_dependency", path); err != nil {
						return err
					}
				}
			}
		case *types.Array:
			work = append(work, typ.Elem())
		case *types.Slice:
			work = append(work, typ.Elem())
		case *types.Pointer:
			work = append(work, typ.Elem())
		case *types.Map:
			work = append(work, typ.Key(), typ.Elem())
		case *types.Chan:
			work = append(work, typ.Elem())
		case *types.Struct:
			for i := 0; i < typ.NumFields(); i++ {
				work = append(work, typ.Field(i).Type())
			}
		case *types.Signature:
			if typ.Recv() != nil {
				work = append(work, typ.Recv().Type())
			}
			for _, tuple := range []*types.Tuple{typ.Params(), typ.Results()} {
				for i := 0; i < tuple.Len(); i++ {
					work = append(work, tuple.At(i).Type())
				}
			}
		}
	}
	return nil
}
func practicalClosureUnparen(e ast.Expr) ast.Expr {
	for {
		p, ok := e.(*ast.ParenExpr)
		if !ok {
			return e
		}
		e = p.X
	}
}
func practicalClosureFunction(info *types.Info, e ast.Expr) (types.Object, *types.Selection) {
	switch e := practicalClosureUnparen(e).(type) {
	case *ast.Ident:
		return info.Uses[e], nil
	case *ast.SelectorExpr:
		if s := info.Selections[e]; s != nil {
			return s.Obj(), s
		}
		return info.Uses[e.Sel], nil
	case *ast.IndexExpr:
		return practicalClosureFunction(info, e.X)
	case *ast.IndexListExpr:
		return practicalClosureFunction(info, e.X)
	}
	return nil, nil
}
func (b *practicalClosureBuilder) walk(ref practicalClosureRef, roots []ast.Node, runtime bool, path []practicalClosureRef) error {
	original := b.originals[ref]
	info := original.loaded.info
	type task struct {
		node      ast.Node
		runtime   bool
		signature bool
	}
	// Repeated ConstSpecs reuse an AST but resolve names at each current
	// declaration. Info.Uses has only one slot per reused Ident, so it cannot
	// by itself describe every inherited declaration's dependency.
	var constantScope *types.Scope
	constantPosition := token.NoPos
	if original.declaration.kind == "const" {
		constantPosition = original.node.Pos()
		constantScope = original.loaded.types.Scope().Innermost(constantPosition)
		if constantScope == nil {
			return fmt.Errorf("constant dependency lost its original lexical scope")
		}
	}
	nonLexical := map[*ast.Ident]bool{}
	work := []task{}
	for i := len(roots) - 1; i >= 0; i-- {
		if practicalClosurePresent(roots[i]) {
			work = append(work, task{roots[i], runtime, false})
		}
	}
	push := func(n ast.Node, run, sig bool) {
		if practicalClosurePresent(n) {
			work = append(work, task{n, run, sig})
		}
	}
	for len(work) > 0 {
		t := work[len(work)-1]
		work = work[:len(work)-1]
		n := t.node
		if expr, ok := n.(ast.Expr); ok {
			if fact, ok := info.Types[expr]; ok && fact.IsType() {
				t.runtime = false
			}
		}
		switch n := n.(type) {
		case *ast.GenDecl:
			if err := b.specs(original.loaded, n); err != nil {
				return err
			}
			for i := len(n.Specs) - 1; i >= 0; i-- {
				spec := n.Specs[i]
				if n.Tok == token.CONST || n.Tok == token.TYPE {
					span, err := b.span(spec)
					if err != nil {
						return err
					}
					role := "type_dependency"
					if n.Tok == token.CONST {
						role = "constant_dependency"
					}
					b.need(practicalClosureRef{ref.packagePath, span}, role, path)
				} else {
					span, err := b.span(spec)
					if err != nil {
						return err
					}
					r := practicalClosureRef{ref.packagePath, span}
					if b.roles[r] == nil {
						b.roles[r] = map[string]bool{}
					}
					b.roles[r]["body_declaration"] = true
					push(spec, t.runtime, false)
				}
			}
			continue
		case *ast.TypeSpec:
			if n.TypeParams != nil {
				if err := b.issue("generic_declaration", ref, n, path); err != nil {
					return err
				}
			}
			push(n.Type, false, false)
			continue
		case *ast.ValueSpec:
			// Local const declarations may inherit expressions at a later iota. Native
			// object values/scopes were checked from the intact package before entry.
			if len(n.Names) > 0 {
				if obj := info.Defs[n.Names[0]]; obj != nil {
					if _, ok := obj.(*types.Const); ok {
						valueRef, ok := b.objects[obj]
						if !ok {
							return fmt.Errorf("local const lost its original declaration")
						}
						b.need(valueRef, "constant_dependency", path)
						continue
					}
				}
			}
			push(n.Type, false, false)
			for i := len(n.Values) - 1; i >= 0; i-- {
				push(n.Values[i], t.runtime, false)
			}
			continue
		case *ast.FuncType:
			if !t.signature {
				if err := b.issue("function_value_type", ref, n, path); err != nil {
					return err
				}
			}
			if n.TypeParams != nil {
				if err := b.issue("generic_declaration", ref, n, path); err != nil {
					return err
				}
			}
			if n.Results != nil {
				count := 0
				for _, field := range n.Results.List {
					width := len(field.Names)
					if width == 0 {
						width = 1
					}
					count += width
				}
				if count > 16 {
					if err := b.issue("result_limit", ref, n.Results, path); err != nil {
						return err
					}
				}
			}
			push(n.Results, false, false)
			push(n.Params, false, false)
			continue
		case *ast.CallExpr:
			obj, selection := practicalClosureFunction(info, n.Fun)
			isType := false
			if fact, ok := info.Types[n.Fun]; ok {
				isType = fact.IsType()
			}
			runArgs := t.runtime
			if builtin, ok := obj.(*types.Builtin); ok {
				if builtin.Name() != "len" && builtin.Name() != "cap" && builtin.Name() != "append" && builtin.Name() != "copy" && builtin.Name() != "make" {
					if err := b.issue("builtin_"+builtin.Name(), ref, n, path); err != nil {
						return err
					}
				}
				if (builtin.Name() == "len" || builtin.Name() == "cap") && info.Types[n].Value != nil {
					runArgs = false
				}
			} else if fn, ok := obj.(*types.Func); ok && t.runtime {
				if selection != nil {
					if _, ok := selection.Recv().Underlying().(*types.Interface); ok {
						if err := b.issue("indirect_call", ref, n, path); err != nil {
							return err
						}
						goto arguments
					}
					if len(selection.Index()) > 1 {
						if err := b.issue("promoted_selector", ref, n, path); err != nil {
							return err
						}
					}
				}
				target, ok := b.objects[fn.Origin()]
				if !ok {
					if err := b.issue("uncaptured_call", ref, n, path); err != nil {
						return err
					}
				} else {
					span, err := b.span(n)
					if err != nil {
						return err
					}
					b.calls = append(b.calls, practicalClosureCall{ref, target, span})
					b.need(target, "body_dependency", path)
				}
			} else if !isType {
				if _, ok := obj.(*types.Builtin); !ok && t.runtime {
					if err := b.issue("indirect_call", ref, n, path); err != nil {
						return err
					}
				}
			}
		arguments:
			for i := len(n.Args) - 1; i >= 0; i-- {
				push(n.Args[i], runArgs, false)
			}
			push(n.Fun, false, false)
			continue
		case *ast.Ident:
			obj := info.Uses[n]
			if obj == nil {
				continue
			}
			field := false
			if variable, ok := obj.(*types.Var); ok {
				field = variable.IsField()
			}
			if constantScope != nil && !nonLexical[n] && !field {
				_, obj = constantScope.LookupParent(n.Name, constantPosition)
				if obj == nil {
					return fmt.Errorf("inherited constant dependency name is unresolved: %s", n.Name)
				}
			}
			switch obj := obj.(type) {
			case *types.TypeName:
				if err := b.object(obj, "type_dependency", path); err != nil {
					return err
				}
				if err := b.types(obj.Type(), path); err != nil {
					return err
				}
			case *types.Const:
				if err := b.object(obj, "constant_dependency", path); err != nil {
					return err
				}
			case *types.Func:
				if err := b.object(obj, "signature_dependency", path); err != nil {
					return err
				}
				if t.runtime {
					if err := b.issue("function_value", ref, n, path); err != nil {
						return err
					}
				}
			case *types.Var:
				if obj.IsField() {
					continue
				}
				if err := b.types(obj.Type(), path); err != nil {
					return err
				}
				if obj.Pkg() != nil && obj.Parent() == obj.Pkg().Scope() {
					if t.runtime {
						if err := b.object(obj, "rejected_dependency", path); err != nil {
							return err
						}
						if err := b.issue("mutable_package_state", ref, n, path); err != nil {
							return err
						}
					} else {
						if err := b.object(obj, "type_dependency", path); err != nil {
							return err
						}
					}
				}
			}
		case *ast.SelectorExpr:
			nonLexical[n.Sel] = true
			if s := info.Selections[n]; s != nil && len(s.Index()) > 1 {
				if err := b.issue("promoted_selector", ref, n, path); err != nil {
					return err
				}
			}
		case *ast.RangeStmt:
			typ := info.Types[n.X].Type
			if typ != nil {
				switch typ.Underlying().(type) {
				case *types.Array, *types.Slice:
				default:
					if err := b.issue("range_kind", ref, n, path); err != nil {
						return err
					}
				}
			}
		case *ast.Ellipsis:
			if n.Elt != nil {
				if err := b.issue("variadic_signature", ref, n, path); err != nil {
					return err
				}
			}
		case *ast.BranchStmt:
			if n.Label != nil || (n.Tok != token.BREAK && n.Tok != token.CONTINUE) {
				if err := b.issue("branch_kind", ref, n, path); err != nil {
					return err
				}
			}
		case *ast.UnaryExpr:
			if n.Op != token.ADD && n.Op != token.SUB && n.Op != token.NOT && n.Op != token.XOR {
				if err := b.issue("unary_kind", ref, n, path); err != nil {
					return err
				}
			}
		case *ast.BadDecl, *ast.BadExpr, *ast.BadStmt, *ast.MapType, *ast.ChanType, *ast.InterfaceType, *ast.StarExpr, *ast.TypeAssertExpr, *ast.TypeSwitchStmt, *ast.FuncLit, *ast.IndexListExpr, *ast.LabeledStmt, *ast.GoStmt, *ast.DeferStmt, *ast.SendStmt, *ast.SelectStmt, *ast.CommClause:
			if err := b.issue(fmt.Sprintf("syntax_%T", n), ref, n, path); err != nil {
				return err
			}
		}
		children := practicalClosureChildren(n)
		for i := len(children) - 1; i >= 0; i-- {
			push(children[i], t.runtime, false)
		}
	}
	return nil
}
func (b *practicalClosureBuilder) process(w practicalClosureWork) error {
	original := b.originals[w.ref]
	if original == nil {
		return fmt.Errorf("closure requires an unknown declaration")
	}
	role := w.role
	if role == "selected_root" {
		role = "body_dependency"
	}
	if b.done[w.ref] == nil {
		b.done[w.ref] = map[string]bool{}
	}
	if b.done[w.ref][role] {
		return nil
	}
	b.done[w.ref][role] = true
	switch n := original.node.(type) {
	case *ast.FuncDecl:
		if n.Doc != nil {
			for _, comment := range n.Doc.List {
				if strings.HasPrefix(comment.Text, "//go:") {
					if err := b.issue("compiler_directive", w.ref, comment, w.path); err != nil {
						return err
					}
				}
			}
		}

		if err := b.walk(w.ref, []ast.Node{n.Recv}, false, w.path); err != nil {
			return err
		}
		// A declaration's own FuncType is a static signature, not a function value.
		if err := b.signature(w.ref, n.Type, w.path); err != nil {
			return err
		}
		if role == "body_dependency" {
			if n.Name.Name == "init" {
				if err := b.issue("initialization_root", w.ref, n, w.path); err != nil {
					return err
				}
			}
			if n.Body == nil {
				return b.issue("missing_body", w.ref, n, w.path)
			}
			return b.walk(w.ref, []ast.Node{n.Body}, true, w.path)
		}
	case *ast.TypeSpec:
		return b.walk(w.ref, []ast.Node{n}, false, w.path)
	case *ast.ValueSpec:
		roots := []ast.Node{}
		if original.typ != nil {
			roots = append(roots, original.typ)
		}
		if role == "constant_dependency" {
			for _, value := range original.values {
				roots = append(roots, value)
			}
		}
		for _, id := range n.Names {
			if obj := original.loaded.info.Defs[id]; obj != nil {
				if err := b.types(obj.Type(), w.path); err != nil {
					return err
				}
			}
		}
		return b.walk(w.ref, roots, false, w.path)
	}
	return nil
}
func (b *practicalClosureBuilder) signature(ref practicalClosureRef, n *ast.FuncType, path []practicalClosureRef) error {
	if n.TypeParams != nil {
		if err := b.issue("generic_declaration", ref, n, path); err != nil {
			return err
		}
	}
	if n.Results != nil {
		count := 0
		for _, f := range n.Results.List {
			if len(f.Names) == 0 {
				count++
			} else {
				count += len(f.Names)
			}
		}
		if count > 16 {
			if err := b.issue("result_limit", ref, n.Results, path); err != nil {
				return err
			}
		}
	}
	return b.walk(ref, []ast.Node{n.Params, n.Results}, false, path)
}
func (b *practicalClosureBuilder) cycles(roots []practicalClosureRef) error {
	edges := map[practicalClosureRef][]practicalClosureCall{}
	for _, call := range b.calls {
		edges[call.caller] = append(edges[call.caller], call)
	}
	state := map[practicalClosureRef]int{}
	type frame struct {
		ref  practicalClosureRef
		next int
	}
	for _, root := range roots {
		if state[root] != 0 {
			continue
		}
		stack := []frame{{root, 0}}
		state[root] = 1
		for len(stack) > 0 {
			top := &stack[len(stack)-1]
			calls := edges[top.ref]
			if top.next == len(calls) {
				state[top.ref] = 2
				stack = stack[:len(stack)-1]
				continue
			}
			call := calls[top.next]
			top.next++
			if state[call.callee] == 1 {
				path := []practicalClosureRef{}
				for _, f := range stack {
					path = append(path, f.ref)
				}
				path = append(path, call.callee)
				b.issues = append(b.issues, practicalClosureIssue{"call_cycle", call.caller, call.span, path})
			} else if state[call.callee] == 0 {
				state[call.callee] = 1
				stack = append(stack, frame{call.callee, 0})
			}
		}
	}
	return nil
}

// Observe selected original declarations only after the owner has reconstructed
// and typechecked the complete captured packages, including unselected bodies.
// Callers select physical roots; they supply no AST, type facts, adapters or
// unreachability oracle. Application initializers are never executed.
func (owner *practicalOriginalPackages) sourceClosure(roots []practicalClosureRef) (practicalSourceClosure, error) {
	if owner == nil {
		return practicalSourceClosure{}, fmt.Errorf("source closure requires an owned original universe")
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	if owner.closed || owner.inputs == nil || owner.loader == nil || owner.loader.syntax == nil {
		return practicalSourceClosure{}, fmt.Errorf("original source closure owner is closed or incomplete")
	}
	for _, err := range []error{owner.inputs.failed, owner.loader.failed, owner.loader.syntax.failed} {
		if err != nil {
			return practicalSourceClosure{}, err
		}
	}
	if len(roots) == 0 {
		return practicalSourceClosure{}, fmt.Errorf("source closure requires explicit roots")
	}
	b := &practicalClosureBuilder{owner: owner, originals: map[practicalClosureRef]*practicalClosureOriginal{}, objects: map[types.Object]practicalClosureRef{}, roles: map[practicalClosureRef]map[string]bool{}, sources: map[string]practicalSourceBuffer{}, done: map[practicalClosureRef]map[string]bool{}}
	if err := b.index(); err != nil {
		return practicalSourceClosure{}, err
	}
	roots = append([]practicalClosureRef(nil), roots...)
	sort.Slice(roots, func(i, j int) bool { return practicalClosureLess(roots[i], roots[j]) })
	for i, ref := range roots {
		original := b.originals[ref]
		if original == nil || original.declaration.kind != "function" || (i > 0 && roots[i-1] == ref) {
			return practicalSourceClosure{}, fmt.Errorf("source closure needs distinct exact original function roots")
		}
		b.need(ref, "selected_root", []practicalClosureRef{ref})
	}
	for cursor := 0; cursor < len(b.work); cursor++ {
		if err := b.process(b.work[cursor]); err != nil {
			return practicalSourceClosure{}, err
		}
	}
	sort.Slice(b.calls, func(i, j int) bool {
		a, c := b.calls[i], b.calls[j]
		if a.caller != c.caller {
			return practicalClosureLess(a.caller, c.caller)
		}
		if a.span.start != c.span.start {
			return a.span.start < c.span.start
		}
		return practicalClosureLess(a.callee, c.callee)
	})
	if err := b.cycles(roots); err != nil {
		return practicalSourceClosure{}, err
	}
	result := practicalSourceClosure{imageDigest: owner.imageDigest, compilerDigest: owner.compilerDigest, mainSourceID: owner.mainSourceID, roots: roots, calls: b.calls}
	for _, issue := range b.issues {
		if b.roles[issue.declaration] == nil {
			b.roles[issue.declaration] = map[string]bool{}
		}
		b.roles[issue.declaration]["rejected_dependency"] = true
	}
	for ref, original := range b.originals {
		d := original.declaration
		for role := range b.roles[ref] {
			d.roles = append(d.roles, role)
		}
		if len(d.roles) == 0 {
			d.roles = []string{"type_check_only"}
		}
		sort.Strings(d.roles)
		result.declarations = append(result.declarations, d)
	}
	sort.Slice(result.declarations, func(i, j int) bool {
		return practicalClosureLess(result.declarations[i].ref, result.declarations[j].ref)
	})
	sort.Slice(b.issues, func(i, j int) bool {
		a, c := b.issues[i], b.issues[j]
		if a.declaration != c.declaration {
			return practicalClosureLess(a.declaration, c.declaration)
		}
		if a.span.start != c.span.start {
			return a.span.start < c.span.start
		}
		return a.code < c.code
	})
	for _, issue := range b.issues {
		if len(result.issues) > 0 {
			p := result.issues[len(result.issues)-1]
			if p.declaration == issue.declaration && p.span == issue.span && p.code == issue.code {
				continue
			}
		}
		result.issues = append(result.issues, issue)
	}
	sort.Slice(b.dependencies, func(i, j int) bool {
		a, c := b.dependencies[i], b.dependencies[j]
		if a.from != c.from {
			return practicalClosureLess(a.from, c.from)
		}
		if a.to != c.to {
			return practicalClosureLess(a.to, c.to)
		}
		return a.role < c.role
	})
	for _, dependency := range b.dependencies {
		if len(result.dependencies) > 0 && result.dependencies[len(result.dependencies)-1] == dependency {
			continue
		}
		result.dependencies = append(result.dependencies, dependency)
	}
	ids := []string{}
	for id := range b.sources {
		ids = append(ids, id)
	}
	sort.Strings(ids)
	for _, id := range ids {
		result.sources = append(result.sources, b.sources[id])
	}
	return result, nil
}
