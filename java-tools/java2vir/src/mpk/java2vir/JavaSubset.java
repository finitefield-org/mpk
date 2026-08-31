package mpk.java2vir;

import com.sun.source.tree.*;
import java.nio.charset.StandardCharsets;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import java.util.TreeSet;
import javax.lang.model.element.Element;
import javax.lang.model.element.ElementKind;
import javax.lang.model.element.ExecutableElement;
import javax.lang.model.element.Modifier;
import javax.lang.model.element.TypeElement;
import javax.lang.model.element.VariableElement;

/** Source admission only. No CFG, code generation, class loading or public entrypoint. */
final class JavaSubset {
    record Binding(String name, ScalarType type, VariableElement element, VariableTree declaration, boolean parameter) {}
    record Method(String id, String owner, String name, MethodTree declaration, ExecutableElement element,
                  ScalarType result, List<Binding> parameters, List<Binding> locals, List<String> callees,
                  Map<ExpressionTree, ScalarType> expressionTypes, Map<LiteralTree, String> integers,
                  Map<MethodInvocationTree, String> callTargets, Map<IdentifierTree, Binding> variableBindings) {}

    static final class Closure {
        private final Selection selection;
        private final List<SourceText> sources;
        private final List<Method> methods;
        private final TreeInventory origins;
        private Closure(Selection selection, List<SourceText> sources, List<Method> methods, TreeInventory origins) {
            this.selection = selection;
            this.sources = List.copyOf(sources);
            this.methods = List.copyOf(methods);
            this.origins = origins;
        }
        Selection selection() { return selection; }
        List<SourceText> sources() { return sources; }
        List<Method> methods() { return methods; }
        TreeInventory origins() { return origins; }
    }

    private record Declaration(String id, String owner, String name, MethodTree tree, ExecutableElement element,
                               ScalarType result, List<Binding> parameters) {}
    private final CompilerSession session;
    private final TreeInventory raw;
    private final TreeInventory typed;
    private final Map<String, Declaration> declarations = new TreeMap<>();
    private final Map<ExecutableElement, Declaration> symbols = new IdentityHashMap<>();
    private final List<Declaration> sourceOrder = new ArrayList<>();

    private JavaSubset(CompilerSession session) {
        this.session = session;
        raw = session.before();
        typed = session.after();
    }

    static Closure admit(CompilerSession session, Selection selection) {
        try { return new JavaSubset(session).validate(selection); }
        catch (FrontendFailure failure) { throw failure; }
        catch (RuntimeException | Error error) { throw FrontendFailure.compilerFailure(error, "subset"); }
    }

    private Closure validate(Selection selection) {
        List<SourceText> sources = session.units().stream().map(unit -> raw.node(unit).source()).toList();
        if (!sources.stream().map(SourceText::path).toList().equals(selection.sources())) throw adapter();
        // Raw declaration/name gates across the complete selection precede bodies.
        for (var unit : session.units()) unit(unit);
        var methods = new TreeMap<String, Method>();
        for (Declaration declaration : sourceOrder) {
            Method method = new Body(declaration).validate();
            methods.put(method.id(), method);
        }
        // Excluded parents, including class and var, have already rejected. Their
        // synthesized children must never reach accepted-tree comparison.
        for (var unit : session.units()) raw.requireUnchanged(unit, typed);
        for (String selected : selection.methods()) if (!methods.containsKey(selected)) throw reject("CALL");

        var reachable = new TreeSet<String>();
        var pending = new ArrayDeque<String>(selection.methods());
        while (!pending.isEmpty()) {
            String id = pending.removeFirst();
            if (reachable.contains(id)) continue;
            FrontendLimits.check("method_closure", (long) reachable.size() + 1, "subset");
            reachable.add(id);
            pending.addAll(methods.get(id).callees());
        }
        if (!reachable.equals(methods.keySet())) throw reject("CALL");
        var ordered = new ArrayList<Method>();
        var emitted = new HashSet<String>();
        while (ordered.size() < methods.size()) {
            Method ready = null;
            for (Method method : methods.values()) {
                if (!emitted.contains(method.id()) && emitted.containsAll(method.callees())) { ready = method; break; }
            }
            if (ready == null) throw reject("CALL");
            ordered.add(ready);
            emitted.add(ready.id());
        }
        return new Closure(selection, sources, ordered, raw);
    }

