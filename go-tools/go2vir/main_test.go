package main

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"reflect"
	"sort"
	"strconv"
	"strings"
	"testing"
)

type invocationFixture struct {
	Schema  string              `json:"schema"`
	Args    []string            `json:"args"`
	Request fixtureLowerRequest `json:"request"`
}

type fixtureLowerRequest struct {
	SourceRoot                  string   `json:"source_root"`
	Package                     string   `json:"package"`
	SemanticProfile             string   `json:"semantic_profile"`
	Target                      string   `json:"target"`
	Function                    string   `json:"function"`
	FrontendBundleID            string   `json:"frontend_bundle_id"`
	FrontendSHA256              string   `json:"frontend_sha256"`
	ReleaseRegistryID           string   `json:"release_registry_id"`
	ReleaseRegistrySHA256       string   `json:"release_registry_sha256"`
	ToolchainBundleID           string   `json:"toolchain_bundle_id"`
	ToolchainRoot               string   `json:"toolchain_root"`
	ToolchainDistributionSHA256 string   `json:"toolchain_distribution_sha256"`
	Contracts                   []string `json:"contracts"`
}

func TestRunPrintsHelpOutsideTheProtocol(t *testing.T) {
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	exitCode := run([]string{"--help"}, &stdout, &stderr)
	if exitCode != 0 {
		t.Fatalf("exit code = %d, want 0", exitCode)
	}
	if stdout.String() != usage {
		t.Fatalf("stdout = %q, want exact usage", stdout.String())
	}
	if stderr.Len() != 0 {
		t.Fatalf("stderr = %q, want empty", stderr.String())
	}
}

func TestRunRequiresClosedProfileAndLauncherArgumentsBeforeSourceReads(t *testing.T) {
	fixture := loadInvocationFixture(t)
	tests := []struct {
		name string
		args []string
	}{
		{name: "no default profile or target", args: []string{"lower", logicalSourceRoot}},
		{name: "wrong command", args: replaceArgument(fixture.Args, 0, "compile")},
		{name: "wrong profile", args: replaceOptionValue(fixture.Args, "--semantic-profile", "mpk.rust.checked.v0")},
		{name: "wrong target", args: replaceOptionValue(fixture.Args, "--target", "linux/arm64")},
		{name: "missing profile", args: removeOptionPair(fixture.Args, "--semantic-profile")},
		{name: "missing target", args: removeOptionPair(fixture.Args, "--target")},
		{name: "reordered singleton", args: swapOptionPairs(fixture.Args, "--package", "--semantic-profile")},
		{name: "duplicate singleton", args: append(copyStrings(fixture.Args), "--target", goTarget)},
		{name: "unknown trailing argument", args: append(copyStrings(fixture.Args), "--debug", "true")},
		{name: "uppercase digest", args: replaceOptionValue(fixture.Args, "--frontend-sha256", strings.ToUpper(fixture.Request.FrontendSHA256))},
		{name: "nonportable contract", args: replaceOptionValue(fixture.Args, "--contract", "../identity.json")},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			var stdout bytes.Buffer
			var stderr bytes.Buffer
			exitCode := run(test.args, &stdout, &stderr)
			if exitCode != 2 {
				t.Fatalf("exit code = %d, want 2; stderr=%s", exitCode, stderr.String())
			}
			if stdout.Len() != 0 {
				t.Fatalf("configuration error stdout = %q, want zero bytes", stdout.String())
			}
			if !strings.HasPrefix(stderr.String(), usage) {
				t.Fatalf("stderr does not begin with usage: %q", stderr.String())
			}
		})
	}
}

func TestLowerArgumentsEnforceContractAndTransportBoundaries(t *testing.T) {
	fixture := loadInvocationFixture(t)
	withoutContracts := removeOptionPair(fixture.Args, "--contract")
	request, err := parseLowerArguments(withoutContracts)
	if err != nil {
		t.Fatalf("zero contracts rejected: %v", err)
	}
	if request.Contracts == nil || len(request.Contracts) != 0 {
		t.Fatalf("zero contracts parsed as %#v, want a non-nil empty list", request.Contracts)
	}

	atLimit := copyStrings(withoutContracts)
	for index := 0; index < maximumContracts; index++ {
		atLimit = append(atLimit, "--contract", "contracts/"+fmtIndex(index)+".json")
	}
	request, err = parseLowerArguments(atLimit)
	if err != nil {
		t.Fatalf("%d contracts rejected: %v", maximumContracts, err)
	}
	if len(request.Contracts) != maximumContracts {
		t.Fatalf("contract count = %d, want %d", len(request.Contracts), maximumContracts)
	}

	aboveLimit := append(copyStrings(atLimit), "--contract", "contracts/999.json")
	assertUsageError(t, aboveLimit)
	assertUsageError(t, append(copyStrings(withoutContracts),
		"--contract", "contracts/A.json",
		"--contract", "contracts/a.json",
	))
	assertUsageError(t, append(copyStrings(withoutContracts),
		"--contract", "contracts/b.json",
		"--contract", "contracts/a.json",
	))

	if err := validateArgumentTransport([]string{strings.Repeat("a", maximumArgumentBytes-1)}); err != nil {
		t.Fatalf("inclusive argument-byte limit rejected: %v", err)
	}
	if err := validateArgumentTransport([]string{strings.Repeat("a", maximumArgumentBytes)}); err == nil {
		t.Fatal("argument bytes above the inclusive limit were accepted")
	}
}

