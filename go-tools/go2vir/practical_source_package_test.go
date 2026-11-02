//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/build"
	"io"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

func practicalObservePackageFiles(snapshot practicalSourceSnapshot, capability, directory string) (*build.Package, error) {
	context, err := practicalTargetBuildContext()
	if err != nil {
		return nil, err
	}
	entries, err := snapshot.readDirectory(capability, directory)
	if err != nil {
		return nil, err
	}
	context.IsDir = func(name string) bool { return name == practicalBuildDirectory }
	context.ReadDir = func(name string) ([]fs.FileInfo, error) {
		if name != practicalBuildDirectory {
			return nil, fmt.Errorf("unrecorded directory")
		}
		rows := []fs.FileInfo{}
		for _, entry := range entries {
			rows = append(rows, practicalBuildFileInfo{name: entry.Name, mode: entry.Type})
		}
		return rows, nil
	}
	context.OpenFile = func(name string) (io.ReadCloser, error) {
		if path.Dir(name) != practicalBuildDirectory {
			return nil, fmt.Errorf("unrecorded source")
		}
		buffer, err := snapshot.buildSource(practicalModuleLocation{capability: capability, directory: directory}, path.Base(name))
		if err != nil {
			return nil, err
		}
		return io.NopCloser(bytes.NewReader(buffer.data)), nil
	}
	return context.ImportDir(practicalBuildDirectory, 0)
}
func practicalPackageFileObservation(pkg *build.Package, err error) map[string]any {
	message := ""
	if err != nil {
		message = err.Error()
	}
	return map[string]any{"name": pkg.Name, "go_files": pkg.GoFiles, "ignored_go_files": pkg.IgnoredGoFiles, "test_go_files": pkg.TestGoFiles, "external_test_go_files": pkg.XTestGoFiles, "imports": pkg.Imports, "test_imports": pkg.TestImports, "external_test_imports": pkg.XTestImports, "binary_only": pkg.BinaryOnly, "invalid_go_files": pkg.InvalidGoFiles, "diagnostic": message}
}
func TestPracticalSourcePackageFilesOriginalHeaders(t *testing.T) {
	practicalContractTestOutput(t, "headers")
	type sample struct {
		name  string
		files map[string][]byte
	}
	base := []byte("package policy\nfunc Root(x uint16)uint16{return x+1}\n")
	cases := []sample{}
	for i, header := range []string{
		"package policy\nimport \"fmt\"\n", "package policy;import(\"fmt\";_ \"unsafe\")\n",
		"package policy\nimport alias \"example.com/原型\"\n", "package policy\nimport . `example.com/raw`\n",
		"package policy\nimport \"\\x43\"\n", "package policy\nimport \"C\"\n", "package documentation\n",
		"package other\n", "package _\n", "package 123\n", "package policy import \"fmt\"\n",
		"package policy\nimport (\"fmt\" \"os\")\n", "package policy\nimport (\"fmt\"\n",
		"package policy\nimport \"\"\n", "package policy\nimport \"bad path\"\n",
		"package policy\nimport \"bad\\u200dpath\"\n", "package policy\nimport \"\\ufffd\"\n",
		"package policy\nimport \"example.com/\\u00a9\"\n", "package policy\nimport \"bad\\ue000path\"\n",
		"\ufeff// original\r\npackage policy\r\nimport `example.com/raw\rpath`\r\n",
		"package policy\n//line caller-forged.go:456\nimport \"fmt\"\n",
		"/* original\n * import \"C\"\n */\npackage policy\nimport \"fmt\"\n",
		"//go:build windows\n\ninvalid header still excluded\n",
	} {
		cases = append(cases, sample{fmt.Sprintf("header-%03d", i), map[string][]byte{"root.go": base, "extra.go": []byte(header + "func Unused(){ go func(){ select{} }() }\n")}})
	}
	for _, name := range []string{"extra_test.go", "extra_windows.go", "extra_arm64.go", "_extra.go", ".extra.go"} {
		cases = append(cases, sample{"filename-" + name, map[string][]byte{"root.go": base, name: []byte("package policy\nimport \"C\"\nfunc Unused(){BROKEN BODY ~~~}\n")}})
	}
	for i, files := range []map[string][]byte{
		{"root.go": base, "a_test.go": []byte("package policy_test\nimport \"testing\"\nfunc TestBad(){ ~~ unsupported body ~~ }\n")},
		{"root.go": base, "z_test.go": []byte("package policy\nimport \"testing\"\nfunc TestBad(){ ~~ unsupported body ~~ }\n")},
		{"root.go": base, "test_test.go": []byte("package policy_test\nimport \"C\"\n")},
		{"root.go": base, "extra.go": append([]byte("package policy\nfunc Body(){"), 0xff, 0)},
		{"root.go": base, "extra_test.go": append([]byte("package policy\nfunc Body(){"), 0xff, 0)},
		{"root.go": base, "extra.go": append([]byte("//go:build windows\n// invalid comment "), append([]byte{0xff}, []byte("\n\npackage bad\n")...)...)},
		{"root.go": []byte("//go:binary-only-package\n\npackage policy\n")},
		{"only_test.go": []byte("package policy_test\nimport \"testing\"\n")},
		{"only.go": []byte("package policy\nimport \"C\"\n")},
		{"only_windows.go": base},
		{"root.go": base, "extra.go": []byte("package policy\n/* #cgo linux CFLAGS: -DORIGINAL */\nimport \"C\"\n")},
	} {
		cases = append(cases, sample{fmt.Sprintf("package-%03d", i), files})
	}
	rows := []any{}
	for _, c := range cases {
		directory, err := filepath.EvalSymlinks(t.TempDir())
		if err != nil {
			t.Fatal(err)
		}
		for name, raw := range c.files {
			if err := os.WriteFile(filepath.Join(directory, name), raw, 0600); err != nil {
				t.Fatal(err)
			}
		}
		context, err := practicalTargetBuildContext()
		if err != nil {
			t.Fatal(err)
		}
		context.IsDir = func(name string) bool { info, err := os.Stat(name); return err == nil && info.IsDir() }
		context.ReadDir = func(name string) ([]fs.FileInfo, error) {
			entries, err := os.ReadDir(name)
			if err != nil {
				return nil, err
			}
			var infos []fs.FileInfo
			for _, entry := range entries {
				info, err := entry.Info()
				if err != nil {
					return nil, err
				}
				infos = append(infos, info)
			}
			return infos, nil
		}
		context.OpenFile = func(name string) (io.ReadCloser, error) { return os.Open(name) }
		pkg, err := context.ImportDir(directory, 0)
		session, failure := newPracticalCaptureSession([]practicalSourceRoot{{Capability: "original", Directory: directory}})
		if failure != nil {
			t.Fatal(failure)
		}
		if _, failure := session.readDirectory("original", ""); failure != nil {
			t.Fatal(failure)
		}
		for name := range c.files {
			kind := "excluded_source"
			for _, active := range pkg.GoFiles {
				if active == name {
					kind = "go_file"
				}
			}
			if _, failure := session.readSource("original", name, kind); failure != nil {
				t.Fatal(failure)
			}
		}
		snapshot, failure := session.seal()
		if failure != nil {
			t.Fatal(failure)
		}
		input, failure := snapshot.snapshotInputs()
		if failure != nil {
			t.Fatal(failure)
		}
		packet := practicalSnapshotInputEvidence(input, true)
		packet["name"] = c.name
		packet["expected"] = practicalPackageFileObservation(pkg, err)
		rows = append(rows, packet)
	}
	retainPracticalSyntaxEvidence(t, "package-header-corpus.json", map[string]any{"cases": rows, "scope": "Actual Go1.25.0 ImportDir on every complete original fixture, including unsupported/malformed bodies that the header-only reader must not rewrite or parse."})
}

