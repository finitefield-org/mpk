//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"strings"
	"testing"
)

func practicalObserveSourceBuild(t *testing.T, name string, raw []byte) (bool, string, bool) {
	t.Helper()
	context, err := practicalTargetBuildContext()
	if err != nil {
		t.Fatal(err)
	}
	context.OpenFile = func(filename string) (io.ReadCloser, error) {
		if filename != path.Join(practicalBuildDirectory, name) {
			return nil, fmt.Errorf("unrecorded source read: %s", filename)
		}
		return io.NopCloser(bytes.NewReader(raw)), nil
	}
	matched, err := context.MatchFile(practicalBuildDirectory, name)
	if err != nil {
		return matched, err.Error(), false
	}
	context.IsDir = func(directory string) bool { return directory == practicalBuildDirectory }
	context.ReadDir = func(directory string) ([]fs.FileInfo, error) {
		if directory != practicalBuildDirectory {
			return nil, fmt.Errorf("unrecorded directory read")
		}
		return []fs.FileInfo{practicalBuildFileInfo{name: name, mode: 0444, size: int64(len(raw))}}, nil
	}
	packageInfo, _ := context.ImportDir(practicalBuildDirectory, 0)
	return matched, "", packageInfo != nil && packageInfo.BinaryOnly
}

func TestPracticalSourceBuildMatchesPinnedTarget(t *testing.T) {
	practicalContractTestOutput(t, "probe")
	type sample struct{ name, file, header string }
	cases := []sample{}
	for _, system := range strings.Fields("aix android darwin dragonfly freebsd hurd illumos ios js linux nacl netbsd openbsd plan9 solaris wasip1 windows zos") {
		cases = append(cases, sample{"filename-os-" + system, "original_" + system + ".go", ""}, sample{"filename-both-" + system, "original_" + system + "_amd64.go", ""}, sample{"os-tag-" + system, "original.go", "//go:build " + system + "\n\n"})
	}
	for _, architecture := range strings.Fields("386 amd64 amd64p32 arm armbe arm64 arm64be loong64 mips mipsle mips64 mips64le mips64p32 mips64p32le ppc ppc64 ppc64le riscv riscv64 s390 s390x sparc sparc64 wasm") {
		cases = append(cases, sample{"filename-arch-" + architecture, "original_" + architecture + ".go", ""}, sample{"arch-tag-" + architecture, "original.go", "//go:build " + architecture + "\n\n"})
	}
	for _, tag := range append(append([]string{}, practicalTargetToolTags[:]...), strings.Fields("go1 go1.0 go1.01 go1.1 go1.21 go1.22 go1.23 go1.24 go1.25 go1.26 go1.999999999999999999999999 gc gccgo unix cgo boringcrypto goexperiment.boringcrypto goexperiment.coverageredesign goexperiment.loopvar amd64.v2 race msan asan custom 名字")...) {
		cases = append(cases, sample{"tag-" + tag, "original.go", "//go:build " + tag + "\n\n"})
	}
	for _, file := range []string{"linux.go", "windows.go", "linux_amd64.go", "original_unknown.go", "original_linux_amd64_test.go", "original_linux_arm64.go", "original_arm64_linux.go", "original_linux.amd64.go", "original_test.go", "_original.go", ".original.go", "original.GO", "original.go.txt", "original.go.go"} {
		cases = append(cases, sample{"filename-" + file, file, ""})
	}
	headers := []string{"//go:build linux && amd64 && !cgo\n\n", "//go:build windows || (unix && !arm64)\n\n", "//go:build !(linux || windows)\n\n", "//go:build !!linux\n\n", "//go:build linux &&\n\n", "//go:build linux\n//go:build amd64\n\n", "//go:build\n\n", "//go:buildsomethingelse windows\n", "//go:buildsomethingelse windows \t\n", "//go:build\u00a0linux\u00a0\n\n", "\ufeff//go:build linux\r\n\r\n", "//go:build lin\rux\n\n", "/* original\n//go:build windows\n*/\n", "/* original */\n//go:build windows\n\n", "// +build windows\n\n", "// +build windows\n", "// +build linux,amd64 windows\n\n", "//+build linux\n// +build !cgo\n\n", "// +build !!linux\n\n", "// +build !bad!tag\n\n", "// +build\n\n", "// +build linux\n// +build windows\n\n", "//go:build linux\n// +build windows\n\n", "//go:binary-only-package\n\n"}
	for i, h := range headers {
		cases = append(cases, sample{fmt.Sprintf("header-%03d", i), "original.go", h})
	}
	for _, n := range []int{99, 100, 101, 102} {
		cases = append(cases, sample{fmt.Sprintf("legacy-size-%d", n), "original.go", "// +build " + strings.Repeat("windows,", n) + "windows\n\n"})
	}
	for _, n := range []int{998, 999, 1000, 1001} {
		cases = append(cases, sample{fmt.Sprintf("modern-depth-%d", n), "original.go", "//go:build " + strings.Repeat("(", n) + "linux" + strings.Repeat(")", n) + "\n\n"})
	}
	cases = append(cases, sample{"ignored-invalid-constraint", "original_windows.go", "//go:build !!linux\n\n"}, sample{"test-binary-marker", "original_test.go", "//go:binary-only-package\n\n"})
	rows := []any{}
	for _, c := range cases {
		raw := []byte(c.header + "package original\nfunc Root(x uint16)uint16{return x+1}\n")
		buffer, err := newPracticalSourceBuffer("original", c.file, "go_file", raw)
		if err != nil {
			t.Fatal(c.name, err)
		}
		matched, diagnostic, binaryOnly := practicalObserveSourceBuild(t, c.file, raw)
		rows = append(rows, map[string]any{"name": c.name, "object": buffer.object, "raw_hex": hex.EncodeToString(raw), "matched": matched, "diagnostic": diagnostic, "binary_only": binaryOnly})
	}
	retainPracticalSyntaxEvidence(t, "build-corpus.json", map[string]any{"cases": rows, "fixed_target": "linux/amd64", "tool_tags": practicalTargetToolTags, "scope": "Actual Go1.25.0 build.Context.MatchFile on every unchanged original byte stream; no package, cgo, source-program or context proof."})
}

