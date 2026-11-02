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

type practicalScalarTypingCase struct {
	label, expr, target, setup, body, mode string
	outside                                bool
	helper                                 bool
}

func practicalScalarTypingCases() []practicalScalarTypingCase {
	cases := []practicalScalarTypingCase{}
	add := func(label, expr, target string) {
		cases = append(cases, practicalScalarTypingCase{label: label, expr: expr, target: target, mode: "value"})
	}
	operands := []string{"x", "y", "u", "v", "s", "count", "0", "1", "-1", "255", "256", "(1<<63)", "((1<<64)-1)", "Word(1)", "Other(1)", "'a'"}
	for _, a := range operands {
		for _, b := range operands {
			for _, op := range []string{"+", "-", "*", "/", "%", "&", "|", "^", "&^", "==", "!=", "<", "<=", ">", ">="} {
				add("binary-"+a+op+b, "("+a+")"+op+"("+b+")", "")
			}
		}
	}
	for _, left := range []string{"x", "u", "s", "1", "'a'", "(1<<63)", "uint8(1)"} {
		for _, right := range []string{"count", "s", "-1", "0", "63", "64", "1074", "1075", "(1<<63)", "((1<<64)-1)", "(1<<64)", "(1<<count)"} {
			for _, op := range []string{"<<", ">>"} {
				for _, target := range []string{"", "Word", "uint64", "int8"} {
					add("shift-"+left+op+right+"-"+target, "("+left+")"+op+"("+right+")", target)
				}
			}
		}
	}
	for _, expr := range []string{"1<<count", "(1<<count)+1", "(1<<count)+256", "-(1<<count)", "^(1<<count)", "(1<<count)<<s", "(1<<count)+('a'<<s)", "(1<<count)==(1<<s)", "(1<<count)==uint64(1)", "(1<<count)<(1<<63)", "(1<<count)<('a')", "(1<<count)/(1-1)", "x/0", "x%(2-2)", "x+256", "v+256", "u+((1<<64)-1)", "uint8(x+256)", "uint64(1<<count)", "uint8((1<<count)+256)", "string(1<<count)", "string(x)", "string(0xd800)", "Word(u)", "Word(Other(1))", "Other(x)", "uint16(x)", "x==y", "x==Other(1)", "b&&c", "b&&(x<y)", "(x<y)&&(u>0)", "!(x<y)", "Flag(x<y)", "b==c", "b==Flag(c)", "text+\"x\"", "text+str", "text<\"x\"", "len(str)", "len(Text(\"abc\"))", "min(x,y,1)", "max(1<<count,2)", "min(1<<count,'a')", "min(x,256)", "max(text,\"x\")", "max(text,str)", "min(b,c)", "min()", "len(x)", "Id(x)", "Id(1<<count)", "Id(v)", "Id(Word(v))", "Combine(Pair(x))", "Combine(Pair(x),u)", "Id(Pair(x))", "Zero(Void())", "Zero()", "Pair(x)", "Opaque(x)", "x(1)", "1.0", "uint16(1.0)", "len([2]byte{})"} {
		for _, target := range []string{"", "Word", "Flag", "uint64"} {
			outside := expr == "1.0" || expr == "uint16(1.0)" || expr == "len([2]byte{})"
			cases = append(cases, practicalScalarTypingCase{label: "edge-" + expr + "-" + target, expr: expr, target: target, mode: "value", outside: outside})
		}
	}
	for _, typ := range []string{"int8", "uint8", "int16", "uint16", "int32", "uint32", "int64", "uint64", "int", "uint", "Word", "Other", "Flag", "Text"} {
		for _, expr := range []string{"x", "u", "s", "b", "c", "text", "str", "1", "-1", "255", "256", "true", "\"a\""} {
			add("cast-"+typ+"-"+expr, typ+"("+expr+")", "")
		}
	}
	for _, body := range []string{
		"var Result=1;_=Result", "var Result='a';_=Result", "var Result=1<<count;_=Result", "var Result Word=1<<count;_=Result", "var Result=Pair(x);_=Result", "var a,Result=Pair(x);_=a;_=Result", "var Result,tail=Pair(x);_=Result;_=tail", "var Result,tail uint64=Pair(x);_=Result;_=tail", "var Result Word;_=Result", "var Result Word=\"bad\";_=Result", "var Result,bad uint8=1,256;_=Result;_=bad", "x,Result:=x+1,2;_=Result", "x,Result:=Other(1),2;_=Result", "Result,tail:=Pair(x);_=Result;_=tail", "var q Word=1;Result:=q;_=Result", "var q=1<<count;Result:=q;_=Result", "const C Word=1;Result:=C;_=Result", "const(A=iota;B;C);Result:=C;_=Result", "var q=1;{q,Result:=q+1,x;_=q;_=Result}", "var a,b uint8=1,256;Result:=a;_=b;_=Result", "var a,b=Pair(x);Result:=Combine(a,b);_=Result", "var a Word;var b uint64;a,Result:=Pair(x);_=a;_=b;_=Result", "var q=missing;Result:=q;_=Result", "var q=q;Result:=q;_=Result",
	} {
		cases = append(cases, practicalScalarTypingCase{label: "binding-" + body, body: body, mode: "binding"})
	}
	for _, expr := range []string{"h.Id(x)", "h.Pair(x)", "h.Id(h.C)", "x+h.C"} {
		cases = append(cases, practicalScalarTypingCase{label: "import-" + expr, expr: expr, mode: "value", helper: true})
	}
	for _, expr := range []string{"Void()", "Pair(x)", "Many()"} {
		cases = append(cases, practicalScalarTypingCase{label: "tuple-" + expr, expr: expr, mode: "tuple"})
	}
	return cases
}
func TestPracticalOriginalScalarTypingReference(t *testing.T) {
	if runtime.Version() != practicalCompilerRelease {
		t.Fatal(runtime.Version())
	}
	out := os.Getenv("MPK_GO_SCALAR_TYPING_NATIVE_OUTPUT")
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
	for index, c := range practicalScalarTypingCases() {
		dir := filepath.Join(out, fmt.Sprintf("case-%05d", index))
		if err := os.Mkdir(dir, 0o700); err != nil {
			t.Fatal(err)
		}
		source := "package policy\n"
		helper := ""
		if c.helper {
			source += "import h \"example.com/scalar/helper\"\n"
			helper = "package foreign\ntype Word uint16\nconst C=1\nfunc Id(x uint16)uint16{return x}\nfunc Pair(x uint16)(uint16,uint64){return x,0}\n"
		}
		source += `type Word uint16
type Other uint16
type Flag bool
type Text string
type Alias=Word
func Id(x Word)Word{return x}
func Pair(x Word)(Word,uint64){return x,0}
func Combine(x Word,u uint64)Word{return x}
func Zero()uint16{return 0}
func Void(){}
func Opaque(x Word)Word
`
		source += "func Many()("
		parts := []string{}
		for n := 0; n < 16; n++ {
			parts = append(parts, fmt.Sprintf("r%d Word", n))
		}
		source += strings.Join(parts, ",") + "){return}\n"
		source += "func Probe(x,y Word,u uint64,v uint16,s int,count uint64,b Flag,c bool,text Text,str string){\n" + c.setup
		start, end := 0, 0
		if c.mode == "binding" {
			source += c.body
		} else if c.mode == "tuple" {
			start = len(source)
			source += c.expr
			end = len(source)
		} else {
			source += "var Result " + c.target + " = "
			start = len(source)
			source += c.expr
			end = len(source)
			source += ";_ = Result"
		}
		source += "\n}\n"
		cap := fmt.Sprintf("scalar-typing/case-%05d", index)
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
		write(filepath.Join(dir, "input.json"), map[string]any{"index": index, "label": c.label, "components_hex": hex.EncodeToString(canonical), "sources": originals, "query_source_object_id": root.object.ID, "query_mode": c.mode, "expression_start_byte": start, "expression_end_byte": end, "target_type": c.target, "outside_scalar_typing": c.outside, "source_typing_or_context_or_program_authority": false})
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
			info := &types.Info{Types: map[ast.Expr]types.TypeAndValue{}, Defs: map[*ast.Ident]types.Object{}, Uses: map[*ast.Ident]types.Object{}, Scopes: map[ast.Node]*types.Scope{}}
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
		result := map[string]any{"accepted": failure == nil, "initializer_body_or_invocation_state_or_program_authority": false}
		if failure != nil {
			result["error"] = failure.Error()
			rejected++
		} else {
			key := func(typ types.Type) (map[string]any, string) {
				t.Helper()
				typ = types.Unalias(typ)
				basic, ok := typ.Underlying().(*types.Basic)
				if !ok {
					t.Fatal("reference scalar result", typ)
				}
				builtin := types.Typ[basic.Kind()].Name()
				var row map[string]any
				if named, ok := typ.(*types.Named); ok {
					obj := named.Obj()
					path := obj.Pkg().Path()
					var spec *ast.TypeSpec
					ast.Inspect(files[path], func(n ast.Node) bool {
						if candidate, ok := n.(*ast.TypeSpec); ok && infos[path].Defs[candidate.Name] == obj {
							spec = candidate
						}
						return true
					})
					if spec == nil {
						t.Fatal("named origin", obj)
					}
					file := set.File(spec.Pos())
					decl, err := practicalTypeKeySeal(practicalTypeDeclarationDomain, map[string]any{"schema": practicalTypeDeclarationSchema, "package_id": pkgIDs[path], "source_object_id": fileIDs[path], "kind": "type", "start_byte": file.Offset(spec.Pos()), "end_byte": file.Offset(spec.End()), "name": obj.Name()})
					if err != nil {
						t.Fatal(err)
					}
					row, err = practicalTypeKeySeal(practicalSourceTypeDomain, map[string]any{"schema": practicalSourceTypeSchema, "kind": "nominal", "declaration_id": decl["id"]})
					if err != nil {
						t.Fatal(err)
					}
				} else {
					var width, signed any
					if basic.Info()&types.IsInteger != 0 {
						width = types.SizesFor("gc", "amd64").Sizeof(basic) * 8
						signed = basic.Info()&types.IsUnsigned == 0
					}
					row, err = practicalTypeKeySeal(practicalSourceTypeDomain, map[string]any{"schema": practicalSourceTypeSchema, "kind": "builtin", "name": builtin, "width": width, "signed": signed})
					if err != nil {
						t.Fatal(err)
					}
				}
				return row, builtin
			}
			info := infos["example.com/scalar"]
			values := []map[string]any{}
			add := func(typ types.Type, value constant.Value, addressable bool) {
				typeKey, builtin := key(typ)
				var integer, boolean, bytesHex any
				if value != nil {
					switch value.Kind() {
					case constant.Int:
						integer = value.ExactString()
					case constant.Bool:
						boolean = constant.BoolVal(value)
					case constant.String:
						bytesHex = hex.EncodeToString([]byte(constant.StringVal(value)))
					default:
						if !c.outside {
							t.Fatal("outside constant", value)
						}
					}
				}
				values = append(values, map[string]any{"type_key": typeKey, "underlying_builtin": builtin, "constant": value != nil, "integer_decimal": integer, "boolean": boolean, "bytes_hex": bytesHex, "addressable": addressable})
			}
			if c.mode == "binding" {
				found := false
				for ident, obj := range info.Defs {
					if ident.Name == "Result" && obj != nil {
						if found {
							t.Fatal("ambiguous Result")
						}
						found = true
						add(obj.Type(), nil, true)
					}
				}
				if !found {
					t.Fatal("missing Result", c.label)
				}
			} else {
				found := false
				for expr, fact := range info.Types {
					f := set.File(expr.Pos())
					if f.Offset(expr.Pos()) == start && f.Offset(expr.End()) == end {
						if found {
							t.Fatal("ambiguous expression", c.label)
						}
						found = true
						if tuple, ok := fact.Type.(*types.Tuple); ok {
							for n := 0; n < tuple.Len(); n++ {
								add(tuple.At(n).Type(), nil, false)
							}
						} else {
							add(fact.Type, fact.Value, fact.Addressable())
						}
					}
				}
				if !found {
					t.Fatal("missing expression", c.label)
				}
			}
			result["values"] = values
			accepted++
		}
		write(filepath.Join(dir, "native.json"), result)
	}
	write(filepath.Join(out, "summary.json"), map[string]any{"cases": len(practicalScalarTypingCases()), "accepted": accepted, "rejected": rejected, "runtime": runtime.Version(), "applications_executed": false})
	t.Logf("actual scalar typing cases %d accepted %d rejected %d", len(practicalScalarTypingCases()), accepted, rejected)
}
