//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"io/fs"
	"os"
	"os/exec"
	"path"
	"path/filepath"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"testing"
)

func practicalEmbeddedPatternHex(patterns []string) []string {
	result := []string{}
	for _, pattern := range patterns {
		result = append(result, hex.EncodeToString([]byte(pattern)))
	}
	return result
}

// Observe all original case files before any resource-selection failure. The
// source bytes and actual native names/types are retained without an adapter.
func practicalEmbeddedSelectionSnapshot(t *testing.T, directory string) practicalSourceSnapshot {
	t.Helper()
	session, err := newPracticalCaptureSession([]practicalSourceRoot{{Capability: "original", Directory: directory}})
	if err != nil {
		t.Fatal(err)
	}
	err = filepath.WalkDir(directory, func(filename string, entry fs.DirEntry, failure error) error {
		if failure != nil {
			return failure
		}
		relative, err := filepath.Rel(directory, filename)
		if err != nil {
			return err
		}
		if relative == "." {
			relative = ""
		}
		relative = filepath.ToSlash(relative)
		if entry.IsDir() {
			_, err = session.readDirectory("original", relative)
			return err
		}
		if !entry.Type().IsRegular() {
			return nil
		}
		kind := "auxiliary"
		if relative == "go.mod" {
			kind = "go_mod"
		}
		if path.Dir(relative) == "." && strings.HasSuffix(relative, ".go") {
			kind = "go_file"
			if strings.HasSuffix(relative, "_test.go") {
				kind = "excluded_source"
			}
		}
		_, err = session.readSource("original", relative, kind)
		return err
	})
	if err != nil {
		t.Fatal(err)
	}
	snapshot, err := session.seal()
	if err != nil {
		t.Fatal(err)
	}
	return snapshot
}

func TestPracticalEmbeddedSelectionOriginalCandidates(t *testing.T) {
	practicalContractTestOutput(t, "matrix")
	filename := filepath.Join("..", "..", "develop", "migrations", "go-practical", "t02-w01", "verification", "embedded-inputs", "native", "native-originals", "matrix", "embedded-source-corpus.json")
	raw, err := os.ReadFile(filename)
	if err != nil {
		t.Fatal(err)
	}
	var corpus struct {
		Cases []struct {
			Name     string            `json:"case"`
			Files    map[string][]byte `json:"whole_original_files"`
			Links    map[string]string `json:"links"`
			Accepted bool              `json:"accepted"`
		} `json:"cases"`
	}
	if err := json.Unmarshal(raw, &corpus); err != nil {
		t.Fatal(err)
	}
	rows := []any{}
	for index, original := range corpus.Cases {
		directory, err := filepath.EvalSymlinks(t.TempDir())
		if err != nil {
			t.Fatal(err)
		}
		for name, data := range original.Files {
			writePracticalTestFile(t, filepath.Join(directory, filepath.FromSlash(name)), data, 0600)
		}
		for name, target := range original.Links {
			if err := os.MkdirAll(filepath.Dir(filepath.Join(directory, name)), 0700); err != nil {
				t.Fatal(err)
			}
			if err := os.Symlink(target, filepath.Join(directory, name)); err != nil {
				t.Fatal(err)
			}
		}
		snapshot := practicalEmbeddedSelectionSnapshot(t, directory)
		input, err := snapshot.snapshotInputs()
		if err != nil {
			t.Fatal(err)
		}
		pkg, buildErr := practicalObservePackageFiles(snapshot, "original", "")
		command := exec.Command(filepath.Join(runtime.GOROOT(), "bin", "go"), "list", "-json", ".")
		command.Dir = directory
		command.Env = []string{"PATH=/usr/bin:/bin", "GOROOT=" + runtime.GOROOT(), "GOTOOLCHAIN=local", "GOENV=off", "GOWORK=off", "GO111MODULE=on", "GOFLAGS=-mod=readonly", "GOPROXY=off", "GOSUMDB=off", "CGO_ENABLED=0", "GODEBUG=", "GOMAXPROCS=1", "GOOS=linux", "GOARCH=amd64", "GOAMD64=v1", "GOEXPERIMENT=", "GOCACHE=" + os.Getenv("GOCACHE"), "GOMODCACHE=" + os.Getenv("GOMODCACHE")}
		var stdout, stderr bytes.Buffer
		command.Stdout = &stdout
		command.Stderr = &stderr
		nativeErr := command.Run()
		originalRow := map[string]any{"case": original.Name, "snapshot": practicalSnapshotInputEvidence(input, true), "original_files": original.Files, "links": original.Links, "native_build": practicalPackageFileObservation(pkg, buildErr), "native_patterns_hex": practicalEmbeddedPatternHex(pkg.EmbedPatterns), "accepted": nativeErr == nil, "native_stdout": stdout.Bytes(), "native_stderr": stderr.Bytes()}
		retainPracticalSyntaxEvidence(t, fmt.Sprintf("original-native-case-%02d.json", index), originalRow)
		if buildErr != nil {
			t.Fatal(original.Name, buildErr)
		}
		if (nativeErr == nil) != original.Accepted {
			t.Fatal("original case/native disposition changed", original.Name, nativeErr, stderr.String())
		}
		files := []string{}
		if nativeErr == nil {
			var native practicalGoPackageExport
			if err := json.Unmarshal(stdout.Bytes(), &native); err != nil {
				t.Fatal(err)
			}
			files = native.EmbedFiles
		}
		originalRow["native_selected_files"] = files
		rows = append(rows, originalRow)
		if err := os.RemoveAll(directory); err != nil {
			t.Fatal(err)
		}
	}
	retainPracticalSyntaxEvidence(t, "complete-embedded-candidates.json", map[string]any{"cases": rows, "original_corpus_raw_hex": hex.EncodeToString(raw), "original_roots_removed": true, "caller_source_adapter_or_fallback": false})
}

