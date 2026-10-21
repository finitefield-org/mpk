package main

import (
	"bytes"
	"fmt"
	"go/ast"
	"go/types"
)

// These are the existing frozen structural limits, not new semantic budgets.
const (
	maximumPracticalStorageTypes  = 4096
	maximumPracticalStorageDepth  = 16
	maximumPracticalStorageFields = 64
	maximumPracticalStorageArray  = 256
)

// Local node numbers only describe this owned native observation. They are
// never source-type IDs, nominal identity proofs or closed-context members.
type practicalStorageType struct {
	kind, display      string
	width              int
	signed             bool
	length             uint64
	child, declaration int
	fields             []practicalStorageField
	depth              int
}
type practicalStorageField struct {
	name, packagePath, tag string
	embedded               bool
	typeNode               int
}
type practicalStorageDeclaration struct {
	name, packagePath, moduleSourceID string
	alias                             bool
	span                              practicalSourceSpan
	source                            practicalSourceBuffer
}
type practicalStorageTypeBuilder struct {
	owner        *practicalOriginalPackages
	nodes        []practicalStorageType
	types        []types.Type
	known        map[types.Type]int
	visiting     map[types.Type]bool
	declarations []practicalStorageDeclaration
	declared     map[*types.TypeName]int
	specs        map[string]map[*types.TypeName]*ast.TypeSpec
}

