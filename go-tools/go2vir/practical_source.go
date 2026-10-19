package main

import (
	"fmt"
	"strings"
	"unicode/utf8"
)

const (
	practicalSourceObjectSchema = "mpk.go.source_object.v1"
	practicalSourceObjectDomain = "MPK-GO-SOURCE-OBJECT-1.0"
)

// A source object describes original bytes. It does not authorize a module,
// compiler, selected declaration, or source-bound proof.
type practicalSourceObject struct {
	Schema     string `json:"schema"`
	ID         string `json:"id"`
	Capability string `json:"capability"`
	Path       string `json:"path"`
	Kind       string `json:"kind"`
	RawSHA256  string `json:"raw_sha256"`
	Bytes      int64  `json:"bytes"`
}

// Keep the owned bytes private; callers receive independent copies.
type practicalSourceBuffer struct {
	object practicalSourceObject
	data   []byte
}

func newPracticalSourceBuffer(capability, path, kind string, data []byte) (practicalSourceBuffer, error) {
	if len(data) > maximumCandidateBytes || kind == "overlay" && len(data) > maximumContractBytes {
		return practicalSourceBuffer{}, fmt.Errorf("practical source object exceeds its byte limit")
	}
	owned := append([]byte(nil), data...)
	object, err := practicalSourceObjectFor(capability, path, kind, owned)
	if err != nil {
		return practicalSourceBuffer{}, err
	}
	return practicalSourceBuffer{object: object, data: owned}, nil
}

func practicalSourceObjectFor(capability, path, kind string, data []byte) (practicalSourceObject, error) {
	if !validPracticalLogicalPath(capability) || !validPracticalLogicalPath(path) {
		return practicalSourceObject{}, fmt.Errorf("practical source capability/path is not a normalized logical path")
	}
	switch kind {
	case "go_file", "go_mod", "go_sum", "overlay", "metadata", "export_data", "excluded_source", "auxiliary":
	default:
		return practicalSourceObject{}, fmt.Errorf("unknown practical source kind %q", kind)
	}
	if len(data) > maximumCandidateBytes || kind == "overlay" && len(data) > maximumContractBytes {
		return practicalSourceObject{}, fmt.Errorf("practical source object exceeds its byte limit")
	}
	object := practicalSourceObject{
		Schema: practicalSourceObjectSchema, Capability: capability, Path: path,
		Kind: kind, RawSHA256: sha256Hex(data), Bytes: int64(len(data)),
	}
	id, err := hashTypedCanonicalJSON(practicalSourceObjectDomain, map[string]any{
		"schema": object.Schema, "capability": object.Capability, "path": object.Path,
		"kind": object.Kind, "raw_sha256": object.RawSHA256, "bytes": object.Bytes,
	})
	if err != nil {
		return practicalSourceObject{}, err
	}
	object.ID = id
	return object, nil
}

func validPracticalLogicalPath(path string) bool {
	if !utf8.ValidString(path) || len(path) == 0 || len(path) > 1024 || strings.HasPrefix(path, "/") || strings.Contains(path, "\\") {
		return false
	}
	for _, character := range path {
		if character < 0x20 {
			return false
		}
	}
	for _, part := range strings.Split(path, "/") {
		if part == "" || part == "." || part == ".." {
			return false
		}
	}
	return true
}

func (buffer practicalSourceBuffer) sourceObject() practicalSourceObject {
	return buffer.object
}

func (buffer practicalSourceBuffer) sourceBytes() []byte {
	return append([]byte(nil), buffer.data...)
}

func (buffer practicalSourceBuffer) canonicalObject() ([]byte, error) {
	expected, err := practicalSourceObjectFor(buffer.object.Capability, buffer.object.Path, buffer.object.Kind, buffer.data)
	if err != nil {
		return nil, err
	}
	if buffer.object != expected {
		return nil, fmt.Errorf("practical source buffer identity is inconsistent")
	}
	return canonicalJSON(buffer.object)
}

func decodePracticalSourceBuffer(record, data []byte) (practicalSourceBuffer, error) {
	parsed, err := decodeStrictJSON(record)
	if err != nil {
		return practicalSourceBuffer{}, err
	}
	fields, ok := parsed.(map[string]jsonValue)
	if !ok || len(fields) != 7 {
		return practicalSourceBuffer{}, fmt.Errorf("practical source object must have exactly seven required fields")
	}
	for _, key := range []string{"schema", "id", "capability", "path", "kind", "raw_sha256", "bytes"} {
		if _, exists := fields[key]; !exists {
			return practicalSourceBuffer{}, fmt.Errorf("practical source object is missing %s", key)
		}
	}
	for _, key := range []string{"schema", "id", "capability", "path", "kind", "raw_sha256"} {
		if _, ok := fields[key].(string); !ok {
			return practicalSourceBuffer{}, fmt.Errorf("practical source object %s must be a string", key)
		}
	}
	size, ok := fields["bytes"].(int64)
	if !ok {
		return practicalSourceBuffer{}, fmt.Errorf("practical source object bytes must be an integer")
	}
	object := practicalSourceObject{
		Schema: fields["schema"].(string), ID: fields["id"].(string),
		Capability: fields["capability"].(string), Path: fields["path"].(string),
		Kind: fields["kind"].(string), RawSHA256: fields["raw_sha256"].(string), Bytes: size,
	}
	buffer, err := newPracticalSourceBuffer(object.Capability, object.Path, object.Kind, data)
	if err != nil {
		return practicalSourceBuffer{}, err
	}
	if object != buffer.object {
		return practicalSourceBuffer{}, fmt.Errorf("practical source object does not match its complete identity and original bytes")
	}
	return buffer, nil
}
