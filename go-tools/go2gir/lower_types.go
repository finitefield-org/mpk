package main

import (
	"fmt"
	"go/ast"
	"go/constant"
	"go/token"
	"go/types"
	"strconv"
)

func (l *girFunctionLowerer) lowerCompositeLit(lit *ast.CompositeLit) (girValue, bool) {
	typ := l.packageLowerer.pkg.TypesInfo.TypeOf(lit)
	if typ == nil {
		l.reject(lit.Pos(), "GIR lowering", "composite literal has no type information")
		return girValue{}, false
	}
	girType, ok := girTypeFromGoType(typ)
	if !ok {
		l.reject(lit.Pos(), "GIR lowering", fmt.Sprintf("composite literal type %s is not lowered by GO-006", typ.String()))
		return girValue{}, false
	}

	switch underlying := typ.Underlying().(type) {
	case *types.Struct:
		return l.lowerStructLiteral(lit, girType, underlying)
	case *types.Array:
		return l.lowerArrayLiteral(lit, girType, underlying)
	default:
		l.reject(lit.Pos(), "GIR lowering", fmt.Sprintf("composite literal type %s is not lowered by GO-006", typ.String()))
		return girValue{}, false
	}
}

func (l *girFunctionLowerer) lowerStructLiteral(lit *ast.CompositeLit, typ girType, structType *types.Struct) (girValue, bool) {
	fields := make([]girField, structType.NumFields())
	fieldSeen := make([]bool, structType.NumFields())
	keyed := false
	unkeyed := false

	for index, element := range lit.Elts {
		if keyValue, ok := element.(*ast.KeyValueExpr); ok {
			keyed = true
			fieldName, ok := structLiteralFieldName(keyValue.Key)
			if !ok {
				l.reject(keyValue.Key.Pos(), "GIR lowering", "struct literal keys must be field names")
				return girValue{}, false
			}
			fieldIndex := structFieldIndex(structType, fieldName)
			if fieldIndex < 0 {
				l.reject(keyValue.Key.Pos(), "GIR lowering", fmt.Sprintf("struct literal field %q is not in %s", fieldName, typeString(l.packageLowerer.pkg.TypesInfo.TypeOf(lit))))
				return girValue{}, false
			}
			if fieldSeen[fieldIndex] {
				l.reject(keyValue.Key.Pos(), "GIR lowering", fmt.Sprintf("duplicate struct literal field %q", fieldName))
				return girValue{}, false
			}
			value, ok := l.lowerExpr(keyValue.Value)
			if !ok {
				return girValue{}, false
			}
			fields[fieldIndex] = girField{Name: fieldName, Value: value}
			fieldSeen[fieldIndex] = true
			continue
		}

		unkeyed = true
		if index >= structType.NumFields() {
			l.reject(element.Pos(), "GIR lowering", "struct literal has more values than fields")
			return girValue{}, false
		}
		field := structType.Field(index)
		value, ok := l.lowerExpr(element)
		if !ok {
			return girValue{}, false
		}
		fields[index] = girField{Name: field.Name(), Value: value}
		fieldSeen[index] = true
	}

	if keyed && unkeyed {
		l.reject(lit.Pos(), "GIR lowering", "mixed keyed and unkeyed struct literals are not lowered by GO-006")
		return girValue{}, false
	}
	for index, seen := range fieldSeen {
		if !seen {
			l.reject(lit.Pos(), "GIR lowering", fmt.Sprintf("struct literal omits field %q; implicit zero values are not lowered by GO-006", structType.Field(index).Name()))
			return girValue{}, false
		}
	}

	id := l.nextTemp()
	l.current.Instructions = append(l.current.Instructions, girInstruction{
		ID:     id,
		Kind:   "MakeStruct",
		Type:   typ,
		Fields: fields,
	})
	return girValue{Var: id}, true
}

