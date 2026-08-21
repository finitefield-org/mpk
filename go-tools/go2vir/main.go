package main

import (
	"fmt"
	"io"
	"os"
	"regexp"
	"strings"
	"unicode/utf8"
)

const (
	usage = "usage: go2vir lower SOURCE_ROOT --package PACKAGE --semantic-profile PROFILE --target TARGET --function FUNCTION --frontend-bundle-id ID --frontend-sha256 SHA256 --release-registry-id ID --release-registry-sha256 SHA256 --toolchain-bundle-id ID --toolchain-root ROOT --toolchain-distribution-sha256 SHA256 [--contract PATH ...]\n"

	goSemanticProfile = "mpk.go.fixed.v0"
	goTarget          = "linux/amd64"
	goPointerWidth    = int64(64)
	logicalSourceRoot = "/mpk/source"
	logicalToolchain  = "/mpk/toolchain"
	registryID        = "mpk.release.registry.v0"

	maximumContracts     = 128
	maximumArgumentBytes = 262_144
)

var (
	idPattern       = regexp.MustCompile(`^[a-z0-9]+(?:[._-][a-z0-9]+)*$`)
	unitSegment     = regexp.MustCompile(`^[A-Za-z0-9_][A-Za-z0-9._~-]*$`)
	asciiIdentifier = regexp.MustCompile(`^[A-Za-z_][A-Za-z0-9_]*$`)
	sha256Pattern   = regexp.MustCompile(`^[0-9a-f]{64}$`)
)

type lowerRequest struct {
	SourceRoot                  string
	Package                     string
	SemanticProfile             string
	Target                      string
	Function                    string
	FrontendBundleID            string
	FrontendSHA256              string
	ReleaseRegistryID           string
	ReleaseRegistrySHA256       string
	ToolchainBundleID           string
	ToolchainRoot               string
	ToolchainDistributionSHA256 string
	Contracts                   []string
}

type namedArgument struct {
	name  string
	apply func(*lowerRequest, string)
}

func main() {
	os.Exit(run(os.Args[1:], os.Stdout, os.Stderr))
}

func run(args []string, stdout io.Writer, stderr io.Writer) int {
	if len(args) == 1 && (args[0] == "-h" || args[0] == "--help") {
		_, _ = io.WriteString(stdout, usage)
		return 0
	}

	request, err := parseLowerArguments(args)
	if err != nil {
		writeUsageError(stderr, err)
		return 2
	}

	// GO-VIR-02-T03 provides the private validated-selection/capture pipeline.
	// GO-VIR-02-T05 supplies its production registered resolver; until then no
	// unregistered filesystem candidate may enter the released command path.
	envelope := newFrontendErrorEnvelope(
		request,
		"capture",
		"GO_FRONTEND_INTERNAL",
		"registered Go release selection is not installed",
	)
	exitCode, err := writeNonSuccessEnvelope(stdout, request, envelope)
	if err != nil {
		_, _ = fmt.Fprintf(stderr, "go2vir protocol emission failed: %v\n", err)
		return 1
	}
	return exitCode
}

func parseLowerArguments(args []string) (lowerRequest, error) {
	request := lowerRequest{Contracts: make([]string, 0)}
	if err := validateArgumentTransport(args); err != nil {
		return request, err
	}
	if len(args) < 24 || args[0] != "lower" {
		return request, fmt.Errorf("go2vir requires the exact lower command")
	}
	request.SourceRoot = args[1]
	expected := []namedArgument{
		{name: "--package", apply: func(r *lowerRequest, value string) { r.Package = value }},
		{name: "--semantic-profile", apply: func(r *lowerRequest, value string) { r.SemanticProfile = value }},
		{name: "--target", apply: func(r *lowerRequest, value string) { r.Target = value }},
		{name: "--function", apply: func(r *lowerRequest, value string) { r.Function = value }},
		{name: "--frontend-bundle-id", apply: func(r *lowerRequest, value string) { r.FrontendBundleID = value }},
		{name: "--frontend-sha256", apply: func(r *lowerRequest, value string) { r.FrontendSHA256 = value }},
		{name: "--release-registry-id", apply: func(r *lowerRequest, value string) { r.ReleaseRegistryID = value }},
		{name: "--release-registry-sha256", apply: func(r *lowerRequest, value string) { r.ReleaseRegistrySHA256 = value }},
		{name: "--toolchain-bundle-id", apply: func(r *lowerRequest, value string) { r.ToolchainBundleID = value }},
		{name: "--toolchain-root", apply: func(r *lowerRequest, value string) { r.ToolchainRoot = value }},
		{name: "--toolchain-distribution-sha256", apply: func(r *lowerRequest, value string) { r.ToolchainDistributionSHA256 = value }},
	}
	position := 2
	for _, argument := range expected {
		if position+1 >= len(args) || args[position] != argument.name {
			return lowerRequest{}, fmt.Errorf("expected %s in the closed launcher order", argument.name)
		}
		if args[position+1] == "" {
			return lowerRequest{}, fmt.Errorf("%s requires a nonempty value", argument.name)
		}
		argument.apply(&request, args[position+1])
		position += 2
	}
	for position < len(args) {
		if position+1 >= len(args) || args[position] != "--contract" {
			return lowerRequest{}, fmt.Errorf("only repeatable --contract pairs may follow the required arguments")
		}
		request.Contracts = append(request.Contracts, args[position+1])
		position += 2
	}
	if err := validateLowerRequest(request); err != nil {
		return lowerRequest{}, err
	}
	return request, nil
}

