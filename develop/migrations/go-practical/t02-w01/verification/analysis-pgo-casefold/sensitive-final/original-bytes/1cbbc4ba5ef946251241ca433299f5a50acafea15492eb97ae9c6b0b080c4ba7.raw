package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"maps"
	"path/filepath"
	"reflect"
	"strings"
	"sync"
)

// A live owned comparison baseline from actual preparation observations and
// the exact image's original archives. Matching data does not attest that a
// kernel executed a command or authorize a registered context/source proof.
type practicalReplayBaseline struct {
	mutex                                 sync.Mutex
	view                                  *practicalReplayView
	modules                               []map[string]jsonValue
	packages                              []map[string]jsonValue
	archives                              map[string]practicalSourceObject
	moduleDiagnostics, packageDiagnostics []byte
	imageDigest                           string
	failed                                error
	closed                                bool
}

type practicalReplayComparison struct {
	modules, packages, exports int
	bytes                      int64
	actual                     practicalSourceSnapshot
}

func decodePracticalCompleteObjects(raw []byte) ([]map[string]jsonValue, error) {
	if len(raw) > maximumCandidateBytes {
		return nil, fmt.Errorf("native observation exceeds its metadata byte bound")
	}
	decoder := json.NewDecoder(bytes.NewReader(raw))
	var result []map[string]jsonValue
	for {
		var encoded json.RawMessage
		if err := decoder.Decode(&encoded); err == io.EOF {
			break
		} else if err != nil {
			return nil, err
		}
		if len(result) >= maximumManifestInputs {
			return nil, fmt.Errorf("native observation exceeds its complete object count")
		}
		value, err := decodeStrictJSON(encoded)
		if err != nil {
			return nil, err
		}
		object, ok := value.(map[string]jsonValue)
		if !ok {
			return nil, fmt.Errorf("native observation contains a non-object")
		}
		result = append(result, object)
	}
	if len(result) == 0 {
		return nil, fmt.Errorf("native observation contains no complete objects")
	}
	return result, nil
}

func practicalOriginalCacheRelative(directory, name string) (string, error) {
	if !practicalReplayAbsolute(directory) || !practicalReplayAbsolute(name) {
		return "", fmt.Errorf("native export cache name is not an original absolute path")
	}
	relative, err := filepath.Rel(directory, name)
	if err != nil || !validPracticalLogicalPath(filepath.ToSlash(relative)) {
		return "", fmt.Errorf("native export escaped its authorized cache root")
	}
	return filepath.ToSlash(relative), nil
}

func (image *practicalReplayImage) originalReplayBaseline(discovery practicalModuleDiscovery, exports practicalPackageExports) (*practicalReplayBaseline, error) {
	if err := discovery.requireOriginalObservation(); err != nil {
		return nil, err
	}
	if err := exports.requireOriginalObservation(); err != nil {
		return nil, err
	}
	if discovery.compilerSHA256 != exports.compilerSHA256 || discovery.mainSourceID != exports.mainSourceID ||
		discovery.observation.compilerRoot != exports.observation.compilerRoot || discovery.observation.directory != exports.observation.directory ||
		discovery.observation.distributionSHA256 != exports.observation.distributionSHA256 {
		return nil, fmt.Errorf("original module and package observations crossed compiler/main ownership")
	}
	view, err := image.openVerifiedView()
	if err != nil {
		return nil, err
	}
	result := &practicalReplayBaseline{view: view, archives: map[string]practicalSourceObject{}, imageDigest: image.contentSHA256,
		moduleDiagnostics: bytes.Clone(discovery.observation.diagnostics), packageDiagnostics: bytes.Clone(exports.observation.diagnostics)}
	success := false
	defer func() {
		if !success {
			_ = result.Close()
		}
	}()
	if image.compiler == nil || image.compilerOriginal != discovery.observation.compilerRoot ||
		image.compiler.selection.Toolchain.DistributionSHA256 != exports.distributionSHA256 ||
		view.plan.origins["original-go-package-exports"] != exports.observation.buildCache {
		return nil, fmt.Errorf("original observations have no matching owned compiler/export image")
	}
	mainFound := false
	for _, object := range view.plan.captured {
		if object.ID == discovery.mainSourceID && object.Kind == "go_mod" {
			mainFound = filepath.Join(view.plan.origins[object.Capability], filepath.FromSlash(object.Path)) == filepath.Join(discovery.observation.directory, "go.mod")
		}
	}
	if !mainFound {
		return nil, fmt.Errorf("original observation lost its captured main manifest")
	}
	result.modules, err = decodePracticalCompleteObjects(discovery.observation.output)
	if err != nil {
		return nil, err
	}
	result.packages, err = decodePracticalCompleteObjects(exports.observation.output)
	if err != nil {
		return nil, err
	}
	seen := map[string]bool{}
	for _, row := range result.packages {
		name, ok := row["ImportPath"].(string)
		if !ok || name == "" || seen[name] {
			return nil, fmt.Errorf("original complete package observation has duplicate or invalid paths")
		}
		seen[name] = true
		file, exists := row["Export"]
		if !exists {
			if name != "unsafe" {
				return nil, fmt.Errorf("original package lost complete archive ownership")
			}
			continue
		}
		original, ok := file.(string)
		if !ok {
			return nil, fmt.Errorf("original export locator is not a string")
		}
		relative, err := practicalOriginalCacheRelative(exports.observation.buildCache, original)
		if err != nil {
			return nil, err
		}
		buffer, err := view.readCaptured("original-go-package-exports", relative, "export_data")
		if err != nil {
			return nil, err
		}
		buildID, ok := row["BuildID"].(string)
		if !ok || validatePracticalExportArchive(buffer.data, buildID) != nil {
			return nil, fmt.Errorf("original export image differs from the complete native archive/build ID")
		}
		result.archives[name] = buffer.object
	}
	count := 0
	for _, object := range view.plan.captured {
		if object.Capability == "original-go-package-exports" {
			count++
		}
	}
	if count != len(result.archives) {
		return nil, fmt.Errorf("original replay baseline has unbound archive inputs")
	}
	success = true
	return result, nil
}