func (l *girFunctionLowerer) lowerArrayLiteral(lit *ast.CompositeLit, typ girType, arrayType *types.Array) (girValue, bool) {
	if int64(len(lit.Elts)) != arrayType.Len() {
		l.reject(lit.Pos(), "GIR lowering", "array literals must provide every fixed-array element in GO-006")
		return girValue{}, false
	}

	elements := make([]girValue, 0, len(lit.Elts))
	for _, element := range lit.Elts {
		if _, ok := element.(*ast.KeyValueExpr); ok {
			l.reject(element.Pos(), "GIR lowering", "keyed array literals are not lowered by GO-006")
			return girValue{}, false
		}
		value, ok := l.lowerExpr(element)
		if !ok {
			return girValue{}, false
		}
		elements = append(elements, value)
	}

	id := l.nextTemp()
	l.current.Instructions = append(l.current.Instructions, girInstruction{
		ID:       id,
		Kind:     "MakeArray",
		Type:     typ,
		Elements: elements,
	})
	return girValue{Var: id}, true
}

func (l *girFunctionLowerer) lowerSelectorExpr(selector *ast.SelectorExpr) (girValue, bool) {
	selection := l.packageLowerer.pkg.TypesInfo.Selections[selector]
	if selection == nil || selection.Kind() != types.FieldVal {
		l.reject(selector.Pos(), "GIR lowering", "only direct struct field reads are lowered by GO-006")
		return girValue{}, false
	}
	if len(selection.Index()) != 1 {
		l.reject(selector.Pos(), "GIR lowering", "promoted or embedded field reads are not lowered by GO-006")
		return girValue{}, false
	}

	base, ok := l.lowerExpr(selector.X)
	if !ok {
		return girValue{}, false
	}
	typ, ok := l.girTypeOf(selector)
	if !ok {
		return girValue{}, false
	}
	id := l.nextTemp()
	l.current.Instructions = append(l.current.Instructions, girInstruction{
		ID:    id,
		Kind:  "Field",
		Type:  typ,
		Base:  &base,
		Field: selector.Sel.Name,
	})
	return girValue{Var: id}, true
}

func (l *girFunctionLowerer) lowerIndexExpr(index *ast.IndexExpr) (girValue, bool) {
	indexedType := l.packageLowerer.pkg.TypesInfo.TypeOf(index.X)
	if indexedType == nil {
		l.reject(index.Pos(), "GIR lowering", "indexed expression has no type information")
		return girValue{}, false
	}
	if _, ok := indexedType.Underlying().(*types.Array); !ok {
		l.reject(index.Pos(), "GIR lowering", "only fixed-array indexing is lowered by GO-006")
		return girValue{}, false
	}

	base, ok := l.lowerExpr(index.X)
	if !ok {
		return girValue{}, false
	}
	indexValue, ok := l.lowerIndexValue(index.Index)
	if !ok {
		return girValue{}, false
	}
	typ, ok := l.girTypeOf(index)
	if !ok {
		return girValue{}, false
	}
	id := l.nextTemp()
	l.current.Instructions = append(l.current.Instructions, girInstruction{
		ID:    id,
		Kind:  "Index",
		Type:  typ,
		Base:  &base,
		Index: &indexValue,
	})
	return girValue{Var: id}, true
}

func (l *girFunctionLowerer) lowerIndexValue(expr ast.Expr) (girValue, bool) {
	switch expr := expr.(type) {
	case *ast.BasicLit:
		if expr.Kind == token.INT {
			return l.lowerIndexIntegerLiteral(expr)
		}
	case *ast.Ident:
		if obj := l.packageLowerer.pkg.TypesInfo.Uses[expr]; obj != nil {
			if constantValue, ok := obj.(*types.Const); ok {
				return l.lowerIndexIntegerConstant(expr.Pos(), constantValue)
			}
		}
	}
	return l.lowerExpr(expr)
}

func (l *girFunctionLowerer) lowerIndexIntegerLiteral(lit *ast.BasicLit) (girValue, bool) {
	typ := l.packageLowerer.pkg.TypesInfo.TypeOf(lit)
	if typ == nil {
		l.reject(lit.Pos(), "GIR lowering", "integer literal has no type information")
		return girValue{}, false
	}
	girType, ok := l.indexIntegerType(lit.Pos(), typ)
	if !ok {
		return girValue{}, false
	}
	value, err := strconv.ParseInt(lit.Value, 0, 64)
	if err != nil {
		l.reject(lit.Pos(), "GIR lowering", fmt.Sprintf("integer literal %q cannot be parsed: %v", lit.Value, err))
		return girValue{}, false
	}
	return girValue{Int: &girIntLiteral{
		Value:  strconv.FormatInt(value, 10),
		Width:  girType.Width,
		Signed: girType.Signed != nil && *girType.Signed,
	}}, true
}