    private void unit(CompilationUnitTree unit) {
        if (unit.getPackage() == null || !unit.getPackageAnnotations().isEmpty()
                || !unit.getImports().isEmpty() || unit.getModule() != null || unit.getTypeDecls().size() != 1)
            throw reject("DECLARATION");
        Tree top = unit.getTypeDecls().getFirst();
        if (!(top instanceof ClassTree owner) || raw.node(top).kind() != Tree.Kind.INTERFACE) throw reject("DECLARATION");
        modifiers(owner.getModifiers(), Set.of("public"), "DECLARATION");
        if (!owner.getTypeParameters().isEmpty() || !owner.getPermitsClause().isEmpty()) throw reject("DECLARATION");
        if (owner.getExtendsClause() != null || !owner.getImplementsClause().isEmpty()) throw reject("INITIALIZATION");
        String packageName = qualified(unit.getPackageName());
        if (!Selection.packageName(packageName)) throw reject("IDENTIFIER");
        var packageTokens = new SourceTokens(raw.node(unit.getPackage()));
        packageTokens.expect("package", "JAVA_SUBSET_DECLARATION");
        qualifiedTokens(packageTokens, packageName, "IDENTIFIER");
        packageTokens.expect(";", "JAVA_SUBSET_DECLARATION");
        packageTokens.expect("", "JAVA_SUBSET_DECLARATION");
        var tokens = new SourceTokens(raw.node(owner));
        tokens.expect("public", "JAVA_SUBSET_DECLARATION");
        tokens.expect("interface", "JAVA_SUBSET_DECLARATION");
        String name = identifier(tokens.next());
        if (!name.equals(raw.node(owner).facts().get("name"))) throw adapter();
        tokens.expect("{", "JAVA_SUBSET_DECLARATION");
        String canonicalOwner = packageName + "." + name;
        if (!raw.node(unit).source().path().equals("src/" + canonicalOwner.replace('.', '/') + ".java")) throw reject("DECLARATION");
        if (!(typed.node(owner).element() instanceof TypeElement element)
                || element.getKind() != ElementKind.INTERFACE
                || !element.getQualifiedName().contentEquals(canonicalOwner)
                || session.elements().getTypeElement(canonicalOwner) != element) throw adapter();
        if (owner.getMembers().isEmpty()) throw reject("DECLARATION");
        var names = new HashSet<String>();
        for (Tree member : owner.getMembers()) {
            if (member instanceof VariableTree || member instanceof BlockTree) throw reject("INITIALIZATION");
            if (!(member instanceof MethodTree method)) throw reject("DECLARATION");
            // Do not skip punctuation between members that javac may omit from its tree.
            tokens.triviaTo(raw.node(member).start(), "JAVA_SUBSET_DECLARATION");
            Declaration declaration = declaration(method, canonicalOwner, element);
            if (!names.add(declaration.name())) throw reject("DECLARATION");
            FrontendLimits.check("method_closure", (long) declarations.size() + 1, "subset");
            if (declarations.putIfAbsent(declaration.id(), declaration) != null
                    || symbols.put(declaration.element(), declaration) != null) throw adapter();
            sourceOrder.add(declaration);
            tokens.seek(raw.node(member).end());
        }
        tokens.expect("}", "JAVA_SUBSET_DECLARATION");
        tokens.expect("", "JAVA_SUBSET_DECLARATION");
    }

