using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.Encodings.Web;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

// CSHARP-03-T03-W03: a private declaration/type handoff, not a success envelope
// or construction proof. W04 owns constructor finalization and invariants.
internal sealed class PracticalEnumMember
{
    internal PracticalEnumMember(string name, string value) { Name = name; Value = value; }
    internal string Name { get; }
    internal string Value { get; }
}

internal sealed class PracticalDataMember
{
    internal PracticalDataMember(string name, string kind, PracticalNormalizedType type,
        bool required, bool stored)
    {
        Name = name;
        Kind = kind;
        Type = type;
        Required = required;
        Stored = stored;
    }
    internal string Name { get; }
    internal string Kind { get; }
    internal PracticalNormalizedType Type { get; }
    internal bool Required { get; }
    internal bool Stored { get; }
}

// A shared DAG of independently expanded zero/null values. Sharing prevents a
// bounded 32-way, 16-edge type graph from causing exponential serialization.
internal sealed class PracticalDefaultValue
{
    internal PracticalDefaultValue(string typeId, string kind, string scalar,
        PracticalDefaultValue[] members)
    {
        TypeId = typeId;
        Kind = kind;
        Scalar = scalar;
        Members = Array.AsReadOnly((PracticalDefaultValue[])members.Clone());
    }
    internal string TypeId { get; }
    internal string Kind { get; }
    internal string Scalar { get; }
    internal IReadOnlyList<PracticalDefaultValue> Members { get; }
}

internal sealed class PracticalDataType
{
    internal PracticalDataType(string id, string kind, string carrier,
        PracticalDataMember[] members, PracticalEnumMember[] enumMembers,
        int depth, PracticalDefaultValue? defaultValue, bool invariantPending)
    {
        Id = id;
        Kind = kind;
        Carrier = carrier;
        Members = Array.AsReadOnly((PracticalDataMember[])members.Clone());
        EnumMembers = Array.AsReadOnly((PracticalEnumMember[])enumMembers.Clone());
        StructuralDepth = depth;
        DefaultValue = defaultValue;
        DefaultInvariantPending = invariantPending;
    }
    internal string Id { get; }
    internal string Kind { get; }
    internal string Carrier { get; }
    internal IReadOnlyList<PracticalDataMember> Members { get; }
    internal IReadOnlyList<PracticalEnumMember> EnumMembers { get; }
    internal int StructuralDepth { get; }
    internal PracticalDefaultValue? DefaultValue { get; }
    internal bool DefaultInvariantPending { get; }
    internal bool DefaultEligible => DefaultValue is not null && !DefaultInvariantPending;
}

internal sealed class PracticalSourceExceptionCandidate
{
    internal PracticalSourceExceptionCandidate(string sourceId, SyntaxReference declaration)
    {
        SourceId = sourceId;
        Declaration = declaration;
    }
    internal string SourceId { get; }
    // The complete immutable original declaration, including its unvalidated
    // base clause, is retained for T04-W04. Classification admits no base.
    internal SyntaxReference Declaration { get; }
}

internal sealed class PracticalDataTypes
{
    private readonly byte[] canonicalBytes;
    internal PracticalDataTypes(PracticalNormalizedSyntax syntax, PracticalDataType[] types,
        byte[] canonicalBytes)
    {
        Syntax = syntax;
        Types = Array.AsReadOnly((PracticalDataType[])types.Clone());
        this.canonicalBytes = (byte[])canonicalBytes.Clone();
    }
    internal PracticalNormalizedSyntax Syntax { get; }
    internal IReadOnlyList<PracticalDataType> Types { get; }
    internal int ArtifactCount => 0;
    internal byte[] CopyCanonicalBytes() => (byte[])canonicalBytes.Clone();
}

internal static class CSharpPracticalDataTypes
{
    internal const int FieldsPropertiesPerTypeMaximum = 32;
    internal const int StructuralTypeNestingMaximum = 16;

    internal static PracticalDataTypes Validate(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs, ImmutableArray<MetadataReference> references,
        Action<CSharpCompilation, PracticalSourceClosure, IReadOnlyList<PracticalDataType>>? validateConstruction = null,
        Action<CSharpCompilation>? validateConstructorLimits = null,
        Action<CSharpCompilation>? validateSignatures = null,
        bool deferDeclaredInvariantProof = false, bool allowInitializerConstruction = false)
    {
        try
        {
            if (selection is null) { throw PracticalFailures.Protocol("selection_shape"); }
            if ((deferDeclaredInvariantProof || allowInitializerConstruction) && validateConstruction is null)
            { throw PracticalFailures.Protocol("missing_invariant_obligation_consumer"); }
            var model = new DataModel(selection.SidecarPaths.Count != 0 || deferDeclaredInvariantProof,
                deferDeclaredInvariantProof, allowInitializerConstruction);
            PracticalNormalizedSyntax syntax = CSharpPracticalSyntaxNormalizer.Normalize(
                selection, inputs, references,
                current => { model.ValidateDeclarations(current); validateSignatures?.Invoke(current); },
                model.ValidateTypes,
                current => { model.ValidateLimits(current); validateConstructorLimits?.Invoke(current); },
                validateConstruction is null ? null :
                    (current, closure) => validateConstruction(current, closure, model.GetTypes()));
            return model.Build(syntax);
        }
        catch (PracticalCaptureFailure) { throw; }
        catch (Exception) { throw PracticalFailures.Protocol("data_types"); }
    }

    internal static PracticalSourceExceptionCandidate? ClassifySourceException(INamedTypeSymbol type)
    {
        if (type.TypeKind != TypeKind.Class || type.DeclaringSyntaxReferences.Length != 1)
        {
            return null;
        }
        var visited = new HashSet<ISymbol>(SymbolEqualityComparer.Default);
        for (INamedTypeSymbol? current = type.BaseType; current is not null; current = current.BaseType)
        {
            if (!visited.Add(current)) { break; }
            if (ExactRuntime(current, "Exception"))
            {
                return new PracticalSourceExceptionCandidate(SourceId(type), type.DeclaringSyntaxReferences[0]);
            }
        }
        return null;
    }

    private static string SourceId(INamedTypeSymbol type) => PracticalIdentity.SourceTypeId(
        type.ContainingNamespace.ToDisplayString(), type.Name);

