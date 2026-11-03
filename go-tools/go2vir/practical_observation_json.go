package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"sort"
	"unicode/utf8"
)

// Only these private records carry complete original command byte streams.
// Keep the common canonical JSON limit unchanged: hexadecimal transport needs
// exactly twice the existing native stdout/stderr byte budgets.
func practicalObservationDomain(schema string) string {
	switch schema {
	case "mpk.go.practical.source_export_inputs.v1":
		return practicalExportInputsDomain
	case "mpk.go.practical.source_export_inputs.v2":
		return practicalCandidateExportInputsDomain
	case "mpk.go.practical.source_module_observations.v1":
		return practicalModuleObservationInputsDomain
	case "mpk.go.practical.source_module_graph_inputs.v2":
		return practicalModuleGraphInputsDomain
	default:
		return ""
	}
}

func practicalObservationHexLimit(field string) int {
	switch field {
	case "metadata_hex":
		return 2 * maximumCandidateBytes
	case "diagnostics_hex":
		return 2 * maximumContractBytes
	default:
		return 0
	}
}

func validPracticalObservationHex(field, value string) bool {
	limit := practicalObservationHexLimit(field)
	if limit == 0 || len(value) > limit || len(value)%2 != 0 {
		return false
	}
	for _, b := range []byte(value) {
		if !(b >= '0' && b <= '9' || b >= 'a' && b <= 'f') {
			return false
		}
	}
	return true
}

// Ordinary members pass through the unchanged canonical codec as a complete
// object, retaining its UTF8, cycle, integer, duplicate and nesting rules.
// Only two root-level lowercase-hex strings have byte-derived limits.
func canonicalPracticalObservationJSON(body map[string]jsonValue) ([]byte, error) {
	schema, ok := body["schema"].(string)
	if !ok || practicalObservationDomain(schema) == "" {
		return nil, fmt.Errorf("unknown private observation schema")
	}
	ordinary := make(map[string]jsonValue, len(body))
	for key, value := range body {
		if practicalObservationHexLimit(key) == 0 {
			ordinary[key] = value
		}
	}
	for _, field := range []string{"metadata_hex", "diagnostics_hex"} {
		value, ok := body[field].(string)
		if !ok || !validPracticalObservationHex(field, value) {
			return nil, fmt.Errorf("invalid original observation byte field %s", field)
		}
	}
	raw, err := canonicalJSON(ordinary)
	if err != nil {
		return nil, err
	}
	parsed, err := decodeStrictJSON(raw)
	if err != nil {
		return nil, err
	}
	values := parsed.(map[string]jsonValue)
	values["metadata_hex"], values["diagnostics_hex"] = body["metadata_hex"], body["diagnostics_hex"]
	keys := make([]string, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Slice(keys, func(i, j int) bool { return compareUTF16(keys[i], keys[j]) < 0 })
	var output bytes.Buffer
	output.WriteByte('{')
	for i, key := range keys {
		if i != 0 {
			output.WriteByte(',')
		}
		appendJSONString(&output, key)
		output.WriteByte(':')
		if practicalObservationHexLimit(key) != 0 {
			output.WriteByte('"')
			output.WriteString(values[key].(string))
			output.WriteByte('"')
		} else if err := appendCanonicalJSON(&output, values[key]); err != nil {
			return nil, err
		}
	}
	output.WriteByte('}')
	return output.Bytes(), nil
}

func sealPracticalObservationJSON(domain string, body map[string]jsonValue) ([]byte, error) {
	schema, _ := body["schema"].(string)
	if domain == "" || practicalObservationDomain(schema) != domain {
		return nil, fmt.Errorf("private observation domain does not match its schema")
	}
	if _, present := body["content_sha256"]; present {
		return nil, fmt.Errorf("private observation body already has a content hash")
	}
	canonical, err := canonicalPracticalObservationJSON(body)
	if err != nil {
		return nil, err
	}
	preimage := make([]byte, 0, len(domain)+1+len(canonical))
	preimage = append(preimage, domain...)
	preimage = append(preimage, 0)
	preimage = append(preimage, canonical...)
	complete := make(map[string]jsonValue, len(body)+1)
	for key, value := range body {
		complete[key] = value
	}
	complete["content_sha256"] = sha256Hex(preimage)
	return canonicalPracticalObservationJSON(complete)
}

func decodePracticalObservationJSON(raw []byte) (map[string]jsonValue, error) {
	if !utf8.Valid(raw) {
		return nil, fmt.Errorf("observation JSON is not UTF8")
	}
	if err := validateJSONStringEscapes(raw); err != nil {
		return nil, err
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	if first, err := decoder.Token(); err != nil || first != json.Delim('{') {
		return nil, fmt.Errorf("observation JSON requires one root object")
	}
	body := map[string]jsonValue{}
	for decoder.More() {
		token, err := decoder.Token()
		if err != nil {
			return nil, err
		}
		key, ok := token.(string)
		if !ok || len(key) > maximumStringBytes {
			return nil, fmt.Errorf("invalid observation JSON field")
		}
		if _, exists := body[key]; exists {
			return nil, fmt.Errorf("duplicate observation JSON field")
		}
		if practicalObservationHexLimit(key) != 0 {
			token, err := decoder.Token()
			if err != nil {
				return nil, err
			}
			value, ok := token.(string)
			if !ok || !validPracticalObservationHex(key, value) {
				return nil, fmt.Errorf("invalid original observation byte field %s", key)
			}
			body[key] = value
		} else {
			value, err := decodeJSONValue(decoder, 1)
			if err != nil {
				return nil, err
			}
			body[key] = value
		}
	}
	if last, err := decoder.Token(); err != nil || last != json.Delim('}') {
		return nil, fmt.Errorf("invalid observation JSON close")
	}
	if _, err := decoder.Token(); err != io.EOF {
		return nil, fmt.Errorf("trailing observation JSON")
	}
	schema, _ := body["schema"].(string)
	if practicalObservationDomain(schema) == "" {
		return nil, fmt.Errorf("unknown private observation schema")
	}
	for _, key := range []string{"metadata_hex", "diagnostics_hex"} {
		if _, ok := body[key].(string); !ok {
			return nil, fmt.Errorf("missing original observation bytes")
		}
	}
	return body, nil
}