func TestRunRejectsAnUninstalledRegisteredFrontendWithoutFallback(t *testing.T) {
	fixture := loadInvocationFixture(t)
	request, err := parseLowerArguments(fixture.Args)
	if err != nil {
		t.Fatalf("parse fixture arguments: %v", err)
	}
	wantRequest := fixture.Request.lowerRequest()
	if !reflect.DeepEqual(request, wantRequest) {
		t.Fatalf("request mismatch:\n got: %#v\nwant: %#v", request, wantRequest)
	}

	var stdout bytes.Buffer
	var stderr bytes.Buffer
	exitCode := run(fixture.Args, &stdout, &stderr)
	if exitCode != 1 {
		t.Fatalf("exit code = %d, want 1; stderr=%s", exitCode, stderr.String())
	}
	if stderr.Len() != 0 {
		t.Fatalf("stderr = %q, want empty for a JSON-bearing status", stderr.String())
	}
	wantEnvelope := newFrontendErrorEnvelope(
		wantRequest,
		"capture",
		"GO_FRONTEND_TOOLCHAIN",
		"registered frontend inventory is invalid",
	)
	want, err := canonicalJSON(wantEnvelope)
	if err != nil {
		t.Fatalf("canonical expected envelope: %v", err)
	}
	want = append(want, '\n')
	if !bytes.Equal(stdout.Bytes(), want) {
		t.Fatalf("stdout is not the exact canonical envelope plus LF:\n got: %s\nwant: %s", stdout.Bytes(), want)
	}
	if bytes.Count(stdout.Bytes(), []byte{'\n'}) != 1 || !bytes.HasSuffix(stdout.Bytes(), []byte{'\n'}) {
		t.Fatalf("stdout transport does not contain exactly one terminal LF: %q", stdout.Bytes())
	}

	parsed := mustStrictObject(t, bytes.TrimSuffix(stdout.Bytes(), []byte{'\n'}))
	assertExactFields(t, parsed, []string{
		"schema", "status", "phase", "source_language", "semantic_profile",
		"semantic_parameters", "selection", "rejected_features", "diagnostics",
	})
	for _, forbidden := range []string{"ssa", "gir", "debug", "binary", "packages"} {
		if _, exists := parsed[forbidden]; exists {
			t.Fatalf("forbidden compatibility/debug field %q was emitted", forbidden)
		}
	}
}

func TestWriteNonSuccessEnvelopeUsesExactStatusExitPairs(t *testing.T) {
	request := loadInvocationFixture(t).Request.lowerRequest()
	functionID := request.Function
	common := nonSuccessEnvelope{
		Schema:             frontendCLISchema,
		SourceLanguage:     "go",
		SemanticProfile:    request.SemanticProfile,
		SemanticParameters: semanticParameters{TargetID: request.Target, PointerWidth: goPointerWidth},
		Selection:          goSelection{Package: request.Package, Function: request.Function},
		RejectedFeatures:   []issue{},
		Diagnostics:        []issue{},
	}
	tests := []struct {
		name     string
		envelope nonSuccessEnvelope
		exitCode int
	}{
		{name: "rejected", envelope: func() nonSuccessEnvelope {
			value := common
			value.Status = "rejected"
			value.Phase = "subset"
			value.RejectedFeatures = []issue{{Code: "GO_SUBSET_MAP", Message: "map type is outside mpk.go.fixed.v0", FunctionID: &functionID}}
			return value
		}(), exitCode: 3},
		{name: "source error", envelope: func() nonSuccessEnvelope {
			value := common
			value.Status = "source-error"
			value.Phase = "source"
			value.Diagnostics = []issue{{Code: "GO_SOURCE_PARSE", Message: "expected expression"}}
			return value
		}(), exitCode: 4},
		{name: "frontend error", envelope: newFrontendErrorEnvelope(request, "capture", "GO_FRONTEND_INTERNAL", "capture unavailable"), exitCode: 1},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			var stdout bytes.Buffer
			exitCode, err := writeNonSuccessEnvelope(&stdout, request, test.envelope)
			if err != nil {
				t.Fatalf("write envelope: %v", err)
			}
			if exitCode != test.exitCode {
				t.Fatalf("exit code = %d, want %d", exitCode, test.exitCode)
			}
			if !bytes.HasSuffix(stdout.Bytes(), []byte{'\n'}) || bytes.Count(stdout.Bytes(), []byte{'\n'}) != 1 {
				t.Fatalf("transport is not exact JSON plus one LF: %q", stdout.Bytes())
			}
			canonical, err := canonicalJSON(test.envelope)
			if err != nil {
				t.Fatalf("canonical envelope: %v", err)
			}
			if !bytes.Equal(stdout.Bytes(), append(canonical, '\n')) {
				t.Fatal("envelope transport contains noncanonical or extra stdout bytes")
			}
		})
	}
}