func (baseline *practicalReplayBaseline) reject(err error) error {
	if baseline.failed == nil {
		baseline.failed = err
	}
	return baseline.failed
}

// The cache mapping is authorized by the local launcher, as for original
// capture roots. This method compares data; it does not manufacture a kernel
// execution observation from a caller-provided pathname, status or callback.
func (baseline *practicalReplayBaseline) compare(moduleOutput, packageOutput, moduleDiagnostics, packageDiagnostics []byte, originalCache string, cache practicalSourceRoot) (result practicalReplayComparison, failure error) {
	baseline.mutex.Lock()
	defer baseline.mutex.Unlock()
	if baseline.closed {
		return result, fmt.Errorf("original replay baseline is closed")
	}
	if baseline.failed != nil {
		return result, baseline.failed
	}
	if err := baseline.view.check(); err != nil {
		return result, baseline.reject(err)
	}
	if !bytes.Equal(moduleDiagnostics, baseline.moduleDiagnostics) || !bytes.Equal(packageDiagnostics, baseline.packageDiagnostics) || !practicalReplayAbsolute(originalCache) {
		return result, baseline.reject(fmt.Errorf("native replay has diagnostics or an invalid derived cache"))
	}
	modules, err := decodePracticalCompleteObjects(moduleOutput)
	if err != nil || !reflect.DeepEqual(modules, baseline.modules) {
		return result, baseline.reject(fmt.Errorf("native replay changed the complete original module observation"))
	}
	packages, err := decodePracticalCompleteObjects(packageOutput)
	if err != nil || len(packages) != len(baseline.packages) {
		return result, baseline.reject(fmt.Errorf("native replay changed complete original package membership"))
	}
	actual, err := newPracticalCaptureSession([]practicalSourceRoot{cache})
	if err != nil {
		return result, baseline.reject(err)
	}
	defer func() {
		// Preserve every actual buffer read before a failed correspondence.
		// A mismatching fresh buffer keeps its own real source identity.
		if failure != nil {
			result.actual, _ = copyPracticalSourceSnapshot(actual)
		}
	}()
	seen := map[string]bool{}
	for i, original := range baseline.packages {
		row := packages[i]
		name, _ := original["ImportPath"].(string)
		expected, hasArchive := baseline.archives[name]
		if !hasArchive {
			if !reflect.DeepEqual(row, original) {
				return result, baseline.reject(fmt.Errorf("native replay changed complete original intrinsic metadata"))
			}
			continue
		}
		locator, ok := row["Export"].(string)
		if !ok {
			return result, baseline.reject(fmt.Errorf("native replay lost its derived archive locator"))
		}
		narrowed := maps.Clone(row)
		narrowed["Export"] = original["Export"]
		if !reflect.DeepEqual(narrowed, original) {
			return result, baseline.reject(fmt.Errorf("native replay changed a complete original package field"))
		}
		relative, err := practicalOriginalCacheRelative(originalCache, locator)
		if err != nil || seen[relative] {
			return result, baseline.reject(fmt.Errorf("native replay archive escaped or duplicated its derived cache"))
		}
		seen[relative] = true
		buffer, err := actual.readSource(cache.Capability, relative, "export_data")
		if err != nil {
			return result, baseline.reject(err)
		}
		buildID, ok := row["BuildID"].(string)
		if !ok || buffer.object.RawSHA256 != expected.RawSHA256 || buffer.object.Bytes != expected.Bytes || validatePracticalExportArchive(buffer.data, buildID) != nil {
			return result, baseline.reject(fmt.Errorf("native replay differs from the whole original archive/build ID"))
		}
		result.exports++
		result.bytes += buffer.object.Bytes
	}
	result.actual, err = actual.seal()
	if err != nil {
		return result, baseline.reject(err)
	}
	if err := baseline.view.check(); err != nil {
		return result, baseline.reject(err)
	}
	result.modules, result.packages = len(modules), len(packages)
	return result, nil
}

func (baseline *practicalReplayBaseline) Close() error {
	baseline.mutex.Lock()
	defer baseline.mutex.Unlock()
	if baseline.closed {
		return nil
	}
	baseline.closed = true
	return baseline.view.Close()
}

func practicalObservationEnvironment(observation *practicalGoObservation, name string) string {
	for _, value := range observation.environment {
		if key, value, ok := strings.Cut(value, "="); ok && key == name {
			return value
		}
	}
	return ""
}
