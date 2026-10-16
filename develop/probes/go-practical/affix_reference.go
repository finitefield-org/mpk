// Original packages are compiled unchanged. Recovery and AST capture are observer-only.
package main

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/format"
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
)

func prefixBody(s, prefix string) bool {
	return len(s) >= len(prefix) && s[:len(prefix)] == prefix
}
func suffixBody(s, suffix string) bool {
	return len(s) >= len(suffix) && s[len(s)-len(suffix):] == suffix
}

type affixInput struct {
	Bound         int    `json:"bound"`
	Source        string `json:"source"`
	Pattern       string `json:"pattern"`
	SourceLength  int    `json:"source_length"`
	PatternLength int    `json:"pattern_length"`
	Index         string `json:"index"`
}
type affixOutput struct {
	Prefix                   bool   `json:"prefix"`
	Suffix                   bool   `json:"suffix"`
	PrefixBody               bool   `json:"prefix_body"`
	SuffixBody               bool   `json:"suffix_body"`
	MaskedPrefix             bool   `json:"masked_prefix"`
	MaskedSuffix             bool   `json:"masked_suffix"`
	PrefixPanicked           bool   `json:"prefix_panicked"`
	SuffixPanicked           bool   `json:"suffix_panicked"`
	UnconditionalPrefixPanic bool   `json:"unconditional_prefix_panic"`
	UnconditionalSuffixPanic bool   `json:"unconditional_suffix_panic"`
	PatternRead              bool   `json:"pattern_read"`
	PatternValue             int    `json:"pattern_value"`
	PrefixRead               bool   `json:"prefix_read"`
	PrefixValue              int    `json:"prefix_value"`
	SuffixRead               bool   `json:"suffix_read"`
	SuffixValue              int    `json:"suffix_value"`
	SuffixStart              string `json:"suffix_start"`
	SuffixMapped             string `json:"suffix_mapped"`
	InputBound               bool   `json:"input_bound"`
	InputUnchanged           bool   `json:"input_unchanged"`
	QueryCovered             bool   `json:"query_covered"`
}

func affixRead(s string, i int64) (value int, defined bool) {
	value = -1
	defer func() {
		if recover() != nil {
			defined = false
		}
	}()
	return int(s[i]), true
}
func affixNative(s, p string, suffix bool) (value, panicked bool) {
	defer func() {
		if recover() != nil {
			panicked = true
		}
	}()
	if suffix {
		return strings.HasSuffix(s, p), false
	}
	return strings.HasPrefix(s, p), false
}
func affixUnguarded(s, p string, suffix bool) (panicked bool) {
	defer func() {
		if recover() != nil {
			panicked = true
		}
	}()
	if suffix {
		_ = s[len(s)-len(p):] == p
	} else {
		_ = s[:len(p)] == p
	}
	return false
}

type bindingFunction struct {
	Package           string `json:"package"`
	Symbol            string `json:"symbol"`
	Path              string `json:"path"`
	FileSHA256        string `json:"file_sha256"`
	Declaration       string `json:"declaration"`
	DeclarationSHA256 string `json:"declaration_sha256"`
}