    private static bool ExactRuntime(ITypeSymbol? type, string name) =>
        type is INamedTypeSymbol named && named.Arity == 0
        && named.DeclaringSyntaxReferences.IsEmpty && named.MetadataName == name
        && named.ContainingNamespace.ToDisplayString() == "System"
        && named.ContainingAssembly?.Identity.Name == "System.Runtime";

    private static bool IntegerCarrier(SpecialType type) => type is
        SpecialType.System_SByte or SpecialType.System_Byte or SpecialType.System_Int16
        or SpecialType.System_UInt16 or SpecialType.System_Int32 or SpecialType.System_UInt32
        or SpecialType.System_Int64 or SpecialType.System_UInt64;

    private static void Modifiers(SyntaxTokenList modifiers, params SyntaxKind[] allowed)
    {
        foreach (SyntaxToken token in modifiers)
        {
            if (!allowed.Contains(token.Kind())) { throw PracticalFailures.Declaration("data_modifier"); }
        }
    }

    private sealed class TypeRecord
    {
        internal TypeRecord(INamedTypeSymbol symbol, BaseTypeDeclarationSyntax syntax, SemanticModel model)
        { Symbol = symbol; Syntax = syntax; Model = model; }
        internal INamedTypeSymbol Symbol { get; }
        internal BaseTypeDeclarationSyntax Syntax { get; }
        internal SemanticModel Model { get; }
        internal List<(ISymbol Symbol, ITypeSymbol Type, bool Stored, bool Required)> Members { get; } = new();
        internal List<TypeRecord> Dependencies { get; } = new();
        internal PracticalDataType? Output { get; set; }
        internal string Kind { get; set; } = "";
        internal int Depth { get; set; }
        internal bool ExceptionCandidate { get; set; }
        internal bool GenericPending { get; set; }
    }

    private sealed class DataModel
    {
        private readonly bool hasSidecars;
        private readonly bool deferDeclaredInvariantProof;
        private readonly bool allowInitializerConstruction;
        private readonly List<TypeRecord> records = new();
        private readonly Dictionary<ISymbol, TypeRecord> bySymbol = new(SymbolEqualityComparer.Default);
        private readonly List<TypeRecord> ordered = new();
        private CSharpCompilation compilation = null!;
        private PracticalDataType? dayOfWeek;

        internal DataModel(bool hasSidecars, bool deferDeclaredInvariantProof, bool allowInitializerConstruction)
        {
            this.hasSidecars = hasSidecars; this.deferDeclaredInvariantProof = deferDeclaredInvariantProof;
            this.allowInitializerConstruction = allowInitializerConstruction;
        }

        internal void ValidateLimits(CSharpCompilation current)
        {
            // These two phase-0 counters precede dependency/compiler errors.
            // Partial/error symbols are tolerated here and rejected by their
            // existing declaration gate, not reclassified as adapter failures.
            var dependencies = new Dictionary<ISymbol, HashSet<ISymbol>>(SymbolEqualityComparer.Default);
            foreach (SyntaxTree tree in current.SyntaxTrees)
            {
                SemanticModel model = current.GetSemanticModel(tree);
                foreach (TypeDeclarationSyntax declaration in tree.GetRoot().DescendantNodes().OfType<TypeDeclarationSyntax>())
                {
                    int count = 0;
                    foreach (MemberDeclarationSyntax member in declaration.Members)
                    {
                        if (member is FieldDeclarationSyntax field && !field.Modifiers.Any(SyntaxKind.StaticKeyword)
                            && !field.Modifiers.Any(SyntaxKind.ConstKeyword))
                        { count = checked(count + field.Declaration.Variables.Count); }
                        else if (member is PropertyDeclarationSyntax property && !property.Modifiers.Any(SyntaxKind.StaticKeyword))
                        { count = checked(count + 1); }
                        if (count > FieldsPropertiesPerTypeMaximum)
                        { throw PracticalFailures.Limit("fields_properties_per_type"); }
                    }
                    if (model.GetDeclaredSymbol(declaration) is not INamedTypeSymbol type) { continue; }
                    if (!dependencies.TryGetValue(type, out HashSet<ISymbol>? targets))
                    {
                        targets = new HashSet<ISymbol>(SymbolEqualityComparer.Default);
                        dependencies.Add(type, targets);
                    }
                    foreach (ISymbol member in type.GetMembers())
                    {
                        ITypeSymbol? value = member switch
                        {
                            IFieldSymbol field when !field.IsStatic && !field.IsImplicitlyDeclared => field.Type,
                            IPropertySymbol property when !property.IsStatic => property.Type,
                            _ => null,
                        };
                        if (value is null) { continue; }
                        ITypeSymbol leaf = Unwrap(value);
                        if (!leaf.DeclaringSyntaxReferences.IsEmpty
                            && SymbolEqualityComparer.Default.Equals(leaf.ContainingAssembly, current.Assembly))
                        { targets.Add(leaf); }
                    }
                }
            }
            var parents = new Dictionary<ISymbol, List<ISymbol>>(SymbolEqualityComparer.Default);
            var remaining = new Dictionary<ISymbol, int>(SymbolEqualityComparer.Default);
            var depth = new Dictionary<ISymbol, int>(SymbolEqualityComparer.Default);
            var ready = new Queue<ISymbol>();
            // Source enums are leaf values even though they have no instance members.
            foreach (ISymbol leaf in dependencies.Values.SelectMany(values => values).ToArray())
            { dependencies.TryAdd(leaf, new HashSet<ISymbol>(SymbolEqualityComparer.Default)); }
            foreach (var pair in dependencies)
            {
                remaining.Add(pair.Key, pair.Value.Count);
                depth.Add(pair.Key, 0);
                if (pair.Value.Count == 0) { ready.Enqueue(pair.Key); }
                foreach (ISymbol target in pair.Value)
                {
                    if (!parents.TryGetValue(target, out List<ISymbol>? users))
                    { users = new List<ISymbol>(); parents.Add(target, users); }
                    users.Add(pair.Key);
                }
            }
            while (ready.Count != 0)
            {
                ISymbol value = ready.Dequeue();
                if (!parents.TryGetValue(value, out List<ISymbol>? users)) { continue; }
                foreach (ISymbol parent in users)
                {
                    depth[parent] = Math.Max(depth[parent], checked(depth[value] + 1));
                    if (depth[parent] > StructuralTypeNestingMaximum)
                    { throw PracticalFailures.Limit("structural_type_nesting"); }
                    if (--remaining[parent] == 0) { ready.Enqueue(parent); }
                }
            }
            // Cyclic remainders belong to the phase-2 type-cycle diagnostic.
        }

