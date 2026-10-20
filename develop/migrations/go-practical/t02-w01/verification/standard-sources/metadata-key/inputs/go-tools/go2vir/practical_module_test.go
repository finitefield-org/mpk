package main

import (
	"bytes"
	"strings"
	"testing"

	"golang.org/x/mod/module"
)

func practicalModuleTestBuffer(t *testing.T, kind, raw string) practicalSourceBuffer {
	t.Helper()
	name := "go.mod"
	if kind == "go_sum" {
		name = "go.sum"
	}
	buffer, err := newPracticalSourceBuffer("module", name, kind, []byte(raw))
	if err != nil {
		t.Fatal(err)
	}
	return buffer
}

func TestPracticalModuleManifestPreservesOriginalDirectives(t *testing.T) {
	for _, goVersion := range []string{"1.22", "1.23", "1.24", "1.25.0"} {
		t.Run(goVersion, func(t *testing.T) {
			raw := "// original bytes, comments and order\nmodule example.com/app\n\ngo " + goVersion + "\ntoolchain go1.25.0\nrequire (\n example.com/dep v1.0.0\n example.com/other v1.1.0 // indirect\n)\nexclude example.com/dep v1.0.1\nreplace example.com/dep => ../dep\nreplace example.com/other v1.1.0 => example.com/fork v1.2.0\nretract [v1.0.0, v1.0.1] // original rationale\n"
			buffer := practicalModuleTestBuffer(t, "go_mod", raw)
			manifest, err := parsePracticalModuleManifest(buffer, true)
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(manifest.source.sourceBytes(), []byte(raw)) || manifest.file.Module.Mod.Path != "example.com/app" ||
				manifest.file.Go.Version != goVersion || manifest.file.Toolchain.Name != practicalCompilerRelease ||
				len(manifest.file.Require) != 2 || len(manifest.file.Exclude) != 1 || len(manifest.file.Replace) != 2 || len(manifest.file.Retract) != 1 ||
				manifest.file.Replace[0].New.Path != "../dep" || manifest.file.Replace[1].New.Version != "v1.2.0" {
				t.Fatal("original module metadata or bytes changed")
			}
		})
	}
	for _, suggestion := range []string{"default", "go1.24.7", "go1.25.0"} {
		if _, err := parsePracticalModuleManifest(practicalModuleTestBuffer(t, "go_mod", "module fixture/policy\ngo 1.25.0\ntoolchain "+suggestion+"\n"), true); err != nil {
			t.Fatalf("satisfiable main suggestion %q: %v", suggestion, err)
		}
	}
	// Type-check-only dependencies can predate the proof language table, and
	// dependency suggestions do not override the main compiler selection.
	for _, raw := range []string{"module example.com/dep\n", "module example.com/dep\ngo 1.21\ntoolchain go1.26.0\n"} {
		if _, err := parsePracticalModuleManifest(practicalModuleTestBuffer(t, "go_mod", raw), false); err != nil {
			t.Fatalf("type-check-only dependency metadata: %v", err)
		}
	}
}

func TestPracticalModuleManifestRejectsUnsatisfiedRequirements(t *testing.T) {
	cases := map[string]string{
		"missing-module": "go 1.25.0\n", "missing-go": "module example.com/app\n",
		"old-language": "module example.com/app\ngo 1.21\n", "future-language": "module example.com/app\ngo 1.26\n",
		"newer-patch": "module example.com/app\ngo 1.25.1\n", "newer-suggestion": "module example.com/app\ngo 1.25.0\ntoolchain go1.25.1\n",
		"unknown-directive": "module example.com/app\ngo 1.25.0\nunknown thing\n",
		"godebug":           "module example.com/app\ngo 1.25.0\ngodebug default=go1.24\n",
		"tool":              "module example.com/app\ngo 1.25.0\ntool example.com/tool\n",
		"ignore":            "module example.com/app\ngo 1.25.0\nignore ./hidden\n",
		"invalid-path":      "module ../app\ngo 1.25.0\n",
	}
	for name, raw := range cases {
		t.Run(name, func(t *testing.T) {
			if _, err := parsePracticalModuleManifest(practicalModuleTestBuffer(t, "go_mod", raw), true); err == nil {
				t.Fatal("invalid or unavailable module requirement accepted")
			}
		})
	}
	newerDependency := practicalModuleTestBuffer(t, "go_mod", "module example.com/dep\ngo 1.25.1\n")
	if _, err := parsePracticalModuleManifest(newerDependency, false); err == nil {
		t.Fatal("dependency patch requirement normalized away")
	}
	buffer := practicalModuleTestBuffer(t, "go_mod", "module example.com/app\ngo 1.25.0\n")
	buffer.data[0] = 'M'
	if _, err := parsePracticalModuleManifest(buffer, true); err == nil {
		t.Fatal("corrupted owned manifest accepted")
	}
	if _, err := parsePracticalModuleManifest(practicalModuleTestBuffer(t, "auxiliary", "module example.com/app\ngo 1.25.0\n"), true); err == nil {
		t.Fatal("wrong source role accepted")
	}
}