func TestPracticalSourceBuildMatchesWholeOriginalCapturedUnits(t *testing.T) {
	practicalContractTestOutput(t, "originals")
	root := filepath.Join("..", "..", "develop", "migrations", "go-practical", "t02-w01", "verification", "snapshot-inputs")
	rows := []any{}
	for _, unit := range []struct{ name, file string }{
		{"pure", "native/native-originals/pure/complete-capture-inputs.json"},
		{"wide", "native/native-originals/wide/complete-capture-inputs.json"},
		{"owned", "native-final/native-originals/owned/capture-inputs.json"},
	} {
		raw, err := os.ReadFile(filepath.Join(root, unit.file))
		if err != nil {
			t.Fatal(err)
		}
		var packet struct {
			Sources []struct {
				Object practicalSourceObject `json:"object"`
				File   string                `json:"raw_file"`
			} `json:"snapshot_sources"`
		}
		if err := json.Unmarshal(raw, &packet); err != nil {
			t.Fatal(err)
		}
		candidates := []any{}
		for _, source := range packet.Sources {
			if source.Object.Kind != "go_file" && source.Object.Kind != "excluded_source" {
				continue
			}
			filename := filepath.Join(root, filepath.Dir(unit.file), "original-source-objects", source.File)
			original, err := os.ReadFile(filename)
			if err != nil {
				t.Fatal(err)
			}
			buffer, err := newPracticalSourceBuffer(source.Object.Capability, source.Object.Path, source.Object.Kind, original)
			if err != nil || buffer.object != source.Object {
				t.Fatal("lost complete original source", source.Object.Path, err)
			}
			matched, diagnostic, binaryOnly := practicalObserveSourceBuild(t, path.Base(source.Object.Path), original)
			if source.Object.Kind == "go_file" && (!matched || diagnostic != "") {
				t.Fatal("original active Go source mismatched fixed target", source.Object.Path, diagnostic)
			}
			candidates = append(candidates, map[string]any{"object": source.Object, "raw_file": source.File, "matched": matched, "diagnostic": diagnostic, "binary_only": binaryOnly})
		}
		rows = append(rows, map[string]any{"unit": unit.name, "packet": unit.file, "complete_original_objects": len(packet.Sources), "candidates": candidates})
	}
	retainPracticalSyntaxEvidence(t, "original-build-candidates.json", map[string]any{"units": rows, "scope": "Every Go/excluded-source object from all three unchanged complete original captured universes; other original source kinds remain retained in the complete snapshot. No original buffer, body, root or error universe is reduced."})
}
