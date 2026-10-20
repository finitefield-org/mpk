package main

import (
	"bytes"
	"fmt"
	"go/ast"
	"go/constant"
	"go/types"
)

// The frozen Go practical string domain counts bytes, not Unicode characters.
const maximumPracticalStringBytes = 256

// An independently owned native compiler observation. Source-expression
// evaluation, nominal identity, registered context and program proofs remain
// separate. In particular nativeType is display data, never a source type ID.
type practicalStringConstant struct {
	packagePath, moduleSourceID, imageDigest, compilerDigest string
	span                                                     practicalSourceSpan
	source                                                   practicalSourceBuffer
	expression, value                                        []byte
	nativeType                                               string
}

// The complete frozen data shape: Word64 length and 256 arbitrary Word8 slots.
// Flat slot/bit indices use their ordinary numeric order; the future ordinary
// term constructor must retain the frozen most-significant-head addresses.
// This value does not itself confer source validity or access authority.
type practicalStringCarrier struct {
	lengthWord [64]bool
	byteSlots  [maximumPracticalStringBytes][8]bool
}

type practicalStringDomainError struct{ bytes int }

func (err *practicalStringDomainError) Error() string {
	return fmt.Sprintf("original Go string has %d bytes, exceeding the frozen 256-byte modeled domain", err.bytes)
}

// The query contains only an original physical locator. No caller AST, typed
// value, package, import callback, supplied bytes or replacement source enters.
func (packages *practicalOriginalPackages) stringConstant(importPath string, span practicalSourceSpan) (practicalStringConstant, error) {
	if packages == nil {
		return practicalStringConstant{}, fmt.Errorf("original string observation requires an owned package universe")
	}
	packages.mutex.Lock()
	defer packages.mutex.Unlock()
	if packages.closed || packages.inputs == nil || packages.loader == nil || packages.loader.syntax == nil {
		return practicalStringConstant{}, fmt.Errorf("original package universe is closed or incomplete")
	}
	for _, failure := range []error{packages.inputs.failed, packages.loader.failed, packages.loader.syntax.failed} {
		if failure != nil {
			return practicalStringConstant{}, failure
		}
	}
	pkg, exists := packages.packages[importPath]
	loaded := packages.loader.packages[importPath]
	if !exists || pkg.intrinsic || loaded == nil || loaded.importPath != importPath || loaded.info == nil {
		return practicalStringConstant{}, fmt.Errorf("string observation is outside the complete original source package universe")
	}
	if span.start < 0 || span.end <= span.start {
		return practicalStringConstant{}, fmt.Errorf("string observation needs one exact original expression span")
	}
	var object practicalSourceObject
	for _, source := range pkg.sources {
		if source.ID == span.sourceID {
			if object.ID != "" {
				return practicalStringConstant{}, fmt.Errorf("string observation has ambiguous source membership")
			}
			object = source
		}
	}
	if object.ID == "" || object.Kind != "go_file" {
		return practicalStringConstant{}, fmt.Errorf("string observation crossed its original package source membership")
	}
	syntax := packages.loader.syntax
	var expression ast.Expr
	var value types.TypeAndValue
	for candidate, fact := range loaded.info.Types {
		binding, found := syntax.nodes[candidate]
		if !found || binding.span != span || fact.Value == nil || fact.Value.Kind() != constant.String || fact.Type == nil || fact.IsType() {
			continue
		}
		basic, isBasic := fact.Type.Underlying().(*types.Basic)
		if !isBasic || basic.Info()&types.IsString == 0 {
			continue
		}
		if expression != nil {
			return practicalStringConstant{}, fmt.Errorf("string observation has more than one original constant expression")
		}
		expression, value = candidate, fact
	}
	if expression == nil {
		return practicalStringConstant{}, fmt.Errorf("original expression is not a native Go constant string")
	}
	actual, raw, err := syntax.spanBytes(expression)
	if err != nil {
		return practicalStringConstant{}, err
	}
	if actual != span {
		return practicalStringConstant{}, fmt.Errorf("original string expression lost its physical source binding")
	}
	source, err := packages.inputs.readSource(object.Capability, object.Path, object.Kind)
	if err != nil {
		return practicalStringConstant{}, err
	}
	parsed := syntax.files[object.ID]
	if source.object != object || parsed == nil || parsed.source.object != object || span.end > len(source.data) ||
		!bytes.Equal(source.data, parsed.source.data) || !bytes.Equal(raw, source.data[span.start:span.end]) {
		return practicalStringConstant{}, packages.inputs.reject(fmt.Errorf("string observation lost its complete original source bytes"))
	}
	// StringVal preserves every native byte, including invalid UTF-8 and NUL.
	// Retain the full value even when later modeled-domain conversion rejects it.
	decoded := []byte(constant.StringVal(value.Value))
	return practicalStringConstant{
		packagePath: importPath, moduleSourceID: loaded.moduleSourceID,
		imageDigest: packages.imageDigest, compilerDigest: packages.compilerDigest,
		span: span, source: source, expression: bytes.Clone(raw), value: decoded,
		nativeType: types.TypeString(value.Type, func(pkg *types.Package) string { return pkg.Path() }),
	}, nil
}

func (value practicalStringConstant) originalSource() practicalSourceBuffer {
	return practicalSourceBuffer{object: value.source.object, data: value.source.sourceBytes()}
}

func (value practicalStringConstant) originalExpression() []byte {
	return bytes.Clone(value.expression)
}
func (value practicalStringConstant) nativeBytes() []byte { return bytes.Clone(value.value) }

func (value practicalStringConstant) carrier() (practicalStringCarrier, error) {
	// An absent or damaged observation must not silently become an empty Go
	// string. This consistency check still grants no source/proof authority.
	if value.packagePath == "" || value.imageDigest == "" || value.compilerDigest == "" || value.nativeType == "" ||
		value.source.object.Kind != "go_file" || value.span.sourceID != value.source.object.ID ||
		value.span.start < 0 || value.span.end <= value.span.start || value.span.end > len(value.source.data) ||
		!bytes.Equal(value.expression, value.source.data[value.span.start:value.span.end]) {
		return practicalStringCarrier{}, fmt.Errorf("string carrier requires one intact original native observation")
	}
	if _, err := value.source.canonicalObject(); err != nil {
		return practicalStringCarrier{}, err
	}
	if len(value.value) > maximumPracticalStringBytes {
		return practicalStringCarrier{}, &practicalStringDomainError{bytes: len(value.value)}
	}
	var result practicalStringCarrier
	count := uint64(len(value.value))
	for bit := range result.lengthWord {
		result.lengthWord[bit] = count&(uint64(1)<<bit) != 0
	}
	for slot, octet := range value.value {
		for bit := range result.byteSlots[slot] {
			result.byteSlots[slot][bit] = octet&(uint8(1)<<bit) != 0
		}
	}
	return result, nil
}