func TestNonSuccessEnvelopeRejectsRepeatedIdentityDrift(t *testing.T) {
	request := loadInvocationFixture(t).Request.lowerRequest()
	base := newFrontendErrorEnvelope(request, "capture", "GO_FRONTEND_INTERNAL", "capture unavailable")
	if err := validateNonSuccessEnvelope(request, base); err != nil {
		t.Fatalf("base envelope rejected: %v", err)
	}

	tests := []struct {
		name   string
		mutate func(*nonSuccessEnvelope)
	}{
		{name: "language", mutate: func(value *nonSuccessEnvelope) { value.SourceLanguage = "rust" }},
		{name: "profile", mutate: func(value *nonSuccessEnvelope) { value.SemanticProfile = "mpk.rust.checked.v0" }},
		{name: "target", mutate: func(value *nonSuccessEnvelope) { value.SemanticParameters.TargetID = "linux/arm64" }},
		{name: "pointer width", mutate: func(value *nonSuccessEnvelope) { value.SemanticParameters.PointerWidth = 32 }},
		{name: "package", mutate: func(value *nonSuccessEnvelope) { value.Selection.Package = "example.com/other" }},
		{name: "function", mutate: func(value *nonSuccessEnvelope) { value.Selection.Function = "example.com/mpk/vector.Other" }},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			mutated := base
			test.mutate(&mutated)
			if err := validateNonSuccessEnvelope(request, mutated); err == nil {
				t.Fatal("identity drift was accepted")
			}
		})
	}
}

func TestNonSuccessEnvelopeRejectsMalformedIssues(t *testing.T) {
	request := loadInvocationFixture(t).Request.lowerRequest()
	base := newFrontendErrorEnvelope(request, "capture", "GO_FRONTEND_INTERNAL", "capture unavailable")
	functionID := request.Function
	tests := []struct {
		name   string
		mutate func(*nonSuccessEnvelope)
	}{
		{name: "null diagnostics", mutate: func(value *nonSuccessEnvelope) { value.Diagnostics = nil }},
		{name: "invalid code", mutate: func(value *nonSuccessEnvelope) { value.Diagnostics[0].Code = "go_error" }},
		{name: "control in message", mutate: func(value *nonSuccessEnvelope) { value.Diagnostics[0].Message = "bad\nmessage" }},
		{name: "invalid span", mutate: func(value *nonSuccessEnvelope) {
			value.Diagnostics[0].Span = &sourceSpan{NormalizedPath: "source.go", Start: 4, End: 4}
		}},
		{name: "later phase without function", mutate: func(value *nonSuccessEnvelope) { value.Phase = "lowering" }},
		{name: "unsorted diagnostics", mutate: func(value *nonSuccessEnvelope) {
			value.Diagnostics = []issue{
				{Code: "GO_FRONTEND_TOOLCHAIN", Message: "z", FunctionID: &functionID},
				{Code: "GO_FRONTEND_INTERNAL", Message: "a", FunctionID: &functionID},
			}
			value.Phase = "lowering"
		}},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			mutated := base
			mutated.Diagnostics = append([]issue(nil), base.Diagnostics...)
			test.mutate(&mutated)
			if err := validateNonSuccessEnvelope(request, mutated); err == nil {
				t.Fatal("malformed issue envelope was accepted")
			}
		})
	}
}

func TestSharedFrontendProtocolJSONStatusesRoundTripCanonically(t *testing.T) {
	protocol := loadStrictObjectFile(t, repoPath("develop/specs/vectors/frontend-protocol-v0.json"))
	statusCases := arrayField(t, protocol, "status_cases")
	validIDs := []string{
		"status.valid_ir_lowered",
		"status.valid_rejected",
		"status.valid_source_error",
		"status.valid_frontend_error",
	}
	seen := make(map[string]struct{}, len(validIDs))
	for _, id := range validIDs {
		caseObject := findCase(t, statusCases, id)
		envelope := protocolStatusInput(t, caseObject)
		assertProtocolEnvelopeShape(t, envelope)
		canonical, err := canonicalJSONValue(envelope)
		if err != nil {
			t.Fatalf("%s canonicalize: %v", id, err)
		}
		roundTrip, err := decodeStrictJSON(canonical)
		if err != nil {
			t.Fatalf("%s strict round trip: %v", id, err)
		}
		reencoded, err := canonicalJSONValue(roundTrip)
		if err != nil {
			t.Fatalf("%s re-encode: %v", id, err)
		}
		if !bytes.Equal(canonical, reencoded) {
			t.Fatalf("%s changed canonical bytes after round trip", id)
		}
		transport := append(copyBytes(canonical), '\n')
		expect := objectField(t, caseObject, "expect")
		if length, exists := optionalInt(expect, "canonical_jcs_utf8_length"); exists && int64(len(canonical)) != length {
			t.Fatalf("%s JCS length = %d, want %d", id, len(canonical), length)
		}
		if length, exists := optionalInt(expect, "stdout_utf8_length"); exists && int64(len(transport)) != length {
			t.Fatalf("%s stdout length = %d, want %d", id, len(transport), length)
		}
		if digest, exists := optionalString(expect, "stdout_sha256"); exists && sha256Hex(transport) != digest {
			t.Fatalf("%s stdout digest = %s, want %s", id, sha256Hex(transport), digest)
		}
		process := objectField(t, caseObject, "process")
		status := stringField(t, envelope, "status")
		if got, want := exitCodeForStatus(status), int(intField(t, process, "exit")); got != want {
			t.Fatalf("%s status exit = %d, vector exit = %d", id, got, want)
		}
		seen[id] = struct{}{}
	}
	if len(seen) != len(validIDs) {
		t.Fatalf("executed status cases = %d, want %d", len(seen), len(validIDs))
	}
}

