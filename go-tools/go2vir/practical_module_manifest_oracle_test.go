//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"sort"
	"testing"
	"unicode"
)

func practicalModuleManifestFacts(parsed practicalModuleManifest) map[string]any {
	file := parsed.file
	requirements, exclusions, replacements, retractions := []any{}, []any{}, []any{}, []any{}
	for _, row := range file.Require {
		requirements = append(requirements, map[string]any{"path_hex": hex.EncodeToString([]byte(row.Mod.Path)), "version": row.Mod.Version, "indirect": row.Indirect})
	}
	for _, row := range file.Exclude {
		exclusions = append(exclusions, map[string]any{"path_hex": hex.EncodeToString([]byte(row.Mod.Path)), "version": row.Mod.Version, "indirect": false})
	}
	for _, row := range file.Replace {
		replacements = append(replacements, map[string]any{"old_path_hex": hex.EncodeToString([]byte(row.Old.Path)), "old_version": row.Old.Version, "new_path_hex": hex.EncodeToString([]byte(row.New.Path)), "new_version": row.New.Version})
	}
	for _, row := range file.Retract {
		retractions = append(retractions, map[string]any{"low_hex": hex.EncodeToString([]byte(row.Low)), "high_hex": hex.EncodeToString([]byte(row.High))})
	}
	var goVersion, language, toolchain any
	if file.Go != nil {
		goVersion, language = file.Go.Version, parsed.language
	}
	if file.Toolchain != nil {
		toolchain = hex.EncodeToString([]byte(file.Toolchain.Name))
	}
	return map[string]any{"module_path_hex": hex.EncodeToString([]byte(file.Module.Mod.Path)), "go_requirement": goVersion, "language_version": language, "toolchain_suggestion_hex": toolchain, "requirements": requirements, "exclusions": exclusions, "replacements": replacements, "retractions": retractions}
}

