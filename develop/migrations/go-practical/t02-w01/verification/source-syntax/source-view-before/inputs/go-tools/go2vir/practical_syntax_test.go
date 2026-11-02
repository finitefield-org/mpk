//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"go/ast"
	"go/token"
	"io/fs"
	"os"
	"path/filepath"
	"reflect"
	"strings"
	"testing"
)

func practicalSyntaxFixture(t *testing.T, sources map[string][]byte) (practicalSourceSnapshot, []practicalSourceRoot) {
	t.Helper()
	roots, _ := practicalTestCaptureRoots(t)
	for name, raw := range sources {
		writePracticalTestFile(t, filepath.Join(roots[0].Directory, name), raw, 0o600)
	}
	session := practicalTestSession(t, roots)
	for name := range sources {
		if _, err := session.readSource(roots[0].Capability, name, "go_file"); err != nil {
			t.Fatal(err)
		}
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	return snapshot, roots
}

func practicalSyntaxImageFor(t *testing.T, snapshot practicalSourceSnapshot) *practicalSyntaxImage {
	t.Helper()
	image, err := newPracticalSyntaxImage(snapshot)
	if err != nil {
		t.Fatal(err)
	}
	return image
}

func retainPracticalSyntaxEvidence(t *testing.T, name string, record any) {
	t.Helper()
	if directory := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"); directory != "" {
		raw, err := json.MarshalIndent(record, "", "  ")
		if err != nil {
			t.Fatal(err)
		}
		file, err := os.OpenFile(filepath.Join(directory, name), os.O_WRONLY|os.O_CREATE|os.O_EXCL, 0o600)
		if err != nil {
			t.Fatal(err)
		}
		_, writeErr := file.Write(append(raw, '\n'))
		closeErr := file.Close()
		if writeErr != nil || closeErr != nil {
			t.Fatal("cannot retain complete original syntax evidence")
		}
	}
}

// Independently inspect every exported AST edge, including File.Comments and
// repeated Imports/Doc links. The production walker must retain exactly this
// complete set, rather than only the nodes reached by ast.Walk(File).
func practicalSyntaxAllNodes(file *ast.File) []ast.Node {
	seen := map[ast.Node]bool{}
	var nodes []ast.Node
	var walk func(reflect.Value)
	walk = func(v reflect.Value) {
		if !v.IsValid() || (v.Kind() == reflect.Interface || v.Kind() == reflect.Pointer || v.Kind() == reflect.Slice) && v.IsNil() {
			return
		}
		for v.Kind() == reflect.Interface {
			v = v.Elem()
		}
		if v.CanInterface() {
			if node, ok := v.Interface().(ast.Node); ok {
				if seen[node] {
					return
				}
				seen[node] = true
				nodes = append(nodes, node)
			}
		}
		switch v.Kind() {
		case reflect.Pointer, reflect.Interface:
			walk(v.Elem())
		case reflect.Struct:
			for i := 0; i < v.NumField(); i++ {
				if v.Field(i).CanInterface() {
					walk(v.Field(i))
				}
			}
		case reflect.Slice:
			for i := 0; i < v.Len(); i++ {
				walk(v.Index(i))
			}
		}
	}
	walk(reflect.ValueOf(file))
	return nodes
}

