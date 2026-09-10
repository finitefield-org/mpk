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

func TestCheckerAgreementWithRustCLIStructuralRelations(t *testing.T) {
	checkOrdinaryCertificates(t, "relations", 22)
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

func TestCheckerAgreementWithRustCLIAggregateFolds(t *testing.T) {
	checkOrdinaryCertificates(t, "aggregate-folds", 1)
}
func TestCheckerAgreementWithRustCLIRecursiveDomains(t *testing.T) {
	checkOrdinaryCertificates(t, "domains", 28)
}
func TestCheckerAgreementWithRustCLIDefaults(t *testing.T) {
	checkOrdinaryCertificates(t, "defaults", 29)
}

func TestCheckerAgreementWithRustCLIFiniteOperations(t *testing.T) {
	checkOrdinaryCertificates(t, "finite-operations", 8)
}

func TestCheckerAgreementWithRustCLIContractCarrierExtension(t *testing.T) {
	checkOrdinaryCertificates(t, "contract-carrier-extension/checker-cases", 22)
}

func TestCheckerAgreementWithRustCLISequenceOperations(t *testing.T) {
	checkOrdinaryCertificates(t, "sequence-operations", 12)
}

func TestCheckerAgreementWithRustCLIConstructionOperations(t *testing.T) {
	checkOrdinaryCertificates(t, "construction-operations", 5)
}

func TestCheckerAgreementWithRustCLIOutcomeOperations(t *testing.T) {
	checkOrdinaryCertificates(t, "outcome-operations", 16)
}

func TestCheckerAgreementWithRustCLIEntryOperations(t *testing.T) {
	checkOrdinaryCertificates(t, "entry-operations", 6)
}

func TestCheckerAgreementWithRustCLICollectionOperations(t *testing.T) {
	checkOrdinaryCertificates(t, "collection-operations", 9)
}

func TestCheckerAgreementWithRustCLIStructuralFoundations(t *testing.T) {
	checkOrdinaryCertificates(t, "structural-foundations", 44)
}

func TestCheckerAgreementWithRustCLIMoneyOperations(t *testing.T) {
	checkOrdinaryCertificates(t, "money-operations", 2)
}

func TestCheckerAgreementWithRustCLISourceObservations(t *testing.T) {
	checkOrdinaryCertificates(t, "source-observations", 4)
}

func TestCheckerAgreementWithRustCLILiteralDefinitions(t *testing.T) {
	checkOrdinaryCertificates(t, "literal-definitions", 64)
}

func TestCheckerAgreementWithRustCLILiteralEdges(t *testing.T) {
	checkOrdinaryCertificates(t, "literal-edges", 2)
}

func TestCheckerAgreementWithRustCLIBoundaryLiterals(t *testing.T) {
	checkOrdinaryCertificates(t, "boundary-literals", 9)
}

func TestCheckerAgreementWithRustCLIBindingProjections(t *testing.T) {
	checkOrdinaryCertificates(t, "binding-projections", 44)
}

func TestCheckerAgreementWithRustCLIBindingOrders(t *testing.T) {
	checkOrdinaryCertificates(t, "binding-orders", 9)
}

func TestCheckerAgreementWithRustCLIBindingRebuilds(t *testing.T) {
	checkOrdinaryCertificates(t, "binding-rebuilds", 45)
}

func TestCheckerAgreementWithRustCLIHexCodecs(t *testing.T) {
	checkOrdinaryCertificates(t, "hex-codecs", 11)
}

func TestCheckerAgreementWithRustCLIIntegerFormats(t *testing.T) {
	checkOrdinaryCertificates(t, "integer-formats", 54)
}

func TestCheckerAgreementWithRustCLIIntegerParsers(t *testing.T) {
	checkOrdinaryCertificates(t, "integer-parsers", 54)
}

func TestCheckerAgreementWithRustCLIDecimalFormats(t *testing.T) {
	checkOrdinaryCertificates(t, "decimal-formats", 6)
}

func TestCheckerAgreementWithRustCLIDecimalFixedFormats(t *testing.T) {
	checkOrdinaryCertificates(t, "decimal-fixed-formats", 6)
}

func TestCheckerAgreementWithRustCLIDecimalParsers(t *testing.T) {
	checkOrdinaryCertificates(t, "decimal-parsers", 6)
}

func TestCheckerAgreementWithRustCLICalendarCodecs(t *testing.T) {
	checkOrdinaryCertificates(t, "calendar-codecs", 3)
}

func TestCheckerAgreementWithRustCLIJsonStringParsers(t *testing.T) {
	checkOrdinaryCertificates(t, "json-string-parsers", 3)
}

func TestCheckerAgreementWithRustCLIJsonStrings(t *testing.T) {
	checkOrdinaryCertificates(t, "json-strings", 3)
}

func TestCheckerAgreementWithRustCLIBoundaryUtf8(t *testing.T) {
	checkOrdinaryCertificates(t, "boundary-utf8", 3)
}

func TestCheckerAgreementWithRustCLIBoundaryFragments(t *testing.T) {
	checkOrdinaryCertificates(t, "boundary-fragments", 3)
}

func TestCheckerAgreementWithRustCLIBoundaryDocuments(t *testing.T) {
	checkOrdinaryCertificates(t, "boundary-documents", 3)
}

func TestCheckerAgreementWithRustCLIBindingGuards(t *testing.T) {
	checkOrdinaryCertificates(t, "binding-guards", 45)
}

func TestCheckerAgreementWithRustCLIBindingRelations(t *testing.T) {
	checkOrdinaryCertificates(t, "binding-relations", 45)
}

func TestCheckerAgreementWithRustCLIJsonKeywords(t *testing.T) {
	checkOrdinaryCertificates(t, "json-keywords", 3)
}

func TestCheckerAgreementWithRustCLIJsonTokens(t *testing.T) {
	checkOrdinaryCertificates(t, "json-tokens", 3)
}

func TestCheckerAgreementWithRustCLIJsonSyntax(t *testing.T) {
	checkOrdinaryCertificates(t, "json-syntax", 3)
}

func TestCheckerAgreementWithRustCLIJsonValues(t *testing.T) {
	checkOrdinaryCertificates(t, "json-values", 7)
}

func TestCheckerAgreementWithRustCLIJsonStringCells(t *testing.T) {
	checkOrdinaryCertificates(t, "json-string-cells", 1)
}

func TestCheckerAgreementWithRustCLIJsonValuesAllScalars(t *testing.T) {
	checkOrdinaryCertificates(t, "json-values-all-scalars", 1)
}

func TestCheckerAgreementWithRustCLIJsonProducts(t *testing.T) {
	checkOrdinaryCertificates(t, "json-products", 7)
}

func TestCheckerAgreementWithRustCLIJsonProductNested(t *testing.T) {
	checkOrdinaryCertificates(t, "json-product-nested", 2)
}

func TestCheckerAgreementWithRustCLIJsonEnums(t *testing.T) {
	checkOrdinaryCertificates(t, "json-enums", 1)
}

func TestCheckerAgreementWithRustCLIJsonBuiltins(t *testing.T) {
	checkOrdinaryCertificates(t, "json-builtins", 1)
}

func TestCheckerAgreementWithRustCLIJsonSequences(t *testing.T) {
	checkOrdinaryCertificates(t, "json-sequences", 1)
}

func TestCheckerAgreementWithRustCLIJsonOrdered(t *testing.T) {
	checkOrdinaryCertificates(t, "json-ordered", 4)
}

func TestCheckerAgreementWithRustCLIJsonTransition(t *testing.T) {
	checkOrdinaryCertificates(t, "json-transition", 2)
}

func TestCheckerAgreementWithRustCLIBoundaryRules(t *testing.T) {
	checkOrdinaryCertificates(t, "boundary-rules", 15)
}

func TestCheckerAgreementWithRustCLIJsonSums(t *testing.T) {
	checkOrdinaryCertificates(t, "json-sums", 1)
}

func TestCheckerAgreementWithRustCLIJsonSemanticProducts(t *testing.T) {
	checkOrdinaryCertificates(t, "json-semantic-products", 1)
}

func TestCheckerAgreementWithRustCLIJsonCalendar(t *testing.T) {
	checkOrdinaryCertificates(t, "json-calendar", 8)
}

func TestCheckerAgreementWithRustCLIJsonCalendarCollection(t *testing.T) {
	checkOrdinaryCertificates(t, "json-calendar-collection", 1)
}

func TestCheckerAgreementWithRustCLIJsonCollectionShared(t *testing.T) {
	checkOrdinaryCertificates(t, "json-collection-shared", 1)
}

func TestCheckerAgreementWithRustCLIDecimalJsonCollectionShared(t *testing.T) {
	checkOrdinaryCertificates(t, "decimal-json-collection-shared", 1)
}

func TestCheckerAgreementWithRustCLIStructuralBoundary(t *testing.T) {
	checkOrdinaryCertificates(t, "structural-boundary", 4)
}

func TestCheckerAgreementWithRustCLIJsonBoundaryFields(t *testing.T) {
	checkOrdinaryCertificates(t, "json-boundary-fields", 15)
}

func TestCheckerAgreementWithRustCLIJsonEnvelopes(t *testing.T) {
	checkOrdinaryCertificates(t, "json-envelopes", 16)
}

func TestCheckerAgreementWithRustCLIJsonTypedDepth(t *testing.T) {
	checkOrdinaryCertificates(t, "json-typed-depth", 16)
}

func TestCheckerAgreementWithRustCLIJsonDepthGuardedEnvelopes(t *testing.T) {
	checkOrdinaryCertificates(t, "json-depth-guarded-envelopes", 17)
}

func TestCheckerAgreementWithRustCLIJsonDepthCompound(t *testing.T) {
	checkOrdinaryCertificates(t, "json-depth-compound", 11)
}

func TestCheckerAgreementWithRustCLIJsonSemanticRoot(t *testing.T) {
	checkOrdinaryCertificates(t, "json-semantic-root", 5)
}

func TestCheckerAgreementWithRustCLIJsonTypedNodes(t *testing.T) {
	checkOrdinaryCertificates(t, "json-typed-nodes", 32)
}

func TestCheckerAgreementWithRustCLIJsonTypedGuardedEnvelopes(t *testing.T) {
	checkOrdinaryCertificates(t, "json-typed-guarded-envelopes", 33)
}

func TestCheckerAgreementWithRustCLIJsonRawLimits(t *testing.T) {
	checkOrdinaryCertificates(t, "json-raw-limits", 5)
}

func TestCheckerAgreementWithRustCLIJsonLimitsGuardedEnvelopes(t *testing.T) {
	checkOrdinaryCertificates(t, "json-limits-guarded-envelopes", 21)
}

func TestCheckerAgreementWithRustCLISourceClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "source-clauses", 8)
}