        internal void ValidateDeclarations(CSharpCompilation current)
        {
            compilation = current;
            foreach (SyntaxTree tree in compilation.SyntaxTrees)
            {
                SemanticModel model = compilation.GetSemanticModel(tree);
                foreach (BaseTypeDeclarationSyntax syntax in tree.GetRoot().DescendantNodes().OfType<BaseTypeDeclarationSyntax>())
                {
                    var symbol = model.GetDeclaredSymbol(syntax) as INamedTypeSymbol
                        ?? throw PracticalFailures.Declaration("data_symbol");
                    var record = new TypeRecord(symbol, syntax, model);
                    records.Add(record);
                    bySymbol.Add(symbol, record);
                    record.ExceptionCandidate = ClassifySourceException(symbol) is not null;
                }
            }
            foreach (TypeRecord record in records.Where(record => !record.ExceptionCandidate))
            { ValidateDeclaration(record); }
        }

        private void ValidateDeclaration(TypeRecord record)
        {
            INamedTypeSymbol symbol = record.Symbol;
            BaseTypeDeclarationSyntax syntax = record.Syntax;
            if (symbol.IsGenericType) { return; } // the frozen generic gate owns this diagnostic
            if (symbol.ContainingType is not null || symbol.IsRefLikeType || symbol.IsRecord
                || symbol.Interfaces.Length != 0 || symbol.DeclaringSyntaxReferences.Length != 1)
            { throw PracticalFailures.Declaration("data_layout"); }
            Modifiers(syntax.Modifiers, SyntaxKind.PublicKeyword, SyntaxKind.InternalKeyword,
                SyntaxKind.ReadOnlyKeyword, SyntaxKind.SealedKeyword, SyntaxKind.StaticKeyword);
            if (syntax is EnumDeclarationSyntax)
            {
                if (symbol.TypeKind != TypeKind.Enum
                    || !SymbolEqualityComparer.Default.Equals(symbol.BaseType,
                        compilation.GetSpecialType(SpecialType.System_Enum)))
                { throw PracticalFailures.Declaration("enum_base"); }
                record.Kind = "enum";
                return;
            }
            if (syntax.BaseList is not null)
            { throw PracticalFailures.Declaration("data_base_list"); }
            if (syntax is StructDeclarationSyntax && symbol.TypeKind == TypeKind.Struct
                && symbol.IsReadOnly && !symbol.IsStatic
                && SymbolEqualityComparer.Default.Equals(symbol.BaseType,
                    compilation.GetSpecialType(SpecialType.System_ValueType)))
            { record.Kind = "readonly_struct"; }
            else if (syntax is ClassDeclarationSyntax && symbol.TypeKind == TypeKind.Class
                && (symbol.IsSealed || symbol.IsStatic)
                && SymbolEqualityComparer.Default.Equals(symbol.BaseType,
                    compilation.GetSpecialType(SpecialType.System_Object)))
            { record.Kind = symbol.IsStatic ? "static_container" : "sealed_class"; }
            else { throw PracticalFailures.Declaration("data_type_kind"); }

            foreach (MemberDeclarationSyntax member in ((TypeDeclarationSyntax)syntax).Members)
            {
                switch (member)
                {
                    case FieldDeclarationSyntax field:
                        Modifiers(field.Modifiers, SyntaxKind.PublicKeyword, SyntaxKind.InternalKeyword,
                            SyntaxKind.PrivateKeyword, SyntaxKind.ReadOnlyKeyword);
                        foreach (VariableDeclaratorSyntax variable in field.Declaration.Variables)
                        {
                            var value = record.Model.GetDeclaredSymbol(variable) as IFieldSymbol
                                ?? throw PracticalFailures.Declaration("data_field");
                            if (!value.IsReadOnly || value.IsStatic || value.IsConst || value.IsVolatile
                                || value.IsFixedSizeBuffer || value.RefKind != RefKind.None
                                || value.IsRequired || variable.Initializer is not null)
                            { throw PracticalFailures.Declaration("data_field"); }
                            record.Members.Add((value, value.Type, true, false));
                        }
                        break;
                    case PropertyDeclarationSyntax property:
                        ValidateProperty(record, property);
                        break;
                    case MethodDeclarationSyntax method:
                        Modifiers(method.Modifiers, SyntaxKind.PublicKeyword, SyntaxKind.InternalKeyword,
                            SyntaxKind.PrivateKeyword, SyntaxKind.StaticKeyword, SyntaxKind.ReadOnlyKeyword);
                        var callable = record.Model.GetDeclaredSymbol(method)
                            ?? throw PracticalFailures.Declaration("data_method");
                        if (callable.IsVirtual || callable.IsOverride || callable.IsAbstract || callable.IsExtern
                            || callable.ReturnsByRef || callable.ReturnsByRefReadonly
                            || !callable.ExplicitInterfaceImplementations.IsEmpty)
                        { throw PracticalFailures.Declaration("data_method"); }
                        break;
                    case ConstructorDeclarationSyntax constructor:
                        Modifiers(constructor.Modifiers, SyntaxKind.PublicKeyword,
                            SyntaxKind.InternalKeyword, SyntaxKind.PrivateKeyword);
                        if (constructor.Initializer?.IsKind(SyntaxKind.BaseConstructorInitializer) == true)
                        { throw PracticalFailures.Declaration("data_base_constructor"); }
                        break;
                    default: throw PracticalFailures.Declaration("data_member");
                }
            }
            // No compiler-owned storage except one exact auto-property backing field.
            foreach (IFieldSymbol field in symbol.GetMembers().OfType<IFieldSymbol>().Where(field => field.IsImplicitlyDeclared))
            {
                if (field.AssociatedSymbol is not IPropertySymbol property
                    || field.IsStatic || field.RefKind != RefKind.None
                    || field.DeclaredAccessibility != Accessibility.Private
                    || !SymbolEqualityComparer.IncludeNullability.Equals(field.Type, property.Type)
                    || !record.Members.Any(member => SymbolEqualityComparer.Default.Equals(member.Symbol, property)
                        && member.Stored))
                { throw PracticalFailures.Declaration("hidden_storage"); }
            }
        }

