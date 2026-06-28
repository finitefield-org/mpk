package mpkcheckerref

import "testing"

func TestVerifyCertificateBytesAcceptsBasicFixtures(t *testing.T) {
	fixtures := []struct {
		path             string
		module           string
		declarationCount int
	}{
		{"fixtures/cert-basic/zero-axiom.hex", "Example.Basic.ZeroAxiom", 0},
		{"fixtures/cert-basic/one-theorem.hex", "Example.Basic.OneTheorem", 1},
	}

	for _, fixture := range fixtures {
		fixture := fixture
		t.Run(fixture.module, func(t *testing.T) {
			report, err := VerifyCertificateBytes(readHexFixture(t, fixture.path))
			if err != nil {
				t.Fatalf("VerifyCertificateBytes() error = %v", err)
			}
			if report.Module != fixture.module {
				t.Fatalf("module = %s, want %s", report.Module, fixture.module)
			}
			if report.DeclarationCount != fixture.declarationCount {
				t.Fatalf("declaration count = %d, want %d", report.DeclarationCount, fixture.declarationCount)
			}
		})
	}
}

func TestVerifyCertificateBytesRejectsCanonicalOrderMismatch(t *testing.T) {
	_, err := VerifyCertificateBytes(readHexFixture(t, "fixtures/cert-canonical/non-canonical/unsorted-name-table.hex"))
	if err == nil {
		t.Fatal("VerifyCertificateBytes() succeeded, want error")
	}
	verifyErr, ok := err.(*VerifyError)
	if !ok {
		t.Fatalf("error type = %T, want *VerifyError", err)
	}
	if verifyErr.Kind != VerifyCanonicalCertificate {
		t.Fatalf("error kind = %s, want %s", verifyErr.Kind, VerifyCanonicalCertificate)
	}
}

func TestBuildExportBlockMatchesEmbeddedBasicFixtures(t *testing.T) {
	for _, fixture := range []string{"zero-axiom", "one-theorem"} {
		fixture := fixture
		t.Run(fixture, func(t *testing.T) {
			cert, err := DecodeCertificate(readHexFixture(t, "fixtures/cert-basic/"+fixture+".hex"))
			if err != nil {
				t.Fatalf("DecodeCertificate() error = %v", err)
			}
			exportBlock, err := BuildExportBlock(cert)
			if err != nil {
				t.Fatalf("BuildExportBlock() error = %v", err)
			}
			if !exportBlocksEqual(exportBlock, cert.ExportBlock) {
				t.Fatalf("rebuilt export block does not match embedded export block")
			}
		})
	}
}
