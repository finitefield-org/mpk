package main

import (
	"errors"
	"fmt"
	"io"
	"sort"
	"strings"
)

type virArtifact struct {
	Schema string    `json:"schema"`
	SHA256 string    `json:"sha256"`
	Value  virModule `json:"value"`
}

type successEnvelope struct {
	Schema             string             `json:"schema"`
	Status             string             `json:"status"`
	Phase              string             `json:"phase"`
	SourceLanguage     string             `json:"source_language"`
	SemanticProfile    string             `json:"semantic_profile"`
	SemanticParameters semanticParameters `json:"semantic_parameters"`
	Selection          goSelection        `json:"selection"`
	IR                 virArtifact        `json:"ir"`
	SourceManifest     sourceManifest     `json:"source_manifest"`
	SourceMap          sourceMap          `json:"source_map"`
	RejectedFeatures   []issue            `json:"rejected_features"`
	Diagnostics        []issue            `json:"diagnostics"`
}

type frontendArtifacts struct {
	Module       virModule
	ModuleJSON   []byte
	Map          sourceMap
	MapJSON      []byte
	Manifest     sourceManifest
	ManifestJSON []byte
	Envelope     successEnvelope
	EnvelopeJSON []byte
}

type artifactBuildError struct {
	code    string
	message string
}

func (e *artifactBuildError) Error() string { return e.code + ": " + e.message }

func artifactFinding(err error) (*loweringFinding, bool) {
	var buildError *artifactBuildError
	if !errors.As(err, &buildError) {
		return nil, false
	}
	return &loweringFinding{Code: buildError.code, Message: buildError.message}, true
}

func lowerPrivatePipeline(request lowerRequest, capture sourceCapture, loaded packageLoadResult, selection validatedLauncherSelection) (frontendArtifacts, []loweringFinding, error) {
	result, findings := lowerLoadedGo(loaded)
	if len(findings) > 0 {
		return frontendArtifacts{}, findings, nil
	}
	selected := 0
	for _, unit := range result.Module.Units {
		for _, function := range unit.Functions {
			if function.ID == request.Function && unit.ID == request.Package {
				selected++
			}
		}
	}
	if selected == 0 {
		return frontendArtifacts{}, []loweringFinding{{Code: "GO_SELECTION_FUNCTION_MISSING", Message: "selected function does not resolve", FunctionID: request.Function}}, nil
	}
	if selected != 1 {
		return frontendArtifacts{}, []loweringFinding{{Code: "GO_SELECTION_FUNCTION_AMBIGUOUS", Message: "selected function is ambiguous", FunctionID: request.Function}}, nil
	}
	if findings := attachContracts(&result.Module, capture, loaded); len(findings) > 0 {
		return frontendArtifacts{}, findings, nil
	}
	if finding := validateGeneratedVIRLimits(result.Module); finding != nil {
		return frontendArtifacts{}, []loweringFinding{*finding}, nil
	}
	moduleJSON, err := hashAndMarshalVIR(&result.Module)
	if err != nil {
		if finding, ok := artifactFinding(err); ok {
			return frontendArtifacts{}, []loweringFinding{*finding}, nil
		}
		return frontendArtifacts{}, nil, err
	}
	mapValue, mapJSON, err := buildSourceMap(result.Module, capture)
	if err != nil {
		if finding, ok := artifactFinding(err); ok {
			return frontendArtifacts{}, []loweringFinding{*finding}, nil
		}
		return frontendArtifacts{}, nil, err
	}
	manifest, manifestJSON, err := buildSourceManifest(request, capture, loaded, selection, result.Module.VIRHash, mapValue.SourceMapHash)
	if err != nil {
		return frontendArtifacts{}, nil, err
	}
	envelope := successEnvelope{
		Schema: frontendCLISchema, Status: "ir-lowered", Phase: "emission",
		SourceLanguage: "go", SemanticProfile: goSemanticProfile,
		SemanticParameters: semanticParameters{TargetID: goTarget, PointerWidth: goPointerWidth},
		Selection:          goSelection{Package: request.Package, Function: request.Function},
		IR:                 virArtifact{Schema: virSchema, SHA256: result.Module.VIRHash, Value: result.Module},
		SourceManifest:     manifest, SourceMap: mapValue,
		RejectedFeatures: []issue{}, Diagnostics: []issue{},
	}
	if err := validateSuccessEnvelope(request, envelope); err != nil {
		return frontendArtifacts{}, nil, err
	}
	envelopeJSON, err := canonicalJSON(envelope)
	if err != nil {
		return frontendArtifacts{}, nil, err
	}
	return frontendArtifacts{
		Module: result.Module, ModuleJSON: moduleJSON,
		Map: mapValue, MapJSON: mapJSON,
		Manifest: manifest, ManifestJSON: manifestJSON,
		Envelope: envelope, EnvelopeJSON: envelopeJSON,
	}, nil, nil
}

func hashAndMarshalVIR(module *virModule) ([]byte, error) {
	module.VIRHash = zeroSHA256()
	strict, err := strictValueFromTyped(*module)
	if err != nil {
		return nil, err
	}
	payload, err := withoutRootField(strict, "vir_hash")
	if err != nil {
		return nil, err
	}
	digest, err := hashCanonicalJSON(virHashDomain, payload)
	if err != nil {
		return nil, err
	}
	module.VIRHash = digest
	canonical, err := canonicalJSON(*module)
	if err != nil {
		return nil, err
	}
	if len(canonical) > maximumVIRBytes {
		return nil, &artifactBuildError{code: "VIR_LIMIT_CANONICAL_JSON_BYTES", message: "VIR exceeds the canonical byte limit"}
	}
	return canonical, nil
}

