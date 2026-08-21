package main

import (
	"fmt"
	"io"
	"regexp"
	"sort"
	"strings"
	"unicode"
	"unicode/utf8"
)

const frontendCLISchema = "mpk.frontend.cli.v0"

const (
	maximumIssues             = 1_024
	maximumIssueMessageBytes  = 4_096
	maximumIssueMessageBudget = 2_097_152
)

var issueCodePattern = regexp.MustCompile(`^[A-Z][A-Z0-9_]*$`)

type semanticParameters struct {
	TargetID     string `json:"target_id"`
	PointerWidth int64  `json:"pointer_width"`
}

type goSelection struct {
	Package  string `json:"package"`
	Function string `json:"function"`
}

type sourceSpan struct {
	NormalizedPath string `json:"normalized_path"`
	Start          int64  `json:"start"`
	End            int64  `json:"end"`
}

type issue struct {
	Code       string      `json:"code"`
	Message    string      `json:"message"`
	FunctionID *string     `json:"function_id,omitempty"`
	Span       *sourceSpan `json:"span,omitempty"`
}

type nonSuccessEnvelope struct {
	Schema             string             `json:"schema"`
	Status             string             `json:"status"`
	Phase              string             `json:"phase"`
	SourceLanguage     string             `json:"source_language"`
	SemanticProfile    string             `json:"semantic_profile"`
	SemanticParameters semanticParameters `json:"semantic_parameters"`
	Selection          goSelection        `json:"selection"`
	RejectedFeatures   []issue            `json:"rejected_features"`
	Diagnostics        []issue            `json:"diagnostics"`
}

func newFrontendErrorEnvelope(request lowerRequest, phase, code, message string) nonSuccessEnvelope {
	return nonSuccessEnvelope{
		Schema:             frontendCLISchema,
		Status:             "frontend-error",
		Phase:              phase,
		SourceLanguage:     "go",
		SemanticProfile:    request.SemanticProfile,
		SemanticParameters: semanticParameters{TargetID: request.Target, PointerWidth: goPointerWidth},
		Selection:          goSelection{Package: request.Package, Function: request.Function},
		RejectedFeatures:   []issue{},
		Diagnostics: []issue{{
			Code:    code,
			Message: message,
		}},
	}
}

func writeNonSuccessEnvelope(stdout io.Writer, request lowerRequest, envelope nonSuccessEnvelope) (int, error) {
	if err := validateNonSuccessEnvelope(request, envelope); err != nil {
		return 1, err
	}
	canonical, err := canonicalJSON(envelope)
	if err != nil {
		return 1, err
	}
	canonical = append(canonical, '\n')
	if err := writeAll(stdout, canonical); err != nil {
		return 1, err
	}
	return exitCodeForStatus(envelope.Status), nil
}

func validateNonSuccessEnvelope(request lowerRequest, envelope nonSuccessEnvelope) error {
	if envelope.Schema != frontendCLISchema || envelope.SourceLanguage != "go" {
		return fmt.Errorf("invalid generic frontend envelope identity")
	}
	if envelope.SemanticProfile != request.SemanticProfile || envelope.SemanticParameters.TargetID != request.Target || envelope.SemanticParameters.PointerWidth != goPointerWidth {
		return fmt.Errorf("frontend envelope semantic identity mismatch")
	}
	if envelope.Selection.Package != request.Package || envelope.Selection.Function != request.Function {
		return fmt.Errorf("frontend envelope selection mismatch")
	}
	if envelope.RejectedFeatures == nil || envelope.Diagnostics == nil {
		return fmt.Errorf("frontend envelope issue arrays must not be null")
	}
	if len(envelope.RejectedFeatures)+len(envelope.Diagnostics) > maximumIssues {
		return fmt.Errorf("frontend envelope issue count exceeds %d", maximumIssues)
	}
	messageBytes := 0
	for _, issues := range [][]issue{envelope.RejectedFeatures, envelope.Diagnostics} {
		for _, value := range issues {
			if err := validateIssue(value, envelope.Phase); err != nil {
				return err
			}
			messageBytes += len(value.Message)
			if messageBytes > maximumIssueMessageBudget {
				return fmt.Errorf("frontend envelope issue messages exceed the combined byte limit")
			}
		}
	}
	if !issuesSorted(envelope.RejectedFeatures) || !issuesSorted(envelope.Diagnostics) {
		return fmt.Errorf("frontend envelope issues are not canonically sorted")
	}
	switch envelope.Status {
	case "rejected":
		if !oneOf(envelope.Phase, "capture", "source", "metadata", "subset", "lowering", "emission") || len(envelope.RejectedFeatures)+len(envelope.Diagnostics) == 0 {
			return fmt.Errorf("invalid rejected envelope")
		}
	case "source-error":
		if !oneOf(envelope.Phase, "capture", "source", "metadata", "typecheck") || len(envelope.RejectedFeatures) != 0 || len(envelope.Diagnostics) == 0 {
			return fmt.Errorf("invalid source-error envelope")
		}
	case "frontend-error":
		if !oneOf(envelope.Phase, "capture", "source", "metadata", "typecheck", "subset", "lowering", "emission") || len(envelope.RejectedFeatures) != 0 || len(envelope.Diagnostics) == 0 {
			return fmt.Errorf("invalid frontend-error envelope")
		}
	default:
		return fmt.Errorf("unsupported non-success envelope status %q", envelope.Status)
	}
	return nil
}

