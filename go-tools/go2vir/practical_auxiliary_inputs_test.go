//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/hex"
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

func practicalAuxiliaryObservation(pkg *build.Package, err error) map[string]any {
	row := practicalPackageFileObservation(pkg, err)
	for key, value := range map[string][]string{"c": pkg.CFiles, "cxx": pkg.CXXFiles, "objective_c": pkg.MFiles, "headers": pkg.HFiles, "fortran": pkg.FFiles, "assembly": pkg.SFiles, "swig": pkg.SwigFiles, "swig_cxx": pkg.SwigCXXFiles, "system_objects": pkg.SysoFiles, "ignored_auxiliary": pkg.IgnoredOtherFiles} {
		row[key] = value
	}
	return row
}
func TestPracticalAuxiliaryBuildOriginalCases(t *testing.T) {
	practicalContractTestOutput(t, "cases")
	type sample struct {
		name, filename string
		raw            []byte
	}
	cases := []sample{}
	for _, extension := range strings.Fields("c cc cpp cxx m h hh hpp hxx f F for f90 s S sx swig swigcxx syso") {
		for _, suffix := range []string{"", "_linux_amd64", "_windows", "_arm64", "_test", "_linux.arm64"} {
			cases = append(cases, sample{"extension-" + extension + suffix, "original" + suffix + "." + extension, []byte("// original untouched auxiliary body\nORIGINAL_BODY\n")})
		}
	}
	for _, extension := range []string{"s", "h", "syso", "S"} {
		for i, raw := range [][]byte{
			[]byte("//go:build windows\n\nRAW_BODY\n"), []byte("//go:build linux && !cgo\n\nRAW_BODY\n"),
			[]byte("// +build windows\n\nRAW_BODY\n"), []byte("// +build windows\nRAW_BODY\n"),
			[]byte("//go:build !!linux\n\nRAW_BODY\n"), []byte("//go:build windows\n//go:build linux\n\nRAW_BODY\n"),
			[]byte("//go:binary-only-package\n\nRAW_BODY\n"), []byte("\ufeff//go:build windows\r\n\r\nRAW_BODY\n"),
			[]byte("\u00a0//go:build windows\n\nRAW_BODY\n"), []byte("; //go:build windows\n\nRAW_BODY\n"),
			[]byte("//go:build windows\n/* unterminated"), []byte("//go:build windows\n// nul\x00\n\nRAW_BODY\n"),
			[]byte("// comment\n\nRAW\x00\xff_BODY\n"), []byte("//go:build windows\n// original \xff comment\n\nRAW_BODY\n"),
		} {
			cases = append(cases, sample{fmt.Sprintf("header-%s-%02d", extension, i), "original." + extension, raw})
		}
	}
	for _, name := range []string{"_hidden.s", ".hidden.h", "asset.bin", "c", "h", "syso", "foo.GO", "foo.syso.txt"} {
		cases = append(cases, sample{"ignored-name-" + name, name, []byte("//go:build !!linux\n\x00\xff\n")})
	}
	rows := []any{}
	for _, c := range cases {
		directory, err := filepath.EvalSymlinks(t.TempDir())
		if err != nil {
			t.Fatal(err)
		}
		files := map[string][]byte{"root.go": []byte("package policy\nfunc Root(x uint16)uint16{return x+1}\n"), c.filename: c.raw}
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
			kind := "auxiliary"
			if name == "root.go" {
				kind = "go_file"
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
		packet["unchanged_auxiliary_hex"] = hex.EncodeToString(c.raw)
		rows = append(rows, packet)
	}
	retainPracticalSyntaxEvidence(t, "auxiliary-corpus.json", map[string]any{"cases": rows, "scope": "Actual original Go1.25.0 ImportDir auxiliary build input selection. All original buffers remain untouched; no C/assembly/other program-language semantics or public profile is supplied."})
}
func TestPracticalAuxiliaryBuildWholeOriginalUniverses(t *testing.T) {
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
	retainPracticalSyntaxEvidence(t, "original-auxiliary-files.json", map[string]any{"units": units, "scope": "Every actual package directory in all three unchanged complete original captured universes, with all nine auxiliary classes and ignored-other inputs. No original source, root/type/error universe or proof bytes are reduced."})
}
