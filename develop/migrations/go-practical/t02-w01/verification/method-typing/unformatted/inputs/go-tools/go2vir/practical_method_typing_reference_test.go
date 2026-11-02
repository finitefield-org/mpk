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

type practicalMethodTypingCase struct {
	label, expr, target, setup, body, mode string
	global                                 string
	outside                                bool
	helper                                 bool
}

func practicalMethodTypingCases() []practicalMethodTypingCase {
 cases:=[]practicalMethodTypingCase{}
 add:=func(label,expr,target string){cases=append(cases,practicalMethodTypingCase{label:label,expr:expr,target:target,mode:"value"})}
 for _,base:=range []string{"code","Code(1)","Alias(1)","Other(1)","good","Good{}","emb","Embed{}","dia","over","aliasHost","err","ifwrap","ptrAlias","pErr"}{
  for _,call:=range []string{"Error()","Mix(code,n)","Mix(PairArgs())","Mix(1<<n,n)","Mix(uint16(1),n)","Mix(65536,n)","Mix()","Mix(code)","Mix(code,n,code)","Pair(code)","Void()","Opaque()","hidden()","Missing()"}{
   expr:=base+"."+call;outside:=base=="err"||base=="ifwrap"||base=="ptrAlias"||base=="pErr"
   cases=append(cases,practicalMethodTypingCase{label:"bound-"+expr,expr:expr,mode:"value",outside:outside})
  }
 }
 for _,typ:=range []string{"Code","Alias","Other","Good","AliasGood","OtherGood","Embed","Diamond","Override","AliasHost","error"}{
  for _,call:=range []string{"Error(good)","Error(code)","Mix(good,code,n)","Mix(emb,code,n)","Mix(plain,code,n)","Mix(PairReceiverArgs())","Void(good)","Pair(good,code)","Opaque(good)"}{cases=append(cases,practicalMethodTypingCase{label:"expression-"+typ+"."+call,expr:typ+"."+call,mode:"value",outside:typ=="error"})}
 }
 for _,expr:=range []string{"code","Code(1)","good","Good{}","AliasHost{}","emb","Embed{}","Diamond{}","Override{}","WrongOverride{}","BadText(0)","BadArgs(0)","BadResults(0)","pErr","&pErr","ptrAlias","ifwrap","InterfaceWrap{}","struct{Good}{Good{}}","struct{N Code}{}","ErrSlice{}","ErrArray{}","nil","err"}{
  for _,target:=range []string{"error","ErrorAlias"}{cases=append(cases,practicalMethodTypingCase{label:"error-assignment-"+expr+"-"+target,expr:expr,target:target,mode:"value",outside:expr=="&pErr"})}
  for _,prefix:=range []string{"error(","WantError("}{cases=append(cases,practicalMethodTypingCase{label:"error-call-"+prefix+expr+")",expr:prefix+expr+")",mode:"value",outside:expr=="&pErr"})}
 }
 for _,expr:=range []string{"code","good","Good{}","emb","dia","over","BadText(0)","pErr","ErrSlice{}","error(ErrSlice{})","error(Code(1))","nil","err"}{add("error-compare-"+expr,"err==("+expr+")","")}
 for _,expr:=range []string{"code.Error","Code.Error","good.Mix","Good.Mix","err.Error"}{cases=append(cases,practicalMethodTypingCase{label:"method-value-"+expr,expr:expr,mode:"value",outside:true})}
 for _,expr:=range []string{"good.Pair(code)","Good.Pair(good,code)","good.Void()","Good.Void(good)","good.Many16()"}{cases=append(cases,practicalMethodTypingCase{label:"tuple-"+expr,expr:expr,mode:"tuple"})}
 for _,expr:=range []string{"(good.Mix)(code,n)","(Good.Mix)(good,code,n)","GetGood().Mix(code,n)","PairGood().Mix(code,n)","Code.Add(1,1<<n)","code.Add(1<<n)","code.Add(65536)","Code.Add(raw,code)","(Alias)(1).Error()","error(Code(1))==err","error(ErrSlice{})==nil","good.P()","Good{}.P()","(*Good).P(&good)","GoAliasPtr{}.Error()"}{outside:=expr=="good.P()"||expr=="(*Good).P(&good)";cases=append(cases,practicalMethodTypingCase{label:"edge-"+expr,expr:expr,mode:"value",outside:outside})}
 for _,body:=range []string{"var Result error=Code(1);_=Result","var Result error=good;_=Result","var Result=good.Mix(code,n);_=Result","var a,Result=good.Pair(code);_=a;_=Result","var Result,tail=good.Pair(code);_=tail;_=Result","var a,Result error=PairGood();_=a;_=Result","var a,Result error=PairMixed();_=a;_=Result","var Result error=ifwrap;_=Result","var a error; a,Result:=good,code;_=a;_=Result","var a Good;a,Result:=err,code;_=a;_=Result"}{cases=append(cases,practicalMethodTypingCase{label:"binding-"+body,body:body,mode:"binding"})}
 for _,expr:=range []string{"foreign.Error()","foreign.Value(raw)","foreign.secret()","h.Good.Value(foreign,raw)","WantError(foreign)","error(foreign)"}{cases=append(cases,practicalMethodTypingCase{label:"import-"+expr,expr:expr,mode:"value",helper:true})}
 params:=[]string{};args:=[]string{};for n:=0;n<70;n++{params=append(params,fmt.Sprintf("p%d Code",n));args=append(args,"1")};cases=append(cases,practicalMethodTypingCase{label:"method-70-original-inputs",expr:"good.Many("+strings.Join(args,",")+")",mode:"value",global:"func(Good)Many("+strings.Join(params,",")+")Code{return p0};\n"})
 parts:=[]string{};for n:=0;n<17;n++{parts=append(parts,fmt.Sprintf("r%d Code",n))};cases=append(cases,practicalMethodTypingCase{label:"method-results17",expr:"good.Many17()",mode:"tuple",outside:true,global:"func(Good)Many17()("+strings.Join(parts,",")+"){return};\n"})
 return cases
}
func TestPracticalOriginalMethodTypingReference(t *testing.T) {
	if runtime.Version() != practicalCompilerRelease {
		t.Fatal(runtime.Version())
	}
	out := os.Getenv("MPK_GO_METHOD_TYPING_NATIVE_OUTPUT")
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
	for index, c := range practicalMethodTypingCases() {
		dir := filepath.Join(out, fmt.Sprintf("case-%05d", index))
		if err := os.Mkdir(dir, 0o700); err != nil {
			t.Fatal(err)
		}
		source := "package policy\n"
		helper := ""
		if c.helper {
			source += "import h \"example.com/scalar/helper\"\n"
            helper = `package foreign
type Good struct{N uint16}
func(Good)Error()string{return "foreign"}
func(g Good)Value(x uint16)uint16{return g.N+x}
func(Good)secret()uint16{return 0}
`
		}
        source+=`type Code uint16
type Alias=Code
type Other Code
type Text string
type ErrorAlias=error
type Good struct{N Code}
type AliasGood=Good
type OtherGood Good
func(c Code)Add(x Code)Code{return c+x}
func(Code)Error()string{return "code"}
func(g Good)Mix(x Code,y uint64)Code{return g.N+x}
func(Good)Error()string{return "good"}
func(Good)Pair(x Code)(Code,uint64){return x,0}
func(Good)Void(){}
func(Good)Opaque()Code
func(Good)hidden()Code{return 0}
func(*Good)P()Code{return 0}
type BadText uint16
func(BadText)Error()Text{return "bad"}
type BadArgs uint16
func(BadArgs)Error(int)string{return "bad"}
type BadResults uint16
func(BadResults)Error()(string,string){return "bad","bad"}
type PErr struct{N Code}
func(*PErr)Error()string{return "pointer"}
type Embed struct{Good}
type Left struct{Good}
type Right struct{Good}
type Diamond struct{Left;Right}
type Override struct{Good;Error uint64}
type WrongOverride struct{Good}
func(WrongOverride)Error()Text{return "wrong"}
type AliasHost struct{N Code}
type ReceiverName=AliasHost
func(ReceiverName)Error()string{return "alias"}
type GoAliasPtr struct{Good}
type PointerName=*GoAliasPtr
func(PointerName)Error()string{return "pointer alias"}
type InterfaceWrap struct{error}
type ErrSlice []Code
func(ErrSlice)Error()string{return "slice"}
type ErrArray [2]Code
func(ErrArray)Error()string{return "array"}
func PairArgs()(Code,uint64){return 1,2}
func PairReceiverArgs()(Good,Code,uint64){return Good{},1,2}
func PairGood()(Good,Good){return Good{},Good{}}
func PairMixed()(Good,Code){return Good{},1}
func GetGood()Good{return Good{}}
func WantError(error)Code{return 0}
`
        parts:=[]string{};for n:=0;n<16;n++{parts=append(parts,fmt.Sprintf("r%d Code",n))};source+="func(Good)Many16()("+strings.Join(parts,",")+"){return}\n"
		source += c.global
        args:="code Code,good Good,plain struct{N Code},emb Embed,dia Diamond,over Override,aliasHost AliasHost,ptrAlias GoAliasPtr,ifwrap InterfaceWrap,pErr PErr,err error,n uint64,raw uint16"
        if c.helper{args+=",foreign h.Good"}
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
		cap := fmt.Sprintf("method-typing/case-%05d", index)
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
	write(filepath.Join(out, "summary.json"), map[string]any{"cases": len(practicalMethodTypingCases()), "accepted": accepted, "rejected": rejected, "runtime": runtime.Version(), "applications_executed": false})
	t.Logf("actual method typing cases %d accepted %d rejected %d", len(practicalMethodTypingCases()), accepted, rejected)
}
