package mpk.java2vir;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

/** Strict sidecar attachment and typed, order-preserving normalization. */
final class JavaContracts {
    private JavaContracts() {}
    private static final Set<String> ROOT = Set.of("schema", "semantic_profile", "method", "requires", "ensures",
            "modifies", "abrupt_completion", "termination");
    private static final Set<String> EXPRESSION = Set.of("parameter", "result", "bool", "int", "op", "args");
    private static final Set<String> UNARY = Set.of("not", "bv_neg", "bv_not");
    private static final Set<String> NARY = Set.of("and", "or");
    private static final Set<String> BINARY = Set.of("eq", "not_eq", "signed_lt", "signed_le", "signed_gt", "signed_ge",
            "bv_add", "bv_sub", "bv_mul", "bv_and", "bv_or", "bv_xor");

    record Attached(String path, String rawInputSha256, String sidecarSha256,
                    Map<String, Object> sidecar, Map<String, Object> normalized, long nodeCount) {}
    record ContractSet(String selectionSha256, List<Attached> methods, long nodeCount) {}
    private record Expr(String kind, Object value, String integerType, List<Expr> args) {}
    private record Parsed(String method, List<Expr> requires, List<Expr> ensures, long nodes) {}
    private record Validated(String path, String rawHash, StrictJson.Value json) {}
    private record Pending(String path, String rawHash, Parsed parsed) {}
    private record Typed(ScalarType type, Map<String, Object> value) {}

    static final class ClosureCounter {
        private long nodes;
        long nodes() { return nodes; }
    }
    static final class MethodCounter {
        private final ClosureCounter closure;
        private long clauses;
        private long nodes;
        MethodCounter(ClosureCounter closure) { this.closure = closure; }
        void clause() { clauses = FrontendLimits.add("contract_clauses", clauses, 1, "subset"); }
        void node(long depth) {
            FrontendLimits.check("contract_depth", depth, "subset");
            long methodNext = FrontendLimits.add("contract_nodes_per_method", nodes, 1, "subset");
            long closureNext = FrontendLimits.add("contract_nodes_per_closure", closure.nodes, 1, "subset");
            nodes = methodNext;
            closure.nodes = closureNext;
        }
        long nodes() { return nodes; }
    }

    static ContractSet attach(CapturedSnapshot snapshot, JavaSubset.Closure closure) {
        try { return attachChecked(snapshot, closure); }
        catch (FrontendFailure failure) { throw failure; }
        catch (VirtualMachineError error) { throw FrontendFailure.of("JAVA_FRONTEND_RESOURCE", "subset"); }
        catch (RuntimeException error) { throw FrontendFailure.of("JAVA_FRONTEND_INTERNAL", "subset"); }
    }

    private static ContractSet attachChecked(CapturedSnapshot snapshot, JavaSubset.Closure closure) {
        Selection selection = snapshot.selection();
        if (!selection.equals(closure.selection())) throw reject("HASH");
        if (closure.sources().size() != selection.sources().size()) throw reject("HASH");
        for (int i = 0; i < closure.sources().size(); i++) {
            SourceText source = closure.sources().get(i);
            if (!source.path().equals(selection.sources().get(i))) throw reject("HASH");
            var captured = snapshot.file(source.path());
            if (!captured.source() || !MessageDigest.isEqual(captured.bytes(), source.text().getBytes(StandardCharsets.UTF_8))) throw reject("HASH");
        }
        var methods = new TreeMap<String, JavaSubset.Method>();
        for (JavaSubset.Method method : closure.methods()) methods.put(method.id(), method);
        var validated = new ArrayList<Validated>();
        // Finish the JSON pass for the whole batch before interpreting an expression.
        for (String path : selection.contracts()) {
            var input = snapshot.file(path);
            if (input.source()) throw reject("HASH");
            validated.add(new Validated(path, input.sha256(), StrictJson.validate(input.bytes())));
        }
        var parsed = new ArrayList<Pending>();
        var counter = new ClosureCounter();
        for (Validated input : validated) parsed.add(new Pending(input.path(), input.rawHash(), parse(input.json(), counter)));
        // No inferred/default sidecar and no unused file may disappear from this set.
        var attached = new TreeMap<String, Pending>();
        for (Pending candidate : parsed) {
            if (!methods.containsKey(candidate.parsed().method())) throw reject("UNUSED");
            if (attached.putIfAbsent(candidate.parsed().method(), candidate) != null) throw reject("DUPLICATE");
        }
        for (String method : methods.keySet()) if (!attached.containsKey(method)) throw reject("MISSING");
        var normalizedByMethod = new TreeMap<String, Attached>();
        // Type failures follow selected sidecar order, independently of the
        // callee-first order required by the eventual successful output.
        for (Pending entry : parsed) {
            JavaSubset.Method method = methods.get(entry.parsed().method());
            Parsed sidecar = entry.parsed();
            var parameters = new TreeMap<String, Typed>();
            for (int i = 0; i < method.parameters().size(); i++) {
                JavaSubset.Binding parameter = method.parameters().get(i);
                parameters.put(parameter.name(), new Typed(parameter.type(), Map.of("var", "arg" + i)));
            }
            var normalized = new TreeMap<String, Object>();
            normalized.put("semantic_context", Protocol.semanticContext());
            normalized.put("unit_id", selection.compilation());
            normalized.put("function_id", method.id());
            normalized.put("requires", normalizeClauses(sidecar.requires(), parameters, method.result(), false));
            normalized.put("ensures", normalizeClauses(sidecar.ensures(), parameters, method.result(), true));
            normalized.put("modifies", List.of());
            normalized.put("panic", "forbidden");
            normalized.put("termination", "total");
            normalized.put("loops", List.of());
            normalized.put("contract_hash", typedHash("MPK-CONTRACT-1.0", normalized));
            Map<String, Object> canonical = sidecar(sidecar);
            normalizedByMethod.put(method.id(), new Attached(entry.path(), entry.rawHash(), typedHash("MPK-JAVA-CONTRACT-SIDECAR-0.1", canonical),
                    canonical, Map.copyOf(normalized), sidecar.nodes()));
        }
        return new ContractSet(typedHash("MPK-JAVA-SELECTION-0.1", selection.envelope()),
                closure.methods().stream().map(method -> normalizedByMethod.get(method.id())).toList(), counter.nodes());
    }

