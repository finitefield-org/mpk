package main

import (
	"bytes"
	"encoding/json"
	"strings"
	"testing"
)

func TestRunAcceptsPackagePath(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer

	exitCode := run([]string{"./sample"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit code = %d, want 0; stderr=%s", exitCode, stderr.String())
	}
	if stderr.Len() != 0 {
		t.Fatalf("stderr = %q, want empty", stderr.String())
	}

	var result cliResult
	if err := json.Unmarshal(stdout.Bytes(), &result); err != nil {
		t.Fatalf("decode stdout: %v\n%s", err, stdout.String())
	}
	if result.Schema != cliSchema {
		t.Fatalf("schema = %q, want %q", result.Schema, cliSchema)
	}
	if result.Status != "accepted" {
		t.Fatalf("status = %q, want accepted", result.Status)
	}
	if result.PackagePath != "./sample" {
		t.Fatalf("package path = %q, want ./sample", result.PackagePath)
	}
}

func TestRunRejectsMissingPackagePath(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer

	exitCode := run(nil, &stdout, &stderr)
	if exitCode != 2 {
		t.Fatalf("exit code = %d, want 2", exitCode)
	}
	if stdout.Len() != 0 {
		t.Fatalf("stdout = %q, want empty", stdout.String())
	}
	if !strings.Contains(stderr.String(), "requires exactly one package path") {
		t.Fatalf("stderr = %q, want package path error", stderr.String())
	}
}

func TestRunRejectsExtraArguments(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer

	exitCode := run([]string{"./one", "./two"}, &stdout, &stderr)
	if exitCode != 2 {
		t.Fatalf("exit code = %d, want 2", exitCode)
	}
	if !strings.Contains(stderr.String(), "requires exactly one package path") {
		t.Fatalf("stderr = %q, want package path error", stderr.String())
	}
}

func TestRunPrintsUsage(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer

	exitCode := run([]string{"--help"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit code = %d, want 0", exitCode)
	}
	if stdout.String() != usage {
		t.Fatalf("stdout = %q, want %q", stdout.String(), usage)
	}
	if stderr.Len() != 0 {
		t.Fatalf("stderr = %q, want empty", stderr.String())
	}
}
