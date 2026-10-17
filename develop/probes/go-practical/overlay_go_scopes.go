// Private source-scope observer. It never executes the observed application.
package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"go/types"
	"os"
	"sort"
)

type request struct {
	Path      string   `json:"path"`
	Source    string   `json:"source"`
	Language  string   `json:"language"`
	Functions []string `json:"functions"`
}
type fieldFact struct {
	Key  string `json:"key"`
	Name string `json:"name"`
	Type string `json:"type"`
}
type typeFact struct {
	ID              string      `json:"id"`
	Kind            string      `json:"kind"`
	Underlying      string      `json:"underlying,omitempty"`
	Width           int64       `json:"width,omitempty"`
	Signed          bool        `json:"signed,omitempty"`
	Element         string      `json:"element,omitempty"`
	Length          int64       `json:"length,omitempty"`
	Fields          []fieldFact `json:"fields,omitempty"`
	ImplementsError bool        `json:"implements_error,omitempty"`
}
type bindingFact struct {
	ID   string `json:"id"`
	Name string `json:"name"`
	Type string `json:"type"`
}
type loopFact struct {
	Kind     string        `json:"kind"`
	Start    int           `json:"start_byte"`
	End      int           `json:"end_byte"`
	Ordinal  int           `json:"ordinal"`
	Bindings []bindingFact `json:"bindings"`
}
type functionFact struct {
	Name     string        `json:"name"`
	Receiver string        `json:"receiver,omitempty"`
	Start    int           `json:"start_byte"`
	End      int           `json:"end_byte"`
	Inputs   []bindingFact `json:"inputs"`
	Results  []bindingFact `json:"results"`
	Loops    []loopFact    `json:"loops"`
}

