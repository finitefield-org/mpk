// Hash the exact canonical bytes emitted by the independent Go canonicalizer.
package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"os"
)

func main() {
	var rows []struct {
		ID        string `json:"id"`
		Canonical string `json:"canonical_hex"`
	}
	if err := json.NewDecoder(os.Stdin).Decode(&rows); err != nil {
		panic(err)
	}
	out := make([]map[string]string, 0, len(rows))
	for _, row := range rows {
		b, err := hex.DecodeString(row.Canonical)
		if err != nil {
			panic(err)
		}
		h := sha256.Sum256(b)
		out = append(out, map[string]string{"id": row.ID, "canonical_sha256": hex.EncodeToString(h[:])})
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
