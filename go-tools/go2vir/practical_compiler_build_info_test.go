//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"debug/buildinfo"
	"encoding/hex"
	"encoding/json"
	"os"
	"path/filepath"
	"runtime"
	"runtime/debug"
	"testing"
)

func practicalBuildInfoFacts(info *debug.BuildInfo) map[string]any {
	var mod func(*debug.Module) any
	mod = func(m *debug.Module) any {
		if m == nil {
			return nil
		}
		return map[string]any{"path_hex": hex.EncodeToString([]byte(m.Path)), "version_hex": hex.EncodeToString([]byte(m.Version)), "sum_hex": hex.EncodeToString([]byte(m.Sum)), "replacement": mod(m.Replace)}
	}
	deps := []any{}
	for _, m := range info.Deps {
		deps = append(deps, mod(m))
	}
	settings := []any{}
	for _, s := range info.Settings {
		settings = append(settings, []string{hex.EncodeToString([]byte(s.Key)), hex.EncodeToString([]byte(s.Value))})
	}
	return map[string]any{"version_hex": hex.EncodeToString([]byte(info.GoVersion)), "path_hex": hex.EncodeToString([]byte(info.Path)), "main": mod(&info.Main), "dependencies": deps, "settings": settings}
}
func TestPracticalCompilerBuildInfoOriginalBytesAndGrammar(t *testing.T) {
	dir := os.Getenv("MPK_GO_BUILD_INFO_FIXTURES")
	if dir == "" {
		t.Skip("original build-info comparison fixtures not requested")
	}
	raw, err := os.ReadFile(filepath.Join(dir, "inputs.json"))
	if err != nil {
		t.Fatal(err)
	}
	var rows []map[string]any
	if err := json.Unmarshal(raw, &rows); err != nil {
		t.Fatal(err)
	}
	for _, tool := range []string{"go", "gofmt", "asm", "cgo", "compile", "cover", "link", "preprofile", "vet"} {
		path := filepath.Join(runtime.GOROOT(), "pkg", "tool", runtime.GOOS+"_"+runtime.GOARCH, tool)
		if tool == "go" || tool == "gofmt" {
			path = filepath.Join(runtime.GOROOT(), "bin", tool)
		}
		rows = append(rows, map[string]any{"name": "original-" + tool, "file": path, "form": "original", "tool": tool, "host": runtime.GOOS + "_" + runtime.GOARCH})
	}
	for _, row := range rows {
		t.Run(row["name"].(string), func(t *testing.T) {
			path := row["file"].(string)
			if !filepath.IsAbs(path) {
				path = filepath.Join(dir, path)
			}
			raw, err := os.ReadFile(path)
			if err != nil {
				t.Fatal(err)
			}
			var info *debug.BuildInfo
			if row["form"] == "text" {
				info, err = debug.ParseBuildInfo(string(raw))
			} else {
				info, err = buildinfo.Read(bytes.NewReader(raw))
			}
			row["raw_sha256"] = sha256Hex(raw)
			row["bytes"] = len(raw)
			row["accepted"] = err == nil
			if err != nil {
				row["error"] = err.Error()
				return
			}
			row["facts"] = practicalBuildInfoFacts(info)
			tool, host := "compile", "linux_amd64"
			if row["form"] == "original" {
				tool = row["tool"].(string)
				host = row["host"].(string)
				if err := validatePracticalCompilerBuildInfo(info, tool, host); err != nil {
					t.Fatal(err)
				}
			}
			if row["form"] == "text" {
				info.GoVersion = "go1.25.0"
			}
			row["binding_accepted"] = validatePracticalCompilerBuildInfo(info, tool, host) == nil
		})
	}
	output, err := json.MarshalIndent(rows, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "native.json"), append(output, '\n'), 0o600); err != nil {
		t.Fatal(err)
	}
}
