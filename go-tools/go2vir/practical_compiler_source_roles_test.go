//go:build go1.25 && (linux || darwin)

package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"testing"
)

func TestPracticalCompilerSourceRolesWholeOriginalSDKAndHeaders(t *testing.T) {
	dir := os.Getenv("MPK_GO_COMPILER_SOURCE_ROLES_OUTPUT")
	if dir == "" {
		t.Skip("original SDK source-role evidence not requested")
	}
	if err := os.MkdirAll(dir, 0o700); err != nil {
		t.Fatal(err)
	}
	observe := func(name string, raw []byte) map[string]any {
		role, err := practicalCompilerSourceRole(name, raw)
		row := map[string]any{"path": name, "bytes": len(raw), "raw_sha256": sha256Hex(raw), "role": role}
		if err != nil {
			row["error"] = err.Error()
		} else {
			row["error"] = nil
		}
		return row
	}
	original := []any{}
	packages := map[string]bool{}
	err := filepath.WalkDir(filepath.Join(runtime.GOROOT(), "src"), func(name string, entry fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if entry.IsDir() {
			return nil
		}
		if !entry.Type().IsRegular() {
			return fmt.Errorf("nonregular original SDK source %s", name)
		}
		raw, err := os.ReadFile(name)
		if err != nil {
			return err
		}
		relative, err := filepath.Rel(runtime.GOROOT(), name)
		if err != nil {
			return err
		}
		relative = filepath.ToSlash(relative)
		row := observe(relative, raw)
		if row["error"] != nil {
			return fmt.Errorf("original SDK source role: %v", row)
		}
		original = append(original, row)
		if strings.HasSuffix(relative, ".go") {
			p := strings.TrimPrefix(filepath.ToSlash(filepath.Dir(name[len(runtime.GOROOT())+1:])), "src/")
			if p != "src" && p != "." {
				packages[p] = true
			}
		}
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}
	packagePaths := []string{}
	for p := range packages {
		packagePaths = append(packagePaths, p)
	}
	sort.Strings(packagePaths)
	probes := []any{}
	bodies := [][]byte{
		[]byte("package documentation;;import \"\\q\""),
		[]byte("package p;import \"C\";;import \"\\q\""),
		[]byte("package documentation;;;;\n\xff"),
		[]byte("package documentation)"),
		[]byte("package documentation;import(\"C\")}"),
		[]byte("package p;import (\"C\"; \"bad path\");func ignored(){ @ "),

		[]byte("package p\nfunc Root(){}\n"), []byte("package documentation\nfunc Root(){}\n"), []byte("package p\nimport \"C\"\nfunc Root(){}\n"),
		[]byte("package documentation\nimport \"bad path\"\nfunc Root(){}\n"), []byte("package p\nimport (\"C\"; \"bad path\")\nfunc Root(){}\n"),
		[]byte("package p\nimport \"\\x43\"\nfunc Root(){}\n"), []byte("package p\nimport `C`\nfunc Root(){}\n"),
		[]byte("package p\nimport \"C\"\ninvalid body @\n"), []byte("package documentation\ninvalid body @\n"),
		[]byte("package p\nimport \"C\"\n@\n"), []byte("package documentation\n@\n"), []byte("package documentation\nfunc @\xff\x00"),
		[]byte("package documentation\nfunc\x00"), []byte("package documentation\nfunc\xff"), []byte("package documentation\nfunc\ufeff"),
		[]byte("package documentation\n0789"), []byte("package documentation\n0x1"), []byte("package documentation\n\"unfinished"),
		[]byte("package documentation\n\x00"), []byte("/*\x00*/package p\n"), []byte("package p\nimport \"\x00\"\n"),
		[]byte("package p\nimport \"C\"\n//line :bad\nfunc Root(){}"), []byte("package documentation\nimporthere @"),
		[]byte("//go:binary-only-package\n\npackage p\n"), []byte("//go:build windows\n\npackage documentation\n"),
		[]byte("//go:build windows\n\n/*\x00*/package p\n"), []byte("//go:build linux\n//go:build linux\n\npackage p\n"),
		[]byte("package _\n"), []byte("package documentation"), []byte("\xef\xbb\xbfpackage documentation\n"),
		[]byte("package p\nimport (\"C\"; \"\\q\")\n"), []byte("package p\nimport (\"bad path\"; \"C\")\n"),
	}
	names := []string{"src/probes/role.go", "src/probes/role_linux.go", "src/probes/role_windows.go", "src/probes/role_amd64.go", "src/probes/role_arm64.go", "src/probes/role_test.go", "src/probes/role_linux_test.go", "src/probes/_role.go", "src/probes/.role.go", "src/probes/role.part_linux.go"}
	for i, body := range bodies {
		for _, name := range names {
			row := observe(name, body)
			row["name"] = fmt.Sprintf("header-%02d-%s", i, filepath.Base(name))
			row["raw_hex"] = hex.EncodeToString(body)
			probes = append(probes, row)
		}
	}
	for _, name := range []string{"src/go.mod", "src/nested/go.mod", "src/nested/go.sum", "src/vendor/modules.txt", "src/nested/vendor/modules.txt", "src/vendor/other.txt", "src/runtime/asm_amd64.s"} {
		row := observe(name, []byte("original arbitrary auxiliary bytes\xff\x00"))
		row["name"] = name
		row["raw_hex"] = hex.EncodeToString([]byte("original arbitrary auxiliary bytes\xff\x00"))
		probes = append(probes, row)
	}
	// No AST-node or source-universe reduction is introduced for a role-only
	// import header. Preserve the complete long original body as well.
	long := []byte("package p\nimport (\n" + strings.Repeat("_ \"example.com/unused\"\n", 40000) + "_ \"C\"\n)\nfunc OriginalBody(){ go func(){select{}}() }\n")
	row := observe("src/probes/long.go", long)
	row["name"] = "long-complete-header-and-body"
	row["raw_hex"] = hex.EncodeToString(long)
	probes = append(probes, row)
	value := map[string]any{"schema": "mpk.private.original_sdk_source_role_observations.v1", "sdk_sources": original, "package_paths": packagePaths, "probes": probes, "go_version": runtime.Version(), "target": "linux_amd64", "cgo_enabled": false}
	data, err := json.MarshalIndent(value, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "original-sdk-source-roles.json"), append(data, '\n'), 0o600); err != nil {
		t.Fatal(err)
	}
}
