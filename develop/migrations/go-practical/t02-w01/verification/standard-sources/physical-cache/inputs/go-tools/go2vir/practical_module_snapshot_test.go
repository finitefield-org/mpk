package main

import (
	"bytes"
	"testing"
)

func TestPracticalModuleSnapshotInfoPreservesOriginalGoMetadata(t *testing.T) {
	for _, raw := range []string{
		`{"Version":"v1.0.0","Time":"2020-01-02T03:04:05Z","Origin":{"VCS":"git","URL":"https://example.com/original","Hash":"recorded-only"},"Name":"old cache field","Extra":{"opaque":true}}`,
		`{"version":"v1.0.0","Time":null}`,
	} {
		source, err := newPracticalSourceBuffer("download", "v1.0.0.info", "metadata", []byte(raw))
		if err != nil || validatePracticalModuleInfo(source, practicalArchiveFixtureIdentity) != nil || !bytes.Equal(source.sourceBytes(), []byte(raw)) {
			t.Fatal("original Go metadata spelling or opaque provenance was rewritten")
		}
	}
}

func TestPracticalModuleSnapshotInfoRejectsAmbiguousOrInvalidMetadata(t *testing.T) {
	for name, raw := range map[string]string{
		"wrong-version": `{"Version":"v1.0.1"}`, "missing-version": `{}`,
		"duplicate": `{"Version":"v1.0.0","Version":"v1.0.1"}`,
		"time-type": `{"Version":"v1.0.0","Time":17}`, "time-value": `{"Version":"v1.0.0","Time":"invalid"}`,
		"origin-type":       `{"Version":"v1.0.0","Origin":"not a Go origin"}`,
		"origin-field-type": `{"Version":"v1.0.0","Origin":{"Hash":true}}`,
		"trailing":          `{"Version":"v1.0.0"} {}`, "not-object": `[]`,
	} {
		t.Run(name, func(t *testing.T) {
			source, err := newPracticalSourceBuffer("download", "v1.0.0.info", "metadata", []byte(raw))
			if err != nil || validatePracticalModuleInfo(source, practicalArchiveFixtureIdentity) == nil {
				t.Fatal("ambiguous or invalid cached module info accepted")
			}
		})
	}
}
