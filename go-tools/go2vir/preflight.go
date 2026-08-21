package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"io"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"

	"golang.org/x/mod/modfile"
)

const (
	maximumCandidateEntries   = 32_768
	maximumManifestInputs     = 32_768
	maximumCandidateBytes     = 16_777_216
	maximumCapturedBytes      = 268_435_456
	maximumContractBytes      = 1_048_576
	maximumTotalContractBytes = 8_388_608
	maximumVisitedDirectories = 32_768
	maximumDirectoryEntries   = 131_072
	maximumTypedSyntaxNodes   = 1_000_000
	goModuleVersion           = "1.23"
	buildManifestInputKind    = "build_manifest"
	lockfileInputKind         = "lockfile"
	sourceInputKind           = "source"
	contractInputKind         = "contract"
)

var auxiliarySuffixes = []string{
	".c", ".cc", ".cpp", ".cxx", ".m", ".h", ".hh", ".hpp", ".hxx",
	".f", ".F", ".for", ".f90", ".s", ".S", ".sx", ".swig", ".swigcxx", ".syso",
}

var constrainedFilenameSuffixes = []string{
	"_aix", "_android", "_darwin", "_dragonfly", "_freebsd", "_hurd", "_illumos",
	"_ios", "_js", "_linux", "_netbsd", "_openbsd", "_plan9", "_solaris", "_wasip1", "_windows",
	"_386", "_amd64", "_amd64p32", "_arm", "_arm64", "_loong64", "_mips", "_mips64",
	"_mips64le", "_mipsle", "_ppc64", "_ppc64le", "_riscv64", "_s390x", "_sparc64", "_wasm",
}

type frontendFailure struct {
	Status  string
	Phase   string
	Code    string
	Message string
	cause   error
}

func (failure *frontendFailure) Error() string {
	return failure.Code + ": " + failure.Message
}

func fail(status, phase, code, message string) error {
	return &frontendFailure{Status: status, Phase: phase, Code: code, Message: message}
}

func failWithCause(status, phase, code, message string, cause error) error {
	return &frontendFailure{Status: status, Phase: phase, Code: code, Message: message, cause: cause}
}

type capturedInput struct {
	Kind           string
	NormalizedPath string
	Bytes          []byte
	SHA256         string
	state          capturedFileState
}

type capturedFileState struct {
	absolutePath string
	info         os.FileInfo
	modTimeNanos int64
}

type directoryEntryState struct {
	name string
	kind os.FileMode
}

type capturedDirectoryState struct {
	absolutePath string
	info         os.FileInfo
	entries      []directoryEntryState
}

type capturedPackage struct {
	ImportPath string
	Name       string
	Directory  string
	Sources    []string
	Imports    []string
}

type sourceCapture struct {
	ModulePath      string
	SelectedPackage string
	Inputs          []capturedInput
	Packages        []capturedPackage
}

type captureObserver interface {
	afterInitialCapture() error
}

type captureObserverFunc func() error

func (function captureObserverFunc) afterInitialCapture() error {
	return function()
}

type captureCounters struct {
	candidates         uint64
	manifestInputs     uint64
	capturedBytes      uint64
	contractCandidates uint64
	contractBytes      uint64
	directories        uint64
	directoryEntries   uint64
}

func (counters *captureCounters) addDirectory(entryCount int) error {
	if counters.directories >= maximumVisitedDirectories {
		return fail("rejected", "capture", "GO_LIMIT_INPUTS", "visited directory limit exceeded")
	}
	entries := uint64(entryCount)
	if entries > maximumDirectoryEntries-counters.directoryEntries {
		return fail("rejected", "capture", "GO_LIMIT_INPUTS", "directory entry limit exceeded")
	}
	counters.directories++
	counters.directoryEntries += entries
	return nil
}