func TestPracticalEmbeddedDirectivesOriginalBuild(t *testing.T) {
	practicalContractTestOutput(t, "headers")
	type sample struct {
		name   string
		source []byte
	}
	cases := []sample{}
	prefix := "package policy\nimport _ \"embed\"\n"
	suffix := "\nvar Unused string\nfunc Root(x uint16)uint16{return x+1}\n"
	for i, line := range []string{
		"//go:embed assets", "\t //go:embed assets/*.txt", "//go:embed \"assets/a.txt\" `assets/b.txt`",
		"//go:embed assets/a.txt assets/a.txt", "//go:embed \"assets/\\x61.txt\"", "//go:embed \"assets/\\141.txt\"", "//go:embed \"assets/\\u539f.txt\"",
		"//go:embed `assets/a\rb.txt`", "//go:embed assets/a.txt\u00a0assets/b.txt", "//go:embed assets/a.txt\u2003assets/b.txt",
		"//go:embed \"\\xff\"", "//go:embed \"\\uFFFD\"", "//go:embed \"\\uD800\"", "//go:embed \"\\400\"", "//go:embed \"\\x4\"",
		"//go:embed \"assets\"suffix", "//go:embed assets `unterminated", "//go:embed \"unterminated", "//go:embed", "//go:embed ",
		"//go:embed\vassets", "//go:embed\fassets", "//go:embed \vassets", "\f//go:embed assets", "//go:embeds assets", "///go:embed assets",
		"/* original */ //go:embed ignored\n//go:embed assets", "/* original\n */ //go:embed ignored\n//go:embed assets",
		"var _ = 0 //go:embed ignored\n//go:embed assets", "//line forged.go:999\n//go:embed assets", "//go:embed assets\r",
		"//go:embed assets\n//go:embed other", "//go:embed assets\n//go:embed \"unterminated",
		"var _ = `\n//go:embed ignored\n`\n//go:embed assets", "var _ = \"//go:embed ignored\"\n//go:embed assets", "var _ = '/'\n//go:embed assets",
		"/* //go:embed ignored */\n//go:embed assets", "// original //go:embed ignored\n//go:embed assets",
		"var _ = \"unfinished\n//go:embed ignored", "/* unfinished\n//go:embed ignored",
	} {
		cases = append(cases, sample{fmt.Sprintf("line-%02d", i), []byte(prefix + line + suffix)})
	}
	for i, source := range []string{
		"//go:embed ignored\n" + prefix + "//go:embed assets" + suffix,
		"// original\n//go:embed beforepackage\n" + prefix + "//go:embed assets" + suffix,
		"package policy\n//go:embed beforeimport\nimport _ \"embed\"\n//go:embed assets" + suffix,
		"package policy\n//go:embed ignored" + suffix,
		"package policy\nimport e `embed`\n//go:embed assets" + suffix,
		"\ufeff" + prefix + "//go:embed assets" + suffix,
		prefix + "//go:embed assets\nvar _ = 0\x00\n//go:embed ignored" + suffix,
		prefix + "//go:embed assets\x00ignored" + suffix,
		prefix + "//go:embed \"original\xffbyte\"" + suffix,
		prefix + "//go:embed \"literal\xe2\x82\"" + suffix,
		prefix + "//go:embed \"\\xff-literal\xff\"" + suffix,
		prefix + "var First int\n//go:embed \"original\xffbyte\"" + suffix,
		prefix + "var First int\n//go:embed \"literal\xe2\x82\"" + suffix,
		prefix + "var First int\n//go:embed \"\\xff-literal\xff\"" + suffix,
	} {
		cases = append(cases, sample{fmt.Sprintf("source-%02d", i), []byte(source)})
	}
	rows := []any{}
	for _, c := range cases {
		directory, err := filepath.EvalSymlinks(t.TempDir())
		if err != nil {
			t.Fatal(err)
		}
		writePracticalTestFile(t, filepath.Join(directory, "policy.go"), c.source, 0600)
		snapshot := practicalEmbeddedSelectionSnapshot(t, directory)
		input, err := snapshot.snapshotInputs()
		if err != nil {
			t.Fatal(err)
		}
		pkg, buildErr := practicalObservePackageFiles(snapshot, "original", "")
		row := practicalSnapshotInputEvidence(input, true)
		row["case"] = c.name
		row["original_source_hex"] = hex.EncodeToString(c.source)
		row["native_observation"] = practicalPackageFileObservation(pkg, buildErr)
		row["native_patterns_hex"] = practicalEmbeddedPatternHex(pkg.EmbedPatterns)
		rows = append(rows, row)
		if err := os.RemoveAll(directory); err != nil {
			t.Fatal(err)
		}
	}
	retainPracticalSyntaxEvidence(t, "original-embedded-directives.json", map[string]any{"cases": rows, "original_roots_removed": true, "native_build_inventory_only": true, "compiler_pragma_or_program_proof": false})
}

