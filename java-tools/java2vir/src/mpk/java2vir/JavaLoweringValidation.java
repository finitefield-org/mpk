package mpk.java2vir;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;
import static mpk.java2vir.JavaIr.*;

/** Checks the finished graph before any artifact can be published. */
final class JavaLoweringValidation {
    private JavaLoweringValidation() {}

    static void validate(Program program) {
        var admitted = program.admitted();
        if (program.functions().size() != admitted.closure().methods().size()
                || program.functions().size() != admitted.contracts().methods().size()) throw fail("CFG");
        var earlier = new HashMap<String, Function>();
        var counter = new ClosureCounter();
        for (int index = 0; index < program.functions().size(); index++) {
            Function function = program.functions().get(index);
            var source = admitted.closure().methods().get(index);
            if (!function.id().equals(source.id()) || !function.name().equals(source.name())
                    || function.parameters().size() != source.parameters().size() || function.locals().size() != source.locals().size()
                    || !function.result().equals(new Value("result0", Type.source(source.result())))
                    || !function.contracts().equals(admitted.contracts().methods().get(index).normalized())) throw fail("CFG");
            for (int n = 0; n < function.parameters().size(); n++)
                require(function.parameters().get(n).equals(new Value("arg" + n, Type.source(source.parameters().get(n).type()))), "CFG");
            var locals = new ArrayList<>(source.locals());
            locals.sort(java.util.Comparator.comparingLong(binding -> admitted.closure().origins().node(binding.declaration()).start()));
            for (int n = 0; n < function.locals().size(); n++)
                require(function.locals().get(n).equals(new Value("local" + n, Type.source(locals.get(n).type()))), "CFG");
            validateFunction(function, earlier, new MethodCounter(counter));
            if (earlier.putIfAbsent(function.id(), function) != null) throw fail("CFG");
        }
    }

    static List<String> features(List<Block> blocks) {
        var features = new TreeSet<String>();
        for (Block block : blocks) {
            if (block.terminator().kind().equals("Branch")) features.add("branch");
            for (Instruction instruction : block.instructions()) switch (instruction.kind()) {
                case "Copy" -> features.add("mutable_local");
                case "Convert" -> features.add("conversion");
                case "CallStatic" -> features.add("call_static");
                default -> { }
            }
        }
        return List.copyOf(features);
    }