func TestCheckerAgreementWithRustCLIPublicDomains(t *testing.T) {
	checkOrdinaryCertificates(t, "public-domains", 7)
}

func TestCheckerAgreementWithRustCLIPublicDefaults(t *testing.T) {
	checkOrdinaryCertificates(t, "public-defaults", 6)
}

func TestCheckerAgreementWithRustCLIStructuralPublic(t *testing.T) {
	checkOrdinaryCertificates(t, "structural-public", 3)
}

func TestCheckerAgreementWithRustCLIConditionalClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "conditional-clauses", 3)
}

func TestCheckerAgreementWithRustCLIStructuralClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "structural-clauses", 2)
}

func TestCheckerAgreementWithRustCLILiteralClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "literal-clauses", 2)
}

func TestCheckerAgreementWithRustCLITotalClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "total-clauses", 2)
}

func TestCheckerAgreementWithRustCLIDefinednessClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "definedness-clauses", 1)
}

func TestCheckerAgreementWithRustCLIPartialReadClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "partial-read-clauses", 1)
}

func TestCheckerAgreementWithRustCLIContractExpressions(t *testing.T) {
	checkOrdinaryCertificates(t, "contract-expressions", 9)
}

func TestCheckerAgreementWithRustCLIExceptionClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "exception-clauses", 2)
}

