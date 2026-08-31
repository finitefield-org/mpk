package mpk.java2vir;

import com.sun.source.tree.CompilationUnitTree;
import com.sun.source.tree.ClassTree;
import com.sun.source.tree.ExpressionTree;
import com.sun.source.tree.IdentifierTree;
import com.sun.source.tree.LiteralTree;
import com.sun.source.tree.MemberSelectTree;
import com.sun.source.tree.MethodTree;
import com.sun.source.tree.ModifiersTree;
import com.sun.source.tree.PrimitiveTypeTree;
import com.sun.source.tree.Tree;
import com.sun.source.tree.VariableTree;
import com.sun.source.util.TreePath;
import com.sun.source.util.TreeScanner;
import com.sun.source.util.Trees;
import java.io.IOException;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import javax.lang.model.element.Element;
import javax.lang.model.type.TypeKind;
import javax.lang.model.type.TypeMirror;
import javax.tools.JavaFileObject;

/** Bounded public-tree inventory. Snapshot first; source subset gates precede comparison. */
final class TreeInventory {
    /** javac may wrap unit source objects. Bind once, in the supplied source order,
     * using the captured immutable CharSequence identity, not URI equality alone.
     * Diagnostics are separately required to return the original source object.
     */
    static final class Origins {
        private record Origin(SourceText source, JavaFileObject compilerSource) {}
        private final Map<CompilationUnitTree, Origin> units = new IdentityHashMap<>();
        Origins(List<CompilationUnitTree> parsed, List<SourceText> sources) throws IOException {
            if (parsed.size() != sources.size()) throw adapter();
            var seen = Collections.newSetFromMap(new IdentityHashMap<JavaFileObject, Boolean>());
            for (int i = 0; i < parsed.size(); i++) {
                var unit = parsed.get(i);
                var file = unit.getSourceFile();
                var source = sources.get(i);
                if (file == null || !seen.add(file) || units.containsKey(unit)
                        || file.getKind() != JavaFileObject.Kind.SOURCE
                        || !source.toUri().equals(file.toUri())
                        || file.getCharContent(false) != source.getCharContent(false)) throw adapter();
                units.put(unit, new Origin(source, file));
            }
        }
        SourceText source(CompilationUnitTree unit) {
            Origin origin = units.get(unit);
            if (origin == null || unit.getSourceFile() != origin.compilerSource()) throw adapter();
            return origin.source();
        }
    }
    record Node(Tree tree, Tree.Kind kind, TreePath path, SourceText source, long start, long end,
                List<Tree> children, Map<String, Object> facts, TypeMirror type, Element element) {
        String spelling() {
            source.span(start, end, false, "typecheck");
            return source.text().substring((int) start, (int) end);
        }
    }
    private record Pending(Tree tree, TreePath path, CompilationUnitTree unit, SourceText source, int depth) {}
    private final List<Node> ordered;
    private final Map<Tree, Node> byTree;

    private TreeInventory(List<Node> ordered, Map<Tree, Node> byTree) {
        this.ordered = List.copyOf(ordered);
        this.byTree = Collections.unmodifiableMap(new IdentityHashMap<>(byTree));
    }
    List<Node> nodes() { return ordered; }
    Node node(Tree tree) {
        Node result = byTree.get(tree);
        if (result == null) throw adapter();
        return result;
    }

    static TreeInventory snapshot(Trees trees, List<CompilationUnitTree> units,
                                  Origins origins, boolean analyzed) {
        var ordered = new ArrayList<Node>();
        var seen = new IdentityHashMap<Tree, Node>();
        var pending = new ArrayDeque<Pending>();
        for (int i = units.size() - 1; i >= 0; i--) {
            CompilationUnitTree unit = units.get(i);
            SourceText source = origins.source(unit);
            pending.push(new Pending(unit, new TreePath(unit), unit, source, 1));
        }
        while (!pending.isEmpty()) {
            Pending item = pending.pop();
            String phase = analyzed ? "typecheck" : "source";
            FrontendLimits.check("syntax_depth", item.depth(), phase);
            FrontendLimits.check("syntax_nodes", (long) ordered.size() + 1, phase);
            Tree tree = item.tree();
            if (seen.containsKey(tree)) throw adapter();
            Tree.Kind kind = requireKnownKind(tree);
            long start = trees.getSourcePositions().getStartPosition(item.unit(), tree);
            long end = trees.getSourcePositions().getEndPosition(item.unit(), tree);
            var children = children(tree, (long) ordered.size() + pending.size() + 1, item.depth(), phase);
            // Raw spelling stays in SourceText; spans/facts are copied before javac can mutate a tree.
            TypeMirror type = analyzed ? trees.getTypeMirror(item.path()) : null;
            Element element = analyzed ? trees.getElement(item.path()) : null;
            Node node = new Node(tree, kind, item.path(), item.source(), start, end,
                    children, facts(tree), type, element);
            seen.put(tree, node);
            ordered.add(node);
            for (int i = children.size() - 1; i >= 0; i--) {
                Tree child = children.get(i);
                // Count pending nodes as well, before an excess child is retained.
                FrontendLimits.check("syntax_nodes", (long) ordered.size() + pending.size() + 1, phase);
                FrontendLimits.check("syntax_depth", (long) item.depth() + 1, phase);
                pending.push(new Pending(child, new TreePath(item.path(), child), item.unit(), item.source(), item.depth() + 1));
            }
        }
        return new TreeInventory(ordered, seen);
    }