func TestCanonicalJSONAndDomainHashesMatchSharedVectors(t *testing.T) {
	hashVectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/vir-hash-v0.json"))
	virVectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/vir-v0.json"))
	modules := arrayField(t, virVectors, "module_cases")
	domains := objectField(t, hashVectors, "domains")
	cases := arrayField(t, hashVectors, "canonical_cases")
	if len(cases) == 0 {
		t.Fatal("canonical hash vector cases are empty")
	}
	for _, value := range cases {
		caseObject := asObject(t, value, "canonical hash case")
		id := stringField(t, caseObject, "id")
		sourceCase := findCase(t, modules, stringField(t, caseObject, "source_case"))
		source := objectField(t, sourceCase, "input")
		target := cloneJSONValue(t, source)
		if pointer, exists := optionalString(caseObject, "source_pointer"); exists && pointer != "/" {
			target = jsonPointer(t, target, pointer)
		}
		for _, field := range stringArrayField(t, caseObject, "excluded_fields") {
			var err error
			target, err = withoutRootField(target, field)
			if err != nil {
				t.Fatalf("%s exclude %s: %v", id, field, err)
			}
		}
		canonical, err := canonicalJSONValue(target)
		if err != nil {
			t.Fatalf("%s canonicalize: %v", id, err)
		}
		if got, want := string(canonical), stringField(t, caseObject, "expected_jcs"); got != want {
			t.Fatalf("%s JCS mismatch:\n got: %s\nwant: %s", id, got, want)
		}
		if got, want := int64(len(canonical)), intField(t, caseObject, "expected_jcs_utf8_length"); got != want {
			t.Fatalf("%s JCS length = %d, want %d", id, got, want)
		}
		domain := objectField(t, domains, stringField(t, caseObject, "domain"))
		domainText := stringField(t, domain, "text")
		if got, want := int64(len(domainText)+1+len(canonical)), intField(t, caseObject, "expected_preimage_length"); got != want {
			t.Fatalf("%s preimage length = %d, want %d", id, got, want)
		}
		digest, err := hashCanonicalJSON(domainText, target)
		if err != nil {
			t.Fatalf("%s hash: %v", id, err)
		}
		if want := stringField(t, caseObject, "expected_sha256"); digest != want {
			t.Fatalf("%s digest = %s, want %s", id, digest, want)
		}
	}

	equivalenceCases := arrayField(t, hashVectors, "canonical_equivalence_cases")
	objectOrder := findCase(t, equivalenceCases, "canonical.object_key_order")
	want := stringField(t, objectOrder, "expected_jcs")
	for _, text := range stringArrayField(t, objectOrder, "json_texts") {
		value, err := decodeStrictJSON([]byte(text))
		if err != nil {
			t.Fatalf("parse equivalence JSON: %v", err)
		}
		canonical, err := canonicalJSONValue(value)
		if err != nil {
			t.Fatalf("canonicalize equivalence JSON: %v", err)
		}
		if string(canonical) != want {
			t.Fatalf("object order JCS = %s, want %s", canonical, want)
		}
	}

	rootExcluded := findCase(t, equivalenceCases, "canonical.root_hash_excluded")
	rootExcludedSource := objectField(t, findCase(t, modules, stringField(t, rootExcluded, "source_case")), "input")
	rootExcludedTarget := applyReplacePatches(t, rootExcludedSource, arrayField(t, rootExcluded, "patches"))
	rootExcludedTarget, err := withoutRootField(rootExcludedTarget, "vir_hash")
	if err != nil {
		t.Fatalf("exclude mutated VIR hash: %v", err)
	}
	assertVectorHash(t, rootExcluded, stringField(t, objectField(t, domains, "vir"), "text"), rootExcludedTarget)

	contractExcluded := findCase(t, equivalenceCases, "canonical.contract_hash_excluded")
	contractSource := objectField(t, findCase(t, modules, stringField(t, contractExcluded, "source_case")), "input")
	contractTarget := jsonPointer(t, contractSource, stringField(t, contractExcluded, "source_pointer"))
	contractTarget = applyReplacePatches(t, contractTarget, arrayField(t, contractExcluded, "patches"))
	contractTarget, err = withoutRootField(contractTarget, "contract_hash")
	if err != nil {
		t.Fatalf("exclude mutated contract hash: %v", err)
	}
	assertVectorHash(t, contractExcluded, stringField(t, objectField(t, domains, "contract"), "text"), contractTarget)

	domainSeparator := findCase(t, equivalenceCases, "canonical.domain_separator_required")
	domainSource := objectField(t, findCase(t, modules, stringField(t, domainSeparator, "source_case")), "input")
	domainTarget, err := withoutRootField(domainSource, "vir_hash")
	if err != nil {
		t.Fatalf("exclude VIR hash for domain case: %v", err)
	}
	virDomain := stringField(t, objectField(t, domains, "vir"), "text")
	assertVectorHash(t, domainSeparator, virDomain, domainTarget)
	wrongDigest, err := hashCanonicalJSON(stringField(t, domainSeparator, "wrong_domain_text"), domainTarget)
	if err != nil {
		t.Fatalf("hash wrong-domain case: %v", err)
	}
	if want := stringField(t, domainSeparator, "wrong_domain_sha256"); wrongDigest != want {
		t.Fatalf("wrong-domain digest = %s, want %s", wrongDigest, want)
	}
	canonicalDomainTarget, err := canonicalJSONValue(domainTarget)
	if err != nil {
		t.Fatalf("canonicalize domain target: %v", err)
	}
	if got, want := sha256Hex(append([]byte(virDomain), canonicalDomainTarget...)), stringField(t, domainSeparator, "without_separator_sha256"); got != want {
		t.Fatalf("separator-free digest = %s, want %s", got, want)
	}

	for _, value := range arrayField(t, hashVectors, "mutation_cases") {
		caseObject := asObject(t, value, "hash mutation case")
		source := objectField(t, findCase(t, modules, stringField(t, caseObject, "source_case")), "input")
		target := applyReplacePatches(t, source, arrayField(t, caseObject, "patches"))
		target, err = withoutRootField(target, "vir_hash")
		if err != nil {
			t.Fatalf("%s exclude VIR hash: %v", stringField(t, caseObject, "id"), err)
		}
		canonical, err := canonicalJSONValue(target)
		if err != nil {
			t.Fatalf("%s canonicalize: %v", stringField(t, caseObject, "id"), err)
		}
		if got, want := int64(len(canonical)), intField(t, caseObject, "expected_jcs_utf8_length"); got != want {
			t.Fatalf("%s JCS length = %d, want %d", stringField(t, caseObject, "id"), got, want)
		}
		assertVectorHash(t, caseObject, virDomain, target)
	}

	ordered := findCase(t, arrayField(t, hashVectors, "ordered_array_cases"), "ordered.contract_clauses_are_not_sorted")
	orderedSource := objectField(t, findCase(t, modules, stringField(t, ordered, "source_case")), "input")
	orderedSource = asObject(t, jsonPointer(t, orderedSource, stringField(t, ordered, "source_pointer")), "ordered contract")
	var orderedDigests []string
	for _, value := range arrayField(t, ordered, "variants") {
		variant := asObject(t, value, "ordered variant")
		target := applyReplacePatches(t, orderedSource, arrayField(t, variant, "patches"))
		target, err = withoutRootField(target, "contract_hash")
		if err != nil {
			t.Fatalf("exclude ordered contract hash: %v", err)
		}
		digest, err := hashCanonicalJSON(stringField(t, objectField(t, domains, "contract"), "text"), target)
		if err != nil {
			t.Fatalf("hash ordered contract: %v", err)
		}
		if want := stringField(t, variant, "expected_sha256"); digest != want {
			t.Fatalf("ordered contract digest = %s, want %s", digest, want)
		}
		orderedDigests = append(orderedDigests, digest)
	}
	if len(orderedDigests) != 2 || orderedDigests[0] == orderedDigests[1] {
		t.Fatal("ordered contract clauses did not affect the digest")
	}

	rawContract := findCase(t, arrayField(t, hashVectors, "raw_contract_cases"), "raw_contract.whitespace_is_traceability_only")
	rawTexts := stringArrayField(t, rawContract, "raw_json_texts")
	rawLengths := intArrayField(t, rawContract, "expected_raw_utf8_lengths")
	rawDigests := stringArrayField(t, rawContract, "expected_raw_sha256")
	for index, text := range rawTexts {
		if len(text) != int(rawLengths[index]) || sha256Hex([]byte(text)) != rawDigests[index] {
			t.Fatalf("raw contract %d does not match its length/digest vector", index)
		}
		if _, err := decodeStrictJSON([]byte(text)); err != nil {
			t.Fatalf("raw contract %d is not strict JSON: %v", index, err)
		}
	}
	if rawDigests[0] == rawDigests[1] {
		t.Fatal("whitespace-distinct raw contracts have the same raw digest")
	}
	normalizedSource := objectField(t, findCase(t, modules, stringField(t, rawContract, "normalized_source_case")), "input")
	normalizedContract := jsonPointer(t, normalizedSource, "/units/0/functions/0/contracts")
	normalizedContract, err = withoutRootField(normalizedContract, "contract_hash")
	if err != nil {
		t.Fatalf("exclude normalized contract hash: %v", err)
	}
	normalizedDigest, err := hashCanonicalJSON(stringField(t, objectField(t, domains, "contract"), "text"), normalizedContract)
	if err != nil {
		t.Fatalf("hash normalized contract: %v", err)
	}
	if want := stringField(t, rawContract, "expected_contract_hash"); normalizedDigest != want {
		t.Fatalf("normalized contract digest = %s, want %s", normalizedDigest, want)
	}
}

