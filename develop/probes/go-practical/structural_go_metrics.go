// Private parser/type-size measurements. The observed application never runs.
package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"go/ast"
	"go/parser"
	"go/token"
	"go/types"
	"os"
)

type metricInput struct {
	ID       string `json:"id"`
	Source   string `json:"source"`
	Language string `json:"language"`
}

func fields(list *ast.FieldList) int {
	if list == nil {
		return 0
	}
	n := 0
	for _, field := range list.List {
		if len(field.Names) == 0 {
			n++
		} else {
			n += len(field.Names)
		}
	}
	return n
}

func main() {
	var input []metricInput
	if err := json.NewDecoder(os.Stdin).Decode(&input); err != nil {
		panic(err)
	}
	output := []map[string]any{}
	for _, in := range input {
		digest := sha256.Sum256([]byte(in.Source))
		row := map[string]any{"id": in.ID, "source_sha256": hex.EncodeToString(digest[:]), "source_bytes": len(in.Source), "language": in.Language, "gc_target": "amd64"}
		fset := token.NewFileSet()
		file, err := parser.ParseFile(fset, "fixture.go", in.Source, parser.AllErrors|parser.ParseComments)
		if err != nil {
			row["kind"], row["error"] = "source_error", err.Error()
			output = append(output, row)
			continue
		}
		nodes, depth, maxDepth, functions, typeDecls, constBindings, varBindings := 0, 0, 0, 0, 0, 0, 0
		maxParams, maxResults, maxFields, loops := 0, 0, 0, 0
		ast.Inspect(file, func(n ast.Node) bool {
			if n == nil {
				depth--
				return true
			}
			nodes++
			depth++
			maxDepth = max(maxDepth, depth)
			switch n := n.(type) {
			case *ast.FuncDecl:
				functions++
				maxParams = max(maxParams, fields(n.Recv)+fields(n.Type.Params))
				maxResults = max(maxResults, fields(n.Type.Results))
			case *ast.TypeSpec:
				typeDecls++
			case *ast.GenDecl:
				for _, s := range n.Specs {
					if v, ok := s.(*ast.ValueSpec); ok {
						if n.Tok == token.CONST {
							constBindings += len(v.Names)
						} else if n.Tok == token.VAR {
							varBindings += len(v.Names)
						}
					}
				}
			case *ast.StructType:
				maxFields = max(maxFields, fields(n.Fields))
			case *ast.ForStmt, *ast.RangeStmt:
				loops++
			}
			return true
		})
		row["ast_nodes"], row["ast_depth"], row["functions"], row["type_decls"] = nodes, maxDepth, functions, typeDecls
		row["const_bindings"], row["var_bindings"], row["max_params"], row["max_results"], row["max_struct_fields"], row["loops"] = constBindings, varBindings, maxParams, maxResults, maxFields, loops
		info := &types.Info{Defs: map[*ast.Ident]types.Object{}}
		conf := types.Config{GoVersion: in.Language, Sizes: types.SizesFor("gc", "amd64")}
		pkg, err := conf.Check("fixture/policy", fset, []*ast.File{file}, info)
		if err != nil {
			row["kind"], row["error"] = "type_error", err.Error()
		} else {
			row["kind"] = "typed_original_package"
			if obj := pkg.Scope().Lookup("BudgetValue"); obj != nil {
				row["budget_value_native_sizeof"] = conf.Sizes.Sizeof(obj.Type())
			}
		}
		output = append(output, row)
	}
	if err := json.NewEncoder(os.Stdout).Encode(output); err != nil {
		panic(err)
	}
}
