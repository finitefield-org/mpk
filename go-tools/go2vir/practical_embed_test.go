//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"maps"
	"os"
	"os/exec"
	"path/filepath"
	"reflect"
	"runtime"
	"strconv"
	"strings"
	"testing"
)

func TestPracticalEmbeddedSourcesMatchOriginalGoCommand(t *testing.T) {
	practicalContractTestOutput(t, "matrix")
	type sample struct {
		name     string
		patterns []string
		files    map[string][]byte
		links    map[string]string
	}
	base := map[string][]byte{"data/a.txt": []byte("original\x00\xff bytes\n"), "data/b.bin": {0, 1, 2, 255}, "data/deep/c.txt": []byte("nested source\n"), "data/.hidden": []byte("hidden original\n"), "data/_private": []byte("private original\n"), "data/.hidden-dir/file.txt": []byte("nested hidden\n"), "data/_private-dir/file.txt": []byte("nested private\n"), "other/asset.txt": []byte("other exact bytes\n")}
	cases := []sample{}
	for i, patterns := range [][]string{{"data"}, {"all:data"}, {"data/*"}, {"data/*.txt"}, {"data/*/*.txt"}, {"data/**"}, {"data/.hidden"}, {"data/_private"}, {"data/.hidden-dir"}, {"data/deep", "data/*.txt"}, {"data/deep", "data/deep"}, {"policy.go"}, {"go.mod"}, {"data/a.txt", "other/asset.txt"}, {"missing"}, {"missing/*"}, {"data/*.missing"}, {"."}, {"../data"}, {"/data"}, {"data//a.txt"}, {"data/["}, {"all:"}, {"data/?.txt"}, {"data/[ab].*"}, {"data/[^a].*"}, {"data/\\a.txt"}} {
		cases = append(cases, sample{fmt.Sprintf("pattern-%02d", i), patterns, maps.Clone(base), nil})
	}
	for _, name := range []string{"data/has space.txt", "data/原型.txt", "data/CON.txt", "data/COM1", "data/name.", "data/bad?.txt", "data/bad:name", "data/.git/file", "data/.git", "data/~short1.txt"} {
		files := map[string][]byte{name: []byte("exact original\n")}
		cases = append(cases, sample{"direct-name-" + name, []string{name}, files, nil}, sample{"walk-name-" + name, []string{"all:data"}, files, nil})
	}
	cases = append(cases,
		sample{"nested-module-walk", []string{"data"}, map[string][]byte{"data/root.txt": []byte("root\n"), "data/module/go.mod": []byte("module example.com/nested\ngo 1.25.0\n"), "data/module/file.txt": []byte("nested module\n")}, nil},
		sample{"nested-module-direct", []string{"data/module/file.txt"}, map[string][]byte{"data/module/go.mod": []byte("module example.com/nested\ngo 1.25.0\n"), "data/module/file.txt": []byte("nested module\n")}, nil},
		sample{"linked-leaf-direct", []string{"data/link.txt"}, map[string][]byte{"data/original.txt": []byte("original\n")}, map[string]string{"data/link.txt": "original.txt"}},
		sample{"linked-leaf-walk", []string{"data"}, map[string][]byte{"data/original.txt": []byte("original\n")}, map[string]string{"data/link.txt": "original.txt"}},
		sample{"linked-directory", []string{"data/link/file.txt"}, map[string][]byte{"other/file.txt": []byte("original\n")}, map[string]string{"data/link": "../other"}},
		sample{"hidden-only-empty", []string{"data"}, map[string][]byte{"data/.hidden": []byte("hidden\n")}, nil},
		sample{"unrelated-asset", []string{"data"}, map[string][]byte{"data/file.txt": []byte("original\n"), "unrelated/bad:name": []byte("unrelated native asset\n")}, nil},
	)
	rows := []any{}
	for _, c := range cases {
		directory, err := filepath.EvalSymlinks(t.TempDir())
		if err != nil {
			t.Fatal(err)
		}
		patterns := []string{}
		for _, p := range c.patterns {
			patterns = append(patterns, strconv.Quote(p))
		}
		source := []byte("package policy\nimport \"embed\"\n//go:embed " + strings.Join(patterns, " ") + "\nvar Unused embed.FS\nfunc Root(x uint16)uint16{return x+1}\n")
		files := maps.Clone(c.files)
		files["go.mod"] = []byte("module example.com/policy\ngo 1.25.0\n")
		files["policy.go"] = source
		for name, raw := range files {
			writePracticalTestFile(t, filepath.Join(directory, filepath.FromSlash(name)), raw, 0600)
		}
		for name, target := range c.links {
			if err := os.MkdirAll(filepath.Dir(filepath.Join(directory, name)), 0700); err != nil {
				t.Fatal(err)
			}
			if err := os.Symlink(target, filepath.Join(directory, name)); err != nil {
				t.Fatal(err)
			}
		}
		command := exec.Command(filepath.Join(runtime.GOROOT(), "bin", "go"), "list", "-json", ".")
		command.Dir = directory
		command.Env = []string{"PATH=/usr/bin:/bin", "GOROOT=" + runtime.GOROOT(), "GOTOOLCHAIN=local", "GOENV=off", "GOWORK=off", "GO111MODULE=on", "GOFLAGS=-mod=readonly", "GOPROXY=off", "GOSUMDB=off", "CGO_ENABLED=0", "GODEBUG=", "GOMAXPROCS=1", "GOOS=linux", "GOARCH=amd64", "GOAMD64=v1", "GOEXPERIMENT=", "GOCACHE=" + os.Getenv("GOCACHE"), "GOMODCACHE=" + os.Getenv("GOMODCACHE")}
		var stdout, stderr bytes.Buffer
		command.Stdout = &stdout
		command.Stderr = &stderr
		nativeErr := command.Run()
		var native practicalGoPackageExport
		if nativeErr == nil {
			if err := json.Unmarshal(stdout.Bytes(), &native); err != nil {
				t.Fatal(err)
			}
		}
		session, err := newPracticalCaptureSession([]practicalSourceRoot{{Capability: "original", Directory: directory}})
		if err != nil {
			t.Fatal(err)
		}
		for name, kind := range map[string]string{"policy.go": "go_file", "go.mod": "go_mod"} {
			if _, err := session.readSource("original", name, kind); err != nil {
				t.Fatal(err)
			}
		}
		collected, err := session.captureEmbeddedSources(practicalModuleLocation{capability: "original"}, c.patterns)
		if (err == nil) != (nativeErr == nil) {
			t.Fatalf("%s capture=%v native=%v: %s", c.name, err, nativeErr, stderr.String())
		}
		row := map[string]any{"case": c.name, "patterns": c.patterns, "whole_original_files": files, "links": c.links, "native_stdout": stdout.Bytes(), "native_stderr": stderr.Bytes(), "accepted": err == nil}
		if err != nil {
			row["error"] = err.Error()
			if session.failed == nil {
				t.Fatal("embed capture failure was not sticky")
			}
			rows = append(rows, row)
			continue
		}
		if !reflect.DeepEqual(collected.files, native.EmbedFiles) {
			t.Fatal(c.name, collected.files, native.EmbedFiles)
		}
		sealed, err := session.seal()
		if err != nil {
			t.Fatal(err)
		}
		packet, err := sealed.snapshotInputs()
		if err != nil {
			t.Fatal(err)
		}
		row["snapshot"] = practicalSnapshotInputEvidence(packet, true)
		row["files"] = collected.files
		row["pattern_files"] = collected.patterns
		row["objects"] = collected.objects
		if err := os.RemoveAll(directory); err != nil {
			t.Fatal(err)
		}
		replayed, err := sealed.replayEmbeddedSources(practicalModuleLocation{capability: "original"}, c.patterns)
		if err != nil || !reflect.DeepEqual(collected, replayed) {
			t.Fatal("original embedded replay differs after removing original roots", c.name, err)
		}
		rows = append(rows, row)
	}
	retainPracticalSyntaxEvidence(t, "embedded-source-corpus.json", map[string]any{"cases": rows, "original_resources_unchanged": true, "original_roots_removed_before_successful_replay": true, "caller_source_adapter_or_fallback": false})
}

