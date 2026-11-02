// Private T01-W02 compiler/closure measurement; not a production frontend.
package main

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"go/ast"
	"go/token"
	"go/types"
	"go/version"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sort"
	"strings"
	"time"

	"golang.org/x/mod/modfile"
	"golang.org/x/tools/go/packages"
)

type request struct {
	Schema       string   `json:"schema"`
	Root         string   `json:"root"`
	Package      string   `json:"package"`
	Function     string   `json:"function"`
	CaptureRoots []string `json:"capture_roots"`
	BuildTags    []string `json:"build_tags"`
}

type finding struct {
	Code     string   `json:"code"`
	Function string   `json:"function"`
	Detail   string   `json:"detail"`
	Path     []string `json:"path_from_root"`
}

type fileFact struct {
	Path       string `json:"path"`
	SHA256     string `json:"sha256"`
	GoVersion  string `json:"go_version,omitempty"`
	SourceRole string `json:"source_role"`
	RawVersion string `json:"raw_go_version,omitempty"`
}

type functionFact struct {
	ID          string `json:"id"`
	Disposition string `json:"disposition"`
	File        string `json:"file"`
	GoVersion   string `json:"go_version"`
	Signature   string `json:"signature"`
}

type moduleFact struct {
	Path        string `json:"path"`
	Version     string `json:"version"`
	GoVersion   string `json:"go_version"`
	Replacement string `json:"replacement"`
	GoModSHA256 string `json:"go_mod_sha256"`
}

type result struct {
	Schema          string         `json:"schema"`
	Compiler        string         `json:"compiler"`
	Target          string         `json:"target"`
	PointerWidth    int64          `json:"pointer_width"`
	Status          string         `json:"status"`
	Scope           string         `json:"scope"`
	Findings        []finding      `json:"findings"`
	Diagnostics     []string       `json:"diagnostics"`
	Functions       []functionFact `json:"functions"`
	TypeConstants   []string       `json:"type_constant_dependencies"`
	LoadedInputs    []fileFact     `json:"observed_loaded_inputs"`
	Modules         []moduleFact   `json:"module_graph"`
	StandardImports []string       `json:"standard_packages"`
}

type declaration struct {
	pkg       *packages.Package
	function  *ast.FuncDecl
	file      *ast.File
	filename  string
	goVersion string
}

type analyzer struct {
	request request
	result  result
	decls   map[string]declaration
	files   map[string]fileFact
	deps    map[string]bool
	graph   map[string][]string
	closure map[string]bool
	roots   []string
}

func main() {
	var req request
	decoder := json.NewDecoder(os.Stdin)
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&req); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(2)
	}
	if err := decoder.Decode(new(any)); err != io.EOF {
		fmt.Fprintln(os.Stderr, "request must be exactly one JSON value")
		os.Exit(2)
	}
	a := analyzer{
		request: req, decls: map[string]declaration{}, files: map[string]fileFact{},
		deps: map[string]bool{}, graph: map[string][]string{}, closure: map[string]bool{},
		result: result{Schema: "mpk.go_practical.module_probe.v0", Compiler: runtime.Version(),
			Target: "linux/amd64", Status: "closed", Scope: "private compiler/closure observations and early exclusions; not complete profile admission or proof",
			Findings: []finding{}, Diagnostics: []string{}, Functions: []functionFact{}, TypeConstants: []string{},
			LoadedInputs: []fileFact{}, Modules: []moduleFact{}, StandardImports: []string{}},
	}
	if err := a.run(); err != nil {
		a.result.Status = "probe_error"
		a.result.Diagnostics = append(a.result.Diagnostics, err.Error())
	}
	a.finish()
	encoder := json.NewEncoder(os.Stdout)
	encoder.SetIndent("", "  ")
	if err := encoder.Encode(a.result); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(2)
	}
}

func (a *analyzer) reject(code, function, detail string) {
	a.result.Findings = append(a.result.Findings, finding{code, function, detail, []string{}})
	a.result.Status = "rejected"
}

func containsPath(root, path string) bool {
	relative, err := filepath.Rel(root, path)
	return err == nil && relative != ".." && !strings.HasPrefix(relative, ".."+string(os.PathSeparator))
}

func (a *analyzer) captured(path string) bool {
	resolved, err := filepath.EvalSymlinks(path)
	if err != nil {
		return false
	}
	for _, root := range a.roots {
		if containsPath(root, resolved) {
			return true
		}
	}
	return false
}

