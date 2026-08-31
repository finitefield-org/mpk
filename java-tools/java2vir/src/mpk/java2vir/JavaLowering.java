package mpk.java2vir;

import com.sun.source.tree.*;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.IdentityHashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.TreeSet;
import javax.lang.model.element.VariableElement;
import static mpk.java2vir.JavaIr.*;

/** Source-tree lowering only; attribution and the conservative closure are already frozen. */
final class JavaLowering {
    private JavaLowering() {}

    static Program lower(JavaAdmission.Program admitted) {
        var counters = new ClosureCounter();
        var contracts = new TreeMap<String, Map<String, Object>>();
        for (var attached : admitted.contracts().methods())
            contracts.put((String) attached.normalized().get("function_id"), attached.normalized());
        var functions = new ArrayList<Function>();
        for (var method : admitted.closure().methods())
            functions.add(new Builder(admitted.closure(), method, contracts, new MethodCounter(counters)).build());
        Program result = new Program(admitted, functions);
        JavaLoweringValidation.validate(result);
        return result;
    }

    private static final class Draft {
        final String label;
        final List<Value> parameters = new ArrayList<>();
        final List<Instruction> instructions = new ArrayList<>();
        Terminator terminator;
        Draft(String label) { this.label = label; }
    }
    private record Expression(Draft block, Value value) {}
    private record Demand(Draft block, String id) {}

    private static final class Builder {
        final JavaSubset.Closure closure;
        final JavaSubset.Method method;
        final Map<String, Map<String, Object>> contracts;
        final MethodCounter counter;
        final List<Draft> drafts = new ArrayList<>();
        final List<Value> parameters = new ArrayList<>(), locals = new ArrayList<>();
        final Map<VariableElement, Value> bindings = new IdentityHashMap<>();
        final Map<VariableTree, Value> declarations = new IdentityHashMap<>();
        int parameterSequence;

        Builder(JavaSubset.Closure closure, JavaSubset.Method method,
                Map<String, Map<String, Object>> contracts, MethodCounter counter) {
            this.closure = closure; this.method = method; this.contracts = contracts; this.counter = counter;
            for (var binding : method.parameters()) {
                var value = new Value("arg" + parameters.size(), Type.source(binding.type()));
                parameters.add(value); bindings.put(binding.element(), value);
            }
            var ordered = new ArrayList<>(method.locals());
            ordered.sort(Comparator.comparingLong(binding -> closure.origins().node(binding.declaration()).start()));
            if (ordered.size() > 65536) throw fail("CFG");
            for (var binding : ordered) {
                var value = new Value("local" + locals.size(), Type.source(binding.type()));
                locals.add(value); bindings.put(binding.element(), value); declarations.put(binding.declaration(), value);
            }
        }

        Function build() {
            Draft entry = block();
            if (statement(entry, method.declaration().getBody()) != null) throw fail("CFG");
            List<Block> blocks = canonicalize(entry);
            return new Function(method.id(), method.name(), parameters, new Value("result0", Type.source(method.result())),
                    locals, blocks, contracts.get(method.id()), JavaLoweringValidation.features(blocks), origin(method.declaration()));
        }
        Draft block() { counter.block(); var block = new Draft("b" + drafts.size()); drafts.add(block); return block; }
        Origin origin(Tree tree) { return Origin.of(closure.origins(), tree); }
        Type type(ExpressionTree tree) {
            ScalarType type = method.expressionTypes().get(tree);
            if (type == null) throw fail("OPERATION");
            return Type.source(type);
        }
        Value binding(IdentifierTree tree) {
            var binding = method.variableBindings().get(tree);
            if (binding == null || !bindings.containsKey(binding.element())) throw fail("CFG");
            return bindings.get(binding.element());
        }
        Value emit(Draft block, String kind, Type type, String op, List<Value> values, Object literal,
                   String target, String function, Origin origin) {
            if (block.terminator != null) throw fail("CFG");
            counter.instruction();
            Value result = new Value("v" + (counter.instructions() - 1), type);
            String hash = function == null ? null : (String) contracts.get(function).get("contract_hash");
            List<String> checks = "bv_sdiv".equals(op) || "bv_srem".equals(op) ? List.of("divisor_nonzero") : List.of();
            block.instructions.add(new Instruction(result, kind, op, values, literal, target, function, hash, checks, origin));
            return result;
        }
        Value constant(Draft block, Type type, Object value, Origin origin) {
            return emit(block, "Const", type, null, List.of(), value, null, null, origin);
        }
        Value convert(Draft block, Value value, Type type, Origin origin) {
            return value.type() == type ? value : emit(block, "Convert", type, null, List.of(value), null, null, null, origin);
        }
        void terminate(Draft block, Terminator terminator) {
            if (block.terminator != null) throw fail("CFG");
            block.terminator = terminator;
        }
        void jump(Draft from, Draft to, List<Value> values, Origin origin) {
            terminate(from, new Terminator("Jump", null, List.of(), List.of(new Edge(to.label, values)), origin));
        }
        void branch(Draft from, Value condition, Draft no, Draft yes, Origin origin) {
            terminate(from, new Terminator("Branch", condition, List.of(),
                    List.of(new Edge(no.label, List.of()), new Edge(yes.label, List.of())), origin));
        }
        Value parameter(Draft block, Type type) {
            parameterCount(block.parameters.size() + 1);
            var value = new Value("q" + parameterSequence++, type);
            block.parameters.add(value);
            return value;
        }

