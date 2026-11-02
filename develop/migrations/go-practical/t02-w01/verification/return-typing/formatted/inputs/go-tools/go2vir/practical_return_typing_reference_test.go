//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"go/types"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"testing"
)

type practicalReturnTypingCase struct {
	label, results, expr, setup, tail, receiver, params, extra string
	nesting                                                    int
	outside, helper                                            bool
}

func practicalReturnTypingCases() []practicalReturnTypingCase {
	cases := []practicalReturnTypingCase{}
	add := func(results, expr string) {
		cases = append(cases, practicalReturnTypingCase{label: results + "/" + expr, results: results, expr: expr})
	}
	for _, result := range []string{"Word", "uint16", "uint64", "int8", "Flag", "bool", "Text", "string", "[2]Word", "[]Word", "Record", "error"} {
		for _, expr := range []string{"x", "v", "u", "Other(1)", "1", "-1", "255", "256", "65535", "65536", "(1<<64)-1", "1<<n", "(1<<n)+65536", "b", "c", "x==1", "true", "text", "str", "\"a\"", "a", "[2]Word{1}", "s", "[]Word{1}", "rec", "Record{}", "Code(1)", "Good{}", "nil", "Id(x)", "Pair(x)"} {
			add(result, expr)
		}
	}
	for _, result := range []string{"Word,uint64", "uint64,Word", "uint16,uint64", "Word,error", "Record,error", "error,error", "r Word,e error", "r Word,t uint64", "_ Word,_ error"} {
		for _, expr := range []string{"Pair(x)", "(Pair(x))", "Mixed(x)", "RecordError()", "Opaque(x)", "x,u", "x,nil", "x,Code(1)", "1<<n,Code(1)", "1<<n,1<<n", "x,c", "x", "x,u,1", "Pair(x),u", "x,Pair(x)"} {
			add(result, expr)
		}
	}
	add("r,t Word", "")
	add("r,t Word", "x,x")
	for _, expr := range []string{"", "Void()", "1", "nil", "Pair(x)"} {
		add("", expr)
	}
	for _, result := range []string{"r Word", "r Word,e error", "_ Word", "_ Word,_ error", "_ error", "Word", "Word,error", "r []Word", "r Record", "r Flag", "r Text"} {
		add(result, "")
	}
	for _, entity := range []string{"var r string;_=r;", "const r=1;_=r;", "type r uint16;var _ r;", "r:=x;_=r;", "r,e:=Mixed(x);_,_=r,e;"} {
		for _, expr := range []string{"", "x,nil"} {
			cases = append(cases, practicalReturnTypingCase{label: "shadow/" + entity + expr, results: "r Word,e error", setup: "{" + entity, expr: expr, tail: "};return"})
		}
	}
	for _, setup := range []string{"if r:=x;true{_=r;", "for _,r:=range s{_=r;", "switch{default:var r Word;_=r;", "if e:=Code(1);true{_=e;"} {
		for _, expr := range []string{"", "x,nil"} {
			cases = append(cases, practicalReturnTypingCase{label: "control-shadow/" + setup + expr, results: "r Word,e error", setup: setup, expr: expr, tail: "};return"})
		}
	}
	for _, decl := range []string{"var r Word;_=r", "const r=1;_=r", "type r uint16;var _ r"} {
		cases = append(cases, practicalReturnTypingCase{label: "not-yet-visible/" + decl, results: "r Word", setup: "{", tail: ";" + decl + "};return"})
	}
	cases = append(cases, practicalReturnTypingCase{label: "prior-assignment", results: "r Word,e error", setup: "r=1;e=Code(1);"})
	cases = append(cases, practicalReturnTypingCase{label: "closed-inner-shadow", results: "r Word", setup: "{var r string;_=r};"})
	for _, expr := range []string{"", "r", "x", "nil", "x,nil"} {
		for nesting := 1; nesting <= 2; nesting++ {
			cases = append(cases, practicalReturnTypingCase{label: fmt.Sprintf("nested%d/%s", nesting, expr), results: "r Word", expr: expr, nesting: nesting})
		}
	}
	for nesting := 1; nesting <= 2; nesting++ {
		cases = append(cases, practicalReturnTypingCase{label: fmt.Sprintf("nested-forward%d", nesting), results: "r Word,e error", expr: "Mixed(x)", nesting: nesting})
	}
	for _, receiver := range []string{"self Code", "Code", "self Alias"} {
		cases = append(cases, practicalReturnTypingCase{label: "method/" + receiver, results: "r Word,e error", receiver: receiver})
	}
	cases = append(cases, practicalReturnTypingCase{label: "receiver-expression", results: "Code", receiver: "self Code", expr: "self"})
	for _, expr := range []string{"h.Id(v)", "h.Pair(v)", "h.Record{}", "h.Error(1)", "nil"} {
		for _, result := range []string{"uint16", "uint16,uint64", "h.Record", "error"} {
			cases = append(cases, practicalReturnTypingCase{label: "foreign/" + result + "/" + expr, results: result, expr: expr, helper: true})
		}
	}
	for arity := 0; arity <= 17; arity++ {
		names := []string{}
		values := []string{}
		unnamed := []string{}
		for n := 0; n < arity; n++ {
			names = append(names, fmt.Sprintf("r%d Word", n))
			values = append(values, "1")
			unnamed = append(unnamed, "Word")
		}
		cases = append(cases, practicalReturnTypingCase{label: fmt.Sprintf("named%d", arity), results: strings.Join(names, ","), outside: arity > 16})
		cases = append(cases, practicalReturnTypingCase{label: fmt.Sprintf("explicit%d", arity), results: strings.Join(unnamed, ","), expr: strings.Join(values, ","), outside: arity > 16})
	}
	results := []string{}
	for n := 0; n < 16; n++ {
		results = append(results, "Word")
	}
	add(strings.Join(results, ","), "Many16()")
	params := []string{}
	for n := 0; n < 70; n++ {
		params = append(params, fmt.Sprintf("p%d Word", n))
	}
	cases = append(cases, practicalReturnTypingCase{label: "seventy-original-parameters", results: "Word", expr: "p69", params: strings.Join(params, ",")})
	for _, c := range []practicalReturnTypingCase{
		{label: "float-result", results: "float64", expr: "1.0"},
		{label: "integer-float-constant", results: "Word", expr: "1.0"},
		{label: "pointer-result", results: "*Word", expr: "nil"},
		{label: "map-result", results: "map[int]Word", expr: "nil"},
		{label: "interface-result", results: "any", expr: "x"},
		{label: "pointer-receiver", results: "Word", receiver: "self *Code", expr: "x"},
		{label: "unused-pointer-input", results: "Word", params: "x Word,p *Word", expr: "x"},
		{label: "variadic-input", results: "Word", params: "x Word,p ...Word", expr: "x"},
	} {
		c.outside = true
		cases = append(cases, c)
	}
	return cases
}
func TestPracticalOriginalReturnTypingReference(t *testing.T) {
	if runtime.Version() != practicalCompilerRelease {
		t.Fatal(runtime.Version())
	}
	out := os.Getenv("MPK_GO_RETURN_TYPING_NATIVE_OUTPUT")
	if out == "" {
		out = t.TempDir()
	}
	write := func(path string, value any) {
		raw, err := json.MarshalIndent(value, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		f, err := os.OpenFile(path, os.O_CREATE|os.O_EXCL|os.O_WRONLY, 0o600)
		if err != nil {
			t.Fatal(err)
		}
		if _, err = f.Write(append(raw, '\n')); err != nil {
			t.Fatal(err)
		}
		if err = f.Close(); err != nil {
			t.Fatal(err)
		}
	}
	accepted, rejected := 0, 0
	for index, c := range practicalReturnTypingCases() {
		dir := filepath.Join(out, fmt.Sprintf("case-%05d", index))
		if err := os.Mkdir(dir, 0o700); err != nil {
			t.Fatal(err)
		}
		source := "package policy\n"
		helper := ""
		if c.helper {
			source += "import h \"example.com/scalar/helper\"\n"
			helper = "package foreign\ntype Record struct{N uint16}\ntype Error uint16\nfunc(Error)Error()string{return \"foreign\"}\nfunc Id(x uint16)uint16{return x}\nfunc Pair(x uint16)(uint16,uint64){return x,0}\n"
		}
		source += `type Word uint16
type Alias=Code
type Other uint16
type Flag bool
type Text string
type Code uint16
func(Code)Error()string{return "code"}
type Good struct{}
func(Good)Error()string{return "good"}
type Record struct{W Word;A [2]Word;S []Word}
func Id(x Word)Word{return x}
func Pair(x Word)(Word,uint64){return x,0}
func Mixed(x Word)(Word,error){return x,nil}
func RecordError()(Record,error){return Record{},nil}
func Opaque(x Word)(Word,error)
func Void(){}
`
		rs := []string{}
		for n := 0; n < 16; n++ {
			rs = append(rs, fmt.Sprintf("r%d Word", n))
		}
		source += "func Many16()(" + strings.Join(rs, ",") + "){return}\n" + c.extra
		params := c.params
		if params == "" {
			params = "x Word,v uint16,u uint64,n uint64,b Flag,c bool,text Text,str string,a [2]Word,s []Word,rec Record,err error"
		}
		result := ""
		if c.results != "" {
			result = "(" + c.results + ")"
		}
		receiver := ""
		if c.receiver != "" {
			receiver = "(" + c.receiver + ")"
		}
		if c.nesting == 0 {
			source += "func " + receiver + "Probe(" + params + ")" + result + "{\n"
		} else {
			source += "func Probe(" + params + ")string{_ = "
			if c.nesting == 2 {
				source += "func()(r bool){_ = "
			}
			source += "func()" + result + "{\n"
		}
		source += c.setup
		start := len(source)
		source += "return"
		if c.expr != "" {
			source += " " + c.expr
		}
		end := len(source)
		source += c.tail + "\n}"
		if c.nesting == 2 {
			source += ";return false}"
		}
		if c.nesting > 0 {
			source += ";return \"outer\"}"
		}
		source += "\n"
		cap := fmt.Sprintf("return-typing/case-%05d", index)
		mod, err := newPracticalSourceBuffer(cap, "go.mod", "go_mod", []byte("module example.com/scalar\ngo 1.25.0\n"))
		if err != nil {
			t.Fatal(err)
		}
		root, err := newPracticalSourceBuffer(cap, "policy.go", "go_file", []byte(source))
		if err != nil {
			t.Fatal(err)
		}
		buffers := []practicalSourceBuffer{mod, root}
		names := map[string]string{"example.com/scalar": "policy"}
		fileIDs := map[string]string{"example.com/scalar": root.object.ID}
		if c.helper {
			h, err := newPracticalSourceBuffer(cap, "helper/helper.go", "go_file", []byte(helper))
			if err != nil {
				t.Fatal(err)
			}
			buffers = append(buffers, h)
			names["example.com/scalar/helper"] = "foreign"
			fileIDs["example.com/scalar/helper"] = h.object.ID
		}
		pkgs := map[string]map[string]any{}
		pkgIDs := map[string]string{}
		pkgRoots := []string{}
		for path, name := range names {
			p, err := practicalTypeKeySeal(practicalTypePackageDomain, map[string]any{"schema": practicalTypePackageSchema, "module_source_object_id": mod.object.ID, "import_path": path, "package_name": name, "source_objects": []string{fileIDs[path]}})
			if err != nil {
				t.Fatal(err)
			}
			id := p["id"].(string)
			pkgs[id] = p
			pkgIDs[path] = id
			pkgRoots = append(pkgRoots, id)
		}
		sort.Strings(pkgRoots)
		scalar, err := practicalTypeKeySeal(practicalSourceTypeDomain, map[string]any{"schema": practicalSourceTypeSchema, "kind": "builtin", "name": "bool", "width": nil, "signed": nil})
		if err != nil {
			t.Fatal(err)
		}
		objects := []practicalSourceObject{}
		originals := []map[string]any{}
		for _, b := range buffers {
			objects = append(objects, b.object)
			originals = append(originals, map[string]any{"object": b.object, "raw_hex": hex.EncodeToString(b.data)})
		}
		sort.Slice(objects, func(i, j int) bool { return objects[i].ID < objects[j].ID })
		components := map[string]any{"schema": practicalTypeComponentsSchema, "source_objects": objects, "packages": practicalTypeKeyRows(pkgs), "declarations": []any{}, "types": []any{scalar}, "underlying": []any{}, "aliases": []any{}, "package_roots": pkgRoots, "type_roots": []string{scalar["id"].(string)}}
		hash, err := hashTypedCanonicalJSON(practicalTypeComponentsDomain, components)
		if err != nil {
			t.Fatal(err)
		}
		components["content_sha256"] = hash
		canonical, err := canonicalJSON(components)
		if err != nil {
			t.Fatal(err)
		}

		write(filepath.Join(dir, "input.json"), map[string]any{"index": index, "label": c.label, "components_hex": hex.EncodeToString(canonical), "sources": originals, "query_source_object_id": root.object.ID, "statement_start_byte": start, "statement_end_byte": end, "outside_return_typing": c.outside, "body_state_or_program_proof": false})
		set := token.NewFileSet()
		infos := map[string]*types.Info{}
		files := map[string]*ast.File{}
		typed := practicalConstantReferenceImports{}
		var failure error
		for _, path := range []string{"example.com/scalar/helper", "example.com/scalar"} {
			if names[path] == "" {
				continue
			}
			raw := source
			if strings.HasSuffix(path, "/helper") {
				raw = helper
			}
			file, err := parser.ParseFile(set, path+".go", raw, parser.AllErrors|parser.SkipObjectResolution)
			if err != nil {
				failure = err
				break
			}
			info := &types.Info{Types: map[ast.Expr]types.TypeAndValue{}, Defs: map[*ast.Ident]types.Object{}, Uses: map[*ast.Ident]types.Object{}, Scopes: map[ast.Node]*types.Scope{}, Selections: map[*ast.SelectorExpr]*types.Selection{}}
			config := types.Config{Importer: typed, GoVersion: "go1.25", Sizes: types.SizesFor("gc", "amd64")}
			pkg, err := config.Check(path, set, []*ast.File{file}, info)
			if err != nil {
				failure = err
				break
			}
			typed[path] = pkg
			infos[path] = info
			files[path] = file
		}

		result := map[string]any{"accepted": failure == nil, "body_state_or_program_proof": false}
		if failure != nil {
			result["error"] = failure.Error()
			rejected++
		} else {
			var key func(types.Type) map[string]any
			key = func(typ types.Type) map[string]any {
				typ = types.Unalias(typ)
				var row map[string]any
				seal := func(fields map[string]any) map[string]any {
					fields["schema"] = practicalSourceTypeSchema
					r, e := practicalTypeKeySeal(practicalSourceTypeDomain, fields)
					if e != nil {
						t.Fatal(e)
					}
					return r
				}
				if named, ok := typ.(*types.Named); ok {
					obj := named.Obj()
					if obj.Pkg() == nil && obj.Name() == "error" {
						return seal(map[string]any{"kind": "builtin", "name": "error", "width": nil, "signed": nil})
					}
					path := obj.Pkg().Path()
					var spec *ast.TypeSpec
					ast.Inspect(files[path], func(n ast.Node) bool {
						if c, ok := n.(*ast.TypeSpec); ok && infos[path].Defs[c.Name] == obj {
							spec = c
						}
						return true
					})
					if spec == nil {
						t.Fatal("named origin", obj)
					}
					file := set.File(spec.Pos())
					decl, e := practicalTypeKeySeal(practicalTypeDeclarationDomain, map[string]any{"schema": practicalTypeDeclarationSchema, "package_id": pkgIDs[path], "source_object_id": fileIDs[path], "kind": "type", "start_byte": file.Offset(spec.Pos()), "end_byte": file.Offset(spec.End()), "name": obj.Name()})
					if e != nil {
						t.Fatal(e)
					}
					return seal(map[string]any{"kind": "nominal", "declaration_id": decl["id"]})
				}
				switch typ := typ.(type) {
				case *types.Basic:
					var width, signed any
					if typ.Info()&types.IsInteger != 0 {
						width = types.SizesFor("gc", "amd64").Sizeof(typ) * 8
						signed = typ.Info()&types.IsUnsigned == 0
					}
					row = map[string]any{"kind": "builtin", "name": types.Typ[typ.Kind()].Name(), "width": width, "signed": signed}
				case *types.Array:
					row = map[string]any{"kind": "array", "length": typ.Len(), "element": key(typ.Elem())["id"]}
				case *types.Slice:
					row = map[string]any{"kind": "slice", "element": key(typ.Elem())["id"]}
				case *types.Struct:
					fields := []map[string]any{}
					for i := 0; i < typ.NumFields(); i++ {
						f := typ.Field(i)
						var pkg any
						if !f.Exported() {
							pkg = pkgIDs[f.Pkg().Path()]
						}
						fields = append(fields, map[string]any{"name": f.Name(), "unexported_package_id": pkg, "type_id": key(f.Type())["id"], "tag_bytes_hex": hex.EncodeToString([]byte(typ.Tag(i))), "embedded": f.Embedded()})
					}
					row = map[string]any{"kind": "struct", "fields": fields}
				default:
					if !c.outside {
						t.Fatal("unmodeled native result", typ)
					}
					return nil
				}
				return seal(row)
			}

			info := infos["example.com/scalar"]
			file := files["example.com/scalar"]
			parents := map[ast.Node]ast.Node{}
			stack := []ast.Node{}
			var query *ast.ReturnStmt
			ast.Inspect(file, func(n ast.Node) bool {
				if n == nil {
					stack = stack[:len(stack)-1]
					return true
				}
				if len(stack) > 0 {
					parents[n] = stack[len(stack)-1]
				}
				stack = append(stack, n)
				if s, ok := n.(*ast.ReturnStmt); ok {
					f := set.File(s.Pos())
					if f.Offset(s.Pos()) == start && f.Offset(s.End()) == end {
						if query != nil {
							t.Fatal("ambiguous return")
						}
						query = s
					}
				}
				return true
			})
			if query == nil {
				t.Fatal("missing return", c.label)
			}
			var owner ast.Node
			var syntax *ast.FuncType
			var body *ast.BlockStmt
			var receiver *ast.FieldList
			var sig *types.Signature
			var name any
			for parent := parents[query]; parent != nil; parent = parents[parent] {
				switch f := parent.(type) {
				case *ast.FuncDecl:
					owner = f
					syntax = f.Type
					body = f.Body
					receiver = f.Recv
					sig = info.Defs[f.Name].Type().(*types.Signature)
					name = f.Name.Name
				case *ast.FuncLit:
					owner = f
					syntax = f.Type
					body = f.Body
					sig = info.TypeOf(f).(*types.Signature)
				}
				if owner != nil {
					break
				}
			}
			if owner == nil {
				t.Fatal("missing enclosing function")
			}
			span := func(n ast.Node) (int, int) { f := set.File(n.Pos()); return f.Offset(n.Pos()), f.Offset(n.End()) }
			fs, fe := span(owner)
			ts, te := span(syntax)
			bs, be := span(body)
			slots := []map[string]any{}
			bindings := []any{}
			values := []map[string]any{}
			addSlots := func(fields *ast.FieldList, tuple *types.Tuple, role string) {
				if fields == nil {
					return
				}
				index := 0
				for _, field := range fields.List {
					count := len(field.Names)
					if count == 0 {
						count = 1
					}
					start, end := span(field)
					typeStart, typeEnd := span(field.Type)
					for j := 0; j < count; j++ {
						var typ types.Type
						if role == "receiver" {
							typ = sig.Recv().Type()
						} else {
							typ = tuple.At(index).Type()
						}
						index++
						var slotName, nameStart, nameEnd any
						if len(field.Names) > 0 {
							ident := field.Names[j]
							slotName = ident.Name
							nameStart, nameEnd = span(ident)
						}
						slots = append(slots, map[string]any{"role": role, "field_start": start, "field_end": end, "type_start": typeStart, "type_end": typeEnd, "name": slotName, "name_start": nameStart, "name_end": nameEnd, "type_key": key(typ)})
						if role == "result" {
							var binding any
							if len(field.Names) > 0 && field.Names[j].Name != "_" {
								ident := field.Names[j]
								obj := info.Defs[ident]
								if obj != sig.Results().At(index-1) {
									t.Fatal("result object origin")
								}
								binding = map[string]any{"name": ident.Name, "name_start": nameStart, "name_end": nameEnd, "declaration_start": fs, "declaration_end": fe, "visible_from": bs}
							}
							bindings = append(bindings, binding)
							values = append(values, map[string]any{"type_key": key(typ), "addressable": false, "constant": false, "untyped_nil": false})
						}
					}
				}
			}
			addSlots(receiver, nil, "receiver")
			addSlots(syntax.Params, sig.Params(), "parameter")
			addSlots(syntax.Results, sig.Results(), "result")
			kind := "FuncDecl"
			if _, ok := owner.(*ast.FuncLit); ok {
				kind = "FuncLit"
			}
			result["function"] = map[string]any{"kind": kind, "start_byte": fs, "end_byte": fe, "type_start": ts, "type_end": te, "body_start": bs, "body_end": be, "name": name, "slots": slots}
			result["implicit_results"] = len(query.Results) == 0
			result["bindings"] = bindings
			result["values"] = values
			calls := []map[string]any{}
			ast.Inspect(query, func(n ast.Node) bool {
				call, ok := n.(*ast.CallExpr)
				if !ok {
					return true
				}
				fun := call.Fun
				for {
					p, ok := fun.(*ast.ParenExpr)
					if !ok {
						break
					}
					fun = p.X
				}
				var obj types.Object
				switch f := fun.(type) {
				case *ast.Ident:
					obj = info.Uses[f]
				case *ast.SelectorExpr:
					if info.Selections[f] != nil {
						return true
					}
					obj = info.Uses[f.Sel]
				}
				fn, ok := obj.(*types.Func)
				if !ok {
					return true
				}
				sig := fn.Type().(*types.Signature)
				if sig.Recv() != nil {
					return true
				}
				path := fn.Pkg().Path()
				var decl *ast.FuncDecl
				ast.Inspect(files[path], func(n ast.Node) bool {
					if d, ok := n.(*ast.FuncDecl); ok && infos[path].Defs[d.Name] == fn {
						decl = d
					}
					return true
				})
				if decl == nil {
					t.Fatal("missing call origin")
				}
				f := set.File(call.Pos())
				df := set.File(decl.Pos())
				var bs, be any
				if decl.Body != nil {
					bs, be = df.Offset(decl.Body.Pos()), df.Offset(decl.Body.End())
				}
				calls = append(calls, map[string]any{"start_byte": f.Offset(call.Pos()), "end_byte": f.Offset(call.End()), "declaration_source": fileIDs[path], "declaration_start": df.Offset(decl.Pos()), "declaration_end": df.Offset(decl.End()), "body_start": bs, "body_end": be})
				return true
			})
			sort.Slice(calls, func(i, j int) bool {
				if calls[i]["start_byte"].(int) == calls[j]["start_byte"].(int) {
					return calls[i]["end_byte"].(int) < calls[j]["end_byte"].(int)
				}
				return calls[i]["start_byte"].(int) < calls[j]["start_byte"].(int)
			})
			result["calls"] = calls
			accepted++
		}
		write(filepath.Join(dir, "native.json"), result)
	}
	write(filepath.Join(out, "summary.json"), map[string]any{"cases": len(practicalReturnTypingCases()), "accepted": accepted, "rejected": rejected, "runtime": runtime.Version(), "sizes": "gc/amd64", "applications_executed": false, "body_state_or_program_proof": false})
}
