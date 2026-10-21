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

type practicalValueTypingCase struct {
	label, expr, target, setup, body, mode string
	outside                                bool
	helper                                 bool
}

func practicalValueTypingCases() []practicalValueTypingCase {
	cases := []practicalValueTypingCase{}
	add := func(label, expr, target string) {
		cases = append(cases, practicalValueTypingCase{label: label, expr: expr, target: target, mode: "value"})
	}
	for _, base := range []string{"a", "raw", "sl", "words", "nums", "Array{}", "Vec{}", "[3]Word{}", "[]Word{}", "\"aé\\xff\"", "text", "str"} {
		for _, index := range []string{"0", "1", "2", "3", "-1", "i", "u", "uint64(1)", "((1<<63)-1)", "(1<<63)", "((1<<64)-1)", "true", "\"x\"", "Word(1)", "1<<u"} {
			add("index-"+base+"-"+index, base+"["+index+"]", "")
		}
		for _, bounds := range []string{":", "0:", ":0", "0:2", "1:3", "3:3", "0:4", "-1:2", "2:1", "i:u", "u:i", "i:", ":u", ":(1<<63)", "1:2:3", ":2:3", "1:3:2", "0:3:4", "i:u:u", "0:0:0", "false:1"} {
			add("slice-"+base+"-"+bounds, base+"["+bounds+"]", "")
		}
		for _, fun := range []string{"len", "cap"} {
			add(fun+"-"+base, fun+"("+base+")", "")
		}
	}
	for _, base := range []string{"rec", "Record{}", "direct", "diamond", "rival", "private", "foreign", "h.Wrapper{}"} {
		for _, field := range []string{"X", "Y", "Inner", "Left", "Right", "hidden", "Exposed", "_", "Missing", "Method"} {
			cases = append(cases, practicalValueTypingCase{label: "field-" + base + "-" + field, expr: base + "." + field, mode: "value", helper: true, outside: field == "Method"})
		}
	}
	for _, expr := range []string{
		"Array{}", "Array{1,2,3}", "Array{3:1}", "Array{2:1}", "Array{1:1,2}", "Array{1:1,1:2}", "Array{2:1,2}", "Array{1,-1}", "Array{65535,65536}", "Array{Word(1),uint16(2)}",
		"Vec{}", "Vec{1:1,2}", "Vec{-1:0}", "Vec{1:0,1:1}", "Vec{i:1}", "Vec{(1<<63):1}", "Vec{128:1}", "Vec{256:1}",
		"[...]Word{}", "[...]Word{1,2,3}", "[...]Word{255:1}", "[...]Word{256:1}", "[...]Word{3:1,1:2,3}", "[257]byte{}",
		"Record{}", "Record{X:1,Y:2}", "Record{1,2}", "Record{1}", "Record{X:1,2}", "Record{1,Y:2}", "Record{X:1,X:2}", "Record{Missing:1}", "Record{X:65536}", "Record{X:uint16(1)}",
		"Direct{X:1}", "Direct{Y:1}", "Direct{Left:Left{Inner:Inner{X:1}}}", "Direct{Left:{Inner:{X:1}}}", "Blank{1,2}", "Blank{_:1}", "Blank{}",
		"[2]Record{{1,2},{X:3}}", "[]Record{{X:1},{Y:2}}", "[][2]Word{{1,2},{2,3}}", "[][]Word{{1,2},{3}}", "[]Record{{Bad:1}}", "[]Record{{X:65536}}", "[]Record{{X:{1}}}",
		"a==raw", "a==other", "a==Array{}", "a==[3]Word{}", "a==[3]uint16{}", "rec==Record{}", "rec==OtherRecord{}", "rec==struct{X Word;Y uint64}{}", "sl==words", "sl==nil", "nil==sl", "sl!=nil", "nil==nil", "sl==Vec(nil)", "HasSlice{}==HasSlice{}", "HasSlice{}==nil", "sl<nil", "bad==bad", "err==nil", "nil==err", "err==err",
		"[]Byte(text)", "[]Rune(text)", "Text([]Byte{})", "string([]Rune{})", "([]Word)(sl)", "([3]Word)(a)", "(Array)(raw)", "Vec{((1<<63)-1):1}", "Array(raw)", "OtherArray(a)", "[3]Word(a)", "[]Word(sl)", "OtherVec(sl)", "[3]Word(sl)", "[3]uint16(sl)", "Record(rec)", "OtherRecord(rec)", "Tagged(rec)", "EmbedA(EmbedB{})", "DifferentInner(rec)", "[]byte(text)", "[]rune(text)", "[]Word(text)", "Text([]byte{})", "string([]rune{})", "string(sl)", "Vec(nil)", "Array(nil)", "error(nil)",
		"len(Array{Id(1),2,3})", "cap(Array{Id(1),2,3})", "len(Array{min(1,2)})", "len(Array{min(i,2)})", "len(GetArray())", "cap(GetArray())", "len(Array{Word(i)})", "len(Array{Word(1)})", "len([2]string{string(i)})", "len(Array{Word(len(sl))})", "len(Array{Word(len(Array{}))})", "len(\"\\xffé\")",
		"GetArray()", "GetSlice()", "GetRecord()", "TakeArray(raw)", "TakeArray(other)", "TakeArray([3]Word{})", "TakeSlice(sl)", "TakeSlice(words)", "TakeSlice(OtherVec{})", "TakeRecord(Record{})", "TakeRecord(OtherRecord{})", "TakeRecord(struct{X Word;Y uint64}{})", "GetArray()[1]", "GetSlice()[1]", "GetRecord().X", "GetArray()[:]",
	} {
		outside := expr == "[...]Word{256:1}" || expr == "[257]byte{}"
		cases = append(cases, practicalValueTypingCase{label: "edge-" + expr, expr: expr, mode: "value", outside: outside})
	}
	for _, target := range []string{"Array", "OtherArray", "[3]Word", "[3]uint16", "Vec", "OtherVec", "[]Word", "[]uint16", "Record", "OtherRecord", "struct{X Word;Y uint64}", "Tagged", "error"} {
		for _, expr := range []string{"a", "raw", "other", "sl", "words", "nums", "rec", "nil", "Array{}", "[]Word{}", "struct{X Word;Y uint64}{}"} {
			add("assignment-"+target+"-"+expr, expr, target)
		}
	}
	for _, body := range []string{
		"var Result Array;_=Result", "var Result=Array{1};_=Result", "var Result [3]Word=a;_=Result", "var Result OtherArray=a;_=Result", "var Result Vec=nil;_=Result", "var Result error=nil;_=Result", "var Result=nil;_=Result",
		"var x,Result=PairAggregate();_=x;_=Result", "var Result,x=PairAggregate();_=x;_=Result", "var x,Result Array=PairAggregate();_=x;_=Result", "var x,Result uint8=1,256;_=x;_=Result",
		"var x Array; x,Result:=GetArray(),1;_=x;_=Result", "var x OtherArray; x,Result:=GetArray(),1;_=x;_=Result", "var x []Word; x,Result:=GetSlice(),1;_=x;_=Result",
		"for Result:=range a{_=Result}", "for _,Result:=range a{_=Result}", "for Result:=range sl{_=Result}", "for _,Result:=range sl{_=Result}", "for _,Result:=range [0]Word{}{_=Result}", "for _,Result:=range [2]Record{}{_=Result}", "for _,Result:=range [2][]Word{}{_=Result}", "for _,Result:=range \"abc\"{_=Result}",
	} {
		cases = append(cases, practicalValueTypingCase{label: "binding-" + body, body: body, mode: "binding", outside: strings.Contains(body, "range \"abc\"")})
	}
	for _, expr := range []string{"PairAggregate()", "VoidAggregate()"} {
		cases = append(cases, practicalValueTypingCase{label: "tuple-" + expr, expr: expr, mode: "tuple"})
	}
	for _, expr := range []string{"h.Record{Exposed:1}", "h.Record{hidden:1}", "h.Record{1,2}", "h.Record{}", "h.Record{Exposed:1}.Exposed", "h.Wrapper{}.Exposed", "h.Wrapper{}.hidden", "h.Wrapper{Record:{Exposed:1}}", "h.Wrapper{Exposed:1}"} {
		cases = append(cases, practicalValueTypingCase{label: "import-" + expr, expr: expr, mode: "value", helper: true})
	}
	for _, depth := range []int{16, 17} {
		setup := "type T0=Word;"
		for n := 1; n <= depth; n++ {
			setup += fmt.Sprintf("type T%d=[1]T%d;", n, n-1)
		}
		cases = append(cases, practicalValueTypingCase{label: fmt.Sprintf("depth-%d", depth), expr: fmt.Sprintf("T%d{}", depth), setup: setup, mode: "value", outside: depth == 17})
	}
	for _, depth := range []int{15, 16} {
		setup := "type T0=Word;"
		for n := 1; n <= depth; n++ {
			setup += fmt.Sprintf("type T%d=[1]T%d;", n, n-1)
		}
		cases = append(cases, practicalValueTypingCase{label: fmt.Sprintf("ellipsis-depth-%d", depth), expr: fmt.Sprintf("[...]T%d{}", depth), setup: setup, mode: "value", outside: depth == 16})
	}
	for _, fields := range []int{64, 65} {
		setup := "type Wide struct{"
		for n := 0; n < fields; n++ {
			setup += fmt.Sprintf("F%d Word;", n)
		}
		setup += "};"
		cases = append(cases, practicalValueTypingCase{label: fmt.Sprintf("fields-%d", fields), expr: "Wide{}", setup: setup, mode: "value", outside: fields == 65})
	}
	for _, expr := range []string{"method.X", "conflict.X", "method.Method", "method.Y"} {
		cases = append(cases, practicalValueTypingCase{label: "method-barrier-" + expr, expr: expr, mode: "value", setup: "var method MethodLeft;var conflict struct{Left;MethodLeft};_=method;_=conflict;", outside: strings.Contains(expr, "Method") || expr == "method.X"})
	}
	return cases
}
func TestPracticalOriginalValueTypingReference(t *testing.T) {
	if runtime.Version() != practicalCompilerRelease {
		t.Fatal(runtime.Version())
	}
	out := os.Getenv("MPK_GO_VALUE_TYPING_NATIVE_OUTPUT")
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
	for index, c := range practicalValueTypingCases() {
		dir := filepath.Join(out, fmt.Sprintf("case-%05d", index))
		if err := os.Mkdir(dir, 0o700); err != nil {
			t.Fatal(err)
		}
		source := "package policy\n"
		helper := ""
		if c.helper {
			source += "import h \"example.com/scalar/helper\"\n"
			helper = "package foreign\ntype Record struct{Exposed uint16;hidden uint16}\ntype Wrapper struct{Record}\n"
		}
		source += `type Word uint16
type Text string
type Byte uint8
type Rune int32
type Array [3]Word
type OtherArray [3]Word
type Vec []Word
type OtherVec []Word
type Record struct{X Word;Y uint64}
type OtherRecord struct{X Word;Y uint64}
type Tagged struct{X Word ` + "`json:\"x\"`" + `;Y uint64}
type DifferentInner struct{X uint16;Y uint64}
type Inner struct{X Word;hidden uint64}
func (Inner) Method()Word{return 0}
type MethodLeft struct{Inner}
func (MethodLeft) X()Word{return 0}
type Left struct{Inner}
type Right struct{Inner}
type Diamond struct{Left;Right}
type Direct struct{Left;X uint64}
type Rival struct{Inner;X uint64}
type Blank struct{X Word;_ uint8}
type HasSlice struct{S []Word}
type EmbedA struct{Inner}
type EmbedB struct{Inner Inner}
func Id(x Word)Word{return x}
func GetArray()Array{return Array{}}
func GetSlice()Vec{return nil}
func GetRecord()Record{return Record{}}
func TakeArray(x Array)Array{return x}
func TakeSlice(x Vec)Vec{return x}
func TakeRecord(x Record)Record{return x}
func PairAggregate()(Array,Vec){return Array{},nil}
func VoidAggregate(){}
`
		args := "a Array,raw [3]Word,other OtherArray,sl Vec,words []Word,nums []uint16,i int,u uint64,text Text,str string,rec Record,direct Direct,diamond Diamond,rival Rival,private Inner,bad HasSlice,err error"
		if c.helper {
			args += ",foreign h.Record"
		}
		source += "func Probe(" + args + "){\n" + c.setup
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
		cap := fmt.Sprintf("value-typing/case-%05d", index)
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
		write(filepath.Join(dir, "input.json"), map[string]any{"index": index, "label": c.label, "components_hex": hex.EncodeToString(canonical), "sources": originals, "query_source_object_id": root.object.ID, "query_mode": c.mode, "expression_start_byte": start, "expression_end_byte": end, "target_type": c.target, "outside_value_typing": c.outside, "source_typing_or_context_or_program_authority": false})
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
			values := []map[string]any{}
			add := func(typ types.Type, value constant.Value, addressable bool) {
				untypedNil := typ == types.Typ[types.UntypedNil]
				if untypedNil {
					var target types.Type
					for ident, obj := range info.Defs {
						if ident.Name == "Result" && obj != nil {
							if target != nil {
								t.Fatal("ambiguous nil destination")
							}
							target = obj.Type()
						}
					}
					if target == nil {
						t.Fatal("missing nil destination")
					}
					typ = target
				}
				typeKey := key(typ)
				var builtin any
				if basic, ok := typ.Underlying().(*types.Basic); ok {
					builtin = types.Typ[basic.Kind()].Name()
				}
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
				values = append(values, map[string]any{"type_key": typeKey, "underlying_builtin": builtin, "constant": value != nil, "integer_decimal": integer, "boolean": boolean, "bytes_hex": bytesHex, "addressable": addressable, "untyped_nil": untypedNil})
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
	write(filepath.Join(out, "summary.json"), map[string]any{"cases": len(practicalValueTypingCases()), "accepted": accepted, "rejected": rejected, "runtime": runtime.Version(), "applications_executed": false})
	t.Logf("actual value typing cases %d accepted %d rejected %d", len(practicalValueTypingCases()), accepted, rejected)
}