    private Declaration declaration(MethodTree method, String owner, TypeElement enclosing) {
        modifiers(method.getModifiers(), Set.of("public", "static"), "DECLARATION");
        if (method.getBody() == null || !method.getTypeParameters().isEmpty() || method.getReceiverParameter() != null
                || method.getDefaultValue() != null) throw reject("DECLARATION");
        if (!method.getThrows().isEmpty()) throw reject("ABRUPT");
        if (!(typed.node(method).element() instanceof ExecutableElement element)
                || element.getKind() != ElementKind.METHOD || element.getEnclosingElement() != enclosing) throw adapter();
        if (element.isVarArgs()) throw reject("DECLARATION");
        var tokens = new SourceTokens(raw.node(method));
        String first = tokens.next(), second = tokens.next();
        if (!(first.equals("public") && second.equals("static") || first.equals("static") && second.equals("public")))
            throw reject("DECLARATION");
        ScalarType result = primitive(method.getReturnType());
        tokens.expect(result.keyword, "JAVA_SUBSET_TYPE");
        String name = identifier(tokens.next());
        if (!name.equals(raw.node(method).facts().get("name")) || !element.getSimpleName().contentEquals(name)
                || ScalarType.resolved(element.getReturnType()) != result
                || !element.getModifiers().equals(Set.of(Modifier.PUBLIC, Modifier.STATIC))) throw adapter();
        tokens.expect("(", "JAVA_SUBSET_DECLARATION");
        var parameters = new ArrayList<Binding>();
        var names = new HashSet<String>();
        long slots = 0;
        if (element.getParameters().size() != method.getParameters().size()) throw adapter();
        for (VariableTree parameter : method.getParameters()) {
            if (!parameters.isEmpty()) tokens.expect(",", "JAVA_SUBSET_DECLARATION");
            modifiers(parameter.getModifiers(), Set.of(), "DECLARATION");
            ScalarType type = primitive(parameter.getType());
            slots = FrontendLimits.add("parameter_slots", slots, type.slots, "subset");
            tokens.expect(type.keyword, "JAVA_SUBSET_TYPE");
            String parameterName = identifier(tokens.next());
            if (!names.add(parameterName)) throw reject("IDENTIFIER");
            VariableElement symbol = variable(parameter, parameterName, type, element, ElementKind.PARAMETER);
            if (element.getParameters().get(parameters.size()) != symbol || parameter.getInitializer() != null) throw adapter();
            parameters.add(new Binding(parameterName, type, symbol, parameter, true));
        }
        tokens.expect(")", "JAVA_SUBSET_DECLARATION");
        tokens.expect("{", "JAVA_SUBSET_DECLARATION");
        String id = owner + "::" + name + "(" + String.join(",", parameters.stream().map(p -> p.type().keyword).toList()) + ")->" + result.keyword;
        FrontendLimits.check("canonical_method_id_bytes", id.getBytes(StandardCharsets.UTF_8).length, "subset");
        return new Declaration(id, owner, name, method, element, result, List.copyOf(parameters));
    }

    private void modifiers(ModifiersTree modifiers, Set<String> expected, String code) {
        TreeInventory.Node node = raw.node(modifiers);
        if (!modifiers.getAnnotations().isEmpty() || !node.facts().get("flags").equals(expected.stream().sorted().toList())) throw reject(code);
        if (!expected.isEmpty()) {
            var tokens = new SourceTokens(node);
            var actual = new HashSet<String>();
            for (int i = 0; i < expected.size(); i++) if (!actual.add(tokens.next())) throw reject(code);
            if (!actual.equals(expected) || !tokens.next().isEmpty()) throw reject(code);
        }
    }

    private ScalarType primitive(Tree tree) {
        if (!(tree instanceof PrimitiveTypeTree primitive)) throw reject("TYPE");
        ScalarType type = ScalarType.keyword(raw.node(tree).spelling());
        if (primitive.getPrimitiveTypeKind() != type.kind || ScalarType.resolved(typed.node(tree).type()) != type) throw adapter();
        return type;
    }

    private VariableElement variable(VariableTree tree, String name, ScalarType type, ExecutableElement owner, ElementKind kind) {
        if (!name.equals(raw.node(tree).facts().get("name")) || !(typed.node(tree).element() instanceof VariableElement element)
                || element.getKind() != kind || element.getEnclosingElement() != owner
                || !element.getSimpleName().contentEquals(name) || ScalarType.resolved(element.asType()) != type) throw adapter();
        return element;
    }

    private String qualified(ExpressionTree tree) {
        if (tree instanceof IdentifierTree) return identifier(raw.node(tree).spelling());
        if (tree instanceof MemberSelectTree select) {
            String prefix = qualified(select.getExpression());
            var tokens = new SourceTokens(raw.node(tree));
            qualifiedTokens(tokens, prefix, "IDENTIFIER");
            tokens.expect(".", "JAVA_SUBSET_IDENTIFIER");
            String name = identifier(tokens.next());
            tokens.expect("", "JAVA_SUBSET_IDENTIFIER");
            if (!name.equals(raw.node(tree).facts().get("name"))) throw adapter();
            return prefix + "." + name;
        }
        throw reject("CALL");
    }