func TestPracticalEmbeddedPatternsWholeOriginalUniverses(t *testing.T) {
	practicalContractTestOutput(t, "originals")
	root := filepath.Join("..", "..", "develop", "migrations", "go-practical", "t02-w01", "verification", "snapshot-inputs")
	units := []any{}
	for _, unit := range []struct{ name, file string }{{"pure", "native/native-originals/pure/complete-capture-inputs.json"}, {"wide", "native/native-originals/wide/complete-capture-inputs.json"}, {"owned", "native-final/native-originals/owned/capture-inputs.json"}} {
		snapshot := practicalRestoreInputSnapshot(t, filepath.Join(root, unit.file))
		locations := map[string]bool{}
		for _, source := range snapshot.buffers {
			if source.object.Kind == "go_file" {
				directory := path.Dir(source.object.Path)
				if directory == "." {
					directory = ""
				}
				locations[practicalCaptureKey(source.object.Capability, directory)] = true
			}
		}
		keys := []string{}
		for key := range locations {
			keys = append(keys, key)
		}
		sort.Strings(keys)
		rows := []any{}
		for _, key := range keys {
			capability, directory, _ := strings.Cut(key, "\x00")
			pkg, err := practicalObservePackageFiles(snapshot, capability, directory)
			row := practicalPackageFileObservation(pkg, err)
			row["capability"] = capability
			row["directory"] = directory
			row["native_patterns_hex"] = practicalEmbeddedPatternHex(pkg.EmbedPatterns)
			row["native_test_patterns_hex"] = practicalEmbeddedPatternHex(pkg.TestEmbedPatterns)
			row["native_external_test_patterns_hex"] = practicalEmbeddedPatternHex(pkg.XTestEmbedPatterns)
			rows = append(rows, row)
		}
		units = append(units, map[string]any{"unit": unit.name, "packet": unit.file, "complete_original_objects": len(snapshot.ids), "packages": rows})
	}
	retainPracticalSyntaxEvidence(t, "whole-original-embedded-patterns.json", map[string]any{"units": units, "all_original_package_directories": true})
}