func TestPracticalSyntaxRetainsPhysicalRawTokensAndLineOrigins(t *testing.T) {
	raw := []byte("// original 雪\r\npackage policy\r\n//line invented.go:700:9\r\n/* block\r\n\r comment */\r\nconst Raw = `a\r\nb\rc`\r\n// tail\r\nfunc 金額(x int) int { return x } // final\r")
	snapshot, roots := practicalSyntaxFixture(t, map[string][]byte{"policy.go": raw})
	image := practicalSyntaxImageFor(t, snapshot)
	parsed, err := image.parse(roots[0].Capability, "policy.go")
	if err != nil {
		t.Fatal(err)
	}
	checked, repaired, remapped := 0, 0, false
	observations := []map[string]any{}
	for _, node := range practicalSyntaxAllNodes(parsed.file) {
		span, actual, err := image.spanBytes(node)
		if err != nil || !bytes.Equal(actual, raw[span.start:span.end]) || span.sourceID != parsed.source.object.ID {
			t.Fatalf("lost original node bytes: %T: %+v, %v", node, span, err)
		}
		physical := image.fileSet.PositionFor(node.Pos(), false)
		adjusted := image.fileSet.PositionFor(node.Pos(), true)
		if physical.Offset != span.start {
			t.Fatal("source origin used an adjusted line offset")
		}
		remapped = remapped || physical.Filename != adjusted.Filename
		switch node := node.(type) {
		case *ast.BasicLit:
			if node.Kind == token.STRING && strings.HasPrefix(node.Value, "`") {
				if string(actual) != "`a\r\nb\rc`" {
					t.Fatalf("raw string physical token truncated: %q", actual)
				}
				repaired++
			}
		case *ast.ValueSpec:
			if len(node.Names) == 1 && node.Names[0].Name == "Raw" {
				if string(actual) != "Raw = `a\r\nb\rc`" {
					t.Fatalf("enclosing source declaration truncated: %q", actual)
				}
				repaired++
			}
		case *ast.Comment:
			start := bytes.Index(raw, []byte(node.Text[:2]))
			if start < 0 || len(actual) < len(node.Text) || !bytes.HasPrefix(actual, []byte(node.Text[:2])) {
				t.Fatal("original comment binding is incomplete")
			}
		case *ast.Ident:
			name, err := image.identifier(node)
			if err != nil || name != node.Name {
				t.Fatal("original Unicode identifier changed")
			}
		}
		observations = append(observations, map[string]any{"kind": reflect.TypeOf(node).String(), "start": span.start, "end": span.end, "raw": actual})
		checked++
	}
	comments := []string{"// original 雪\r", "//line invented.go:700:9\r", "/* block\r\n\r comment */", "// tail\r", "// final\r"}
	var actualComments []string
	for _, group := range parsed.file.Comments {
		for _, comment := range group.List {
			_, actual, err := image.spanBytes(comment)
			if err != nil {
				t.Fatal(err)
			}
			actualComments = append(actualComments, string(actual))
		}
	}
	if !reflect.DeepEqual(comments, actualComments) {
		t.Fatalf("complete original comment tokens differ: %q", actualComments)
	}
	if checked < 15 || repaired != 2 || !remapped {
		t.Fatal("physical CR/line-directive controls were not exercised")
	}
	retainPracticalSyntaxEvidence(t, "physical-origins.json", map[string]any{"source": parsed.source.object, "raw": raw, "nodes": observations})
}

func TestPracticalSyntaxReadsAfterOriginalRemovalAndOwnsReturnedBytes(t *testing.T) {
	raw := []byte("package policy\nfunc Selected(x int64) int64 { return x }\nfunc Unused(m map[string]int) int {return len(m)}\n")
	snapshot, roots := practicalSyntaxFixture(t, map[string][]byte{"policy.go": raw})
	for _, root := range roots {
		if err := os.RemoveAll(root.Directory); err != nil {
			t.Fatal(err)
		}
	}
	image := practicalSyntaxImageFor(t, snapshot)
	parsed, err := image.parse(roots[0].Capability, "policy.go")
	if err != nil || len(parsed.file.Decls) != 2 || !bytes.Equal(parsed.source.data, raw) {
		t.Fatal("original source was substituted, stubbed or reopened", err)
	}
	span, returned, err := image.spanBytes(parsed.file.Decls[1])
	if err != nil {
		t.Fatal(err)
	}
	returned[0] ^= 1
	again, bytesAgain, err := image.spanBytes(parsed.file.Decls[1])
	if err != nil || span != again || !bytes.Equal(bytesAgain, raw[span.start:span.end]) {
		t.Fatal("returned node bytes alias original captured storage")
	}
	if !bytes.Contains(bytesAgain, []byte("map[string]int")) {
		t.Fatal("unselected original body was removed before package type checking")
	}
	count := image.count
	second, err := image.parse(roots[0].Capability, "policy.go")
	if err != nil || second != parsed || image.count != count {
		t.Fatal("same original source was reparsed or charged twice")
	}
}

func TestPracticalSyntaxReturnedSourceCannotMutateCachedOriginal(t *testing.T) {
	raw := []byte("package policy\ntype Amount int64\n")
	snapshot, roots := practicalSyntaxFixture(t, map[string][]byte{"policy.go": raw})
	image := practicalSyntaxImageFor(t, snapshot)
	parsed, err := image.parse(roots[0].Capability, "policy.go")
	if err != nil {
		t.Fatal(err)
	}
	node := parsed.file.Decls[0]
	span, _, err := image.spanBytes(node)
	if err != nil {
		t.Fatal(err)
	}
	parsed.source.data[span.start] ^= 1
	_, actual, err := image.spanBytes(node)
	if err != nil || !bytes.Equal(actual, raw[span.start:span.end]) {
		t.Fatal("returned source bytes changed cached original node evidence")
	}
	second, err := image.parse(roots[0].Capability, "policy.go")
	if err != nil || !bytes.Equal(second.source.data, raw) {
		t.Fatal("returned source bytes changed the next original-source read")
	}
}

