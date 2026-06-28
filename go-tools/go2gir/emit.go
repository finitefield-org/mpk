package main

import (
	"bytes"
	"crypto/sha256"
	"encoding/base64"
	"encoding/binary"
	"encoding/hex"
	"encoding/json"
)

const (
	girEmitSchema  = "mpk.gir.emit.v0"
	girBinaryMagic = "MPK_GIR_V0\x00"
)

type girEmission struct {
	Schema        string `json:"schema"`
	GIRHash       string `json:"gir_hash"`
	CanonicalJSON string `json:"canonical_json"`
	BinaryBase64  string `json:"binary_base64"`
}

// girCanonicalModule is the GIR hash input and intentionally excludes GIRHash.
type girCanonicalModule struct {
	SchemaVersion string       `json:"schema_version"`
	Packages      []girPackage `json:"packages"`
}

func emitCanonicalGIR(module girModule) (girModule, girEmission, error) {
	canonicalJSON, err := canonicalGIRJSON(module)
	if err != nil {
		return girModule{}, girEmission{}, err
	}
	canonicalBinary := canonicalGIRBinary(canonicalJSON)
	girHash := hashGIRBinary(canonicalBinary)

	module.GIRHash = girHash
	return module, girEmission{
		Schema:        girEmitSchema,
		GIRHash:       girHash,
		CanonicalJSON: string(canonicalJSON),
		BinaryBase64:  base64.StdEncoding.EncodeToString(canonicalBinary),
	}, nil
}

func canonicalGIRJSON(module girModule) ([]byte, error) {
	payload := girCanonicalModule{
		SchemaVersion: module.SchemaVersion,
		Packages:      module.Packages,
	}
	var buffer bytes.Buffer
	encoder := json.NewEncoder(&buffer)
	encoder.SetEscapeHTML(false)
	if err := encoder.Encode(payload); err != nil {
		return nil, err
	}
	return bytes.TrimSuffix(buffer.Bytes(), []byte("\n")), nil
}

func canonicalGIRBinary(canonicalJSON []byte) []byte {
	payload := make([]byte, 0, len(girBinaryMagic)+8+len(canonicalJSON))
	payload = append(payload, []byte(girBinaryMagic)...)
	var length [8]byte
	binary.BigEndian.PutUint64(length[:], uint64(len(canonicalJSON)))
	payload = append(payload, length[:]...)
	payload = append(payload, canonicalJSON...)
	return payload
}

func hashGIRBinary(binaryPayload []byte) string {
	sum := sha256.Sum256(binaryPayload)
	return hex.EncodeToString(sum[:])
}
