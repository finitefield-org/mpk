package main

import (
	"fmt"
	"go/constant"
	"go/types"
	"math/big"
	"strconv"
)

func virTypeFromGo(typ types.Type) (virType, bool) {
	return virTypeFromGoDepth(typ, 0)
}

func virTypeFromGoDepth(typ types.Type, depth int) (virType, bool) {
	if typ == nil || depth > 16 {
		return virType{}, false
	}
	if named, ok := typ.(*types.Named); ok {
		object := named.Obj()
		if _, ok := named.Underlying().(*types.Struct); ok && object != nil && object.Pkg() != nil {
			return virType{Kind: "struct", ID: object.Pkg().Path() + "." + object.Name()}, true
		}
		// Named primitives and arrays are intentionally outside the profile.
		return virType{}, false
	}
	if basic, ok := typ.Underlying().(*types.Basic); ok {
		if basic.Kind() == types.Bool || basic.Kind() == types.UntypedBool {
			return virType{Kind: "bool"}, true
		}
		if width, signed, ok := fixedInteger(typ); ok {
			return virType{Kind: "bv", Width: width, Signed: boolPointer(signed)}, true
		}
		return virType{}, false
	}
	if array, ok := typ.Underlying().(*types.Array); ok {
		if array.Len() < 0 || array.Len() > 256 {
			return virType{}, false
		}
		element, ok := virTypeFromGoDepth(array.Elem(), depth+1)
		if !ok {
			return virType{}, false
		}
		return virType{Kind: "array", Length: array.Len(), Element: &element}, true
	}
	return virType{}, false
}

func virStructDecl(named *types.Named) (virTypeDecl, bool) {
	object := named.Obj()
	structure, ok := named.Underlying().(*types.Struct)
	if !ok || object == nil || object.Pkg() == nil || !validASCIIIdentifier(object.Name()) || structure.NumFields() > 64 {
		return virTypeDecl{}, false
	}
	fields := make([]virField, 0, structure.NumFields())
	for index := 0; index < structure.NumFields(); index++ {
		field := structure.Field(index)
		if field.Embedded() || !validASCIIIdentifier(field.Name()) {
			return virTypeDecl{}, false
		}
		fieldType, ok := virTypeFromGoDepth(field.Type(), 1)
		if !ok {
			return virTypeDecl{}, false
		}
		fields = append(fields, virField{Name: field.Name(), Type: fieldType})
	}
	return virTypeDecl{
		ID:   object.Pkg().Path() + "." + object.Name(),
		Name: object.Name(), Fields: fields,
	}, true
}

func fixedInteger(typ types.Type) (int64, bool, bool) {
	if typ == nil {
		return 0, false, false
	}
	basic, ok := typ.Underlying().(*types.Basic)
	if !ok {
		return 0, false, false
	}
	switch basic.Kind() {
	case types.Int8:
		return 8, true, true
	case types.Int16:
		return 16, true, true
	case types.Int32:
		return 32, true, true
	case types.Int64:
		return 64, true, true
	case types.Uint8:
		return 8, false, true
	case types.Uint16:
		return 16, false, true
	case types.Uint32:
		return 32, false, true
	case types.Uint64:
		return 64, false, true
	default:
		return 0, false, false
	}
}

func integerFromConstant(value constant.Value, typ virType) (virInteger, error) {
	if typ.Kind != "bv" || typ.Signed == nil {
		return virInteger{}, fmt.Errorf("integer constant lacks a fixed-width type")
	}
	text := value.ExactString()
	integer, ok := new(big.Int).SetString(text, 10)
	if !ok {
		return virInteger{}, fmt.Errorf("integer constant is not representable")
	}
	if !integerFits(integer, typ.Width, *typ.Signed) {
		return virInteger{}, fmt.Errorf("integer constant does not fit its fixed-width type")
	}
	return virInteger{Value: integer.String(), Width: typ.Width, Signed: *typ.Signed}, nil
}

func integerFits(value *big.Int, width int64, signed bool) bool {
	if width != 8 && width != 16 && width != 32 && width != 64 {
		return false
	}
	if signed {
		minimum := new(big.Int).Lsh(big.NewInt(1), uint(width-1))
		minimum.Neg(minimum)
		maximum := new(big.Int).Lsh(big.NewInt(1), uint(width-1))
		maximum.Sub(maximum, big.NewInt(1))
		return value.Cmp(minimum) >= 0 && value.Cmp(maximum) <= 0
	}
	maximum := new(big.Int).Lsh(big.NewInt(1), uint(width))
	maximum.Sub(maximum, big.NewInt(1))
	return value.Sign() >= 0 && value.Cmp(maximum) <= 0
}

func parseContractInteger(text string, width int64, signed bool) (virInteger, error) {
	value, ok := new(big.Int).SetString(text, 0)
	if !ok || !integerFits(value, width, signed) {
		return virInteger{}, fmt.Errorf("contract integer does not fit its declared type")
	}
	return virInteger{Value: value.String(), Width: width, Signed: signed}, nil
}

func zeroLiteral(typ virType) (virValue, bool) {
	switch typ.Kind {
	case "bool":
		value := false
		return virValue{Bool: &value}, true
	case "bv":
		if typ.Signed == nil {
			return virValue{}, false
		}
		return virValue{Int: &virInteger{Value: strconv.FormatInt(0, 10), Width: typ.Width, Signed: *typ.Signed}}, true
	default:
		return virValue{}, false
	}
}

func typeEqual(left, right virType) bool {
	if left.Kind != right.Kind || left.Width != right.Width || left.Length != right.Length || left.ID != right.ID {
		return false
	}
	if (left.Signed == nil) != (right.Signed == nil) || left.Signed != nil && *left.Signed != *right.Signed {
		return false
	}
	if (left.Element == nil) != (right.Element == nil) {
		return false
	}
	return left.Element == nil || typeEqual(*left.Element, *right.Element)
}