func main() {
	var in request
	if err := json.NewDecoder(os.Stdin).Decode(&in); err != nil {
		panic(err)
	}
	fset := token.NewFileSet()
	file, err := parser.ParseFile(fset, in.Path, in.Source, 0)
	if err != nil {
		panic(err)
	}
	info := &types.Info{Defs: map[*ast.Ident]types.Object{}, Uses: map[*ast.Ident]types.Object{}, Scopes: map[ast.Node]*types.Scope{}}
	conf := types.Config{GoVersion: in.Language, Sizes: types.SizesFor("gc", "amd64")}
	if _, err = conf.Check("fixture/policy", fset, []*ast.File{file}, info); err != nil {
		panic(err)
	}
	key := func(t types.Type) string {
		return types.TypeString(types.Unalias(t), func(p *types.Package) string { return p.Path() })
	}
	typesSeen := map[string]typeFact{}
	var recordType func(types.Type) string
	recordType = func(raw types.Type) string {
		t := types.Unalias(raw)
		id := key(t)
		if _, ok := typesSeen[id]; ok {
			return id
		}
		fact := typeFact{ID: id, Kind: "unsupported"}
		typesSeen[id] = fact // terminate on source-recursive types; profile review is separate
		switch t := t.(type) {
		case *types.Named:
			if t == types.Universe.Lookup("error").Type() {
				fact.Kind = "error"
			} else {
				fact.Kind, fact.Underlying = "named", recordType(t.Underlying())
				fact.ImplementsError = types.Implements(t, types.Universe.Lookup("error").Type().Underlying().(*types.Interface))
			}
		case *types.Basic:
			if t.Info()&types.IsBoolean != 0 {
				fact.Kind = "bool"
			} else if t.Info()&types.IsString != 0 {
				fact.Kind = "string"
			} else if t.Info()&types.IsInteger != 0 && t.Kind() != types.Uintptr && t.Info()&types.IsUntyped == 0 {
				fact.Kind, fact.Width, fact.Signed = "word", conf.Sizes.Sizeof(t)*8, t.Info()&types.IsUnsigned == 0
			}
		case *types.Array:
			fact.Kind, fact.Element, fact.Length = "array", recordType(t.Elem()), t.Len()
		case *types.Slice:
			fact.Kind, fact.Element = "slice", recordType(t.Elem())
		case *types.Struct:
			fact.Kind, fact.Fields = "struct", []fieldFact{}
			for i := 0; i < t.NumFields(); i++ {
				f := t.Field(i)
				if f.Embedded() {
					fact.Kind = "unsupported"
				}
				fact.Fields = append(fact.Fields, fieldFact{fmt.Sprintf("field:%d", i), f.Name(), recordType(f.Type())})
			}
		}
		typesSeen[id] = fact
		return id
	}
	// Contracts may use admitted predeclared types absent from source signatures.
	// Their meanings come from the pinned universe/target, not caller records.
	for _, k := range []types.BasicKind{types.Bool, types.Int, types.Int8, types.Int16, types.Int32, types.Int64, types.Uint, types.Uint8, types.Uint16, types.Uint32, types.Uint64, types.String} {
		recordType(types.Typ[k])
	}
	recordType(types.Universe.Lookup("error").Type())
	for _, decl := range file.Decls {
		if d, ok := decl.(*ast.GenDecl); ok {
			for _, spec := range d.Specs {
				if s, ok := spec.(*ast.TypeSpec); ok {
					recordType(info.Defs[s.Name].Type())
				}
			}
		}
	}
	binding := func(v *types.Var, slot int) bindingFact {
		id := fmt.Sprintf("binding:%d", fset.PositionFor(v.Pos(), false).Offset)
		if !v.Pos().IsValid() {
			id = fmt.Sprintf("unnamed-slot:%d", slot)
		}
		return bindingFact{id, v.Name(), recordType(v.Type())}
	}
	wanted := map[string]bool{}
	for _, n := range in.Functions {
		wanted[n] = true
	}
	functions := []functionFact{}
	for _, decl := range file.Decls {
		fn, ok := decl.(*ast.FuncDecl)
		if !ok || !wanted[fn.Name.Name] {
			continue
		}
		sig := info.Defs[fn.Name].Type().(*types.Signature)
		out := functionFact{Name: fn.Name.Name, Start: fset.PositionFor(fn.Pos(), false).Offset, End: fset.PositionFor(fn.End(), false).Offset, Inputs: []bindingFact{}, Results: []bindingFact{}, Loops: []loopFact{}}
		if sig.Recv() != nil {
			out.Receiver = recordType(sig.Recv().Type())
			out.Inputs = append(out.Inputs, binding(sig.Recv(), 0))
		}
		for i := 0; i < sig.Params().Len(); i++ {
			out.Inputs = append(out.Inputs, binding(sig.Params().At(i), i))
		}
		for i := 0; i < sig.Results().Len(); i++ {
			out.Results = append(out.Results, binding(sig.Results().At(i), i))
		}
		ast.Inspect(fn.Body, func(node ast.Node) bool {
			if _, literal := node.(*ast.FuncLit); literal {
				return false
			}
			var body *ast.BlockStmt
			kind := ""
			excluded := map[types.Object]bool{}
			switch n := node.(type) {
			case *ast.ForStmt:
				body, kind = n.Body, "for"
			case *ast.RangeStmt:
				body, kind = n.Body, "range"
				if n.Tok == token.DEFINE {
					for _, e := range []ast.Expr{n.Key, n.Value} {
						if id, ok := e.(*ast.Ident); ok {
							excluded[info.Defs[id]] = true
						}
					}
				}
			default:
				return true
			}
			scope, position := info.Scopes[node], body.Lbrace-1
			fact := loopFact{Kind: kind, Start: fset.PositionFor(node.Pos(), false).Offset, End: fset.PositionFor(node.End(), false).Offset, Ordinal: len(out.Loops), Bindings: []bindingFact{}}
			names := map[string]bool{}
			for s := scope; s != nil; s = s.Parent() {
				for _, n := range s.Names() {
					names[n] = true
				}
			}
			ordered := []string{}
			for n := range names {
				ordered = append(ordered, n)
			}
			sort.Strings(ordered)
			for _, n := range ordered {
				_, obj := scope.LookupParent(n, position)
				if v, ok := obj.(*types.Var); ok && n != "_" && !excluded[obj] && (v.Pkg() == nil || v.Parent() != v.Pkg().Scope()) {
					fact.Bindings = append(fact.Bindings, binding(v, 0))
				}
			}
			out.Loops = append(out.Loops, fact)
			return true
		})
		functions = append(functions, out)
	}
	typeKeys, typeList := []string{}, []typeFact{}
	for id := range typesSeen {
		typeKeys = append(typeKeys, id)
	}
	sort.Strings(typeKeys)
	for _, id := range typeKeys {
		typeList = append(typeList, typesSeen[id])
	}
	h := sha256.Sum256([]byte(in.Source))
	aliases := map[string]string{"byte": "uint8", "rune": "int32"}
	for id, obj := range info.Defs {
		if n, ok := obj.(*types.TypeName); ok && n.IsAlias() {
			aliases["fixture/policy."+id.Name] = key(n.Type())
		}
	}
	if err := json.NewEncoder(os.Stdout).Encode(map[string]any{"path": in.Path, "source_sha256": hex.EncodeToString(h[:]), "language": in.Language, "functions": functions, "types": typeList, "aliases": aliases}); err != nil {
		panic(err)
	}
}