    private static void validateFunction(Function function, Map<String, Function> earlier, MethodCounter counter) {
        require(!function.blocks().isEmpty() && function.locals().size() <= 65536 && function.parameters().size() <= 256, "CFG");
        require(function.features().equals(features(function.blocks())), "OPERATION");
        var blocks = new HashMap<String, Block>();
        var predecessors = new HashMap<String, List<String>>();
        var useCounts = new HashMap<String, Integer>();
        int p = 0, t = 0;
        for (Block block : function.blocks()) {
            counter.block();
            require(block.label().equals("bb" + blocks.size()) && block.terminator() != null, "CFG");
            blocks.put(block.label(), block); predecessors.put(block.label(), new ArrayList<>());
            parameterCount(block.parameters().size());
            for (Value value : block.parameters()) {
                require(value.id().equals("p" + p++), "CFG"); require(!value.type().unsigned(), "SHIFT_PATTERN");
            }
            for (Instruction instruction : block.instructions()) {
                counter.instruction();
                require(instruction.result().id().equals("t" + t++), "CFG");
                for (Value value : instruction.operands()) useCounts.merge(value.id(), 1, Integer::sum);
            }
            Terminator end = block.terminator();
            if (end.condition() != null) useCounts.merge(end.condition().id(), 1, Integer::sum);
            for (Value value : end.values()) useCounts.merge(value.id(), 1, Integer::sum);
            for (Edge edge : end.edges()) for (Value value : edge.arguments()) useCounts.merge(value.id(), 1, Integer::sum);
        }
        require(blocks.get("bb0").parameters().isEmpty(), "CFG");
        for (Block block : function.blocks()) for (Edge edge : block.terminator().edges()) {
            require(blocks.containsKey(edge.label()), "CFG");
            predecessors.get(edge.label()).add(block.label());
        }
        var queue = new ArrayDeque<String>(); queue.add("bb0");
        var bfs = new ArrayList<String>();
        var seen = new HashSet<String>();
        while (!queue.isEmpty()) {
            String label = queue.removeFirst();
            if (!seen.add(label)) continue;
            bfs.add(label);
            for (Edge edge : blocks.get(label).terminator().edges()) queue.add(edge.label());
        }
        require(bfs.equals(function.blocks().stream().map(Block::label).toList()), "CFG");
        require(predecessors.get("bb0").isEmpty(), "CFG");

        // A topological pass also checks definite assignment, independently of
        // BFS numbering (a join can be numbered before its deeper predecessor).
        var remaining = new HashMap<String, Integer>();
        for (var entry : predecessors.entrySet()) remaining.put(entry.getKey(), entry.getValue().size());
        queue.add("bb0");
        var initializedAtExit = new HashMap<String, Set<String>>();
        var globals = new HashMap<String, Type>();
        for (Value value : function.parameters()) globals.put(value.id(), value.type());
        var locals = new HashMap<String, Type>();
        for (Value value : function.locals()) locals.put(value.id(), value.type());
        int checked = 0;
        while (!queue.isEmpty()) {
            String label = queue.removeFirst();
            Block block = blocks.get(label);
            var initialized = new HashSet<String>();
            boolean first = true;
            for (String predecessor : predecessors.get(label)) {
                if (first) initialized.addAll(initializedAtExit.get(predecessor));
                else initialized.retainAll(initializedAtExit.get(predecessor));
                first = false;
            }
            var available = new HashMap<>(globals);
            for (String id : initialized) available.put(id, locals.get(id));
            for (Value value : block.parameters()) available.put(value.id(), value.type());
            for (Instruction instruction : block.instructions()) {
                for (Value operand : instruction.operands()) available(available, operand);
                instruction(instruction, locals, earlier);
                if (instruction.kind().equals("Copy")) {
                    initialized.add(instruction.target()); available.put(instruction.target(), instruction.result().type());
                }
                require(available.putIfAbsent(instruction.result().id(), instruction.result().type()) == null, "CFG");
            }
            shifts(block, useCounts);
            Terminator end = block.terminator();
            if (end.condition() != null) available(available, end.condition());
            for (Value value : end.values()) available(available, value);
            switch (end.kind()) {
                case "Return" -> require(end.condition() == null && end.edges().isEmpty()
                        && end.values().size() == 1 && end.values().getFirst().type() == function.result().type(), "CFG");
                case "Jump" -> require(end.condition() == null && end.values().isEmpty() && end.edges().size() == 1, "CFG");
                case "Branch" -> require(end.condition() != null && end.condition().type() == Type.BOOL
                        && end.values().isEmpty() && end.edges().size() == 2
                        && !end.edges().get(0).label().equals(end.edges().get(1).label()), "CFG");
                default -> throw fail("CFG");
            }
            for (Value value : end.values()) require(!value.type().unsigned(), "SHIFT_PATTERN");
            for (Edge edge : end.edges()) {
                var expected = blocks.get(edge.label()).parameters();
                require(expected.size() == edge.arguments().size(), "CFG");
                for (int n = 0; n < expected.size(); n++) {
                    Value value = edge.arguments().get(n);
                    available(available, value);
                    require(!value.type().unsigned(), "SHIFT_PATTERN");
                    require(value.type() == expected.get(n).type(), "CFG");
                }
            }
            initializedAtExit.put(label, initialized); checked++;
            for (Edge edge : end.edges()) if (remaining.merge(edge.label(), -1, Integer::sum) == 0) queue.add(edge.label());
        }
        require(checked == blocks.size(), "CFG");
    }

    static void checks(List<String> actual, List<String> expected) {
        if (actual.equals(expected)) return;
        if (actual.size() < expected.size()) throw fail("CHECK_MISSING");
        if (actual.size() > expected.size() || !new HashSet<>(actual).equals(new HashSet<>(expected))) throw fail("CHECK_EXTRA");
        throw fail("CHECK_ORDER");
    }

