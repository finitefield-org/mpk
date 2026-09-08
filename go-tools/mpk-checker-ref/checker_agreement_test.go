//go:build checkeragreement

package mpkcheckerref

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"testing"
)

func TestCheckerAgreementWithRustCLI(t *testing.T) {
	fixtures := checkerAgreementFixtures(t)
	if len(fixtures) == 0 {
		t.Fatal("checker agreement corpus is empty")
	}

	root := absoluteRepoRoot(t)
	for _, fixture := range fixtures {
		fixture := fixture
		t.Run(relativeFixtureName(t, root, fixture), func(t *testing.T) {
			candidate := readHexFile(t, fixture)
			goReport, goErr := VerifyCertificateBytes(candidate)
			if goErr != nil {
				if e, ok := goErr.(*VerifyError); !ok || e.Kind == VerifyInternalInvariant {
					t.Fatalf("Go checker execution failed: %v", goErr)
				}
			}
			goAccepted := goErr == nil
			rustReport, rustOutput := rustCLIChecks(t, root, candidate)
			rustAccepted := rustReport.Verdict == "accepted"
			if goAccepted != rustAccepted {
				t.Fatalf(
					"checker disagreement: Go accepted=%v error=%v; Rust accepted=%v output=%s",
					goAccepted,
					goErr,
					rustAccepted,
					rustOutput,
				)
			}
			if goAccepted {
				if err := rustReport.agrees(goReport); err != nil {
					t.Fatal(err)
				}
			}
		})
	}
}

func checkerAgreementFixtures(t *testing.T) []string {
	t.Helper()
	root := absoluteRepoRoot(t)
	patterns := []string{
		"fixtures/cert-basic/*.hex",
		"fixtures/cert-decode/invalid/*.hex",
		"fixtures/cert-canonical/non-canonical/*.hex",
		"fixtures/program-certificate/*.hex",
		"proofs/std/logic/*.hex",
		"proofs/std/eq/*.hex",
		"proofs/std/bool/*.hex",
		"proofs/std/nat/*.hex",
		"proofs/std/int/*.hex",
		"proofs/std/bitvec/*.hex",
		"proofs/std/array/*.hex",
		"proofs/go/base/*.hex",
		"proofs/program/base/*.hex",
	}

	var fixtures []string
	for _, pattern := range patterns {
		matches, err := filepath.Glob(filepath.Join(root, pattern))
		if err != nil {
			t.Fatalf("glob %s: %v", pattern, err)
		}
		fixtures = append(fixtures, matches...)
	}
	sort.Strings(fixtures)
	return fixtures
}

// A process failure is not a proof rejection. In particular, a Cargo build
// failure must never make a negative fixture appear to agree with Go.
type rustAgreementReport struct {
	Verdict          string  `json:"verdict"`
	Module           *string `json:"module"`
	DeclarationCount *int    `json:"declaration_count"`
	AxiomCount       *uint64 `json:"axiom_count"`
	Hashes           struct {
		Export      *string `json:"export"`
		AxiomReport *string `json:"axiom_report"`
		Certificate string  `json:"certificate"`
	} `json:"hashes"`
	ErrorCode *string `json:"error_code"`
}

func parseRustAgreementReport(output []byte, status int, candidate []byte) (rustAgreementReport, error) {
	var r rustAgreementReport
	if err := json.Unmarshal(output, &r); err != nil {
		return r, fmt.Errorf("Rust checker did not return a verifier report: %w", err)
	}
	if r.Hashes.Certificate != HashHex(CertificateHash(candidate)) {
		return r, fmt.Errorf("Rust checker report is not bound to the submitted bytes")
	}
	switch r.Verdict {
	case "accepted":
		if status != 0 || r.Module == nil || r.DeclarationCount == nil || r.AxiomCount == nil || r.Hashes.Export == nil || r.Hashes.AxiomReport == nil || r.ErrorCode != nil {
			return r, fmt.Errorf("invalid accepted Rust checker report or exit status")
		}
	case "rejected":
		if status != 1 || r.ErrorCode == nil || !rustProofRejection(*r.ErrorCode) || r.Module != nil || r.DeclarationCount != nil || r.AxiomCount != nil || r.Hashes.Export != nil || r.Hashes.AxiomReport != nil {
			return r, fmt.Errorf("Rust checker execution failed or rejection report is invalid")
		}
	default:
		return r, fmt.Errorf("missing Rust checker verdict")
	}
	return r, nil
}

func rustProofRejection(code string) bool {
	switch code {
	case "KERNEL_CANONICAL_CERTIFICATE", "KERNEL_UNSUPPORTED_FEATURE",
		"KERNEL_EXPORT_BLOCK_MISMATCH", "KERNEL_AXIOM_REPORT_MISMATCH",
		"KERNEL_HASH_MISMATCH", "KERNEL_MISSING_NAME", "KERNEL_MISSING_GLOBAL",
		"KERNEL_OUT_OF_ORDER_DECLARATION_DEPENDENCY", "KERNEL_CORE_CHECK":
		return true
	default:
		return false
	}
}

