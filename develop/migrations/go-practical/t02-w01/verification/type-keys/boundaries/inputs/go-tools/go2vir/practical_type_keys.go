package main

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"sort"
	"unicode"
	"unicode/utf8"
)

const (
	practicalTypePackageSchema     = "mpk.go.type_package.v2"
	practicalTypePackageDomain     = "MPK-GO-TYPE-PACKAGE-2.0"
	practicalTypeDeclarationSchema = "mpk.go.type_declaration.v2"
	practicalTypeDeclarationDomain = "MPK-GO-TYPE-DECLARATION-2.0"
	practicalSourceTypeSchema      = "mpk.go.source_type.v2"
	practicalSourceTypeDomain      = "MPK-GO-SOURCE-TYPE-2.0"
	practicalTypeComponentsSchema  = "mpk.go.type_components.v2"
	practicalTypeComponentsDomain  = "MPK-GO-TYPE-COMPONENTS-2.0"
)

// These are private, internally consistent key candidates and owned original
// bytes, not a complete capture context, source typing witness or program proof.
// Neither physical preparation paths nor native display names enter these keys.
type practicalTypeKeyComponents struct {
	canonical []byte
	sources   []practicalSourceBuffer
	nodeKeys  []string
}
type practicalTypeKeyBuilder struct {
	owner                         *practicalOriginalPackages
	sources                       map[string]practicalSourceBuffer
	packages, declarations, types map[string]map[string]any
	packageIDs                    map[string]string
	underlying, aliases           map[string]string
}

func practicalTypeKeySeal(domain string, body map[string]any) (map[string]any, error) {
	id, err := hashTypedCanonicalJSON(domain, body)
	if err != nil {
		return nil, err
	}
	body["id"] = id
	return body, nil
}
func (b *practicalTypeKeyBuilder) source(buffer practicalSourceBuffer) error {
	if _, err := buffer.canonicalObject(); err != nil {
		return err
	}
	if prior, exists := b.sources[buffer.object.ID]; exists && (prior.object != buffer.object || !bytes.Equal(prior.data, buffer.data)) {
		return fmt.Errorf("type component source crossed its complete original identity")
	}
	b.sources[buffer.object.ID] = practicalSourceBuffer{object: buffer.object, data: buffer.sourceBytes()}
	return nil
}

// The caller holds the original-package mutex. Package scope includes the full
// active original source set and its real go.mod object, never a supplied name.
func (b *practicalTypeKeyBuilder) packageKey(importPath string) (string, error) {
	if id, ok := b.packageIDs[importPath]; ok {
		return id, nil
	}
	pkg, ok := b.owner.packages[importPath]
	loaded := b.owner.loader.packages[importPath]
	if !ok || pkg.intrinsic || loaded == nil || loaded.types == nil || loaded.types.Path() != importPath || loaded.types.Name() != pkg.name || len(pkg.sources) == 0 {
		return "", fmt.Errorf("type component package is outside the owned original universe")
	}
	manifest, ok := b.owner.inputs.snapshot.buffers[loaded.moduleSourceID]
	if !ok || manifest.object.Kind != "go_mod" {
		return "", fmt.Errorf("type component package has no original module manifest")
	}
	if err := b.source(manifest); err != nil {
		return "", err
	}
	ids := make([]string, 0, len(pkg.sources))
	for _, object := range pkg.sources {
		buffer, err := b.owner.inputs.readSource(object.Capability, object.Path, object.Kind)
		if err != nil {
			return "", err
		}
		if buffer.object != object || object.Kind != "go_file" {
			return "", fmt.Errorf("type component package crossed its original active source membership")
		}
		if err := b.source(buffer); err != nil {
			return "", err
		}
		ids = append(ids, object.ID)
	}
	sort.Strings(ids)
	for i := 1; i < len(ids); i++ {
		if ids[i-1] == ids[i] {
			return "", fmt.Errorf("duplicate type package source")
		}
	}
	row, err := practicalTypeKeySeal(practicalTypePackageDomain, map[string]any{"schema": practicalTypePackageSchema, "module_source_object_id": loaded.moduleSourceID, "import_path": importPath, "package_name": pkg.name, "source_objects": ids})
	if err != nil {
		return "", err
	}
	id := row["id"].(string)
	b.packages[id] = row
	b.packageIDs[importPath] = id
	return id, nil
}
func practicalTypeKeyRows(rows map[string]map[string]any) []map[string]any {
	ids := make([]string, 0, len(rows))
	for id := range rows {
		ids = append(ids, id)
	}
	sort.Strings(ids)
	result := make([]map[string]any, 0, len(ids))
	for _, id := range ids {
		result = append(result, rows[id])
	}
	return result
}
func practicalTypeKeySet(ids []string) []string {
	seen := map[string]bool{}
	for _, id := range ids {
		seen[id] = true
	}
	result := make([]string, 0, len(seen))
	for id := range seen {
		result = append(result, id)
	}
	sort.Strings(result)
	return result
}