func validateIssue(value issue, phase string) error {
	if !utf8.ValidString(value.Code) || len(value.Code) < 1 || len(value.Code) > 128 || !issueCodePattern.MatchString(value.Code) {
		return fmt.Errorf("frontend issue has an invalid code")
	}
	if !utf8.ValidString(value.Message) || len(value.Message) < 1 || len(value.Message) > maximumIssueMessageBytes || containsControl(value.Message) {
		return fmt.Errorf("frontend issue has an invalid normalized message")
	}
	if value.FunctionID != nil && !validGoFunctionID(*value.FunctionID) {
		return fmt.Errorf("frontend issue has an invalid function ID")
	}
	truncationMarker := value.Code == "GO_LIMIT_DIAGNOSTICS_TRUNCATED"
	if truncationMarker && (value.FunctionID != nil || value.Span != nil) {
		return fmt.Errorf("diagnostic truncation marker must be function- and span-free")
	}
	if oneOf(phase, "subset", "lowering", "emission") && value.FunctionID == nil && !truncationMarker {
		return fmt.Errorf("frontend issue in phase %s requires a function ID", phase)
	}
	if value.Span != nil {
		if !validPortablePath(value.Span.NormalizedPath) || value.Span.Start < 0 || value.Span.Start >= value.Span.End {
			return fmt.Errorf("frontend issue has an invalid source span")
		}
	}
	return nil
}

func containsControl(value string) bool {
	for _, character := range value {
		if unicode.IsControl(character) {
			return true
		}
	}
	return false
}

func validGoFunctionID(value string) bool {
	if value == "" || len(value) > 1024 {
		return false
	}
	parts := strings.Split(value, ".")
	if len(parts) < 2 {
		return false
	}
	if validGoUnitID(strings.Join(parts[:len(parts)-1], ".")) && validASCIIIdentifier(parts[len(parts)-1]) {
		return true
	}
	if len(parts) >= 3 && validGoUnitID(strings.Join(parts[:len(parts)-2], ".")) && validASCIIIdentifier(parts[len(parts)-2]) && validASCIIIdentifier(parts[len(parts)-1]) {
		return true
	}
	return false
}

func exitCodeForStatus(status string) int {
	switch status {
	case "ir-lowered":
		return 0
	case "frontend-error":
		return 1
	case "rejected":
		return 3
	case "source-error":
		return 4
	default:
		return 1
	}
}

func issuesSorted(issues []issue) bool {
	return sort.SliceIsSorted(issues, func(left, right int) bool {
		return compareIssues(issues[left], issues[right]) < 0
	})
}

func compareIssues(left, right issue) int {
	leftPath, leftStart, leftEnd := issueSpanKey(left)
	rightPath, rightStart, rightEnd := issueSpanKey(right)
	if comparison := strings.Compare(leftPath, rightPath); comparison != 0 {
		return comparison
	}
	if comparison := compareInt64(leftStart, rightStart); comparison != 0 {
		return comparison
	}
	if comparison := strings.Compare(left.Code, right.Code); comparison != 0 {
		return comparison
	}
	if comparison := strings.Compare(left.Message, right.Message); comparison != 0 {
		return comparison
	}
	leftFunction := ""
	rightFunction := ""
	if left.FunctionID != nil {
		leftFunction = *left.FunctionID
	}
	if right.FunctionID != nil {
		rightFunction = *right.FunctionID
	}
	if comparison := strings.Compare(leftFunction, rightFunction); comparison != 0 {
		return comparison
	}
	return compareInt64(leftEnd, rightEnd)
}

func issueSpanKey(value issue) (string, int64, int64) {
	if value.Span == nil {
		return "", 0, 0
	}
	return value.Span.NormalizedPath, value.Span.Start, value.Span.End
}

func compareInt64(left, right int64) int {
	switch {
	case left < right:
		return -1
	case left > right:
		return 1
	default:
		return 0
	}
}

func oneOf(value string, accepted ...string) bool {
	for _, candidate := range accepted {
		if value == candidate {
			return true
		}
	}
	return false
}

func writeAll(writer io.Writer, bytes []byte) error {
	for len(bytes) > 0 {
		written, err := writer.Write(bytes)
		if err != nil {
			return err
		}
		if written <= 0 || written > len(bytes) {
			return io.ErrShortWrite
		}
		bytes = bytes[written:]
	}
	return nil
}