func (r rustAgreementReport) agrees(g VerifyReport) error {
	if r.Verdict != "accepted" || r.Module == nil || r.DeclarationCount == nil || r.AxiomCount == nil || r.Hashes.Export == nil || r.Hashes.AxiomReport == nil {
		return fmt.Errorf("missing accepted Rust checker report")
	}
	if *r.Module != g.Module || *r.DeclarationCount != g.DeclarationCount || *r.AxiomCount != g.AxiomCount || *r.Hashes.Export != HashHex(g.ExportHash) || *r.Hashes.AxiomReport != HashHex(g.AxiomReportHash) || r.Hashes.Certificate != HashHex(g.CertificateHash) {
		return fmt.Errorf("accepted checker reports disagree on module, declarations, axiom inventory or hashes")
	}
	return nil
}

func rustCLIChecks(t *testing.T, root string, candidate []byte) (rustAgreementReport, string) {
	t.Helper()
	// Read the original once, then send exactly the same decoded bytes to Rust.
	path := filepath.Join(t.TempDir(), "candidate.mpcert")
	if err := os.WriteFile(path, candidate, 0600); err != nil {
		t.Fatal(err)
	}
	cmd := exec.Command("cargo", "run", "--quiet", "-p", "mpk-cli", "--", "check", path)
	cmd.Dir = root
	output, err := cmd.CombinedOutput()
	status := 0
	if err != nil {
		if e, ok := err.(*exec.ExitError); ok {
			status = e.ExitCode()
		} else {
			t.Fatalf("run Rust checker: %v\n%s", err, output)
		}
	}
	r, err := parseRustAgreementReport(output, status, candidate)
	if err != nil {
		t.Fatalf("Rust checker execution/protocol failure: %v\n%s", err, output)
	}
	return r, string(output)
}

func absoluteRepoRoot(t *testing.T) string {
	t.Helper()
	root, err := filepath.Abs(repoRoot())
	if err != nil {
		t.Fatalf("repo root abs path: %v", err)
	}
	return root
}

func relativeFixtureName(t *testing.T, root string, fixture string) string {
	t.Helper()
	relative, err := filepath.Rel(root, fixture)
	if err != nil {
		t.Fatalf("fixture relative path: %v", err)
	}
	return relative
}

func TestCheckerAgreementWithRustCLIReportProtocol(t *testing.T) {
	candidate := []byte("malformed certificate")
	hash := HashHex(CertificateHash(candidate))
	rejected := fmt.Sprintf(`{"verdict":"rejected","hashes":{"certificate":%q},"error_code":"KERNEL_CANONICAL_CERTIFICATE"}`, hash)
	if _, err := parseRustAgreementReport([]byte(rejected), 1, candidate); err != nil {
		t.Fatal(err)
	}
	for _, row := range []struct {
		name   string
		output string
		status int
		bytes  []byte
	}{
		{"cargo_failure", "error: could not compile mpk-cli", 101, candidate},
		{"signal", rejected, -1, candidate},
		{"rejection_with_success_exit", rejected, 0, candidate},
		{"wrong_bytes", rejected, 1, []byte("different")},
		{"missing_error", fmt.Sprintf(`{"verdict":"rejected","hashes":{"certificate":%q}}`, hash), 1, candidate},
		{"internal_error", fmt.Sprintf(`{"verdict":"rejected","hashes":{"certificate":%q},"error_code":"KERNEL_INTERNAL_INVARIANT"}`, hash), 1, candidate},
		{"unknown_error", fmt.Sprintf(`{"verdict":"rejected","hashes":{"certificate":%q},"error_code":"CARGO_FAILED"}`, hash), 1, candidate},
		{"missing_acceptance_fields", fmt.Sprintf(`{"verdict":"accepted","hashes":{"certificate":%q}}`, hash), 0, candidate},
		{"trailing_output", rejected + "\nnot JSON", 1, candidate},
	} {
		t.Run(row.name, func(t *testing.T) {
			if _, err := parseRustAgreementReport([]byte(row.output), row.status, row.bytes); err == nil {
				t.Fatal("execution/protocol failure was treated as a verifier verdict")
			}
		})
	}
	g := VerifyReport{Module: "Example", DeclarationCount: 0, AxiomCount: 0, CertificateHash: CertificateHash(candidate)}
	accepted := fmt.Sprintf(`{"verdict":"accepted","module":"Example","declaration_count":0,"axiom_count":0,"hashes":{"certificate":%q,"export":%q,"axiom_report":%q}}`, hash, HashHex(g.ExportHash), HashHex(g.AxiomReportHash))
	r, err := parseRustAgreementReport([]byte(accepted), 0, candidate)
	if err != nil {
		t.Fatal(err)
	}
	if err := r.agrees(g); err != nil {
		t.Fatal(err)
	}
	for _, mutate := range []func(*VerifyReport){
		func(g *VerifyReport) { g.Module = "Other" },
		func(g *VerifyReport) { g.DeclarationCount++ },
		func(g *VerifyReport) { g.AxiomCount++ },
		func(g *VerifyReport) { g.ExportHash[0]++ },
		func(g *VerifyReport) { g.AxiomReportHash[0]++ },
		func(g *VerifyReport) { g.CertificateHash[0]++ },
	} {
		changed := g
		mutate(&changed)
		if err := r.agrees(changed); err == nil {
			t.Fatal("accepted report mismatch was ignored")
		}
	}
}