    private static void instruction(Instruction instruction, Map<String, Type> locals, Map<String, Function> earlier) {
        Type type = instruction.result().type();
        List<Value> operands = instruction.operands();
        String op = instruction.op();
        checks(instruction.checks(), instruction.kind().equals("BinOp") && ("bv_sdiv".equals(op) || "bv_srem".equals(op))
                ? List.of("divisor_nonzero") : List.of());
        require(instruction.kind().equals("Const") == (instruction.literal() != null), "OPERATION");
        require(instruction.kind().equals("Copy") == (instruction.target() != null), "OPERATION");
        require(instruction.kind().equals("CallStatic") ? instruction.function() != null && instruction.contractHash() != null
                : instruction.function() == null && instruction.contractHash() == null, "OPERATION");
        require((instruction.kind().equals("BinOp") || instruction.kind().equals("UnaryOp")) == (op != null), "OPERATION");
        switch (instruction.kind()) {
            case "Const" -> {
                require(operands.isEmpty() && !type.unsigned(), "OPERATION");
                if (type == Type.BOOL) require(instruction.literal() instanceof Boolean, "OPERATION");
                else {
                    require(instruction.literal() instanceof String, "OPERATION");
                    String value = (String) instruction.literal();
                    try {
                        long number = Long.parseLong(value);
                        require(Long.toString(number).equals(value) && (type == Type.I64 || number == (int) number), "OPERATION");
                    } catch (NumberFormatException error) { throw fail("OPERATION"); }
                }
            }
            case "Copy" -> {
                require(!type.unsigned() && operands.stream().noneMatch(value -> value.type().unsigned()), "SHIFT_PATTERN");
                require(operands.size() == 1 && operands.getFirst().type() == type && locals.get(instruction.target()) == type, "OPERATION");
            }
            case "Convert" -> {
                require(operands.size() == 1, "OPERATION");
                Type from = operands.getFirst().type();
                require(type != Type.BOOL && from != Type.BOOL && from != type, "OPERATION");
                require(from.signed() && type.signed() || from.width() == type.width() && from.unsigned() != type.unsigned(), "SHIFT_PATTERN");
            }
            case "UnaryOp" -> {
                require(operands.size() == 1 && operands.getFirst().type() == type, "OPERATION");
                require(type == Type.BOOL && op.equals("not") || type.signed() && List.of("bv_neg", "bv_not").contains(op), "OPERATION");
            }
            case "BinOp" -> {
                require(operands.size() == 2, "OPERATION");
                Type left = operands.get(0).type(), right = operands.get(1).type();
                switch (op) {
                    case "eq", "not_eq" -> require(type == Type.BOOL && left == right && !left.unsigned(), "OPERATION");
                    case "signed_lt", "signed_le", "signed_gt", "signed_ge" -> require(type == Type.BOOL && left.signed() && left == right, "OPERATION");
                    case "bv_add", "bv_sub", "bv_mul", "bv_sdiv", "bv_srem", "bv_and", "bv_or", "bv_xor" ->
                            require(type.signed() && left == type && right == type, "OPERATION");
                    case "bv_shl", "bv_ashr", "bv_lshr" -> require(left == type && right == Type.I32
                            && (op.equals("bv_lshr") ? type.unsigned() : type.signed()), "SHIFT_PATTERN");
                    default -> throw fail("OPERATION");
                }
            }
            case "CallStatic" -> {
                require(!type.unsigned() && operands.stream().noneMatch(value -> value.type().unsigned()), "SHIFT_PATTERN");
                Function callee = earlier.get(instruction.function());
                require(callee != null && callee.result().type() == type && operands.size() == callee.parameters().size()
                        && instruction.contractHash().equals(callee.contracts().get("contract_hash")), "OPERATION");
                for (int n = 0; n < operands.size(); n++) require(operands.get(n).type() == callee.parameters().get(n).type(), "OPERATION");
            }
            default -> throw fail("OPERATION");
        }
    }

    private static void shifts(Block block, Map<String, Integer> uses) {
        List<Instruction> instructions = block.instructions();
        var carrierInstructions = new HashSet<String>();
        for (int index = 0; index < instructions.size(); index++) {
            Instruction shift = instructions.get(index);
            if (!shift.kind().equals("BinOp") || !List.of("bv_shl", "bv_ashr", "bv_lshr").contains(shift.op())) continue;
            boolean logical = shift.op().equals("bv_lshr");
            int start = index - (logical ? 3 : 2);
            require(start >= 0 && (!logical || index + 1 < instructions.size()), "SHIFT_PATTERN");
            Instruction mask = instructions.get(start), and = instructions.get(start + 1);
            require(mask.kind().equals("Const") && mask.result().type() == Type.I32
                    && (shift.result().type().width() == 32 ? "31" : "63").equals(mask.literal()), "SHIFT_PATTERN");
            require(and.kind().equals("BinOp") && "bv_and".equals(and.op()) && and.result().type() == Type.I32
                    && and.operands().size() == 2 && and.operands().get(1).equals(mask.result())
                    && shift.operands().get(1).equals(and.result()), "SHIFT_PATTERN");
            int end = logical ? index + 1 : index;
            for (int n = start; n <= end; n++) require(shift.origin().equals(instructions.get(n).origin()), "SHIFT_PATTERN");
            if (logical) {
                Instruction before = instructions.get(index - 1), after = instructions.get(index + 1);
                require(before.kind().equals("Convert") && before.result().type() == shift.result().type()
                        && before.operands().getFirst().type().signed()
                        && before.operands().getFirst().type().width() == shift.result().type().width()
                        && shift.operands().get(0).equals(before.result()), "SHIFT_PATTERN");
                require(after.kind().equals("Convert") && after.result().type() == before.operands().getFirst().type()
                        && after.operands().getFirst().equals(shift.result())
                        && uses.getOrDefault(before.result().id(), 0) == 1 && uses.getOrDefault(shift.result().id(), 0) == 1, "SHIFT_PATTERN");
                carrierInstructions.add(before.result().id()); carrierInstructions.add(shift.result().id()); carrierInstructions.add(after.result().id());
            }
        }
        for (Instruction instruction : instructions) if (instruction.result().type().unsigned()
                || instruction.operands().stream().anyMatch(value -> value.type().unsigned()))
            require(carrierInstructions.contains(instruction.result().id()), "SHIFT_PATTERN");
    }

    private static void available(Map<String, Type> available, Value value) { require(available.get(value.id()) == value.type(), "CFG"); }
    private static void require(boolean condition, String code) { if (!condition) throw fail(code); }
}