func TestPracticalEmbeddedGlobOriginalPinnedCases(t *testing.T) {
	practicalContractTestOutput(t, "glob")
	filename := filepath.Join(runtime.GOROOT(), "src", "path", "match_test.go")
	raw, err := os.ReadFile(filename)
	if err != nil {
		t.Fatal(err)
	}
	file, err := parser.ParseFile(token.NewFileSet(), filename, raw, 0)
	if err != nil {
		t.Fatal(err)
	}
	rows := []any{}
	for _, decl := range file.Decls {
		gen, ok := decl.(*ast.GenDecl)
		if !ok {
			continue
		}
		for _, spec := range gen.Specs {
			v, ok := spec.(*ast.ValueSpec)
			if !ok || len(v.Names) != 1 || v.Names[0].Name != "matchTests" {
				continue
			}
			for i, elt := range v.Values[0].(*ast.CompositeLit).Elts {
				c := elt.(*ast.CompositeLit)
				pattern, err := strconv.Unquote(c.Elts[0].(*ast.BasicLit).Value)
				if err != nil {
					t.Fatal(err)
				}
				name, err := strconv.Unquote(c.Elts[1].(*ast.BasicLit).Value)
				if err != nil {
					t.Fatal(err)
				}
				matched, matchErr := path.Match(pattern, name)
				if matched != (c.Elts[2].(*ast.Ident).Name == "true") || (matchErr != nil) != (c.Elts[3].(*ast.Ident).Name != "nil") {
					retainPracticalSyntaxEvidence(t, "unexpected-original-native-glob.json", map[string]any{"case": i, "pattern_hex": hex.EncodeToString([]byte(pattern)), "name_hex": hex.EncodeToString([]byte(name)), "matched": matched, "bad_pattern": matchErr != nil, "complete_original_test_source_hex": hex.EncodeToString(raw)})
					t.Fatal("pinned native expected disposition changed", i)
				}
				rows = append(rows, map[string]any{"case": i, "pattern_hex": hex.EncodeToString([]byte(pattern)), "name_hex": hex.EncodeToString([]byte(name)), "matched": matched, "bad_pattern": matchErr != nil})
			}
		}
	}
	if len(rows) == 0 {
		t.Fatal("pinned original corpus absent")
	}
	originalCount := len(rows)
	for i, c := range []struct{ pattern, name string }{
		{"*?*[\ufffd]", "中"}, {"*?*?", "中"}, {"*[\ufffd]*[\ufffd]", "中"}, {"?", "\xff"}, {"*", "\xffa"}, {"\ufffd", "\xff"}, {"[\ufffd]", "\xff"}, {"*?x", "中\xffx"},
		{"[a/b]", "/"}, {"a*", "a\x00b"}, {"\xff", "\xff"}, {"[\xff]", "a"}, {"*a[", "a"}, {"", ""}, {"", "a"}, {"[z-a]", "m"}, {"*\xfe", "a\xfe"},
	} {
		matched, err := path.Match(c.pattern, c.name)
		rows = append(rows, map[string]any{"case": fmt.Sprintf("bytes-%02d", i), "pattern_hex": hex.EncodeToString([]byte(c.pattern)), "name_hex": hex.EncodeToString([]byte(c.name)), "matched": matched, "bad_pattern": err != nil})
	}
	retainPracticalSyntaxEvidence(t, "original-pinned-globs.json", map[string]any{"cases": rows, "all_original_pinned_cases": originalCount, "additional_actual_raw_byte_cases": len(rows) - originalCount, "complete_original_test_source_hex": hex.EncodeToString(raw), "original_source_path": "src/path/match_test.go"})
}

