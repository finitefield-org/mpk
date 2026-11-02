//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/constant"
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

type practicalBodyTypingCase struct {
	label, body, results, params, receiver, extra string
	outside, helper                               bool
}

func practicalBodyTypingCases() []practicalBodyTypingCase {
	cases := []practicalBodyTypingCase{}
	add := func(body, results string) {
		cases = append(cases, practicalBodyTypingCase{label: body + "/" + results, body: body, results: results})
	}
	for _, cond := range []string{"c", "b", "x==1", "n>0", "true", "false", "1", "x", "s", "nil", "1<<n", "Flag(true)", "Id(x)==1", "Pair(x)"} {
		for _, shape := range []int{0, 1, 2, 3, 4} {
			body := ""
			switch shape {
			case 0:
				body = "if " + cond + "{x++}"
			case 1:
				body = "if q:=x;" + cond + "{_=q}else if c{x++}else{x--}"
			case 2:
				body = "for " + cond + "{break}"
			case 3:
				body = "for i:=0;" + cond + ";i++{_=i;continue}"
			case 4:
				body = "for ;" + cond + ";x++{break}"
			}
			add(body, "")
		}
	}
	for _, tag := range []string{"x", "v", "n", "b", "c", "text", "a", "s", "err", "1", "true", "nil", ""} {
		for _, value := range []string{"1", "x", "v", "b", "c", "true", "text", "\"x\"", "a", "[2]Word{}", "s", "nil", "Code(1)", "Good{}", "1<<n"} {
			add("switch "+tag+"{case "+value+":x++;default:x--}", "")
		}
	}
	for _, expr := range []string{"a", "s", "[]Word{}", "make([]Word,2)", "GetSlice()", "\"x\"", "n", "nil"} {
		outside := expr == "\"x\"" || expr == "n"
		for _, clause := range []string{"", "i:=", "_,q:=", "i,q:=", "n=", "_,x=", "_,s[0]=", "x=", "_=", "_:=", "i,i:="} {
			body := "for " + clause + "range " + expr + "{"
			if strings.Contains(clause, "i") {
				body += "_=i;"
			}
			if strings.Contains(clause, "q") {
				body += "_=q;"
			}
			body += "if c{continue};break}"
			cases = append(cases, practicalBodyTypingCase{label: body, body: body, outside: outside})
		}
	}
	for _, body := range []string{
		"var q Word;_=q", "var q Word;q=1", "q:=x;_=q", "q:=x;q=x", "var q Word;q++", "var q Word;q+=1", "var q [2]Word;q[0]=1", "var q Record;q.W=1", "var q,_=Pair(x);_=q", "var _,q=Pair(x);_=q", "var _ Word", "var _=nil", "var q,bad uint8=1,256;_,_=q,bad", "x,q:=x,1;_=q", "x,q:=Other(1),1;_=q", "q,q:=1,2;_=q", "var q=missing;_=q",
		"const(C=iota;_;D);var _ uint8=D", "const(_=1/0)", "const(_ uint8=256)", "const(_=Id(x))", "const(C=iota;D;E);var _ [E]Word", "const(C uint16=iota;D);var _ uint16=D", "const(C=iota;D \"bad\");_=C", "const(_=nil)", "const _=len(\"x\")", "const(C=1;D=C+1);var _ [D]Word", "const(C=C);_=C", "const(C,D=1,2);_,_=C,D", "const(C=1;_,_=2,3)",
		"type L Word;var q L=1;_=q", "type L=Word;var q L=x;_=q", "type _ uint16", "type L struct{W Word};var q L;_=q", "type L struct{N L};var _ L",
		"Void()", "Id(x)", "Pair(x)", "(Void())", "Good{}.Error()", "copy(s,s)", "(copy(s,s))", "append(s,x)", "len(s)", "cap(a)", "make([]Word,2)", "Word(x)", "x+1", "x", "min(x,1)",
		"switch x{case 1:case 1:}", "switch x{case 1,1:}", "switch x{case 1:case Word(1):}", "switch x{default:default:}", "switch err{case Code(1):case OtherCode(1):}", "switch err{case Code(1):case Code(1):}", "switch s{case nil:case nil:}", "switch s{case []Word(nil):}", "switch a{case [2]Word{}:case [2]Word{}:}",
		"break", "continue", "for{switch x{case 1:continue;default:break};break}", "switch x{case 1:continue}", "for{for{break};continue}", "if true{break}", "for i:=0;i<1;q:=1{_,_=i,q}", "if x:=1;true{_=x}", "for i:=0;i<1;i++{}", "for i:=0;i<1;i++{_=i}",
		"return", "return 1", "if false{var _ Word=\"bad\"}", "return;var _ Word=\"bad\"", "if false{const _=1/0}", "for false{var _ Word=\"bad\"}", "switch x{default:var _ Word=\"bad\"}",
	} {
		add(body, "")
	}
	for _, body := range []string{
		"return x", "", "if c{return x}", "if c{return x}else{return 0}", "if c{return x}else if b{return 1}else{return 2}", "if true{return x}", "for{}", "for{continue}", "for{if false{break}}", "for{for{break}}", "for{switch x{default:break}}", "for{switch x{default:continue}}", "for true{}", "for false{return x}", "for range s{return x}", "switch x{default:return x}", "switch x{case 1:return x}", "switch x{case 1:return x;default:return 0}", "switch x{case 1:break;default:return x}", "switch x{case 1:for{};default:return x}", "{return x};", "return x;Void()", "return x;;", "return x;var q=1;_=q", "return Pair(x)",
	} {
		add(body, "Word")
	}
	for _, body := range []string{"return Pair(x)", "return x,0", "return x,nil", "r=x;return", "if c{r:=x;_=r;return};return", "r,q:=x,1;_=q;return", "for _,r:=range s{_=r;return};return"} {
		for _, results := range []string{"r Word,t uint64", "r Word,e error", "_ Word,_ error"} {
			add(body, results)
		}
	}
	for _, body := range []string{"var q=h.Record{};_=q", "h.Id(v)", "return h.Pair(v)", "switch h.Error(1){case h.Error(1):}", "var q error=h.Error(1);_=q"} {
		cases = append(cases, practicalBodyTypingCase{label: "helper/" + body, body: body, helper: true, results: func() string {
			if strings.HasPrefix(body, "return") {
				return "uint16,uint64"
			}
			return ""
		}()})
	}
	cases = append(cases, practicalBodyTypingCase{label: "method-body", receiver: "self Code", body: "return self", results: "Code"})
	for _, body := range []string{"L:for{break L}", "goto L;L:return", "for{switch x{case 1:fallthrough;default:break};break}", "defer Void()", "go Void()", "select{}", "switch any(x).(type){case Word:}", "var f=func(){return};f()", "var p *Word;_=p", "panic(\"x\")", "const N=len(a);var _ [N]Word", "const N=cap(a);var _ [N]Word"} {
		cases = append(cases, practicalBodyTypingCase{label: "outside/" + body, body: body, outside: true})
	}
	params := []string{}
	for n := 0; n < 70; n++ {
		params = append(params, fmt.Sprintf("p%d Word", n))
	}
	cases = append(cases, practicalBodyTypingCase{label: "seventy-parameters", params: strings.Join(params, ","), results: "Word", body: "return p69"})
	body := ""
	for n := 0; n < 200; n++ {
		body += "if c{x++}else{x--};"
	}
	add(body, "")
	return cases
}
func TestPracticalOriginalBodyTypingReference(t *testing.T) {
	if runtime.Version() != practicalCompilerRelease {
		t.Fatal(runtime.Version())
	}
	out := os.Getenv("MPK_GO_BODY_TYPING_NATIVE_OUTPUT")
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
	for index, c := range practicalBodyTypingCases() {
		dir := filepath.Join(out, fmt.Sprintf("case-%05d", index))
		if err := os.Mkdir(dir, 0o700); err != nil {
			t.Fatal(err)
		}
		source := "package policy\n"
		helper := ""
		if c.helper {
			source += "import h \"example.com/scalar/helper\"\nvar _ h.Record\n"
			helper = "package foreign\ntype Record struct{N uint16}\ntype Error uint16\nfunc(Error)Error()string{return \"foreign\"}\nfunc Id(x uint16)uint16{return x}\nfunc Pair(x uint16)(uint16,uint64){return x,0}\n"
		}
		source += `type Word uint16
type Other uint16
type Flag bool
type Text string
type Code uint16
func(Code)Error()string{return "code"}
type OtherCode uint16
func(OtherCode)Error()string{return "other"}
type Good struct{}
func(Good)Error()string{return "good"}
type Record struct{W Word;A [2]Word;S []Word}
func Id(x Word)Word{return x}
func Pair(x Word)(Word,uint64){return x,0}
func Void(){}
func GetSlice()[]Word{return nil}
func Opaque(x Word)Word
`
		source += c.extra
		params := c.params
		if params == "" {
			params = "x Word,v uint16,n int,b Flag,c bool,text Text,a [2]Word,s []Word,err error"
		}
		annotation := ""
		if c.results != "" {
			annotation = "(" + c.results + ")"
		}
		receiver := ""
		if c.receiver != "" {
			receiver = "(" + c.receiver + ")"
		}
		start := len(source)
		source += "func " + receiver + "Probe(" + params + ")" + annotation + "{\n" + c.body + "\n}"
		end := len(source)
		source += "\n"
		cap := fmt.Sprintf("body-typing/case-%05d", index)
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

		write(filepath.Join(dir, "input.json"), map[string]any{"index": index, "label": c.label, "components_hex": hex.EncodeToString(canonical), "sources": originals, "query_source_object_id": root.object.ID, "function_start_byte": start, "function_end_byte": end, "outside_body_typing": c.outside, "evaluation_state_or_program_proof": false})
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

		result := map[string]any{"accepted": failure == nil, "evaluation_state_or_program_proof": false}
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
			var fn *ast.FuncDecl
			for _, d := range file.Decls {
				if f, ok := d.(*ast.FuncDecl); ok && f.Name.Name == "Probe" {
					fn = f
				}
			}
			if fn == nil {
				t.Fatal("missing function")
			}
			f := set.File(fn.Pos())
			if f.Offset(fn.Pos()) != start || f.Offset(fn.End()) != end {
				t.Fatal("function origin")
			}
			sig := info.Defs[fn.Name].Type().(*types.Signature)
			span := func(n ast.Node) (int, int) { f := set.File(n.Pos()); return f.Offset(n.Pos()), f.Offset(n.End()) }
			ts, te := span(fn.Type)
			bs, be := span(fn.Body)
			slots := []map[string]any{}
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
					a, b := span(field)
					ta, tb := span(field.Type)
					for j := 0; j < count; j++ {
						var typ types.Type
						if role == "receiver" {
							typ = sig.Recv().Type()
						} else {
							typ = tuple.At(index).Type()
						}
						index++
						var name, ns, ne any
						if len(field.Names) > 0 {
							name = field.Names[j].Name
							ns, ne = span(field.Names[j])
						}
						slots = append(slots, map[string]any{"role": role, "field_start": a, "field_end": b, "type_start": ta, "type_end": tb, "name": name, "name_start": ns, "name_end": ne, "type_key": key(typ)})
					}
				}
			}
			addSlots(fn.Recv, nil, "receiver")
			addSlots(fn.Type.Params, sig.Params(), "parameter")
			addSlots(fn.Type.Results, sig.Results(), "result")
			result["function"] = map[string]any{"start_byte": start, "end_byte": end, "type_start": ts, "type_end": te, "body_start": bs, "body_end": be, "name": fn.Name.Name, "slots": slots}
			parents := map[ast.Node]ast.Node{}
			stack := []ast.Node{}
			nodes := []ast.Stmt{}
			ast.Inspect(fn.Body, func(n ast.Node) bool {
				if n == nil {
					stack = stack[:len(stack)-1]
					return true
				}
				if len(stack) > 0 {
					parents[n] = stack[len(stack)-1]
				}
				stack = append(stack, n)
				if s, ok := n.(ast.Stmt); ok {
					nodes = append(nodes, s)
				}
				return true
			})
			targets := map[*ast.BranchStmt]ast.Stmt{}
			breaks := map[ast.Stmt]bool{}
			branches := []map[string]any{}
			for _, s := range nodes {
				if b, ok := s.(*ast.BranchStmt); ok && b.Label == nil && (b.Tok == token.BREAK || b.Tok == token.CONTINUE) {
					for p := parents[b]; p != nil; p = parents[p] {
						target, ok := p.(ast.Stmt)
						if !ok {
							continue
						}
						loop := false
						br := false
						switch target.(type) {
						case *ast.ForStmt, *ast.RangeStmt:
							loop = true
							br = true
						case *ast.SwitchStmt, *ast.TypeSwitchStmt, *ast.SelectStmt:
							br = true
						}
						if b.Tok == token.CONTINUE && !loop || b.Tok == token.BREAK && !br {
							continue
						}
						targets[b] = target
						if b.Tok == token.BREAK {
							breaks[target] = true
						}
						a, z := span(b)
						ta, tz := span(target)
						branches = append(branches, map[string]any{"start_byte": a, "end_byte": z, "target_start": ta, "target_end": tz, "operator": b.Tok.String()})
						break
					}
				}
			}
			var term func(ast.Stmt) bool
			var ends func([]ast.Stmt) bool
			ends = func(list []ast.Stmt) bool {
				for n := len(list) - 1; n >= 0; n-- {
					if _, empty := list[n].(*ast.EmptyStmt); !empty {
						return term(list[n])
					}
				}
				return false
			}
			term = func(s ast.Stmt) bool {
				switch s := s.(type) {
				case *ast.ReturnStmt:
					return true
				case *ast.BlockStmt:
					return ends(s.List)
				case *ast.CaseClause:
					return ends(s.Body)
				case *ast.IfStmt:
					return s.Else != nil && term(s.Body) && term(s.Else)
				case *ast.ForStmt:
					return s.Cond == nil && !breaks[s]
				case *ast.SwitchStmt:
					hasDefault := false
					for _, clause := range s.Body.List {
						c := clause.(*ast.CaseClause)
						if len(c.List) == 0 {
							hasDefault = true
						}
						if !ends(c.Body) {
							return false
						}
					}
					return hasDefault && !breaks[s]
				default:
					return false
				}
			}
			statements := []map[string]any{}
			uses := []map[string]any{}
			constants := []map[string]any{}
			locals := []map[string]any{}
			use := func(x ast.Expr, role string, expected types.Type) {
				typ := info.TypeOf(x)
				nilValue := typ == types.Typ[types.UntypedNil]
				if nilValue {
					typ = expected
				} else if basic, ok := typ.(*types.Basic); ok && basic.Info()&types.IsUntyped != 0 {
					if expected != nil {
						if _, isInterface := expected.Underlying().(*types.Interface); isInterface {
							typ = types.Default(typ)
						} else {
							typ = expected
						}
					} else {
						typ = types.Default(typ)
					}
				}
				a, b := span(x)
				uses = append(uses, map[string]any{"start_byte": a, "end_byte": b, "role": role, "type_key": key(typ), "untyped_nil": nilValue})
			}
			for _, s := range nodes {
				a, b := span(s)
				kind := fmt.Sprintf("%T", s)
				kind = strings.TrimPrefix(kind, "*ast.")
				values := []map[string]any{}
				switch s := s.(type) {
				case *ast.AssignStmt:
					rhs := []types.Type{}
					for _, x := range s.Rhs {
						if tuple, ok := info.TypeOf(x).(*types.Tuple); ok {
							for n := 0; n < tuple.Len(); n++ {
								rhs = append(rhs, tuple.At(n).Type())
							}
						} else {
							rhs = append(rhs, info.TypeOf(x))
						}
					}
					for n, x := range s.Lhs {
						var typ types.Type
						if id, ok := ast.Unparen(x).(*ast.Ident); ok && id.Name == "_" {
							if n < len(rhs) {
								typ = types.Default(rhs[n])
							}
						} else {
							typ = info.TypeOf(x)
						}
						if typ != nil {
							values = append(values, key(typ))
						}
					}
				case *ast.IncDecStmt:
					values = append(values, key(info.TypeOf(s.X)))
				case *ast.ReturnStmt:
					for n := 0; n < sig.Results().Len(); n++ {
						values = append(values, key(sig.Results().At(n).Type()))
					}
				case *ast.IfStmt:
					use(s.Cond, "condition", nil)
				case *ast.ForStmt:
					if s.Cond != nil {
						use(s.Cond, "condition", nil)
					}
				case *ast.RangeStmt:
					use(s.X, "range_source", nil)
					for n, x := range []ast.Expr{s.Key, s.Value} {
						if x != nil {
							if id, ok := ast.Unparen(x).(*ast.Ident); ok && id.Name == "_" {
								continue
							}
							role := "range_key"
							if n == 1 {
								role = "range_value"
							}
							use(x, role, nil)
						}
					}
				case *ast.SwitchStmt:
					var tag types.Type = types.Typ[types.Bool]
					if s.Tag != nil {
						tag = types.Default(info.TypeOf(s.Tag))
						use(s.Tag, "switch_tag", nil)
					}
					for _, clause := range s.Body.List {
						for _, x := range clause.(*ast.CaseClause).List {
							use(x, "case_value", tag)
						}
					}
				case *ast.DeclStmt:
					if g, ok := s.Decl.(*ast.GenDecl); ok && g.Tok == token.CONST {
						for _, spec := range g.Specs {
							v := spec.(*ast.ValueSpec)
							vs, ve := span(v)
							rows := []map[string]any{}
							for _, id := range v.Names {
								obj := info.Defs[id].(*types.Const)
								typ := obj.Type()
								var typeKey any
								kind := "integer"
								if basic, ok := typ.(*types.Basic); !ok || basic.Info()&types.IsUntyped == 0 {
									typeKey = key(typ)
								}
								value := obj.Val()
								var integer, boolean, bytesHex any
								switch value.Kind() {
								case constant.Int:
									integer = value.ExactString()
									if basic, ok := typ.(*types.Basic); ok && basic.Kind() == types.UntypedRune {
										kind = "rune"
									}
								case constant.Bool:
									kind = "bool"
									boolean = constant.BoolVal(value)
								case constant.String:
									kind = "string"
									bytesHex = hex.EncodeToString([]byte(constant.StringVal(value)))
								default:
									if !c.outside {
										t.Fatal("outside constant", value)
									}
								}
								rows = append(rows, map[string]any{"kind": kind, "type_key": typeKey, "integer_decimal": integer, "boolean": boolean, "bytes_hex": bytesHex})
							}
							constants = append(constants, map[string]any{"start_byte": vs, "end_byte": ve, "values": rows})
						}
					}
				}
				statements = append(statements, map[string]any{"start_byte": a, "end_byte": b, "kind": kind, "value_types": values, "terminating": term(s)})
			}
			for id, obj := range info.Defs {
				v, ok := obj.(*types.Var)
				if !ok || id.Name == "_" || v.IsField() {
					continue
				}
				p := parents[id]
				var declaration ast.Node
				switch p := p.(type) {
				case *ast.ValueSpec:
					declaration = p
				case *ast.AssignStmt:
					declaration = p
				case *ast.RangeStmt:
					declaration = p
				}
				if declaration == nil {
					continue
				}
				a, b := span(id)
				da, db := span(declaration)
				locals = append(locals, map[string]any{"name": id.Name, "name_start": a, "name_end": b, "declaration_start": da, "declaration_end": db, "type_key": key(v.Type())})
			}
			sortRows := func(rows []map[string]any, start, end string) {
				sort.Slice(rows, func(i, j int) bool {
					if rows[i][start].(int) == rows[j][start].(int) {
						return rows[i][end].(int) < rows[j][end].(int)
					}
					return rows[i][start].(int) < rows[j][start].(int)
				})
			}
			sortRows(statements, "start_byte", "end_byte")
			sortRows(uses, "start_byte", "end_byte")
			sortRows(branches, "start_byte", "end_byte")
			sortRows(constants, "start_byte", "end_byte")
			sortRows(locals, "name_start", "name_end")
			result["statements"] = statements
			result["expressions"] = uses
			result["branches"] = branches
			result["constants"] = constants
			result["locals"] = locals
			result["terminating"] = term(fn.Body)
			accepted++
		}
		write(filepath.Join(dir, "native.json"), result)
	}
	write(filepath.Join(out, "summary.json"), map[string]any{"cases": len(practicalBodyTypingCases()), "accepted": accepted, "rejected": rejected, "runtime": runtime.Version(), "sizes": "gc/amd64", "applications_executed": false, "evaluation_state_or_program_proof": false})
}