func (counters *captureCounters) admitCandidate(size int64, contract, prospectiveInput bool) error {
	if size < 0 || size > maximumCandidateBytes {
		return fail("rejected", "capture", "GO_LIMIT_INPUT_BYTES", "captured candidate exceeds the byte limit")
	}
	if counters.candidates >= maximumCandidateEntries {
		return fail("rejected", "capture", "GO_LIMIT_INPUTS", "loader candidate limit exceeded")
	}
	if uint64(size) > maximumCapturedBytes-counters.capturedBytes {
		return fail("rejected", "capture", "GO_LIMIT_INPUT_BYTES", "total captured bytes exceed the limit")
	}
	if prospectiveInput && counters.manifestInputs >= maximumManifestInputs {
		return fail("rejected", "capture", "GO_LIMIT_INPUTS", "manifest input limit exceeded")
	}
	if contract {
		if size > maximumContractBytes || counters.contractBytes > maximumTotalContractBytes-uint64(size) {
			return fail("rejected", "capture", "GO_LIMIT_INPUT_BYTES", "contract candidate bytes exceed the limit")
		}
		if counters.contractCandidates >= maximumContracts {
			return fail("rejected", "capture", "GO_LIMIT_INPUTS", "contract candidate limit exceeded")
		}
	}
	counters.candidates++
	counters.capturedBytes += uint64(size)
	if prospectiveInput {
		counters.manifestInputs++
	}
	if contract {
		counters.contractCandidates++
		counters.contractBytes += uint64(size)
	}
	return nil
}

type captureBuilder struct {
	root                string
	request             lowerRequest
	counters            captureCounters
	directories         map[string]capturedDirectoryState
	inputs              map[string]capturedInput
	packages            map[string]capturedPackage
	discoveredContracts map[string]struct{}
}

func captureSourceTree(root string, request lowerRequest) (sourceCapture, error) {
	return captureSourceTreeObserved(root, request, nil)
}

