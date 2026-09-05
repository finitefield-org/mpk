using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

// Private W07 handoff. Operands are captured Roslyn operations, not re-evaluated
// syntax strings. Steps describe normal execution; exception edges terminate it.
// Predicates are obligations, never claims that T06 has discharged a proof.
internal sealed record PracticalArrayStep(string Site, string Path, string Operation, IOperation Source,
    IReadOnlyList<string> Arrays, IOperation? Operand = null, string Predicate = "",
    string Exception = "");
internal sealed record PracticalArrays(PracticalConstruction Construction,
    IReadOnlyList<PracticalArrayStep> Steps)
{
    internal int ArtifactCount => 0;
}

internal static class CSharpPracticalArrays
{
    internal const int MaximumLength = 4096;
    internal const string ForeachWriteConflict = "active_foreach_read_borrow";

    // T04-W02 supplies the active borrow; no positive foreach source is admitted here.
    internal static void RequireWritable(bool unique, bool borrowed)
    {
        if (borrowed) { throw PracticalFailures.Type(ForeachWriteConflict); }
        if (!unique) { throw PracticalFailures.Type("array_frozen_write"); }
    }

    internal static PracticalArrays Validate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs, ImmutableArray<MetadataReference> references,
        IReadOnlyList<PracticalTypeInvariantClaim>? invariantClaims = null)
    {
        var analyzer = new Analyzer();
        PracticalConstruction construction = CSharpPracticalConstruction.Validate(selection, inputs, references,
            invariantClaims, allowInitializers: true, allowStructuralEquality: true,
            validateArrays: analyzer.Analyze, validateArrayLimits: ValidateLimits);
        return new PracticalArrays(construction, Array.AsReadOnly(analyzer.Steps.ToArray()));
    }

    private static void ValidateLimits(CSharpCompilation compilation)
    {
        foreach (SyntaxTree tree in compilation.SyntaxTrees) {
            SemanticModel model = compilation.GetSemanticModel(tree);
            foreach (SyntaxNode node in tree.GetRoot().DescendantNodes()) {
                if (node is InitializerExpressionSyntax initializer && initializer.IsKind(SyntaxKind.ArrayInitializerExpression)) {
                    int count = 0;
                    foreach (ExpressionSyntax unused in initializer.Expressions) {
                        count = checked(count + 1);
                        if (count > MaximumLength) { throw PracticalFailures.Limit("array_elements"); }
                    }
                }
                if (node is ArrayCreationExpressionSyntax creation) {
                    foreach (ExpressionSyntax size in creation.Type.RankSpecifiers.SelectMany(rank=>rank.Sizes)) {
                        Optional<object?> value = model.GetConstantValue(size);
                        object? constant = value.HasValue ? value.Value : null;
                        if (constant is int i && i > MaximumLength || constant is uint u && u > MaximumLength
                            || constant is long l && l > MaximumLength || constant is ulong ul && ul > MaximumLength)
                        { throw PracticalFailures.Limit("array_elements"); }
                    }
                }
            }
        }
    }

    private sealed class Storage
    {
        internal int? Length;
        internal bool Complete;
        internal bool Frozen;
        internal bool SymbolicWrites;
        internal HashSet<int> Initialized = new();
        internal HashSet<int> PossiblyInitialized = new();
        internal Storage Copy() => new() { Length = Length, Complete = Complete, Frozen = Frozen, SymbolicWrites = SymbolicWrites,
            Initialized = new(Initialized), PossiblyInitialized = new(PossiblyInitialized) };
    }
    private sealed class State
    {
        internal bool Live = true;
        internal Dictionary<ILocalSymbol, HashSet<string>> Locals = new(SymbolEqualityComparer.Default);
        internal Dictionary<string, Storage> Arrays = new(StringComparer.Ordinal);
        internal State Copy() => new() { Live = Live,
            Locals = Locals.ToDictionary(pair => pair.Key, pair => new HashSet<string>(pair.Value),
                (IEqualityComparer<ILocalSymbol>)SymbolEqualityComparer.Default),
            Arrays = Arrays.ToDictionary(pair => pair.Key, pair => pair.Value.Copy(), StringComparer.Ordinal) };
        internal void Join(State a, State b)
        {
            if (!a.Live || !b.Live) {
                State live = a.Live ? a : b;
                Live = live.Live; Locals = live.Locals; Arrays = live.Arrays; return;
            }
            Live = true; Locals.Clear(); Arrays.Clear();
            foreach (ILocalSymbol local in a.Locals.Keys.Concat(b.Locals.Keys).Distinct(SymbolEqualityComparer.Default).Cast<ILocalSymbol>()) {
                var aliases = new HashSet<string>();
                if (a.Locals.TryGetValue(local, out var left)) { aliases.UnionWith(left); }
                if (b.Locals.TryGetValue(local, out var right)) { aliases.UnionWith(right); }
                Locals.Add(local, aliases);
            }
            foreach (string id in a.Arrays.Keys.Concat(b.Arrays.Keys).Distinct(StringComparer.Ordinal)) {
                if (!a.Arrays.TryGetValue(id, out Storage? left)) { Arrays.Add(id,b.Arrays[id].Copy()); continue; }
                if (!b.Arrays.TryGetValue(id, out Storage? right)) { Arrays.Add(id,left.Copy()); continue; }
                Storage joined = left.Copy(); joined.Complete &= right.Complete; joined.Frozen |= right.Frozen;
                joined.SymbolicWrites |= right.SymbolicWrites;
                joined.Initialized.IntersectWith(right.Initialized);
                joined.PossiblyInitialized.UnionWith(right.PossiblyInitialized);
                Arrays.Add(id, joined);
            }
        }
    }
    private sealed class Analyzer
    {
        internal readonly List<PracticalArrayStep> Steps = new();
        private CSharpCompilation compilation = null!;
        private IReadOnlyList<PracticalDataType> types = null!;
        private string path = "entry";
        private static string Site(IOperation operation) => operation.Syntax.SyntaxTree.FilePath + ":"
            + operation.Syntax.SpanStart.ToString(System.Globalization.CultureInfo.InvariantCulture) + ":"
            + operation.Kind.ToString();
        private void Step(IOperation source, string operation, IEnumerable<string>? arrays = null,
            IOperation? operand = null, string predicate = "", string exception = "") =>
            Steps.Add(new(Site(source), path, operation, source,
                Array.AsReadOnly((arrays ?? Array.Empty<string>()).OrderBy(id => id,StringComparer.Ordinal).ToArray()),
                operand, predicate, exception));

        internal void Analyze(CSharpCompilation current, IReadOnlyList<PracticalDataType> dataTypes)
        {
            compilation = current; types = dataTypes;
            foreach (SyntaxTree tree in current.SyntaxTrees) {
                SemanticModel semantic = current.GetSemanticModel(tree);
                foreach (SyntaxNode node in tree.GetRoot().DescendantNodes()) {
                    ITypeSymbol? type = node is TypeSyntax syntax ? semantic.GetTypeInfo(syntax).Type : null;
                    if (type is IArrayTypeSymbol { ElementType: IArrayTypeSymbol }) { Fail("array_jagged"); }
                    if (node is CollectionExpressionSyntax or StackAllocArrayCreationExpressionSyntax
                        or ImplicitStackAllocArrayCreationExpressionSyntax or RangeExpressionSyntax
                        || node.IsKind(SyntaxKind.IndexExpression)) { Fail("array_source_form"); }
                    // Validate inference independently of var, including return/argument contexts.
                    if (node is ImplicitArrayCreationExpressionSyntax implicitArray) {
                        var inferred = semantic.GetTypeInfo(node).Type as IArrayTypeSymbol;
                        if (inferred is null || implicitArray.Initializer.Expressions.Any(element =>
                            !SymbolEqualityComparer.IncludeNullability.Equals(semantic.GetTypeInfo(element).Type, inferred.ElementType)))
                        { Fail("array_best_common_type"); }
                    }
                    if (node is BaseMethodDeclarationSyntax or AccessorDeclarationSyntax
                        || node is ArrowExpressionClauseSyntax && node.Parent is PropertyDeclarationSyntax
                        || node is EqualsValueClauseSyntax && (node.Parent is PropertyDeclarationSyntax
                            || node.Parent is VariableDeclaratorSyntax variable && variable.Parent?.Parent is FieldDeclarationSyntax)) {
                        IOperation? body = semantic.GetOperation(node);
                        if (node is EqualsValueClauseSyntax equals) { body = semantic.GetOperation(equals.Value); }
                        if (body is null && node is ArrowExpressionClauseSyntax arrow) { body = semantic.GetOperation(arrow.Expression); }
                        if (body is not null) {
                            path = Site(body); var state = new State();
                            if (semantic.GetDeclaredSymbol(node) is IMethodSymbol method) {
                                foreach (IParameterSymbol parameter in method.Parameters.Where(p=>p.Type is IArrayTypeSymbol)) {
                                    Step(body,"parameter_profile_bound",new[]{"parameter:"+Site(body)+":"+parameter.Ordinal},
                                        predicate:parameter.NullableAnnotation == NullableAnnotation.Annotated
                                            ? "absent || 0 <= length <= 4096" : "0 <= length <= 4096");
                                }
                            }
                            var result = Visit(body, state);
                            Publish(body, result, state, node is EqualsValueClauseSyntax ? "storage_freeze" : "return_transfer");
                            foreach (string id in state.Arrays.Where(pair => state.Live && !pair.Value.Frozen).Select(pair=>pair.Key).ToArray())
                            { Step(body, "discard_on_exit", new[]{id}); }
                        }
                    }
                }
            }
        }
        private static void Fail(string code) => throw PracticalFailures.Type(code);
        private static HashSet<string> Empty() => new(StringComparer.Ordinal);
        private bool DefaultEligible(ITypeSymbol type)
        {
            if (type.IsReferenceType) { return type.NullableAnnotation == NullableAnnotation.Annotated; }
            if (type is INamedTypeSymbol named && named.OriginalDefinition.SpecialType == SpecialType.System_Nullable_T) { return true; }
            string id = PracticalExactTypeNormalizer.Normalize(type, compilation).Id;
            PracticalDataType? data = types.SingleOrDefault(item => item.Id == id);
            return data is null || data.DefaultEligible;
        }
        private HashSet<string> External(IOperation operation, State state)
        {
            if (operation.Type is not IArrayTypeSymbol) { return Empty(); }
            string id = "readonly:" + Site(operation);
            state.Arrays.TryAdd(id,new Storage { Complete = true, Frozen = true });
            Step(operation,"profile_bound",new[]{id}, operand:operation,
                predicate:operation.Type.NullableAnnotation == NullableAnnotation.Annotated
                    ? "absent || 0 <= length <= 4096" : "0 <= length <= 4096");
            return new(){id};
        }
        private HashSet<string> Visit(IOperation operation, State state)
        {
            if (!state.Live) { return Empty(); }
            switch (operation) {
                case ILoopOperation: Fail("array_loop_handoff"); break;
                case ITryOperation: Fail("array_exception_control_handoff"); break;
                case IConditionalOperation conditional:
                    Visit(conditional.Condition,state);
                    State yes = state.Copy(), no = state.Copy(); string previous = path;
                    path = previous + "/" + Site(operation) + ":true";
                    var yesValue = Visit(conditional.WhenTrue,yes);
                    path = previous + "/" + Site(operation) + ":false";
                    var noValue = conditional.WhenFalse is null ? Empty() : Visit(conditional.WhenFalse,no);
                    path = previous; state.Join(yes,no);
                    Step(operation,"merge",yesValue.Concat(noValue)); yesValue.UnionWith(noValue); return yesValue;
                case IBinaryOperation binary when binary.OperatorKind is BinaryOperatorKind.ConditionalAnd or BinaryOperatorKind.ConditionalOr:
                    Visit(binary.LeftOperand,state); State skip = state.Copy(), execute = state.Copy();
                    string outer = path; path += "/" + Site(operation) + ":rhs";
                    Visit(binary.RightOperand,execute); path = outer; state.Join(skip,execute);
                    Step(operation,"merge"); return Empty();
                case ICoalesceOperation coalesce:
                    var present = Visit(coalesce.Value,state); State nonnull = state.Copy(), absent = state.Copy();
                    string saved = path; path += "/" + Site(operation) + ":null";
                    var fallback = Visit(coalesce.WhenNull,absent); path = saved;
                    state.Join(nonnull,absent); present.UnionWith(fallback);
                    Step(operation,"merge",present); return present;
                case ISwitchOperation or ISwitchExpressionOperation or IBranchOperation or IConditionalAccessOperation
                    or ICoalesceAssignmentOperation or IIsPatternOperation or IAnonymousFunctionOperation or ILocalFunctionOperation:
                    Fail("array_control_handoff"); break;
                case ILocalReferenceOperation local:
                    return state.Locals.TryGetValue(local.Local,out var value) ? new(value) : External(operation,state);
                case IVariableDeclaratorOperation declaration:
                    if (declaration.Initializer is not null) {
                        var initial = Visit(declaration.Initializer.Value,state);
                        if (declaration.Symbol.Type is IArrayTypeSymbol) {
                            if (!FreshExpression(declaration.Initializer.Value)) { Publish(operation,initial,state,"alias_freeze"); }
                            state.Locals[declaration.Symbol] = initial;
                        }
                    }
                    return Empty();
                case IArrayCreationOperation creation: return Allocate(creation,state);
                case IConversionOperation conversion:
                    if (conversion.Operand.Type is IArrayTypeSymbol && conversion.Type is IArrayTypeSymbol
                        && !SymbolEqualityComparer.IncludeNullability.Equals(
                            ((IArrayTypeSymbol)conversion.Operand.Type).ElementType, ((IArrayTypeSymbol)conversion.Type).ElementType))
                    { Fail("array_covariance"); }
                    return Visit(conversion.Operand,state);
                case IParenthesizedOperation parentheses: return Visit(parentheses.Operand,state);
                case ISimpleAssignmentOperation assignment:
                    if (assignment.Target is IArrayElementReferenceOperation target) {
                        var storage = Address(target,state);
                        Visit(assignment.Value,state);
                        Step(assignment,"evaluate_value",storage,assignment.Value);
                        Write(target,storage,state,false); return Empty();
                    }
                    // C# evaluates a storage receiver before its right hand side.
                    if (assignment.Target is not ILocalReferenceOperation) { Visit(assignment.Target,state); }
                    var assigned = Visit(assignment.Value,state);
                    if (assignment.Target is ILocalReferenceOperation targetLocal) {
                        if (targetLocal.Type is IArrayTypeSymbol) {
                            if (!FreshExpression(assignment.Value)) { Publish(operation,assigned,state,"alias_freeze"); }
                            state.Locals[targetLocal.Local] = assigned;
                        }
                    } else { Publish(operation,assigned,state,"storage_freeze"); }
                    return assigned;
                case ICompoundAssignmentOperation compound when compound.Target is IArrayElementReferenceOperation compoundTarget:
                    var compoundStorage = Address(compoundTarget,state); Read(compoundTarget,compoundStorage,state);
                    Visit(compound.Value,state);
                    Step(compound,"evaluate_value",compoundStorage,compound.Value);
                    Write(compoundTarget,compoundStorage,state,true); return Empty();
                case IIncrementOrDecrementOperation increment when increment.Target is IArrayElementReferenceOperation incrementTarget:
                    var incrementStorage = Address(incrementTarget,state); Read(incrementTarget,incrementStorage,state);
                    Write(incrementTarget,incrementStorage,state,true); return Empty();
                case IArrayElementReferenceOperation element:
                    var readStorage = Address(element,state); Read(element,readStorage,state); return Empty();
                case IInvocationOperation call:
                    if (call.Instance is not null) { Visit(call.Instance,state); }
                    var arguments = Empty();
                    foreach (IArgumentOperation argument in call.Arguments) { arguments.UnionWith(Visit(argument.Value,state)); }
                    Publish(operation,arguments,state,"call_freeze"); return External(operation,state);
                case IObjectCreationOperation creation:
                    var constructorArguments = Empty();
                    foreach (IArgumentOperation argument in creation.Arguments) { constructorArguments.UnionWith(Visit(argument.Value,state)); }
                    Publish(operation,constructorArguments,state,"wrapper_freeze");
                    if (creation.Initializer is not null) { Visit(creation.Initializer,state); }
                    return Empty();
                case IReturnOperation returned:
                    var result = returned.ReturnedValue is null ? Empty() : Visit(returned.ReturnedValue,state);
                    Publish(operation,result,state,"return_transfer");
                    Step(operation,"discard_on_exit",state.Arrays.Where(pair=>!pair.Value.Frozen).Select(pair=>pair.Key));
                    state.Live=false; return Empty();
                case IThrowOperation thrown:
                    if (thrown.Exception is not null) { Visit(thrown.Exception,state); }
                    Step(operation,"discard_exception",state.Arrays.Where(pair=>!pair.Value.Frozen).Select(pair=>pair.Key)); state.Live=false; return Empty();
                case IPropertyReferenceOperation property when property.Instance?.Type is IArrayTypeSymbol && property.Property.Name == "Length":
                    var lengthStorage = Visit(property.Instance,state);
                    Step(operation,"null_check",lengthStorage,exception:"NullReferenceException");
                    Step(operation,"length",lengthStorage); return Empty();
                default:
                    foreach (IOperation child in operation.ChildOperations) { Visit(child,state); }
                    return External(operation,state);
            }
            return Empty();
        }
        private static bool FreshExpression(IOperation operation) => operation switch {
            IArrayCreationOperation => true,
            IConditionalOperation { WhenFalse: not null } c => FreshExpression(c.WhenTrue) && FreshExpression(c.WhenFalse),
            IParenthesizedOperation p => FreshExpression(p.Operand),
            IConversionOperation { Conversion.IsIdentity: true } c => FreshExpression(c.Operand),
            _ => false,
        };
        private HashSet<string> Allocate(IArrayCreationOperation creation, State state)
        {
            if (creation.Type is not IArrayTypeSymbol array || !array.IsSZArray || array.Rank != 1
                || array.ElementType is IArrayTypeSymbol || creation.DimensionSizes.Length != 1)
            { Fail("array_shape"); return Empty(); }
            if (creation.Initializer is not null && creation.Syntax is ArrayCreationExpressionSyntax explicitCreation
                && explicitCreation.Type.RankSpecifiers.SelectMany(rank=>rank.Sizes).Any(size=>size is not OmittedArraySizeExpressionSyntax))
            { Fail("array_explicit_length_initializer"); }
            IOperation dimension = creation.DimensionSizes[0];
            RequireInt(dimension); Visit(dimension,state);
            string id = "array:" + Site(creation); var ids = new HashSet<string>{id};
            int? length = ConstantInt(dimension);
            if (length > MaximumLength) { throw PracticalFailures.Limit("array_elements"); }
            Step(creation,"evaluate_length",ids,dimension);
            Step(creation,"csharp_length_check",ids,predicate:"length >= 0",exception:"OverflowException");
            Step(creation,"profile_bound",ids,predicate:"length <= 4096");
            state.Arrays[id] = new Storage { Length = length,
                Complete = length == 0 || DefaultEligible(array.ElementType) };
            Step(creation,"allocate_unique",ids,predicate:state.Arrays[id].Complete ? "recursive_default" : "uninitialized");
            if (creation.Initializer is not null) {
                foreach (IOperation element in creation.Initializer.ElementValues) {
                    Visit(element,state); Step(creation,"initialize_element",ids,element,predicate:"element_public_invariant");
                }
                state.Arrays[id].Complete = true;
                Step(creation,"complete_initializer",ids);
            }
            if (length < 0) { state.Live=false; }
            return ids;
        }
        private static int? ConstantInt(IOperation operation) => operation.ConstantValue.HasValue
            && operation.ConstantValue.Value is int value ? value : null;
        private static void RequireInt(IOperation operation)
        {
            if (operation.Type?.SpecialType != SpecialType.System_Int32
                || operation is IConversionOperation { IsImplicit: true, Conversion.IsIdentity: false })
            { Fail("array_exact_int"); }
        }
        private HashSet<string> Address(IArrayElementReferenceOperation element, State state)
        {
            var ids = Visit(element.ArrayReference,state);
            Step(element,"evaluate_array",ids,element.ArrayReference);
            if (element.Indices.Length != 1) { Fail("array_index_shape"); }
            IOperation index = element.Indices[0]; RequireInt(index); Visit(index,state);
            Step(element,"evaluate_index",ids,index);
            return ids;
        }
        private void Bounds(IArrayElementReferenceOperation element, HashSet<string> ids)
        {
            Step(element,"null_check",ids,exception:"NullReferenceException");
            Step(element,"index_lower_bound",ids,predicate:"index >= 0",exception:"IndexOutOfRangeException");
            Step(element,"index_upper_bound",ids,predicate:"index < length",exception:"IndexOutOfRangeException");
        }
        private void Read(IArrayElementReferenceOperation element, HashSet<string> ids, State state)
        {
            Bounds(element,ids);
            int? index = ConstantInt(element.Indices[0]);
            foreach (string id in ids) {
                Storage storage = state.Arrays[id];
                if (index is int bound && (bound < 0 || storage.Length is int size && bound >= size)) { continue; }
                if (!storage.Complete && !(index is int known && storage.Initialized.Contains(known))) {
                    if (index is int && storage.Length is int && !storage.SymbolicWrites) { Fail("array_uninitialized_read"); }
                    Step(element,"initialized_read_vc",new[]{id},predicate:"initialized(index)");
                }
            }
            Step(element,"read",ids);
        }
        private void Write(IArrayElementReferenceOperation element, HashSet<string> ids, State state, bool readModifyWrite)
        {
            IOperation receiver = element.ArrayReference;
            while (receiver is IParenthesizedOperation parentheses) { receiver = parentheses.Operand; }
            if (receiver is not ILocalReferenceOperation) { Fail("array_unique_local_write"); }
            Bounds(element,ids);
            int? index = ConstantInt(element.Indices[0]);
            foreach (string id in ids) {
                Storage storage = state.Arrays[id]; RequireWritable(!storage.Frozen,false);
                if (!storage.Complete) {
                    bool uncertainComplete = storage.Length is null || storage.SymbolicWrites;
                    if (!uncertainComplete && (readModifyWrite || index is int known && storage.PossiblyInitialized.Contains(known)))
                    { Fail("array_duplicate_initialization"); }
                    Step(element,"first_write_vc",new[]{id},predicate:readModifyWrite
                        ? "complete && element_public_invariant"
                        : "(complete || !initialized(index)) && element_public_invariant");
                    storage.SymbolicWrites |= index is null || storage.Length is null;
                    if (index is int concrete && concrete >= 0 && storage.Length is int length && concrete < length) {
                        storage.Initialized.Add(concrete); storage.PossiblyInitialized.Add(concrete);
                        storage.Complete = storage.Initialized.Count == length;
                    }
                }
            }
            Step(element,"functional_update",ids,element.Parent,predicate:"element_public_invariant");
        }
        private void Publish(IOperation operation, IEnumerable<string> arrays, State state, string kind)
        {
            foreach (string id in arrays.Distinct(StringComparer.Ordinal)) {
                Storage storage = state.Arrays[id];
                if (!storage.Complete) {
                    if (storage.Length is int && !storage.SymbolicWrites) { Fail("array_incomplete_publication"); }
                    Step(operation,"complete_publication_vc",new[]{id},predicate:"forall i in [0,length): initialized(i)");
                }
                storage.Frozen=true; Step(operation,kind,new[]{id});
            }
        }
    }
}