func validateArgumentTransport(args []string) error {
	total := 0
	for _, argument := range args {
		if !utf8.ValidString(argument) {
			return fmt.Errorf("arguments must be valid UTF-8")
		}
		if len(argument) >= maximumArgumentBytes || total > maximumArgumentBytes-len(argument)-1 {
			return fmt.Errorf("launcher arguments exceed the byte limit")
		}
		total += len(argument) + 1
	}
	return nil
}

func validateLowerRequest(request lowerRequest) error {
	if request.SourceRoot != logicalSourceRoot {
		return fmt.Errorf("SOURCE_ROOT must be %s", logicalSourceRoot)
	}
	if request.SemanticProfile != goSemanticProfile {
		return fmt.Errorf("--semantic-profile must be %s", goSemanticProfile)
	}
	if request.Target != goTarget {
		return fmt.Errorf("--target must be %s", goTarget)
	}
	if err := validateGoSelection(request.Package, request.Function); err != nil {
		return err
	}
	if err := validateID("--frontend-bundle-id", request.FrontendBundleID); err != nil {
		return err
	}
	if err := validateDigest("--frontend-sha256", request.FrontendSHA256); err != nil {
		return err
	}
	if request.ReleaseRegistryID != registryID {
		return fmt.Errorf("--release-registry-id must be %s", registryID)
	}
	if err := validateDigest("--release-registry-sha256", request.ReleaseRegistrySHA256); err != nil {
		return err
	}
	if err := validateID("--toolchain-bundle-id", request.ToolchainBundleID); err != nil {
		return err
	}
	if request.ToolchainRoot != logicalToolchain {
		return fmt.Errorf("--toolchain-root must be %s", logicalToolchain)
	}
	if err := validateDigest("--toolchain-distribution-sha256", request.ToolchainDistributionSHA256); err != nil {
		return err
	}
	return validateContractPaths(request.Contracts)
}

func validateGoSelection(packageID, functionID string) error {
	if !validGoUnitID(packageID) || strings.Contains(packageID, "...") {
		return fmt.Errorf("--package must be a canonical Go import path")
	}
	switch packageID {
	case "main", "all", "std", "cmd":
		return fmt.Errorf("--package uses a reserved Go package pattern")
	}
	prefix := packageID + "."
	if !strings.HasPrefix(functionID, prefix) || len(functionID) > 1024 {
		return fmt.Errorf("--function must belong to --package")
	}
	declaration := strings.TrimPrefix(functionID, prefix)
	parts := strings.Split(declaration, ".")
	if len(parts) < 1 || len(parts) > 2 {
		return fmt.Errorf("--function must be a canonical Go function or value-method ID")
	}
	for _, part := range parts {
		if !validASCIIIdentifier(part) {
			return fmt.Errorf("--function contains an invalid declaration name")
		}
	}
	return nil
}

func validGoUnitID(value string) bool {
	if value == "" || len(value) > 1024 || strings.HasPrefix(value, "/") || strings.HasSuffix(value, "/") || strings.Contains(value, `\`) || strings.Contains(value, "://") {
		return false
	}
	for _, segment := range strings.Split(value, "/") {
		if segment == "." || segment == ".." || !unitSegment.MatchString(segment) {
			return false
		}
	}
	return true
}

func validASCIIIdentifier(value string) bool {
	return value != "_" && len(value) <= 255 && asciiIdentifier.MatchString(value)
}

func validateID(name, value string) error {
	if len(value) < 1 || len(value) > 128 || !idPattern.MatchString(value) {
		return fmt.Errorf("%s has an invalid release identifier", name)
	}
	return nil
}

func validateDigest(name, value string) error {
	if !sha256Pattern.MatchString(value) {
		return fmt.Errorf("%s must be 64 lowercase hexadecimal characters", name)
	}
	return nil
}

func validateContractPaths(paths []string) error {
	if len(paths) > maximumContracts {
		return fmt.Errorf("too many --contract arguments")
	}
	previous := ""
	seenFolded := make(map[string]struct{}, len(paths))
	for index, path := range paths {
		if !validPortablePath(path) {
			return fmt.Errorf("--contract path %d is not portable", index)
		}
		if index > 0 && previous >= path {
			return fmt.Errorf("--contract paths must be strictly sorted by bytes")
		}
		folded := strings.ToLower(path)
		if _, exists := seenFolded[folded]; exists {
			return fmt.Errorf("--contract paths collide under ASCII case folding")
		}
		seenFolded[folded] = struct{}{}
		previous = path
	}
	return nil
}

func validPortablePath(path string) bool {
	if path == "" || len(path) > 1024 || strings.HasPrefix(path, "/") || strings.HasSuffix(path, "/") || strings.ContainsAny(path, `\:`) {
		return false
	}
	for _, component := range strings.Split(path, "/") {
		if component == "" || len(component) > 255 || component == "." || component == ".." || strings.HasSuffix(component, ".") || windowsDeviceName(component) {
			return false
		}
		for _, character := range []byte(component) {
			if !((character >= 'A' && character <= 'Z') || (character >= 'a' && character <= 'z') || (character >= '0' && character <= '9') || character == '.' || character == '_' || character == '-') {
				return false
			}
		}
	}
	return true
}

func windowsDeviceName(component string) bool {
	base := strings.ToUpper(strings.SplitN(component, ".", 2)[0])
	if base == "CON" || base == "PRN" || base == "AUX" || base == "NUL" {
		return true
	}
	if len(base) == 4 && (strings.HasPrefix(base, "COM") || strings.HasPrefix(base, "LPT")) && base[3] >= '1' && base[3] <= '9' {
		return true
	}
	return false
}

func writeUsageError(stderr io.Writer, err error) {
	_, _ = io.WriteString(stderr, usage)
	_, _ = fmt.Fprintf(stderr, "go2vir: %v\n", err)
}