func captureSourceTreeObserved(root string, request lowerRequest, observer captureObserver) (sourceCapture, error) {
	absoluteRoot, err := filepath.Abs(root)
	if err != nil {
		return sourceCapture{}, fail("rejected", "capture", "GO_CAPTURE_PATH", "source root cannot be normalized")
	}
	rootInfo, err := os.Lstat(absoluteRoot)
	if err != nil || rootInfo.Mode()&os.ModeSymlink != 0 || !rootInfo.IsDir() {
		return sourceCapture{}, fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", "source root is not a regular directory capability")
	}
	builder := &captureBuilder{
		root:                absoluteRoot,
		request:             request,
		directories:         make(map[string]capturedDirectoryState),
		inputs:              make(map[string]capturedInput),
		packages:            make(map[string]capturedPackage),
		discoveredContracts: make(map[string]struct{}),
	}
	rootEntries, err := builder.visitDirectory("")
	if err != nil {
		return sourceCapture{}, err
	}
	if vendor, exists := rootEntries["vendor"]; exists && vendor.kind.IsDir() {
		return sourceCapture{}, fail("rejected", "metadata", "GO_MODULE_POLICY", "vendor directory is forbidden")
	}
	if vendor, exists := rootEntries["vendor"]; exists && vendor.kind&os.ModeSymlink != 0 {
		return sourceCapture{}, fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", "vendor path is a symbolic link")
	}
	for _, workspace := range []string{"go.work", "go.work.sum"} {
		if entry, exists := rootEntries[workspace]; exists {
			if !entry.kind.IsRegular() {
				return sourceCapture{}, fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", workspace+" is not a regular file")
			}
			if _, err := builder.captureCandidate(workspace, buildManifestInputKind, false, false); err != nil {
				return sourceCapture{}, err
			}
			return sourceCapture{}, fail("rejected", "metadata", "GO_WORKSPACE_FORBIDDEN", "Go workspace files are forbidden")
		}
	}
	moduleEntry, exists := rootEntries["go.mod"]
	if !exists {
		return sourceCapture{}, fail("source-error", "capture", "GO_MODULE_MISSING", "root go.mod is required")
	}
	if !moduleEntry.kind.IsRegular() {
		return sourceCapture{}, fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", "go.mod is not a regular file")
	}
	moduleInput, err := builder.captureCandidate("go.mod", buildManifestInputKind, false, true)
	if err != nil {
		return sourceCapture{}, err
	}
	modulePath, err := validateModuleFile(moduleInput.Bytes, request.Package)
	if err != nil {
		return sourceCapture{}, err
	}
	if sumEntry, exists := rootEntries["go.sum"]; exists {
		if !sumEntry.kind.IsRegular() {
			return sourceCapture{}, fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", "go.sum is not a regular file")
		}
		sum, err := builder.captureCandidate("go.sum", lockfileInputKind, false, true)
		if err != nil {
			return sourceCapture{}, err
		}
		if len(sum.Bytes) != 0 {
			return sourceCapture{}, fail("rejected", "metadata", "GO_MODULE_DEPENDENCY", "go.sum must be empty")
		}
	}

	worklist := []string{request.Package}
	queued := map[string]struct{}{request.Package: {}}
	for len(worklist) > 0 {
		sort.Strings(worklist)
		importPath := worklist[0]
		worklist = worklist[1:]
		if _, loaded := builder.packages[importPath]; loaded {
			continue
		}
		packageRecord, err := builder.capturePackage(modulePath, importPath)
		if err != nil {
			return sourceCapture{}, err
		}
		builder.packages[importPath] = packageRecord
		for _, dependency := range packageRecord.Imports {
			if _, exists := queued[dependency]; !exists {
				queued[dependency] = struct{}{}
				worklist = append(worklist, dependency)
			}
		}
	}

	discovered := make([]string, 0, len(builder.discoveredContracts))
	for path := range builder.discoveredContracts {
		discovered = append(discovered, path)
	}
	sort.Strings(discovered)
	if !equalStrings(discovered, request.Contracts) {
		return sourceCapture{}, fail("rejected", "subset", "GO_CONTRACT_FUNCTION", "explicit contract paths do not equal discovered contract candidates")
	}
	inputPaths := make([]string, 0, len(builder.inputs))
	for path := range builder.inputs {
		inputPaths = append(inputPaths, path)
	}
	if err := validateCapturedPathUniqueness(inputPaths); err != nil {
		return sourceCapture{}, err
	}
	if observer != nil {
		if err := observer.afterInitialCapture(); err != nil {
			return sourceCapture{}, err
		}
	}
	if err := builder.revalidate(); err != nil {
		return sourceCapture{}, err
	}

	inputs := make([]capturedInput, 0, len(builder.inputs))
	for _, input := range builder.inputs {
		input.state = capturedFileState{}
		inputs = append(inputs, input)
	}
	sort.Slice(inputs, func(left, right int) bool {
		if inputs[left].NormalizedPath != inputs[right].NormalizedPath {
			return inputs[left].NormalizedPath < inputs[right].NormalizedPath
		}
		return inputs[left].Kind < inputs[right].Kind
	})
	packages := make([]capturedPackage, 0, len(builder.packages))
	for _, packageRecord := range builder.packages {
		packages = append(packages, packageRecord)
	}
	sort.Slice(packages, func(left, right int) bool { return packages[left].ImportPath < packages[right].ImportPath })
	return sourceCapture{ModulePath: modulePath, SelectedPackage: request.Package, Inputs: inputs, Packages: packages}, nil
}

func validateModuleFile(content []byte, selectedPackage string) (string, error) {
	if len(content) == 0 {
		return "", fail("source-error", "capture", "GO_MODULE_INVALID", "go.mod must be nonempty")
	}
	parsed, err := modfile.Parse("go.mod", content, nil)
	if err != nil {
		return "", fail("source-error", "capture", "GO_MODULE_INVALID", "go.mod is malformed")
	}
	if parsed.Module == nil || parsed.Module.Mod.Path == "" || parsed.Go == nil || parsed.Go.Version != goModuleVersion {
		return "", fail("source-error", "capture", "GO_MODULE_INVALID", "go.mod requires one module directive and go "+goModuleVersion)
	}
	if parsed.Toolchain != nil || len(parsed.Godebug) != 0 || len(parsed.Require) != 0 || len(parsed.Exclude) != 0 || len(parsed.Replace) != 0 || len(parsed.Retract) != 0 || len(parsed.Tool) != 0 || len(parsed.Ignore) != 0 {
		return "", fail("rejected", "metadata", "GO_MODULE_POLICY", "go.mod contains a forbidden directive")
	}
	modulePath := parsed.Module.Mod.Path
	if !validGoUnitID(modulePath) || strings.Contains(modulePath, "...") || oneOf(modulePath, "main", "all", "std", "cmd") {
		return "", fail("source-error", "capture", "GO_MODULE_INVALID", "module path is not canonical")
	}
	if selectedPackage != modulePath && !strings.HasPrefix(selectedPackage, modulePath+"/") {
		return "", fail("source-error", "capture", "GO_PACKAGE_MISSING", "selected package is outside the root module")
	}
	return modulePath, nil
}