        Draft statement(Draft current, StatementTree tree) {
            if (current == null) throw fail("CFG");
            return switch (tree.getKind()) {
                case BLOCK -> {
                    for (StatementTree next : ((BlockTree) tree).getStatements()) current = statement(current, next);
                    yield current;
                }
                case VARIABLE -> {
                    var declaration = (VariableTree) tree;
                    Value target = declarations.get(declaration);
                    if (target == null) throw fail("CFG");
                    Expression value = expression(current, declaration.getInitializer());
                    Value converted = convert(value.block(), value.value(), target.type(), origin(declaration.getInitializer()));
                    emit(value.block(), "Copy", target.type(), null, List.of(converted), null, target.id(), null, origin(tree));
                    yield value.block();
                }
                case EXPRESSION_STATEMENT -> {
                    var assignment = (AssignmentTree) ((ExpressionStatementTree) tree).getExpression();
                    Value target = binding((IdentifierTree) assignment.getVariable());
                    Expression value = expression(current, assignment.getExpression());
                    Value converted = convert(value.block(), value.value(), target.type(), origin(assignment.getExpression()));
                    emit(value.block(), "Copy", target.type(), null, List.of(converted), null, target.id(), null, origin(tree));
                    yield value.block();
                }
                case RETURN -> {
                    var returned = ((ReturnTree) tree).getExpression();
                    Expression value = expression(current, returned);
                    Value converted = convert(value.block(), value.value(), Type.source(method.result()), origin(returned));
                    terminate(value.block(), new Terminator("Return", null, List.of(converted), List.of(), origin(tree)));
                    yield null;
                }
                case IF -> {
                    var conditional = (IfTree) tree;
                    Expression condition = expression(current, conditional.getCondition());
                    Draft no = block(), yes = block();
                    branch(condition.block(), condition.value(), no, yes, origin(conditional.getCondition()));
                    Draft noEnd = conditional.getElseStatement() == null ? no : statement(no, conditional.getElseStatement());
                    Draft yesEnd = statement(yes, conditional.getThenStatement());
                    if (noEnd == null && yesEnd == null) yield null;
                    Draft join = block();
                    if (noEnd != null) jump(noEnd, join, List.of(), origin(tree));
                    if (yesEnd != null) jump(yesEnd, join, List.of(), origin(tree));
                    yield join;
                }
                default -> throw fail("CFG");
            };
        }