func TestStrictJSONRejectsDuplicateAndForbiddenContractOrProtocolInput(t *testing.T) {
	tests := []struct {
		name string
		json string
	}{
		{name: "duplicate protocol key", json: `{"schema":"mpk.frontend.cli.v0","schema":"mpk.frontend.cli.v0"}`},
		{name: "duplicate contract key", json: `{"schema":"mpk.go.contract.v0","function":"F","function":"G"}`},
		{name: "float", json: `{"value":1.0}`},
		{name: "exponent", json: `{"value":1e0}`},
		{name: "integer over maximum", json: `{"value":9007199254740992}`},
		{name: "lone high surrogate", json: `{"value":"\ud800"}`},
		{name: "lone low surrogate", json: `{"value":"\udc00"}`},
		{name: "second value", json: `{}` + `{}`},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if _, err := decodeStrictJSON([]byte(test.json)); err == nil {
				t.Fatal("strict JSON input was accepted")
			}
		})
	}
}

func TestStrictJSONEnforcesTheExactContainerDepthLimit(t *testing.T) {
	atLimit := strings.Repeat("[", maximumJSONDepth) + "0" + strings.Repeat("]", maximumJSONDepth)
	if _, err := decodeStrictJSON([]byte(atLimit)); err != nil {
		t.Fatalf("JSON at depth %d rejected: %v", maximumJSONDepth, err)
	}
	aboveLimit := "[" + atLimit + "]"
	if _, err := decodeStrictJSON([]byte(aboveLimit)); err == nil {
		t.Fatalf("JSON above depth %d was accepted", maximumJSONDepth)
	}
}