func (builder *captureBuilder) capturePackage(modulePath, importPath string) (capturedPackage, error) {
	if importPath != modulePath && !strings.HasPrefix(importPath, modulePath+"/") {
		return capturedPackage{}, fail("rejected", "source", "GO_SUBSET_IMPORT", "external and standard-library imports are forbidden")
	}
	relative := strings.TrimPrefix(importPath, modulePath)
	relative = strings.TrimPrefix(relative, "/")
	if relative != "" && !validPortablePath(relative) {
		return capturedPackage{}, fail("rejected", "capture", "GO_CAPTURE_PATH", "selected package directory is not portable")
	}
	if err := builder.visitPathComponents(relative); err != nil {
		return capturedPackage{}, err
	}
	entries, err := builder.visitDirectory(relative)
	if err != nil {
		return capturedPackage{}, err
	}

	names := make([]string, 0, len(entries))
	for name := range entries {
		names = append(names, name)
	}
	sort.Strings(names)
	packageName := ""
	sources := make([]string, 0)
	imports := make(map[string]struct{})
	for _, name := range names {
		entry := entries[name]
		normalizedPath := joinPortable(relative, name)
		switch {
		case strings.HasSuffix(name, "_test.go"):
			continue
		case strings.HasSuffix(name, ".go"):
			if !entry.kind.IsRegular() {
				return capturedPackage{}, fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", "Go candidate is not a regular file")
			}
			input, captureErr := builder.captureCandidate(normalizedPath, sourceInputKind, false, true)
			if captureErr != nil {
				return capturedPackage{}, captureErr
			}
			if strings.HasPrefix(name, ".") || strings.HasPrefix(name, "_") || constrainedGoFilename(name) {
				return capturedPackage{}, fail("rejected", "source", "GO_BUILD_CONSTRAINT", "Go filename uses a forbidden selection rule")
			}
			parsedName, parsedImports, parseErr := parseCapturedGoFile(normalizedPath, input.Bytes)
			if parseErr != nil {
				return capturedPackage{}, parseErr
			}
			if packageName == "" {
				packageName = parsedName
			} else if packageName != parsedName {
				return capturedPackage{}, fail("source-error", "source", "GO_SOURCE_PARSE", "package clauses disagree")
			}
			for _, dependency := range parsedImports {
				if dependency != modulePath && !strings.HasPrefix(dependency, modulePath+"/") {
					return capturedPackage{}, fail("rejected", "source", "GO_SUBSET_IMPORT", "external and standard-library imports are forbidden")
				}
				imports[dependency] = struct{}{}
			}
			sources = append(sources, normalizedPath)
		case auxiliaryCandidate(name):
			if !entry.kind.IsRegular() {
				return capturedPackage{}, fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", "compiler auxiliary candidate is not a regular file")
			}
			if _, captureErr := builder.captureCandidate(normalizedPath, sourceInputKind, false, false); captureErr != nil {
				return capturedPackage{}, captureErr
			}
			return capturedPackage{}, fail("rejected", "source", "GO_CGO_OR_AUX_SOURCE", "compiler auxiliary sources are forbidden")
		case contractCandidate(name):
			if !entry.kind.IsRegular() {
				return capturedPackage{}, fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", "contract candidate is not a regular file")
			}
			if _, captureErr := builder.captureCandidate(normalizedPath, contractInputKind, true, true); captureErr != nil {
				return capturedPackage{}, captureErr
			}
			builder.discoveredContracts[normalizedPath] = struct{}{}
		}
	}
	if len(sources) == 0 {
		return capturedPackage{}, fail("source-error", "capture", "GO_PACKAGE_MISSING", "selected package contains no Go source")
	}
	if !validASCIIIdentifier(packageName) {
		return capturedPackage{}, fail("source-error", "source", "GO_SOURCE_PARSE", "package name is not accepted")
	}
	sort.Strings(sources)
	importList := make([]string, 0, len(imports))
	for dependency := range imports {
		importList = append(importList, dependency)
	}
	sort.Strings(importList)
	return capturedPackage{ImportPath: importPath, Name: packageName, Directory: relative, Sources: sources, Imports: importList}, nil
}

