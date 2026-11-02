//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/version"
	"os"
	"path/filepath"
	"sort"
	"testing"

	"golang.org/x/mod/modfile"
)

func practicalGraphManifestFacts(file *modfile.File) map[string]any {
	var module, goVersion, implicitGo any
	if file.Module != nil {
		module = hex.EncodeToString([]byte(file.Module.Mod.Path))
	}
	pruned, newer := false, false
	if file.Go != nil {
		goVersion = file.Go.Version
		pruned = version.Compare("go"+file.Go.Version, "go1.17") >= 0
		newer = version.Compare("go"+file.Go.Version, practicalCompilerRelease) > 0
		if version.Compare("go"+file.Go.Version, "go1.21") >= 0 {
			implicitGo = file.Go.Version
		}
	}
	requirements, retractions, ignores := []any{}, []any{}, []any{}
	for _, r := range file.Require {
		requirements = append(requirements, map[string]any{"path_hex": hex.EncodeToString([]byte(r.Mod.Path)), "version": r.Mod.Version, "indirect": r.Indirect})
	}
	for _, r := range file.Retract {
		retractions = append(retractions, map[string]any{"low_hex": hex.EncodeToString([]byte(r.Low)), "high_hex": hex.EncodeToString([]byte(r.High))})
	}
	for _, r := range file.Ignore {
		ignores = append(ignores, hex.EncodeToString([]byte(r.Path)))
	}
	return map[string]any{"module_path_hex": module, "go_requirement": goVersion, "requirements": requirements, "retractions": retractions, "ignored_paths_hex": ignores, "pruned": pruned, "implicit_go_requirement": implicitGo, "requires_newer_compiler": newer}
}

func TestPracticalGraphManifestOriginalLaxOracle(t *testing.T) {
	practicalContractTestOutput(t, "graph-manifests")
	rows := []any{}
	observe := func(name string, origin any, source practicalSourceBuffer) {
		before := source.sourceBytes()
		file, err := modfile.ParseLax(source.object.Path, source.sourceBytes(), nil)
		var facts any
		if err == nil {
			if file.Toolchain != nil || len(file.Exclude) != 0 || len(file.Replace) != 0 || len(file.Godebug) != 0 || len(file.Tool) != 0 {
				t.Fatal("native dependency parser retained a main-only directive")
			}
			facts = practicalGraphManifestFacts(file)
		}
		if !bytes.Equal(source.sourceBytes(), before) {
			t.Fatal("lax parser rewrote original bytes")
		}
		rows = append(rows, map[string]any{"case": name, "origin_main": origin, "object": source.object, "raw_hex": hex.EncodeToString(before), "accepted": err == nil, "error": fmt.Sprint(err), "facts": facts})
	}
	// Preserve all old original source objects and both original mode labels.
	previous := filepath.Join("..", "..", "develop", "migrations", "go-practical", "t02-w01", "verification", "module-manifests", "native-toolchain", "native-originals", "module-manifests", "complete-original-module-manifests.json")
	raw, err := os.ReadFile(previous)
	if err != nil {
		t.Fatal(err)
	}
	var old struct {
		Cases []struct {
			Case   string                `json:"case"`
			Main   bool                  `json:"main"`
			Object practicalSourceObject `json:"object"`
			RawHex string                `json:"raw_hex"`
		} `json:"cases"`
	}
	if err := json.Unmarshal(raw, &old); err != nil {
		t.Fatal(err)
	}
	if len(old.Cases) != 656 {
		t.Fatal("original strict corpus was narrowed")
	}
	for _, row := range old.Cases {
		data, err := hex.DecodeString(row.RawHex)
		if err != nil {
			t.Fatal(err)
		}
		source, err := newPracticalSourceBuffer(row.Object.Capability, row.Object.Path, row.Object.Kind, data)
		if err != nil || source.object != row.Object {
			t.Fatal("original source identity changed", err)
		}
		observe(row.Case, row.Main, source)
	}
	cases := map[string][]byte{}
	for _, head := range []string{"future", "require prefix", "go", "toolchain", "replace", "exclude", "godebug", "tool"} {
		for _, body := range []string{"", "\n", "\n  malformed \"bad\\q\" [ x , y ] extra\n", "\n (\n original\n)\n"} {
			cases["ignored-block-"+head+"-"+hex.EncodeToString([]byte(body))] = []byte(head + " (" + body + ")\n")
		}
		cases["ignored-line-"+head] = []byte(head + " \"bad\\q\" extra [ token ]\n")
	}
	for _, v := range []string{"1.16", "1.17", "1.20", "1.21", "1.25", "1.25.0", "1.25.1", "1.25.0rc1", "1.25rc01", "1.25foo0", "1.25beta", "1.25.00", "1.25.0-original", "v1.25", "v1.25.0", "v1.16-original", "1.025", "1.025.0", "01.25-original", "1.0rc0", "1.99", "999999999999999999999.0", "1.999999999999999999999999999.0", "0.25", "1.25" + string([]byte{255}), "1.25-" + string([]byte{0xe2, 0x82})} {
		cases["lax-go-"+hex.EncodeToString([]byte(v))] = []byte("go " + v + "\n")
	}
	for _, v := range []string{"", "v1", "[v1,v2]", "[v2,v1] additional fields", "v1 additional fields", "[", "[v1", "[v1,v2", "(v1,v2)", "[v1,v2)", "\"bad\\q\"", "[\"bad\\q\",v2]", "[v1,\"bad\\q\"]", "[v1,v2] \"bad\\q\""} {
		cases["lax-retract-"+hex.EncodeToString([]byte(v))] = []byte("retract " + v + "\n")
	}
	for _, path := range []string{"\"\"", "\"./\\xff\"", "\"./" + string([]byte{255}) + "\"", "`raw`", "\"bad\\q\"", "one two", "[", "CON", "example.com/\u03c3"} {
		for _, verb := range []string{"module", "ignore", "tool", "future"} {
			cases[verb+"-"+hex.EncodeToString([]byte(path))] = []byte(verb + " " + path + "\n")
		}
	}
	cases["ignored-unclosed-quote"] = []byte("future \"never closes\n")
	cases["ignored-unclosed-block"] = []byte("future (\nvalue\n")
	cases["ignored-block-closing-suffix"] = []byte("future (\nvalue\n) extra\n")
	cases["ignored-block-comment"] = []byte("future /* bad */\n")
	cases["lax-module-no-go"] = []byte("module example.com/old\nfuture malformed [ syntax ]\n")
	cases["lax-block-require"] = []byte("require (\nexample.com/a v1\nexample.com/a v1.2 // indirect\n)\n")
	cases["lax-block-ignore"] = []byte("ignore (\n\"./assets\"\n\"./\\xff\"\n)\n")
	names := []string{}
	for name := range cases {
		names = append(names, name)
	}
	sort.Strings(names)
	for _, name := range names {
		source, err := newPracticalSourceBuffer("module-graph-manifest", "go.mod", "go_mod", cases[name])
		if err != nil {
			t.Fatal(err)
		}
		observe(name, nil, source)
	}
	retainPracticalSyntaxEvidence(t, "complete-original-graph-manifests.json", map[string]any{"cases": rows, "previous_original_mode_cases": 656, "complete_original_inputs_retained": true, "MVS_or_profile_activated": false, "parser": "pinned original modfile.ParseLax; original mode labels retain source-corpus provenance only"})
}