    private static void qualifiedTokens(SourceTokens tokens, String name, String code) {
        boolean first = true;
        for (String part : name.split("\\.")) {
            if (!first) tokens.expect(".", "JAVA_SUBSET_" + code);
            first = false;
            if (!identifier(tokens.next()).equals(part)) throw reject(code);
        }
    }

    private final class Body {
        private final Declaration method;
        private final Map<Element, Binding> active = new IdentityHashMap<>();
        private final Set<String> names = new HashSet<>();
        private final List<Binding> locals = new ArrayList<>();
        private final ArrayDeque<List<Element>> scopes = new ArrayDeque<>();
        private final Map<ExpressionTree, ScalarType> types = new IdentityHashMap<>();
        private final Map<LiteralTree, String> integers = new IdentityHashMap<>();
        private final Map<MethodInvocationTree, String> calls = new IdentityHashMap<>();
        private final Map<IdentifierTree, Binding> bindings = new IdentityHashMap<>();
        private final Set<String> callees = new TreeSet<>();
        Body(Declaration method) {
            this.method = method;
            for (Binding parameter : method.parameters()) { names.add(parameter.name()); active.put(parameter.element(), parameter); }
        }
        Method validate() {
            if (!statement(method.tree().getBody())) throw reject("CONTROL_FLOW");
            return new Method(method.id(), method.owner(), method.name(), method.tree(), method.element(), method.result(),
                    method.parameters(), List.copyOf(locals), List.copyOf(callees), immutableIdentity(types),
                    immutableIdentity(integers), immutableIdentity(calls), immutableIdentity(bindings));
        }

        private boolean statement(StatementTree tree) {
            return switch (raw.node(tree).kind()) {
                case BLOCK -> {
                    var block = (BlockTree) tree;
                    if (block.isStatic()) throw reject("INITIALIZATION");
                    scopes.push(new ArrayList<>());
                    boolean returned = false;
                    for (StatementTree child : block.getStatements()) {
                        if (returned) throw reject("CONTROL_FLOW");
                        returned = statement(child);
                    }
                    for (Element element : scopes.pop()) active.remove(element);
                    yield returned;
                }
                case VARIABLE -> { local((VariableTree) tree); yield false; }
                case EXPRESSION_STATEMENT -> {
                    ExpressionTree expression = ((ExpressionStatementTree) tree).getExpression();
                    if (!(expression instanceof AssignmentTree assignment)) {
                        if (expression instanceof UnaryTree || expression instanceof CompoundAssignmentTree) throw reject("OPERATION");
                        throw reject("CONTROL_FLOW");
                    }
                    if (!(assignment.getVariable() instanceof IdentifierTree)) throw reject("CONTROL_FLOW");
                    identifier(raw.node(assignment.getVariable()).spelling());
                    Binding target = active.get(typed.node(assignment.getVariable()).element());
                    if (target == null || target.parameter()) throw reject("CONTROL_FLOW");
                    expression(assignment.getVariable());
                    conversion(expression(assignment.getExpression()), target.type(), "local_assignment");
                    expectType(assignment, target.type());
                    yield false;
                }
                case IF -> {
                    var branch = (IfTree) tree;
                    if (expression(branch.getCondition()) != ScalarType.BOOLEAN) throw reject("TYPE");
                    boolean yes = statement(branch.getThenStatement());
                    boolean no = branch.getElseStatement() != null && statement(branch.getElseStatement());
                    yield yes && no;
                }
                case RETURN -> {
                    ExpressionTree value = ((ReturnTree) tree).getExpression();
                    if (value == null) throw reject("CONTROL_FLOW");
                    conversion(expression(value), method.result(), "return");
                    yield true;
                }
                case THROW, TRY, ASSERT -> throw reject("ABRUPT");
                case SYNCHRONIZED -> throw reject("PURITY");
                case CLASS, INTERFACE, ENUM, RECORD, ANNOTATION_TYPE -> throw reject("DECLARATION");
                default -> throw reject("CONTROL_FLOW");
            };
        }