        private static void ValidateProperty(TypeRecord record, PropertyDeclarationSyntax syntax)
        {
            Modifiers(syntax.Modifiers, SyntaxKind.PublicKeyword, SyntaxKind.InternalKeyword,
                SyntaxKind.PrivateKeyword, SyntaxKind.RequiredKeyword, SyntaxKind.ReadOnlyKeyword);
            var property = record.Model.GetDeclaredSymbol(syntax) as IPropertySymbol
                ?? throw PracticalFailures.Declaration("data_property");
            if (property.IsStatic || property.IsIndexer || property.IsVirtual || property.IsOverride
                || property.IsAbstract || property.ReturnsByRef || property.ReturnsByRefReadonly
                || property.GetMethod is null || syntax.Initializer is not null
                || !property.ExplicitInterfaceImplementations.IsEmpty)
            { throw PracticalFailures.Declaration("data_property"); }
            bool auto = syntax.ExpressionBody is null && syntax.AccessorList is not null
                && syntax.AccessorList.Accessors.All(accessor => accessor.Body is null
                    && accessor.ExpressionBody is null && !accessor.SemicolonToken.IsMissing);
            bool init = property.SetMethod is not null;
            if (init && (!auto || !property.SetMethod!.IsInitOnly)
                || property.IsRequired && (!init || !auto))
            { throw PracticalFailures.Declaration("data_property"); }
            if (syntax.AccessorList is not null)
            {
                foreach (AccessorDeclarationSyntax accessor in syntax.AccessorList.Accessors)
                {
                    Modifiers(accessor.Modifiers, SyntaxKind.PublicKeyword,
                        SyntaxKind.InternalKeyword, SyntaxKind.PrivateKeyword);
                    if (!accessor.IsKind(SyntaxKind.GetAccessorDeclaration)
                        && !accessor.IsKind(SyntaxKind.InitAccessorDeclaration))
                    { throw PracticalFailures.Declaration("data_property"); }
                }
            }
            record.Members.Add((property, property.Type, auto, property.IsRequired));
        }

        internal void ValidateTypes(CSharpCompilation current)
        {
            if (!ReferenceEquals(current, compilation)) { throw PracticalFailures.Protocol("data_compilation"); }
            foreach (TypeRecord record in records.Where(record => !record.ExceptionCandidate && !record.Symbol.IsGenericType))
            {
                foreach (var member in record.Members)
                {
                    try { PracticalExactTypeNormalizer.Normalize(member.Type, compilation); }
                    catch (PracticalCaptureFailure failure) when (failure.Family == PracticalDiagnosticFamily.CSHARP_PRACTICAL_GENERIC)
                    {
                        // Do not let phase 3 mask an independent W03 type failure.
                        // W01's complete generic pass supplies the eventual failure.
                        record.GenericPending = true;
                        continue;
                    }
                    ITypeSymbol leaf = Unwrap(member.Type);
                    if (bySymbol.TryGetValue(leaf, out TypeRecord? dependency))
                    {
                        if (dependency.ExceptionCandidate || dependency.Kind == "static_container")
                        { throw PracticalFailures.Type("data_member_type"); }
                        if (!record.Dependencies.Contains(dependency)) { record.Dependencies.Add(dependency); }
                    }
                }
            }
            DependencyOrder();
            foreach (TypeRecord record in ordered)
            {
                if (record.GenericPending || record.Dependencies.Any(dependency => dependency.GenericPending))
                { record.GenericPending = true; continue; }
                if (record.Kind == "enum") { record.Output = EnumType(record.Symbol); continue; }
                if (record.Kind == "static_container") { continue; }
                var members = record.Members.Select(member => new PracticalDataMember(member.Symbol.Name,
                    member.Symbol is IFieldSymbol ? "field" : "property",
                    PracticalExactTypeNormalizer.Normalize(member.Type, compilation), member.Required, member.Stored)).ToArray();
                var defaults = new List<PracticalDefaultValue>();
                bool eligible = record.Kind == "readonly_struct";
                foreach (var member in record.Members)
                {
                    if (member.Required) { eligible = false; }
                    if (!member.Stored) { continue; }
                    PracticalDefaultValue? value = Default(member.Type);
                    if (value is null) { eligible = false; }
                    else { defaults.Add(value); }
                }
                PracticalDefaultValue? zero = eligible
                    ? new PracticalDefaultValue(SourceId(record.Symbol), "product", "", defaults.ToArray()) : null;
                record.Output = new PracticalDataType(SourceId(record.Symbol), record.Kind, "",
                    members, Array.Empty<PracticalEnumMember>(), record.Depth, zero,
                    eligible && hasSidecars);
            }
            ValidateOperations();
        }

        private static ITypeSymbol Unwrap(ITypeSymbol type)
        {
            while (true)
            {
                if (type is IArrayTypeSymbol array) { type = array.ElementType; }
                else if (type is INamedTypeSymbol named && named.OriginalDefinition.SpecialType == SpecialType.System_Nullable_T)
                { type = named.TypeArguments[0]; }
                else { return type; }
            }
        }

        private void DependencyOrder()
        {
            var colors = new Dictionary<TypeRecord, int>();
            foreach (TypeRecord root in records.Where(record => !record.ExceptionCandidate && !record.Symbol.IsGenericType))
            {
                var pending = new Stack<(TypeRecord Record, bool Exit)>();
                pending.Push((root, false));
                while (pending.Count != 0)
                {
                    var (record, exit) = pending.Pop();
                    if (exit)
                    {
                        record.Depth = record.Dependencies.Count == 0 ? 0
                            : record.Dependencies.Max(dependency => checked(dependency.Depth + 1));
                        if (record.Depth > StructuralTypeNestingMaximum)
                        { throw PracticalFailures.Limit("structural_type_nesting"); }
                        colors[record] = 2;
                        ordered.Add(record);
                        continue;
                    }
                    if (colors.TryGetValue(record, out int color))
                    {
                        if (color == 1) { throw PracticalFailures.Type("type_cycle"); }
                        continue;
                    }
                    colors[record] = 1;
                    pending.Push((record, true));
                    for (int index = record.Dependencies.Count - 1; index >= 0; index--)
                    { pending.Push((record.Dependencies[index], false)); }
                }
            }
        }