func TestCanonicalJSONRejectsInvalidProgrammaticStringsAndCycles(t *testing.T) {
	if _, err := canonicalJSON(map[string]string{"value": string([]byte{0xff})}); err == nil {
		t.Fatal("invalid UTF-8 programmatic string was silently repaired")
	}
	cyclic := make(map[string]any)
	cyclic["self"] = cyclic
	if _, err := canonicalJSON(cyclic); err == nil {
		t.Fatal("cyclic programmatic JSON input was accepted")
	}
}

func TestInvocationFixtureContainsNoPlaceholders(t *testing.T) {
	fixture := loadInvocationFixture(t)
	if fixture.Schema != "mpk.go2vir.invocation_fixture.v0" {
		t.Fatalf("fixture schema = %q", fixture.Schema)
	}
	for index, argument := range fixture.Args {
		if argument == "" || strings.ContainsAny(argument, "<>") {
			t.Fatalf("fixture argument %d is empty or a placeholder: %q", index, argument)
		}
	}
}

func (fixture fixtureLowerRequest) lowerRequest() lowerRequest {
	return lowerRequest{
		SourceRoot:                  fixture.SourceRoot,
		Package:                     fixture.Package,
		SemanticProfile:             fixture.SemanticProfile,
		Target:                      fixture.Target,
		Function:                    fixture.Function,
		FrontendBundleID:            fixture.FrontendBundleID,
		FrontendSHA256:              fixture.FrontendSHA256,
		ReleaseRegistryID:           fixture.ReleaseRegistryID,
		ReleaseRegistrySHA256:       fixture.ReleaseRegistrySHA256,
		ToolchainBundleID:           fixture.ToolchainBundleID,
		ToolchainRoot:               fixture.ToolchainRoot,
		ToolchainDistributionSHA256: fixture.ToolchainDistributionSHA256,
		Contracts:                   fixture.Contracts,
	}
}

func loadInvocationFixture(t *testing.T) invocationFixture {
	t.Helper()
	bytes := mustReadFile(t, "testdata/valid-invocation.json")
	value, err := decodeStrictJSON(bytes)
	if err != nil {
		t.Fatalf("strict invocation fixture: %v", err)
	}
	canonical, err := canonicalJSONValue(value)
	if err != nil {
		t.Fatalf("canonical invocation fixture: %v", err)
	}
	var fixture invocationFixture
	if err := json.Unmarshal(canonical, &fixture); err != nil {
		t.Fatalf("decode invocation fixture: %v", err)
	}
	return fixture
}

func protocolStatusInput(t *testing.T, caseObject map[string]jsonValue) map[string]jsonValue {
	t.Helper()
	if input, exists := caseObject["input"]; exists {
		return asObject(t, input, "status input")
	}
	construction := objectField(t, caseObject, "construction")
	if stringField(t, construction, "fixture") != "canonical_from_dependencies" {
		t.Fatalf("unsupported valid status construction")
	}
	virVectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/vir-v0.json"))
	mapVectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/source-map-v0.json"))
	manifestVectors := loadStrictObjectFile(t, repoPath("develop/specs/vectors/source-manifest-v0.json"))
	virCase := findCase(t, arrayField(t, virVectors, "module_cases"), stringField(t, construction, "vir_case"))
	mapCase := findCase(t, arrayField(t, mapVectors, "map_cases"), stringField(t, construction, "source_map_case"))
	manifestCase := findCase(t, arrayField(t, manifestVectors, "manifest_cases"), stringField(t, construction, "source_manifest_case"))
	vir := objectField(t, virCase, "input")
	return map[string]jsonValue{
		"schema":              frontendCLISchema,
		"status":              "ir-lowered",
		"phase":               "emission",
		"source_language":     construction["source_language"],
		"semantic_profile":    construction["semantic_profile"],
		"semantic_parameters": construction["semantic_parameters"],
		"selection":           construction["selection"],
		"ir": map[string]jsonValue{
			"schema": "mpk.vir.v0",
			"sha256": vir["vir_hash"],
			"value":  vir,
		},
		"source_manifest":   objectField(t, manifestCase, "input"),
		"source_map":        objectField(t, mapCase, "input"),
		"rejected_features": []jsonValue{},
		"diagnostics":       []jsonValue{},
	}
}

