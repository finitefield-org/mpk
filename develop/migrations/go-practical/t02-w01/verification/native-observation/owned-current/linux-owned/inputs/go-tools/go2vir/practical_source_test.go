package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"strings"
	"testing"
)

func TestPracticalSourceFinalFrozenVector(t *testing.T) {
	var catalog struct {
		Vectors []struct {
			Expected string                `json:"expected"`
			Node     practicalSourceObject `json:"node"`
			Raw      string                `json:"raw_bytes_hex"`
			Fixture  struct {
				Path   string `json:"path"`
				SHA256 string `json:"sha256"`
				NodeID string `json:"node_id"`
			} `json:"fixture_ref"`
		} `json:"vectors"`
	}
	if err := json.Unmarshal(mustReadFile(t, repoPath("develop/migrations/go-practical/t01-w05/source-kind/catalog.json")), &catalog); err != nil {
		t.Fatal(err)
	}
	checked := 0
	for _, vector := range catalog.Vectors {
		if vector.Expected != "metadata_linked" {
			continue
		}
		checked++
		raw, err := hex.DecodeString(vector.Raw)
		if err != nil {
			t.Fatal(err)
		}
		buffer, err := newPracticalSourceBuffer(vector.Node.Capability, vector.Node.Path, vector.Node.Kind, raw)
		if err != nil || buffer.sourceObject() != vector.Node {
			t.Fatalf("final frozen source object differs: %#v, %v", buffer.sourceObject(), err)
		}
		record, err := buffer.canonicalObject()
		if err != nil {
			t.Fatal(err)
		}
		if _, err := decodePracticalSourceBuffer(record, raw); err != nil {
			t.Fatal(err)
		}
	}
	if checked != 1 {
		t.Fatalf("checked %d frozen source vectors, want 1", checked)
	}
	negative := 0
	for _, vector := range catalog.Vectors {
		if vector.Expected != "source_kind_rejected" {
			continue
		}
		// Read the exact original negative node referenced by the frozen catalog.
		// Do not relabel or regenerate the historical candidate graph.
		data := mustReadFile(t, repoPath(vector.Fixture.Path))
		if sha256Hex(data) != vector.Fixture.SHA256 {
			t.Fatal("original negative fixture identity changed")
		}
		var fixture struct {
			Nodes []json.RawMessage `json:"nodes"`
		}
		if err := json.Unmarshal(data, &fixture); err != nil {
			t.Fatal(err)
		}
		for _, raw := range fixture.Nodes {
			var object practicalSourceObject
			if err := json.Unmarshal(raw, &object); err != nil {
				t.Fatal(err)
			}
			if object.ID != vector.Fixture.NodeID {
				continue
			}
			negative++
			if _, err := newPracticalSourceBuffer(object.Capability, object.Path, object.Kind, nil); err == nil {
				t.Fatal("historical candidate source kind accepted")
			}
			if _, err := decodePracticalSourceBuffer(raw, nil); err == nil {
				t.Fatal("original historical candidate source object accepted")
			}
		}
	}
	if negative != 1 {
		t.Fatalf("checked %d frozen negative source objects, want 1", negative)
	}
}

func TestPracticalSourceOwnsExactOriginalBytes(t *testing.T) {
	raw := []byte{0xff, 0, 'A', 0xc0, 0x80, '\r', '\n'}
	buffer, err := newPracticalSourceBuffer("dependencies/Example@v1", "data/原文.bin", "auxiliary", raw)
	if err != nil {
		t.Fatal(err)
	}
	original := append([]byte(nil), raw...)
	raw[0] = 0
	exposed := buffer.sourceBytes()
	exposed[1] = 1
	object := buffer.sourceObject()
	object.ID = strings.Repeat("0", 64)
	if !bytes.Equal(buffer.sourceBytes(), original) || buffer.sourceObject().RawSHA256 != sha256Hex(original) {
		t.Fatal("source byte ownership or raw identity changed through a caller alias")
	}
	record, err := buffer.canonicalObject()
	if err != nil {
		t.Fatal(err)
	}
	if _, err := decodePracticalSourceBuffer(record, original); err != nil {
		t.Fatal(err)
	}
	if _, err := decodePracticalSourceBuffer(record, raw); err == nil {
		t.Fatal("changed original bytes were accepted")
	}
	for _, changed := range [][3]string{{"other", "data/原文.bin", "auxiliary"}, {"dependencies/Example@v1", "other.bin", "auxiliary"}, {"dependencies/Example@v1", "data/原文.bin", "export_data"}} {
		other, err := newPracticalSourceBuffer(changed[0], changed[1], changed[2], original)
		if err != nil || other.sourceObject().ID == buffer.sourceObject().ID {
			t.Fatalf("capability/path/kind missing from complete preimage: %v", err)
		}
	}
}