func TestPracticalEmbeddedSourcesKeepCompleteOriginalOwner(t *testing.T) {
	practicalContractTestOutput(t, "owned")
	owner, _ := practicalStorageTestOwnerWithOverlays(t, map[string][]byte{
		"go.mod":            []byte("module example.com/policy\ngo 1.25.0\n"),
		"policy.go":         []byte("package policy\nimport \"embed\"\n//go:embed all:assets\nvar Unused embed.FS\nfunc Root(x uint16)uint16{return x+1}\n"),
		"assets/binary.dat": {0, 1, 0, 255, 128}, "assets/.hidden": []byte("complete hidden original\n"), "assets/deep/data.txt": []byte("nested original\n"),
		"assets/empty/.ignored": []byte("exact original\n"),
	}, "example.com/policy", nil)
	defer owner.Close()
	input, err := owner.captureInputs([]practicalClosureRef{practicalClosureTestRef(t, owner, "example.com/policy", "Root")})
	if err != nil {
		t.Fatal(err)
	}
	pkg := owner.packages["example.com/policy"]
	if len(pkg.embedded) != 4 {
		t.Fatal("original embedded assets omitted", pkg.embedded)
	}
	selected := []any{}
	for _, obj := range pkg.embedded {
		buffer, err := owner.originalBytes(obj)
		if err != nil {
			t.Fatal(err)
		}
		selected = append(selected, map[string]any{"object": obj, "raw_hex": hex.EncodeToString(buffer.data)})
	}
	inventory, err := owner.inventory()
	if err != nil {
		t.Fatal(err)
	}
	for i := range inventory {
		if inventory[i].importPath == "example.com/policy" {
			inventory[i].embedded[0].Path = "caller-replaced"
		}
	}
	if owner.packages["example.com/policy"].embedded[0].Path == "caller-replaced" {
		t.Fatal("embedded inventory aliases original")
	}
	location := practicalModuleLocation{capability: pkg.sources[0].Capability}
	expected, err := owner.inputs.snapshot.replayEmbeddedSources(location, []string{"all:assets"})
	if err != nil {
		t.Fatal(err)
	}
	for _, kind := range []string{"missing-byte-buffer", "changed-original-bytes", "missing-directory"} {
		candidate := owner.inputs.snapshot
		candidate.buffers = maps.Clone(candidate.buffers)
		candidate.directories = maps.Clone(candidate.directories)
		if kind == "missing-directory" {
			delete(candidate.directories, practicalCaptureKey(location.capability, "assets/deep"))
		} else {
			obj := pkg.embedded[0]
			buffer := candidate.buffers[obj.ID]
			if kind == "missing-byte-buffer" {
				delete(candidate.buffers, obj.ID)
			} else {
				buffer.data = bytes.Clone(buffer.data)
				buffer.data[0] ^= 1
				candidate.buffers[obj.ID] = buffer
			}
		}
		if _, err := candidate.replayEmbeddedSources(location, []string{"all:assets"}); err == nil {
			t.Fatal("damaged original embedded input accepted", kind)
		}
	}
	retainPracticalSyntaxEvidence(t, "complete-embedded-capture.json", practicalCaptureInputEvidence(t, input))
	retainPracticalSyntaxEvidence(t, "original-embedded-resources.json", map[string]any{"selected": selected, "files": expected.files, "patterns": expected.patterns, "all_original_objects": len(input.snapshot.sources), "controls": []string{"missing-byte-buffer", "changed-original-bytes", "missing-directory"}, "returned_inventory_independent": true, "registered_context_or_program_proof": false})
}