func digest(path string) (string, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}
	sum := sha256.Sum256(data)
	return hex.EncodeToString(sum[:]), nil
}

func (a *analyzer) addFile(path, role, goVersion string) error {
	if !a.captured(path) {
		return fmt.Errorf("loaded input outside capture capabilities: %s", path)
	}
	value, err := digest(path)
	if err != nil {
		return err
	}
	a.files[path] = fileFact{path, value, version.Lang(goVersion), role, goVersion}
	return nil
}

func allowedLanguage(value string) bool {
	switch value {
	case "go1.22", "go1.23", "go1.24", "go1.25":
		return true
	}
	return false
}

func (a *analyzer) run() error {
	if a.request.Schema != "mpk.go_practical.module_probe_request.v0" || runtime.Version() != "go1.25.0" {
		return fmt.Errorf("exact probe schema and Go 1.25.0 compiler required")
	}
	if !filepath.IsAbs(a.request.Root) || strings.Contains(a.request.Package, "...") {
		return fmt.Errorf("absolute root and literal package selection required")
	}
	a.roots = append(a.roots, a.request.Root)
	a.roots = append(a.roots, a.request.CaptureRoots...)
	a.roots = append(a.roots, os.Getenv("GOMODCACHE"))
	for i, root := range a.roots {
		if !filepath.IsAbs(root) {
			return fmt.Errorf("capture roots must be absolute")
		}
		resolved, err := filepath.EvalSymlinks(root)
		if err != nil {
			return err
		}
		a.roots[i] = resolved
	}
	for _, name := range []string{"go.work", "go.work.sum", "vendor"} {
		if _, err := os.Lstat(filepath.Join(a.request.Root, name)); err == nil {
			a.reject("GO_PROBE_BUILD_POLICY", "", "unsupported workspace/vendor input: "+name)
			return nil
		}
	}
	modPath := filepath.Join(a.request.Root, "go.mod")
	if !a.captured(modPath) {
		return fmt.Errorf("module manifest is outside capture capabilities")
	}
	modBytes, err := os.ReadFile(modPath)
	if err != nil {
		return err
	}
	mod, err := modfile.Parse(modPath, modBytes, nil)
	if err != nil || mod.Go == nil {
		a.reject("GO_PROBE_MODULE_POLICY", "", "module must have a valid go directive")
		return nil
	}
	if !allowedLanguage(version.Lang("go" + mod.Go.Version)) {
		a.reject("GO_PROBE_LANGUAGE_VERSION", "", mod.Go.Version)
		return nil
	}
	if version.Compare("go"+mod.Go.Version, runtime.Version()) > 0 || mod.Toolchain != nil && version.Compare(mod.Toolchain.Name, runtime.Version()) > 0 {
		a.reject("GO_PROBE_TOOLCHAIN_REQUIREMENT", "", "pinned compiler cannot satisfy module/toolchain requirement")
		return nil
	}
	for _, replacement := range mod.Replace {
		if replacement.New.Version == "" {
			path := replacement.New.Path
			if !filepath.IsAbs(path) {
				path = filepath.Join(a.request.Root, path)
			}
			if !a.captured(path) {
				a.reject("GO_PROBE_REPLACEMENT_ESCAPE", "", replacement.Old.Path)
				return nil
			}
		}
	}
	if err := a.addFile(modPath, "build_manifest", ""); err != nil {
		return err
	}
	sumPath := filepath.Join(a.request.Root, "go.sum")
	if _, err := os.Stat(sumPath); err == nil {
		if err := a.addFile(sumPath, "lockfile", ""); err != nil {
			return err
		}
	}
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
	defer cancel()
	command := exec.CommandContext(ctx, "go", "list", "-mod=readonly", "-m", "-json", "all")
	command.Dir, command.Env = a.request.Root, os.Environ()
	moduleBytes, err := command.Output()
	if err != nil {
		a.result.Status = "source_error"
		a.result.Diagnostics = append(a.result.Diagnostics, "offline module graph unavailable")
		if exit, ok := err.(*exec.ExitError); ok {
			a.result.Diagnostics = append(a.result.Diagnostics, string(exit.Stderr))
		}
		return nil
	}
	modules := json.NewDecoder(strings.NewReader(string(moduleBytes)))
	for {
		var module packages.Module
		if err := modules.Decode(&module); err == io.EOF {
			break
		} else if err != nil {
			return err
		}
		actual := &module
		replacement := ""
		if module.Replace != nil {
			actual = module.Replace
			replacement = actual.Path + "@" + actual.Version
		}
		if actual.GoMod == "" {
			return fmt.Errorf("resolved module lacks go.mod: %s", module.Path)
		}
		if err := a.addFile(actual.GoMod, "resolved_module_manifest", ""); err != nil {
			return err
		}
		a.result.Modules = append(a.result.Modules, moduleFact{module.Path, module.Version, actual.GoVersion, replacement, a.files[actual.GoMod].SHA256})
	}
	flags := []string{"-mod=readonly", "-trimpath"}
	if len(a.request.BuildTags) != 0 {
		flags = append(flags, "-tags="+strings.Join(a.request.BuildTags, ","))
	}
	loaded, err := packages.Load(&packages.Config{
		Context: ctx, Dir: a.request.Root, Env: os.Environ(), Tests: false, BuildFlags: flags,
		Mode: packages.NeedName | packages.NeedFiles | packages.NeedCompiledGoFiles |
			packages.NeedImports | packages.NeedDeps | packages.NeedModule |
			packages.NeedSyntax | packages.NeedTypes | packages.NeedTypesInfo | packages.NeedTypesSizes,
	}, a.request.Package)
	if err != nil {
		a.result.Status = "source_error"
		a.result.Diagnostics = append(a.result.Diagnostics, err.Error())
		return nil
	}
	if len(loaded) != 1 || loaded[0].PkgPath != a.request.Package {
		return fmt.Errorf("package selection did not resolve exactly once")
	}
	all := map[string]*packages.Package{}
	packages.Visit(loaded, func(pkg *packages.Package) bool {
		all[pkg.PkgPath] = pkg
		return true
	}, nil)
	paths := make([]string, 0, len(all))
	for path := range all {
		paths = append(paths, path)
	}
	sort.Strings(paths)
	for _, path := range paths {
		pkg := all[path]
		for _, err := range pkg.Errors {
			a.result.Diagnostics = append(a.result.Diagnostics, err.Error())
		}
		if pkg.Module == nil {
			a.result.StandardImports = append(a.result.StandardImports, path)
			continue
		}
		if pkg.Types == nil || pkg.TypesInfo == nil || pkg.TypesSizes == nil {
			continue // Actual type errors are reported below, never admitted.
		}
		for _, filename := range pkg.IgnoredFiles {
			if err := a.addFile(filename, "build_excluded_source", ""); err != nil {
				return err
			}
		}
		for _, filename := range pkg.EmbedFiles {
			if err := a.addFile(filename, "embedded_asset_observation", ""); err != nil {
				return err
			}
		}
		for _, filename := range pkg.OtherFiles {
			if err := a.addFile(filename, "auxiliary_build_input", ""); err != nil {
				return err
			}
		}
		width := 8 * pkg.TypesSizes.Sizeof(types.Typ[types.Int])
		if width != 64 {
			return fmt.Errorf("target integer width mismatch: %d", width)
		}
		a.result.PointerWidth = 8 * pkg.TypesSizes.Sizeof(types.Typ[types.Uintptr])
		for _, file := range pkg.Syntax {
			filename := pkg.Fset.Position(file.Pos()).Filename
			rawVersion := pkg.TypesInfo.FileVersions[file]
			goVersion := version.Lang(rawVersion)
			if err := a.addFile(filename, "typecheck_source", rawVersion); err != nil {
				return err
			}
			for _, decl := range file.Decls {
				fn, ok := decl.(*ast.FuncDecl)
				if !ok || fn.Body == nil {
					continue
				}
				object, ok := pkg.TypesInfo.Defs[fn.Name].(*types.Func)
				if !ok {
					continue
				}
				id := functionID(object)
				if fn.Name.Name == "init" || fn.Name.Name == "_" {
					id += fmt.Sprintf("@%s:%d", filename, pkg.Fset.Position(fn.Pos()).Offset)
				}
				a.decls[id] = declaration{pkg, fn, file, filename, goVersion}
			}
		}
	}
	if len(a.result.Diagnostics) != 0 {
		a.result.Status = "source_error"
		return nil
	}
	if _, exists := a.decls[a.request.Function]; !exists {
		a.reject("GO_PROBE_SELECTION", "", a.request.Function)
		return nil
	}
	queue := []string{a.request.Function}
	for len(queue) != 0 {
		id := queue[0]
		queue = queue[1:]
		if a.closure[id] {
			continue
		}
		a.closure[id] = true
		decl := a.decls[id]
		if !allowedLanguage(decl.goVersion) {
			a.reject("GO_PROBE_LANGUAGE_VERSION", id, decl.goVersion)
		}
		object := decl.pkg.TypesInfo.Defs[decl.function.Name].(*types.Func)
		signature := object.Type().(*types.Signature)
		if signature.TypeParams().Len() != 0 {
			a.reject("GO_PROBE_GENERICS", id, "generic body in verification closure")
		}
		a.checkType(signature.Params(), id, map[types.Type]bool{})
		a.checkType(signature.Results(), id, map[types.Type]bool{})
		if signature.Recv() != nil {
			a.checkType(signature.Recv().Type(), id, map[types.Type]bool{})
		}
		ast.Inspect(decl.function, func(node ast.Node) bool {
			switch n := node.(type) {
			case *ast.GoStmt, *ast.DeferStmt, *ast.SendStmt, *ast.SelectStmt, *ast.FuncLit, *ast.TypeAssertExpr:
				a.reject("GO_PROBE_UNSUPPORTED_EFFECT", id, fmt.Sprintf("%T", node))
			case *ast.MapType, *ast.ChanType, *ast.StarExpr:
				a.reject("GO_PROBE_UNSUPPORTED_TYPE", id, fmt.Sprintf("%T", node))
			case *ast.UnaryExpr:
				if n.Op == token.AND || n.Op == token.ARROW {
					a.reject("GO_PROBE_UNSUPPORTED_EFFECT", id, n.Op.String())
				}
			case *ast.Ident:
				used := decl.pkg.TypesInfo.Uses[n]
				if used == nil {
					break
				}
				switch used := used.(type) {
				case *types.Var:
					if used.Pkg() != nil && used.Parent() == used.Pkg().Scope() {
						a.reject("GO_PROBE_GLOBAL_STATE", id, used.Pkg().Path()+"."+used.Name())
					}
				case *types.TypeName, *types.Const:
					if used.Pkg() != nil {
						a.deps[used.Pkg().Path()+"."+used.Name()] = true
					}
				}
			case *ast.CallExpr:
				if value := decl.pkg.TypesInfo.Types[n.Fun]; value.IsType() {
					break
				}
				callee, builtin, dynamic := resolveCall(decl.pkg.TypesInfo, n.Fun)
				if builtin != "" {
					switch builtin {
					case "len", "cap", "make", "copy", "append":
					default:
						a.reject("GO_PROBE_UNSUPPORTED_BUILTIN", id, builtin)
					}
					break
				}
				if callee == nil || dynamic {
					a.reject("GO_PROBE_INDIRECT_CALL", id, "call target is not a direct concrete function")
					break
				}
				if callee.Type().(*types.Signature).TypeParams().Len() != 0 {
					a.reject("GO_PROBE_GENERICS", id, functionID(callee))
				}
				calleeID := functionID(callee)
				if _, captured := a.decls[calleeID]; !captured {
					a.reject("GO_PROBE_UNMODELED_EXTERNAL_CALL", id, calleeID)
					break
				}
				a.graph[id] = append(a.graph[id], calleeID)
				queue = append(queue, calleeID)
			}
			return true
		})
	}
	visiting, done := map[string]bool{}, map[string]bool{}
	var visit func(string)
	visit = func(id string) {
		if visiting[id] {
			a.reject("GO_PROBE_CALL_CYCLE", id, "recursive verification closure")
			return
		}
		if done[id] {
			return
		}
		visiting[id] = true
		for _, callee := range a.graph[id] {
			visit(callee)
		}
		visiting[id], done[id] = false, true
	}
	visit(a.request.Function)
	return nil
}