func captureBinding(goroot string) {
	functions := []bindingFunction{}
	for _, spec := range []struct{ path, pkg string }{{"src/strings/strings.go", "strings"}, {"src/internal/stringslite/strings.go", "internal/stringslite"}} {
		raw, err := os.ReadFile(filepath.Join(goroot, spec.path))
		if err != nil {
			panic(err)
		}
		fileset := token.NewFileSet()
		file, err := parser.ParseFile(fileset, spec.path, raw, 0)
		if err != nil {
			panic(err)
		}
		for _, name := range []string{"HasPrefix", "HasSuffix"} {
			found := false
			for _, decl := range file.Decls {
				f, ok := decl.(*ast.FuncDecl)
				if !ok || f.Name.Name != name {
					continue
				}
				if found || f.Recv != nil {
					panic("ambiguous selected function")
				}
				found = true
				var buffer bytes.Buffer
				if err := format.Node(&buffer, fileset, f); err != nil {
					panic(err)
				}
				text := bytes.TrimSpace(buffer.Bytes())
				functions = append(functions, bindingFunction{Package: spec.pkg, Symbol: spec.pkg + "." + name, Path: spec.path, FileSHA256: fmt.Sprintf("%x", sha256.Sum256(raw)), Declaration: string(text), DeclarationSHA256: fmt.Sprintf("%x", sha256.Sum256(text))})
			}
			if !found {
				panic("missing selected function")
			}
		}
	}
	if err := json.NewEncoder(os.Stdout).Encode(struct {
		Scope      string            `json:"scope"`
		GoVersion  string            `json:"go_version"`
		Host       string            `json:"host"`
		Registered bool              `json:"registered"`
		Functions  []bindingFunction `json:"functions"`
	}{"exact selected AST/source identities; private model correspondence only, no production adapter or whole-package admission", runtime.Version(), runtime.GOOS + "/" + runtime.GOARCH, false, functions}); err != nil {
		panic(err)
	}
}
func main() {
	if len(os.Args) == 3 && os.Args[1] == "--binding" {
		captureBinding(os.Args[2])
		return
	}
	if len(os.Args) != 2 {
		panic("input JSON path required")
	}
	raw, err := os.ReadFile(os.Args[1])
	if err != nil {
		panic(err)
	}
	var rows []affixInput
	if err := json.Unmarshal(raw, &rows); err != nil {
		panic(err)
	}
	out := make([]affixOutput, 0, len(rows))
	for _, r := range rows {
		s, err := hex.DecodeString(r.Source)
		if err != nil {
			panic(err)
		}
		p, err := hex.DecodeString(r.Pattern)
		if err != nil {
			panic(err)
		}
		if r.SourceLength < 0 || r.SourceLength > len(s) || r.PatternLength < 0 || r.PatternLength > len(p) || r.Bound < 0 {
			panic("invalid byte backing")
		}
		i, err := strconv.ParseInt(r.Index, 10, 64)
		if err != nil {
			panic(err)
		}
		beforeS, beforeP := bytes.Clone(s), bytes.Clone(p)
		source, pattern := string(s[:r.SourceLength]), string(p[:r.PatternLength])
		ms, mp := bytes.Clone(s), bytes.Clone(p)
		for j := r.SourceLength; j < len(ms); j++ {
			ms[j] = 0
		}
		for j := r.PatternLength; j < len(mp); j++ {
			mp[j] = 0
		}
		a := affixOutput{PrefixBody: prefixBody(source, pattern), SuffixBody: suffixBody(source, pattern), MaskedPrefix: strings.HasPrefix(string(ms[:r.SourceLength]), string(mp[:r.PatternLength])), MaskedSuffix: strings.HasSuffix(string(ms[:r.SourceLength]), string(mp[:r.PatternLength])), UnconditionalPrefixPanic: affixUnguarded(source, pattern, false), UnconditionalSuffixPanic: affixUnguarded(source, pattern, true), InputBound: r.SourceLength <= r.Bound && r.PatternLength <= r.Bound, InputUnchanged: bytes.Equal(s, beforeS) && bytes.Equal(p, beforeP)}
		a.Prefix, a.PrefixPanicked = affixNative(source, pattern, false)
		a.Suffix, a.SuffixPanicked = affixNative(source, pattern, true)
		a.PatternValue, a.PatternRead = affixRead(pattern, i)
		a.PrefixValue, a.PrefixRead = affixRead(source, i)
		start := uint64(r.SourceLength) - uint64(r.PatternLength)
		mapped := start + uint64(i)
		a.SuffixStart = strconv.FormatUint(start, 10)
		a.SuffixMapped = strconv.FormatUint(mapped, 10)
		a.SuffixValue, a.SuffixRead = affixRead(source, int64(mapped))
		padding := uint64(1)
		for padding < uint64(r.Bound) {
			padding *= 2
		}
		a.QueryCovered = uint64(i) == uint64(i)&(padding-1)
		out = append(out, a)
	}
	if err := json.NewEncoder(os.Stdout).Encode(out); err != nil {
		panic(err)
	}
}
