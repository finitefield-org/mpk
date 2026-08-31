package mpk.java2vir;

import com.sun.source.tree.ExpressionStatementTree;
import com.sun.source.tree.ExpressionTree;
import com.sun.source.tree.IfTree;
import com.sun.source.tree.ReturnTree;
import com.sun.source.tree.VariableTree;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.Map;
import static mpk.java2vir.JavaIr.*;

/** Only original captured source objects and their exact UTF-16 boundaries are admitted. */
final class JavaSourceMaps {
    private JavaSourceMaps() {}

    static Map<String, Object> emit(Program program, String virHash) {
        var entries = new ArrayList<Map<String, Object>>();
        String unit = program.admitted().closure().selection().compilation();
        var functions = new ArrayList<>(program.functions());
        functions.sort(Comparator.comparing(Function::id));
        for (Function function : functions) {
            var method = program.admitted().closure().methods().stream().filter(source -> source.id().equals(function.id())).findFirst().orElseThrow();
            var owner = program.admitted().closure().origins().node(method.declaration());
            entries.add(entry(program, owner, function.origin(), Map.of("kind", "function", "unit_id", unit, "function_id", function.id())));
            if (function.origin().tree() != method.declaration()) throw failure("RANGE");
            for (Block block : function.blocks()) for (Instruction instruction : block.instructions()) {
                entries.add(entry(program, owner, instruction.origin(), Map.of("kind", "instruction", "unit_id", unit,
                        "function_id", function.id(), "block", block.label(), "instruction", instruction.result().id())));
                var tree = instruction.origin().tree();
                if (instruction.kind().equals("Copy") ? !(tree instanceof VariableTree || tree instanceof ExpressionStatementTree)
                        : !(tree instanceof ExpressionTree)) throw failure("RANGE");
            }
            for (Block block : function.blocks()) {
                Terminator end = block.terminator();
                entries.add(entry(program, owner, end.origin(),
                        Map.of("kind", "terminator", "unit_id", unit, "function_id", function.id(), "block", block.label())));
                var tree = end.origin().tree();
                boolean valid = switch (end.kind()) {
                    case "Return" -> tree instanceof ReturnTree;
                    case "Branch" -> tree instanceof ExpressionTree;
                    case "Jump" -> tree instanceof ExpressionTree || tree instanceof IfTree;
                    default -> false;
                };
                if (!valid) throw failure("RANGE");
            }
        }
        return CanonicalJson.artifact(Map.of("schema", "mpk.source_map.v1", "semantic_context", Protocol.semanticContext(),
                "source_ir_schema", "mpk.vir.v1", "source_ir_hash", virHash, "entries", entries),
                "source_map_hash", "MPK-SOURCE-MAP-1.0", "source_map_canonical_bytes");
    }

    private static Map<String, Object> entry(Program program, TreeInventory.Node owner, Origin origin, Map<String, Object> reference) {
        var closure = program.admitted().closure();
        if (origin == null || origin.tree() == null || closure.sources().stream().noneMatch(source -> source == origin.source())) throw failure("EXTERNAL");
        TreeInventory.Node original;
        try { original = closure.origins().node(origin.tree()); }
        catch (FrontendFailure error) { throw failure("EXTERNAL"); }
        if (original.source() != origin.source()) throw failure("EXTERNAL");
        Map<String, Object> range = range(origin.source(), origin.start(), origin.end());
        // Valid coordinates from another token are not valid provenance for this node.
        if (origin.start() != original.start() || origin.end() != original.end()
                || original.source() != owner.source() || original.start() < owner.start() || original.end() > owner.end()) throw failure("RANGE");
        return Map.of("reference", reference, "origin", range);
    }

    static Map<String, Object> range(SourceText source, long start, long end) {
        if (source == null) throw failure("EXTERNAL");
        if (start < 0 || start >= end || end > source.text().length()) throw failure("RANGE");
        int first, last;
        try { first = source.byteOffset(start); last = source.byteOffset(end); }
        catch (FrontendFailure error) { throw failure("UTF16"); }
        return Map.of("kind", "source", "input_kind", "source", "normalized_path", source.path(), "start", first, "end", last);
    }
    private static FrontendFailure failure(String code) { return FrontendFailure.of("JAVA_SOURCE_MAP_" + code, "emission"); }
}