        Expression expression(Draft current, ExpressionTree tree) {
            Type result = type(tree);
            Origin owner = origin(tree);
            return switch (tree.getKind()) {
                case IDENTIFIER -> new Expression(current, binding((IdentifierTree) tree));
                case PARENTHESIZED -> expression(current, ((ParenthesizedTree) tree).getExpression());
                case BOOLEAN_LITERAL, INT_LITERAL, LONG_LITERAL -> new Expression(current, constant(current, result,
                        result == Type.BOOL ? ((LiteralTree) tree).getValue() : method.integers().get((LiteralTree) tree), owner));
                case TYPE_CAST -> {
                    Expression value = expression(current, ((TypeCastTree) tree).getExpression());
                    yield new Expression(value.block(), convert(value.block(), value.value(), result, owner));
                }
                case UNARY_MINUS, LOGICAL_COMPLEMENT, BITWISE_COMPLEMENT -> {
                    Expression value = expression(current, ((UnaryTree) tree).getExpression());
                    String op = switch (tree.getKind()) { case UNARY_MINUS -> "bv_neg"; case LOGICAL_COMPLEMENT -> "not"; default -> "bv_not"; };
                    yield new Expression(value.block(), emit(value.block(), "UnaryOp", result, op, List.of(value.value()), null, null, null, owner));
                }
                case CONDITIONAL_EXPRESSION -> {
                    var conditional = (ConditionalExpressionTree) tree;
                    yield choice(current, tree, conditional.getCondition(), conditional.getFalseExpression(), conditional.getTrueExpression());
                }
                case CONDITIONAL_AND, CONDITIONAL_OR -> {
                    var binary = (BinaryTree) tree;
                    boolean and = tree.getKind() == Tree.Kind.CONDITIONAL_AND;
                    yield choice(current, tree, binary.getLeftOperand(), and ? null : binary.getRightOperand(), and ? binary.getRightOperand() : null);
                }
                case METHOD_INVOCATION -> {
                    var call = (MethodInvocationTree) tree;
                    var arguments = new ArrayList<Value>();
                    for (ExpressionTree argument : call.getArguments()) {
                        Expression value = expression(current, argument);
                        current = value.block(); arguments.add(value.value());
                    }
                    yield new Expression(current, emit(current, "CallStatic", result, null, arguments, null, null, method.callTargets().get(call), owner));
                }
                default -> {
                    if (!(tree instanceof BinaryTree binary)) throw fail("OPERATION");
                    Expression left = expression(current, binary.getLeftOperand());
                    Expression right = expression(left.block(), binary.getRightOperand());
                    Draft end = right.block();
                    Value lhs = left.value(), rhs = right.value();
                    String op = operation(tree.getKind());
                    if (List.of("bv_shl", "bv_ashr", "bv_lshr").contains(op)) {
                        Value mask = constant(end, Type.I32, lhs.type() == Type.I32 ? "31" : "63", owner);
                        rhs = emit(end, "BinOp", Type.I32, "bv_and", List.of(rhs, mask), null, null, null, owner);
                        if (op.equals("bv_lshr")) lhs = convert(end, lhs, lhs.type().carrier(), owner);
                        Value shifted = emit(end, "BinOp", lhs.type(), op, List.of(lhs, rhs), null, null, null, owner);
                        yield new Expression(end, convert(end, shifted, result, owner));
                    }
                    yield new Expression(end, emit(end, "BinOp", result, op, List.of(lhs, rhs), null, null, null, owner));
                }
            };
        }

        Expression choice(Draft current, ExpressionTree owner, ExpressionTree conditionTree, ExpressionTree noTree, ExpressionTree yesTree) {
            Expression condition = expression(current, conditionTree);
            Draft no = block(), yes = block();
            branch(condition.block(), condition.value(), no, yes, origin(owner));
            Expression noValue = noTree == null ? new Expression(no, constant(no, Type.BOOL, false, origin(owner))) : expression(no, noTree);
            Expression yesValue = yesTree == null ? new Expression(yes, constant(yes, Type.BOOL, true, origin(owner))) : expression(yes, yesTree);
            Draft join = block();
            Value joined = parameter(join, type(owner));
            jump(noValue.block(), join, List.of(noValue.value()), origin(owner));
            jump(yesValue.block(), join, List.of(yesValue.value()), origin(owner));
            return new Expression(join, joined);
        }