func TestPracticalSyntaxSeparatesCapabilityAndSourceIdentity(t *testing.T) {
	roots, _ := practicalTestCaptureRoots(t)
	roots[0].Capability = "module"
	roots[1].Capability = "module/sub"
	for i, relative := range []string{"sub/policy.go", "policy.go"} {
		writePracticalTestFile(t, filepath.Join(roots[i].Directory, relative), []byte("package policy\ntype Amount int64\n"), 0o600)
	}
	session := practicalTestSession(t, roots)
	for i, relative := range []string{"sub/policy.go", "policy.go"} {
		if _, err := session.readSource(roots[i].Capability, relative, "go_file"); err != nil {
			t.Fatal(err)
		}
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	image := practicalSyntaxImageFor(t, snapshot)
	a, err := image.parse(roots[0].Capability, "sub/policy.go")
	if err != nil {
		t.Fatal(err)
	}
	b, err := image.parse(roots[1].Capability, "policy.go")
	if err != nil || a.source.object.ID == b.source.object.ID || image.fileSet.File(a.file.Pos()).Name() == image.fileSet.File(b.file.Pos()).Name() {
		t.Fatal("joined capability/path spelling confused distinct original source objects", err)
	}
}

func TestPracticalSyntaxRejectsMissingWrongRoleAndCorruptedInputs(t *testing.T) {
	for _, test := range []string{"unread", "absent", "wrong-role", "corrupt-bytes", "wrong-case", "escape"} {
		t.Run(test, func(t *testing.T) {
			roots, _ := practicalTestCaptureRoots(t)
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "policy.go"), []byte("package policy\n"), 0o600)
			writePracticalTestFile(t, filepath.Join(roots[0].Directory, "unread.go"), []byte("package policy\n"), 0o600)
			session := practicalTestSession(t, roots)
			kind := "go_file"
			if test == "wrong-role" {
				kind = "excluded_source"
			}
			if _, err := session.readSource(roots[0].Capability, "policy.go", kind); err != nil {
				t.Fatal(err)
			}
			snapshot, err := session.seal()
			if err != nil {
				t.Fatal(err)
			}
			image := practicalSyntaxImageFor(t, snapshot)
			name := "policy.go"
			switch test {
			case "unread":
				name = "unread.go"
			case "absent":
				name = "absent.go"
			case "wrong-case":
				name = "Policy.go"
			case "escape":
				name = "../policy.go"
			case "corrupt-bytes":
				image.snapshot.buffers[snapshot.ids[0]].data[0] ^= 1
			}
			if _, err := image.parse(roots[0].Capability, name); err == nil {
				t.Fatal("unbound original source was parsed")
			} else if test == "unread" && !errors.Is(err, errPracticalNotCaptured) || test == "absent" && !errors.Is(err, fs.ErrNotExist) {
				t.Fatal("uncaptured input was confused with observed absence", err)
			}
			if _, err := image.parse(roots[0].Capability, "policy.go"); err == nil {
				t.Fatal("ignored failed source read did not close the syntax transaction")
			}
		})
	}
	if _, err := newPracticalSyntaxImage(practicalSourceSnapshot{}); err == nil {
		t.Fatal("unsealed syntax image was accepted")
	}
}

func TestPracticalSyntaxRejectsForeignAndChangedNodeBindings(t *testing.T) {
	snapshot, roots := practicalSyntaxFixture(t, map[string][]byte{"policy.go": []byte("package policy\ntype Amount int64\n")})
	for _, test := range []string{"foreign", "position", "identifier"} {
		t.Run(test, func(t *testing.T) {
			image := practicalSyntaxImageFor(t, snapshot)
			parsed, err := image.parse(roots[0].Capability, "policy.go")
			if err != nil {
				t.Fatal(err)
			}
			name := parsed.file.Decls[0].(*ast.GenDecl).Specs[0].(*ast.TypeSpec).Name
			switch test {
			case "foreign":
				name = &ast.Ident{NamePos: name.NamePos, Name: name.Name}
			case "position":
				name.NamePos++
			case "identifier":
				name.Name = "Credit"
			}
			if _, err := image.identifier(name); err == nil {
				t.Fatal("foreign/changed compiler node acquired original-source authority")
			}
		})
	}
}

