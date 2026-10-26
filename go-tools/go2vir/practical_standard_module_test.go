//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"golang.org/x/mod/modfile"
)

func practicalStrictModuleFacts(file *modfile.File) map[string]any {
	require, exclude, replace, retract, debug, tools, ignore := []any{}, []any{}, []any{}, []any{}, []any{}, []any{}, []any{}
	for _, r := range file.Require {
		require = append(require, map[string]any{"path_hex": hex.EncodeToString([]byte(r.Mod.Path)), "version": r.Mod.Version, "indirect": r.Indirect})
	}
	for _, r := range file.Exclude {
		exclude = append(exclude, map[string]any{"path_hex": hex.EncodeToString([]byte(r.Mod.Path)), "version": r.Mod.Version, "indirect": false})
	}
	for _, r := range file.Replace {
		replace = append(replace, map[string]any{"old_path_hex": hex.EncodeToString([]byte(r.Old.Path)), "old_version": r.Old.Version, "new_path_hex": hex.EncodeToString([]byte(r.New.Path)), "new_version": r.New.Version})
	}
	for _, r := range file.Retract {
		retract = append(retract, map[string]any{"low_hex": hex.EncodeToString([]byte(r.Low)), "high_hex": hex.EncodeToString([]byte(r.High))})
	}
	for _, r := range file.Godebug {
		debug = append(debug, map[string]any{"key_hex": hex.EncodeToString([]byte(r.Key)), "value_hex": hex.EncodeToString([]byte(r.Value))})
	}
	for _, r := range file.Tool {
		tools = append(tools, hex.EncodeToString([]byte(r.Path)))
	}
	for _, r := range file.Ignore {
		ignore = append(ignore, hex.EncodeToString([]byte(r.Path)))
	}
	var module, goVersion, toolchain any
	if file.Module != nil {
		module = hex.EncodeToString([]byte(file.Module.Mod.Path))
	}
	if file.Go != nil {
		goVersion = file.Go.Version
	}
	if file.Toolchain != nil {
		toolchain = hex.EncodeToString([]byte(file.Toolchain.Name))
	}
	return map[string]any{"module_path_hex": module, "go_requirement": goVersion, "toolchain_suggestion_hex": toolchain, "requirements": require, "exclusions": exclude, "replacements": replace, "retractions": retract, "debug_settings": debug, "tools_hex": tools, "ignored_paths_hex": ignore}
}
func TestPracticalStandardModuleStrictOriginalOracle(t *testing.T) {
	dir := os.Getenv("MPK_GO_STANDARD_MODULE_OUTPUT")
	if dir == "" {
		t.Skip("original strict SDK manifest observations not requested")
	}
	if err := os.MkdirAll(dir, 0o700); err != nil {
		t.Fatal(err)
	}
	rows := []any{}
	observe := func(name string, raw []byte) {
		source, err := newPracticalSourceBuffer("strict-original-module", "src/go.mod", "go_mod", raw)
		if err != nil {
			t.Fatal(err)
		}
		parsed, parseErr := modfile.Parse(name, raw, nil)
		_, stdErr := parsePracticalStandardModule(name, raw)
		var facts any
		if parseErr == nil {
			facts = practicalStrictModuleFacts(parsed)
		}
		rows = append(rows, map[string]any{"name": name, "raw_hex": hex.EncodeToString(raw), "source_object": source.object, "parse_accepted": parseErr == nil, "parse_error": fmt.Sprint(parseErr), "standard_accepted": stdErr == nil, "standard_error": fmt.Sprint(stdErr), "facts": facts})
	}
	path := filepath.Join("..", "..", "develop", "migrations", "go-practical", "t02-w01", "verification", "module-manifests", "native-toolchain", "native-originals", "module-manifests", "complete-original-module-manifests.json")
	data, err := os.ReadFile(path)
	if err != nil {
		t.Fatal(err)
	}
	var previous struct {
		Cases []struct {
			RawHex string `json:"raw_hex"`
		}
	}
	if err := json.Unmarshal(data, &previous); err != nil {
		t.Fatal(err)
	}
	for i, row := range previous.Cases {
		raw, err := hex.DecodeString(row.RawHex)
		if err != nil {
			t.Fatal(err)
		}
		observe(fmt.Sprintf("complete-prior-%04d", i), raw)
		observe(fmt.Sprintf("complete-prior-std-%04d", i), bytes.Replace(raw, []byte("example.com/policy"), []byte("std"), 1))
	}
	base := "module std\ngo 1.25\n"
	for _, value := range []string{"", "0.1", "1", "1.0", "1.16", "1.22", "1.25", "1.25.0", "1.25rc1", "1.25rc00", "1.25.0beta1", "1.25.0beta00", "1.25future2", "1.25garbage", "v1.25", "v1.25garbage", "1.25.1", "1.26", "2.0", strings.Repeat("9", 300) + ".0", "01.25", "1.025", "\"1.25\""} {
		observe("go-"+value, []byte("module std\ngo "+value+"\n"))
	}
	for _, directive := range []string{"godebug", "tool", "ignore", "toolchain"} {
		for _, value := range []string{"", "panicnil=1", "=", "key=", "=value", "a=b=c", "bad", "a,b=c", "\"a=b\"", "`raw`", "\"\\xff\"", "default", "go1", "go1.", "go1.future", "go1.99.0", "go1." + string([]byte{255}), "\"original path\""} {
			observe(directive+"-"+hex.EncodeToString([]byte(value)), []byte(base+directive+" "+value+"\n"))
		}
	}
	observe("all-complete-families", []byte(base+"toolchain go1.99.0\ngodebug (\n=\nkey=value\nkey=other\n)\ntool (\nexample.com/tool\n\"quoted tool\"\n)\nignore (\nassets\n\"\\xff\"\n)\nrequire example.com/a v1 // indirect\nexclude example.com/b v1.2+meta\nreplace example.com/a => ../original\nretract [v2, original]\n"))
	observe("long-complete-directives", []byte(base+strings.Repeat("godebug key=value\n", 40000)+"// original suffix\n"))
	err = filepath.WalkDir(filepath.Join(runtime.GOROOT(), "src"), func(name string, entry fs.DirEntry, err error) error {
		if err != nil {
			return err
		}
		if entry.IsDir() || entry.Name() != "go.mod" {
			return nil
		}
		raw, err := os.ReadFile(name)
		if err != nil {
			return err
		}
		relative, err := filepath.Rel(runtime.GOROOT(), name)
		if err != nil {
			return err
		}
		observe("sdk-"+filepath.ToSlash(relative), raw)
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}
	encoded, err := json.MarshalIndent(map[string]any{"schema": "mpk.private.strict_standard_module_observations.v1", "go_version": runtime.Version(), "prior_original_cases": len(previous.Cases), "cases": rows}, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(dir, "complete-original-strict-modules.json"), append(encoded, '\n'), 0o600); err != nil {
		t.Fatal(err)
	}
}
