package main

import (
	"bytes"
	"fmt"
	"go/ast"
	"go/parser"
	"go/scanner"
	"go/token"
)

// This is the frozen practical capture budget. The predecessor keeps its
// separate maximumTypedSyntaxNodes value and unchanged loading behavior.
const maximumPracticalSyntaxNodes = 4_194_304

// Physical offsets refer to the complete original go_file object. They are
// independent of //line positions and of CR normalization in AST token text.
// This private locator is not a nominal type identity or a semantic proof.
type practicalSourceSpan struct {
	sourceID string
	start    int
	end      int
}

type practicalSyntaxNode struct {
	span practicalSourceSpan
	pos  token.Pos
	end  token.Pos
}

type practicalParsedSource struct {
	source practicalSourceBuffer
	file   *ast.File
}

func (parsed *practicalParsedSource) ownedView() *practicalParsedSource {
	// The AST is shared untrusted compiler data. Returned original bytes must
	// never share the cache's source-evidence allocation, on first or later reads.
	return &practicalParsedSource{
		source: practicalSourceBuffer{object: parsed.source.object, data: parsed.source.sourceBytes()},
		file:   parsed.file,
	}
}

// The original snapshot stays closed: parsing never opens a filesystem or
// tries a replacement source. Trees and their physical origins are private,
// untrusted compiler inputs. Package build selection, original type checking,
// export/compiler identity and source-correspondence proofs remain required.
type practicalSyntaxImage struct {
	snapshot practicalSourceSnapshot
	fileSet  *token.FileSet
	files    map[string]*practicalParsedSource
	nodes    map[ast.Node]practicalSyntaxNode
	count    int
	failed   error
}

func newPracticalSyntaxImage(snapshot practicalSourceSnapshot) (*practicalSyntaxImage, error) {
	if snapshot.contentSHA256 == "" || snapshot.contentSHA256 != practicalSnapshotDigest(snapshot) {
		return nil, fmt.Errorf("original practical syntax requires an intact sealed source snapshot")
	}
	return &practicalSyntaxImage{
		snapshot: snapshot, fileSet: token.NewFileSet(),
		files: make(map[string]*practicalParsedSource), nodes: make(map[ast.Node]practicalSyntaxNode),
	}, nil
}

func (image *practicalSyntaxImage) reject(err error) error {
	if image.failed == nil {
		image.failed = err
	}
	return image.failed
}

func (image *practicalSyntaxImage) parse(capability, relative string) (*practicalParsedSource, error) {
	if image.failed != nil {
		return nil, image.failed
	}
	source, err := image.snapshot.readSource(capability, relative, "go_file")
	if err != nil {
		return nil, image.reject(err)
	}
	if _, err := source.canonicalObject(); err != nil {
		return nil, image.reject(err)
	}
	if parsed, exists := image.files[source.object.ID]; exists {
		return parsed.ownedView(), nil
	}
	// A capability may itself contain '/'. The source ID, rather than a joined
	// capability/path spelling, gives every original token file a unique name.
	file, err := parser.ParseFile(image.fileSet, source.object.ID+".go", source.data,
		parser.ParseComments|parser.AllErrors|parser.SkipObjectResolution)
	if err != nil {
		return nil, image.reject(fmt.Errorf("original Go syntax %s/%s: %w", capability, relative, err))
	}
	tokenFile := image.fileSet.File(file.Pos())
	ends, err := practicalPhysicalTokenEnds(tokenFile, source.data)
	if err != nil {
		return nil, image.reject(err)
	}
	local := make(map[ast.Node]practicalSyntaxNode)
	inspect := func(node ast.Node) bool {
		if node == nil || err != nil {
			return false
		}
		if _, exists := local[node]; exists {
			return false
		}
		if image.count+len(local) >= maximumPracticalSyntaxNodes {
			err = fmt.Errorf("original practical syntax exceeds the complete tree-node budget")
			return false
		}
		span, spanErr := practicalPhysicalNodeSpan(tokenFile, source, node, ends)
		if spanErr != nil {
			err = spanErr
			return false
		}
		local[node] = practicalSyntaxNode{span: span, pos: node.Pos(), end: node.End()}
		return true
	}
	ast.Inspect(file, inspect)
	// ast.Walk deliberately skips File.Comments. Unattached comments are
	// nevertheless materialized original AST nodes; shared Doc/Comment links
	// must not count the same node twice.
	for _, comment := range file.Comments {
		ast.Inspect(comment, inspect)
	}
	if err != nil {
		return nil, image.reject(err)
	}
	parsed := &practicalParsedSource{source: source, file: file}
	image.files[source.object.ID] = parsed
	for node, binding := range local {
		image.nodes[node] = binding
	}
	image.count += len(local)
	return parsed.ownedView(), nil
}