func validateSuccessEnvelope(request lowerRequest, envelope successEnvelope) error {
	if envelope.Schema != frontendCLISchema || envelope.Status != "ir-lowered" || envelope.Phase != "emission" || envelope.SourceLanguage != "go" {
		return fmt.Errorf("success envelope identity is invalid")
	}
	if envelope.SemanticProfile != request.SemanticProfile || envelope.SemanticParameters.TargetID != request.Target || envelope.SemanticParameters.PointerWidth != goPointerWidth {
		return fmt.Errorf("success envelope semantic identity differs from the request")
	}
	if envelope.Selection != (goSelection{Package: request.Package, Function: request.Function}) || envelope.IR.Schema != virSchema || envelope.IR.SHA256 != envelope.IR.Value.VIRHash {
		return fmt.Errorf("success envelope selection or VIR linkage is invalid")
	}
	if envelope.SourceMap.SourceIRHash != envelope.IR.SHA256 || envelope.SourceManifest.VIRHash != envelope.IR.SHA256 || envelope.SourceManifest.SourceMapHash != envelope.SourceMap.SourceMapHash {
		return fmt.Errorf("success envelope artifact hashes are not linked")
	}
	if envelope.RejectedFeatures == nil || envelope.Diagnostics == nil || len(envelope.RejectedFeatures) != 0 || len(envelope.Diagnostics) != 0 {
		return fmt.Errorf("successful lowering must have empty issue arrays")
	}
	return nil
}

func writeSuccessEnvelope(stdout io.Writer, artifacts frontendArtifacts) error {
	bytes := append(append([]byte{}, artifacts.EnvelopeJSON...), '\n')
	return writeAll(stdout, bytes)
}

func loweringFindingsEnvelope(request lowerRequest, findings []loweringFinding) (nonSuccessEnvelope, error) {
	if len(findings) == 0 {
		return nonSuccessEnvelope{}, fmt.Errorf("lowering rejection requires at least one finding")
	}
	sortLoweringFindings(findings)
	status, phase := loweringFindingDisposition(findings[0].Code)
	envelope := nonSuccessEnvelope{
		Schema: frontendCLISchema, Status: status, Phase: phase,
		SourceLanguage: "go", SemanticProfile: request.SemanticProfile,
		SemanticParameters: semanticParameters{TargetID: request.Target, PointerWidth: goPointerWidth},
		Selection:          goSelection{Package: request.Package, Function: request.Function},
		RejectedFeatures:   []issue{}, Diagnostics: []issue{},
	}
	matching := make([]loweringFinding, 0, len(findings))
	for _, finding := range findings {
		findingStatus, findingPhase := loweringFindingDisposition(finding.Code)
		if findingStatus == status && findingPhase == phase {
			matching = append(matching, finding)
		}
	}
	truncated := len(matching) > maximumIssues
	if truncated {
		matching = matching[:maximumIssues-1]
	}
	for _, finding := range matching {
		functionID := finding.FunctionID
		if functionID == "" {
			functionID = request.Function
		}
		value := issue{Code: finding.Code, Message: finding.Message, FunctionID: &functionID}
		if finding.Origin.Kind == "source" && finding.Origin.NormalizedPath != "" && finding.Origin.End > finding.Origin.Start {
			value.Span = &sourceSpan{NormalizedPath: finding.Origin.NormalizedPath, Start: finding.Origin.Start, End: finding.Origin.End}
		}
		if status == "frontend-error" {
			envelope.Diagnostics = append(envelope.Diagnostics, value)
		} else {
			envelope.RejectedFeatures = append(envelope.RejectedFeatures, value)
		}
	}
	if truncated {
		marker := issue{Code: "GO_LIMIT_DIAGNOSTICS_TRUNCATED", Message: "additional diagnostics were truncated"}
		envelope.Diagnostics = append(envelope.Diagnostics, marker)
	}
	sort.Slice(envelope.RejectedFeatures, func(i, j int) bool {
		return compareIssues(envelope.RejectedFeatures[i], envelope.RejectedFeatures[j]) < 0
	})
	sort.Slice(envelope.Diagnostics, func(i, j int) bool { return compareIssues(envelope.Diagnostics[i], envelope.Diagnostics[j]) < 0 })
	if err := validateNonSuccessEnvelope(request, envelope); err != nil {
		return nonSuccessEnvelope{}, err
	}
	return envelope, nil
}

func loweringFindingDisposition(code string) (string, string) {
	if strings.HasPrefix(code, "GO_FRONTEND_") {
		return "frontend-error", "lowering"
	}
	if strings.HasPrefix(code, "GO_LOWER_") || strings.HasPrefix(code, "GO_SOURCE_MAP_") {
		return "rejected", "lowering"
	}
	if strings.HasPrefix(code, "VIR_LIMIT_") || strings.HasPrefix(code, "SOURCE_MAP_LIMIT_") {
		return "rejected", "emission"
	}
	return "rejected", "subset"
}
