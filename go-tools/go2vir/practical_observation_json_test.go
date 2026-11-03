//go:build go1.25 && (linux || darwin)

package main

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func practicalObservationJSONTestFile(t *testing.T, name string, raw []byte) {
	t.Helper()
	if output := os.Getenv("MPK_GO_PRACTICAL_SYNTAX_EVIDENCE"); output != "" {
		if err := os.WriteFile(filepath.Join(output, name), raw, 0o600); err != nil {
			t.Fatal(err)
		}
	}
}

func TestPracticalObservationJSONPreservesExactNativeByteBudgets(t *testing.T) {
	practicalContractTestOutput(t, "observation-json")
	rows := []any{}
	for _, schema := range []string{"mpk.go.practical.source_export_inputs.v1", "mpk.go.practical.source_export_inputs.v2", "mpk.go.practical.source_module_observations.v1", "mpk.go.practical.source_module_graph_inputs.v2"} {
		for _, field := range []string{"metadata_hex", "diagnostics_hex"} {
			maximum := practicalObservationHexLimit(field) / 2
			for _, size := range []int{0, 1, maximumStringBytes / 2, maximumStringBytes/2 + 1, maximum} {
				name := fmt.Sprintf("%s-%s-%d", schema, field, size)
				original := bytes.Repeat([]byte{0, 0xff, '\n', 0xe2, 0x82, 0xac, '"', '\\'}, (size+7)/8)[:size]
				body := map[string]any{"schema": schema, "metadata_hex": "", "diagnostics_hex": "", "original_bytes": int64(size)}
				body[field] = hex.EncodeToString(original)
				canonical, err := sealPracticalObservationJSON(practicalObservationDomain(schema), body)
				if err != nil {
					t.Fatal(name, err)
				}
				decoded, err := decodePracticalObservationJSON(canonical)
				if err != nil {
					t.Fatal(name, err)
				}
				restored, err := hex.DecodeString(decoded[field].(string))
				if err != nil || !bytes.Equal(restored, original) {
					t.Fatal("transport changed original bytes", name, err)
				}
				if 2*size <= maximumStringBytes {
					oldHash, err := hashTypedCanonicalJSON(practicalObservationDomain(schema), body)
					if err != nil || oldHash != decoded["content_sha256"] {
						t.Fatal("existing private hash changed", name, err)
					}
				} else if _, err := canonicalJSON(body); err == nil {
					t.Fatal("common JSON string limit changed")
				}
				practicalObservationJSONTestFile(t, name+".canonical.json", canonical)
				practicalObservationJSONTestFile(t, name+".original.raw", original)
				rows = append(rows, map[string]any{"case": name, "schema": schema, "field": field, "raw_bytes": size, "raw_sha256": sha256Hex(original), "canonical_file": name + ".canonical.json", "original_file": name + ".original.raw", "canonical_sha256": sha256Hex(canonical), "content_sha256": decoded["content_sha256"], "same_original_bytes": true})
			}
		}
	}
	retainPracticalSyntaxEvidence(t, "complete-observation-json-budgets.json", map[string]any{"cases": rows, "common_string_bytes_unchanged": maximumStringBytes, "native_stdout_bytes_unchanged": maximumCandidateBytes, "native_stderr_bytes_unchanged": maximumContractBytes, "source_or_context_authority": false})
}

func TestPracticalObservationJSONRejectsEscapesAndNonByteFields(t *testing.T) {
	practicalContractTestOutput(t, "observation-json-controls")
	rows := []any{}
	base := func() map[string]any {
		return map[string]any{"schema": "mpk.go.practical.source_export_inputs.v1", "metadata_hex": "", "diagnostics_hex": ""}
	}
	for _, name := range []string{"metadata-over", "diagnostics-over", "odd-hex", "uppercase-hex", "nonhex", "null", "array", "missing", "unknown-schema", "ordinary-over", "nested-metadata-over", "unsafe-integer", "cycle", "invalid-utf8"} {
		body := base()
		switch name {
		case "metadata-over":
			body["metadata_hex"] = strings.Repeat("00", maximumCandidateBytes+1)
		case "diagnostics-over":
			body["diagnostics_hex"] = strings.Repeat("00", maximumContractBytes+1)
		case "odd-hex":
			body["metadata_hex"] = "0"
		case "uppercase-hex":
			body["metadata_hex"] = "FF"
		case "nonhex":
			body["metadata_hex"] = "gg"
		case "null":
			body["metadata_hex"] = nil
		case "array":
			body["metadata_hex"] = []any{}
		case "missing":
			delete(body, "diagnostics_hex")
		case "unknown-schema":
			body["schema"] = "mpk.go.practical.caller_observation.v1"
		case "ordinary-over":
			body["ordinary"] = strings.Repeat("a", maximumStringBytes+1)
		case "nested-metadata-over":
			body["nested"] = map[string]any{"metadata_hex": strings.Repeat("a", maximumStringBytes+1)}
		case "unsafe-integer":
			body["ordinary"] = maximumJSONInteger + 1
		case "cycle":
			body["ordinary"] = body
		case "invalid-utf8":
			body["ordinary"] = string([]byte{255})
		}
		if _, err := sealPracticalObservationJSON(practicalExportInputsDomain, body); err == nil {
			t.Fatal("invalid programmatic observation accepted", name)
		}
		raw, marshalErr := json.Marshal(body)
		var decodeError string
		if marshalErr == nil && name != "invalid-utf8" {
			if _, err := decodePracticalObservationJSON(raw); err == nil {
				t.Fatal("invalid serialized observation accepted", name)
			} else {
				decodeError = err.Error()
			}
			practicalObservationJSONTestFile(t, name+".rejected.json", raw)
		}
		rows = append(rows, map[string]any{"case": name, "marshal_error": fmt.Sprint(marshalErr), "decode_error": decodeError, "programmatic_rejected": true, "complete_input_file": name + ".rejected.json", "serialized_input_retained": marshalErr == nil && name != "invalid-utf8"})
	}
	for name, raw := range map[string]string{
		"duplicate-root":   `{"schema":"mpk.go.practical.source_export_inputs.v1","metadata_hex":"","metadata_hex":"00","diagnostics_hex":""}`,
		"duplicate-nested": `{"schema":"mpk.go.practical.source_export_inputs.v1","metadata_hex":"","diagnostics_hex":"","nested":{"a":1,"a":2}}`,
		"lone-surrogate":   `{"schema":"mpk.go.practical.source_export_inputs.v1","metadata_hex":"\ud800","diagnostics_hex":""}`,
		"trailing":         `{"schema":"mpk.go.practical.source_export_inputs.v1","metadata_hex":"","diagnostics_hex":""}{}`,
		"fraction":         `{"schema":"mpk.go.practical.source_export_inputs.v1","metadata_hex":"","diagnostics_hex":"","ordinary":1.0}`,
	} {
		if _, err := decodePracticalObservationJSON([]byte(raw)); err == nil {
			t.Fatal("malformed observation accepted", name)
		}
		practicalObservationJSONTestFile(t, name+".rejected.json", []byte(raw))
		rows = append(rows, map[string]any{"case": name, "complete_input_file": name + ".rejected.json", "serialized_input_retained": true})
	}
	if _, err := sealPracticalObservationJSON(practicalModuleGraphInputsDomain, base()); err == nil {
		t.Fatal("cross-schema hash domain accepted")
	}
	retainPracticalSyntaxEvidence(t, "complete-observation-json-controls.json", map[string]any{"cases": rows, "cross_domain_rejected": true, "common_codec_unchanged": true})
}