    static List<Tree> children(Tree tree, long retained, int depth, String phase) {
        var children = new ArrayList<Tree>();
        tree.accept(new TreeScanner<Void, Void>() {
            @Override public Void scan(Tree child, Void unused) {
                if (child != null) {
                    FrontendLimits.check("syntax_nodes", retained + children.size() + 1, phase);
                    FrontendLimits.check("syntax_depth", (long) depth + 1, phase);
                    children.add(child);
                }
                return null; // Direct children only; the explicit stack controls traversal/depth.
            }
        }, null);
        return List.copyOf(children);
    }

    private static Map<String, Object> facts(Tree tree) {
        if (tree instanceof ClassTree declaration) return Map.of("name", declaration.getSimpleName().toString());
        if (tree instanceof IdentifierTree identifier) return Map.of("name", identifier.getName().toString());
        if (tree instanceof MemberSelectTree member) return Map.of("name", member.getIdentifier().toString());
        if (tree instanceof VariableTree variable) return Map.of("name", variable.getName().toString());
        if (tree instanceof MethodTree method) return Map.of("name", method.getName().toString());
        if (tree instanceof ModifiersTree modifiers)
            return Map.of("flags", modifiers.getFlags().stream().map(Object::toString).sorted().toList());
        if (tree instanceof PrimitiveTypeTree primitive) return Map.of("primitive", primitive.getPrimitiveTypeKind().name());
        if (tree instanceof LiteralTree literal)
            return Map.of("literal", String.valueOf(literal.getValue()));
        return Map.of();
    }

    /** Called by T05 only after a raw subtree has passed its source admission gate. */
    void requireUnchanged(Tree root, TreeInventory after) {
        var pending = new ArrayDeque<Tree>();
        pending.push(root);
        while (!pending.isEmpty()) {
            Tree tree = pending.pop();
            Node old = node(tree);
            Node now = after.node(tree);
            if (old.kind() != now.kind() || old.start() != now.start() || old.end() != now.end()
                    || old.source() != now.source() || !old.children().equals(now.children())
                    || !old.facts().equals(now.facts())) throw adapter();
            if (old.kind() == Tree.Kind.ERRONEOUS) throw adapter();
            // Empty modifier nodes are permitted to have NOPOS. No other accepted
            // node may acquire a fabricated or missing origin.
            if (!(tree instanceof ModifiersTree && old.facts().get("flags").equals(List.of())))
                old.source().span(old.start(), old.end(), false, "typecheck");
            if (now.type() != null || tree instanceof ExpressionTree || tree instanceof VariableTree
                    || tree instanceof MethodTree || tree instanceof ClassTree) requireKnownType(now.type());
            if ((tree instanceof IdentifierTree || tree instanceof MemberSelectTree || tree instanceof VariableTree
                    || tree instanceof MethodTree || tree instanceof ClassTree) && now.element() == null) throw adapter();
            old.children().forEach(pending::push);
        }
    }

    static void requireKnownType(TypeMirror type) {
        if (type == null || type.getKind() == TypeKind.ERROR || type.getKind() == TypeKind.OTHER)
            throw adapter();
    }

    static Tree.Kind requireKnownKind(Tree tree) {
        Tree.Kind kind = tree.getKind();
        if (kind == null || kind == Tree.Kind.OTHER) throw adapter();
        return kind;
    }

    private static FrontendFailure adapter() { return FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", "typecheck"); }
}