// Only the original package and an exact physical expression span are inputs.
// The Go compiler observations produce the keys; caller IDs, types, declaration
// adapters and source buffers cannot replace any of the owned original inputs.
func (owner *practicalOriginalPackages) typeKeys(importPath string, span practicalSourceSpan) (practicalTypeKeyComponents, error) {
	route, err := owner.storageRoute(importPath, span)
	if err != nil {
		return practicalTypeKeyComponents{}, err
	}
	owner.mutex.Lock()
	defer owner.mutex.Unlock()
	if owner.closed || owner.inputs == nil || owner.loader == nil {
		return practicalTypeKeyComponents{}, fmt.Errorf("original type component owner is closed")
	}
	if owner.inputs.failed != nil {
		return practicalTypeKeyComponents{}, owner.inputs.failed
	}
	b := practicalTypeKeyBuilder{owner: owner, sources: map[string]practicalSourceBuffer{}, packages: map[string]map[string]any{}, declarations: map[string]map[string]any{}, types: map[string]map[string]any{}, packageIDs: map[string]string{}, underlying: map[string]string{}, aliases: map[string]string{}}
	rootPackage, err := b.packageKey(importPath)
	if err != nil {
		return practicalTypeKeyComponents{}, err
	}
	decls := make([]string, len(route.declarations))
	for i, d := range route.declarations {
		pkg, err := b.packageKey(d.packagePath)
		if err != nil {
			return practicalTypeKeyComponents{}, err
		}
		if d.moduleSourceID != owner.loader.packages[d.packagePath].moduleSourceID {
			return practicalTypeKeyComponents{}, fmt.Errorf("type declaration crossed its original module")
		}
		original, err := owner.storageOriginalSpan(d.packagePath, d.span)
		if err != nil {
			return practicalTypeKeyComponents{}, err
		}
		if original.object != d.source.object || !bytes.Equal(original.data, d.source.data) {
			return practicalTypeKeyComponents{}, fmt.Errorf("type declaration differs from owned original bytes")
		}
		kind := "type"
		if d.alias {
			kind = "alias_type"
		}
		row, err := practicalTypeKeySeal(practicalTypeDeclarationDomain, map[string]any{"schema": practicalTypeDeclarationSchema, "package_id": pkg, "source_object_id": d.span.sourceID, "kind": kind, "start_byte": d.span.start, "end_byte": d.span.end, "name": d.name})
		if err != nil {
			return practicalTypeKeyComponents{}, err
		}
		id := row["id"].(string)
		b.declarations[id] = row
		decls[i] = id
	}
	keys := make([]string, len(route.types))
	for i, node := range route.types {
		row := map[string]any{"schema": practicalSourceTypeSchema}
		switch node.kind {
		case "alias", "builtin_alias":
			keys[i] = keys[node.child]
			if node.kind == "alias" {
				b.aliases[decls[node.declaration]] = keys[i]
			}
			continue
		case "named":
			row["kind"] = "nominal"
			row["declaration_id"] = decls[node.declaration]
		case "bool", "string", "error", "word":
			name := node.builtin
			if node.kind == "error" {
				name = "error"
			}
			row["kind"] = "builtin"
			row["name"] = name
			row["width"] = nil
			row["signed"] = nil
			if node.kind == "word" {
				row["width"] = node.width
				row["signed"] = node.signed
			}
		case "array":
			row["kind"] = "array"
			row["length"] = node.length
			row["element"] = keys[node.child]
		case "slice":
			row["kind"] = "slice"
			row["element"] = keys[node.child]
		case "struct":
			row["kind"] = "struct"
			fields := make([]map[string]any, 0, len(node.fields))
			for _, field := range node.fields {
				var packageID any
				first, _ := utf8.DecodeRuneInString(field.name)
				if !unicode.IsUpper(first) {
					id, err := b.packageKey(field.packagePath)
					if err != nil {
						return practicalTypeKeyComponents{}, err
					}
					packageID = id
				}
				fields = append(fields, map[string]any{"name": field.name, "unexported_package_id": packageID, "type_id": keys[field.typeNode], "tag_bytes_hex": hex.EncodeToString([]byte(field.tag)), "embedded": field.embedded})
			}
			row["fields"] = fields
		default:
			return practicalTypeKeyComponents{}, fmt.Errorf("unknown original storage type component")
		}
		row, err = practicalTypeKeySeal(practicalSourceTypeDomain, row)
		if err != nil {
			return practicalTypeKeyComponents{}, err
		}
		id := row["id"].(string)
		keys[i] = id
		b.types[id] = row
		if node.kind == "named" {
			b.underlying[id] = keys[node.child]
		}
	}
	// Every observed type is retained as a root. This includes index expression
	// types and aliases that are transparent in the semantic type-key graph.
	roots := practicalTypeKeySet(keys)
	sources := make([]practicalSourceObject, 0, len(b.sources))
	buffers := make([]practicalSourceBuffer, 0, len(b.sources))
	for _, buffer := range b.sources {
		sources = append(sources, buffer.object)
		buffers = append(buffers, buffer)
	}
	sort.Slice(sources, func(i, j int) bool { return sources[i].ID < sources[j].ID })
	sort.Slice(buffers, func(i, j int) bool { return buffers[i].object.ID < buffers[j].object.ID })
	underlying := make([]map[string]any, 0, len(b.underlying))
	for _, id := range practicalSortedKeys(b.underlying) {
		underlying = append(underlying, map[string]any{"type_id": id, "underlying_type_id": b.underlying[id]})
	}
	aliases := make([]map[string]any, 0, len(b.aliases))
	for _, id := range practicalSortedKeys(b.aliases) {
		aliases = append(aliases, map[string]any{"declaration_id": id, "target_type_id": b.aliases[id]})
	}
	root := map[string]any{"schema": practicalTypeComponentsSchema, "source_objects": sources, "packages": practicalTypeKeyRows(b.packages), "declarations": practicalTypeKeyRows(b.declarations), "types": practicalTypeKeyRows(b.types), "underlying": underlying, "aliases": aliases, "package_roots": []string{rootPackage}, "type_roots": roots}
	hash, err := hashTypedCanonicalJSON(practicalTypeComponentsDomain, root)
	if err != nil {
		return practicalTypeKeyComponents{}, err
	}
	root["content_sha256"] = hash
	canonical, err := canonicalJSON(root)
	if err != nil {
		return practicalTypeKeyComponents{}, err
	}
	return practicalTypeKeyComponents{canonical: canonical, sources: buffers, nodeKeys: keys}, nil
}
func practicalSortedKeys(values map[string]string) []string {
	result := make([]string, 0, len(values))
	for id := range values {
		result = append(result, id)
	}
	sort.Strings(result)
	return result
}
