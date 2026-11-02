//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"os"
	"path"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

// Native struct/time observations retain the complete original analysis owner.
func TestPracticalAnalysisInfoDifferential(t *testing.T) {
	roots := practicalAnalysisTestRoots(t, "")
	session := practicalTestSession(t, roots)
	inputs, err := session.capturePracticalAnalysis(practicalModuleLocation{capability: roots[0].Capability}, practicalModuleLocation{capability: roots[1].Capability})
	if err != nil { t.Fatal(err) }
	original, err := session.seal()
	if err != nil { t.Fatal(err) }
	originalInputs, err := original.snapshotInputs()
	if err != nil || len(originalInputs.sources) != 213 || len(inputs.vendor) != 133 || len(inputs.modules) != 3 { t.Fatalf("original owner differs: %v", err) }
	base := inputs.modules[0].info
	identity := inputs.modules[0].identity
	raw := base.sourceBytes()
	replace := func(old, new string) []byte {
		t.Helper(); if !bytes.Contains(raw, []byte(old)) { t.Fatalf("missing original info anchor %q", old) }
		return bytes.Replace(raw, []byte(old), []byte(new), 1)
	}
	appendField := func(field []byte) []byte {
		t.Helper(); at := bytes.LastIndexByte(raw, '}'); if at < 0 { t.Fatal("missing object end") }
		return bytes.Join([][]byte{raw[:at], []byte(","), field, raw[at:]}, nil)
	}
	const stamp = `"Time":"2025-08-07T14:51:59Z"`
	timeValue := func(value string) []byte { return replace(stamp, `"Time":`+value) }
	type probe struct { name string; data []byte; want bool }
	probes := []probe{
		{"wrong-version", replace(`"Version":"v0.27.0"`, `"Version":"v0.27.1"`), false},
		{"version-before-exact", replace(`"Version":"v0.27.0"`, `"version":"v0.0.0","Version":"v0.27.0"`), true},
		{"version-after-exact", replace(`"Version":"v0.27.0"`, `"Version":"v0.27.0","version":"v0.0.0"`), false},
		{"version-null-after", replace(`"Version":"v0.27.0"`, `"Version":"v0.27.0","version":null`), true},
		{"version-bad-type-before", replace(`"Version":"v0.27.0"`, `"version":12,"Version":"v0.27.0"`), false},
		{"unicode-fold-version", replace(`"Version":"v0.27.0"`, `"Verſion":"v0.27.0"`), true},
		{"duplicate-version", replace(`"Version":"v0.27.0"`, `"Version":"v0.27.0","Version":"v0.27.0"`), false},
		{"time-null", timeValue("null"), true},
		{"time-type", timeValue("17"), false},
		{"time-single-hour", timeValue(`"2025-08-07T4:51:59Z"`), true},
		{"time-comma-fraction", timeValue(`"2025-08-07T14:51:59,123456789012345Z"`), true},
		{"time-zone-24-60", timeValue(`"2025-08-07T14:51:59+24:60"`), true},
		{"time-zone-25", timeValue(`"2025-08-07T14:51:59+25:00"`), false},
		{"time-zone-61", timeValue(`"2025-08-07T14:51:59+00:61"`), false},
		{"time-leap-day", timeValue(`"2024-02-29T14:51:59Z"`), true},
		{"time-non-leap-day", timeValue(`"2025-02-29T14:51:59Z"`), false},
		{"time-leap-second", timeValue(`"2025-08-07T14:51:60Z"`), false},
		{"time-escaped-z", timeValue(`"2025-08-07T14:51:59\u005a"`), false},
		{"time-lowercase-z", timeValue(`"2025-08-07T14:51:59z"`), false},
		{"time-missing-zone", timeValue(`"2025-08-07T14:51:59"`), false},
		{"time-fraction-no-digits", timeValue(`"2025-08-07T14:51:59.Z"`), false},
		{"origin-field-null", replace(`"VCS":"git"`, `"VCS":null`), true},
		{"origin-field-type", replace(`"VCS":"git"`, `"VCS":true`), false},
		{"unicode-fold-origin-field", replace(`"VCS":"git"`, `"VCſ":true`), false},
		{"origin-type", replace(`"Origin":{`, `"Origin":"not-an-object","extra":{`), false},
		{"unknown-object", appendField([]byte(`"Extra":{"opaque":[true,null,1]}`)), true},
		{"unknown-floating-number", appendField([]byte(`"Extra":1.0`)), false},
		{"unknown-duplicate-key", appendField([]byte(`"Extra":{"a":1,"a":1}`)), false},
		{"unknown-invalid-utf8", appendField(append([]byte(`"Extra":"`), []byte{0xff, '"'}...)), false},
		{"string-at-limit", appendField([]byte(`"Extra":"`+strings.Repeat("x", maximumStringBytes)+`"`)), true},
		{"string-one-over", appendField([]byte(`"Extra":"`+strings.Repeat("x", maximumStringBytes+1)+`"`)), false},
		{"depth-at-limit", appendField([]byte(`"Extra":`+strings.Repeat("[", maximumJSONDepth-1)+"0"+strings.Repeat("]", maximumJSONDepth-1))), true},
		{"depth-one-over", appendField([]byte(`"Extra":`+strings.Repeat("[", maximumJSONDepth)+"0"+strings.Repeat("]", maximumJSONDepth))), false},
	}
	type sourceRecord struct { Object practicalSourceObject `json:"object"`; RawFile string `json:"raw_file"` }
	type observation struct { Name string `json:"name"`; Source sourceRecord `json:"source"`; Version string `json:"version"`; Accepted bool `json:"accepted"` }
	out := os.Getenv("MPK_GO_ANALYSIS_INFO_OUTPUT")
	if out != "" { if err := os.MkdirAll(filepath.Join(out, "observed-bytes"), 0o700); err != nil { t.Fatal(err) } }
	store := func(source practicalSourceBuffer) sourceRecord {
		t.Helper(); filename := source.object.RawSHA256+".raw"; data := source.sourceBytes()
		if out != "" {
			file := filepath.Join(out, "observed-bytes", filename)
			old, err := os.ReadFile(file)
			if os.IsNotExist(err) { if err := os.WriteFile(file, data, 0o600); err != nil { t.Fatal(err) } } else if err != nil || !bytes.Equal(old, data) { t.Fatal("different source bytes share a digest") }
		}
		return sourceRecord{source.object, filename}
	}
	var rows []observation
	for _, module := range inputs.modules {
		if err := validatePracticalModuleInfo(module.info, module.identity); err != nil { t.Fatal(err) }
		rows = append(rows, observation{"original-"+module.identity.Path, store(module.info), module.identity.Version, true})
	}
	var rewritten = map[string]practicalSourceBuffer{}
	for _, probe := range probes {
		source, err := newPracticalSourceBuffer(base.object.Capability, base.object.Path, "metadata", probe.data); if err != nil { t.Fatal(err) }
		err = validatePracticalModuleInfo(source, identity)
		if (err == nil) != probe.want { t.Fatalf("%s: want accepted=%v: %v", probe.name, probe.want, err) }
		rewritten[probe.name] = source
		rows = append(rows, observation{probe.name, store(source), identity.Version, err == nil})
	}
	type snapshotRow struct { Name string `json:"name"`; Snapshot string `json:"snapshot_hex"`; SourceIDs []string `json:"source_ids"`; Changed *sourceRecord `json:"changed"`; Accepted bool `json:"accepted"` }
	var snapshots []snapshotRow
	for _, probe := range []struct { name, source string; unread, want bool }{
		{"original-owner", "", false, true}, {"valid-rewrite", "unknown-object", false, true}, {"wrong-version", "wrong-version", false, false}, {"escaped-time", "time-escaped-z", false, false}, {"unread-info", "", true, false},
	} {
		candidate := practicalSourceSnapshot{buffers: map[string]practicalSourceBuffer{}, paths: map[string]string{}, directories: original.directories, ids: append([]string(nil), original.ids...), roots: append([]string(nil), original.roots...)}
		for key, value := range original.buffers { candidate.buffers[key] = value }; for key, value := range original.paths { candidate.paths[key] = value }
		var changed *sourceRecord
		if probe.source != "" || probe.unread {
			key := practicalCaptureKey(base.object.Capability, base.object.Path); old := candidate.paths[key]
			delete(candidate.paths, key); delete(candidate.buffers, old)
			ids := make([]string, 0, len(candidate.ids)); for _, id := range candidate.ids { if id != old { ids = append(ids,id) } }; candidate.ids = ids
			if !probe.unread { source := rewritten[probe.source]; candidate.paths[key] = source.object.ID; candidate.buffers[source.object.ID] = source; candidate.ids = append(candidate.ids,source.object.ID); sort.Strings(candidate.ids); row := store(source); changed = &row }
		}
		candidate.contentSHA256 = practicalSnapshotDigest(candidate)
		wire, err := candidate.snapshotInputs(); if err != nil { t.Fatal(err) }
		var checkErr error
		for _, module := range inputs.modules {
			name := path.Join(module.identity.Path, "@v", module.identity.Version+".info")
			source, err := candidate.readSource(roots[1].Capability, name, "metadata"); if err != nil { checkErr = err; break }
			if err := validatePracticalModuleInfo(source, module.identity); err != nil { checkErr = err; break }
		}
		if (checkErr == nil) != probe.want { t.Fatalf("%s: %v", probe.name, checkErr) }
		snapshots = append(snapshots, snapshotRow{probe.name, hex.EncodeToString(wire.canonical), candidate.ids, changed, checkErr == nil})
	}
	if out != "" {
		data, err := json.MarshalIndent(map[string]any{"observations":rows,"snapshot_controls":snapshots,"original_sources":len(originalInputs.sources),"vendor_files":len(inputs.vendor),"root_or_vcs_authority_claimed":false}, "", "  "); if err != nil { t.Fatal(err) }
		if err := os.WriteFile(filepath.Join(out,"native.json"), append(data,'\n'), 0o600); err != nil { t.Fatal(err) }
	}
}