        private void local(VariableTree variable) {
            modifiers(variable.getModifiers(), Set.of(), "CONTROL_FLOW");
            var tokens = new SourceTokens(raw.node(variable));
            String keyword = tokens.next();
            if (keyword.equals("var")) throw reject("TYPE");
            ScalarType type = primitive(variable.getType());
            if (!keyword.equals(type.keyword)) throw reject("TYPE");
            String name = identifier(tokens.next());
            if (!names.add(name)) throw reject("CONTROL_FLOW");
            tokens.expect("=", "JAVA_SUBSET_CONTROL_FLOW");
            if (variable.getInitializer() == null) throw reject("CONTROL_FLOW");
            tokens.seek(raw.node(variable.getInitializer()).end());
            // javac splits multi-declarators into overlapping VariableTree spans.
            tokens.expect(";", "JAVA_SUBSET_CONTROL_FLOW");
            tokens.expect("", "JAVA_SUBSET_CONTROL_FLOW");
            conversion(expression(variable.getInitializer()), type, "local_initializer");
            VariableElement element = variable(variable, name, type, method.element(), ElementKind.LOCAL_VARIABLE);
            Binding binding = new Binding(name, type, element, variable, false);
            if (active.put(element, binding) != null || scopes.isEmpty()) throw adapter();
            scopes.peek().add(element);
            locals.add(binding);
        }

        private ScalarType expression(ExpressionTree tree) {
            ScalarType result = switch (raw.node(tree).kind()) {
                case BOOLEAN_LITERAL, INT_LITERAL, LONG_LITERAL -> literal((LiteralTree) tree);
                case CHAR_LITERAL, STRING_LITERAL, FLOAT_LITERAL, DOUBLE_LITERAL, NULL_LITERAL, UNARY_PLUS -> throw reject("LITERAL");
                case IDENTIFIER -> {
                    String name = identifier(raw.node(tree).spelling());
                    Element element = typed.node(tree).element();
                    if (element == null) throw adapter();
                    Binding binding = active.get(element);
                    if (binding == null) throw reject("CONTROL_FLOW");
                    if (!binding.name().equals(name) || !name.equals(raw.node(tree).facts().get("name"))) throw adapter();
                    bindings.put((IdentifierTree) tree, binding);
                    yield binding.type();
                }
                case PARENTHESIZED -> expression(((ParenthesizedTree) tree).getExpression());
                case UNARY_MINUS, LOGICAL_COMPLEMENT, BITWISE_COMPLEMENT -> {
                    var unary = (UnaryTree) tree;
                    ScalarType operand = expression(unary.getExpression());
                    if (unary.getKind() == Tree.Kind.LOGICAL_COMPLEMENT ? operand != ScalarType.BOOLEAN : !operand.integer()) throw reject("OPERATION");
                    yield operand;
                }
                case MULTIPLY, DIVIDE, REMAINDER, PLUS, MINUS, LEFT_SHIFT, RIGHT_SHIFT, UNSIGNED_RIGHT_SHIFT,
                        LESS_THAN, GREATER_THAN, LESS_THAN_EQUAL, GREATER_THAN_EQUAL, EQUAL_TO, NOT_EQUAL_TO,
                        AND, XOR, OR, CONDITIONAL_AND, CONDITIONAL_OR -> binary((BinaryTree) tree);
                case TYPE_CAST -> {
                    var cast = (TypeCastTree) tree;
                    ScalarType target = primitive(cast.getType());
                    conversion(expression(cast.getExpression()), target, "explicit_cast");
                    yield target;
                }
                case CONDITIONAL_EXPRESSION -> {
                    var conditional = (ConditionalExpressionTree) tree;
                    if (expression(conditional.getCondition()) != ScalarType.BOOLEAN) throw reject("TYPE");
                    ScalarType yes = expression(conditional.getTrueExpression());
                    conversion(expression(conditional.getFalseExpression()), yes, "conditional_arm");
                    yield yes;
                }
                case METHOD_INVOCATION -> call((MethodInvocationTree) tree);
                case ASSIGNMENT -> throw reject("CONTROL_FLOW");
                case SWITCH_EXPRESSION -> throw reject("CONTROL_FLOW");
                case NEW_CLASS, NEW_ARRAY, ARRAY_ACCESS -> throw reject("PURITY");
                case MEMBER_SELECT, MEMBER_REFERENCE, LAMBDA_EXPRESSION -> throw reject("CALL");
                default -> throw reject("OPERATION");
            };
            expectType(tree, result);
            return result;
        }

        private void expectType(ExpressionTree tree, ScalarType expected) {
            if (ScalarType.resolved(typed.node(tree).type()) != expected) throw adapter();
            types.put(tree, expected);
        }