func parseCapturedGoFile(path string, content []byte) (string, []string, error) {
	packageOnly, err := parser.ParseFile(token.NewFileSet(), path, content, parser.PackageClauseOnly)
	if err != nil {
		return "", nil, fail("source-error", "source", "GO_SOURCE_PARSE", "Go source is malformed")
	}
	if packageOnly.Name.Name == "documentation" {
		return "", nil, fail("rejected", "source", "GO_BUILD_CONSTRAINT", "package documentation is forbidden")
	}
	file, err := parser.ParseFile(token.NewFileSet(), path, content, parser.ParseComments)
	if err != nil {
		return "", nil, fail("source-error", "source", "GO_SOURCE_PARSE", "Go source is malformed")
	}
	for _, commentGroup := range file.Comments {
		for _, comment := range commentGroup.List {
			text := comment.Text
			switch {
			case strings.HasPrefix(text, "//go:build"), strings.HasPrefix(text, "// +build"):
				return "", nil, fail("rejected", "source", "GO_BUILD_CONSTRAINT", "build constraints are forbidden")
			case strings.HasPrefix(text, "//go:"), strings.HasPrefix(text, "//line "), strings.HasPrefix(text, "/*line "):
				return "", nil, fail("rejected", "source", "GO_SOURCE_DIRECTIVE", "source directives are forbidden")
			}
		}
	}
	imports := make([]string, 0, len(file.Imports))
	for _, importSpec := range file.Imports {
		if importSpec.Name != nil && (importSpec.Name.Name == "." || importSpec.Name.Name == "_" || !validASCIIIdentifier(importSpec.Name.Name)) {
			return "", nil, fail("rejected", "source", "GO_SUBSET_IMPORT", "special or invalid import aliases are forbidden")
		}
		path, unquoteErr := strconv.Unquote(importSpec.Path.Value)
		if unquoteErr != nil || !validGoUnitID(path) {
			return "", nil, fail("source-error", "source", "GO_SOURCE_PARSE", "import path is malformed")
		}
		if oneOf(path, "C", "unsafe", "reflect") {
			return "", nil, fail("rejected", "source", "GO_SUBSET_IMPORT", "special Go imports are forbidden")
		}
		imports = append(imports, path)
	}
	sort.Strings(imports)
	return file.Name.Name, imports, nil
}

func (builder *captureBuilder) visitPathComponents(relative string) error {
	if relative == "" {
		return nil
	}
	components := strings.Split(relative, "/")
	for index := range components {
		path := strings.Join(components[:index+1], "/")
		entries, err := builder.visitDirectory(path)
		if err != nil {
			return err
		}
		if entry, exists := entries["go.mod"]; exists {
			if !entry.kind.IsRegular() {
				return fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", "nested go.mod is not a regular file")
			}
			if _, err := builder.captureCandidate(joinPortable(path, "go.mod"), buildManifestInputKind, false, false); err != nil {
				return err
			}
			return fail("rejected", "metadata", "GO_MODULE_POLICY", "selected import closure crosses a nested module")
		}
	}
	return nil
}

