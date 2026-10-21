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
	"sort"
	"strings"
	"testing"
)

// This importer belongs only to the reference test. Every returned package is
// checked from the case's original retained bytes; no filesystem fallback.
type practicalConstantReferenceImports map[string]*types.Package

func (p practicalConstantReferenceImports) Import(path string) (*types.Package, error) {
	if v := p[path]; v != nil {
		return v, nil
	}
	return nil, fmt.Errorf("reference import absent: %s", path)
}

type practicalConstantReferenceCase struct {
	label, source, helper string
	unsupported           bool
}

func practicalConstantReferenceCases() []practicalConstantReferenceCase {
	cases := []practicalConstantReferenceCase{}
	add := func(label, expr string) {
		cases = append(cases, practicalConstantReferenceCase{label: label, source: "package policy\nconst Result = " + expr + "\n"})
	}
	numbers := []string{"0", "1", "-1", "2", "-2", "2147483647", "2147483648", "4294967295", "4294967296", "-4294967296", "9223372036854775807", "18446744073709551615", "-18446744073709551616", "((1<<96)+17)", "-((1<<127)+9)", "(1<<255)", "((1<<511)-1)"}
	for i, a := range numbers {
		for j, b := range numbers {
			for _, op := range []string{"+", "-", "*", "/", "%", "&", "|", "^", "&^", "==", "!=", "<", "<=", ">", ">="} {
				add(fmt.Sprintf("pair-%02d-%02d-%s", i, j, op), "("+a+")"+op+"("+b+")")
			}
		}
	}
	for i, a := range numbers {
		for _, count := range []string{"0", "1", "7", "8", "31", "32", "63", "64", "127", "255", "511", "512", "1074", "1075", "-1", "(1<<64)"} {
			for _, op := range []string{"<<", ">>"} {
				add(fmt.Sprintf("shift-%02d-%s-%s", i, count, op), "("+a+")"+op+count)
			}
		}
	}
	for _, typ := range []string{"int8", "int16", "int32", "int64", "int", "uint8", "uint16", "uint32", "uint64", "uint", "byte", "rune"} {
		for _, expr := range []string{"0", "1", "-1", "127", "128", "255", "256", "(1<<31)", "(1<<63)", "((1<<64)-1)"} {
			add("convert-"+typ+"-"+expr, typ+"("+expr+")")
		}
		for _, op := range []string{"+", "-", "^"} {
			add("unary-"+op+typ, op+typ+"(1)")
		}
	}
	for _, expr := range []string{
		"0b_1010+0o_17+0x_FF+0_600", "'a'+1", "'é'+1", "'\\xff'", "'\\377'", "'\\u00ff'", "'\\U0001f600'", "^1", "^(1<<511)", "-(1<<511)", "1<<512", "0>>1075", "uint8(255)+1", "int8(-128)/-1", "int8(-128)%-1", "int8(-128)*-1", "int8(-128)>>7", "^uint8(1)", "uint8(1)<<8", "uint64(0)<<1074",
		"true&&false", "!false", "true==false", "false&&(1/0==0)", "true||(1/0==0)", "min(2,'A',3)", "max(2,'A',3)", "min(uint8(2),3)", "max(uint8(2),256)", "min(true,false)", "min()",
		`"\xff"+"\x00"`, "`a\r\nb`", `"é"<"z"`, `"\xff">"�"`, `len("\xff\x00é")`, `string(255)`, `string(-1)`, `string(0xd800)`, `string(0x110000)`, `string(0)`, `string('é')`, `min("a","b","\xff")`, `max("a","b","\xff")`,
	} {
		add("edge-"+expr, expr)
	}
	snippets := []string{
		"const Result string=65",
		"const Result int16=int8(1)",
		"const (A=iota;B;C;Result=C)",
		"const(A,B=iota,1<<iota; C,D; Result=D)",
		"const(_=iota;_=iota;Result=iota)",
		"const(A uint8=iota;B;Result=B)",
		"const(A uint8=255;Result=A+1)",
		"const Huge=1<<100;const Result int8=Huge>>98",
		"type Amount uint16;type Alias=Amount;const Result=Amount(1)+Alias(2)",
		"type Amount uint16;type Other uint16;const Result=Amount(1)+Other(2)",
		"type Amount uint16;type Other uint16;const Result=Other(Amount(1))",
		"type Amount uint16;type Other Amount;const Result=Other(2)",
		"type Amount uint16;const Result uint16=Amount(1)",
		"type Amount uint16;const Result Amount=1",
		"type Amount=uint16;const Result=Amount(1)+uint16(2)",
		"type Text string;const Result=Text(65)",
		"type Text string;const Result Text=65",
		"type Flag bool;const Result=!Flag(true)",
		"type Flag bool;const Result=Flag(true)==Flag(false)",
		"const A=B;const B=A;const Result=A",
		"type A=B;type B=A;const Result=A(1)",
		"const x=5;func f(){const(_=x+iota;x;z);const Result=z;_=Result}",
		"type T uint8;func f(){const(A T=iota;T;Result);_=Result}",
		"const true=false;const Result=true",
		"func f(){const iota=iota;const Result=iota;_=Result}",
		"var variable bool;const Result=false&&variable",
		"func call()bool{return false};const Result=false&&call()",
		"var int=1;const Result=int(2)",
		"const Result=iota",
	}
	for i, s := range snippets {
		cases = append(cases, practicalConstantReferenceCase{label: fmt.Sprintf("declarations-%02d", i), source: "package policy\n" + s + "\n"})
	}
	helper := "package foreign\ntype Amount uint16\nconst Value Amount=19\nconst Bytes=\"\\xff\\x00\"\n"
	for i, s := range []string{
		"import \"example.com/constant/helper\"\nconst Result=foreign.Value+1",
		"import alias \"example.com/constant/helper\"\nconst Result=alias.Amount(7)",
		"import . \"example.com/constant/helper\"\nconst Result=Value+1",
		"import \"example.com/constant/helper\"\nconst Result=len(foreign.Bytes)",
		"import \"example.com/constant/helper\"\nconst Result=foreign.private",
	} {
		cases = append(cases, practicalConstantReferenceCase{label: fmt.Sprintf("imports-%02d", i), source: "package policy\n" + s + "\n", helper: helper})
	}
	for i, expr := range []string{"uint16(1.0)", "len([3]byte{})", "1.5", "1i"} {
		cases = append(cases, practicalConstantReferenceCase{label: fmt.Sprintf("outside-reader-%02d", i), source: "package policy\nconst Result=" + expr + "\n", unsupported: true})
	}
	return cases
}
func TestPracticalScalarConstantReference(t *testing.T) {
	out := os.Getenv("MPK_GO_CONSTANT_NATIVE_OUTPUT")
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
	for index, c := range practicalConstantReferenceCases() {
		dir := filepath.Join(out, fmt.Sprintf("case-%05d", index))
		if err := os.Mkdir(dir, 0o700); err != nil {
			t.Fatal(err)
		}
		capability := fmt.Sprintf("constant-cases/case-%05d", index)
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
		components := map[string]any{"schema": practicalTypeComponentsSchema, "source_objects": sources, "packages": practicalTypeKeyRows(packages), "declarations": []any{}, "types": []any{scalar}, "underlying": []any{}, "aliases": []any{}, "package_roots": roots, "type_roots": []string{scalar["id"]}}
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
		write(filepath.Join(dir, "input.json"), map[string]any{"index": index, "label": c.label, "components_hex": hex.EncodeToString(canonical), "sources": originals, "query_source_object_id": root.object.ID, "query_name": "Result", "outside_scalar_reader": c.unsupported, "source_typing_or_context_or_program_authority": false})
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
	write(filepath.Join(out, "summary.json"), map[string]any{"cases": len(practicalConstantReferenceCases()), "accepted": accepted, "rejected": rejected, "runtime": "go1.25.0", "applications_executed": false})
	t.Logf("actual scalar constant cases %d accepted %d rejected %d", len(practicalConstantReferenceCases()), accepted, rejected)
}
func confSize(basic *types.Basic) int64 { return types.SizesFor("gc", "amd64").Sizeof(basic) }
