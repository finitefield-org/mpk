package mpkcheckerref

import (
	"encoding/hex"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestHashVectorsMatchRust(t *testing.T) {
	contents, err := os.ReadFile(filepath.Join(repoRoot(), "fixtures/cert-hash/vectors.csv"))
	if err != nil {
		t.Fatalf("read hash vectors: %v", err)
	}

	for lineIndex, rawLine := range strings.Split(string(contents), "\n") {
		line := strings.TrimSpace(rawLine)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		fields := strings.Split(line, ",")
		if len(fields) != 3 {
			t.Fatalf("line %d: field count = %d, want 3", lineIndex+1, len(fields))
		}

		payload, err := hex.DecodeString(fields[1])
		if err != nil {
			t.Fatalf("line %d: decode payload: %v", lineIndex+1, err)
		}
		got := HashHex(HashWithDomain(HashDomain(fields[0]), payload))
		if got != fields[2] {
			t.Fatalf("line %d: hash = %s, want %s", lineIndex+1, got, fields[2])
		}
	}
}

func TestBasicFixtureHashesMatchRust(t *testing.T) {
	records := readBasicHashRecords(t)
	for _, fixture := range []string{"zero-axiom", "one-theorem"} {
		fixtureBytes := readHexFixture(t, "fixtures/cert-basic/"+fixture+".hex")
		cert, err := DecodeCertificate(fixtureBytes)
		if err != nil {
			t.Fatalf("%s: DecodeCertificate() error = %v", fixture, err)
		}

		recomputed := RecomputeHashes(cert, fixtureBytes)
		record := records[fixture]
		if record.fixture == "" {
			t.Fatalf("%s: missing hash record", fixture)
		}

		if got := HashHex(recomputed.ExportHash); got != record.exportHash {
			t.Fatalf("%s: export hash = %s, want %s", fixture, got, record.exportHash)
		}
		if recomputed.ExportHash != cert.Hashes.ExportHash {
			t.Fatalf("%s: recomputed export hash does not match embedded hash", fixture)
		}
		if got := HashHex(recomputed.AxiomReportHash); got != record.axiomReportHash {
			t.Fatalf("%s: axiom report hash = %s, want %s", fixture, got, record.axiomReportHash)
		}
		if recomputed.AxiomReportHash != cert.Hashes.AxiomReportHash {
			t.Fatalf("%s: recomputed axiom report hash does not match embedded hash", fixture)
		}
		if got := HashHex(recomputed.CertificateHash); got != record.certificateHash {
			t.Fatalf("%s: certificate hash = %s, want %s", fixture, got, record.certificateHash)
		}
	}
}

func TestHashHexRendersLowercaseHex(t *testing.T) {
	var hash HashBytes
	hash[0] = 0xab
	hash[31] = 0xef

	got := HashHex(hash)
	want := "ab000000000000000000000000000000000000000000000000000000000000ef"
	if got != want {
		t.Fatalf("HashHex() = %s, want %s", got, want)
	}
}

type basicHashRecord struct {
	fixture         string
	exportHash      string
	axiomReportHash string
	certificateHash string
}

func readBasicHashRecords(t *testing.T) map[string]basicHashRecord {
	t.Helper()
	contents, err := os.ReadFile(filepath.Join(repoRoot(), "fixtures/cert-basic/hashes.csv"))
	if err != nil {
		t.Fatalf("read basic hashes: %v", err)
	}

	records := make(map[string]basicHashRecord)
	for lineIndex, rawLine := range strings.Split(string(contents), "\n") {
		line := strings.TrimSpace(rawLine)
		if line == "" {
			continue
		}
		if lineIndex == 0 {
			if line != "fixture,export_hash,axiom_report_hash,certificate_hash" {
				t.Fatalf("unexpected basic hash csv header: %q", line)
			}
			continue
		}
		fields := strings.Split(line, ",")
		if len(fields) != 4 {
			t.Fatalf("line %d: field count = %d, want 4", lineIndex+1, len(fields))
		}
		records[fields[0]] = basicHashRecord{
			fixture:         fields[0],
			exportHash:      fields[1],
			axiomReportHash: fields[2],
			certificateHash: fields[3],
		}
	}
	return records
}
