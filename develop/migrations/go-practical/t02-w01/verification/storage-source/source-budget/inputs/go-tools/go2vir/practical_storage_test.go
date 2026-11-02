//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
)

func TestPracticalStorageRequiresOriginalOwner(t *testing.T) {
	for _, owner := range []*practicalOriginalPackages{nil, {}} {
		if value, err := owner.storageRoute("example.com/policy", practicalSourceSpan{}); err == nil || value.source.object.ID != "" {
			t.Fatal("unowned storage route accepted", err)
		}
	}
}
func practicalStorageTestOwner(t *testing.T, files map[string][]byte) (*practicalOriginalPackages, map[string]practicalSourceBuffer) {
	t.Helper()
	compiler := practicalPackageTestCompiler(t)
	roots, _ := practicalTestCaptureRoots(t)
	roots = append(roots, practicalSourceRoot{Capability: "original-go-sdk", Directory: compiler.selection.GoRoot})
	main, cache := practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability}
	for name, raw := range files {
		writePracticalTestFile(t, filepath.Join(roots[0].Directory, name), raw, 0o600)
	}
	retainPracticalSyntaxEvidence(t, "storage-original-inputs.json", map[string]any{"roots": roots, "whole_original_sources": files})
	session := practicalTestSession(t, roots)
	discovery, err := session.discoverPracticalModules(main, cache, compiler.selection)
	if err != nil {
		var failure *practicalGoDiscoveryFailure
		if errors.As(err, &failure) {
			stdout, stderr := failure.originalOutput()
			retainPracticalSyntaxEvidence(t, "storage-module-failure.json", map[string]any{"stdout": stdout, "stderr": stderr, "error": err.Error()})
		}
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "storage-original-modules.json", map[string]any{"stdout": discovery.output, "stderr": discovery.observation.diagnostics, "compiler_sha256": discovery.compilerSHA256})
	if err := session.capturePracticalStandardLibrary(compiler, roots[2].Capability); err != nil {
		t.Fatal(err)
	}
	exports, err := session.capturePracticalPackageExports(compiler, discovery, roots[2].Capability, main, "example.com/policy")
	if err != nil {
		var failure *practicalGoDiscoveryFailure
		if errors.As(err, &failure) {
			exports.output, exports.diagnostics = failure.originalOutput()
		}
	}
	retainPracticalSyntaxEvidence(t, "storage-original-exports.json", map[string]any{"stdout": exports.output, "stderr": exports.diagnostics, "error": fmt.Sprint(err)})
	if err != nil {
		t.Fatal(err)
	}
	sealed, err := session.sealPracticalReplaySources()
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalStandardSnapshot(t, sealed.snapshot)
	retainPracticalStandardSnapshot(t, exports.snapshot)
	image, err := materializePracticalReplaySources(t.TempDir(), []practicalReplaySources{sealed, exports.replay}, &compiler, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	defer image.Close()
	for _, root := range roots {
		if err := removePracticalReplayImage(root.Directory); err != nil {
			t.Fatal(err)
		}
		if _, err := os.Lstat(root.Directory); !os.IsNotExist(err) {
			t.Fatal("original storage preparation path survived", err)
		}
	}
	for _, snapshot := range []practicalSourceSnapshot{sealed.snapshot, exports.snapshot} {
		for _, buffer := range snapshot.buffers {
			clear(buffer.data)
		}
	}
	owner, err := image.originalPackages(main, cache, discovery, exports, roots[2].Capability)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(owner.Close)
	if err := image.Close(); err != nil {
		t.Fatal(err)
	}
	result := map[string]practicalSourceBuffer{}
	for _, pkg := range owner.packages {
		for _, object := range pkg.sources {
			if object.Capability == roots[0].Capability && strings.HasSuffix(object.Path, ".go") {
				buffer, err := owner.originalBytes(object)
				if err != nil {
					t.Fatal(err)
				}
				result[object.Path] = buffer
			}
		}
	}
	return owner, result
}
func practicalStorageTestSpan(t *testing.T, source practicalSourceBuffer, expression string) practicalSourceSpan {
	t.Helper()
	raw := []byte(expression)
	start := bytes.Index(source.data, raw)
	if start < 0 || bytes.Index(source.data[start+len(raw):], raw) >= 0 {
		needle := []byte("_ = " + expression)
		start = bytes.Index(source.data, needle)
		if start < 0 || bytes.Index(source.data[start+len(needle):], needle) >= 0 {
			t.Fatal("storage test needs one original physical expression", expression)
		}
		start += 4
	}
	return practicalSourceSpan{sourceID: source.object.ID, start: start, end: start + len(raw)}
}
func practicalStorageTestEvidence(route practicalStorageRoute) map[string]any {
	nodes := []map[string]any{}
	for n, node := range route.types {
		fields := []map[string]any{}
		for _, field := range node.fields {
			fields = append(fields, map[string]any{"name": field.name, "package": field.packagePath, "tag": field.tag, "embedded": field.embedded, "type_node": field.typeNode})
		}
		nodes = append(nodes, map[string]any{"node": n, "kind": node.kind, "native_display_only": node.display, "width": node.width, "signed": node.signed, "length": node.length, "child": node.child, "declaration": node.declaration, "fields": fields, "depth": node.depth})
	}
	decls := []map[string]any{}
	span := func(s practicalSourceSpan) map[string]any {
		return map[string]any{"source_id": s.sourceID, "start": s.start, "end": s.end}
	}
	for _, d := range route.declarations {
		decls = append(decls, map[string]any{"name": d.name, "package": d.packagePath, "module_source_id": d.moduleSourceID, "alias": d.alias, "span": span(d.span), "source_object": d.source.object, "whole_original_source": d.source.sourceBytes()})
	}
	steps := []map[string]any{}
	for _, step := range route.steps {
		steps = append(steps, map[string]any{"kind": step.kind, "parent_type_node": step.parent, "result_type_node": step.result, "field": step.field, "span": span(step.span), "index": map[string]any{"span": span(step.index.span), "width": step.index.width, "signed": step.index.signed, "constant": step.index.constant, "native_display_only": step.index.nativeDisplay, "type_node": step.index.typeNode, "defaulted": step.index.defaulted}})
	}
	return map[string]any{"package": route.packagePath, "module_source_id": route.moduleSourceID, "image_digest": route.imageDigest, "compiler_digest": route.compilerDigest, "span": span(route.span), "source_object": route.source.object, "whole_original_source": route.source.sourceBytes(), "original_expression": bytes.Clone(route.expression), "root": map[string]any{"kind": route.root.kind, "type_node": route.root.typeNode, "expression": span(route.root.expression), "binding": span(route.root.binding), "package": route.root.bindingPackage, "name": route.root.bindingName, "scope": route.root.bindingScope}, "steps": steps, "result_type_node": route.result, "native_types": nodes, "original_declarations": decls, "source_type_ID_or_ordinary_semantics_proof": false, "registered_context_or_live_owner_permission_proof": false}
}
func TestPracticalStorageCapturesOriginalTypedRoutes(t *testing.T) {
	var source strings.Builder
	source.WriteString("package policy\r\nimport \"example.com/policy/helper\"\r\ntype Count uint16\r\ntype CountAlias = Count\r\ntype OtherCount uint16\r\ntype Row struct { Values [4]Count; Marker uint8 `json:\"marker\"` }\r\ntype Root struct { Rows [3]Row; Copy [256]byte; Ref []Row }\r\ntype Embedded struct { Row; Extra uint64 }\r\n")
	source.WriteString("type Wide struct {\r\n")
	for i := 0; i < 64; i++ {
		fmt.Fprintf(&source, "F%02d uint64\r\n", i)
	}
	source.WriteString("}\r\n")
	for d := 1; d <= 17; d++ {
		child := "byte"
		if d > 1 {
			child = fmt.Sprintf("Depth%d", d-1)
		}
		fmt.Fprintf(&source, "type Depth%d struct { Child %s }\r\n", d, child)
	}
	source.WriteString("type TooWide struct {\r\n")
	for i := 0; i < 65; i++ {
		fmt.Fprintf(&source, "G%02d byte\r\n", i)
	}
	source.WriteString("}\r\n")
	source.WriteString("type Recursive struct { Next []Recursive }\r\ntype Hidden struct { _ [0]*int; Value byte }\r\ntype Generic[T any] struct { Value T }\r\ntype ErrorAlias = error\r\ntype OtherError error\r\n")
	source.WriteString("func Paths(root Root, rows []Row, index uint64, signed int64, small uint8, embed Embedded, wide Wide, alias CountAlias, other OtherCount, text string, p *Root, values map[int]byte) Count {\r\n")
	for _, expr := range []string{"root.Rows[index].Values[signed]", "rows[index].Values[small]", "root.Ref[index].Values[0]", "root.Rows[1].Values[2]", "root.Copy[255]", "embed.Values[1]", "wide.F63", "alias", "other", "helper.Exported[0].Values[3]", "p.Rows[0].Values[1]", "text[0]", "values[0]", "([4]byte{})[0]", "Count(1)", "[]byte{7, 8}[1]", "rows[0:1][0].Marker"} {
		fmt.Fprintf(&source, "_ = %s\r\n", expr)
	}
	source.WriteString("return 0\r\n}\r\n")
	for d := 16; d <= 17; d++ {
		fmt.Fprintf(&source, "func Path%d(deep%d Depth%d) byte { return deep%d%s }\r\n", d, d, d, d, strings.Repeat(".Child", d))
	}
	source.WriteString("func Limits(too TooWide, large [257]byte, recursive Recursive, hidden Hidden, generic Generic[int], iface OtherError, plain ErrorAlias) { _ = too.G00; _ = large[0]; _ = recursive; _ = hidden.Value; _ = generic.Value; _ = iface; _ = plain }\r\nfunc init(){panic(\"source application must not execute\")}\r\n")
	source.WriteString("type Repeated struct { A struct{Value byte}; B struct{Value byte}; C struct{Value byte `json:\"x\"`}; D struct{Value byte `json:\"y\"`} }\r\n")
	source.WriteString("func More(arr [4]byte, u8 uint8, u16 uint16, u32 uint32, u64 uint64, i8 int8, i16 int16, i32 int32, i64 int64, nativeInt int, nativeUint uint, indexAlias CountAlias, repeated Repeated, empty [0]Count) {\r\n")
	indexTypes := []struct {
		expression string
		width      int
		signed     bool
	}{
		{"arr[u8]", 8, false}, {"arr[u16]", 16, false}, {"arr[u32]", 32, false}, {"arr[u64]", 64, false},
		{"arr[i8]", 8, true}, {"arr[i16]", 16, true}, {"arr[i32]", 32, true}, {"arr[i64]", 64, true},
		{"arr[nativeInt]", 64, true}, {"arr[nativeUint]", 64, false}, {"arr[indexAlias]", 16, false},
	}
	for _, row := range indexTypes {
		fmt.Fprintf(&source, "_ = %s\r\n", row.expression)
	}
	source.WriteString("_ = repeated; _ = empty; _ = arr[1 << 1]\r\n}\r\n")
	helper := []byte("package helper\ntype Amount uint16\ntype Record struct{Values [4]Amount}\nvar Exported [1]Record\n")
	files := map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": []byte(source.String()), "helper/helper.go": helper}
	owner, original := practicalStorageTestOwner(t, files)
	file := original["policy.go"]
	cases := []struct {
		expression, root string
		kinds            []string
	}{
		{"root.Rows[index].Values[signed]", "value_binding", []string{"field", "array_index", "field", "array_index"}},
		{"rows[index].Values[small]", "slice_descriptor", []string{"slice_index", "field", "array_index"}},
		{"root.Ref[index].Values[0]", "slice_descriptor", []string{"slice_index", "field", "array_index"}},
		{"root.Rows[1].Values[2]", "value_binding", []string{"field", "array_index", "field", "array_index"}},
		{"root.Copy[255]", "value_binding", []string{"field", "array_index"}},
		{"embed.Values[1]", "value_binding", []string{"field", "field", "array_index"}},
		{"wide.F63", "value_binding", []string{"field"}},
		{"helper.Exported[0].Values[3]", "value_binding", []string{"array_index", "field", "array_index"}},
		{"[]byte{7, 8}[1]", "slice_descriptor", []string{"slice_index"}},
		{"rows[0:1][0].Marker", "slice_descriptor", []string{"slice_index", "field"}},
		{"deep16" + strings.Repeat(".Child", 16), "value_binding", strings.Fields(strings.Repeat("field ", 16))},
		{"alias", "value_binding", []string{}},
		{"other", "value_binding", []string{}},
		{"plain", "value_binding", []string{}},
	}
	cases = append(cases, struct {
		expression, root string
		kinds            []string
	}{"repeated", "value_binding", []string{}}, struct {
		expression, root string
		kinds            []string
	}{"empty", "value_binding", []string{}}, struct {
		expression, root string
		kinds            []string
	}{"arr[1 << 1]", "value_binding", []string{"array_index"}})
	for _, row := range indexTypes {
		cases = append(cases, struct {
			expression, root string
			kinds            []string
		}{row.expression, "value_binding", []string{"array_index"}})
	}
	for i, row := range cases {
		t.Run(fmt.Sprint(i), func(t *testing.T) {
			span := practicalStorageTestSpan(t, file, row.expression)
			route, err := owner.storageRoute("example.com/policy", span)
			if err != nil {
				t.Fatal(err)
			}
			retainPracticalSyntaxEvidence(t, fmt.Sprintf("storage-route-%02d.json", i), practicalStorageTestEvidence(route))
			if route.root.kind != row.root || !bytes.Equal(route.expression, []byte(row.expression)) || route.span != span {
				t.Fatal("root or exact source bytes differ")
			}
			kinds := []string{}
			for _, s := range route.steps {
				kinds = append(kinds, s.kind)
			}
			if !reflect.DeepEqual(kinds, row.kinds) {
				t.Fatal("typed route differs", kinds, row.kinds)
			}
			for _, d := range route.declarations {
				if !bytes.Equal(d.source.data, files[d.source.object.Path]) {
					t.Fatal("original declaration source differs")
				}
			}
			if i == 0 {
				if route.steps[1].index.width != 64 || route.steps[1].index.signed || route.steps[1].index.constant != "" || route.steps[3].index.width != 64 || !route.steps[3].index.signed {
					t.Fatal("runtime indices lost full original integer typing")
				}
			}
			if i == 1 && route.steps[2].index.width != 8 {
				t.Fatal("narrow native index type was silently widened")
			}
			if i == 2 && string(file.data[route.root.expression.start:route.root.expression.end]) != "root.Ref" {
				t.Fatal("slice path did not pivot to its actual descriptor")
			}
			if i == 3 && (route.steps[1].index.constant != "1" || route.steps[3].index.constant != "2") {
				t.Fatal("original constant indices changed")
			}
			if i == 4 {
				found := false
				for _, n := range route.types {
					found = found || n.kind == "array" && n.length == 256
				}
				if !found || route.steps[1].index.constant != "255" {
					t.Fatal("complete array length/index lost")
				}
			}
			if i == 6 {
				if route.steps[0].field != 63 {
					t.Fatal("last field ordinal truncated")
				}
				found := false
				for _, n := range route.types {
					found = found || n.kind == "struct" && len(n.fields) == 64
				}
				if !found {
					t.Fatal("full struct layout omitted")
				}
			}
			if i == 7 && route.root.bindingPackage != "example.com/policy/helper" {
				t.Fatal("imported variable lost its original binding")
			}
			if i == 11 {
				alias, named := false, false
				for _, d := range route.declarations {
					alias = alias || d.name == "CountAlias" && d.alias
					named = named || d.name == "Count" && !d.alias
				}
				if !alias || !named {
					t.Fatal("native alias and target declarations were conflated")
				}
			}
			if i == 12 {
				if len(route.declarations) != 1 || route.declarations[0].name != "OtherCount" {
					t.Fatal("distinct named types were conflated")
				}
			}
			if i == 13 {
				found := false
				for _, n := range route.types {
					found = found || n.kind == "error"
				}
				if !found {
					t.Fatal("predeclared error alias lost its native target")
				}
			}
			if row.expression == "repeated" {
				n := route.result
				for route.types[n].kind == "named" || route.types[n].kind == "alias" {
					n = route.types[n].child
				}
				fields := route.types[n].fields
				if len(fields) != 4 || fields[0].typeNode != fields[1].typeNode || fields[2].typeNode == fields[3].typeNode {
					t.Fatal("anonymous shape equality lost field/tag identity")
				}
			}
			if row.expression == "empty" {
				if route.types[route.result].kind != "array" || route.types[route.result].length != 0 || route.types[route.result].child < 0 {
					t.Fatal("zero-length array lost its complete element closure")
				}
			}
			if row.expression == "arr[1 << 1]" && route.steps[0].index.constant != "2" {
				t.Fatal("native folded index or original expression lost")
			}
			for _, index := range indexTypes {
				if row.expression == index.expression && (route.steps[0].index.width != index.width || route.steps[0].index.signed != index.signed || route.steps[0].index.constant != "") {
					t.Fatal("full native signed/unsigned index type differs", index.expression)
				}
			}
			// Returned evidence must not mutate the owner or the next complete query.
			expected := practicalStorageTestEvidence(route)
			clear(route.source.data)
			clear(route.expression)
			for _, d := range route.declarations {
				clear(d.source.data)
			}
			for k := range route.types {
				route.types[k].kind = "damaged"
				clear(route.types[k].fields)
			}
			again, err := owner.storageRoute("example.com/policy", span)
			if err != nil || !reflect.DeepEqual(expected, practicalStorageTestEvidence(again)) {
				t.Fatal("caller alias damaged original storage facts", err)
			}
		})
	}
	for i, expression := range []string{"p.Rows[0].Values[1]", "text[0]", "values[0]", "([4]byte{})[0]", "Count(1)", "deep17" + strings.Repeat(".Child", 17), "too.G00", "large[0]", "recursive", "hidden.Value", "generic.Value", "iface"} {
		t.Run(fmt.Sprintf("reject-%d", i), func(t *testing.T) {
			span := practicalStorageTestSpan(t, file, expression)
			route, err := owner.storageRoute("example.com/policy", span)
			retainPracticalSyntaxEvidence(t, fmt.Sprintf("storage-rejection-%02d.json", i), map[string]any{"expression": expression, "span": map[string]any{"source_id": span.sourceID, "start": span.start, "end": span.end}, "error": fmt.Sprint(err), "result_has_source": route.source.object.ID != "", "source_admission_claimed": false})
			if err == nil || route.source.object.ID != "" {
				t.Fatal("nonpath or unsupported complete layout supplied a route", expression, err)
			}
		})
	}
	valid := practicalStorageTestSpan(t, file, "root.Copy[255]")
	for _, bad := range []practicalSourceSpan{{sourceID: valid.sourceID, start: valid.start, end: valid.end - 1}, {sourceID: valid.sourceID, start: -1, end: valid.end}, {sourceID: valid.sourceID, start: valid.start, end: len(file.data) + 1}, {sourceID: original["helper/helper.go"].object.ID, start: valid.start, end: valid.end}} {
		if _, err := owner.storageRoute("example.com/policy", bad); err == nil {
			t.Fatal("crossed/inexact physical storage locator accepted")
		}
	}
	if _, err := owner.storageRoute("example.com/policy/helper", valid); err == nil {
		t.Fatal("storage query crossed package membership")
	}
	owner.Close()
	if _, err := owner.storageRoute("example.com/policy", valid); err == nil {
		t.Fatal("retired owner supplied a storage route")
	}
}