func TestPracticalEmbeddedPatternGroupsOriginalBuild(t *testing.T) {
	practicalContractTestOutput(t, "groups")
	production := []byte("package policy\nimport _ \"embed\"\n//go:embed assets/prod.txt\nvar Unused string\nfunc Root(x uint16)uint16{return x+1}\n")
	internal := []byte("package policy\nimport _ \"embed\"\n//go:embed [\nvar Internal string\n")
	external := []byte("package policy_test\nimport _ \"embed\"\n//go:embed missing-external all:missing-external\nvar External string\n")
	rows := []any{}
	for index, tests := range []map[string][]byte{{"a_test.go": internal}, {"a_test.go": external}, {"a_test.go": external, "b_test.go": internal}} {
		directory, err := filepath.EvalSymlinks(t.TempDir())
		if err != nil {
			t.Fatal(err)
		}
		files := map[string][]byte{"go.mod": []byte("module example.com/policy\ngo 1.25.0\n"), "policy.go": production, "assets/prod.txt": []byte("all original production bytes\x00\xff\n")}
		for name, data := range tests {
			files[name] = data
		}
		for name, data := range files {
			writePracticalTestFile(t, filepath.Join(directory, filepath.FromSlash(name)), data, 0600)
		}
		snapshot := practicalEmbeddedSelectionSnapshot(t, directory)
		input, err := snapshot.snapshotInputs()
		if err != nil {
			t.Fatal(err)
		}
		pkg, buildErr := practicalObservePackageFiles(snapshot, "original", "")
		command := exec.Command(filepath.Join(runtime.GOROOT(), "bin", "go"), "list", "-json", ".")
		command.Dir = directory
		command.Env = []string{"PATH=/usr/bin:/bin", "GOROOT=" + runtime.GOROOT(), "GOTOOLCHAIN=local", "GOENV=off", "GOWORK=off", "GO111MODULE=on", "GOFLAGS=-mod=readonly", "GOPROXY=off", "GOSUMDB=off", "CGO_ENABLED=0", "GODEBUG=", "GOMAXPROCS=1", "GOOS=linux", "GOARCH=amd64", "GOAMD64=v1", "GOEXPERIMENT=", "GOCACHE=" + os.Getenv("GOCACHE"), "GOMODCACHE=" + os.Getenv("GOMODCACHE")}
		var stdout, stderr bytes.Buffer
		command.Stdout = &stdout
		command.Stderr = &stderr
		nativeErr := command.Run()
		row := map[string]any{"case": index, "snapshot": practicalSnapshotInputEvidence(input, true), "original_files": files, "native_build": practicalPackageFileObservation(pkg, buildErr), "native_patterns_hex": practicalEmbeddedPatternHex(pkg.EmbedPatterns), "native_test_patterns_hex": practicalEmbeddedPatternHex(pkg.TestEmbedPatterns), "native_external_test_patterns_hex": practicalEmbeddedPatternHex(pkg.XTestEmbedPatterns), "native_stdout": stdout.Bytes(), "native_stderr": stderr.Bytes(), "accepted": nativeErr == nil}
		retainPracticalSyntaxEvidence(t, fmt.Sprintf("original-group-%02d.json", index), row)
		if nativeErr != nil || buildErr != nil {
			t.Fatal(index, nativeErr, buildErr, stderr.String())
		}
		var native practicalGoPackageExport
		if err := json.Unmarshal(stdout.Bytes(), &native); err != nil {
			t.Fatal(err)
		}
		if len(native.EmbedFiles) != 1 || native.EmbedFiles[0] != "assets/prod.txt" {
			t.Fatal("test patterns contaminated original production resources", native.EmbedFiles)
		}
		row["native_selected_files"] = native.EmbedFiles
		rows = append(rows, row)
		if err := os.RemoveAll(directory); err != nil {
			t.Fatal(err)
		}
	}
	retainPracticalSyntaxEvidence(t, "original-test-pattern-groups.json", map[string]any{"cases": rows, "original_roots_removed": true, "test_patterns_not_production_resources": true})
}