func (a *analyzer) checkType(t types.Type, function string, seen map[types.Type]bool) {
	if t == nil || seen[t] {
		return
	}
	seen[t] = true
	switch t := t.(type) {
	case *types.Named:
		if t.Obj().Pkg() == nil && t.Obj().Name() == "error" {
			return
		}
		if t.Obj().Pkg() != nil {
			a.deps[t.Obj().Pkg().Path()+"."+t.Obj().Name()] = true
		}
		a.checkType(t.Underlying(), function, seen)
	case *types.Alias:
		a.checkType(types.Unalias(t), function, seen)
	case *types.Tuple:
		for i := 0; i < t.Len(); i++ {
			a.checkType(t.At(i).Type(), function, seen)
		}
	case *types.Array:
		a.checkType(t.Elem(), function, seen)
	case *types.Slice:
		a.checkType(t.Elem(), function, seen)
	case *types.Struct:
		for i := 0; i < t.NumFields(); i++ {
			a.checkType(t.Field(i).Type(), function, seen)
		}
	case *types.Pointer, *types.Map, *types.Chan, *types.Interface, *types.Signature, *types.TypeParam:
		a.reject("GO_PROBE_UNSUPPORTED_TYPE", function, types.TypeString(t, nil))
	case *types.Basic:
		if t.Info()&(types.IsFloat|types.IsComplex) != 0 || t.Kind() == types.Uintptr || t.Kind() == types.UnsafePointer {
			a.reject("GO_PROBE_UNSUPPORTED_TYPE", function, t.Name())
		}
	}
}

