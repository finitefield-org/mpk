using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using System.IO;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.FlowAnalysis;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

// CSHARP-03-T03-W04. Private analysis only; no success envelope or invariant
// discharge. Normalized operation bodies remain the source of expression code.
// A typed, private contract handoff. This is deliberately not a second JSON
// sidecar schema: T06-W01 owns parsing/serialization of the frozen expression
// union. These expression nodes carry no truth/verification attestation.
internal enum PracticalInvariantOperator { Boolean, Int32, Member, Not, And, Equal }
internal sealed class PracticalInvariantExpression
{
    internal PracticalInvariantExpression(PracticalInvariantOperator operation, string typeId,
        string value, params PracticalInvariantExpression[] children)
    {
        Operation = operation; TypeId = typeId; Value = value;
        Children = Array.AsReadOnly((PracticalInvariantExpression[])children.Clone());
    }
    internal PracticalInvariantOperator Operation { get; }
    internal string TypeId { get; }
    internal string Value { get; }
    internal IReadOnlyList<PracticalInvariantExpression> Children { get; }
}
internal sealed class PracticalTypeInvariantClaim
{
    internal PracticalTypeInvariantClaim(string typeId, string sourceSha256,
        PracticalInvariantExpression? construction, params PracticalInvariantExpression[] invariants)
    {
        TypeId = typeId; SourceSha256 = sourceSha256; Construction = construction;
        Invariants = Array.AsReadOnly((PracticalInvariantExpression[])invariants.Clone());
    }
    internal string TypeId { get; }
    internal string SourceSha256 { get; }
    internal PracticalInvariantExpression? Construction { get; }
    internal IReadOnlyList<PracticalInvariantExpression> Invariants { get; }
}
internal sealed record PracticalConstructionStep(string Kind, string Target, int Ordinal);
internal sealed record PracticalConstructionBlock(int Ordinal, uint DefinitelyAssigned,
    uint PossiblyAssigned, IReadOnlyList<PracticalConstructionStep> Steps,
    int? Fallthrough, int? Conditional);
internal sealed record PracticalConstructorPlan(string Id, string TypeId, string? DelegatesTo,
    bool Synthesized, IReadOnlyList<PracticalConstructionBlock> Blocks,
    uint DefinitelyAssigned, uint PossiblyAssigned, bool HasNormalExit);
internal sealed record PracticalDirectCall(string Target, string ReceiverType,
    IReadOnlyList<string> EvaluationOrder, bool NullCheckAfterArguments,
    IReadOnlyList<IOperation> Operands, string Site);
internal sealed record PracticalReceiverFunction(string Id, IReadOnlyList<PracticalNormalizedType> Parameters,
    PracticalNormalizedType Result, PracticalNormalizedCallable Body);
internal sealed record PracticalInvariantObligation(string TypeId, string Site,
    string Kind, IReadOnlyList<string> Members, bool Discharged = false,
    PracticalInvariantExpression? Expression = null);
// W05: each execution of Begin allocates a fresh transaction; Site identifies
// source provenance, never a reusable runtime object identity. Only Finalize
// publishes a value. Exception edges from initializer evaluations discard it.
internal enum PracticalInitializationStepKind
{
    EvaluateArgument, Begin, InvokeConstructor, ConstructionInvariant,
    EvaluateInitializer, Assign, PublicInvariant, Finalize,
}
internal sealed record PracticalInitializationStep(PracticalInitializationStepKind Kind,
    string Target, IOperation? Expression, string ExceptionalExit);
internal sealed record PracticalInitializationPlan(string Site, string TypeId, string ConstructorId,
    IReadOnlyList<PracticalInitializationStep> Steps, IReadOnlyList<string> MemberOrder,
    uint DefinitelyAssigned, uint PossiblyAssigned, bool HasNormalExit);

internal sealed class PracticalConstruction
{
    internal PracticalConstruction(PracticalDataTypes data, PracticalConstructorPlan[] constructors,
        PracticalDirectCall[] calls, PracticalInvariantObligation[] obligations, PracticalReceiverFunction[] functions,
        PracticalInitializationPlan[] initializations)
    {
        Data = data;
        Constructors = Array.AsReadOnly(constructors);
        Calls = Array.AsReadOnly(calls);
        Functions = Array.AsReadOnly(functions);
        Obligations = Array.AsReadOnly(obligations);
        Initializations = Array.AsReadOnly(initializations);
    }
    internal PracticalDataTypes Data { get; }
    internal IReadOnlyList<PracticalConstructorPlan> Constructors { get; }
    internal IReadOnlyList<PracticalDirectCall> Calls { get; }
    internal IReadOnlyList<PracticalReceiverFunction> Functions { get; }
    internal IReadOnlyList<PracticalInvariantObligation> Obligations { get; }
    internal IReadOnlyList<PracticalInitializationPlan> Initializations { get; }
    internal int ArtifactCount => 0;
}

internal static class CSharpPracticalConstruction
{
    internal const int ConstructorsPerTypeMaximum = 8;

