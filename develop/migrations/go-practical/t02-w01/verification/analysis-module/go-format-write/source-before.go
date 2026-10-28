//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path"
	"path/filepath"
	"sort"
	"testing"
)

// Compare module/checksum facts relative to the full original snapshot.
// The snapshot itself remains untrusted and cached .info remains separate.
func TestPracticalAnalysisModuleDifferential(t *testing.T) {
	roots := practicalAnalysisTestRoots(t, "")
	session := practicalTestSession(t, roots)
	inputs, err := session.capturePracticalAnalysis(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability})
	if err != nil { t.Fatal(err) }
	original, err := session.seal()
	if err != nil { t.Fatal(err) }
	originalInputs, err := original.snapshotInputs()
	if err != nil || len(originalInputs.sources) != 213 || len(inputs.vendor) != 133 || len(inputs.modules) != 3 { t.Fatalf("original owner differs: %v", err) }
	lookup := func(capability, name, kind string) practicalSourceBuffer {
		t.Helper()
		source, err := original.readSource(capability, name, kind)
		if err != nil { t.Fatal(err) }
		return source
	}
	frontend := roots[0].Capability
	cache := roots[1].Capability
	mod := lookup(frontend, "go.mod", "go_mod").sourceBytes()
	sums := lookup(frontend, "go.sum", "go_sum").sourceBytes()
	cacheModPath := "golang.org/x/mod/@v/v0.27.0.mod"
	cacheMod := lookup(cache, cacheModPath, "go_mod").sourceBytes()
	zipHashPath := "golang.org/x/mod/@v/v0.27.0.ziphash"
	zipHash := lookup(cache, zipHashPath, "metadata").sourceBytes()
	replace := func(raw []byte, old, new string) []byte {
		t.Helper()
		if !bytes.Contains(raw, []byte(old)) { t.Fatalf("missing mutation anchor %q", old) }
		return bytes.Replace(raw, []byte(old), []byte(new), 1)
	}
	type probe struct { name, capability, filename, kind string; data []byte; unread, want bool }
	probes := []probe{
		{name: "original", want: true},
		{name: "frontend-comment", capability: frontend, filename: "go.mod", kind: "go_mod", data: append(bytes.Clone(mod), []byte("\n// retained comment\n")...), want: true},
		{name: "wrong-module", capability: frontend, filename: "go.mod", kind: "go_mod", data: replace(mod, practicalFrontendModule, "example.com/caller")},
		{name: "wrong-go-version", capability: frontend, filename: "go.mod", kind: "go_mod", data: replace(mod, "go 1.23.0", "go 1.24.0")},
		{name: "wrong-dependency-version", capability: frontend, filename: "go.mod", kind: "go_mod", data: replace(mod, "golang.org/x/tools v0.36.0", "golang.org/x/tools v0.36.1")},
		{name: "wrong-indirect", capability: frontend, filename: "go.mod", kind: "go_mod", data: replace(mod, " // indirect", "")},
		{name: "duplicate-requirement", capability: frontend, filename: "go.mod", kind: "go_mod", data: append(bytes.Clone(mod), []byte("\nrequire golang.org/x/mod v0.27.0\n")...)},
		{name: "replacement", capability: frontend, filename: "go.mod", kind: "go_mod", data: append(bytes.Clone(mod), []byte("\nreplace golang.org/x/mod => ../caller\n")...)},
		{name: "exclusion", capability: frontend, filename: "go.mod", kind: "go_mod", data: append(bytes.Clone(mod), []byte("\nexclude golang.org/x/mod v0.27.0\n")...)},
		{name: "retraction", capability: frontend, filename: "go.mod", kind: "go_mod", data: append(bytes.Clone(mod), []byte("\nretract v1.0.0\n")...)},
		{name: "toolchain", capability: frontend, filename: "go.mod", kind: "go_mod", data: append(bytes.Clone(mod), []byte("\ntoolchain go1.25.0\n")...)},
		{name: "wrong-archive-sum", capability: frontend, filename: "go.sum", kind: "go_sum", data: replace(sums, practicalAnalysisPins[0].sum, "h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")},
		{name: "wrong-mod-sum", capability: frontend, filename: "go.sum", kind: "go_sum", data: replace(sums, practicalAnalysisPins[0].modSum, "h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")},
		{name: "duplicate-identical-sum", capability: frontend, filename: "go.sum", kind: "go_sum", data: append(bytes.Clone(sums), []byte("golang.org/x/mod v0.27.0 "+practicalAnalysisPins[0].sum+"\n")...), want: true},
		{name: "conflicting-sum", capability: frontend, filename: "go.sum", kind: "go_sum", data: append(bytes.Clone(sums), []byte("golang.org/x/mod v0.27.0 h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=\n")...)},
		{name: "unread-sums", capability: frontend, filename: "go.sum", kind: "go_sum", unread: true},
		{name: "changed-cached-mod", capability: cache, filename: cacheModPath, kind: "go_mod", data: append(bytes.Clone(cacheMod), '\n')},
		{name: "unread-cached-mod", capability: cache, filename: cacheModPath, kind: "go_mod", unread: true},
		{name: "wrong-ziphash", capability: cache, filename: zipHashPath, kind: "metadata", data: []byte("h1:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")},
		{name: "ziphash-trim-space", capability: cache, filename: zipHashPath, kind: "metadata", data: append(append([]byte("\u00a0\t"), zipHash...), []byte("\r\n")...), want: true},
	}
	check := func(snapshot practicalSourceSnapshot) error {
		modSource, err := snapshot.readSource(frontend, "go.mod", "go_mod"); if err != nil { return err }
		manifest, err := parsePracticalModuleManifest(modSource, false); if err != nil { return err }
		sumSource, err := snapshot.readSource(frontend, "go.sum", "go_sum"); if err != nil { return err }
		originalSums, err := parsePracticalModuleSums(sumSource); if err != nil { return err }
		if err := validatePracticalAnalysisManifest(manifest, originalSums); err != nil { return err }
		for _, module := range inputs.modules {
			prefix := path.Join(module.identity.Path, "@v", module.identity.Version)
			cached, err := snapshot.readSource(cache, prefix+".mod", "go_mod"); if err != nil { return err }
			if err := originalSums.verifyManifest(module.identity, cached); err != nil { return err }
			if _, err := parsePracticalModuleManifest(cached, false); err != nil { return err }
			hash, err := snapshot.readSource(cache, prefix+".ziphash", "metadata"); if err != nil { return err }
			if string(bytes.TrimSpace(hash.data)) != originalSums.entries[module.identity] || module.archive.sum != originalSums.entries[module.identity] { return fmt.Errorf("archive and checksum mismatch") }
			if _, present := module.archive.members["go.mod"]; present {
				if err := module.archive.verifyContent("go.mod", cached); err != nil { return err }
			}
		}
		return nil
	}
	type sourceRecord struct { Object practicalSourceObject `json:"object"`; RawFile string `json:"raw_file"` }
	type observation struct { Name string `json:"name"`; Snapshot string `json:"snapshot_hex"`; SourceIDs []string `json:"source_ids"`; Changed *sourceRecord `json:"changed"`; Accepted bool `json:"accepted"` }
	var rows []observation
	out := os.Getenv("MPK_GO_ANALYSIS_MODULE_OUTPUT")
	if out != "" { if err := os.MkdirAll(filepath.Join(out, "changed-bytes"), 0o700); err != nil { t.Fatal(err) } }
	for _, probe := range probes {
		candidate := practicalSourceSnapshot{buffers: map[string]practicalSourceBuffer{}, paths: map[string]string{}, directories: original.directories, ids: append([]string(nil), original.ids...), roots: append([]string(nil), original.roots...)}
		for key, value := range original.buffers { candidate.buffers[key] = value }
		for key, value := range original.paths { candidate.paths[key] = value }
		var changed *sourceRecord
		if probe.filename != "" {
			key := practicalCaptureKey(probe.capability, probe.filename)
			oldID := candidate.paths[key]
			if oldID == "" { t.Fatalf("%s: missing original source", probe.name) }
			delete(candidate.buffers, oldID); delete(candidate.paths, key)
			ids := make([]string, 0, len(candidate.ids))
			for _, id := range candidate.ids { if id != oldID { ids = append(ids, id) } }
			candidate.ids = ids
			if !probe.unread {
				source, err := newPracticalSourceBuffer(probe.capability, probe.filename, probe.kind, probe.data); if err != nil { t.Fatal(err) }
				candidate.buffers[source.object.ID] = source; candidate.paths[key] = source.object.ID; candidate.ids = append(candidate.ids, source.object.ID); sort.Strings(candidate.ids)
				filename := source.object.RawSHA256 + ".raw"
				changed = &sourceRecord{source.object, filename}
				if out != "" { if err := os.WriteFile(filepath.Join(out, "changed-bytes", filename), source.sourceBytes(), 0o600); err != nil { t.Fatal(err) } }
			}
		}
		candidate.contentSHA256 = practicalSnapshotDigest(candidate)
		wire, err := candidate.snapshotInputs(); if err != nil { t.Fatalf("%s snapshot: %v", probe.name, err) }
		err = check(candidate)
		if (err == nil) != probe.want { t.Fatalf("%s accepted=%v error=%v", probe.name, probe.want, err) }
		rows = append(rows, observation{probe.name, hex.EncodeToString(wire.canonical), candidate.ids, changed, err == nil})
	}
	if out != "" {
		data, err := json.MarshalIndent(map[string]any{"cases": rows, "original_sources": len(originalInputs.sources), "vendor_files": len(inputs.vendor), "info_or_root_authority_claimed": false}, "", "  "); if err != nil { t.Fatal(err) }
		if err := os.WriteFile(filepath.Join(out, "native.json"), append(data, '\n'), 0o600); err != nil { t.Fatal(err) }
	}
}
