package mpk.java2vir;

import com.sun.source.tree.Tree;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/** Private, immutable lowering model. No compiler or host identity is serialized. */
final class JavaIr {
    private JavaIr() {}

    enum Type {
        BOOL, I32, I64, U32, U64;
        static Type source(ScalarType type) {
            return switch (type) { case BOOLEAN -> BOOL; case INT -> I32; case LONG -> I64; };
        }
        boolean signed() { return this == I32 || this == I64; }
        boolean unsigned() { return this == U32 || this == U64; }
        int width() { return this == I32 || this == U32 ? 32 : 64; }
        Type carrier() { return this == I32 ? U32 : U64; }
        Map<String, Object> json() {
            return this == BOOL ? Map.of("kind", "bool") : Map.of("kind", "bv", "width", width(), "signed", signed());
        }
    }
    record Value(String id, Type type) {
        Map<String, Object> reference() { return Map.of("var", id); }
        Map<String, Object> binding() { return Map.of("id", id, "type", type.json()); }
    }
    record Origin(SourceText source, Tree tree, long start, long end) {
        static Origin of(TreeInventory inventory, Tree tree) {
            var node = inventory.node(tree);
            return new Origin(node.source(), tree, node.start(), node.end());
        }
    }
    record Instruction(Value result, String kind, String op, List<Value> operands, Object literal,
                       String target, String function, String contractHash, List<String> checks, Origin origin) {
        Instruction { operands = List.copyOf(operands); checks = List.copyOf(checks); }
        Instruction rewrite(Value renamed, List<Value> values) {
            return new Instruction(renamed, kind, op, values, literal, target, function, contractHash, checks, origin);
        }
        Map<String, Object> json() {
            var value = new TreeMap<String, Object>();
            value.put("id", result.id()); value.put("kind", kind); value.put("type", result.type().json());
            value.put("safety_checks", checks.stream().map(check -> Map.of("kind", check)).toList());
            switch (kind) {
                case "Const" -> value.put("value", result.type() == Type.BOOL ? Map.of("bool", literal)
                        : Map.of("int", Map.of("value", literal, "width", result.type().width(), "signed", result.type().signed())));
                case "Copy" -> { value.put("target", target); value.put("value", operands.getFirst().reference()); }
                case "UnaryOp", "Convert" -> value.put("value", operands.getFirst().reference());
                case "BinOp" -> { value.put("lhs", operands.get(0).reference()); value.put("rhs", operands.get(1).reference()); }
                case "CallStatic" -> {
                    value.put("function", function); value.put("contract_hash", contractHash);
                    value.put("args", operands.stream().map(Value::reference).toList());
                }
                default -> throw fail("OPERATION");
            }
            if (kind.equals("UnaryOp") || kind.equals("BinOp")) value.put("op", op);
            return Map.copyOf(value);
        }
    }
    record Edge(String label, List<Value> arguments) { Edge { arguments = List.copyOf(arguments); } }
    // Branch edges are always false, then true; this is also the traversal order.
    record Terminator(String kind, Value condition, List<Value> values, List<Edge> edges, Origin origin) {
        Terminator { values = List.copyOf(values); edges = List.copyOf(edges); }
        Map<String, Object> json() {
            return switch (kind) {
                case "Return" -> Map.of("kind", kind, "values", values.stream().map(Value::reference).toList());
                case "Jump" -> Map.of("kind", kind, "label", edges.getFirst().label(),
                        "args", edges.getFirst().arguments().stream().map(Value::reference).toList());
                case "Branch" -> Map.of("kind", kind, "cond", condition.reference(),
                        "else_label", edges.get(0).label(), "else_args", edges.get(0).arguments().stream().map(Value::reference).toList(),
                        "then_label", edges.get(1).label(), "then_args", edges.get(1).arguments().stream().map(Value::reference).toList());
                default -> throw fail("CFG");
            };
        }
    }
    record Block(String label, List<Value> parameters, List<Instruction> instructions, Terminator terminator) {
        Block { parameters = List.copyOf(parameters); instructions = List.copyOf(instructions); }
        Map<String, Object> json() {
            return Map.of("label", label, "parameters", parameters.stream().map(Value::binding).toList(),
                    "instructions", instructions.stream().map(Instruction::json).toList(), "terminator", terminator.json());
        }
    }
    record Function(String id, String name, List<Value> parameters, Value result, List<Value> locals,
                    List<Block> blocks, Map<String, Object> contracts, List<String> features, Origin origin) {
        Function {
            parameters = List.copyOf(parameters); locals = List.copyOf(locals); blocks = List.copyOf(blocks);
            contracts = Map.copyOf(contracts); features = List.copyOf(features);
        }
        Map<String, Object> json(String unit) {
            return Map.of("id", id, "unit_id", unit, "name", name, "params", parameters.stream().map(Value::binding).toList(),
                    "results", List.of(result.binding()), "locals", locals.stream().map(Value::binding).toList(),
                    "blocks", blocks.stream().map(Block::json).toList(), "contracts", contracts, "features_used", features);
        }
    }
    record Program(JavaAdmission.Program admitted, List<Function> functions) {
        Program { functions = List.copyOf(functions); }
    }

    /** Both increments are checked before either counter changes or a node is retained. */
    static final class ClosureCounter {
        private long instructions, blocks;
        long instructions() { return instructions; }
        long blocks() { return blocks; }
    }
    static final class MethodCounter {
        private final ClosureCounter closure;
        private long instructions, blocks;
        MethodCounter(ClosureCounter closure) { this.closure = closure; }
        void instruction() {
            long local = FrontendLimits.add("instructions_per_method", instructions, 1, "lowering");
            long total = FrontendLimits.add("instructions_per_closure", closure.instructions, 1, "lowering");
            instructions = local; closure.instructions = total;
        }
        void block() {
            long local = FrontendLimits.add("cfg_blocks_per_method", blocks, 1, "lowering");
            long total = FrontendLimits.add("cfg_blocks_per_closure", closure.blocks, 1, "lowering");
            blocks = local; closure.blocks = total;
        }
        long instructions() { return instructions; }
        long blocks() { return blocks; }
    }
    static void parameterCount(int count) { if (count > 4096) throw fail("CFG"); }
    static FrontendFailure fail(String suffix) { return FrontendFailure.of("JAVA_LOWERING_" + suffix, "lowering"); }
}