    internal static PracticalConstruction Validate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs, ImmutableArray<MetadataReference> references,
        IReadOnlyList<PracticalTypeInvariantClaim>? invariantClaims = null, bool allowInitializers = false)
    {
        try
        {
            var model = new Model(allowInitializers);
            PracticalDataTypes data = CSharpPracticalDataTypes.Validate(selection, inputs, references,
                (current, closure, types) =>
                {
                    model.Analyze(current);
                    if (closure.Sidecars.Count != 0) { throw PracticalFailures.Object("unbound_invariant"); }
                    model.Finish(types, closure, invariantClaims);
                }, ValidateLimits, ValidateSignatures, deferDeclaredInvariantProof: invariantClaims is not null,
                allowInitializerConstruction: allowInitializers);
            return model.Build(data);
        }
        catch (PracticalCaptureFailure) { throw; }
        catch (Exception) { throw PracticalFailures.Protocol("construction_adapter"); }
    }

    private static string TypeId(INamedTypeSymbol type) => PracticalIdentity.SourceTypeId(
        type.ContainingNamespace.ToDisplayString(), type.Name);
    private static string CallableId(IMethodSymbol method, CSharpCompilation compilation) =>
        PracticalIdentity.CallableId(method.MethodKind == MethodKind.Constructor ? "constructor" : "method",
            method.ContainingNamespace.ToDisplayString(), TypeId(method.ContainingType),
            method.MethodKind == MethodKind.Constructor ? method.ContainingType.Name : method.Name,
            method.Parameters.Select(parameter => PracticalExactTypeNormalizer.Normalize(parameter.Type, compilation).Id),
            method.MethodKind == MethodKind.Constructor ? TypeId(method.ContainingType)
                : PracticalExactTypeNormalizer.Normalize(method.ReturnType, compilation, true).Id);

    private static void ValidateLimits(CSharpCompilation compilation)
    {
        foreach (SyntaxTree tree in compilation.SyntaxTrees)
        foreach (TypeDeclarationSyntax type in tree.GetRoot().DescendantNodes().OfType<TypeDeclarationSyntax>())
        {
            int count = type.Members.OfType<ConstructorDeclarationSyntax>()
                .Count(ctor => !ctor.Modifiers.Any(SyntaxKind.StaticKeyword));
            if (count > ConstructorsPerTypeMaximum)
            { throw PracticalFailures.Limit("constructors_per_type"); }
        }
    }

    private static void ValidateSignatures(CSharpCompilation compilation)
    {
        foreach (SyntaxTree tree in compilation.SyntaxTrees)
        foreach (SyntaxNode node in tree.GetRoot().DescendantNodes())
        {
            if (node is ParameterSyntax parameter && (parameter.Default is not null
                || parameter.Modifiers.Count != 0))
            { throw PracticalFailures.Declaration("parameter_form"); }
            if (node is ArgumentSyntax argument && (argument.NameColon is not null
                || argument.RefKindKeyword.RawKind != 0))
            { throw PracticalFailures.Declaration("argument_form"); }
            if (node is ImplicitObjectCreationExpressionSyntax or AnonymousObjectCreationExpressionSyntax)
            { throw PracticalFailures.Declaration("explicit_construction_required"); }
        }
    }

    internal static void RequireSynthesizedBody(byte[] actual, byte[] prefix, int expectedToken)
    {
        byte[] expected = new byte[prefix.Length + 5];
        prefix.CopyTo(expected, 0);
        System.Buffers.Binary.BinaryPrimitives.WriteInt32LittleEndian(expected.AsSpan(prefix.Length, 4), expectedToken);
        expected[^1] = 0x2a;
        if (!actual.SequenceEqual(expected)) { throw PracticalFailures.Object("synthesized_body_shape"); }
    }

    private readonly record struct State(uint Must, uint May)
    {
        internal State Assign(uint bit) => new(Must | bit, May | bit);
        internal State Merge(State other) => new(Must & other.Must, May | other.May);
    }

    private sealed class Model
    {
        private readonly bool allowInitializers;
        internal Model(bool allowInitializers) { this.allowInitializers = allowInitializers; }
        private readonly List<(IObjectCreationOperation Operation, INamedTypeSymbol Type, PracticalConstructorPlan Constructor)> initializers = new();
        private readonly List<PracticalInitializationPlan> initializationPlans = new();
        private CSharpCompilation compilation = null!;
        private readonly Dictionary<ISymbol, PracticalConstructorPlan> plans = new(SymbolEqualityComparer.Default);
        private readonly HashSet<ISymbol> visiting = new(SymbolEqualityComparer.Default);
        private readonly Dictionary<ISymbol, ISymbol[]> storage = new(SymbolEqualityComparer.Default);
        private readonly List<PracticalDirectCall> calls = new();
        private Dictionary<CaptureId, ISymbol> assignmentCaptures = new();
        private readonly List<(INamedTypeSymbol Type, string Site)> creations = new();
        private readonly List<(string Type, string Site)> defaults = new();

        internal void Analyze(CSharpCompilation current)
        {
            compilation = current;
            var types = new List<INamedTypeSymbol>();
            foreach (SyntaxTree tree in compilation.SyntaxTrees)
            {
                SemanticModel model = compilation.GetSemanticModel(tree);
                foreach (TypeDeclarationSyntax syntax in tree.GetRoot().DescendantNodes().OfType<TypeDeclarationSyntax>())
                {
                    var type = (INamedTypeSymbol)model.GetDeclaredSymbol(syntax)!;
                    if (type.IsStatic) { continue; }
                    types.Add(type);
                    storage.Add(type, type.GetMembers().Where(member => member is IFieldSymbol { IsImplicitlyDeclared: false, IsStatic: false }
                        || member is IPropertySymbol property && IsAuto(property)).ToArray());
                    ValidateSynthesized(type);
                }
            }
            foreach (SyntaxTree tree in compilation.SyntaxTrees)
            {
                SemanticModel semantic = compilation.GetSemanticModel(tree);
                foreach (ExpressionSyntax expression in tree.GetRoot().DescendantNodes().OfType<ExpressionSyntax>())
                {
                    IOperation? target = semantic.GetOperation(expression) switch
                    {
                        ICompoundAssignmentOperation assignment => assignment.Target,
                        IIncrementOrDecrementOperation increment => increment.Target,
                        _ => null,
                    };
                    if (target is IFieldReferenceOperation or IPropertyReferenceOperation)
                    { throw PracticalFailures.Object("duplicate_member_assignment"); }
                }
            }
            ValidateEmittedSynthesis(types);
            foreach (INamedTypeSymbol type in types)
            foreach (IMethodSymbol constructor in type.InstanceConstructors)
            {
                if (type.TypeKind == TypeKind.Struct && constructor.IsImplicitlyDeclared) { continue; }
                AnalyzeConstructor(constructor);
            }
            foreach (SyntaxTree tree in compilation.SyntaxTrees)
            {
                SemanticModel model = compilation.GetSemanticModel(tree);
                foreach (SyntaxNode node in tree.GetRoot().DescendantNodes())
                {
                    if (node is InvocationExpressionSyntax && model.GetOperation(node) is IInvocationOperation call)
                    { AddCall(call); }
                    if (node is DefaultExpressionSyntax && model.GetTypeInfo(node).Type is INamedTypeSymbol defaultType
                        && defaultType.IsValueType && !defaultType.DeclaringSyntaxReferences.IsEmpty)
                    { defaults.Add((TypeId(defaultType), Site(node))); }
                    if (node is ExpressionSyntax && model.GetOperation(node) is IPropertyReferenceOperation access
                        && access.Syntax == node && access.Property.DeclaringSyntaxReferences.Length != 0
                        && access.Parent is not ISimpleAssignmentOperation { Target: IPropertyReferenceOperation })
                    {
                        IMethodSymbol getter = access.Property.GetMethod!;
                        calls.Add(new(CallableId(getter, compilation), TypeId(getter.ContainingType),
                            Array.AsReadOnly(new[] { "receiver", "call" }), getter.ContainingType.IsReferenceType,
                            Array.AsReadOnly(new[] { access.Instance! }), Site(node)));
                    }
                    if (node is ObjectCreationExpressionSyntax && model.GetOperation(node) is IObjectCreationOperation creation
                        && creation.Type is INamedTypeSymbol type && storage.ContainsKey(type))
                    {
                        if (creation.Initializer is not null && !allowInitializers)
                        { throw PracticalFailures.Object("initializer_requires_w05"); }
                        ValidateArguments(creation.Arguments, creation.Constructor!);
                        PracticalConstructorPlan plan = AnalyzeConstructor(creation.Constructor!);
                        if (allowInitializers) { initializers.Add((creation, type, plan)); }
                        else if (plan.HasNormalExit) { creations.Add((type, Site(node))); }
                    }
                }
            }
        }

        private static bool IsAuto(IPropertySymbol property) =>
            property.DeclaringSyntaxReferences.SingleOrDefault()?.GetSyntax() is PropertyDeclarationSyntax
            { ExpressionBody: null, AccessorList: not null } syntax
            && syntax.AccessorList.Accessors.All(accessor => accessor.Body is null && accessor.ExpressionBody is null);

        private void ValidateSynthesized(INamedTypeSymbol type)
        {
            foreach (IFieldSymbol field in type.GetMembers().OfType<IFieldSymbol>().Where(field => field.IsImplicitlyDeclared))
            {
                if (field.AssociatedSymbol is not IPropertySymbol property || !IsAuto(property)
                    || field.Name != "<" + property.Name + ">k__BackingField"
                    || field.DeclaredAccessibility != Accessibility.Private || field.IsStatic || field.IsConst
                    || field.IsVolatile || field.RefKind != RefKind.None
                    || field.IsReadOnly != (property.SetMethod is null || property.SetMethod.IsInitOnly)
                    || !SymbolEqualityComparer.IncludeNullability.Equals(field.Type, property.Type))
                { throw PracticalFailures.Object("synthesized_backing_field"); }
            }
            foreach (IPropertySymbol property in type.GetMembers().OfType<IPropertySymbol>().Where(IsAuto))
            {
                if (type.GetMembers().OfType<IFieldSymbol>().Count(field =>
                    SymbolEqualityComparer.Default.Equals(field.AssociatedSymbol, property)) != 1
                    || property.GetMethod is not { IsStatic: false, IsVirtual: false, Parameters.Length: 0 }
                    || property.SetMethod is { IsInitOnly: false })
                { throw PracticalFailures.Object("synthesized_accessor"); }
            }
        }

        private PracticalConstructorPlan AnalyzeConstructor(IMethodSymbol constructor)
        {
            if (plans.TryGetValue(constructor, out PracticalConstructorPlan? existing)) { return existing; }
            if (!visiting.Add(constructor)) { throw PracticalFailures.Object("constructor_cycle"); }
            INamedTypeSymbol type = constructor.ContainingType;
            string id = CallableId(constructor, compilation);
            if (constructor.IsImplicitlyDeclared)
            {
                if (constructor.Parameters.Length != 0 || constructor.IsStatic || constructor.IsExtern
                    || constructor.DeclaredAccessibility != Accessibility.Public
                    || type.TypeKind == TypeKind.Class && (type.BaseType?.SpecialType != SpecialType.System_Object
                        || type.InstanceConstructors.Any(candidate => !candidate.IsImplicitlyDeclared)))
                { throw PracticalFailures.Object("synthesized_constructor"); }
                var implicitPlan = new PracticalConstructorPlan(id, TypeId(type), null, true,
                    Array.AsReadOnly(Array.Empty<PracticalConstructionBlock>()), 0, 0, true);
                plans.Add(constructor, implicitPlan); visiting.Remove(constructor); return implicitPlan;
            }
            var syntax = (ConstructorDeclarationSyntax)constructor.DeclaringSyntaxReferences.Single().GetSyntax();
            SemanticModel semantic = compilation.GetSemanticModel(syntax.SyntaxTree);
            var body = semantic.GetOperation(syntax) as IConstructorBodyOperation
                ?? throw PracticalFailures.Object("constructor_body");
            State initial = new(0, 0);
            string? delegatesTo = null;
            bool delegatedReturns = true;
            if (syntax.Initializer is not null)
            {
                if (!syntax.Initializer.IsKind(SyntaxKind.ThisConstructorInitializer)
                    || semantic.GetSymbolInfo(syntax.Initializer).Symbol is not IMethodSymbol target
                    || !SymbolEqualityComparer.Default.Equals(target.ContainingType, type))
                { throw PracticalFailures.Object("constructor_delegation"); }
                PracticalConstructorPlan delegated = AnalyzeConstructor(target);
                initial = new(delegated.DefinitelyAssigned, delegated.PossiblyAssigned);
                delegatesTo = delegated.Id;
                delegatedReturns = delegated.HasNormalExit;
            }
            ControlFlowGraph graph = ControlFlowGraph.Create(body);
            assignmentCaptures = new();
            IOperation[] operations = graph.Blocks.SelectMany(block => block.Operations
                .Concat(block.BranchValue is null ? Array.Empty<IOperation>() : new[] { block.BranchValue }))
                .SelectMany(Descendants).ToArray();
            foreach (IFlowCaptureOperation capture in operations.OfType<IFlowCaptureOperation>())
            {
                ISymbol? member = capture.Value switch
                {
                    IFieldReferenceOperation { Instance: IInstanceReferenceOperation } field => field.Field,
                    IPropertyReferenceOperation { Instance: IInstanceReferenceOperation } property => property.Property,
                    _ => null,
                };
                if (member is null) { continue; }
                IFlowCaptureReferenceOperation[] uses = operations.OfType<IFlowCaptureReferenceOperation>()
                    .Where(reference => reference.Id.Equals(capture.Id)).ToArray();
                if (uses.Length != 0 && uses.All(reference => reference.Parent is ISimpleAssignmentOperation assignment
                    && ReferenceEquals(assignment.Target, reference)))
                { assignmentCaptures.Add(capture.Id, member); }
            }
            var incoming = new State?[graph.Blocks.Length];
            incoming[0] = new State(0, 0);
            var queue = new Queue<int>(); queue.Enqueue(0);
            while (queue.Count != 0)
            {
                int ordinal = queue.Dequeue(); BasicBlock block = graph.Blocks[ordinal];
                State state = incoming[ordinal]!.Value;
                bool continues = true;
                foreach (IOperation operation in block.Operations)
                {
                    Visit(operation, type, ref state, initial, false, null);
                    if (!delegatedReturns && IsDelegation(operation, type)) { continues = false; break; }
                }
                if (!continues) { continue; }
                if (block.BranchValue is not null) { Visit(block.BranchValue, type, ref state, initial, false, null); }
                foreach (ControlFlowBranch? branch in new[] { block.FallThroughSuccessor, block.ConditionalSuccessor })
                {
                    if (branch?.Destination is not BasicBlock destination || !destination.IsReachable) { continue; }
                    State merged = incoming[destination.Ordinal] is State prior ? prior.Merge(state) : state;
                    if (incoming[destination.Ordinal] != merged)
                    { incoming[destination.Ordinal] = merged; queue.Enqueue(destination.Ordinal); }
                }
            }
            var blocks = new List<PracticalConstructionBlock>();
            foreach (BasicBlock block in graph.Blocks)
            {
                if (incoming[block.Ordinal] is not State state) { continue; }
                State start = state;
                var steps = new List<PracticalConstructionStep>();
                bool continues = true;
                foreach (IOperation operation in block.Operations)
                {
                    Visit(operation, type, ref state, initial, true, steps);
                    if (!delegatedReturns && IsDelegation(operation, type)) { continues = false; break; }
                }
                if (continues && block.BranchValue is not null) { Visit(block.BranchValue, type, ref state, initial, true, steps); }
                blocks.Add(new(block.Ordinal, start.Must, start.May, steps.AsReadOnly(),
                    continues ? block.FallThroughSuccessor?.Destination?.Ordinal : null,
                    continues ? block.ConditionalSuccessor?.Destination?.Ordinal : null));
            }
            State? exit = incoming[^1];
            var result = new PracticalConstructorPlan(id, TypeId(type), delegatesTo, false,
                blocks.AsReadOnly(), exit?.Must ?? 0, exit?.May ?? 0, exit is not null);
            plans.Add(constructor, result); visiting.Remove(constructor); return result;
        }

        private static IEnumerable<IOperation> Descendants(IOperation operation)
        {
            var pending = new Stack<IOperation>(); pending.Push(operation);
            while (pending.Count != 0)
            {
                IOperation current = pending.Pop(); yield return current;
                foreach (IOperation child in current.ChildOperations.Reverse()) { pending.Push(child); }
            }
        }

        private static bool IsDelegation(IOperation operation, INamedTypeSymbol type) =>
            Descendants(operation).OfType<IInvocationOperation>().Any(call =>
                call.TargetMethod.MethodKind == MethodKind.Constructor
                && SymbolEqualityComparer.Default.Equals(call.TargetMethod.ContainingType, type));

        private uint MemberBit(ISymbol symbol, INamedTypeSymbol type)
        {
            int index = Array.FindIndex(storage[type], member => SymbolEqualityComparer.Default.Equals(member, symbol));
            if (index < 0) { throw PracticalFailures.Object("construction_member"); }
            return 1u << index;
        }

        private void Visit(IOperation operation, INamedTypeSymbol type, ref State state, State delegated,
            bool validate, List<PracticalConstructionStep>? steps)
        {
            if (operation is IFlowCaptureOperation capture && assignmentCaptures.ContainsKey(capture.Id))
            { return; } // Only a compiler temporary for a direct assignment location, never an observable read.
            IOperation? updateTarget = operation switch
            {
                ICompoundAssignmentOperation compound => compound.Target,
                IIncrementOrDecrementOperation increment => increment.Target,
                _ => null,
            };
            if (validate && updateTarget is IFieldReferenceOperation or IPropertyReferenceOperation)
            { throw PracticalFailures.Object("duplicate_member_assignment"); }
            if (operation is ISimpleAssignmentOperation assignment)
            {
                ISymbol? member = assignment.Target switch
                {
                    IFieldReferenceOperation { Instance: IInstanceReferenceOperation } field => field.Field,
                    IPropertyReferenceOperation { Instance: IInstanceReferenceOperation } property => property.Property,
                    IFlowCaptureReferenceOperation reference when assignmentCaptures.TryGetValue(reference.Id, out ISymbol? captured) => captured,
                    _ => null,
                };
                if (member is not null)
                {
                    Visit(assignment.Value, type, ref state, delegated, validate, steps);
                    uint bit = MemberBit(member, type);
                    // Compiler zero initialization is not an explicit source write.
                    if (!assignment.IsImplicit)
                    {
                        if (validate && (state.May & bit) != 0) { throw PracticalFailures.Object("duplicate_member_assignment"); }
                        if (validate && member is IPropertySymbol { IsRequired: true })
                        { throw PracticalFailures.Object("required_constructor_assignment"); }
                        state = state.Assign(bit);
                        steps?.Add(new("assign", member.Name, steps.Count));
                    }
                    return;
                }
            }
            if (operation is IInvocationOperation call && call.TargetMethod.MethodKind == MethodKind.Constructor)
            {
                foreach (IArgumentOperation argument in call.Arguments) { Visit(argument.Value, type, ref state, delegated, validate, steps); }
                if (SymbolEqualityComparer.Default.Equals(call.TargetMethod.ContainingType, type))
                {
                    if (validate) { ValidateArguments(call.Arguments, call.TargetMethod); }
                    state = new(state.Must | delegated.Must, state.May | delegated.May);
                    steps?.Add(new("delegate", CallableId(call.TargetMethod, compilation), steps.Count));
                }
                else if (!call.IsImplicit || call.TargetMethod.ContainingType.SpecialType != SpecialType.System_Object
                    || call.Arguments.Length != 0)
                { throw PracticalFailures.Object("constructor_base"); }
                return;
            }
            if (operation is IFieldReferenceOperation { Instance: IInstanceReferenceOperation } read)
            {
                if (validate && (state.Must & MemberBit(read.Field, type)) == 0)
                { throw PracticalFailures.Object("member_not_definitely_assigned"); }
                steps?.Add(new("read", read.Field.Name, steps.Count)); return;
            }
            if (operation is IInstanceReferenceOperation)
            {
                if (validate) { throw PracticalFailures.Object("unfinished_receiver_escape"); }
                return;
            }
            foreach (IOperation child in operation.ChildOperations) { Visit(child, type, ref state, delegated, validate, steps); }
        }

        private static void ValidateArguments(ImmutableArray<IArgumentOperation> arguments, IMethodSymbol method)
        {
            if (arguments.Length != method.Parameters.Length || method.IsGenericMethod || method.IsExtensionMethod
                || method.Parameters.Any(parameter => parameter.RefKind != RefKind.None || parameter.IsOptional || parameter.IsParams))
            { throw PracticalFailures.Object("call_signature"); }
            for (int index = 0; index < arguments.Length; index++)
            {
                IArgumentOperation argument = arguments[index];
                if (argument.ArgumentKind != ArgumentKind.Explicit || argument.Parameter?.Ordinal != index
                    || argument.InConversion.IsUserDefined || argument.OutConversion.IsUserDefined)
                { throw PracticalFailures.Object("call_argument"); }
            }
        }

        private static string Site(SyntaxNode node) => node.SyntaxTree.FilePath + ":" + node.SpanStart + ":" + node.Span.Length;

        private void AddCall(IInvocationOperation call)
        {
            IMethodSymbol method = call.TargetMethod;
            if (method.DeclaringSyntaxReferences.IsEmpty) { return; }
            ValidateArguments(call.Arguments, method);
            if (call.IsVirtual || method.IsAbstract || method.IsVirtual || method.IsOverride || method.ReducedFrom is not null)
            { throw PracticalFailures.Object("instance_dispatch"); }
            var order = new List<string>();
            if (!method.IsStatic) { order.Add("receiver"); }
            order.AddRange(call.Arguments.Select((_, index) => "argument:" + index));
            order.Add("call");
            calls.Add(new(CallableId(method, compilation), method.IsStatic ? "" : TypeId(method.ContainingType),
                order.AsReadOnly(), !method.IsStatic && method.ContainingType.IsReferenceType,
                Array.AsReadOnly((method.IsStatic ? Array.Empty<IOperation>() : new[] { call.Instance! })
                    .Concat(call.Arguments.Select(argument => argument.Value)).ToArray()), Site(call.Syntax)));
        }

        internal void Finish(IReadOnlyList<PracticalDataType> types, PracticalSourceClosure closure,
            IReadOnlyList<PracticalTypeInvariantClaim>? invariantClaims)
        {
            var obligations = new List<PracticalInvariantObligation>();
            Dictionary<string, PracticalTypeInvariantClaim> claims = BindClaims(types, closure, invariantClaims);

            foreach (var pair in plans)
            {
                PracticalConstructorPlan plan = pair.Value;
                if (!plan.HasNormalExit) { continue; }
                PracticalDataType type = types.Single(type => type.Id == plan.TypeId);
                ISymbol[] members = storage[((IMethodSymbol)pair.Key).ContainingType];
                var defaulted = new List<string>();
                for (int index = 0; index < members.Length; index++)
                {
                    if ((plan.DefinitelyAssigned & (1u << index)) != 0) { continue; }
                    PracticalDataMember member = type.Members.Single(member => member.Name == members[index].Name);
                    if (member.Required) { continue; }
                    if (allowInitializers && members[index] is IPropertySymbol { SetMethod.IsInitOnly: true })
                    { continue; } // Its constructor zero storage is finalized by this creation's initializer.
                    if (!DefaultAvailable(member.Type, types))
                    { throw PracticalFailures.Object("member_without_final_value"); }
                    defaulted.Add(member.Name);
                    EmitDefaultObligations(member.Type.Id, plan.Id + ":default:" + member.Name, types, obligations);
                }
                if (defaulted.Count != 0)
                { obligations.Add(new(type.Id, plan.Id, "recursive_default", defaulted.AsReadOnly())); }
                bool init = members.OfType<IPropertySymbol>().Any(property => property.SetMethod is not null);
                obligations.Add(new(type.Id, plan.Id, init ? "construction_invariant" : "public_invariant",
                    Array.AsReadOnly(members.Where(member => !init || member is not IPropertySymbol { IsRequired: true })
                        .Where(member => !allowInitializers || member is not IPropertySymbol { SetMethod.IsInitOnly: true }
                            || (plan.DefinitelyAssigned & MemberBit(member, ((IMethodSymbol)pair.Key).ContainingType)) != 0)
                        .Select(member => member.Name).ToArray())));
            }
            foreach (var initializer in initializers)
            { FinalizeInitializer(initializer.Operation, initializer.Type, initializer.Constructor, types, obligations); }
            foreach (var creation in creations)
            {
                obligations.Add(new(TypeId(creation.Type), creation.Site, "public_invariant",
                    Array.AsReadOnly(storage[creation.Type].Select(member => member.Name).ToArray())));
                if (storage[creation.Type].OfType<IPropertySymbol>().Any(property => property.IsRequired))
                { throw PracticalFailures.Object("required_completion_requires_w05"); }
            }
            foreach (var value in defaults)
            { EmitDefaultObligations(value.Type, value.Site, types, obligations); }
            foreach (PracticalInvariantObligation obligation in obligations.ToArray())
            {
                if (!claims.TryGetValue(obligation.TypeId, out PracticalTypeInvariantClaim? claim)) { continue; }
                if (obligation.Kind == "construction_invariant" && claim.Construction is not null)
                { obligations.Add(obligation with { Kind = "declared_construction_invariant", Expression = claim.Construction }); }
                else if (obligation.Kind == "public_invariant" || obligation.Kind == "default_public_invariant")
                {
                    foreach (PracticalInvariantExpression expression in claim.Invariants)
                    { obligations.Add(obligation with { Kind = "declared_" + obligation.Kind, Expression = expression }); }
                }
            }
            completedObligations = obligations.ToArray();
        }

        private void FinalizeInitializer(IObjectCreationOperation creation, INamedTypeSymbol type,
            PracticalConstructorPlan constructor, IReadOnlyList<PracticalDataType> types,
            List<PracticalInvariantObligation> obligations)
        {
            string site = Site(creation.Syntax);
            ISymbol[] members = storage[type];
            State state = new(constructor.DefinitelyAssigned, constructor.PossiblyAssigned);
            var assignments = new List<(IPropertySymbol Property, IOperation Value)>();
            if (creation.Initializer is not null)
            {
                if (!creation.Initializer.Syntax.IsKind(SyntaxKind.ObjectInitializerExpression))
                { throw PracticalFailures.Object("initializer_shape"); }
                foreach (IOperation initializer in creation.Initializer.Initializers)
                {
                    if (initializer is not ISimpleAssignmentOperation assignment || assignment.IsRef
                        || assignment.Target is not IPropertyReferenceOperation target
                        || target.Instance is not IInstanceReferenceOperation { ReferenceKind: InstanceReferenceKind.ImplicitReceiver }
                        || target.Arguments.Length != 0
                        || !SymbolEqualityComparer.Default.Equals(target.Property.ContainingType, type)
                        || !IsAuto(target.Property) || target.Property.SetMethod is not { IsInitOnly: true })
                    { throw PracticalFailures.Object("initializer_target"); }
                    uint bit = MemberBit(target.Property, type);
                    if ((state.May & bit) != 0) { throw PracticalFailures.Object("duplicate_member_assignment"); }
                    state = state.Assign(bit);
                    assignments.Add((target.Property, assignment.Value));
                }
            }
            foreach (IPropertySymbol required in members.OfType<IPropertySymbol>().Where(property => property.IsRequired))
            {
                if (!assignments.Any(assignment => SymbolEqualityComparer.Default.Equals(assignment.Property, required)))
                { throw PracticalFailures.Object("required_member_missing"); }
            }
            var steps = new List<PracticalInitializationStep>();
            foreach (IArgumentOperation argument in creation.Arguments)
            { steps.Add(new(PracticalInitializationStepKind.EvaluateArgument, "argument:" + argument.Parameter!.Ordinal, argument.Value, "no_value")); }
            steps.Add(new(PracticalInitializationStepKind.Begin, site, null, "no_value"));
            steps.Add(new(PracticalInitializationStepKind.InvokeConstructor, constructor.Id, null, "discard"));
            if (constructor.HasNormalExit)
            {
                if (members.OfType<IPropertySymbol>().Any(property => property.SetMethod is not null))
                { steps.Add(new(PracticalInitializationStepKind.ConstructionInvariant, constructor.Id, null, "discard")); }
                foreach (var assignment in assignments)
                {
                    steps.Add(new(PracticalInitializationStepKind.EvaluateInitializer, assignment.Property.Name, assignment.Value, "discard"));
                    steps.Add(new(PracticalInitializationStepKind.Assign, assignment.Property.Name, null, "discard"));
                }
                PracticalDataType dataType = types.Single(candidate => candidate.Id == constructor.TypeId);
                var defaulted = new List<string>();
                for (int index = 0; index < members.Length; index++)
                {
                    if ((state.Must & (1u << index)) != 0) { continue; }
                    PracticalDataMember member = dataType.Members.Single(member => member.Name == members[index].Name);
                    if (!DefaultAvailable(member.Type, types)) { throw PracticalFailures.Object("member_without_final_value"); }
                    defaulted.Add(member.Name);
                    EmitDefaultObligations(member.Type.Id, site + ":default:" + member.Name, types, obligations);
                }
                if (defaulted.Count != 0)
                { obligations.Add(new(constructor.TypeId, site, "recursive_default", defaulted.AsReadOnly())); }
                obligations.Add(new(constructor.TypeId, site, "public_invariant",
                    Array.AsReadOnly(members.Select(member => member.Name).ToArray())));
                steps.Add(new(PracticalInitializationStepKind.PublicInvariant, site, null, "discard"));
                steps.Add(new(PracticalInitializationStepKind.Finalize, site, null, "discard"));
            }
            initializationPlans.Add(new(site, constructor.TypeId, constructor.Id, steps.AsReadOnly(),
                Array.AsReadOnly(members.Select(member => member.Name).ToArray()), state.Must, state.May, constructor.HasNormalExit));
        }

        private PracticalInvariantObligation[] completedObligations = Array.Empty<PracticalInvariantObligation>();
        internal PracticalConstruction Build(PracticalDataTypes data) => new(data,
            plans.Values.OrderBy(plan => plan.Id, StringComparer.Ordinal).ToArray(),
            calls.ToArray(), completedObligations, ReceiverFunctions(data), initializationPlans.ToArray());

        private PracticalReceiverFunction[] ReceiverFunctions(PracticalDataTypes data)
        {
            var functions = new List<PracticalReceiverFunction>();
            foreach (SyntaxTree tree in compilation.SyntaxTrees)
            {
                SemanticModel semantic = compilation.GetSemanticModel(tree);
                foreach (SyntaxNode node in tree.GetRoot().DescendantNodes())
                {
                    IMethodSymbol? method = node switch
                    {
                        MethodDeclarationSyntax declaration => semantic.GetDeclaredSymbol(declaration),
                        PropertyDeclarationSyntax property => (semantic.GetDeclaredSymbol(property) as IPropertySymbol)?.GetMethod,
                        _ => null,
                    };
                    if (method is null || method.IsStatic) { continue; }
                    string id = CallableId(method, compilation);
                    PracticalNormalizedType receiver = PracticalExactTypeNormalizer.Normalize(method.ContainingType, compilation,
                        topLevelNullability: NullableAnnotation.NotAnnotated);
                    var parameters = new[] { receiver }.Concat(method.Parameters.Select(parameter =>
                        PracticalExactTypeNormalizer.Normalize(parameter.Type, compilation))).ToArray();
                    functions.Add(new(id, Array.AsReadOnly(parameters),
                        PracticalExactTypeNormalizer.Normalize(method.ReturnType, compilation, true),
                        data.Syntax.Callables.Single(callable => callable.Id == id)));
                }
            }
            return functions.OrderBy(function => function.Id, StringComparer.Ordinal).ToArray();
        }

        private void ValidateEmittedSynthesis(IEnumerable<INamedTypeSymbol> types)
        {
            using var stream = new MemoryStream();
            if (!compilation.Emit(stream).Success) { throw PracticalFailures.Object("synthesized_emit"); }
            stream.Position = 0;
            using var pe = new PEReader(stream);
            MetadataReader reader = pe.GetMetadataReader();
            foreach (INamedTypeSymbol type in types)
            {
                TypeDefinition definition = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(candidate =>
                    reader.GetString(candidate.Name) == type.MetadataName
                    && reader.GetString(candidate.Namespace) == type.ContainingNamespace.ToDisplayString());
                foreach (MethodDefinitionHandle handle in definition.GetMethods())
                {
                    MethodDefinition method = reader.GetMethodDefinition(handle);
                    string name = reader.GetString(method.Name);
                    bool implicitConstructor = name == ".ctor" && type.TypeKind == TypeKind.Class
                        && type.InstanceConstructors.Length == 1 && type.InstanceConstructors[0].IsImplicitlyDeclared;
                    IPropertySymbol? property = type.GetMembers().OfType<IPropertySymbol>().SingleOrDefault(candidate =>
                        IsAuto(candidate) && (name == "get_" + candidate.Name || name == "set_" + candidate.Name));
                    if (!implicitConstructor && property is null) { continue; }
                    MethodBodyBlock body = pe.GetMethodBody(method.RelativeVirtualAddress);
                    if (body.ExceptionRegions.Length != 0 || !body.LocalSignature.IsNil)
                    { throw PracticalFailures.Object("synthesized_body_shape"); }
                    byte[] il = body.GetILBytes() ?? throw PracticalFailures.Object("synthesized_body_shape");
                    if (implicitConstructor)
                    {
                        if (il.Length != 7) { throw PracticalFailures.Object("synthesized_constructor_il"); }
                        int token = System.Buffers.Binary.BinaryPrimitives.ReadInt32LittleEndian(il.AsSpan(2, 4));
                        EntityHandle called = MetadataTokens.EntityHandle(token);
                        if (called.Kind != HandleKind.MemberReference) { throw PracticalFailures.Object("synthesized_constructor_il"); }
                        MemberReference reference = reader.GetMemberReference((MemberReferenceHandle)called);
                        if (reference.Parent.Kind != HandleKind.TypeReference || reader.GetString(reference.Name) != ".ctor"
                            || !reader.GetBlobBytes(reference.Signature).SequenceEqual(new byte[] { 0x20, 0, 1 }))
                        { throw PracticalFailures.Object("synthesized_constructor_il"); }
                        TypeReference parent = reader.GetTypeReference((TypeReferenceHandle)reference.Parent);
                        if (reader.GetString(parent.Name) != "Object" || reader.GetString(parent.Namespace) != "System"
                            || parent.ResolutionScope.Kind != HandleKind.AssemblyReference
                            || reader.GetString(reader.GetAssemblyReference((AssemblyReferenceHandle)parent.ResolutionScope).Name) != "System.Runtime")
                        { throw PracticalFailures.Object("synthesized_constructor_il"); }
                        RequireSynthesizedBody(il, new byte[] { 0x02, 0x28 }, token);
                    }
                    else
                    {
                        FieldDefinitionHandle field = definition.GetFields().Single(field => reader.GetString(reader.GetFieldDefinition(field).Name)
                            == "<" + property!.Name + ">k__BackingField");
                        RequireSynthesizedBody(il, name.StartsWith("get_", StringComparison.Ordinal)
                            ? new byte[] { 0x02, 0x7b } : new byte[] { 0x02, 0x03, 0x7d }, MetadataTokens.GetToken(field));
                    }
                }
            }
        }

        private Dictionary<string, PracticalTypeInvariantClaim> BindClaims(IReadOnlyList<PracticalDataType> types, PracticalSourceClosure closure,
            IReadOnlyList<PracticalTypeInvariantClaim>? supplied)
        {
            var result = new Dictionary<string, PracticalTypeInvariantClaim>(StringComparer.Ordinal);
            if (supplied is null) { return result; } // No declared claims; member domains still emit obligations.
            foreach (PracticalTypeInvariantClaim claim in supplied)
            {
                PracticalDataType? type = types.SingleOrDefault(type => type.Id == claim.TypeId);
                PracticalDeclaration? declaration = closure.Declarations.SingleOrDefault(
                    declaration => declaration.Kind == PracticalDeclarationKind.Type && declaration.Id == claim.TypeId);
                if (type is null || declaration is null || !result.TryAdd(claim.TypeId, claim)
                    || claim.SourceSha256 != closure.Sources[declaration.SourceOrdinal].RawSha256)
                { throw PracticalFailures.Object("invariant_attachment"); }
                INamedTypeSymbol? symbol = storage.Keys.OfType<INamedTypeSymbol>().SingleOrDefault(symbol => TypeId(symbol) == type.Id);
                bool init = symbol is not null && storage[symbol].OfType<IPropertySymbol>().Any(property => property.SetMethod is not null);
                if (init != (claim.Construction is not null))
                { throw PracticalFailures.Object("construction_invariant_attachment"); }
                foreach (PracticalInvariantExpression expression in claim.Invariants)
                { CheckInvariant(expression, type, symbol); }
                if (claim.Construction is not null) { CheckInvariant(claim.Construction, type, symbol); }
            }
            if (types.Any(type => closure.Declarations.Any(declaration => declaration.Kind == PracticalDeclarationKind.Type
                && declaration.Id == type.Id) && !result.ContainsKey(type.Id)))
            { throw PracticalFailures.Object("missing_type_invariant"); }
            return result;
        }

        private void CheckInvariant(PracticalInvariantExpression root, PracticalDataType type, INamedTypeSymbol? symbol)
        {
            var checkedTypes = new Dictionary<PracticalInvariantExpression, string>();
            var pending = new Stack<(PracticalInvariantExpression Expression, bool Finish)>();
            pending.Push((root, false));
            while (pending.Count != 0)
            {
                var item = pending.Pop();
                if (checkedTypes.ContainsKey(item.Expression)) { continue; }
                if (!item.Finish)
                {
                    pending.Push((item.Expression, true));
                    foreach (PracticalInvariantExpression child in item.Expression.Children) { pending.Push((child, false)); }
                }
                else
                {
                    checkedTypes.Add(item.Expression, CheckInvariantNode(item.Expression, type, symbol,
                        item.Expression.Children.Select(child => checkedTypes[child]).ToArray()));
                }
            }
            if (checkedTypes[root] != PracticalIdentity.PrimitiveId("bool"))
            { throw PracticalFailures.Object("ill_typed_invariant"); }
        }

        private static string CheckInvariantNode(PracticalInvariantExpression expression, PracticalDataType type,
            INamedTypeSymbol? symbol, string[] children)
        {
            string boolean = PracticalIdentity.PrimitiveId("bool");
            string integer = PracticalIdentity.PrimitiveId("i32");
            bool valid = expression.Operation switch
            {
                PracticalInvariantOperator.Boolean => children.Length == 0 && expression.TypeId == boolean
                    && expression.Value is "true" or "false",
                PracticalInvariantOperator.Int32 => children.Length == 0 && expression.TypeId == integer
                    && int.TryParse(expression.Value, System.Globalization.NumberStyles.AllowLeadingSign,
                        System.Globalization.CultureInfo.InvariantCulture, out int value)
                    && value.ToString(System.Globalization.CultureInfo.InvariantCulture) == expression.Value,
                PracticalInvariantOperator.Member => children.Length == 0 && symbol is not null
                    && type.Members.Any(member => member.Stored && member.Type.Id == expression.TypeId
                        && expression.Value == symbol.ContainingNamespace.ToDisplayString() + "." + symbol.Name + "." + member.Name),
                PracticalInvariantOperator.Not => expression.Value == "" && expression.TypeId == boolean
                    && children.SequenceEqual(new[] { boolean }),
                PracticalInvariantOperator.And => expression.Value == "" && expression.TypeId == boolean
                    && children.SequenceEqual(new[] { boolean, boolean }),
                // Class identity is never introduced by a contract expression.
                PracticalInvariantOperator.Equal => expression.Value == "" && expression.TypeId == boolean
                    && children.Length == 2 && children[0] == children[1] && children[0] is var childType
                    && (childType == boolean || childType == integer),
                _ => false,
            };
            if (!valid)
            { throw PracticalFailures.Object("ill_typed_invariant"); }
            return expression.TypeId;
        }

        private static void EmitDefaultObligations(string typeId, string site,
            IReadOnlyList<PracticalDataType> types, List<PracticalInvariantObligation> obligations)
        {
            var pending = new Queue<string>(); pending.Enqueue(typeId);
            var seen = new HashSet<string>(StringComparer.Ordinal);
            while (pending.Count != 0)
            {
                string current = pending.Dequeue();
                if (!seen.Add(current)) { continue; }
                PracticalDataType? type = types.SingleOrDefault(type => type.Id == current);
                if (type?.DefaultValue is null) { continue; } // Primitive or nullable-none leaf.
                obligations.Add(new(type.Id, site, "default_public_invariant",
                    Array.AsReadOnly(type.Members.Where(member => member.Stored).Select(member => member.Name).ToArray())));
                foreach (PracticalDataMember member in type.Members.Where(member => member.Stored))
                { pending.Enqueue(member.Type.Id); }
            }
        }

        private static bool DefaultAvailable(PracticalNormalizedType type, IReadOnlyList<PracticalDataType> types)
        {
            if (type.Nullability == "annotated" || type.Arguments.Count == 1 && type.Id == PracticalIdentity.ClosedInstanceId("option", type.Arguments[0].Id))
            { return true; }
            PracticalDataType? source = types.SingleOrDefault(candidate => candidate.Id == type.Id);
            if (source is not null) { return source.DefaultValue is not null; }
            return new[] { "bool", "i8", "u8", "i16", "u16", "i32", "u32", "i64", "u64", "char", "f32", "f64", "decimal", "guid", "date", "time", "duration" }
                .Any(primitive => type.Id == PracticalIdentity.PrimitiveId(primitive));
        }
    }
}

// W05 composes W04 without changing the earlier stage's admission boundary.
internal static class CSharpPracticalInitialization
{
    internal static PracticalConstruction Validate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs, ImmutableArray<MetadataReference> references,
        IReadOnlyList<PracticalTypeInvariantClaim>? invariantClaims = null) =>
        CSharpPracticalConstruction.Validate(selection, inputs, references, invariantClaims, allowInitializers: true);
}
