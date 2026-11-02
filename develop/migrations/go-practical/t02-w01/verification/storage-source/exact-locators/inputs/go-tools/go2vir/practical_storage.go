package main

import (
	"bytes"
	"fmt"
	"go/ast"
	"go/constant"
	"go/types"
)

type practicalStorageIndex struct {
	span          practicalSourceSpan
	typeNode      int
	nativeDisplay string
	defaulted     bool
	width         int
	signed        bool
	// Empty means a runtime expression, not zero. Constants retain their
	// complete mathematical value; this observation never truncates an index.
	constant string
}
type practicalStorageStep struct {
	kind           string
	parent, result int
	field          uint64
	index          practicalStorageIndex
	span           practicalSourceSpan
}
type practicalStorageRoot struct {
	kind                                      string // value_binding or slice_descriptor
	typeNode                                  int
	expression                                practicalSourceSpan
	binding                                   practicalSourceSpan
	bindingPackage, bindingName, bindingScope string
}

// Native source facts, not serialized locations or source/store permissions.
// A slice_index is the first operation after a descriptor pivot. Lowering must
// resolve that descriptor's existing owner/path/offset before extending it.
type practicalStorageRoute struct {
	packagePath, moduleSourceID, imageDigest, compilerDigest string
	span                                                     practicalSourceSpan
	source                                                   practicalSourceBuffer
	expression                                               []byte
	root                                                     practicalStorageRoot
	steps                                                    []practicalStorageStep
	result                                                   int
	types                                                    []practicalStorageType
	declarations                                             []practicalStorageDeclaration
}