// go/scanner and go/parser remove carriage returns from raw strings and
// comments. ast.BasicLit.End and ast.Comment.End consequently need an exact
// original-token end, also inherited by enclosing declarations/comments.
func practicalPhysicalTokenEnds(file *token.File, raw []byte) (map[int]int, error) {
	if file == nil || file.Size() != len(raw) {
		return nil, fmt.Errorf("original syntax token file does not match the full source bytes")
	}
	ends := make(map[int]int)
	var scan scanner.Scanner
	var scanErr error
	scan.Init(file, raw, func(_ token.Position, message string) {
		if scanErr == nil {
			scanErr = fmt.Errorf("original Go tokenization: %s", message)
		}
	}, scanner.ScanComments)
	for {
		pos, kind, literal := scan.Scan()
		if kind == token.EOF {
			break
		}
		start := file.Offset(pos)
		if kind != token.COMMENT && (kind != token.STRING || start >= len(raw) || raw[start] != '`') {
			continue
		}
		var end int
		switch {
		case kind == token.STRING:
			i := bytes.IndexByte(raw[start+1:], '`')
			if i < 0 {
				return nil, fmt.Errorf("original raw string has no complete closing token")
			}
			end = start + 1 + i + 1
		case bytes.HasPrefix(raw[start:], []byte("//")):
			i := bytes.IndexByte(raw[start:], '\n')
			end = len(raw)
			if i >= 0 {
				end = start + i
			}
		case bytes.HasPrefix(raw[start:], []byte("/*")):
			i := bytes.Index(raw[start+2:], []byte("*/"))
			if i < 0 {
				return nil, fmt.Errorf("original block comment has no complete closing token")
			}
			end = start + 2 + i + 2
		default:
			return nil, fmt.Errorf("original comment token does not match its source bytes")
		}
		logicalEnd := start + len(literal)
		if logicalEnd != end {
			ends[logicalEnd] = end
		}
	}
	return ends, scanErr
}

func practicalPhysicalNodeSpan(file *token.File, source practicalSourceBuffer, node ast.Node, ends map[int]int) (practicalSourceSpan, error) {
	base, size := token.Pos(file.Base()), file.Size()
	pos, end := node.Pos(), node.End()
	if empty, ok := node.(*ast.EmptyStmt); ok && empty.Implicit {
		end = pos // An inserted semicolon has no original byte, even at EOF.
	}
	if pos < base || end < pos || end > base+token.Pos(size) {
		return practicalSourceSpan{}, fmt.Errorf("original syntax node is outside its physical source file")
	}
	start, finish := int(pos-base), int(end-base)
	if actual, exists := ends[finish]; exists {
		finish = actual
	}
	if finish < start || finish > len(source.data) {
		return practicalSourceSpan{}, fmt.Errorf("original syntax token end is outside the complete source bytes")
	}
	return practicalSourceSpan{sourceID: source.object.ID, start: start, end: finish}, nil
}

func (image *practicalSyntaxImage) spanBytes(node ast.Node) (practicalSourceSpan, []byte, error) {
	if image.failed != nil {
		return practicalSourceSpan{}, nil, image.failed
	}
	binding, exists := image.nodes[node]
	if !exists || node.Pos() != binding.pos || node.End() != binding.end {
		return practicalSourceSpan{}, nil, image.reject(fmt.Errorf("syntax node has no unchanged original physical binding"))
	}
	source := image.files[binding.span.sourceID].source
	return binding.span, bytes.Clone(source.data[binding.span.start:binding.span.end]), nil
}

func (image *practicalSyntaxImage) identifier(node *ast.Ident) (string, error) {
	_, raw, err := image.spanBytes(node)
	if err != nil {
		return "", err
	}
	if string(raw) != node.Name {
		return "", image.reject(fmt.Errorf("syntax identifier differs from its original source token"))
	}
	return node.Name, nil
}