func TestPracticalModuleSumsBindExactManifestBytes(t *testing.T) {
	// Independent Go1.25-observed vector retained in T01-W02/module-probes.json.
	const raw = "module example.com/pracdep\n\ngo 1.22\n"
	const hash = "h1:RNha+Ovptpv+50nN2NH8md8DuZZwwlLUzexiDm4kE8g="
	line := "example.com/pracdep v1.0.0/go.mod " + hash
	source := practicalModuleTestBuffer(t, "go_sum", "\r\n"+line+"\n"+line+"\r\n")
	sums, err := parsePracticalModuleSums(source)
	if err != nil {
		t.Fatal(err)
	}
	identity := module.Version{Path: "example.com/pracdep", Version: "v1.0.0"}
	manifest := practicalModuleTestBuffer(t, "go_mod", raw)
	if err := sums.verifyManifest(identity, manifest); err != nil {
		t.Fatalf("Go-observed original manifest sum differs: %v", err)
	}
	if len(sums.entries) != 1 || !bytes.Equal(source.sourceBytes(), sums.source.sourceBytes()) {
		t.Fatal("duplicate checksum handling changed original bytes")
	}
	for _, invalid := range []struct {
		name string
		id   module.Version
		raw  string
	}{
		{"changed-bytes", identity, raw + "\n"},
		{"wrong-version", module.Version{Path: identity.Path, Version: "v1.0.1"}, raw},
		{"wrong-module", module.Version{Path: "example.com/other", Version: identity.Version}, raw},
	} {
		t.Run(invalid.name, func(t *testing.T) {
			if err := sums.verifyManifest(invalid.id, practicalModuleTestBuffer(t, "go_mod", invalid.raw)); err == nil {
				t.Fatal("missing or stale checksum accepted")
			}
		})
	}
	for _, content := range []string{"", strings.Replace(line, "v1.0.0/go.mod", "v1.0.0", 1)} {
		missing, err := parsePracticalModuleSums(practicalModuleTestBuffer(t, "go_sum", content))
		if err != nil || missing.verifyManifest(identity, manifest) == nil {
			t.Fatal("archive checksum or empty file substituted for a manifest checksum")
		}
	}
}

func TestPracticalModuleSumsRejectMalformedOrConflictingInputs(t *testing.T) {
	const prefix = "example.com/pracdep v1.0.0/go.mod "
	const valid = "h1:RNha+Ovptpv+50nN2NH8md8DuZZwwlLUzexiDm4kE8g="
	const other = "h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="
	cases := map[string]string{
		"conflict":    prefix + valid + "\n" + prefix + other + "\n",
		"extra-field": prefix + valid + " extra", "missing-field": prefix,
		"algorithm": prefix + strings.Replace(valid, "h1:", "h2:", 1), "short": prefix + "h1:YQ==",
		"base64":               prefix + "h1:" + strings.Repeat("!", 43) + "=",
		"noncanonical-version": "example.com/pracdep v1.0/go.mod " + valid,
		"wrong-major":          "example.com/pracdep v2.0.0/go.mod " + valid,
		"bad-suffix":           "example.com/pracdep v1.0.0/go.mod/go.mod " + valid,
		"bad-path":             "../dep v1.0.0/go.mod " + valid, "binary": string([]byte{0xff, 0}),
	}
	for name, raw := range cases {
		t.Run(name, func(t *testing.T) {
			if _, err := parsePracticalModuleSums(practicalModuleTestBuffer(t, "go_sum", raw)); err == nil {
				t.Fatal("malformed or conflicting original sums accepted")
			}
		})
	}
	if _, err := parsePracticalModuleSums(practicalModuleTestBuffer(t, "go_mod", "")); err == nil {
		t.Fatal("wrong checksum source role accepted")
	}
}