    private static Parsed parse(StrictJson.Value json, ClosureCounter closure) {
        Map<String, StrictJson.Value> root = json.exact(ROOT);
        if (!root.get("schema").string().equals("mpk.java.contract.v0")
                || !root.get("semantic_profile").string().equals("mpk.java.scalar.v0")
                || !root.get("abrupt_completion").string().equals("forbidden")
                || !root.get("termination").string().equals("total")) throw reject("IDENTITY");
        root.get("modifies").elements(0, "JAVA_CONTRACT_IDENTITY");
        String method = root.get("method").string();
        if (!Selection.methodId(method)) throw reject("IDENTITY");
        var counter = new MethodCounter(closure);
        List<Expr> requires = clauses(root.get("requires"), counter);
        List<Expr> ensures = clauses(root.get("ensures"), counter);
        if (ensures.isEmpty()) throw reject("SHAPE");
        return new Parsed(method, requires, ensures, counter.nodes());
    }

    private static List<Expr> clauses(StrictJson.Value value, MethodCounter counter) {
        var result = new ArrayList<Expr>();
        for (StrictJson.Value clause : value.elements(64, "JAVA_LIMIT_CONTRACT_CLAUSES")) {
            counter.clause();
            result.add(expression(clause, 1, counter));
        }
        return List.copyOf(result);
    }

    private static Expr expression(StrictJson.Value value, long depth, MethodCounter counter) {
        counter.node(depth);
        var fields = value.fields(EXPRESSION);
        if (fields.keySet().equals(Set.of("parameter"))) return new Expr("parameter", fields.get("parameter").string(), null, List.of());
        if (fields.keySet().equals(Set.of("result"))) {
            return new Expr("result", fields.get("result").raw(), null, List.of());
        }
        if (fields.keySet().equals(Set.of("bool"))) return new Expr("bool", fields.get("bool").bool(), null, List.of());
        if (fields.keySet().equals(Set.of("int"))) {
            var integer = fields.get("int").exact(Set.of("decimal", "type"));
            return new Expr("int", integer.get("decimal").string(), integer.get("type").string(), List.of());
        }
        if (!fields.keySet().equals(Set.of("op", "args"))) throw reject("SHAPE");
        String op = fields.get("op").string();
        var arguments = fields.get("args").elements(64, "JAVA_CONTRACT_SHAPE");
        if (UNARY.contains(op) && arguments.size() != 1 || BINARY.contains(op) && arguments.size() != 2
                || NARY.contains(op) && arguments.size() < 2) throw reject("SHAPE");
        var children = new ArrayList<Expr>();
        for (var argument : arguments) children.add(expression(argument, depth + 1, counter));
        return new Expr("operator", op, null, List.copyOf(children));
    }

