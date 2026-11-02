package main

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
)

func sha256Hex(input []byte) string {
	digest := sha256.Sum256(input)
	return hex.EncodeToString(digest[:])
}

func hashCanonicalJSON(domain string, value jsonValue) (string, error) {
	if domain == "" {
		return "", fmt.Errorf("hash domain must not be empty")
	}
	for _, character := range []byte(domain) {
		if character < 0x21 || character > 0x7e {
			return "", fmt.Errorf("hash domain must be printable ASCII without spaces")
		}
	}
	canonical, err := canonicalJSONValue(value)
	if err != nil {
		return "", err
	}
	preimage := make([]byte, 0, len(domain)+1+len(canonical))
	preimage = append(preimage, domain...)
	preimage = append(preimage, 0)
	preimage = append(preimage, canonical...)
	return sha256Hex(preimage), nil
}

func withoutRootField(value jsonValue, field string) (jsonValue, error) {
	object, ok := value.(map[string]jsonValue)
	if !ok {
		return nil, fmt.Errorf("hash exclusion requires a root object")
	}
	if _, exists := object[field]; !exists {
		return nil, fmt.Errorf("hash exclusion field %q is missing", field)
	}
	copy := make(map[string]jsonValue, len(object)-1)
	for key, item := range object {
		if key != field {
			copy[key] = item
		}
	}
	return copy, nil
}