        private PracticalDataType EnumType(INamedTypeSymbol symbol)
        {
            if (symbol.EnumUnderlyingType is not INamedTypeSymbol underlying || !IntegerCarrier(underlying.SpecialType))
            { throw PracticalFailures.Type("enum_carrier"); }
            var members = new List<PracticalEnumMember>();
            foreach (IFieldSymbol field in symbol.GetMembers().OfType<IFieldSymbol>().Where(field => !field.IsImplicitlyDeclared))
            {
                if (!field.IsConst || !field.IsStatic || !field.HasConstantValue || field.ConstantValue is null
                    || !SymbolEqualityComparer.Default.Equals(field.Type, symbol)
                    || !ExactBoxedInteger(field.ConstantValue, underlying.SpecialType))
                { throw PracticalFailures.Type("enum_member"); }
                members.Add(new PracticalEnumMember(field.Name,
                    Convert.ToString(field.ConstantValue, CultureInfo.InvariantCulture)!));
            }
            bool framework = ExactRuntime(symbol, "DayOfWeek");
            if (framework)
            {
                string[] names = { "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday" };
                if (underlying.SpecialType != SpecialType.System_Int32 || members.Count != names.Length
                    || members.Where((member, index) => member.Name != names[index]
                        || member.Value != index.ToString(CultureInfo.InvariantCulture)).Any())
                { throw PracticalFailures.Type("day_of_week_shape"); }
            }
            string id = framework ? PracticalIdentity.PrimitiveId("day_of_week") : SourceId(symbol);
            PracticalDefaultValue? zero = members.Any(member => member.Value == "0")
                ? new PracticalDefaultValue(id, "enum", "0", Array.Empty<PracticalDefaultValue>()) : null;
            return new PracticalDataType(id, "enum", PracticalExactTypeNormalizer.Normalize(underlying, compilation).Id,
                Array.Empty<PracticalDataMember>(), members.ToArray(), 0, zero,
                zero is not null && !framework && hasSidecars);
        }

        private static bool ExactBoxedInteger(object value, SpecialType carrier) => carrier switch
        {
            SpecialType.System_SByte => value is sbyte,
            SpecialType.System_Byte => value is byte,
            SpecialType.System_Int16 => value is short,
            SpecialType.System_UInt16 => value is ushort,
            SpecialType.System_Int32 => value is int,
            SpecialType.System_UInt32 => value is uint,
            SpecialType.System_Int64 => value is long,
            SpecialType.System_UInt64 => value is ulong,
            _ => false,
        };

        private PracticalDefaultValue? Default(ITypeSymbol type)
        {
            PracticalNormalizedType normalized = PracticalExactTypeNormalizer.Normalize(type, compilation);
            if (type.IsReferenceType)
            {
                return type.NullableAnnotation == NullableAnnotation.Annotated
                    ? new PracticalDefaultValue(normalized.Id, "none", "", Array.Empty<PracticalDefaultValue>()) : null;
            }
            if (type is INamedTypeSymbol named && named.OriginalDefinition.SpecialType == SpecialType.System_Nullable_T)
            { return new PracticalDefaultValue(normalized.Id, "none", "", Array.Empty<PracticalDefaultValue>()); }
            if (bySymbol.TryGetValue(type, out TypeRecord? record)) { return record.Output?.DefaultValue; }
            if (ExactRuntime(type, "DayOfWeek"))
            {
                dayOfWeek ??= EnumType((INamedTypeSymbol)type);
                return dayOfWeek.DefaultValue;
            }
            string scalar = type.SpecialType switch
            {
                SpecialType.System_Boolean => "false",
                SpecialType.System_Single => "00000000",
                SpecialType.System_Double => "0000000000000000",
                _ when ExactRuntime(type, "Guid") => "00000000-0000-0000-0000-000000000000",
                _ when ExactRuntime(type, "DateOnly") => "0001-01-01",
                _ when ExactRuntime(type, "TimeOnly") => "00:00:00.0000000",
                _ => "0",
            };
            return new PracticalDefaultValue(normalized.Id, "scalar", scalar, Array.Empty<PracticalDefaultValue>());
        }

        private static bool Enum(ITypeSymbol? type) => type?.TypeKind == TypeKind.Enum;
        private static bool ClassIdentity(ITypeSymbol? type) => type?.IsReferenceType == true
            && type.SpecialType != SpecialType.System_String;