// These are regenerated from original validated practical VIR by the Rust
// carrier tests. Agreement on rejection is not sufficient for this corpus.
func TestCheckerAgreementWithRustCLICSharpCarriers(t *testing.T) {
	checkOrdinaryCertificates(t, "carriers", 24)
}

func TestCheckerAgreementWithRustCLIIntegerCircuits(t *testing.T) {
	checkOrdinaryCertificates(t, "integer-circuits", 7)
}

func TestCheckerAgreementWithRustCLITemporalCircuits(t *testing.T) {
	checkOrdinaryCertificates(t, "temporal-circuits", 4)
}

func TestCheckerAgreementWithRustCLICalendarCircuits(t *testing.T) {
	checkOrdinaryCertificates(t, "calendar-circuits", 5)
}

func TestCheckerAgreementWithRustCLIFloatingCircuits(t *testing.T) {
	checkOrdinaryCertificates(t, "floating-circuits", 8)
}

func TestCheckerAgreementWithRustCLIFloatingConversions(t *testing.T) {
	checkOrdinaryCertificates(t, "conversion-circuits", 6)
}

func TestCheckerAgreementWithRustCLIDecimalCircuits(t *testing.T) {
	checkOrdinaryCertificates(t, "decimal-circuits", 7)
}

func TestCheckerAgreementWithRustCLIDecimalArithmetic(t *testing.T) {
	checkOrdinaryCertificates(t, "decimal-arithmetic-circuits", 8)
}

func TestCheckerAgreementWithRustCLIStringBasics(t *testing.T) {
	checkOrdinaryCertificates(t, "string-basic-circuits", 9)
}

func TestCheckerAgreementWithRustCLIStringConstruction(t *testing.T) {
	checkOrdinaryCertificates(t, "string-construction-circuits", 8)
}

func TestCheckerAgreementWithRustCLIStringOrdinal(t *testing.T) {
	checkOrdinaryCertificates(t, "string-ordinal-circuits", 10)
}

func TestCheckerAgreementWithRustCLIBoolean(t *testing.T) {
	checkOrdinaryCertificates(t, "boolean-circuits", 1)
}

func TestCheckerAgreementWithRustCLIOrderedFolds(t *testing.T) {
	checkOrdinaryCertificates(t, "ordered-folds", 6)
}

func TestCheckerAgreementWithRustCLIScalarDomainRanges(t *testing.T) {
	checkOrdinaryCertificates(t, "scalar-domain-ranges", 2)
}

func TestCheckerAgreementWithRustCLIScalarDomains(t *testing.T) {
	checkOrdinaryCertificates(t, "scalar-domains", 25)
}

func TestCheckerAgreementWithRustCLIStructuralStorage(t *testing.T) {
	checkOrdinaryCertificates(t, "structural-storage", 26)
}

func checkOrdinaryCertificates(t *testing.T, directory string, expectedCount int) {
	t.Helper()
	root := absoluteRepoRoot(t)
	fixtures, err := filepath.Glob(filepath.Join(root, "develop/migrations/csharp-03/ordinary-foundation", directory, "*.hex"))
	if err != nil || len(fixtures) != expectedCount {
		t.Fatalf("incomplete ordinary carrier corpus: count=%d error=%v", len(fixtures), err)
	}
	for _, path := range fixtures {
		t.Run(filepath.Base(path), func(t *testing.T) {
			candidate := readHexFile(t, path)
			g, err := VerifyCertificateBytes(candidate)
			if err != nil || g.AxiomCount != 0 {
				t.Fatalf("carrier must be accepted with zero axioms: report=%+v error=%v", g, err)
			}
			r, _ := rustCLIChecks(t, root, candidate)
			if err := r.agrees(g); err != nil {
				t.Fatal(err)
			}
			// Actual hash-corrupted bytes must be rejected by both processes.
			candidate[len(candidate)-1] ^= 1
			_, goErr := VerifyCertificateBytes(candidate)
			if e, ok := goErr.(*VerifyError); !ok || e.Kind == VerifyInternalInvariant {
				t.Fatalf("Go must reject the changed hash with a proof error: %v", goErr)
			}
			r, _ = rustCLIChecks(t, root, candidate)
			if r.Verdict != "rejected" {
				t.Fatal("Rust accepted a changed certificate hash")
			}
		})
	}
}
