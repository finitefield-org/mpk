package mpkcheckerref

import (
	"encoding/hex"
	"errors"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"testing"
)

func TestDecodePositiveFixtures(t *testing.T) {
	fixtures := []struct {
		path   string
		module string
	}{
		{"fixtures/cert-encoding/minimal-empty.hex", "Example.Empty"},
		{"fixtures/cert-basic/zero-axiom.hex", "Example.Basic.ZeroAxiom"},
		{"fixtures/cert-basic/one-theorem.hex", "Example.Basic.OneTheorem"},
	}

	for _, fixture := range fixtures {
		fixture := fixture
		t.Run(filepath.Base(fixture.path), func(t *testing.T) {
			cert, err := DecodeCertificate(readHexFixture(t, fixture.path))
			if err != nil {
				t.Fatalf("DecodeCertificate() error = %v", err)
			}
			if cert.Module != fixture.module {
				t.Fatalf("module = %q, want %q", cert.Module, fixture.module)
			}
		})
	}
}

func TestDecodeInvalidFixtures(t *testing.T) {
	paths := globFixtures(t, "fixtures/cert-decode/invalid/*.hex")
	if len(paths) == 0 {
		t.Fatal("invalid fixture set is empty")
	}

	for _, path := range paths {
		path := path
		t.Run(filepath.Base(path), func(t *testing.T) {
			_, err := DecodeCertificate(readHexFile(t, path))
			if err == nil {
				t.Fatal("DecodeCertificate() succeeded, want error")
			}
			var decodeError *DecodeError
			if !errors.As(err, &decodeError) {
				t.Fatalf("error type = %T, want *DecodeError", err)
			}
		})
	}
}

func TestDecodeMinimalFixtureShape(t *testing.T) {
	cert, err := DecodeCertificate(readHexFixture(t, "fixtures/cert-encoding/minimal-empty.hex"))
	if err != nil {
		t.Fatalf("DecodeCertificate() error = %v", err)
	}

	if len(cert.Imports) != 0 ||
		len(cert.NameTable) != 0 ||
		len(cert.LevelTable) != 0 ||
		len(cert.TermTable) != 0 ||
		len(cert.ProofNodeTable) != 0 ||
		len(cert.Declarations) != 0 ||
		len(cert.TheoryCertificates) != 0 ||
		len(cert.ExportBlock) != 0 ||
		len(cert.AxiomReport.Entries) != 0 ||
		len(cert.AxiomReport.DeclarationDependencies) != 0 ||
		cert.SourceManifest != nil {
		t.Fatalf("minimal fixture decoded with non-empty sections: %#v", cert)
	}
}

func TestDecodeOneTheoremFixtureShape(t *testing.T) {
	cert, err := DecodeCertificate(readHexFixture(t, "fixtures/cert-basic/one-theorem.hex"))
	if err != nil {
		t.Fatalf("DecodeCertificate() error = %v", err)
	}

	if got, want := len(cert.LevelTable), 2; got != want {
		t.Fatalf("level count = %d, want %d", got, want)
	}
	if got, want := len(cert.TermTable), 2; got != want {
		t.Fatalf("term count = %d, want %d", got, want)
	}
	if got, want := len(cert.Declarations), 1; got != want {
		t.Fatalf("declaration count = %d, want %d", got, want)
	}
	declaration := cert.Declarations[0]
	if declaration.Tag != DeclTheorem {
		t.Fatalf("declaration tag = %d, want theorem", declaration.Tag)
	}
	if declaration.Type != 1 || declaration.Proof != 0 {
		t.Fatalf("theorem type/proof = %d/%d, want 1/0", declaration.Type, declaration.Proof)
	}
}

func readHexFixture(t *testing.T, repoRelativePath string) []byte {
	t.Helper()
	return readHexFile(t, filepath.Join(repoRoot(), repoRelativePath))
}

func readHexFile(t *testing.T, path string) []byte {
	t.Helper()
	contents, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("read fixture: %v", err)
	}
	rawHex := strings.Map(func(r rune) rune {
		if r == ' ' || r == '\n' || r == '\r' || r == '\t' {
			return -1
		}
		return r
	}, string(contents))
	data, err := hex.DecodeString(rawHex)
	if err != nil {
		t.Fatalf("decode fixture hex: %v", err)
	}
	return data
}

func globFixtures(t *testing.T, pattern string) []string {
	t.Helper()
	paths, err := filepath.Glob(filepath.Join(repoRoot(), pattern))
	if err != nil {
		t.Fatalf("glob fixtures: %v", err)
	}
	sort.Strings(paths)
	return paths
}

func repoRoot() string {
	return filepath.Clean(filepath.Join("..", ".."))
}