        private void ValidateOperations()
        {
            foreach (SyntaxTree tree in compilation.SyntaxTrees)
            {
                SemanticModel model = compilation.GetSemanticModel(tree);
                HashSet<ILocalSymbol> publishedAliases = PublishedArrayAliases(tree.GetRoot(), model);
                Dictionary<ILocalSymbol, int> publications = ConstructionPublications(tree.GetRoot(), model);
                foreach (SyntaxNode node in tree.GetRoot().DescendantNodes())
                {
                    // Enum declaration constant expressions define the closed carrier table;
                    // numeric computation there is not runtime enum arithmetic.
                    if (node.AncestorsAndSelf().OfType<EnumMemberDeclarationSyntax>().Any()) { continue; }
                    if (node is WithExpressionSyntax || node is RefExpressionSyntax
                        || node is RefTypeSyntax || node is MakeRefExpressionSyntax
                        || node is RefTypeExpressionSyntax || node is RefValueExpressionSyntax
                        || node is ArgumentSyntax argument && argument.RefKindKeyword.RawKind != 0
                        || node is ParameterSyntax parameter && parameter.Modifiers.Any(token =>
                            token.IsKind(SyntaxKind.RefKeyword) || token.IsKind(SyntaxKind.InKeyword)
                            || token.IsKind(SyntaxKind.OutKeyword)))
                    { throw PracticalFailures.Type("identity_or_reference_escape"); }
                    if (node.IsKind(SyntaxKind.DefaultLiteralExpression))
                    { throw PracticalFailures.Type("target_typed_default"); }
                    if (node is TypeSyntax typeSyntax && model.GetTypeInfo(typeSyntax).Type is ITypeSymbol type
                        && ExactRuntime(type, "DayOfWeek"))
                    { dayOfWeek ??= EnumType((INamedTypeSymbol)type); }
                    if (node is DefaultExpressionSyntax exactDefault)
                    {
                        ITypeSymbol? defaultType = model.GetTypeInfo(exactDefault.Type).Type;
                        if (defaultType?.IsReferenceType == true)
                        {
                            defaultType = defaultType.WithNullableAnnotation(
                                exactDefault.Type is NullableTypeSyntax
                                    ? NullableAnnotation.Annotated : NullableAnnotation.NotAnnotated);
                        }
                        RequireDefault(defaultType);
                    }
                    if (node is ObjectCreationExpressionSyntax creation
                        && model.GetOperation(creation) is IObjectCreationOperation operation
                        && operation.Type is INamedTypeSymbol created && created.IsValueType
                        && operation.Arguments.Length == 0
                        && (operation.Constructor is null || operation.Constructor.IsImplicitlyDeclared))
                    {
                        // W05's fresh initialized struct starts in temporary CLR
                        // zero storage; it is not publication of default(T).
                        if (!(allowInitializerConstruction && operation.Initializer is not null
                            && created.TypeKind == TypeKind.Struct && !created.DeclaringSyntaxReferences.IsEmpty))
                        { RequireDefault(created); }
                    }
                    if (node is not ExpressionSyntax expression) { continue; }
                    TypeInfo information = model.GetTypeInfo(expression);
                    if (information.Type is not null && information.ConvertedType is not null
                        && (Enum(UnwrapOptional(information.Type)) || Enum(UnwrapOptional(information.ConvertedType)))
                        && !SameEnumOrNullableLift(information.Type, information.ConvertedType))
                    { throw PracticalFailures.Type("enum_conversion"); }
                    IOperation? value = model.GetOperation(expression);
                    switch (value)
                    {
                        case IConversionOperation conversion when
                            (Enum(UnwrapOptional(conversion.Type)) || Enum(UnwrapOptional(conversion.Operand.Type)))
                            && !SameEnumOrNullableLift(conversion.Operand.Type, conversion.Type):
                            throw PracticalFailures.Type("enum_conversion");
                        case IBinaryOperation binary when Enum(UnwrapOptional(binary.LeftOperand.Type))
                            || Enum(UnwrapOptional(binary.RightOperand.Type)):
                            if (binary.OperatorKind is not BinaryOperatorKind.Equals and not BinaryOperatorKind.NotEquals)
                            { throw PracticalFailures.Type("enum_arithmetic"); }
                            break;
                        case IUnaryOperation unary when Enum(UnwrapOptional(unary.Operand.Type)):
                            throw PracticalFailures.Type("enum_arithmetic");
                        case IIncrementOrDecrementOperation increment when Enum(UnwrapOptional(increment.Target.Type)):
                            throw PracticalFailures.Type("enum_arithmetic");
                        case ICompoundAssignmentOperation compound when Enum(UnwrapOptional(compound.Target.Type)):
                            throw PracticalFailures.Type("enum_arithmetic");
                        case IBinaryOperation binary when binary.OperatorKind is BinaryOperatorKind.Equals or BinaryOperatorKind.NotEquals
                            && (ClassIdentity(binary.LeftOperand.Type) || ClassIdentity(binary.RightOperand.Type)):
                            throw PracticalFailures.Type("class_identity");
                        case ISimpleAssignmentOperation assignment:
                            ValidateWrite(assignment.Target, expression, model, publishedAliases, publications);
                            break;
                        case ICompoundAssignmentOperation assignment:
                            ValidateWrite(assignment.Target, expression, model, publishedAliases, publications);
                            break;
                        case IIncrementOrDecrementOperation increment:
                            ValidateWrite(increment.Target, expression, model, publishedAliases, publications);
                            break;
                    }
                }
            }
        }

        private static ITypeSymbol? UnwrapOptional(ITypeSymbol? type) => type is INamedTypeSymbol named
            && named.OriginalDefinition.SpecialType == SpecialType.System_Nullable_T ? named.TypeArguments[0] : type;

        private static bool SameEnumOrNullableLift(ITypeSymbol? from, ITypeSymbol? to) =>
            SymbolEqualityComparer.Default.Equals(from, to)
            || from is not null && from.TypeKind == TypeKind.Enum
                && SymbolEqualityComparer.Default.Equals(from, UnwrapOptional(to));

        private void RequireDefault(ITypeSymbol? type)
        {
            if (type is not null && bySymbol.TryGetValue(type, out TypeRecord? record)
                && (record.GenericPending || record.Symbol.IsGenericType)) { return; }
            PracticalDefaultValue? value = null;
            try { if (type is not null) { value = Default(type); } }
            catch (PracticalCaptureFailure failure) when (failure.Family == PracticalDiagnosticFamily.CSHARP_PRACTICAL_GENERIC)
            { return; } // the phase-3 capture gate will reject before building a result
            if (type is null || value is null)
            { throw PracticalFailures.Type("ineligible_default"); }
            // Opaque sidecars cannot prove an invariant or semantic-role default.
            // W04/W13 must resolve those claims before any such value is published.
            if (hasSidecars && !deferDeclaredInvariantProof && type.IsValueType && bySymbol.ContainsKey(type))
            { throw PracticalFailures.Type("default_invariant_pending"); }
        }