func assertProtocolEnvelopeShape(t *testing.T, envelope map[string]jsonValue) {
	t.Helper()
	common := []string{"schema", "status", "phase", "source_language", "semantic_profile", "semantic_parameters", "selection"}
	status := stringField(t, envelope, "status")
	fields := append(copyStrings(common), "rejected_features", "diagnostics")
	if status == "ir-lowered" {
		fields = append(fields, "ir", "source_manifest", "source_map")
	}
	assertExactFields(t, envelope, fields)
	if stringField(t, envelope, "schema") != frontendCLISchema {
		t.Fatalf("wrong frontend schema")
	}
	if status == "ir-lowered" && stringField(t, envelope, "phase") != "emission" {
		t.Fatalf("success has wrong phase")
	}
}

func loadStrictObjectFile(t *testing.T, path string) map[string]jsonValue {
	t.Helper()
	return mustStrictObject(t, mustReadFile(t, path))
}

func mustStrictObject(t *testing.T, input []byte) map[string]jsonValue {
	t.Helper()
	value, err := decodeStrictJSON(input)
	if err != nil {
		t.Fatalf("strict JSON: %v", err)
	}
	return asObject(t, value, "JSON root")
}

func asObject(t *testing.T, value jsonValue, field string) map[string]jsonValue {
	t.Helper()
	object, ok := value.(map[string]jsonValue)
	if !ok {
		t.Fatalf("%s is %T, want object", field, value)
	}
	return object
}

func objectField(t *testing.T, object map[string]jsonValue, field string) map[string]jsonValue {
	t.Helper()
	return asObject(t, requiredField(t, object, field), field)
}

func arrayField(t *testing.T, object map[string]jsonValue, field string) []jsonValue {
	t.Helper()
	value, ok := requiredField(t, object, field).([]jsonValue)
	if !ok {
		t.Fatalf("%s is not an array", field)
	}
	return value
}

func stringArrayField(t *testing.T, object map[string]jsonValue, field string) []string {
	t.Helper()
	values := arrayField(t, object, field)
	strings := make([]string, len(values))
	for index, value := range values {
		text, ok := value.(string)
		if !ok {
			t.Fatalf("%s[%d] is not a string", field, index)
		}
		strings[index] = text
	}
	return strings
}

func stringField(t *testing.T, object map[string]jsonValue, field string) string {
	t.Helper()
	value, ok := requiredField(t, object, field).(string)
	if !ok {
		t.Fatalf("%s is not a string", field)
	}
	return value
}

func intField(t *testing.T, object map[string]jsonValue, field string) int64 {
	t.Helper()
	value, ok := requiredField(t, object, field).(int64)
	if !ok {
		t.Fatalf("%s is not an integer", field)
	}
	return value
}

func intArrayField(t *testing.T, object map[string]jsonValue, field string) []int64 {
	t.Helper()
	values := arrayField(t, object, field)
	integers := make([]int64, len(values))
	for index, value := range values {
		integer, ok := value.(int64)
		if !ok {
			t.Fatalf("%s[%d] is not an integer", field, index)
		}
		integers[index] = integer
	}
	return integers
}

func optionalString(object map[string]jsonValue, field string) (string, bool) {
	value, exists := object[field]
	if !exists {
		return "", false
	}
	text, ok := value.(string)
	return text, ok
}

func optionalInt(object map[string]jsonValue, field string) (int64, bool) {
	value, exists := object[field]
	if !exists {
		return 0, false
	}
	integer, ok := value.(int64)
	return integer, ok
}

func requiredField(t *testing.T, object map[string]jsonValue, field string) jsonValue {
	t.Helper()
	value, exists := object[field]
	if !exists {
		t.Fatalf("missing field %s", field)
	}
	return value
}

func findCase(t *testing.T, cases []jsonValue, id string) map[string]jsonValue {
	t.Helper()
	for _, value := range cases {
		object := asObject(t, value, "case")
		if stringField(t, object, "id") == id {
			return object
		}
	}
	t.Fatalf("missing case %s", id)
	return nil
}

func assertExactFields(t *testing.T, object map[string]jsonValue, expected []string) {
	t.Helper()
	actual := make([]string, 0, len(object))
	for field := range object {
		actual = append(actual, field)
	}
	sort.Strings(actual)
	sort.Strings(expected)
	if !reflect.DeepEqual(actual, expected) {
		t.Fatalf("fields = %v, want %v", actual, expected)
	}
}

func cloneJSONValue(t *testing.T, value jsonValue) jsonValue {
	t.Helper()
	canonical, err := canonicalJSONValue(value)
	if err != nil {
		t.Fatalf("clone canonicalize: %v", err)
	}
	clone, err := decodeStrictJSON(canonical)
	if err != nil {
		t.Fatalf("clone parse: %v", err)
	}
	return clone
}

func applyReplacePatches(t *testing.T, source jsonValue, patches []jsonValue) jsonValue {
	t.Helper()
	target := cloneJSONValue(t, source)
	for _, value := range patches {
		patch := asObject(t, value, "JSON patch")
		if operation := stringField(t, patch, "op"); operation != "replace" {
			t.Fatalf("unsupported JSON patch operation %q", operation)
		}
		path := stringField(t, patch, "path")
		replacement := cloneJSONValue(t, requiredField(t, patch, "value"))
		replaceJSONPointer(t, target, path, replacement)
	}
	return target
}