        List<Block> canonicalize(Draft entry) {
            var byLabel = new HashMap<String, Draft>();
            for (Draft block : drafts) byLabel.put(block.label, block);
            var queue = new ArrayDeque<Draft>(); queue.add(entry);
            var visited = new HashSet<String>();
            var order = new ArrayList<Draft>();
            var predecessors = new HashMap<Draft, List<Draft>>();
            for (Draft block : drafts) predecessors.put(block, new ArrayList<>());
            while (!queue.isEmpty()) {
                Draft block = queue.removeFirst();
                if (!visited.add(block.label)) continue;
                order.add(block);
                if (block.terminator == null) throw fail("CFG");
                for (Edge edge : block.terminator.edges()) {
                    Draft target = byLabel.get(edge.label());
                    if (target == null) throw fail("CFG");
                    queue.add(target); predecessors.get(target).add(block);
                }
            }
            if (order.size() != drafts.size()) throw fail("CFG");

            // VIR temporaries are block-scoped, even when their defining block
            // dominates the use. Thread live values across every intervening edge.
            var owners = new HashMap<String, Draft>();
            var values = new LinkedHashMap<String, Value>();
            var rank = new HashMap<String, Integer>();
            var needed = new HashMap<Draft, TreeSet<String>>();
            for (Draft block : order) {
                var defined = new ArrayList<Value>(block.parameters);
                for (Instruction instruction : block.instructions) defined.add(instruction.result());
                for (Value value : defined) {
                    owners.put(value.id(), block); rank.put(value.id(), rank.size()); values.put(value.id(), value);
                }
                needed.put(block, new TreeSet<>(Comparator.comparingInt(rank::get)));
            }
            var demands = new ArrayDeque<Demand>();
            for (Draft block : order) for (Value use : uses(block)) if (owners.containsKey(use.id())) demands.add(new Demand(block, use.id()));
            while (!demands.isEmpty()) {
                Demand demand = demands.removeFirst();
                if (owners.get(demand.id()) == demand.block() || needed.get(demand.block()).contains(demand.id())) continue;
                if (predecessors.get(demand.block()).isEmpty()) throw fail("CFG");
                parameterCount(demand.block().parameters.size() + needed.get(demand.block()).size() + 1);
                needed.get(demand.block()).add(demand.id());
                for (Draft predecessor : predecessors.get(demand.block())) demands.add(new Demand(predecessor, demand.id()));
            }
            var carried = new HashMap<Draft, Map<String, Value>>();
            for (Draft block : order) {
                var map = new HashMap<String, Value>();
                for (String id : needed.get(block)) map.put(id, parameter(block, values.get(id).type()));
                carried.put(block, map);
            }
            var labels = new HashMap<String, String>();
            var renamed = new HashMap<String, Value>();
            for (Value value : parameters) renamed.put(value.id(), value);
            for (Value value : locals) renamed.put(value.id(), value);
            int p = 0, t = 0;
            for (Draft block : order) {
                labels.put(block.label, "bb" + labels.size());
                for (Value value : block.parameters) renamed.put(value.id(), new Value("p" + p++, value.type()));
                for (Instruction instruction : block.instructions) renamed.put(instruction.result().id(), new Value("t" + t++, instruction.result().type()));
            }
            var result = new ArrayList<Block>();
            for (Draft block : order) {
                java.util.function.UnaryOperator<Value> rewrite = value -> renamed.get(carried.get(block).getOrDefault(value.id(), value).id());
                var instructions = block.instructions.stream().map(instruction -> instruction.rewrite(renamed.get(instruction.result().id()),
                        instruction.operands().stream().map(rewrite).toList())).toList();
                var edges = new ArrayList<Edge>();
                for (Edge edge : block.terminator.edges()) {
                    var arguments = new ArrayList<>(edge.arguments());
                    for (String id : needed.get(byLabel.get(edge.label()))) arguments.add(values.get(id));
                    edges.add(new Edge(labels.get(edge.label()), arguments.stream().map(rewrite).toList()));
                }
                Terminator old = block.terminator;
                var terminator = new Terminator(old.kind(), old.condition() == null ? null : rewrite.apply(old.condition()),
                        old.values().stream().map(rewrite).toList(), edges, old.origin());
                result.add(new Block(labels.get(block.label), block.parameters.stream().map(value -> renamed.get(value.id())).toList(), instructions, terminator));
            }
            return List.copyOf(result);
        }

        List<Value> uses(Draft block) {
            var uses = new ArrayList<Value>();
            for (Instruction instruction : block.instructions) uses.addAll(instruction.operands());
            if (block.terminator.condition() != null) uses.add(block.terminator.condition());
            uses.addAll(block.terminator.values());
            for (Edge edge : block.terminator.edges()) uses.addAll(edge.arguments());
            return uses;
        }
    }

    private static String operation(Tree.Kind kind) {
        return switch (kind) {
            case PLUS -> "bv_add"; case MINUS -> "bv_sub"; case MULTIPLY -> "bv_mul";
            case DIVIDE -> "bv_sdiv"; case REMAINDER -> "bv_srem";
            case AND -> "bv_and"; case OR -> "bv_or"; case XOR -> "bv_xor";
            case LEFT_SHIFT -> "bv_shl"; case RIGHT_SHIFT -> "bv_ashr"; case UNSIGNED_RIGHT_SHIFT -> "bv_lshr";
            case EQUAL_TO -> "eq"; case NOT_EQUAL_TO -> "not_eq";
            case LESS_THAN -> "signed_lt"; case LESS_THAN_EQUAL -> "signed_le";
            case GREATER_THAN -> "signed_gt"; case GREATER_THAN_EQUAL -> "signed_ge";
            default -> throw fail("OPERATION");
        };
    }
}