        // A local may name receiver/parameter storage even after one or more
        // reassignments. Build a conservative, flow-independent alias graph;
        // only fresh construction storage can escape this publication barrier.
        private static HashSet<ILocalSymbol> PublishedArrayAliases(SyntaxNode root, SemanticModel model)
        {
            var published = new HashSet<ILocalSymbol>(SymbolEqualityComparer.Default);
            var users = new Dictionary<ILocalSymbol, HashSet<ILocalSymbol>>(SymbolEqualityComparer.Default);
            var pending = new Queue<ILocalSymbol>();
            foreach (SyntaxNode node in root.DescendantNodes())
            {
                var targets = new HashSet<ILocalSymbol>(SymbolEqualityComparer.Default);
                IOperation? value = null;
                if (node is VariableDeclaratorSyntax variable && variable.Initializer is not null)
                {
                    if (model.GetDeclaredSymbol(variable) is ILocalSymbol local) { targets.Add(local); }
                    value = model.GetOperation(variable.Initializer.Value);
                }
                else if (node is AssignmentExpressionSyntax assignment
                    && model.GetOperation(assignment) is ISimpleAssignmentOperation operation)
                {
                    if (operation.Target is ILocalReferenceOperation local) { targets.Add(local.Local); }
                    else if (operation.Target is IArrayElementReferenceOperation element)
                    { ArrayDependencies(element.ArrayReference, targets); }
                    value = operation.Value;
                }
                if (value is null) { continue; }
                var dependencies = new HashSet<ILocalSymbol>(SymbolEqualityComparer.Default);
                bool isPublished = ArrayDependencies(value, dependencies);
                foreach (ILocalSymbol target in targets.Where(target => target.Type is IArrayTypeSymbol))
                {
                    if (isPublished && published.Add(target)) { pending.Enqueue(target); }
                    foreach (ILocalSymbol dependency in dependencies)
                    {
                        if (!users.TryGetValue(dependency, out HashSet<ILocalSymbol>? dependents))
                        {
                            dependents = new HashSet<ILocalSymbol>(SymbolEqualityComparer.Default);
                            users.Add(dependency, dependents);
                        }
                        dependents.Add(target);
                    }
                }
            }
            while (pending.Count != 0)
            {
                ILocalSymbol local = pending.Dequeue();
                if (!users.TryGetValue(local, out HashSet<ILocalSymbol>? dependents)) { continue; }
                foreach (ILocalSymbol dependent in dependents)
                { if (published.Add(dependent)) { pending.Enqueue(dependent); } }
            }
            return published;
        }

        private static bool ArrayDependencies(IOperation root, HashSet<ILocalSymbol> dependencies)
        {
            bool published = false;
            var pending = new Stack<IOperation>(); pending.Push(root);
            while (pending.Count != 0)
            {
                IOperation operation = pending.Pop();
                if (operation is IArrayCreationOperation creation)
                {
                    // Dimension expressions cannot alias the newly allocated array.
                    if (creation.Initializer is not null) { pending.Push(creation.Initializer); }
                    continue;
                }
                // Reading a scalar from an array copies a value; that array is
                // not an alias of the enclosing initializer or conditional arm.
                if (operation.Type is not IArrayTypeSymbol && operation is not IArrayInitializerOperation)
                { continue; }
                if (operation.Type is IArrayTypeSymbol)
                {
                    if (operation is IFieldReferenceOperation or IPropertyReferenceOperation
                        or IParameterReferenceOperation or IInvocationOperation)
                    { published = true; }
                    else if (operation is ILocalReferenceOperation local) { dependencies.Add(local.Local); }
                }
                foreach (IOperation child in operation.ChildOperations) { pending.Push(child); }
            }
            return published;
        }

        // Construction can publish a previously fresh array through a structural
        // value. Track that point across aliases so later writes cannot change
        // the already constructed value. Earlier buffer writes remain W07-owned.
        private static Dictionary<ILocalSymbol, int> ConstructionPublications(SyntaxNode root, SemanticModel model)
        {
            var edges = new Dictionary<ILocalSymbol, HashSet<ILocalSymbol>>(SymbolEqualityComparer.Default);
            var firstPublication = new Dictionary<ILocalSymbol, int>(SymbolEqualityComparer.Default);
            foreach (SyntaxNode node in root.DescendantNodes())
            {
                var locals = new HashSet<ILocalSymbol>(SymbolEqualityComparer.Default);
                IOperation? value = null;
                if (node is VariableDeclaratorSyntax variable && variable.Initializer is not null)
                {
                    if (model.GetDeclaredSymbol(variable) is ILocalSymbol local) { locals.Add(local); }
                    value = model.GetOperation(variable.Initializer.Value);
                }
                else if (node is AssignmentExpressionSyntax assignment
                    && model.GetOperation(assignment) is ISimpleAssignmentOperation assigned)
                {
                    value = assigned.Value;
                    if (assigned.Target is ILocalReferenceOperation reference) { locals.Add(reference.Local); }
                    else if (assigned.Target is IArrayElementReferenceOperation element)
                    { ArrayDependencies(element.ArrayReference, locals); }
                    else if (assigned.Target is IFieldReferenceOperation or IPropertyReferenceOperation)
                    { Publish(value, node); }
                }
                if (value is not null)
                {
                    var dependencies = new HashSet<ILocalSymbol>(SymbolEqualityComparer.Default);
                    ArrayDependencies(value, dependencies);
                    foreach (ILocalSymbol local in locals.Where(local => local.Type is IArrayTypeSymbol))
                    {
                        foreach (ILocalSymbol dependency in dependencies)
                        { Connect(local, dependency); Connect(dependency, local); }
                    }
                }
                if (node is BaseObjectCreationExpressionSyntax && model.GetOperation(node) is IObjectCreationOperation creation)
                { foreach (IArgumentOperation argument in creation.Arguments) { Publish(argument.Value, node); } }
                if (node is InvocationExpressionSyntax && model.GetOperation(node) is IInvocationOperation invocation
                    && !invocation.TargetMethod.ReturnsVoid
                    && (invocation.Type is IArrayTypeSymbol
                        || invocation.Type is INamedTypeSymbol result && !result.DeclaringSyntaxReferences.IsEmpty))
                { foreach (IArgumentOperation argument in invocation.Arguments) { Publish(argument.Value, node); } }
            }
            var resultTimes = new Dictionary<ILocalSymbol, int>(SymbolEqualityComparer.Default);
            foreach (ILocalSymbol rootLocal in firstPublication.Keys)
            {
                if (resultTimes.ContainsKey(rootLocal)) { continue; }
                var component = new HashSet<ILocalSymbol>(SymbolEqualityComparer.Default);
                var pending = new Stack<ILocalSymbol>(); pending.Push(rootLocal);
                int earliest = int.MaxValue;
                while (pending.Count != 0)
                {
                    ILocalSymbol local = pending.Pop();
                    if (!component.Add(local)) { continue; }
                    if (firstPublication.TryGetValue(local, out int start)) { earliest = Math.Min(earliest, start); }
                    if (edges.TryGetValue(local, out HashSet<ILocalSymbol>? aliases))
                    { foreach (ILocalSymbol alias in aliases) { pending.Push(alias); } }
                }
                foreach (ILocalSymbol local in component) { resultTimes.Add(local, earliest); }
            }
            return resultTimes;

            void Connect(ILocalSymbol source, ILocalSymbol target)
            {
                if (!edges.TryGetValue(source, out HashSet<ILocalSymbol>? targets))
                { targets = new HashSet<ILocalSymbol>(SymbolEqualityComparer.Default); edges.Add(source, targets); }
                targets.Add(target);
            }
            void Publish(IOperation value, SyntaxNode syntax)
            {
                if (value.Type is not IArrayTypeSymbol) { return; }
                var dependencies = new HashSet<ILocalSymbol>(SymbolEqualityComparer.Default);
                ArrayDependencies(value, dependencies);
                int start = syntax.Span.End;
                // A later iteration can write before this syntactic publication.
                foreach (SyntaxNode ancestor in syntax.Ancestors())
                {
                    if (ancestor is ForStatementSyntax or CommonForEachStatementSyntax or WhileStatementSyntax or DoStatementSyntax)
                    { start = Math.Min(start, ancestor.SpanStart); }
                }
                foreach (ILocalSymbol dependency in dependencies)
                {
                    if (!firstPublication.TryGetValue(dependency, out int previous) || start < previous)
                    { firstPublication[dependency] = start; }
                }
            }
        }