func TestPracticalModuleManifestOriginalOracle(t *testing.T) {
	practicalContractTestOutput(t, "module-manifests")
	rows := []any{}
	observe := func(name string, raw []byte) {
		for _, main := range []bool{false, true} {
			source, err := newPracticalSourceBuffer("module-manifest", "go.mod", "go_mod", raw)
			if err != nil {
				t.Fatal(err)
			}
			parsed, err := parsePracticalModuleManifest(source, main)
			var facts any
			if err == nil {
				facts = practicalModuleManifestFacts(parsed)
				if !bytes.Equal(parsed.source.data, raw) {
					t.Fatal("original manifest rewritten")
				}
			}
			rows = append(rows, map[string]any{"case": name, "main": main, "object": source.object, "raw_hex": hex.EncodeToString(raw), "accepted": err == nil, "error": fmt.Sprint(err), "facts": facts})
		}
	}
	base := "module example.com/policy\ngo 1.25.0\n"
	observe("complete-directives", []byte("// original comment\nmodule example.com/policy\ngo 1.25.0\ntoolchain go1.25.0\nrequire (\nexample.com/A v1 // indirect\nexample.com/A v1.2 // indirect; original reason\n)\nexclude example.com/B v1.2.3+build\nreplace (\nexample.com/A => \"../local original\"\nexample.com/A v1 => example.com/Fork v2.0.0\n)\nretract (\n v1\n [v2.0.0, v1.0.0]\n original-non-version\n)\n"))
	cases := map[string]string{
		"toolchain-invalid-utf8-suffix":    base + "toolchain go1.25.0-original-" + string([]byte{255}) + "\n",
		"toolchain-incomplete-utf8-suffix": base + "toolchain go1.25.0-original-" + string([]byte{0xe2, 0x82}) + "\n",
		"no-go":                            "module example.com/policy\n", "no-module": "go 1.25.0\n", "empty": "",
		"quoted-module": "module \"example.com/policy\"\ngo 1.25\n", "empty-blocks": base + "require ()\nreplace ( )\nexclude (\n)\nretract ()\nmodule ()\ngodebug ()\ntool ()\nignore ()\n",
		"module-block": "module (\nexample.com/policy\n)\ngo 1.25.0\n", "empty-module": "module \"\"\ngo 1.25\n",
		"duplicate-module": base + "module example.com/policy\n", "duplicate-go": base + "go 1.25\n", "duplicate-toolchain": base + "toolchain default\ntoolchain default\n",
		"go-block": "module example.com/policy\ngo (\n1.25\n)\n", "toolchain-block": base + "toolchain (\ndefault\n)\n",
		"unknown-line": base + "future directive\n", "unknown-block": base + "future ()\n", "unknown-prefix-block": base + "require prefix (\n)\n",
		"forbidden-godebug": base + "godebug panicnil=1\n", "forbidden-tool": base + "tool example.com/tool\n", "forbidden-ignore": base + "ignore assets\n",
		"unclosed-block": base + "require (\nexample.com/a v1.0.0\n", "same-line-block": base + "require (example.com/a v1.0.0)\n", "closing-suffix": base + "require (\nexample.com/a v1.0.0\n) extra\n",
		"nested-block": base + "require (\n (\nexample.com/a v1.0.0\n)\n)\n", "block-comment": base + "/* comment */\n",
		"raw-quoted-module": "module `example.com/policy`\ngo 1.25\n", "invalid-string": base + "require \"bad\\q\" v1.0.0\n",
		"literal-quoted-go": "module example.com/policy\ngo \"1.25\"\n", "quoted-require": base + "require \"example.com/A\" \"v1.2+meta\"\n",
		"quoted-path-escape":          base + "replace example.com/a => \"\\u002e/local\"\n",
		"duplicate-require":           base + "require example.com/a v1.0.0\nrequire example.com/a v1.1.0\n",
		"duplicate-replacement":       base + "replace example.com/a => ./a\nreplace example.com/a => ./a\n",
		"dependency-future-toolchain": "module example.com/a\ngo 1.22\ntoolchain go1.99.0\n",
		"retract-after-module":        "retract [v1,v2]\nmodule example.com/policy\ngo 1.25\n",
		"retract-extra":               base + "retract v1.0.0 extra\n", "retract-no-close": base + "retract [v1.0.0,v2.0.0\n",
		"invalid-utf8-comment":        base + "// " + string([]byte{255}) + " original\n",
		"invalid-utf8-require":        base + "require example.com/" + string([]byte{255}) + " v1.0.0\n",
		"quoted-invalid-utf8-require": base + "require \"example.com/" + string([]byte{0xe2, 0x82}) + "\" v1.0.0\n",
		"quoted-invalid-utf8-replace": base + "replace example.com/a => \"./" + string([]byte{255}) + "\"\n",
		"retract-open-bracket":        base + "retract [\n",
		"escaped-path-bytes":          base + "replace example.com/a => \"./\\xff\"\n",
	}
	names := []string{}
	for name := range cases {
		names = append(names, name)
	}
	sort.Strings(names)
	for _, name := range names {
		observe(name, []byte(cases[name]))
	}
	for _, path := range []string{"example.com/policy", "local", "a..b", ".hidden", "foo+bar", "main", "all", "std", "cmd", "-bad", "a//b", "a/", "a.", "CON", "foo~1", "example.com/v1", "example.com/σ", "example.com/!bad"} {
		observe("module-path-"+path, []byte("module "+path+"\ngo 1.25\n"))
	}
	for _, goVersion := range []string{"1.0", "1.16", "1.21", "1.22", "1.23", "1.24", "1.25", "1.25.0", "1.25.1", "1.26", "1.25rc1", "1.25alpha0", "1.25foo1", "1.25.0rc1", "1.25rc01", "0.25", "01.25", "1.025", "1.25.00", "999999999999999999999.0", "1.999999999999999999999999999999999"} {
		observe("go-"+goVersion, []byte("module example.com/policy\ngo "+goVersion+"\n"))
	}
	for _, toolchain := range []string{"default", "go1", "go1.25", "go1.25.0", "go1.25.0-original", "go1.26.0", "go2.0.0", "go1.foo", "go1.25beta", "go1.25rc01", "\"default\""} {
		observe("toolchain-"+toolchain, []byte(base+"toolchain "+toolchain+"\n"))
	}
	for _, path := range []string{"example.com/a", "example.com/a/v2", "example.com/a/v0", "example.com/a/v1", "example.com/a/v01", "example.com/a/v2.1", "gopkg.in/a.v0", "gopkg.in/a.v0-unstable", "gopkg.in/a.v1", "gopkg.in/a.v2", "gopkg.in/a.v2-unstable", "gopkg.in/a"} {
		for _, version := range []string{"v0", "v1", "v1.2", "v1.2.3", "v2.0.0", "v2.0.0+incompatible", "v1.2.3+meta", "v1.2.3-pre.01", "v1.2.3-pre.0", "v01.2.3", "v1.2.03", "v1.2+meta", "v1.2.3+", "v0.0.0-20161208181325-20d25e280405", "v99999999999999999999.0.0+incompatible"} {
			observe("require-"+path+"@"+version, []byte(base+"require "+path+" "+version+"\n"))
		}
	}
	for _, target := range []string{"./a", "../a", ".", "..", "/absolute", "a", "a@v1.0.0", ".\\a", "C:/a", "C:\\a", "\\a"} {
		observe("local-replace-"+target, []byte(base+"replace example.com/a => "+target+"\n"))
		observe("versioned-replace-"+target, []byte(base+"replace example.com/a => "+target+" v1\n"))
	}
	for _, comment := range []string{"indirect", "indirect original", "indirect; original", " indirect;", "INDIRECT", "indirect; original more", "indirect\u0085", "indirect;\u0085original"} {
		observe("indirect-"+comment, []byte(base+"require example.com/a v1.0.0 // "+comment+"\n"))
	}
	for r := rune(0); r <= unicode.MaxRune; r++ {
		if unicode.IsSpace(r) {
			observe(fmt.Sprintf("space-%x", r), []byte(base+"require example.com/a"+string(r)+"v1.0.0\n"))
		}
	}
	retainPracticalSyntaxEvidence(t, "complete-original-module-manifests.json", map[string]any{"cases": rows, "complete_original_inputs_retained": true, "dependency_versions_selected_or_profile_activated": false, "original_parser": "strict original modfile.Parse with the unchanged practical module dispositions"})
}
