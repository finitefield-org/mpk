//go:build checkeragreement

package mpkcheckerref

import (
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
			_, goErr := VerifyCertificateBytes(readHexFile(t, fixture))
			goAccepted := goErr == nil

			rustAccepted, rustOutput := rustCLIAccepts(t, root, fixture)
			if goAccepted != rustAccepted {
				t.Fatalf(
					"checker disagreement: Go accepted=%v error=%v; Rust accepted=%v output=%s",
					goAccepted,
					goErr,
					rustAccepted,
					rustOutput,
				)
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
		"proofs/std/logic/*.hex",
		"proofs/std/eq/*.hex",
		"proofs/std/bool/*.hex",
		"proofs/std/nat/*.hex",
		"proofs/std/int/*.hex",
		"proofs/std/bitvec/*.hex",
		"proofs/std/array/*.hex",
		"proofs/go/base/*.hex",
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

func rustCLIAccepts(t *testing.T, root string, fixture string) (bool, string) {
	t.Helper()
	cmd := exec.Command("cargo", "run", "--quiet", "-p", "mpk-cli", "--", "check", fixture)
	cmd.Dir = root
	output, err := cmd.CombinedOutput()
	if err == nil {
		return true, string(output)
	}
	if _, ok := err.(*exec.ExitError); ok {
		return false, string(output)
	}
	t.Fatalf("run Rust checker: %v\n%s", err, string(output))
	return false, ""
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