func (l *girFunctionLowerer) lowerIndexIntegerConstant(pos token.Pos, constantValue *types.Const) (girValue, bool) {
	girType, ok := l.indexIntegerType(pos, constantValue.Type())
	if !ok {
		return girValue{}, false
	}
	value, exact := constant.Int64Val(constantValue.Val())
	if !exact {
		l.reject(pos, "GIR lowering", "array index constant cannot be represented as int64")
		return girValue{}, false
	}
	return girValue{Int: &girIntLiteral{
		Value:  strconv.FormatInt(value, 10),
		Width:  girType.Width,
		Signed: girType.Signed != nil && *girType.Signed,
	}}, true
}

func (l *girFunctionLowerer) indexIntegerType(pos token.Pos, typ types.Type) (girType, bool) {
	if girType, ok := girTypeFromGoType(typ); ok && girType.Kind == "bv" {
		return girType, true
	}
	if isUntypedInteger(typ) || isIntegerLiteralDefaultInt(typ) {
		return girType{Kind: "bv", Width: 64, Signed: boolPtr(true)}, true
	}
	l.reject(pos, "GIR lowering", "array index must have a fixed-width integer or integer literal type")
	return girType{}, false
}

func girNamedType(named *types.Named) (girType, bool) {
	underlying := named.Underlying()
	switch underlying := underlying.(type) {
	case *types.Struct:
		return girStructType(namedTypeName(named), underlying)
	case *types.Array:
		arrayType, ok := girArrayType(underlying)
		if ok {
			arrayType.Name = namedTypeName(named)
		}
		return arrayType, ok
	default:
		return girTypeFromGoType(underlying)
	}
}

func girStructType(name string, structType *types.Struct) (girType, bool) {
	fields := make([]girFieldType, 0, structType.NumFields())
	for i := 0; i < structType.NumFields(); i++ {
		field := structType.Field(i)
		if field.Embedded() {
			return girType{}, false
		}
		fieldType, ok := girTypeFromGoType(field.Type())
		if !ok {
			return girType{}, false
		}
		fields = append(fields, girFieldType{
			Name: field.Name(),
			Type: fieldType,
		})
	}
	return girType{Kind: "struct", Name: name, Fields: fields}, true
}

func girArrayType(arrayType *types.Array) (girType, bool) {
	elementType, ok := girTypeFromGoType(arrayType.Elem())
	if !ok {
		return girType{}, false
	}
	return girType{
		Kind:    "array",
		Length:  arrayType.Len(),
		Element: &elementType,
	}, true
}

func structLiteralFieldName(expr ast.Expr) (string, bool) {
	switch expr := expr.(type) {
	case *ast.Ident:
		return expr.Name, true
	default:
		return "", false
	}
}

func structFieldIndex(structType *types.Struct, name string) int {
	for i := 0; i < structType.NumFields(); i++ {
		if structType.Field(i).Name() == name {
			return i
		}
	}
	return -1
}

func namedTypeName(named *types.Named) string {
	obj := named.Obj()
	if obj == nil {
		return ""
	}
	if obj.Pkg() == nil {
		return obj.Name()
	}
	return obj.Pkg().Path() + "." + obj.Name()
}

func isUntypedInteger(typ types.Type) bool {
	basic, ok := typ.Underlying().(*types.Basic)
	if !ok {
		return false
	}
	return basic.Kind() == types.UntypedInt || basic.Kind() == types.UntypedRune
}

func isIntegerLiteralDefaultInt(typ types.Type) bool {
	basic, ok := typ.Underlying().(*types.Basic)
	if !ok {
		return false
	}
	return basic.Kind() == types.Int
}

func typeString(typ types.Type) string {
	if typ == nil {
		return "<unknown>"
	}
	return typ.String()
}