func functionID(fn *types.Func) string {
	prefix := fn.Pkg().Path() + "."
	signature := fn.Type().(*types.Signature)
	if receiver := signature.Recv(); receiver != nil {
		t := types.Unalias(receiver.Type())
		if pointer, ok := t.(*types.Pointer); ok {
			t = types.Unalias(pointer.Elem())
		}
		if named, ok := t.(*types.Named); ok {
			prefix += named.Obj().Name() + "."
		}
	}
	return prefix + fn.Name()
}

func resolveCall(info *types.Info, expression ast.Expr) (*types.Func, string, bool) {
	switch expr := expression.(type) {
	case *ast.ParenExpr:
		return resolveCall(info, expr.X)
	case *ast.IndexExpr:
		return resolveCall(info, expr.X)
	case *ast.IndexListExpr:
		return resolveCall(info, expr.X)
	case *ast.Ident:
		switch object := info.Uses[expr].(type) {
		case *types.Func:
			return object, "", false
		case *types.Builtin:
			return nil, object.Name(), false
		}
	case *ast.SelectorExpr:
		if selection := info.Selections[expr]; selection != nil {
			_, dynamic := selection.Recv().Underlying().(*types.Interface)
			function, _ := selection.Obj().(*types.Func)
			return function, "", dynamic
		}
		function, _ := info.Uses[expr.Sel].(*types.Func)
		return function, "", false
	}
	return nil, "", true
}