func TestCheckerAgreementWithRustCLIQuantifiers(t *testing.T) {
	checkOrdinaryCertificates(t, "quantifiers", 6)
}

func TestCheckerAgreementWithRustCLITaggedMake(t *testing.T) {
	checkOrdinaryCertificates(t, "tagged-make", 1)
}

func TestCheckerAgreementWithRustCLITransitionClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "transition-clauses", 2)
}

func TestCheckerAgreementWithRustCLICollectionClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "collection-clauses", 9)
}

func TestCheckerAgreementWithRustCLIBindingClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "binding-clauses", 15)
}

func TestCheckerAgreementWithRustCLIIntegerCodecClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "integer-codec-clauses", 10)
}

func TestCheckerAgreementWithRustCLIFixedCodecClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "fixed-codec-clauses", 7)
}

func TestCheckerAgreementWithRustCLIDecimalCodecClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "decimal-codec-clauses", 6)
}

func TestCheckerAgreementWithRustCLIFloatingClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "floating-clauses", 5)
}

func TestCheckerAgreementWithRustCLIDecimalClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "decimal-clauses", 9)
}

func TestCheckerAgreementWithRustCLICalendarClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "calendar-clauses", 8)
}

func TestCheckerAgreementWithRustCLIStringClauses(t *testing.T) {
	checkOrdinaryCertificates(t, "string-clauses", 6)
}

func TestCheckerAgreementWithRustCLIIntegerData(t *testing.T) {
	checkOrdinaryCertificates(t, "integer-data", 3)
}

func TestCheckerAgreementWithRustCLIStructuralData(t *testing.T) {
	checkOrdinaryCertificates(t, "structural-data", 5)
}

func TestCheckerAgreementWithRustCLIFloatingData(t *testing.T) {
	checkOrdinaryCertificates(t, "floating-data", 3)
}

func TestCheckerAgreementWithRustCLIDecimalData(t *testing.T) {
	checkOrdinaryCertificates(t, "decimal-data", 3)
}