func TestPracticalSourceRejectsForgedRecordsAndOldKinds(t *testing.T) {
	buffer, err := newPracticalSourceBuffer("main", "policy.go", "go_file", []byte("package policy\n"))
	if err != nil {
		t.Fatal(err)
	}
	record, err := buffer.canonicalObject()
	if err != nil {
		t.Fatal(err)
	}
	parsed := mustStrictObject(t, record)
	for _, field := range []string{"schema", "id", "capability", "path", "kind", "raw_sha256", "bytes"} {
		t.Run("missing_"+field, func(t *testing.T) {
			forged := cloneJSONValue(t, parsed).(map[string]jsonValue)
			delete(forged, field)
			raw, _ := canonicalJSONValue(forged)
			if _, err := decodePracticalSourceBuffer(raw, buffer.sourceBytes()); err == nil {
				t.Fatalf("missing %s accepted", field)
			}
		})
	}
	for _, test := range []struct {
		field string
		value any
	}{
		{"schema", "mpk.go.source_object.v0"}, {"id", strings.Repeat("0", 64)},
		{"capability", "other"}, {"path", "other.go"},
		{"kind", "source"}, {"raw_sha256", strings.ToUpper(buffer.sourceObject().RawSHA256)},
		{"bytes", true}, {"bytes", int64(-1)}, {"bytes", int64(0)}, {"proof", true},
	} {
		forged := cloneJSONValue(t, parsed).(map[string]jsonValue)
		forged[test.field] = test.value
		raw, _ := canonicalJSONValue(forged)
		if _, err := decodePracticalSourceBuffer(raw, buffer.sourceBytes()); err == nil {
			t.Fatalf("forged %s=%v accepted", test.field, test.value)
		}
	}
	for _, raw := range [][]byte{
		append(append([]byte(nil), record...), record...),
		bytes.Replace(record, []byte(`"bytes":15`), []byte(`"bytes":15.0`), 1),
		append([]byte(`{"schema":"duplicate",`), record[1:]...),
	} {
		if _, err := decodePracticalSourceBuffer(raw, buffer.sourceBytes()); err == nil {
			t.Fatal("non-integral, duplicate or trailing JSON accepted")
		}
	}
	for _, kind := range []string{"source", "unknown", "GO_FILE"} {
		if _, err := newPracticalSourceBuffer("main", "policy.go", kind, nil); err == nil {
			t.Fatalf("old/unknown kind %s accepted", kind)
		}
	}
	if _, err := (practicalSourceBuffer{}).canonicalObject(); err == nil {
		t.Fatal("uninitialized buffer accepted")
	}
	empty, err := newPracticalSourceBuffer("main", "go.sum", "go_sum", nil)
	if err != nil {
		t.Fatal(err)
	}
	emptyRecord, err := empty.canonicalObject()
	if err != nil {
		t.Fatal(err)
	}
	if _, err := decodePracticalSourceBuffer(bytes.Replace(emptyRecord, []byte(`"bytes":0`), []byte(`"bytes":null`), 1), nil); err == nil {
		t.Fatal("null was accepted as an empty source's integer byte count")
	}
}

func TestPracticalSourcePathsKindsAndLimits(t *testing.T) {
	for _, kind := range []string{"go_file", "go_mod", "go_sum", "overlay", "metadata", "export_data", "excluded_source", "auxiliary"} {
		if _, err := newPracticalSourceBuffer("main", "original", kind, nil); err != nil {
			t.Fatalf("frozen kind %s rejected: %v", kind, err)
		}
	}
	for _, path := range []string{"", ".", "..", "../x", "/x", "x/../y", "x//y", "x/", "x\\y", "x\x00y", string([]byte{0xff}), strings.Repeat("x", 1025)} {
		if _, err := newPracticalSourceBuffer("main", path, "go_file", nil); err == nil {
			t.Fatalf("invalid path %q accepted", path)
		}
		if _, err := newPracticalSourceBuffer(path, "policy.go", "go_file", nil); err == nil {
			t.Fatalf("invalid capability %q accepted", path)
		}
	}
	if _, err := newPracticalSourceBuffer("main", strings.Repeat("x", 1024), "go_file", nil); err != nil {
		t.Fatalf("inclusive logical-path byte boundary rejected: %v", err)
	}
	first, err := newPracticalSourceBuffer("main", "Å.go", "go_file", nil)
	if err != nil {
		t.Fatal(err)
	}
	second, err := newPracticalSourceBuffer("main", "A\u030a.go", "go_file", nil)
	if err != nil || first.sourceObject().ID == second.sourceObject().ID {
		t.Fatal("source path Unicode normalization changed identity")
	}
	for _, test := range []struct {
		kind  string
		limit int
	}{{"go_file", maximumCandidateBytes}, {"overlay", maximumContractBytes}} {
		if _, err := newPracticalSourceBuffer("main", "original", test.kind, make([]byte, test.limit)); err != nil {
			t.Fatalf("inclusive %s byte boundary rejected: %v", test.kind, err)
		}
		if _, err := newPracticalSourceBuffer("main", "original", test.kind, make([]byte, test.limit+1)); err == nil {
			t.Fatalf("%s above byte boundary accepted", test.kind)
		}
	}
}

func TestPracticalSourceDoesNotActivateInstalledProfile(t *testing.T) {
	fixture := loadInvocationFixture(t)
	args := replaceOptionValue(fixture.Args, "--semantic-profile", "mpk.go.practical.v1")
	var stdout, stderr bytes.Buffer
	if code := run(args, &stdout, &stderr); code != 2 || stdout.Len() != 0 {
		t.Fatalf("inactive practical profile reached public lowering: code=%d stdout=%s", code, stdout.Bytes())
	}
	if !strings.HasPrefix(stderr.String(), usage) {
		t.Fatalf("inactive profile was not rejected before source reads: %s", stderr.Bytes())
	}
}