func (builder *captureBuilder) visitDirectory(relative string) (map[string]directoryEntryState, error) {
	if state, exists := builder.directories[relative]; exists {
		return directoryStateMap(state.entries), nil
	}
	if relative != "" && !validPortablePath(relative) {
		return nil, fail("rejected", "capture", "GO_CAPTURE_PATH", "visited directory path is not portable")
	}
	absolute := builder.root
	if relative != "" {
		absolute = filepath.Join(builder.root, filepath.FromSlash(relative))
	}
	info, err := os.Lstat(absolute)
	if err != nil {
		return nil, fail("source-error", "capture", "GO_PACKAGE_MISSING", "package directory is missing")
	}
	if info.Mode()&os.ModeSymlink != 0 || !info.IsDir() {
		return nil, fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", "visited package path is not a regular directory")
	}
	if builder.counters.directories >= maximumVisitedDirectories {
		return nil, fail("rejected", "capture", "GO_LIMIT_INPUTS", "visited directory limit exceeded")
	}
	remainingEntries := int(maximumDirectoryEntries - builder.counters.directoryEntries)
	entries, overflow, err := readDirectoryState(absolute, remainingEntries)
	if err != nil {
		return nil, fail("frontend-error", "capture", "GO_FRONTEND_SANDBOX", "package directory cannot be enumerated")
	}
	if overflow {
		return nil, fail("rejected", "capture", "GO_LIMIT_INPUTS", "directory entry limit exceeded")
	}
	if err := builder.counters.addDirectory(len(entries)); err != nil {
		return nil, err
	}
	builder.directories[relative] = capturedDirectoryState{absolutePath: absolute, info: info, entries: entries}
	return directoryStateMap(entries), nil
}

func readDirectoryState(path string, maximumEntries int) ([]directoryEntryState, bool, error) {
	directory, err := os.Open(path)
	if err != nil {
		return nil, false, err
	}
	defer directory.Close()
	result := make([]directoryEntryState, 0, min(maximumEntries, 256))
	for {
		remaining := maximumEntries - len(result)
		batchSize := min(remaining, 256)
		if batchSize == 0 {
			batchSize = 1
		}
		entries, readErr := directory.ReadDir(batchSize)
		if len(entries) > remaining {
			return nil, true, nil
		}
		for _, entry := range entries {
			info, statErr := os.Lstat(filepath.Join(path, entry.Name()))
			if statErr != nil {
				return nil, false, statErr
			}
			result = append(result, directoryEntryState{name: entry.Name(), kind: info.Mode().Type()})
		}
		if readErr == io.EOF {
			break
		}
		if readErr != nil {
			return nil, false, readErr
		}
		if len(entries) == 0 {
			return nil, false, fmt.Errorf("directory enumeration made no progress")
		}
	}
	sort.Slice(result, func(left, right int) bool { return result[left].name < result[right].name })
	return result, false, nil
}

func directoryStateMap(entries []directoryEntryState) map[string]directoryEntryState {
	result := make(map[string]directoryEntryState, len(entries))
	for _, entry := range entries {
		result[entry.name] = entry
	}
	return result
}

func (builder *captureBuilder) captureCandidate(path, kind string, contract, prospectiveInput bool) (capturedInput, error) {
	if existing, exists := builder.inputs[path]; exists {
		return existing, nil
	}
	if !validPortablePath(path) {
		return capturedInput{}, fail("rejected", "capture", "GO_CAPTURE_PATH", "candidate path is not portable")
	}
	absolute := filepath.Join(builder.root, filepath.FromSlash(path))
	before, err := os.Lstat(absolute)
	if err != nil || before.Mode()&os.ModeSymlink != 0 || !before.Mode().IsRegular() {
		return capturedInput{}, fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", "candidate is not a regular file")
	}
	if err := builder.counters.admitCandidate(before.Size(), contract, prospectiveInput); err != nil {
		return capturedInput{}, err
	}
	file, err := openRegularNoFollow(absolute)
	if err != nil {
		return capturedInput{}, fail("rejected", "capture", "GO_CAPTURE_FILE_KIND", "candidate cannot be opened without following links")
	}
	defer file.Close()
	opened, err := file.Stat()
	if err != nil || !opened.Mode().IsRegular() || !os.SameFile(before, opened) {
		return capturedInput{}, fail("rejected", "capture", "GO_CAPTURE_CHANGED", "candidate identity changed while opening")
	}
	content, err := io.ReadAll(io.LimitReader(file, maximumCandidateBytes+1))
	if err != nil || len(content) > maximumCandidateBytes {
		return capturedInput{}, fail("rejected", "capture", "GO_LIMIT_INPUT_BYTES", "candidate cannot be captured within the byte limit")
	}
	after, err := file.Stat()
	if err != nil || after.Size() != int64(len(content)) || !sameFileState(opened, after) {
		return capturedInput{}, fail("rejected", "capture", "GO_CAPTURE_CHANGED", "candidate changed during capture")
	}
	input := capturedInput{
		Kind:           kind,
		NormalizedPath: path,
		Bytes:          append([]byte(nil), content...),
		SHA256:         sha256Hex(content),
		state: capturedFileState{
			absolutePath: absolute,
			info:         after,
			modTimeNanos: after.ModTime().UnixNano(),
		},
	}
	if prospectiveInput {
		builder.inputs[path] = input
	}
	return input, nil
}