func TestPracticalSyntaxBudgetCountsEveryDistinctOriginalTreeNode(t *testing.T) {
	raw := []byte("package policy\n// Shared doc\nfunc Root(x int) int {return x}\n")
	snapshot, roots := practicalSyntaxFixture(t, map[string][]byte{"a.go": raw, "b.go": raw})
	first := practicalSyntaxImageFor(t, snapshot)
	parsed, err := first.parse(roots[0].Capability, "a.go")
	if err != nil {
		t.Fatal(err)
	}
	visits := 0
	ast.Inspect(parsed.file, func(n ast.Node) bool {
		if n != nil {
			visits++
		}
		return true
	})
	for _, group := range parsed.file.Comments {
		ast.Inspect(group, func(n ast.Node) bool {
			if n != nil {
				visits++
			}
			return true
		})
	}
	if first.count != len(first.nodes) || first.count != len(practicalSyntaxAllNodes(parsed.file)) || visits <= first.count || maximumPracticalSyntaxNodes != 4_194_304 || maximumTypedSyntaxNodes != 1_000_000 {
		t.Fatal("node visits, distinct trees or frozen old/practical budgets were confused")
	}
	image := practicalSyntaxImageFor(t, snapshot)
	image.count = maximumPracticalSyntaxNodes - first.count
	if _, err := image.parse(roots[0].Capability, "a.go"); err != nil || image.count != maximumPracticalSyntaxNodes {
		t.Fatal("exact remaining syntax budget rejected", err)
	}
	if _, err := image.parse(roots[0].Capability, "a.go"); err != nil {
		t.Fatal("cached original tree consumed a second budget", err)
	}
	if _, err := image.parse(roots[0].Capability, "b.go"); err == nil || len(image.files) != 1 || len(image.nodes) != first.count {
		t.Fatal("over-budget original tree was installed or partially retained")
	}
}

func TestPracticalSyntaxPreservesAll81OriginalAdmissionSources(t *testing.T) {
	raw, err := os.ReadFile("../../develop/migrations/go-practical/t01-w04/source-admission/source-vectors.json")
	if err != nil || sha256Hex(raw) != "81e7c48cb20d4c51c40b95edd00f54942420d3f3a6225e40159fd1c9d884f0cc" {
		t.Fatal("complete original source catalog changed", err)
	}
	var catalog struct {
		Cases []struct {
			ID       string
			Source   string
			Expected string `json:"expected_status"`
		}
	}
	if err := json.Unmarshal(raw, &catalog); err != nil || len(catalog.Cases) != 81 {
		t.Fatal("original source catalog is incomplete", err)
	}
	sources := map[string][]byte{}
	for _, c := range catalog.Cases {
		sources["cases/"+c.ID+".go"] = []byte(c.Source)
	}
	snapshot, roots := practicalSyntaxFixture(t, sources)
	observations := []map[string]any{}
	parsedCount, rejectedCount, nodeCount := 0, 0, 0
	for _, c := range catalog.Cases {
		t.Run(c.ID, func(t *testing.T) {
			image := practicalSyntaxImageFor(t, snapshot)
			parsed, err := image.parse(roots[0].Capability, "cases/"+c.ID+".go")
			if c.Expected == "source_error" {
				if err == nil {
					t.Fatal("original syntax error in an unselected body was removed")
				}
				rejectedCount++
				original, readErr := snapshot.readSource(roots[0].Capability, "cases/"+c.ID+".go", "go_file")
				if readErr != nil {
					t.Fatal(readErr)
				}
				observations = append(observations, map[string]any{"id": c.ID, "source": original.object, "raw": []byte(c.Source), "syntax_error": err.Error()})
				return
			}
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(parsed.source.data, []byte(c.Source)) || image.count != len(image.nodes) {
				t.Fatal("original complete source or AST nodes changed")
			}
			var nodes []map[string]any
			allNodes := practicalSyntaxAllNodes(parsed.file)
			if len(allNodes) != image.count {
				t.Fatal("materialized original AST nodes were omitted")
			}
			for _, node := range allNodes {
				span, physical, err := image.spanBytes(node)
				if err != nil || !bytes.Equal(physical, []byte(c.Source)[span.start:span.end]) {
					t.Fatal("original node span changed", err)
				}
				nodes = append(nodes, map[string]any{"kind": reflect.TypeOf(node).String(), "start": span.start, "end": span.end})
			}
			parsedCount++
			nodeCount += image.count
			observations = append(observations, map[string]any{"id": c.ID, "source": parsed.source.object, "raw": parsed.source.data, "nodes": nodes, "distinct_nodes": image.count, "typecheck_or_admission_claimed": false})
		})
	}
	if parsedCount != 80 || rejectedCount != 1 {
		t.Fatal("complete original syntax matrix was narrowed")
	}
	retainPracticalSyntaxEvidence(t, "all81-originals.json", map[string]any{"parsed": parsedCount, "syntax_errors": rejectedCount, "distinct_nodes": nodeCount, "cases": observations})
}
