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

type practicalAggregateConstantCase struct {
	label, source, helper string
	unsupported           bool
}

func practicalAggregateConstantCases() []practicalAggregateConstantCase {
	cases := []practicalAggregateConstantCase{}
	add := func(label, source string) {
		cases = append(cases, practicalAggregateConstantCase{label: label, source: "package policy\ntype W uint16\n" + source + "\n"})
	}
	for _, n := range []int{0, 1, 2, 3, 7, 16, 64, 255, 256} {
		for _, builtin := range []string{"len", "cap"} {
			for _, shape := range []int{0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10} {
				typ := fmt.Sprintf("[%d]W", n)
				decl, expr := "", "a"
				switch shape {
				case 0:
					decl = "var a " + typ
				case 1:
					decl = "var a=" + typ + "{}"
				case 2:
					decl = "type A=" + typ + ";type B=A;var a B"
				case 3:
					decl = "type A " + typ + ";type B A;var a B"
				case 4:
					decl = "var a,b=" + typ + "{},0;var _=b"
				case 5:
					decl = "var a struct{V " + typ + "}"
					expr = "a.V"
				case 6:
					decl = "var a [2]" + typ
					expr = "a[1]"
				case 7:
					decl = "func Get()" + typ + "{return " + typ + "{}};var a=Get()"
				case 8:
					decl = "type R struct{};func(R)Get()" + typ + "{return " + typ + "{}};var a=R{}.Get()"
				case 9:
					decl = "func Pair()(int," + typ + "){return 0," + typ + "{}};var _,a=Pair()"
				case 10:
					decl = "var a " + typ
					expr = "(" + expr + ")"
				}
				add(fmt.Sprintf("shape-%d-%s-%d", n, builtin, shape), decl+";const Result="+builtin+"("+expr+")")
			}
			for _, form := range []string{"parameter", "short", "named-result", "local-type", "inferred", "local-read"} {
				typ := fmt.Sprintf("[%d]W", n)
				head, body := "func Probe()", "var a "+typ+";"
				switch form {
				case "parameter":
					head = "func Probe(a " + typ + ")"
					body = ""
				case "short":
					body = "a:=" + typ + "{};"
				case "named-result":
					head = "func Probe()(a " + typ + ")"
					body = ""
				case "local-type":
					body = "type A=" + typ + ";var a A;"
				case "inferred":
					body = "var a=" + typ + "{};"
				}
				body += "const Result=" + builtin + "(a);var _ [Result]W;_=Result;"
				if form == "named-result" {
					body += "return"
				}
				add(fmt.Sprintf("local-%d-%s-%s", n, builtin, form), head+"{"+body+"}")
			}
		}
	}
	for _, builtin := range []string{"len", "cap"} {
		for _, expr := range []string{"[3]W{}", "[...]W{}", "[...]W{4:1}", "[...]W{2:1,7:2,3}", "[2][3]W{{1},{2}}[0]", "[2]struct{A [3]W}{{A:[3]W{}}}[0].A", "[2]W{x,1}", "[2]W{W(1),2}", "[2]W{min(1,2),max(0,1)}", "[2]W{W(len(a)),0}", "[2]W{W(cap(a)),0}", "[2]W{W(len(\"abc\")),0}", "Get()", "R{}.Get()", "[2]W{Id(x),0}", "[2]W{r.Id(),0}", "[2]W{W(len(s)),0}", "[2]W{W(copy(s,s)),0}", "[2]W{W(len(GetSlice())),0}", "[2]W{1/0,0}", "[2]W{3:0}", "[2]W{0:1,0:2}", "[2]W{true,0}", "s", "\"abc\"", "x", "nil", "[2]W{}...", ""} {
			add("expression-"+builtin+"-"+expr, "var a[3]W;var x W;var s[]W;type R struct{};var r R;func Id(x W)W{return x};func(R)Id()W{return 1};func Get()[2]W{return [2]W{}};func(R)Get()[2]W{return [2]W{}};func GetSlice()[]W{return nil};const Result="+builtin+"("+expr+")")
		}
	}
	for i, decl := range []string{
		"const(A=len([iota+1]W{});B;Result)",
		"const(A uint8=cap([iota+1]W{});B;Result)",
		"const(A,B=len([iota+1]W{}),cap([2*iota+1]W{});C,D;_,Result)",
		"const iota=4;const(A=len([iota+1]W{});Result)",
		"const N=3;func Probe(){const(A=len([N+iota]W{});N;Result);_=Result}",
		"type T=[3]W;func Probe(){const(A=len(T{});T;Result);_=Result}",
		"var a[3]W;func Probe(){const(A=len(a);a;Result);_=Result}",
		"var a[3]W;const(A=len(a);B=1;Result=A+B)",
		"type A [3]W;type B=A;const Result=len(B{})",
		"type A [3]W;const Result A=len(A{})",
		"var a[256]W;const Result uint8=len(a)",
		"var a[255]W;const Result uint8=cap(a)",
		"var a[3]W;const Result int8=len(a)+1",
		"var a[3]W;const Result=W(len(a))",
		"const Result=len(a);var a[Result]W",
		"const Result=cap(a);var a[Result]W",
		"const Result=len(a);var a=[Result]W{}",
		"const Result=len(a);type A=[Result]W;var a A",
		"const Result=len(a);type A[Result]W;var a A",
		"const Result=len(a);func Get()[Result]W;var a=Get()",
		"const Result=len(a);type R struct{};func(R)Get()[Result]W;var a=R{}.Get()",
		"const A=len(b);var a[A]W;const Result=len(a);var b[Result]W",
		"var a=[]W{};const Result=len(a)",
		"var a string;const Result=len(a)",
		"const Result=len(len)",
		"func len(x [3]W)int{return 3};const Result=len([3]W{})",
		"var cap=1;const Result=cap([3]W{})",
		"const Result=len([3]W{},[2]W{})",
		"const Result=cap()",
	} {
		add(fmt.Sprintf("context-cycle-%02d", i), decl)
	}
	helper := "package foreign\ntype W uint16;type A [5]W;type Alias=A;var Value Alias;var Inferred=[7]W{};func Get()[9]W{return [9]W{}};const N=len(Value)\n"
	for i, decl := range []string{
		"import h \"example.com/constant/helper\";const Result=len(h.Value)",
		"import h \"example.com/constant/helper\";const Result=cap(h.Inferred)",
		"import h \"example.com/constant/helper\";var a=h.Get();const Result=len(a)",
		"import h \"example.com/constant/helper\";const Result=len(h.Get())",
		"import h \"example.com/constant/helper\";const Result=len(h.Alias{})",
		"import h \"example.com/constant/helper\";const Result=h.N+1",
		"import . \"example.com/constant/helper\";const Result=len(Value)",
		"import \"example.com/constant/helper\";const Result=cap(foreign.Value)",
	} {
		cases = append(cases, practicalAggregateConstantCase{label: fmt.Sprintf("import-%02d", i), source: "package policy\n" + decl + "\n", helper: helper})
	}
	for i, decl := range []string{
		"var a *[3]byte;const Result=len(a)",
		"var a *[3]byte;const Result=cap(a)",
		"const Result=len([3]float64{})",
		"const Result=len([3]complex64{})",
		"const Result=len([257]byte{})",
	} {
		cases = append(cases, practicalAggregateConstantCase{label: fmt.Sprintf("outside-%02d", i), source: "package policy\n" + decl + "\n", unsupported: true})
	}
	return cases
}
func TestPracticalOriginalAggregateConstantsReference(t *testing.T) {
	if runtime.Version() != practicalCompilerRelease {
		t.Fatal("constant reference requires the pinned compiler", runtime.Version())
	}
	out := os.Getenv("MPK_GO_AGGREGATE_CONSTANTS_NATIVE_OUTPUT")
	if out == "" {
		out = t.TempDir()
	}
	write := func(path string, value any) {
		raw, err := json.MarshalIndent(value, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		file, err := os.OpenFile(path, os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0o600)
		if err != nil {
			t.Fatal(err)
		}
		if _, err = file.Write(append(raw, '\n')); err != nil {
			t.Fatal(err)
		}
		if err = file.Close(); err != nil {
			t.Fatal(err)
		}
	}
	accepted, rejected := 0, 0
	for index, c := range practicalAggregateConstantCases() {
		dir := filepath.Join(out, fmt.Sprintf("case-%05d", index))
		if err := os.Mkdir(dir, 0o700); err != nil {
			t.Fatal(err)
		}
		capability := fmt.Sprintf("aggregate-constant-cases/case-%05d", index)
		module, err := newPracticalSourceBuffer(capability, "go.mod", "go_mod", []byte("module example.com/constant\ngo 1.25.0\n"))
		if err != nil {
			t.Fatal(err)
		}
		root, err := newPracticalSourceBuffer(capability, "policy.go", "go_file", []byte(c.source))
		if err != nil {
			t.Fatal(err)
		}
		buffers := []practicalSourceBuffer{module, root}
		names := map[string]string{"example.com/constant": "policy"}
		fileIDs := map[string]string{"example.com/constant": root.object.ID}
		if c.helper != "" {
			helper, err := newPracticalSourceBuffer(capability, "helper/helper.go", "go_file", []byte(c.helper))
			if err != nil {
				t.Fatal(err)
			}
			buffers = append(buffers, helper)
			names["example.com/constant/helper"] = "foreign"
			fileIDs["example.com/constant/helper"] = helper.object.ID
		}
		packages := map[string]map[string]any{}
		packageIDs := map[string]string{}
		roots := []string{}
		for path, name := range names {
			p, err := practicalTypeKeySeal(practicalTypePackageDomain, map[string]any{"schema": practicalTypePackageSchema, "module_source_object_id": module.object.ID, "import_path": path, "package_name": name, "source_objects": []string{fileIDs[path]}})
			if err != nil {
				t.Fatal(err)
			}
			id := p["id"].(string)
			packages[id] = p
			packageIDs[path] = id
			roots = append(roots, id)
		}
		sort.Strings(roots)
		scalar, err := practicalTypeKeySeal(practicalSourceTypeDomain, map[string]any{"schema": practicalSourceTypeSchema, "kind": "builtin", "name": "bool", "width": nil, "signed": nil})
		if err != nil {
			t.Fatal(err)
		}
		sources := []practicalSourceObject{}
		originals := []map[string]any{}
		for _, b := range buffers {
			sources = append(sources, b.object)
			originals = append(originals, map[string]any{"object": b.object, "raw_hex": hex.EncodeToString(b.data)})
		}
		sort.Slice(sources, func(i, j int) bool { return sources[i].ID < sources[j].ID })
		components := map[string]any{"schema": practicalTypeComponentsSchema, "source_objects": sources, "packages": practicalTypeKeyRows(packages), "declarations": []any{}, "types": []any{scalar}, "underlying": []any{}, "aliases": []any{}, "package_roots": roots, "type_roots": []string{scalar["id"].(string)}}
		hash, err := hashTypedCanonicalJSON(practicalTypeComponentsDomain, components)
		if err != nil {
			t.Fatal(err)
		}
		components["content_sha256"] = hash
		canonical, err := canonicalJSON(components)
		if err != nil {
			t.Fatal(err)
		}
		// All source and candidate bytes exist before the real compiler is called.
		write(filepath.Join(dir, "input.json"), map[string]any{"index": index, "label": c.label, "components_hex": hex.EncodeToString(canonical), "sources": originals, "query_source_object_id": root.object.ID, "query_name": "Result", "outside_aggregate_reader": c.unsupported, "source_typing_or_context_or_program_authority": false})
		fileSet := token.NewFileSet()
		info := map[string]*types.Info{}
		files := map[string]*ast.File{}
		typed := practicalConstantReferenceImports{}
		var failure error
		for _, path := range []string{"example.com/constant/helper", "example.com/constant"} {
			if names[path] == "" {
				continue
			}
			raw := c.source
			if strings.HasSuffix(path, "/helper") {
				raw = c.helper
			}
			file, err := parser.ParseFile(fileSet, path+".go", raw, parser.AllErrors|parser.SkipObjectResolution)
			if err != nil {
				failure = err
				break
			}
			files[path] = file
			facts := &types.Info{Defs: map[*ast.Ident]types.Object{}}
			conf := types.Config{GoVersion: "go1.25", Sizes: types.SizesFor("gc", "amd64"), Importer: typed, Error: func(err error) {
				if failure == nil {
					failure = err
				}
			}}
			pkg, err := conf.Check(path, fileSet, []*ast.File{file}, facts)
			info[path] = facts
			if err != nil {
				failure = err
				break
			}
			typed[path] = pkg
		}
		native := map[string]any{"accepted": failure == nil, "source_typing_or_context_or_program_authority": false}
		if failure != nil {
			native["error"] = failure.Error()
			rejected++
		} else {
			var result *types.Const
			for id, obj := range info["example.com/constant"].Defs {
				if id.Name == "Result" {
					result, _ = obj.(*types.Const)
				}
			}
			if result == nil {
				t.Fatal("missing original result constant", c.label)
			}
			typ := types.Unalias(result.Type())
			kind := "integer"
			var key any
			var underlying any
			basic, _ := typ.Underlying().(*types.Basic)
			if basic == nil {
				t.Fatal("missing scalar underlying type", c.label)
			}
			switch result.Val().Kind() {
			case constant.Bool:
				kind = "bool"
				native["boolean"] = constant.BoolVal(result.Val())
			case constant.String:
				kind = "string"
				native["bytes_hex"] = hex.EncodeToString([]byte(constant.StringVal(result.Val())))
			case constant.Int:
				native["integer_decimal"] = result.Val().ExactString()
			default:
				kind = "outside"
			}
			if basic.Kind() == types.UntypedRune {
				kind = "rune"
			}
			if basic.Info()&types.IsUntyped == 0 {
				builtin := basic.Name()
				if builtin == "byte" {
					builtin = "uint8"
				}
				if builtin == "rune" {
					builtin = "int32"
				}
				underlying = builtin
				if named, ok := typ.(*types.Named); ok {
					obj := named.Obj()
					path := obj.Pkg().Path()
					var spec *ast.TypeSpec
					ast.Inspect(files[path], func(n ast.Node) bool {
						if s, ok := n.(*ast.TypeSpec); ok && info[path].Defs[s.Name] == obj {
							spec = s
						}
						return true
					})
					if spec == nil {
						t.Fatal("missing original named definition", c.label)
					}
					tf := fileSet.File(spec.Pos())
					decl, err := practicalTypeKeySeal(practicalTypeDeclarationDomain, map[string]any{"schema": practicalTypeDeclarationSchema, "package_id": packageIDs[path], "source_object_id": fileIDs[path], "kind": "type", "start_byte": tf.Offset(spec.Pos()), "end_byte": tf.Offset(spec.End()), "name": obj.Name()})
					if err != nil {
						t.Fatal(err)
					}
					key, err = practicalTypeKeySeal(practicalSourceTypeDomain, map[string]any{"schema": practicalSourceTypeSchema, "kind": "nominal", "declaration_id": decl["id"]})
					if err != nil {
						t.Fatal(err)
					}
				} else {
					var width, signed any
					if basic.Info()&types.IsInteger != 0 {
						width = confSize(basic) * 8
						signed = basic.Info()&types.IsUnsigned == 0
					}
					key, err = practicalTypeKeySeal(practicalSourceTypeDomain, map[string]any{"schema": practicalSourceTypeSchema, "kind": "builtin", "name": builtin, "width": width, "signed": signed})
					if err != nil {
						t.Fatal(err)
					}
				}
			}
			for _, field := range []string{"integer_decimal", "boolean", "bytes_hex"} {
				if _, ok := native[field]; !ok {
					native[field] = nil
				}
			}
			native["kind"] = kind
			native["type_key"] = key
			native["underlying_builtin"] = underlying
			accepted++
		}
		write(filepath.Join(dir, "native.json"), native)
	}
	write(filepath.Join(out, "summary.json"), map[string]any{"cases": len(practicalAggregateConstantCases()), "accepted": accepted, "rejected": rejected, "runtime": runtime.Version(), "applications_executed": false})
	t.Logf("actual scalar constant cases %d accepted %d rejected %d", len(practicalAggregateConstantCases()), accepted, rejected)
}
