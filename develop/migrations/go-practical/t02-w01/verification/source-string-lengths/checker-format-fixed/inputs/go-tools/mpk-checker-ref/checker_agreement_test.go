//go:build checkeragreement

package mpkcheckerref

import (
	"bytes"
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
	ErrorCode   *string `json:"error_code"`
	ErrorDetail *string `json:"error_detail"`
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
	r, output, err := runRustAgreementCommand(cmd, candidate)
	if err != nil {
		t.Fatalf("Rust checker execution/protocol failure: %v\n%s", err, output)
	}
	return r, output
}

func runRustAgreementCommand(cmd *exec.Cmd, candidate []byte) (rustAgreementReport, string, error) {
	// Cargo diagnostics are stderr, while the verifier protocol is stdout.
	// Do not filter or search for JSON in a combined stream: malformed stdout,
	// a build failure, and a report printed only to stderr must still fail closed.
	var diagnostics bytes.Buffer
	cmd.Stderr = &diagnostics
	output, err := cmd.Output()
	detail := fmt.Sprintf("stdout:\n%s\nstderr:\n%s", output, diagnostics.Bytes())
	status := 0
	if err != nil {
		if e, ok := err.(*exec.ExitError); ok {
			status = e.ExitCode()
		} else {
			return rustAgreementReport{}, detail, fmt.Errorf("run Rust checker: %w", err)
		}
	}
	r, err := parseRustAgreementReport(output, status, candidate)
	return r, detail, err
}

func TestRustAgreementProcessHelper(t *testing.T) {
	mode := os.Getenv("MPK_CHECKER_AGREEMENT_PROCESS_CASE")
	if mode == "" {
		return
	}
	candidate := []byte("malformed certificate")
	hash := HashHex(CertificateHash(candidate))
	report := fmt.Sprintf(`{"verdict":"rejected","hashes":{"certificate":%q},"error_code":"KERNEL_CANONICAL_CERTIFICATE"}`, hash)
	status := 1
	switch mode {
	case "warning_accept":
		report = fmt.Sprintf(`{"verdict":"accepted","module":"Example","declaration_count":0,"axiom_count":0,"hashes":{"certificate":%q,"export":"example-export","axiom_report":"example-axioms"}}`, hash)
		fmt.Fprintln(os.Stderr, "warning: compiler diagnostic, not a verifier report")
		fmt.Fprintln(os.Stdout, report)
		status = 0
	case "warning":
		fmt.Fprintln(os.Stderr, "warning: compiler diagnostic, not a verifier report")
		fmt.Fprintln(os.Stdout, report)
	case "stderr_report":
		fmt.Fprintln(os.Stderr, report)
	case "stdout_warning":
		fmt.Fprintln(os.Stdout, "warning: protocol contamination")
		fmt.Fprintln(os.Stdout, report)
	case "cargo_failure":
		fmt.Fprintln(os.Stderr, "error: could not compile mpk-cli")
		fmt.Fprintln(os.Stdout, report)
		status = 101
	default:
		os.Exit(99)
	}
	os.Exit(status)
}

func TestCheckerAgreementRustProcessStreams(t *testing.T) {
	for _, mode := range []string{"warning", "warning_accept", "stderr_report", "stdout_warning", "cargo_failure"} {
		t.Run(mode, func(t *testing.T) {
			cmd := exec.Command(os.Args[0], "-test.run=^TestRustAgreementProcessHelper$")
			cmd.Env = append(os.Environ(), "MPK_CHECKER_AGREEMENT_PROCESS_CASE="+mode)
			r, detail, err := runRustAgreementCommand(cmd, []byte("malformed certificate"))
			if mode == "warning" || mode == "warning_accept" {
				verdict := "rejected"
				if mode == "warning_accept" {
					verdict = "accepted"
				}
				if err != nil || r.Verdict != verdict {
					t.Fatalf("stderr warning corrupted stdout report: %v\n%s", err, detail)
				}
			} else if err == nil {
				t.Fatalf("process/protocol failure became a verifier verdict: %s", detail)
			}
		})
	}
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