func (builder *captureBuilder) revalidate() error {
	for _, directory := range builder.directories {
		info, err := os.Lstat(directory.absolutePath)
		if err != nil || info.Mode()&os.ModeSymlink != 0 || !info.IsDir() || !os.SameFile(directory.info, info) {
			return fail("rejected", "capture", "GO_CAPTURE_CHANGED", "visited directory identity changed during capture")
		}
		entries, overflow, err := readDirectoryState(directory.absolutePath, len(directory.entries))
		if err != nil || overflow || !equalDirectoryStates(entries, directory.entries) {
			return fail("rejected", "capture", "GO_CAPTURE_CHANGED", "visited directory namespace changed during capture")
		}
	}
	for _, input := range builder.inputs {
		info, err := os.Lstat(input.state.absolutePath)
		if err != nil || info.Mode()&os.ModeSymlink != 0 || !info.Mode().IsRegular() || !os.SameFile(input.state.info, info) || info.Size() != int64(len(input.Bytes)) || info.ModTime().UnixNano() != input.state.modTimeNanos {
			return fail("rejected", "capture", "GO_CAPTURE_CHANGED", "captured file changed before snapshot sealing")
		}
	}
	return nil
}

func sameFileState(left, right os.FileInfo) bool {
	return os.SameFile(left, right) && left.Size() == right.Size() && left.Mode() == right.Mode() && left.ModTime().UnixNano() == right.ModTime().UnixNano()
}

func equalDirectoryStates(left, right []directoryEntryState) bool {
	if len(left) != len(right) {
		return false
	}
	for index := range left {
		if left[index] != right[index] {
			return false
		}
	}
	return true
}

func auxiliaryCandidate(name string) bool {
	for _, suffix := range auxiliarySuffixes {
		if strings.HasSuffix(name, suffix) {
			return true
		}
	}
	return false
}

func contractCandidate(name string) bool {
	lower := strings.ToLower(name)
	return lower == "contract.json" || strings.HasSuffix(lower, ".contract.json") || strings.HasSuffix(lower, "_contract.json")
}

func constrainedGoFilename(name string) bool {
	base := strings.TrimSuffix(name, ".go")
	for _, suffix := range constrainedFilenameSuffixes {
		if strings.HasSuffix(base, suffix) {
			return true
		}
	}
	return false
}

func joinPortable(directory, name string) string {
	if directory == "" {
		return name
	}
	return directory + "/" + name
}

func equalStrings(left, right []string) bool {
	if len(left) != len(right) {
		return false
	}
	for index := range left {
		if left[index] != right[index] {
			return false
		}
	}
	return true
}

func validateCapturedPathUniqueness(paths []string) error {
	seenFolded := make(map[string]string, len(paths))
	for _, path := range paths {
		folded := strings.ToLower(path)
		if prior, collision := seenFolded[folded]; collision && prior != path {
			return fail("rejected", "capture", "GO_CAPTURE_PATH", "captured input paths collide under ASCII case folding")
		}
		seenFolded[folded] = path
	}
	return nil
}

func countSyntaxNodes(files []*ast.File, maximum uint64) error {
	count := uint64(0)
	for _, file := range files {
		for range ast.Preorder(file) {
			count++
			if count > maximum {
				return fail("rejected", "source", "GO_LIMIT_SYNTAX", "typed syntax node limit exceeded")
			}
		}
	}
	return nil
}
