//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"go/ast"
	"go/types"
	"runtime"
	"sort"
	"strings"
	"testing"
	"unicode"
	"unicode/utf8"
)

// Only the native reference reads Go object maps. The consumer receives the
// whole original source universe and compares independently derived keys.
func practicalSignatureReferenceKey(owner *practicalOriginalPackages, typ types.Type) (string, error) {
	native := newPracticalStorageTypes(owner)
	root, err := native.capture(typ)
	if err != nil {
		return "", err
	}
	b := practicalTypeKeyBuilder{owner: owner, sources: map[string]practicalSourceBuffer{}, packages: map[string]map[string]any{}, packageIDs: map[string]string{}}
	keys := make([]string, len(native.nodes))
	for i, node := range native.nodes {
		row := map[string]any{"schema": practicalSourceTypeSchema}
		switch node.kind {
		case "alias", "builtin_alias":
			keys[i] = keys[node.child]
			continue
		case "named":
			d := native.declarations[node.declaration]
			pkg, err := b.packageKey(d.packagePath)
			if err != nil {
				return "", err
			}
			declaration, err := practicalTypeKeySeal(practicalTypeDeclarationDomain, map[string]any{"schema": practicalTypeDeclarationSchema, "package_id": pkg, "source_object_id": d.span.sourceID, "kind": "type", "start_byte": d.span.start, "end_byte": d.span.end, "name": d.name})
			if err != nil {
				return "", err
			}
			row["kind"] = "nominal"
			row["declaration_id"] = declaration["id"]
		case "bool", "string", "word", "error":
			name := node.builtin
			if node.kind == "error" {
				name = "error"
			}
			row["kind"] = "builtin"
			row["name"] = name
			row["width"] = nil
			row["signed"] = nil
			if node.kind == "word" {
				row["width"] = node.width
				row["signed"] = node.signed
			}
		case "array":
			row["kind"] = "array"
			row["length"] = node.length
			row["element"] = keys[node.child]
		case "slice":
			row["kind"] = "slice"
			row["element"] = keys[node.child]
		case "struct":
			fields := []map[string]any{}
			for _, f := range node.fields {
				var pkg any
				first, _ := utf8.DecodeRuneInString(f.name)
				if !unicode.IsUpper(first) {
					id, err := b.packageKey(f.packagePath)
					if err != nil {
						return "", err
					}
					pkg = id
				}
				fields = append(fields, map[string]any{"name": f.name, "type_id": keys[f.typeNode], "tag_bytes_hex": hex.EncodeToString([]byte(f.tag)), "embedded": f.embedded, "unexported_package_id": pkg})
			}
			row["kind"] = "struct"
			row["fields"] = fields
		default:
			return "", fmt.Errorf("unhandled native type %s", node.kind)
		}
		key, err := practicalTypeKeySeal(practicalSourceTypeDomain, row)
		if err != nil {
			return "", err
		}
		keys[i] = key["id"].(string)
	}
	return keys[root], nil
}
func TestPracticalOriginalSourceSignaturesReference(t *testing.T) {
	if runtime.Version() != practicalCompilerRelease {
		t.Fatal("reference requires pinned compiler", runtime.Version())
	}
	source := `package policy
import h "example.com/policy/helper"
type Word uint16
type Alias=Word
type Anchor struct{Foreign h.Record; Words [2]Word; View []Word; Bytes []byte; Raw struct{X uint16 "\xff"}; Empty struct{}; E error}
var Shared Word
var Forward Later
type Later uint16
var Fn func(x Word)(Word,string)
func Zero(){}
func Unnamed(Word,[]Word,struct{X uint16 "\xff"})(uint16,string){return 0,""}
func Named(x,y Alias,_ Word)(left Word,right []byte){var groupedA,groupedB Word;_,_=groupedA,groupedB;return}
func (w Word) Pair(v Word)(Word,Word){return w,v}
func (Word) NoReceiverName(){}
func Declared(keep Word,p *Word)(out Word){var own Word;_=own;return}
func Variadic(keep Word,rest ...Word){}
func Generic[T any](x T)T{return x}
func Probe(anchor Anchor){
 _=anchor
 var explicit Word
 var assigned Alias=1
 var groupedA,groupedB [2]Word
 type Local=Word
 var local Local
 {type Word uint8;var shadow Word;_=shadow}
 var foreign h.Record
 var folded [(1<<100)>>99]Word
 var implicit=1
 short:=explicit
 for key,value:=range []Word{explicit}{_,_=key,value}
 literal:=func(p Word)(r Word){var inside Word;_=inside;return p}
 _,_,_,_,_,_,_,_,_,_,_=explicit,assigned,groupedA,groupedB,local,foreign,folded,implicit,short,literal,Shared
}
func init(){panic("application must not execute")}
`
	for n := 0; n <= 17; n++ {
		results := []string{}
		for i := 0; i < n; i++ {
			results = append(results, fmt.Sprintf("r%d Word", i))
		}
		signature := ""
		if n > 0 {
			signature = "(" + strings.Join(results, ",") + ")"
		}
		source += fmt.Sprintf("func Arity%d()%s{return}\n", n, signature)
	}
	params := []string{}
	for n := 0; n < 70; n++ {
		params = append(params, fmt.Sprintf("p%d", n))
	}
	source += "func Many(" + strings.Join(params, ",") + " byte){}\n"
	files := map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": []byte(source), "helper/helper.go": []byte("package helper\ntype Record struct{private uint16;Exported byte}\n")}
	owner, original := practicalStorageTestOwner(t, files)
	keys, err := owner.typeKeys("example.com/policy", practicalStorageTestSpan(t, original["policy.go"], "anchor"))
	if err != nil {
		t.Fatal(err)
	}
	buffers := []map[string]any{}
	for _, b := range keys.sources {
		buffers = append(buffers, map[string]any{"object": b.object, "raw": b.sourceBytes()})
	}
	retainPracticalSyntaxEvidence(t, "signature-components.json", map[string]any{"canonical_components": bytes.Clone(keys.canonical), "sources": buffers})
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	loaded := owner.loader.packages["example.com/policy"]
	annotated := map[*ast.Ident]bool{}
	addNames := func(fields *ast.FieldList) {
		if fields != nil {
			for _, field := range fields.List {
				for _, name := range field.Names {
					annotated[name] = true
				}
			}
		}
	}
	functions := []map[string]any{}
	for _, file := range loaded.files {
		parents := map[ast.Node]ast.Node{}
		stack := []ast.Node{}
		ast.Inspect(file, func(n ast.Node) bool {
			if n == nil {
				stack = stack[:len(stack)-1]
				return false
			}
			if len(stack) > 0 {
				parents[n] = stack[len(stack)-1]
			}
			stack = append(stack, n)
			return true
		})
		ast.Inspect(file, func(n ast.Node) bool {
			if v, ok := n.(*ast.ValueSpec); ok && v.Type != nil {
				for _, name := range v.Names {
					annotated[name] = true
				}
			}
			var syntax *ast.FuncType
			var receiver *ast.FieldList
			var native *types.Signature
			var name any
			switch f := n.(type) {
			case *ast.FuncDecl:
				syntax = f.Type
				receiver = f.Recv
				native = loaded.info.Defs[f.Name].Type().(*types.Signature)
				name = f.Name.Name
				addNames(receiver)
				addNames(syntax.Params)
				addNames(syntax.Results)
			case *ast.FuncLit:
				syntax = f.Type
				native = loaded.info.Types[f].Type.(*types.Signature)
				addNames(syntax.Params)
				addNames(syntax.Results)
			case *ast.FuncType:
				switch parents[f].(type) {
				case *ast.FuncDecl, *ast.FuncLit:
					return true
				}
				syntax = f
				native = loaded.info.Types[f].Type.(*types.Signature)
			default:
				return true
			}
			span, _, err := owner.loader.syntax.spanBytes(n)
			if err != nil {
				t.Fatal(err)
			}
			expected := ""
			if native.TypeParams().Len() > 0 || native.Variadic() {
				expected = "Unsupported"
			} else if native.Results().Len() > 16 {
				expected = "Limit"
			}
			slots := []map[string]any{}
			addSlots := func(fields *ast.FieldList, tuple *types.Tuple, role string) {
				if fields == nil {
					return
				}
				index := 0
				for _, field := range fields.List {
					fs, _, err := owner.loader.syntax.spanBytes(field)
					if err != nil {
						t.Fatal(err)
					}
					count := len(field.Names)
					if count == 0 {
						count = 1
					}
					for j := 0; j < count; j++ {
						var actual types.Type
						if role == "receiver" {
							actual = native.Recv().Type()
						} else {
							actual = tuple.At(index).Type()
						}
						index++
						key, keyError := practicalSignatureReferenceKey(owner, actual)
						var typeID any = key
						if keyError != nil {
							typeID = nil
							if expected == "" {
								expected = "Type(Unsupported)"
							}
						}
						var slotName, nameStart, nameEnd any
						if len(field.Names) > 0 {
							ident := field.Names[j]
							ns, _, err := owner.loader.syntax.spanBytes(ident)
							if err != nil {
								t.Fatal(err)
							}
							slotName = ident.Name
							nameStart = ns.start
							nameEnd = ns.end
						}
						slots = append(slots, map[string]any{"role": role, "field_start_byte": fs.start, "field_end_byte": fs.end, "name": slotName, "name_start_byte": nameStart, "name_end_byte": nameEnd, "type_id": typeID})
					}
				}
			}
			addSlots(receiver, nil, "receiver")
			addSlots(syntax.Params, native.Params(), "parameter")
			addSlots(syntax.Results, native.Results(), "result")
			functions = append(functions, map[string]any{"source_object_id": span.sourceID, "start_byte": span.start, "end_byte": span.end, "syntax_kind": fmt.Sprintf("%T", n), "name": name, "slots": slots, "expected_reader_error": expected})
			return true
		})
	}
	variables := []map[string]any{}
	for ident, obj := range loaded.info.Defs {
		variable, ok := obj.(*types.Var)
		if !ok || variable.IsField() || ident.Name == "_" {
			continue
		}
		// Bare function-type parameter names have no lexical value binding.
		if obj.Parent() == nil {
			continue
		}
		span, _, err := owner.loader.syntax.spanBytes(ident)
		if err != nil {
			t.Fatal(err)
		}
		expected := ""
		var typeID any
		if !annotated[ident] {
			expected = "InferenceRequired"
		} else {
			key, err := practicalSignatureReferenceKey(owner, variable.Type())
			if err != nil {
				expected = "Type(Unsupported)"
			} else {
				typeID = key
			}
		}
		variables = append(variables, map[string]any{"source_object_id": span.sourceID, "start_byte": span.start, "end_byte": span.end, "name": ident.Name, "type_id": typeID, "expected_reader_error": expected})
	}
	order := func(rows []map[string]any) {
		sort.Slice(rows, func(i, j int) bool {
			a, b := rows[i], rows[j]
			if a["source_object_id"] != b["source_object_id"] {
				return a["source_object_id"].(string) < b["source_object_id"].(string)
			}
			return a["start_byte"].(int) < b["start_byte"].(int)
		})
	}
	order(functions)
	order(variables)
	retainPracticalSyntaxEvidence(t, "signature-observations.json", map[string]any{"functions": functions, "variables": variables, "runtime": runtime.Version(), "applications_executed": false, "whole_body_or_program_authority": false})
	t.Logf("actual original functions %d variables %d", len(functions), len(variables))
}