        private static void ValidateWrite(IOperation target, ExpressionSyntax expression, SemanticModel model,
            HashSet<ILocalSymbol> publishedAliases, Dictionary<ILocalSymbol, int> publications)
        {
            if (target is IArrayElementReferenceOperation element)
            {
                var dependencies = new HashSet<ILocalSymbol>(SymbolEqualityComparer.Default);
                if (ArrayDependencies(element.ArrayReference, dependencies) || dependencies.Overlaps(publishedAliases))
                { throw PracticalFailures.Type("reachable_mutation"); }
                if (dependencies.Any(local => publications.TryGetValue(local, out int start) && start <= expression.SpanStart))
                { throw PracticalFailures.Type("published_construction_mutation"); }
            }
            ISymbol? member = target switch
            {
                IFieldReferenceOperation field => field.Field,
                IPropertyReferenceOperation property => property.Property,
                _ => null,
            };
            if (member is null)
            {
                if (target is IInstanceReferenceOperation) { throw PracticalFailures.Type("whole_this_assignment"); }
                return; // local/array construction state is owned by later tasks
            }
            IOperation? receiver = target switch
            {
                IFieldReferenceOperation field => field.Instance,
                IPropertyReferenceOperation property => property.Instance,
                _ => null,
            };
            ConstructorDeclarationSyntax? constructor = expression.Ancestors().OfType<ConstructorDeclarationSyntax>().FirstOrDefault();
            if (constructor is not null && receiver is IInstanceReferenceOperation
                && SymbolEqualityComparer.Default.Equals(model.GetDeclaredSymbol(constructor)?.ContainingType, member.ContainingType))
            { return; } // assignment order/definite state belongs to W04
            if (member is IPropertySymbol { SetMethod.IsInitOnly: true }
                && expression.Parent is InitializerExpressionSyntax { RawKind: (int)SyntaxKind.ObjectInitializerExpression })
            { return; } // unique initializer transaction belongs to W05
            throw PracticalFailures.Type("member_mutation");
        }

        internal IReadOnlyList<PracticalDataType> GetTypes() => Array.AsReadOnly(
            ordered.Where(record => record.Output is not null).Select(record => record.Output!)
                .Concat(dayOfWeek is null ? Array.Empty<PracticalDataType>() : new[] { dayOfWeek }).ToArray());

        internal PracticalDataTypes Build(PracticalNormalizedSyntax syntax)
        {
            if (records.Any(record => record.ExceptionCandidate))
            { throw PracticalFailures.Protocol("exception_handoff_required"); }
            var types = ordered.Where(record => record.Output is not null).Select(record => record.Output!).ToList();
            if (dayOfWeek is not null) { types.Insert(0, dayOfWeek); }
            using var output = new MemoryStream();
            using (var writer = new Utf8JsonWriter(output, new JsonWriterOptions
                { Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping }))
            {
                writer.WriteStartObject();
                writer.WriteString("schema", "mpk.csharp_practical.data_types.v1");
                writer.WriteStartArray("types");
                foreach (PracticalDataType type in types)
                {
                    writer.WriteStartObject();
                    writer.WriteString("id", type.Id);
                    writer.WriteString("kind", type.Kind);
                    writer.WriteString("carrier", type.Carrier);
                    writer.WriteNumber("structural_depth", type.StructuralDepth);
                    writer.WriteBoolean("default_eligible", type.DefaultEligible);
                    writer.WriteBoolean("default_invariant_pending", type.DefaultInvariantPending);
                    writer.WriteStartArray("enum_members");
                    foreach (PracticalEnumMember member in type.EnumMembers)
                    {
                        writer.WriteStartArray(); writer.WriteStringValue(member.Name);
                        writer.WriteStringValue(member.Value); writer.WriteEndArray();
                    }
                    writer.WriteEndArray();
                    writer.WriteStartArray("members");
                    foreach (PracticalDataMember member in type.Members)
                    {
                        writer.WriteStartObject();
                        writer.WriteString("name", member.Name);
                        writer.WriteString("kind", member.Kind);
                        writer.WriteString("type", member.Type.CanonicalKey);
                        writer.WriteBoolean("stored", member.Stored);
                        writer.WriteBoolean("required", member.Required);
                        writer.WriteEndObject();
                    }
                    writer.WriteEndArray(); writer.WriteEndObject();
                }
                writer.WriteEndArray(); writer.WriteEndObject();
            }
            return new PracticalDataTypes(syntax, types.ToArray(), output.ToArray());
        }
    }
}