func replaceJSONPointer(t *testing.T, target jsonValue, pointer string, replacement jsonValue) {
	t.Helper()
	segments := pointerSegments(t, pointer)
	if len(segments) == 0 {
		t.Fatal("root replacement is not supported by these vectors")
	}
	parent := target
	for _, segment := range segments[:len(segments)-1] {
		parent = jsonPointerSegment(t, parent, pointer, segment)
	}
	last := segments[len(segments)-1]
	switch container := parent.(type) {
	case map[string]jsonValue:
		if _, exists := container[last]; !exists {
			t.Fatalf("pointer %s cannot replace missing object field %s", pointer, last)
		}
		container[last] = replacement
	case []jsonValue:
		index := jsonPointerIndex(t, pointer, last, len(container))
		container[index] = replacement
	default:
		t.Fatalf("pointer %s has scalar parent %T", pointer, parent)
	}
}

func jsonPointer(t *testing.T, value jsonValue, pointer string) jsonValue {
	t.Helper()
	if pointer == "" || pointer == "/" {
		return value
	}
	current := value
	for _, segment := range pointerSegments(t, pointer) {
		current = jsonPointerSegment(t, current, pointer, segment)
	}
	return current
}

func pointerSegments(t *testing.T, pointer string) []string {
	t.Helper()
	if pointer == "" {
		return nil
	}
	if !strings.HasPrefix(pointer, "/") {
		t.Fatalf("invalid JSON pointer %q", pointer)
	}
	rawSegments := strings.Split(strings.TrimPrefix(pointer, "/"), "/")
	segments := make([]string, len(rawSegments))
	for index, raw := range rawSegments {
		segments[index] = strings.ReplaceAll(strings.ReplaceAll(raw, "~1", "/"), "~0", "~")
	}
	return segments
}

func jsonPointerSegment(t *testing.T, current jsonValue, pointer, segment string) jsonValue {
	t.Helper()
	switch container := current.(type) {
	case map[string]jsonValue:
		value, exists := container[segment]
		if !exists {
			t.Fatalf("pointer %s missing object field %s", pointer, segment)
		}
		return value
	case []jsonValue:
		return container[jsonPointerIndex(t, pointer, segment, len(container))]
	default:
		t.Fatalf("pointer %s traverses scalar %T", pointer, current)
		return nil
	}
}

func jsonPointerIndex(t *testing.T, pointer, segment string, length int) int {
	t.Helper()
	index, err := strconv.Atoi(segment)
	if err != nil || index < 0 || index >= length || strconv.Itoa(index) != segment {
		t.Fatalf("pointer %s has invalid array index %s", pointer, segment)
	}
	return index
}

func assertVectorHash(t *testing.T, caseObject map[string]jsonValue, domain string, target jsonValue) {
	t.Helper()
	digest, err := hashCanonicalJSON(domain, target)
	if err != nil {
		t.Fatalf("%s hash: %v", stringField(t, caseObject, "id"), err)
	}
	if want := stringField(t, caseObject, "expected_sha256"); digest != want {
		t.Fatalf("%s digest = %s, want %s", stringField(t, caseObject, "id"), digest, want)
	}
}

func repoPath(path string) string {
	return filepath.Join("..", "..", filepath.FromSlash(path))
}

func mustReadFile(t *testing.T, path string) []byte {
	t.Helper()
	bytes, err := os.ReadFile(path)
	if err != nil {
		t.Fatalf("read %s: %v", path, err)
	}
	return bytes
}

func copyStrings(values []string) []string {
	return append([]string(nil), values...)
}

func copyBytes(values []byte) []byte {
	return append([]byte(nil), values...)
}

func fmtIndex(index int) string {
	text := strconv.Itoa(index)
	return strings.Repeat("0", 3-len(text)) + text
}

func assertUsageError(t *testing.T, args []string) {
	t.Helper()
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	if exitCode := run(args, &stdout, &stderr); exitCode != 2 {
		t.Fatalf("exit code = %d, want 2; stderr=%s", exitCode, stderr.String())
	}
	if stdout.Len() != 0 {
		t.Fatalf("configuration error stdout = %q, want zero bytes", stdout.String())
	}
}

func replaceArgument(args []string, index int, value string) []string {
	copy := copyStrings(args)
	copy[index] = value
	return copy
}

func replaceOptionValue(args []string, option, value string) []string {
	copy := copyStrings(args)
	for index := range copy {
		if copy[index] == option && index+1 < len(copy) {
			copy[index+1] = value
			return copy
		}
	}
	panic("missing fixture option " + option)
}

func removeOptionPair(args []string, option string) []string {
	for index := range args {
		if args[index] == option && index+1 < len(args) {
			copy := copyStrings(args)
			return append(copy[:index], copy[index+2:]...)
		}
	}
	panic("missing fixture option " + option)
}

func swapOptionPairs(args []string, left, right string) []string {
	copy := copyStrings(args)
	leftIndex := -1
	rightIndex := -1
	for index, value := range copy {
		if value == left {
			leftIndex = index
		}
		if value == right {
			rightIndex = index
		}
	}
	if leftIndex < 0 || rightIndex < 0 {
		panic("missing fixture option pair")
	}
	copy[leftIndex], copy[rightIndex] = copy[rightIndex], copy[leftIndex]
	copy[leftIndex+1], copy[rightIndex+1] = copy[rightIndex+1], copy[leftIndex+1]
	return copy
}