func (a *analyzer) finish() {
	paths := map[string][]string{a.request.Function: {a.request.Function}}
	queue := []string{a.request.Function}
	for len(queue) != 0 {
		id := queue[0]
		queue = queue[1:]
		callees := append([]string{}, a.graph[id]...)
		sort.Strings(callees)
		for _, callee := range callees {
			if _, seen := paths[callee]; !seen {
				paths[callee] = append(append([]string{}, paths[id]...), callee)
				queue = append(queue, callee)
			}
		}
	}
	for i := range a.result.Findings {
		if path, exists := paths[a.result.Findings[i].Function]; exists {
			a.result.Findings[i].Path = path
		}
	}
	for id, decl := range a.decls {
		disposition := "type_check_only"
		if a.closure[id] {
			disposition = "body_dependency"
		}
		if id == a.request.Function {
			disposition = "selected_root"
		}
		object := decl.pkg.TypesInfo.Defs[decl.function.Name]
		signature := types.TypeString(object.Type(), func(pkg *types.Package) string { return pkg.Path() })
		a.result.Functions = append(a.result.Functions, functionFact{id, disposition, decl.filename, decl.goVersion, signature})
	}
	for dep := range a.deps {
		a.result.TypeConstants = append(a.result.TypeConstants, dep)
	}
	for _, file := range a.files {
		a.result.LoadedInputs = append(a.result.LoadedInputs, file)
	}
	sort.Slice(a.result.Functions, func(i, j int) bool { return a.result.Functions[i].ID < a.result.Functions[j].ID })
	sort.Slice(a.result.LoadedInputs, func(i, j int) bool { return a.result.LoadedInputs[i].Path < a.result.LoadedInputs[j].Path })
	sort.Slice(a.result.Modules, func(i, j int) bool { return a.result.Modules[i].Path < a.result.Modules[j].Path })
	sort.Slice(a.result.Findings, func(i, j int) bool {
		x, y := a.result.Findings[i], a.result.Findings[j]
		return x.Function+"\x00"+x.Code+"\x00"+x.Detail < y.Function+"\x00"+y.Code+"\x00"+y.Detail
	})
	sort.Strings(a.result.TypeConstants)
	sort.Strings(a.result.StandardImports)
	sort.Strings(a.result.Diagnostics)
}