func newPracticalStorageTypes(owner *practicalOriginalPackages) *practicalStorageTypeBuilder {
	return &practicalStorageTypeBuilder{owner: owner, known: map[types.Type]int{}, visiting: map[types.Type]bool{}, declared: map[*types.TypeName]int{}, specs: map[string]map[*types.TypeName]*ast.TypeSpec{}}
}
func practicalStorageDisplay(t types.Type) string {
	return types.TypeString(t, func(p *types.Package) string { return p.Path() })
}
func (b *practicalStorageTypeBuilder) declaration(obj *types.TypeName) (int, error) {
	if n, ok := b.declared[obj]; ok {
		return n, nil
	}
	if obj == nil || obj.Pkg() == nil {
		return 0, fmt.Errorf("storage type has no original declaration")
	}
	name := obj.Pkg().Path()
	loaded := b.owner.loader.packages[name]
	pkg, ok := b.owner.packages[name]
	if !ok || pkg.intrinsic || loaded == nil || loaded.types != obj.Pkg() || loaded.info == nil {
		return 0, fmt.Errorf("storage type declaration is outside the owned original package universe")
	}
	index, exists := b.specs[name]
	if !exists {
		index = map[*types.TypeName]*ast.TypeSpec{}
		for _, file := range loaded.files {
			ast.Inspect(file, func(node ast.Node) bool {
				if spec, ok := node.(*ast.TypeSpec); ok {
					if object, ok := loaded.info.Defs[spec.Name].(*types.TypeName); ok {
						index[object] = spec
					}
				}
				return true
			})
		}
		b.specs[name] = index
	}
	spec := index[obj]
	if spec == nil || spec.Name.Name != obj.Name() || spec.Name.Pos() != obj.Pos() || spec.Assign.IsValid() != obj.IsAlias() {
		return 0, fmt.Errorf("storage type lost its exact original declaration")
	}
	span, raw, err := b.owner.loader.syntax.spanBytes(spec)
	if err != nil {
		return 0, err
	}
	source, err := b.owner.storageOriginalSpan(name, span)
	if err != nil {
		return 0, err
	}
	if !bytes.Equal(raw, source.data[span.start:span.end]) {
		return 0, fmt.Errorf("storage type declaration differs from original source bytes")
	}
	n := len(b.declarations)
	b.declarations = append(b.declarations, practicalStorageDeclaration{name: obj.Name(), packagePath: name, moduleSourceID: loaded.moduleSourceID, alias: obj.IsAlias(), span: span, source: source})
	b.declared[obj] = n
	return n, nil
}
func (b *practicalStorageTypeBuilder) capture(t types.Type) (int, error) {
	if t == nil {
		return 0, fmt.Errorf("storage observation has no native type")
	}
	if n, ok := b.known[t]; ok {
		return n, nil
	}
	if b.visiting[t] {
		return 0, fmt.Errorf("storage type closure is recursive, including zero-length and slice edges")
	}
	if len(b.known)+len(b.visiting) >= maximumPracticalStorageTypes {
		return 0, fmt.Errorf("storage type closure exceeds the frozen type count")
	}
	b.visiting[t] = true
	defer delete(b.visiting, t)
	node := practicalStorageType{display: practicalStorageDisplay(t), child: -1, declaration: -1}
	child := func(t types.Type, edge int) error {
		n, err := b.capture(t)
		if err != nil {
			return err
		}
		node.child = n
		node.depth = b.nodes[n].depth + edge
		return nil
	}
	switch v := t.(type) {
	case *types.Alias:
		if v.TypeParams() != nil && v.TypeParams().Len() != 0 || v.TypeArgs() != nil && v.TypeArgs().Len() != 0 {
			return 0, fmt.Errorf("generic aliases are outside the frozen storage type domain")
		}
		if v.Obj().Pkg() == nil {
			if v.Obj().Parent() != types.Universe || (v.Obj().Name() != "byte" && v.Obj().Name() != "rune") {
				return 0, fmt.Errorf("unknown predeclared storage alias")
			}
			node.kind = "builtin_alias"
		} else {
			node.kind = "alias"
			n, err := b.declaration(v.Obj())
			if err != nil {
				return 0, err
			}
			node.declaration = n
		}
		if err := child(v.Rhs(), 0); err != nil {
			return 0, err
		}
	case *types.Named:
		if v == types.Universe.Lookup("error").Type() {
			node.kind = "error"
			break
		}
		if v.TypeParams() != nil && v.TypeParams().Len() != 0 || v.TypeArgs() != nil && v.TypeArgs().Len() != 0 {
			return 0, fmt.Errorf("generic named types are outside the frozen storage type domain")
		}
		node.kind = "named"
		n, err := b.declaration(v.Obj())
		if err != nil {
			return 0, err
		}
		node.declaration = n
		if _, ok := v.Underlying().(*types.Interface); ok {
			return 0, fmt.Errorf("a named interface is not the predeclared error type")
		}
		if err := child(v.Underlying(), 0); err != nil {
			return 0, err
		}
	case *types.Basic:
		switch v.Kind() {
		case types.Bool:
			node.kind = "bool"
		case types.String:
			node.kind = "string"
		case types.Int8, types.Int16, types.Int32, types.Int64, types.Int, types.Uint8, types.Uint16, types.Uint32, types.Uint64, types.Uint:
			node.kind = "word"
			node.signed = v.Info()&types.IsUnsigned == 0
			switch v.Kind() {
			case types.Int8, types.Uint8:
				node.width = 8
			case types.Int16, types.Uint16:
				node.width = 16
			case types.Int32, types.Uint32:
				node.width = 32
			default:
				node.width = 64
			}
		default:
			return 0, fmt.Errorf("native scalar %s is outside the frozen storage type domain", node.display)
		}
	case *types.Array:
		if v.Len() < 0 || v.Len() > maximumPracticalStorageArray {
			return 0, fmt.Errorf("storage array exceeds the frozen 256-element limit")
		}
		node.kind = "array"
		node.length = uint64(v.Len())
		if err := child(v.Elem(), 1); err != nil {
			return 0, err
		}
	case *types.Slice:
		node.kind = "slice"
		if err := child(v.Elem(), 1); err != nil {
			return 0, err
		}
	case *types.Struct:
		if v.NumFields() > maximumPracticalStorageFields {
			return 0, fmt.Errorf("storage struct exceeds the frozen 64-field limit")
		}
		node.kind = "struct"
		node.depth = 1
		for i := 0; i < v.NumFields(); i++ {
			field := v.Field(i)
			n, err := b.capture(field.Type())
			if err != nil {
				return 0, err
			}
			pkg := ""
			if field.Pkg() != nil {
				pkg = field.Pkg().Path()
			}
			node.fields = append(node.fields, practicalStorageField{name: field.Name(), packagePath: pkg, tag: v.Tag(i), embedded: field.Embedded(), typeNode: n})
			if d := b.nodes[n].depth + 1; d > node.depth {
				node.depth = d
			}
		}
	default:
		return 0, fmt.Errorf("native storage type %s is outside the frozen type domain", node.display)
	}
	if node.depth > maximumPracticalStorageDepth {
		return 0, fmt.Errorf("storage aggregate nesting exceeds the frozen depth16")
	}
	n := len(b.nodes)
	b.nodes = append(b.nodes, node)
	b.types = append(b.types, t)
	b.known[t] = n
	return n, nil
}