    private static List<Map<String, Object>> normalizeClauses(List<Expr> clauses, Map<String, Typed> parameters,
                                                            ScalarType resultType, boolean allowResult) {
        var normalized = new ArrayList<Map<String, Object>>();
        for (Expr clause : clauses) {
            Typed expression = normalize(clause, parameters, resultType, allowResult);
            if (expression.type() != ScalarType.BOOLEAN) throw reject("TYPE");
            normalized.add(expression.value());
        }
        return List.copyOf(normalized);
    }

    private static Typed normalize(Expr expression, Map<String, Typed> parameters, ScalarType resultType, boolean allowResult) {
        String op = expression.kind();
        switch (op) {
            case "parameter": {
                Typed parameter = parameters.get(expression.value());
                if (parameter == null) throw reject("TYPE");
                return parameter;
            }
            case "result": {
                if (!allowResult || !expression.value().equals("0")) throw reject("TYPE");
                return new Typed(resultType, Map.of("result", 0));
            }
            case "bool": return new Typed(ScalarType.BOOLEAN, Map.of("bool", expression.value()));
            case "int": {
                ScalarType type = switch (expression.integerType()) {
                    case "i32" -> ScalarType.INT; case "i64" -> ScalarType.LONG;
                    default -> throw reject("TYPE");
                };
                String decimal = (String) expression.value();
                if (decimal.length() > 20 || !decimal.matches("0|-?[1-9][0-9]*")) throw reject("TYPE");
                long number;
                try { number = Long.parseLong(decimal); }
                catch (NumberFormatException error) { throw reject("TYPE"); }
                if (type == ScalarType.INT && number != (int) number) throw reject("TYPE");
                return new Typed(type, Map.of("int", Map.of("value", decimal, "width", type.width(), "signed", true)));
            }
            default: break;
        }
        op = (String) expression.value();
        int minimum, maximum;
        if (UNARY.contains(op)) { minimum = 1; maximum = 1; }
        else if (NARY.contains(op)) { minimum = 2; maximum = 64; }
        else if (BINARY.contains(op)) { minimum = 2; maximum = 2; }
        else throw reject("OPERATOR");
        if (expression.args().size() < minimum || expression.args().size() > maximum) throw reject("SHAPE");
        var children = new ArrayList<Typed>();
        for (Expr argument : expression.args()) children.add(normalize(argument, parameters, resultType, allowResult));
        ScalarType type = children.getFirst().type();
        if (UNARY.contains(op)) {
            if (op.equals("not") ? type != ScalarType.BOOLEAN : !type.integer()) throw reject("TYPE");
            return new Typed(type, Map.of("op", op, "value", children.getFirst().value()));
        }
        if (NARY.contains(op)) {
            if (children.stream().anyMatch(child -> child.type() != ScalarType.BOOLEAN)) throw reject("TYPE");
            return new Typed(ScalarType.BOOLEAN, Map.of("op", op, "args", children.stream().map(Typed::value).toList()));
        }
        if (children.get(1).type() != type) throw reject("TYPE");
        ScalarType output = type;
        if (op.equals("eq") || op.equals("not_eq")) output = ScalarType.BOOLEAN;
        else {
            if (!type.integer()) throw reject("TYPE");
            if (op.startsWith("signed_")) output = ScalarType.BOOLEAN;
        }
        return new Typed(output, Map.of("op", op, "lhs", children.get(0).value(), "rhs", children.get(1).value()));
    }

    private static Map<String, Object> sidecar(Parsed parsed) {
        return Map.of("schema", "mpk.java.contract.v0", "semantic_profile", "mpk.java.scalar.v0", "method", parsed.method(),
                "requires", parsed.requires().stream().map(JavaContracts::sidecarExpression).toList(),
                "ensures", parsed.ensures().stream().map(JavaContracts::sidecarExpression).toList(),
                "modifies", List.of(), "abrupt_completion", "forbidden", "termination", "total");
    }
    private static Map<String, Object> sidecarExpression(Expr expression) {
        return switch (expression.kind()) {
            case "parameter", "bool" -> Map.of(expression.kind(), expression.value());
            case "result" -> Map.of("result", 0);
            case "int" -> Map.of("int", Map.of("decimal", expression.value(), "type", expression.integerType()));
            default -> Map.of("op", expression.value(), "args", expression.args().stream().map(JavaContracts::sidecarExpression).toList());
        };
    }
    static String typedHash(String domain, Object value) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            digest.update((domain + "\0").getBytes(StandardCharsets.US_ASCII));
            return HexFormat.of().formatHex(digest.digest(Protocol.json(value).getBytes(StandardCharsets.UTF_8)));
        } catch (NoSuchAlgorithmException error) { throw reject("HASH"); }
    }
    private static FrontendFailure reject(String suffix) { return FrontendFailure.of("JAVA_CONTRACT_" + suffix, "subset"); }
}