// Caller holds the package owner mutex. Membership is checked in the complete
// original export/source inventory, including when declarations cross imports.
func (owner *practicalOriginalPackages) storageOriginalSpan(importPath string, span practicalSourceSpan) (practicalSourceBuffer, error) {
	pkg, ok := owner.packages[importPath]
	if !ok || pkg.intrinsic || span.start < 0 || span.end <= span.start {
		return practicalSourceBuffer{}, fmt.Errorf("storage span has no original source package membership")
	}
	var object practicalSourceObject
	for _, source := range pkg.sources {
		if source.ID == span.sourceID {
			if object.ID != "" {
				return practicalSourceBuffer{}, fmt.Errorf("storage span has ambiguous source membership")
			}
			object = source
		}
	}
	if object.ID == "" || object.Kind != "go_file" {
		return practicalSourceBuffer{}, fmt.Errorf("storage span crossed its original package source membership")
	}
	source, err := owner.inputs.readSource(object.Capability, object.Path, object.Kind)
	if err != nil {
		return practicalSourceBuffer{}, err
	}
	parsed := owner.loader.syntax.files[object.ID]
	if source.object != object || parsed == nil || parsed.source.object != object || span.end > len(source.data) || !bytes.Equal(source.data, parsed.source.data) {
		return practicalSourceBuffer{}, owner.inputs.reject(fmt.Errorf("storage observation lost its complete original source bytes"))
	}
	if _, err := source.canonicalObject(); err != nil {
		return practicalSourceBuffer{}, err
	}
	return practicalSourceBuffer{object: source.object, data: source.sourceBytes()}, nil
}
func (owner *practicalOriginalPackages) storageRoute(importPath string, span practicalSourceSpan) (practicalStorageRoute, error) {
	if owner == nil {
		return practicalStorageRoute{}, fmt.Errorf("storage route requires an owned original package universe")
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	if owner.closed || owner.inputs == nil || owner.loader == nil || owner.loader.syntax == nil {
		return practicalStorageRoute{}, fmt.Errorf("original package universe is closed or incomplete")
	}
	for _, err := range []error{owner.inputs.failed, owner.loader.failed, owner.loader.syntax.failed} {
		if err != nil {
			return practicalStorageRoute{}, err
		}
	}
	source, err := owner.storageOriginalSpan(importPath, span)
	if err != nil {
		return practicalStorageRoute{}, err
	}
	loaded := owner.loader.packages[importPath]
	if loaded == nil || loaded.info == nil {
		return practicalStorageRoute{}, fmt.Errorf("storage route has no original native types")
	}
	var expression ast.Expr
	for candidate, fact := range loaded.info.Types {
		binding, exists := owner.loader.syntax.nodes[candidate]
		if !exists || binding.span != span || fact.Type == nil || fact.IsType() || !fact.Addressable() {
			continue
		}
		if expression != nil {
			return practicalStorageRoute{}, fmt.Errorf("storage route span is ambiguous")
		}
		expression = candidate
	}
	if expression == nil {
		return practicalStorageRoute{}, fmt.Errorf("storage route requires one original addressable value expression")
	}
	actual, raw, err := owner.loader.syntax.spanBytes(expression)
	if err != nil {
		return practicalStorageRoute{}, err
	}
	if actual != span || !bytes.Equal(raw, source.data[span.start:span.end]) {
		return practicalStorageRoute{}, fmt.Errorf("storage expression lost its exact physical source span")
	}
	builder := newPracticalStorageTypes(owner)
	result, err := builder.capture(loaded.info.Types[expression].Type)
	if err != nil {
		return practicalStorageRoute{}, err
	}
	route := practicalStorageRoute{packagePath: importPath, moduleSourceID: loaded.moduleSourceID, imageDigest: owner.imageDigest, compilerDigest: owner.compilerDigest, span: span, source: source, expression: bytes.Clone(raw), result: result}
	walker := practicalStorageWalker{owner: owner, loaded: loaded, builder: builder, route: &route}
	if err := walker.walk(expression); err != nil {
		return practicalStorageRoute{}, err
	}
	if len(route.steps) > maximumPracticalStorageDepth {
		return practicalStorageRoute{}, fmt.Errorf("storage path exceeds the frozen 16 components")
	}
	route.types, route.declarations = builder.nodes, builder.declarations
	return route, nil
}

type practicalStorageWalker struct {
	owner   *practicalOriginalPackages
	loaded  *practicalTypedPackage
	builder *practicalStorageTypeBuilder
	route   *practicalStorageRoute
}

func (w *practicalStorageWalker) span(expr ast.Expr) (practicalSourceSpan, error) {
	span, _, err := w.owner.loader.syntax.spanBytes(expr)
	return span, err
}
func (w *practicalStorageWalker) binding(expr ast.Expr, obj *types.Var) error {
	if obj == nil || obj.IsField() || obj.Pkg() == nil {
		return fmt.Errorf("storage root is not an original variable binding")
	}
	pkg := w.owner.loader.packages[obj.Pkg().Path()]
	if pkg == nil || pkg.info == nil || pkg.types != obj.Pkg() {
		return fmt.Errorf("storage root binding is outside the owned native package universe")
	}
	var declaration *ast.Ident
	for identifier, object := range pkg.info.Defs {
		if object == obj {
			if declaration != nil {
				return fmt.Errorf("storage root binding is ambiguous")
			}
			declaration = identifier
		}
	}
	if declaration == nil || declaration.Name != obj.Name() || declaration.Pos() != obj.Pos() {
		return fmt.Errorf("storage root lost its exact native declaration")
	}
	decl, err := w.span(declaration)
	if err != nil {
		return err
	}
	if _, err := w.owner.storageOriginalSpan(obj.Pkg().Path(), decl); err != nil {
		return err
	}
	span, err := w.span(expr)
	if err != nil {
		return err
	}
	n, err := w.builder.capture(obj.Type())
	if err != nil {
		return err
	}
	scope := "local"
	if obj.Parent() == pkg.types.Scope() {
		scope = "package"
	}
	w.route.root = practicalStorageRoot{kind: "value_binding", typeNode: n, expression: span, binding: decl, bindingPackage: obj.Pkg().Path(), bindingName: obj.Name(), bindingScope: scope}
	return nil
}
func (w *practicalStorageWalker) add(step practicalStorageStep) error {
	if len(w.route.steps) >= maximumPracticalStorageDepth {
		return fmt.Errorf("storage path exceeds the frozen 16 components")
	}
	w.route.steps = append(w.route.steps, step)
	return nil
}
func (w *practicalStorageWalker) index(expr ast.Expr) (practicalStorageIndex, error) {
	fact, ok := w.loaded.info.Types[expr]
	if !ok || fact.Type == nil || fact.IsType() {
		return practicalStorageIndex{}, fmt.Errorf("storage index has no original native value type")
	}
	t := types.Default(fact.Type)
	basic, ok := t.Underlying().(*types.Basic)
	if !ok || basic.Info()&types.IsInteger == 0 {
		return practicalStorageIndex{}, fmt.Errorf("storage index is not an original integer expression")
	}
	n, err := w.builder.capture(t)
	if err != nil {
		return practicalStorageIndex{}, err
	}
	typeNode := n
	for w.builder.nodes[n].kind == "named" || w.builder.nodes[n].kind == "alias" || w.builder.nodes[n].kind == "builtin_alias" {
		n = w.builder.nodes[n].child
	}
	shape := w.builder.nodes[n]
	if shape.kind != "word" {
		return practicalStorageIndex{}, fmt.Errorf("storage index is outside the frozen integer domain")
	}
	span, err := w.span(expr)
	if err != nil {
		return practicalStorageIndex{}, err
	}
	result := practicalStorageIndex{span: span, typeNode: typeNode, nativeDisplay: practicalStorageDisplay(fact.Type), defaulted: t != fact.Type, width: shape.width, signed: shape.signed}
	if fact.Value != nil {
		if fact.Value.Kind() != constant.Int {
			return practicalStorageIndex{}, fmt.Errorf("storage index has a noninteger constant")
		}
		result.constant = fact.Value.ExactString()
	}
	return result, nil
}
func (w *practicalStorageWalker) walk(expr ast.Expr) error {
	fact, ok := w.loaded.info.Types[expr]
	if !ok || fact.Type == nil || fact.IsType() {
		return fmt.Errorf("storage path lost an original value expression")
	}
	switch e := expr.(type) {
	case *ast.ParenExpr:
		return w.walk(e.X)
	case *ast.Ident:
		obj, _ := w.loaded.info.Uses[e].(*types.Var)
		return w.binding(e, obj)
	case *ast.SelectorExpr:
		selection := w.loaded.info.Selections[e]
		if selection == nil {
			obj, _ := w.loaded.info.Uses[e.Sel].(*types.Var)
			return w.binding(e, obj)
		}
		if selection.Kind() != types.FieldVal || selection.Indirect() {
			return fmt.Errorf("storage selection is not a direct nonpointer field path")
		}
		if err := w.walk(e.X); err != nil {
			return err
		}
		current := w.loaded.info.Types[e.X].Type
		span, err := w.span(e)
		if err != nil {
			return err
		}
		for _, ordinal := range selection.Index() {
			structure, ok := current.Underlying().(*types.Struct)
			if !ok || ordinal < 0 || ordinal >= structure.NumFields() {
				return fmt.Errorf("native promoted field path crossed a nonstruct or missing field")
			}
			parent, err := w.builder.capture(current)
			if err != nil {
				return err
			}
			current = structure.Field(ordinal).Type()
			child, err := w.builder.capture(current)
			if err != nil {
				return err
			}
			if err := w.add(practicalStorageStep{kind: "field", parent: parent, result: child, field: uint64(ordinal), span: span}); err != nil {
				return err
			}
		}
		if !types.Identical(current, fact.Type) {
			return fmt.Errorf("native field path changed its exact result type")
		}
		return nil
	case *ast.IndexExpr:
		parentType := w.loaded.info.Types[e.X].Type
		if parentType == nil {
			return fmt.Errorf("native indexed value has no original type")
		}
		kind := "array_index"
		switch container := parentType.Underlying().(type) {
		case *types.Array:
			if !types.Identical(container.Elem(), fact.Type) {
				return fmt.Errorf("native array index changed its exact element type")
			}
			if err := w.walk(e.X); err != nil {
				return err
			}
		case *types.Slice:
			if !types.Identical(container.Elem(), fact.Type) {
				return fmt.Errorf("native slice index changed its exact element type")
			}
			kind = "slice_index"
			span, err := w.span(e.X)
			if err != nil {
				return err
			}
			n, err := w.builder.capture(parentType)
			if err != nil {
				return err
			}
			w.route.root = practicalStorageRoot{kind: "slice_descriptor", typeNode: n, expression: span}
			// Reading this descriptor must pivot to its captured live location.
			// Earlier paths belong to descriptor evaluation, not this backing path.
			w.route.steps = nil
		default:
			return fmt.Errorf("native indexing is not an array/slice storage path")
		}
		parent, err := w.builder.capture(parentType)
		if err != nil {
			return err
		}
		child, err := w.builder.capture(fact.Type)
		if err != nil {
			return err
		}
		index, err := w.index(e.Index)
		if err != nil {
			return err
		}
		span, err := w.span(e)
		if err != nil {
			return err
		}
		return w.add(practicalStorageStep{kind: kind, parent: parent, result: child, index: index, span: span})
	default:
		return fmt.Errorf("native expression is not a structural storage path")
	}
}
