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
"runtime/debug"
	"sort"
	"strings"
	"testing"
)

type practicalSliceBuiltinCase struct {
	label, expr, target, setup, body, mode string
	global                                 string
	outside                                bool
	helper                                 bool
}

func practicalSliceBuiltinCases() []practicalSliceBuiltinCase {
	cases := []practicalSliceBuiltinCase{}
	add := func(label, expr, target string) {
		cases = append(cases, practicalSliceBuiltinCase{label: label, expr: expr, target: target, mode: "value"})
	}
	sizes := []string{"0", "1", "-1", "127", "128", "256", "257", "((1<<63)-1)", "(1<<63)", "uint64(1)", "uint64(1<<63)", "i", "u", "1<<u", "((1<<63)<<u)", "('a'<<40)", "('a'<<u)", "true", "\"x\""}
	for _, typ := range []string{"Vec", "AliasVec", "[]Word", "[]Record", "[][]Word", "Bytes", "[]Byte", "[3]Word", "Word", "error"} {
		for _, n := range sizes {
			add("make-"+typ+"-"+n, "make("+typ+","+n+")", "")
			for _, cap := range []string{"0", "2", "i", "u"} {
				add("make-"+typ+"-"+n+"-"+cap, "make("+typ+","+n+","+cap+")", "")
			}
		}
	}
	for _, base := range []string{"sl", "words", "otherSl", "[]Word{}", "Vec(nil)", "nil", "a", "bytes", "namedBytes", "definedBytes", "records", "slices", "errs"} {
		add("append-zero-"+base, "append("+base+")", "")
		for _, value := range []string{"x", "Word(1)", "uint16(1)", "1", "-1", "65535", "65536", "true", "\"x\"", "nil", "1<<u", "((1<<63)<<u)", "Record{}", "OtherRecord{}", "struct{X Word;Y uint64}{}", "[]Word{}", "Vec{}", "sl"} {
			add("append-one-"+base+"-"+value, "append("+base+","+value+")", "")
		}
		for _, value := range []string{"sl", "words", "otherSl", "nums", "nil", "bytes", "namedBytes", "definedBytes", "str", "text", "\"é\\xff\"", "[]Byte{}", "records", "slices", "errs"} {
			add("append-spread-"+base+"-"+value, "append("+base+","+value+"...)", "")
		}
	}
	for _, dst := range []string{"sl", "words", "otherSl", "a", "nil", "bytes", "namedBytes", "definedBytes", "records", "slices", "errs"} {
		for _, src := range []string{"sl", "words", "otherSl", "nums", "nil", "bytes", "namedBytes", "definedBytes", "str", "text", "\"é\\xff\"", "[]Byte{}", "records", "slices", "errs", "a", "0"} {
			add("copy-"+dst+"-"+src, "copy("+dst+","+src+")", "")
		}
	}
	for _, expr := range []string{
		"make()", "make(Vec)", "make(Vec,0,1,2)", "make(Vec,1,0)", "make(Vec,0,2...)", "make(Vec,PairSizes())", "make((Vec),0,0)", "make(map[string]Word)", "make(chan Word,1)", "make([]float64,0,1)",
		"append()", "append(nil,1)", "append(sl,1,2,3)", "append(sl,1,uint16(2))", "append(sl,nil...)", "append(sl,1,sl...)", "append(sl,GetSlice()...)", "append(sl,GetSlice())", "append(sl,[]Word{}...)", "append(sl,1<<u,65536)", "append(sl,{1})",
		"append(PairSliceWord())", "append(PairSliceWord()...)", "append(PairSlices())", "append(PairSlices()...)", "append(PairBytesString())", "append(PairBytesString()...)", "append(GetSlice())", "append(PairSliceThree())", "append(VoidAggregate())",
		"copy()", "copy(sl)", "copy(sl,sl,sl)", "copy(sl,sl...)", "copy(PairSlices())", "copy(PairSliceWord())", "copy(PairBytesString())", "copy(PairSliceThree())", "copy(sl,PairSlices())", "copy(VoidAggregate())",
		"append(make(Vec,0,3),1)", "copy(make(Vec,1,3),append(sl,1))", "len(append(sl,1))", "cap(make(Vec,i,u))", "len([2]int{copy(sl,sl)})", "len([2]Vec{make(Vec,0,1)})", "len([2]Vec{append(sl)})", "make([][256]Word,0,1)",
	} {
		outside := expr == "make(map[string]Word)" || expr == "make(chan Word,1)" || expr == "make([]float64,0,1)"
		cases = append(cases, practicalSliceBuiltinCase{label: "edge-" + expr, expr: expr, mode: "value", outside: outside})
	}
	for _, target := range []string{"Vec", "OtherVec", "[]Word", "int", "Word", "uint64"} {
		for _, expr := range []string{"make(Vec,0,1)", "append(sl,1)", "copy(sl,sl)"} {
			add("target-"+target+"-"+expr, expr, target)
		}
	}
	for _, body := range []string{"var Result=make(Vec,1,2);_=Result", "var Result=append(sl,1);_=Result", "var Result=copy(sl,sl);_=Result", "var a,Result=copy(sl,sl),append(sl,1);_=a;_=Result", "var Result,tail uint8=copy(sl,sl),0;_=Result;_=tail", "var old OtherVec;old,Result:=append(sl,1),copy(sl,sl);_=old;_=Result"} {
		cases = append(cases, practicalSliceBuiltinCase{label: "binding-" + body, body: body, mode: "binding"})
	}
	for _, expr := range []string{"make(h.Vec,0,1)", "append(foreignSlice,1)", "append(foreignSlice,Word(1))", "copy(foreignSlice,foreignSlice)"} {
		cases = append(cases, practicalSliceBuiltinCase{label: "import-" + expr, expr: expr, mode: "value", helper: true})
	}
	for _, name := range []string{"make", "append", "copy"} {
		cases = append(cases, practicalSliceBuiltinCase{label: "shadow-" + name, expr: name + "(1)", mode: "value", global: "func " + name + "(x int)int{return x};\n"})
	}
	parts := []string{}
	for n := 0; n < 200; n++ {
		parts = append(parts, "1")
	}
	add("append-200-original-arguments", "append(sl,"+strings.Join(parts, ",")+")", "")
	return cases
}
func TestPracticalOriginalSliceBuiltinReference(t *testing.T) {
	if runtime.Version() != practicalCompilerRelease {
		t.Fatal(runtime.Version())
	}
	out := os.Getenv("MPK_GO_SLICE_BUILTIN_NATIVE_OUTPUT")
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
	accepted, rejected, panicked := 0, 0, 0
	for index, c := range practicalSliceBuiltinCases() {
		dir := filepath.Join(out, fmt.Sprintf("case-%05d", index))
		if err := os.Mkdir(dir, 0o700); err != nil {
			t.Fatal(err)
		}
		source := "package policy\n"
		helper := ""
		if c.helper {
			source += "import h \"example.com/scalar/helper\"\n"
			helper = "package foreign\ntype Record struct{Exposed uint16;hidden uint16}\ntype Wrapper struct{Record}\ntype Word uint16\ntype Vec []Word\n"
		}
		source += `type Word uint16
type Text string
type Byte uint8
type Rune int32
type Array [3]Word
type OtherArray [3]Word
type AliasVec=Vec
type Bytes []byte
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
func PairSizes()(int,int){return 1,2}
func PairSliceWord()(Vec,Word){return nil,1}
func PairSlices()(Vec,OtherVec){return nil,nil}
func PairBytesString()(Bytes,Text){return nil,"a"}
func PairSliceThree()(Vec,Word,Word){return nil,1,2}
func VoidAggregate(){}
`
		source += c.global
		args := "a Array,raw [3]Word,other OtherArray,sl Vec,words []Word,nums []uint16,i int,u uint64,text Text,str string,rec Record,direct Direct,diamond Diamond,rival Rival,private Inner,bad HasSlice,err error,otherSl OtherVec,bytes []byte,namedBytes Bytes,definedBytes []Byte,records []Record,slices [][]Word,errs []error,x Word"
		if c.helper {
			args += ",foreign h.Record,foreignSlice h.Vec"
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
		cap := fmt.Sprintf("slice-builtins/case-%05d", index)
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
        var nativePanic map[string]any
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
			var pkg *types.Package
            func(){
                defer func(){if recovered:=recover();recovered!=nil{nativePanic=map[string]any{"message":fmt.Sprint(recovered),"stack":string(debug.Stack())}}}()
                pkg,err=config.Check(path,set,[]*ast.File{file},info)
            }()
            if nativePanic!=nil{break}
			if err != nil {
				failure = err
				break
			}
			typed[path] = pkg
			infos[path] = info
			files[path] = file
		}
		result := map[string]any{"accepted": failure == nil&&nativePanic==nil, "initializer_body_or_invocation_state_or_program_authority": false}
		if nativePanic!=nil{result["native_panic"]=nativePanic;panicked++} else if failure != nil {
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
	write(filepath.Join(out, "summary.json"), map[string]any{"cases": len(practicalSliceBuiltinCases()), "accepted": accepted, "rejected": rejected, "runtime"
"runtime/debug": runtime.Version(), "applications_executed": false})
	t.Logf("actual slice builtin cases %d accepted %d rejected %d panicked %d", len(practicalSliceBuiltinCases()), accepted, rejected,panicked)
}
