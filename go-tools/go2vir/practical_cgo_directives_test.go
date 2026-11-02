//go:build go1.25 && (linux || darwin)

package main

import (
	"fmt"
	"io"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

func TestPracticalOriginalCgoDirectiveHeaders(t *testing.T) {
	practicalContractTestOutput(t, "headers")
	type sample struct{ name, source string }
	cases := []sample{}
	for i, command := range []string{
		"#cgo CFLAGS: -DORIGINAL=1", "#cgo CPPFLAGS: -I${SRCDIR}/include", "#cgo CXXFLAGS: '-DNAME=original value'",
		"#cgo FFLAGS: -O2", "#cgo LDFLAGS: -L./original -Wl,$ORIGIN", "#cgo pkg-config: original >= 1.0",
		"#cgo windows UNKNOWN: 'unterminated", "#cgo linux UNKNOWN: -DVALUE", "#cgo linux,amd64 CFLAGS: -DVALUE",
		"#cgo linux,!amd64 UNKNOWN: -DVALUE", "#cgo windows linux CFLAGS: -DVALUE", "#cgo (linux&&amd64) CFLAGS: -DVALUE",
		"#cgo (linux||windows) UNKNOWN: -DVALUE", "#cgo !(linux||windows) UNKNOWN: 'bad", "#cgo !!linux UNKNOWN: invalid",
		"#cgo cgo UNKNOWN: 'bad", "#cgo !cgo CFLAGS: -DVALUE", "#cgo go1.25 CFLAGS: -DVALUE", "#cgo go1.26 UNKNOWN: 'bad",
		"#cgo amd64.v1 CFLAGS: -DVALUE", "#cgo goexperiment.dwarf5 CFLAGS: -DVALUE", "#cgo amd64.v2 UNKNOWN: 'bad",
		"#cgo CFLAGS -DVALUE", "#cgo windows UNKNOWN -DVALUE", "#cgo : -DVALUE", "#cgo CFLAGS:", "#cgo CFLAGS: \"\"",
		"#cgo CFLAGS: 'two words'", "#cgo CFLAGS: pre\"two words\"post", "#cgo CFLAGS: 'a''b'", "#cgo CFLAGS: a\\ b",
		"#cgo CFLAGS: dangling\\", "#cgo CFLAGS: 'dangling", "#cgo CFLAGS: \"dangling", "#cgo CFLAGS: \"a\\\"b\"",
		"#cgo CFLAGS: ${SRCDIR}", "#cgo CFLAGS: ${SRCDIR}${SRCDIR}", "#cgo CFLAGS: '${SRCDIR}/a b'",
		"#cgo CFLAGS: ${SRCDIR}/bad?", "#cgo CFLAGS: $UNEXPANDED", "#cgo CFLAGS: -D原型=値",
		"#cgo CFLAGS: 'control\u0085unicode'", "#cgo CFLAGS: 'bad\targument'", "#cgo CFLAGS: 'safe $@%!~^'",
		"#cgo CFLAGS: bad&arg", "#cgo CFLAGS: bad;arg", "#cgo CFLAGS: bad[arg]", "#cgo CFLAGS: bad(arg)",
		"#cgo noescape original", "#cgo nocallback original", "#cgo noescape original extra", "#cgo nocallback",
		"#cgox UNKNOWN: invalid", "#cgo\u00a0UNKNOWN: invalid", "#cgo\tCFLAGS: -DVALUE", "#cgo CFLAGS: -DVALUE\r",
		"#cgo \u2003linux\u2003 CFLAGS: -DVALUE", "#cgo linux, UNKNOWN: -DVALUE", "#cgo 名字 UNKNOWN: -DVALUE",
		"#cgo windows CFLAGS: bad?", "#cgo linux CFLAGS: bad?", "#cgo CFLAGS: first\n#cgo linux UNKNOWN: second",
	} {
		cases = append(cases, sample{fmt.Sprintf("directive-%03d", i), "package policy\n/*\n" + command + "\n*/\nimport \"C\"\nfunc Unused(){ ~~ original body ~~ }\n"})
	}
	for _, n := range []int{99, 100, 101, 102} {
		cases = append(cases, sample{fmt.Sprintf("condition-legacy-bound-%d", n), "package policy\n// #cgo " + strings.Repeat("linux,", n) + "linux UNKNOWN: -DVALUE\nimport \"C\"\n"})
	}
	for _, n := range []int{998, 999, 1000, 1001} {
		cases = append(cases, sample{fmt.Sprintf("condition-modern-bound-%d", n), "package policy\n// #cgo " + strings.Repeat("(", n) + "linux" + strings.Repeat(")", n) + " UNKNOWN: -DVALUE\nimport \"C\"\n"})
	}
	for i, source := range []string{
		"package policy\n// #cgo UNKNOWN: -DVALUE\nimport \"C\"\n",
		"package policy\n// #cgo UNKNOWN: -DVALUE\n\nimport \"C\"\n",
		"package policy; /* #cgo UNKNOWN: -DVALUE */\nimport \"C\"\n",
		"package policy\n/* #cgo UNKNOWN: -DVALUE */ import \"C\"\n",
		"package policy\n/* #cgo UNKNOWN: -DVALUE */\nimport (\"C\")\n",
		"package policy\n/* #cgo UNKNOWN: -DVALUE */\nimport (\"C\";_ \"unsafe\")\n",
		"package policy\nimport (\n// #cgo UNKNOWN: -DVALUE\n\"C\"\n)\n",
		"package policy\n// #cgo UNKNOWN: -DVALUE\nimport (\n// #cgo CFLAGS: -DORIGINAL\n\"C\"\n)\n",
		"package policy\nimport (\n/* #cgo UNKNOWN: -DVALUE */\n_ \"fmt\"\n\"C\"\n)\n",
		"package policy\nimport (\n/* #cgo UNKNOWN: -DVALUE */\n_ \"fmt\"\n// #cgo CFLAGS: -DORIGINAL\n\"C\"\n)\n",
		"package policy\n// #cgo UNKNOWN: -DVALUE\n//line renamed.go:500\nimport \"C\"\n",
		"package policy\r\n// #cgo UN\rKNOWN: -DVALUE\r\nimport \"\\x43\"\r\n",
		"package policy\n/* #cgo CFLAGS: first*\r/second */\nimport \"C\"\n",
		"package documentation\n// #cgo UNKNOWN: -DVALUE\nimport \"C\"\n",
		"//go:build windows\n\npackage policy\n// #cgo UNKNOWN: -DVALUE\nimport \"C\"\n",
		"package policy\n// #cgo UNKNOWN: -DVALUE\nimport \"fmt\"\n",
		"package policy\n//go:compilerignored\n// #cgo CFLAGS: -DORIGINAL\nimport \"C\"\n",
		"package policy\nimport \"C\"\n// #cgo UNKNOWN: -DVALUE\nvar Unused=1\n",
	} {
		cases = append(cases, sample{fmt.Sprintf("comment-group-%03d", i), source})
	}
	rows := []any{}
	for _, c := range cases {
		directory, err := filepath.EvalSymlinks(t.TempDir())
		if err != nil {
			t.Fatal(err)
		}
		files := map[string][]byte{"root.go": []byte("package policy\nfunc Root(x uint16)uint16{return x+1}\n"), "cgo.go": []byte(c.source)}
		for name, raw := range files {
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
			infos := []fs.FileInfo{}
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
		for name := range files {
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
		packet["expected"] = practicalAuxiliaryObservation(pkg, err)
		rows = append(rows, packet)
	}
	retainPracticalSyntaxEvidence(t, "cgo-directive-corpus.json", map[string]any{"cases": rows, "scope": "Actual Go1.25.0 import-C documentation validation with cgo fixed disabled. No flags/hooks/C code execute and all original source bytes remain unchanged."})
}

func TestPracticalOriginalCgoWholePackageUniverses(t *testing.T) {
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
			row := practicalAuxiliaryObservation(pkg, err)
			row["capability"] = capability
			row["directory"] = directory
			rows = append(rows, row)
		}
		units = append(units, map[string]any{"unit": unit.name, "packet": unit.file, "complete_original_objects": len(snapshot.ids), "packages": rows})
	}
	retainPracticalSyntaxEvidence(t, "original-cgo-package-files.json", map[string]any{"units": units, "scope": "Every original package directory, all original Go/excluded/auxiliary bytes and complete unchanged snapshots; cgo remains disabled."})
}

func TestPracticalOriginalCgoOwnedCapture(t *testing.T) {
	practicalContractTestOutput(t, "owned")
	owner, _ := practicalStorageTestOwnerWithOverlays(t, map[string][]byte{
		"go.mod":           []byte("module example.com/policy\ngo 1.25.0\n"),
		"policy.go":        []byte("package policy\nimport \"example.com/policy/helper\"\nfunc Root(x uint16)uint16{return helper.Work(x)}\n"),
		"helper/helper.go": []byte("package helper\nfunc Work(x uint16)uint16{return x+1}\n"),
		"cgo.go":           []byte("package policy\n// #cgo CFLAGS: -DORIGINAL\nimport \"C\"\nfunc Unused(){ unsupportedCgoOnly() }\n"),
	}, "example.com/policy", nil)
	defer owner.Close()
	input, err := owner.captureInputs([]practicalClosureRef{practicalClosureTestRef(t, owner, "example.com/policy", "Root")})
	if err != nil {
		t.Fatal(err)
	}
	retainPracticalSyntaxEvidence(t, "complete-cgo-capture.json", practicalCaptureInputEvidence(t, input))
}