        private ScalarType literal(LiteralTree tree) {
            TreeInventory.Node node = raw.node(tree);
            var tokens = new SourceTokens(node);
            String spelling = tokens.next();
            if (tree.getKind() == Tree.Kind.BOOLEAN_LITERAL) {
                if (!Set.of("true", "false").contains(spelling) || !tokens.next().isEmpty()) throw reject("LITERAL");
                if (!spelling.equals(node.facts().get("literal"))) throw adapter();
                return ScalarType.BOOLEAN;
            }
            boolean negative = spelling.equals("-");
            if (negative) spelling = tokens.next();
            ScalarType type = tree.getKind() == Tree.Kind.INT_LITERAL ? ScalarType.INT : ScalarType.LONG;
            if (!spelling.matches(type == ScalarType.INT ? "0|[1-9][0-9]*" : "(0|[1-9][0-9]*)L") || !tokens.next().isEmpty()) throw reject("LITERAL");
            if (type == ScalarType.LONG) spelling = spelling.substring(0, spelling.length() - 1);
            String signed = (negative ? "-" : "") + spelling;
            long value;
            try { value = Long.parseLong(signed); }
            catch (NumberFormatException error) { throw reject("LITERAL"); }
            if (type == ScalarType.INT && value != (int) value) throw reject("LITERAL");
            String canonical = Long.toString(value);
            if (!canonical.equals(node.facts().get("literal"))) throw adapter();
            integers.put(tree, canonical);
            return type;
        }

        private ScalarType binary(BinaryTree tree) {
            ScalarType left = expression(tree.getLeftOperand()), right = expression(tree.getRightOperand());
            return switch (tree.getKind()) {
                case LEFT_SHIFT, RIGHT_SHIFT, UNSIGNED_RIGHT_SHIFT -> {
                    if (!left.integer() || right != ScalarType.INT) throw reject("OPERATION");
                    yield left;
                }
                case EQUAL_TO, NOT_EQUAL_TO -> {
                    if (left != right) throw reject("TYPE");
                    yield ScalarType.BOOLEAN;
                }
                case CONDITIONAL_AND, CONDITIONAL_OR -> {
                    if (left != ScalarType.BOOLEAN || right != ScalarType.BOOLEAN) throw reject("OPERATION");
                    yield ScalarType.BOOLEAN;
                }
                default -> {
                    if (left != right) throw reject("TYPE");
                    if (!left.integer()) throw reject("OPERATION");
                    yield switch (tree.getKind()) {
                        case LESS_THAN, GREATER_THAN, LESS_THAN_EQUAL, GREATER_THAN_EQUAL -> ScalarType.BOOLEAN;
                        default -> left;
                    };
                }
            };
        }

        private ScalarType call(MethodInvocationTree tree) {
            if (!tree.getTypeArguments().isEmpty()) throw reject("CALL");
            Element symbol = typed.node(tree).element();
            Declaration callee = symbols.get(symbol);
            if (callee == null) throw reject("CALL");
            ExpressionTree target = tree.getMethodSelect();
            if (typed.node(target).element() != callee.element()) throw adapter();
            String spelling = qualified(target);
            if (target instanceof IdentifierTree) {
                if (!method.owner().equals(callee.owner()) || !spelling.equals(callee.name())) throw reject("CALL");
            } else if (!spelling.equals(callee.owner() + "." + callee.name())) throw reject("CALL");
            if (tree.getArguments().size() != callee.parameters().size()) throw reject("CALL");
            for (int i = 0; i < tree.getArguments().size(); i++)
                conversion(expression(tree.getArguments().get(i)), callee.parameters().get(i).type(), "call_argument");
            calls.put(tree, callee.id());
            callees.add(callee.id());
            return callee.result();
        }
    }

    private static <K, V> Map<K, V> immutableIdentity(Map<K, V> map) {
        return Collections.unmodifiableMap(new IdentityHashMap<>(map));
    }
    private static String identifier(String raw) {
        if (!Selection.identifier(raw)) throw reject("IDENTIFIER");
        return raw;
    }
    private static void conversion(ScalarType from, ScalarType to, String context) {
        if (!ScalarType.conversion(from, to, context)) throw reject("CONVERSION");
    }
    private static FrontendFailure reject(String code) { return FrontendFailure.of("JAVA_SUBSET_" + code, "subset"); }
    private static FrontendFailure adapter() { return FrontendFailure.of("JAVA_TOOLCHAIN_ADAPTER", "typecheck"); }
}
