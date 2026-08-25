package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"unicode"

	mpkcheckerref "github.com/finitefield-org/mpk/go-tools/mpk-checker-ref"
)

func main() {
	if len(os.Args) != 3 || os.Args[1] != "verify" {
		fmt.Fprintln(os.Stderr, "usage: mpk-checker-ref verify <certificate.mpcert|fixture.hex|->")
		os.Exit(2)
	}

	bytes, err := readCertificateInput(os.Args[2])
	if err != nil {
		writeRejected("input", err.Error())
		os.Exit(1)
	}

	report, err := mpkcheckerref.VerifyCertificateBytes(bytes)
	if err != nil {
		if verifyErr, ok := err.(*mpkcheckerref.VerifyError); ok {
			writeRejected(string(verifyErr.Kind), verifyErr.Detail)
		} else {
			writeRejected("verify", err.Error())
		}
		os.Exit(1)
	}

	writeJSON(verifyOutput{
		Verdict:          "accepted",
		Module:           report.Module,
		DeclarationCount: report.DeclarationCount,
		AxiomCount:       report.AxiomCount,
		Hashes: &hashOutput{
			Export:      mpkcheckerref.HashHex(report.ExportHash),
			AxiomReport: mpkcheckerref.HashHex(report.AxiomReportHash),
			Certificate: mpkcheckerref.HashHex(report.CertificateHash),
		},
	})
}

func readCertificateInput(path string) ([]byte, error) {
	return readCertificateInputFrom(path, os.Stdin)
}

func readCertificateInputFrom(path string, stdin io.Reader) ([]byte, error) {
	if path == "-" {
		bytes, err := io.ReadAll(stdin)
		if err != nil {
			return nil, fmt.Errorf("read stdin: %w", err)
		}
		return bytes, nil
	}
	bytes, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("read %s: %w", path, err)
	}
	if filepath.Ext(path) != ".hex" {
		return bytes, nil
	}

	var builder strings.Builder
	for _, r := range string(bytes) {
		if !unicode.IsSpace(r) {
			builder.WriteRune(r)
		}
	}
	decoded, err := hex.DecodeString(builder.String())
	if err != nil {
		return nil, fmt.Errorf("decode hex fixture %s: %w", path, err)
	}
	return decoded, nil
}

func writeRejected(kind string, detail string) {
	writeJSON(verifyOutput{
		Verdict:     "rejected",
		ErrorKind:   kind,
		ErrorDetail: detail,
	})
}

func writeJSON(output verifyOutput) {
	encoder := json.NewEncoder(os.Stdout)
	encoder.SetEscapeHTML(false)
	if err := encoder.Encode(output); err != nil {
		fmt.Fprintf(os.Stderr, "encode output: %v\n", err)
		os.Exit(1)
	}
}

type verifyOutput struct {
	Verdict          string      `json:"verdict"`
	Module           string      `json:"module,omitempty"`
	DeclarationCount int         `json:"declaration_count,omitempty"`
	AxiomCount       uint64      `json:"axiom_count,omitempty"`
	Hashes           *hashOutput `json:"hashes,omitempty"`
	ErrorKind        string      `json:"error_kind,omitempty"`
	ErrorDetail      string      `json:"error_detail,omitempty"`
}

type hashOutput struct {
	Export      string `json:"export"`
	AxiomReport string `json:"axiom_report"`
	Certificate string `json:"certificate"`
}