func practicalRestoreInputSnapshot(t *testing.T, filename string) practicalSourceSnapshot {
	t.Helper()
	raw, err := os.ReadFile(filename)
	if err != nil {
		t.Fatal(err)
	}
	var packet struct {
		Snapshot string `json:"snapshot_hex"`
		Sources  []struct {
			Object practicalSourceObject `json:"object"`
			File   string                `json:"raw_file"`
		} `json:"snapshot_sources"`
	}
	if err := json.Unmarshal(raw, &packet); err != nil {
		t.Fatal(err)
	}
	encoded, err := hex.DecodeString(packet.Snapshot)
	if err != nil {
		t.Fatal(err)
	}
	var wire struct {
		Capabilities []string                          `json:"capabilities"`
		Directories  []practicalSnapshotInputDirectory `json:"directories"`
		Digest       string                            `json:"snapshot_sha256"`
	}
	if err := json.Unmarshal(encoded, &wire); err != nil {
		t.Fatal(err)
	}
	snapshot := practicalSourceSnapshot{roots: wire.Capabilities, buffers: map[string]practicalSourceBuffer{}, paths: map[string]string{}, directories: map[string][]practicalDirectoryEntry{}}
	for _, directory := range wire.Directories {
		entries := []practicalDirectoryEntry{}
		for _, entry := range directory.Entries {
			name, err := hex.DecodeString(entry.NameHex)
			if err != nil {
				t.Fatal(err)
			}
			entries = append(entries, practicalDirectoryEntry{Name: string(name), Type: os.FileMode(entry.TypeBits)})
		}
		snapshot.directories[practicalCaptureKey(directory.Capability, directory.Path)] = entries
	}
	for _, row := range packet.Sources {
		data, err := os.ReadFile(filepath.Join(filepath.Dir(filename), "original-source-objects", row.File))
		if err != nil {
			t.Fatal(err)
		}
		buffer, err := newPracticalSourceBuffer(row.Object.Capability, row.Object.Path, row.Object.Kind, data)
		if err != nil || buffer.object != row.Object {
			t.Fatal("lost original", err)
		}
		snapshot.buffers[row.Object.ID] = buffer
		snapshot.ids = append(snapshot.ids, row.Object.ID)
		snapshot.paths[practicalCaptureKey(row.Object.Capability, row.Object.Path)] = row.Object.ID
	}
	sort.Strings(snapshot.ids)
	snapshot.contentSHA256 = practicalSnapshotDigest(snapshot)
	if snapshot.contentSHA256 != wire.Digest {
		t.Fatal("crossed original snapshot")
	}
	return snapshot
}
func TestPracticalSourcePackageFilesWholeOriginalUniverses(t *testing.T) {
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
			rows = append(rows, row)
		}
		units = append(units, map[string]any{"unit": unit.name, "packet": unit.file, "complete_original_objects": len(snapshot.ids), "packages": rows})
	}
	retainPracticalSyntaxEvidence(t, "original-package-files.json", map[string]any{"units": units, "scope": "Every package directory containing a captured active Go source in all three complete unchanged original universes. All raw source buffers and native names/types are restored exactly; no caller package/source adapter or source rewriting."})
}

func TestPracticalSourcePackageFilesKeepUnselectedOriginalFile(t *testing.T) {
	practicalContractTestOutput(t, "owned")
	owner, _ := practicalStorageTestOwnerWithOverlays(t, map[string][]byte{
		"go.mod":           []byte("module example.com/policy\ngo 1.25.0\n"),
		"policy.go":        []byte("package policy\nimport \"example.com/policy/helper\"\nfunc Root(x uint16)uint16{return helper.Work(x)}\n"),
		"unselected.go":    []byte("//go:build go1.21\n\npackage policy\nfunc Unused(c chan int,f func(int)int)int{return f(<-c)}\nfunc init(){panic(\"not executed\")}\n"),
		"helper/helper.go": []byte("package helper\nfunc Work(x uint16)uint16{return x+1}\n"),
	}, "example.com/policy", nil)
	defer owner.Close()
	input, err := owner.captureInputs([]practicalClosureRef{practicalClosureTestRef(t, owner, "example.com/policy", "Root")})
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "complete-package-inputs.json", practicalCaptureInputEvidence(t, input))
}
