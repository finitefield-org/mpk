//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"os/exec"
	"path/filepath"
	"reflect"
	"sort"
	"strings"
	"testing"
	"time"
)

func TestPracticalGo125MetaPackageNamesAndLiteralDots(t *testing.T) {
	compiler := practicalTestRegisteredCompiler(t)
	filesystem, err := newLoaderFilesystem()
	if err != nil {
		t.Fatal(err)
	}
	defer filesystem.Close()
	root := t.TempDir()
	run := func(directory string, arguments ...string) ([]byte, error) {
		ctx, cancel := context.WithTimeout(context.Background(), 120*time.Second)
		defer cancel()
		command := exec.CommandContext(ctx, compiler.selection.GoExecutable, arguments...)
		command.Dir, command.Env = directory, pinnedLoaderEnvironment(compiler.selection, filesystem)
		output, err := command.CombinedOutput()
		if ctx.Err() != nil {
			t.Fatal("original Go package-pattern observation timed out", ctx.Err())
		}
		return output, err
	}
	for _, name := range []string{"all", "std", "cmd", "tool", "work"} {
		directory := filepath.Join(root, name)
		writePracticalTestFile(t, filepath.Join(directory, "go.mod"), []byte("module "+name+"\ngo 1.25.0\n"), 0o600)
		writePracticalTestFile(t, filepath.Join(directory, "policy.go"), []byte("package policy\nfunc Root() uint16 { return 7 }\n"), 0o600)
		output, err := run(directory, "list", "-mod=readonly", "-json", ".")
		if err == nil || !strings.Contains(string(output), "is not an importable package") {
			t.Fatalf("Go treated meta-package %q as an importable package: %v %s", name, err, output)
		}
	}
	directory := filepath.Join(root, "dots")
	writePracticalTestFile(t, filepath.Join(directory, "go.mod"), []byte("module example.com/demo\ngo 1.25.0\n"), 0o600)
	for _, name := range []string{"p...q", "p123q"} {
		writePracticalTestFile(t, filepath.Join(directory, name, "policy.go"), []byte("package policy\nfunc Root() uint16 { return 7 }\n"), 0o600)
	}
	decode := func(output []byte) []string {
		decoder := json.NewDecoder(bytes.NewReader(output))
		var names []string
		for {
			var row struct{ ImportPath string }
			if err := decoder.Decode(&row); err == io.EOF {
				break
			} else if err != nil {
				t.Fatal("original Go package output is invalid", err)
			}
			names = append(names, row.ImportPath)
		}
		sort.Strings(names)
		return names
	}
	literal, err := run(filepath.Join(directory, "p...q"), "list", "-mod=readonly", "-json", ".")
	if err != nil || !reflect.DeepEqual(decode(literal), []string{"example.com/demo/p...q"}) {
		t.Fatal("Go did not load the actual dot-containing package from its directory", err, string(literal))
	}
	pattern, err := run(directory, "list", "-mod=readonly", "-json", "example.com/demo/p...q")
	if err != nil || !reflect.DeepEqual(decode(pattern), []string{"example.com/demo/p...q", "example.com/demo/p123q"}) {
		t.Fatal("Go did not expand the ambiguous package pattern", err, string(pattern))
	}
}
