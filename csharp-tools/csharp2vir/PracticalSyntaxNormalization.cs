using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Collections.ObjectModel;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Security.Cryptography;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;
using System.Threading;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

// CSHARP-03-T03-W02 owns this private normalization handoff. It is deliberately
// absent from csharp2vir.csproj and every installed build-input manifest.

internal sealed class PracticalNormalizedType
{
    private readonly ReadOnlyCollection<PracticalNormalizedType> arguments;
    private string? canonicalKey;

    internal PracticalNormalizedType(
        string id,
        string nullability,
        PracticalNormalizedType[] arguments)
    {
        Id = id;
        Nullability = nullability;
        this.arguments = Array.AsReadOnly(
            (PracticalNormalizedType[])arguments.Clone());
    }

    internal string Id { get; }

    internal string Nullability { get; }

    internal IReadOnlyList<PracticalNormalizedType> Arguments => arguments;

    internal string CanonicalKey => canonicalKey ??= BuildCanonicalKey(this);

    private static string BuildCanonicalKey(PracticalNormalizedType root)
    {
        var result = new StringBuilder();
        var pending = new Stack<PracticalNormalizedType>();
        pending.Push(root);
        while (pending.Count != 0)
        {
            PracticalNormalizedType current = pending.Pop();
            AppendPart(result, current.Id);
            AppendPart(result, current.Nullability);
            result.Append(current.Arguments.Count.ToString(CultureInfo.InvariantCulture))
                .Append(':');
            for (int index = current.Arguments.Count - 1; index >= 0; index--)
            {
                pending.Push(current.Arguments[index]);
            }
        }

        return result.ToString();
    }

    private static void AppendPart(StringBuilder target, string value)
    {
        target.Append(value.Length.ToString(CultureInfo.InvariantCulture))
            .Append(':')
            .Append(value);
    }
}

internal sealed class PracticalExactTypeBinding
{
    internal PracticalExactTypeBinding(
        string callableId,
        int localOrdinal,
        PracticalNormalizedType type)
    {
        CallableId = callableId;
        LocalOrdinal = localOrdinal;
        Type = type;
    }

    internal string CallableId { get; }

    internal int LocalOrdinal { get; }

    internal PracticalNormalizedType Type { get; }
}

internal sealed class PracticalNormalizedCallable
{
    private readonly byte[] bodyBytes;

    internal PracticalNormalizedCallable(string id, byte[] bodyBytes)
    {
        Id = id;
        this.bodyBytes = (byte[])bodyBytes.Clone();
        BodySha256 = Convert.ToHexString(SHA256.HashData(this.bodyBytes)).ToLowerInvariant();
    }

    internal string Id { get; }

    internal string BodySha256 { get; }

    internal byte[] CopyBodyBytes() => (byte[])bodyBytes.Clone();
}

internal sealed class PracticalNormalizedSyntax
{
    private readonly ReadOnlyCollection<PracticalNormalizedCallable> callables;
    private readonly ReadOnlyCollection<PracticalExactTypeBinding> exactTypes;
    private readonly byte[] canonicalBytes;

    internal PracticalNormalizedSyntax(
        PracticalSourceClosure sourceClosure,
        PracticalNormalizedCallable[] callables,
        PracticalExactTypeBinding[] exactTypes,
        byte[] canonicalBytes)
    {
        SourceClosure = sourceClosure;
        this.callables = Array.AsReadOnly(
            (PracticalNormalizedCallable[])callables.Clone());
        this.exactTypes = Array.AsReadOnly(
            (PracticalExactTypeBinding[])exactTypes.Clone());
        this.canonicalBytes = (byte[])canonicalBytes.Clone();
        SemanticSha256 = Convert.ToHexString(SHA256.HashData(this.canonicalBytes)).ToLowerInvariant();
    }

    internal PracticalSourceClosure SourceClosure { get; }

    internal IReadOnlyList<PracticalNormalizedCallable> Callables => callables;

    internal IReadOnlyList<PracticalExactTypeBinding> ExactTypes => exactTypes;

    internal string SemanticSha256 { get; }

    internal byte[] CopyCanonicalBytes() => (byte[])canonicalBytes.Clone();
}

internal static class PracticalExactTypeNormalizer
{
    internal static PracticalNormalizedType Normalize(
        ITypeSymbol type,
        Compilation compilation,
        bool allowVoid = false,
        NullableAnnotation topLevelNullability = NullableAnnotation.None)
    {
        if (type is null)
        {
            throw PracticalFailures.Type("exact_type");
        }

        var wrapperIds = new List<string>();
        var wrapperNullabilities = new List<string>();
        ITypeSymbol current = type;
        NullableAnnotation currentTopLevelNullability = topLevelNullability;
        while (true)
        {
            ValidateLeafShape(current);
            if (current is IArrayTypeSymbol array)
            {
                if (!array.IsSZArray || array.Rank != 1)
                {
                    throw PracticalFailures.Type("exact_type");
                }

                wrapperIds.Add("bounded_sequence");
                wrapperNullabilities.Add(ReferenceNullability(
                    array,
                    currentTopLevelNullability));
                current = array.ElementType;
                currentTopLevelNullability = NullableAnnotation.None;
                continue;
            }

            if (current is INamedTypeSymbol nullable && IsExactNullable(nullable))
            {
                if (nullable.TypeArguments.Length != 1
                    || !nullable.TypeArguments[0].IsValueType)
                {
                    throw PracticalFailures.Generic("nullable_shape");
                }

                wrapperIds.Add("option");
                wrapperNullabilities.Add("value");
                current = nullable.TypeArguments[0];
                currentTopLevelNullability = NullableAnnotation.None;
                continue;
            }

            break;
        }

        if (current is INamedTypeSymbol named && named.IsGenericType)
        {
            throw PracticalFailures.Generic("constructed_type");
        }

        string? token = current.SpecialType switch
        {
            SpecialType.System_Void when allowVoid => "unit",
            SpecialType.System_Boolean => "bool",
            SpecialType.System_SByte => "i8",
            SpecialType.System_Byte => "u8",
            SpecialType.System_Int16 => "i16",
            SpecialType.System_UInt16 => "u16",
            SpecialType.System_Int32 => "i32",
            SpecialType.System_UInt32 => "u32",
            SpecialType.System_Int64 => "i64",
            SpecialType.System_UInt64 => "u64",
            SpecialType.System_Char => "char",
            SpecialType.System_Single => "f32",
            SpecialType.System_Double => "f64",
            SpecialType.System_Decimal => "decimal",
            SpecialType.System_String => "string",
            _ => FrameworkToken(current),
        };
        PracticalNormalizedType normalized;
        if (token is not null)
        {
            normalized = new PracticalNormalizedType(
                PracticalIdentity.PrimitiveId(token),
                current.IsReferenceType
                    ? ReferenceNullability(current, currentTopLevelNullability)
                    : "value",
                Array.Empty<PracticalNormalizedType>());
        }
        else if (SymbolEqualityComparer.Default.Equals(
                current.ContainingAssembly,
                compilation.Assembly)
            && current is INamedTypeSymbol source
            && !source.DeclaringSyntaxReferences.IsEmpty
            && source.ContainingType is null)
        {
            normalized = new PracticalNormalizedType(
                PracticalIdentity.SourceTypeId(
                    source.ContainingNamespace.ToDisplayString(),
                    source.Name),
                source.IsReferenceType
                    ? ReferenceNullability(source, currentTopLevelNullability)
                    : "value",
                Array.Empty<PracticalNormalizedType>());
        }
        else
        {
            throw PracticalFailures.Type("exact_type");
        }

        for (int index = wrapperIds.Count - 1; index >= 0; index--)
        {
            normalized = new PracticalNormalizedType(
                PracticalIdentity.ClosedInstanceId(wrapperIds[index], normalized.Id),
                wrapperNullabilities[index],
                new[] { normalized });
        }

        return normalized;
    }

    private static void ValidateLeafShape(ITypeSymbol type)
    {
        if (type.TypeKind == TypeKind.Dynamic)
        {
            throw PracticalFailures.Declaration("dynamic_inference");
        }

        if (type.TypeKind is TypeKind.Error or TypeKind.TypeParameter
            || type.IsAnonymousType)
        {
            throw PracticalFailures.Type("exact_type");
        }
    }

    private static string ReferenceNullability(
        ITypeSymbol type,
        NullableAnnotation topLevelNullability)
    {
        NullableAnnotation annotation = topLevelNullability == NullableAnnotation.None
            ? type.NullableAnnotation
            : topLevelNullability;
        return annotation switch
        {
            NullableAnnotation.Annotated => "annotated",
            NullableAnnotation.NotAnnotated => "not_annotated",
            _ => throw PracticalFailures.Type("nullable_inference"),
        };
    }

    private static bool IsExactNullable(INamedTypeSymbol type)
    {
        INamedTypeSymbol definition = type.OriginalDefinition;
        return definition.Arity == 1
            && definition.TypeKind == TypeKind.Struct
            && definition.TypeParameters.Length == 1
            && definition.TypeParameters[0].HasValueTypeConstraint
            && string.Equals(definition.MetadataName, "Nullable`1", StringComparison.Ordinal)
            && definition.DeclaringSyntaxReferences.IsEmpty
            && string.Equals(
                definition.ContainingNamespace.ToDisplayString(),
                "System",
                StringComparison.Ordinal)
            && string.Equals(
                definition.ContainingAssembly?.Identity.Name,
                "System.Runtime",
                StringComparison.Ordinal);
    }

    private static string? FrameworkToken(ITypeSymbol type)
    {
        if (type is not INamedTypeSymbol named
            || named.Arity != 0
            || !named.DeclaringSyntaxReferences.IsEmpty
            || !string.Equals(
                named.ContainingNamespace.ToDisplayString(),
                "System",
                StringComparison.Ordinal)
            || !string.Equals(
                named.ContainingAssembly?.Identity.Name,
                "System.Runtime",
                StringComparison.Ordinal))
        {
            return null;
        }

        return named.MetadataName switch
        {
            "DateOnly" => "date",
            "TimeOnly" => "time",
            "TimeSpan" => "duration",
            "Guid" => "guid",
            "DayOfWeek" => "day_of_week",
            _ => null,
        };
    }
}

internal static class CSharpPracticalSyntaxNormalizer
{
    private const long ReferenceTotalBytes = 6_046_008;
    private const int ReferenceCanonicalBytes = 24_670;
    private const string ReferenceInventorySha256 =
        "30623f64b7d85564260e62464e652bfaa89eb56e0e55193989bfb99538ba6cad";
    private static readonly byte[] ReferenceInventoryDomain = Encoding.ASCII.GetBytes(
        "MPK-CSHARP-REFERENCE-INVENTORY-0.1\0");

    internal static PracticalNormalizedSyntax Normalize(
        PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> capturedInputs,
        ImmutableArray<MetadataReference> references,
        Action<CSharpCompilation>? validateDataDeclarations = null,
        Action<CSharpCompilation>? validateDataTypes = null,
        Action<CSharpCompilation>? validateDataLimits = null,
        Action<CSharpCompilation, PracticalSourceClosure>? validateConstruction = null)
    {
        try
        {
            PracticalSourceClosure closure = CSharpPracticalCapture.Validate(
                selection,
                capturedInputs,
                references,
                current =>
                {
                    if (validateConstruction is not null)
                    {
                        // W04 must not let a phase-6 construction finding mask
                        // a prerequisite syntax/declaration failure.
                        var prerequisite = new SyntaxState(current, current.SyntaxTrees.ToImmutableArray());
                        ValidateImportsAndDirectives(prerequisite);
                        ValidateExpressionBodies(prerequisite);
                        ValidateVarContexts(prerequisite);
                    }
                    validateDataDeclarations?.Invoke(current);
                },
                validateDataTypes,
                validateDataLimits, validateConstruction);
            SyntaxState state = CreateState(selection, closure, references);
            ValidateImportsAndDirectives(state);
            ValidateExpressionBodies(state);
            ValidateVarContexts(state);
            return new PracticalSyntaxModel(state, closure).Build();
        }
        catch (PracticalCaptureFailure)
        {
            throw;
        }
        catch (Exception)
        {
            throw PracticalFailures.Protocol("syntax_normalizer");
        }
    }

    private static SyntaxState CreateState(
        PracticalSourceSelection selection,
        PracticalSourceClosure closure,
        ImmutableArray<MetadataReference> references)
    {
        ImmutableArray<MetadataReference> snapshots = SnapshotReferences(references);
        var parseOptions = new CSharpParseOptions(
            (LanguageVersion)1400,
            DocumentationMode.None,
            SourceCodeKind.Regular,
            preprocessorSymbols: Array.Empty<string>());
        var trees = ImmutableArray.CreateBuilder<SyntaxTree>(closure.Sources.Count);
        foreach (PracticalSourceFile source in closure.Sources)
        {
            trees.Add(CSharpSyntaxTree.ParseText(
                source.Text,
                parseOptions,
                source.Path,
                Encoding.UTF8,
                CancellationToken.None));
        }

        var options = new CSharpCompilationOptions(
            OutputKind.DynamicallyLinkedLibrary,
            optimizationLevel: OptimizationLevel.Release,
            checkOverflow: true,
            allowUnsafe: false,
            platform: Platform.X64,
            warningLevel: 9999,
            generalDiagnosticOption: ReportDiagnostic.Error,
            nullableContextOptions: NullableContextOptions.Enable,
            concurrentBuild: false,
            deterministic: true,
            metadataImportOptions: MetadataImportOptions.Public);
        CSharpCompilation compilation = CSharpCompilation.Create(
            selection.CompilationId,
            trees.MoveToImmutable(),
            snapshots,
            options);
        if (compilation.GetDiagnostics(CancellationToken.None).Any(diagnostic =>
            !diagnostic.IsSuppressed
            && diagnostic.Severity is DiagnosticSeverity.Error or DiagnosticSeverity.Warning))
        {
            throw PracticalFailures.Protocol("syntax_recompile");
        }

        return new SyntaxState(compilation, compilation.SyntaxTrees.ToImmutableArray());
    }

    private static ImmutableArray<MetadataReference> SnapshotReferences(
        ImmutableArray<MetadataReference> references)
    {
        if (references.IsDefault || references.Length != 167)
        {
            throw PracticalFailures.Dependency("ambient_reference");
        }

        var records = new List<SyntaxReferenceRecord>(references.Length);
        try
        {
            foreach (MetadataReference reference in references)
            {
                if (reference is not PortableExecutableReference portable
                    || !portable.Properties.Equals(MetadataReferenceProperties.Assembly)
                    || string.IsNullOrEmpty(portable.FilePath))
                {
                    throw PracticalFailures.Dependency("ambient_reference");
                }

                var file = new FileInfo(portable.FilePath);
                string name = file.Name;
                if (!name.EndsWith(".dll", StringComparison.Ordinal)
                    || file.LinkTarget is not null
                    || (file.Attributes & FileAttributes.ReparsePoint) != 0)
                {
                    throw PracticalFailures.Dependency("reference_projection");
                }

                long length = file.Length;
                if (length < 0 || length > ReferenceTotalBytes)
                {
                    throw PracticalFailures.Dependency("reference_projection");
                }

                byte[] image = new byte[checked((int)length)];
                using (var stream = new FileStream(
                    file.FullName,
                    FileMode.Open,
                    FileAccess.Read,
                    FileShare.Read,
                    64 * 1024,
                    FileOptions.SequentialScan))
                {
                    int offset = 0;
                    while (offset < image.Length)
                    {
                        int read = stream.Read(image, offset, image.Length - offset);
                        if (read == 0)
                        {
                            break;
                        }

                        offset = checked(offset + read);
                    }

                    if (offset != image.Length
                        || stream.ReadByte() != -1
                        || stream.Length != length)
                    {
                        throw PracticalFailures.Dependency("reference_projection");
                    }
                }

                string path = "ref/net10.0/" + name;
                records.Add(new SyntaxReferenceRecord(
                    path,
                    image.LongLength,
                    Convert.ToHexString(SHA256.HashData(image)).ToLowerInvariant(),
                    MetadataReference.CreateFromImage(
                        ImmutableArray.Create(image),
                        MetadataReferenceProperties.Assembly,
                        documentation: null,
                        filePath: path)));
            }
        }
        catch (PracticalCaptureFailure)
        {
            throw;
        }
        catch (Exception error) when (
            error is ArgumentException
            or IOException
            or NotSupportedException
            or UnauthorizedAccessException
            or System.Security.SecurityException)
        {
            throw PracticalFailures.Dependency("reference_projection");
        }

        records.Sort((left, right) => string.CompareOrdinal(left.Path, right.Path));
        long total = 0;
        string? previous = null;
        foreach (SyntaxReferenceRecord record in records)
        {
            if (previous is not null && string.CompareOrdinal(previous, record.Path) >= 0)
            {
                throw PracticalFailures.Dependency("ambient_reference");
            }

            total = checked(total + record.SizeBytes);
            previous = record.Path;
        }

        byte[] canonical = CanonicalReferenceInventory(records);
        var preimage = new byte[checked(ReferenceInventoryDomain.Length + canonical.Length)];
        Buffer.BlockCopy(ReferenceInventoryDomain, 0, preimage, 0, ReferenceInventoryDomain.Length);
        Buffer.BlockCopy(canonical, 0, preimage, ReferenceInventoryDomain.Length, canonical.Length);
        string hash = Convert.ToHexString(SHA256.HashData(preimage)).ToLowerInvariant();
        if (total != ReferenceTotalBytes
            || canonical.Length != ReferenceCanonicalBytes
            || !string.Equals(hash, ReferenceInventorySha256, StringComparison.Ordinal))
        {
            throw PracticalFailures.Dependency("reference_projection");
        }

        return records.Select(record => (MetadataReference)record.Reference).ToImmutableArray();
    }

    private static byte[] CanonicalReferenceInventory(
        IReadOnlyList<SyntaxReferenceRecord> records)
    {
        using var output = new MemoryStream(ReferenceCanonicalBytes);
        using (var writer = new Utf8JsonWriter(
            output,
            new JsonWriterOptions
            {
                Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping,
                Indented = false,
                SkipValidation = false,
            }))
        {
            writer.WriteStartArray();
            foreach (SyntaxReferenceRecord record in records)
            {
                writer.WriteStartObject();
                writer.WriteString("path", record.Path);
                writer.WriteString("sha256", record.Sha256);
                writer.WriteNumber("size_bytes", record.SizeBytes);
                writer.WriteEndObject();
            }
            writer.WriteEndArray();
        }

        return output.ToArray();
    }

    private static void ValidateImportsAndDirectives(SyntaxState state)
    {
        foreach (SyntaxTree tree in state.Trees)
        {
            SemanticModel model = state.Compilation.GetSemanticModel(tree, false);
            SyntaxNode root = tree.GetRoot(CancellationToken.None);
            foreach (UsingDirectiveSyntax directive in root.DescendantNodes()
                .OfType<UsingDirectiveSyntax>())
            {
                ISymbol? imported = directive.Name is null
                    ? null
                    : model.GetSymbolInfo(directive.Name, CancellationToken.None).Symbol;
                bool disallowedToken = directive.ChildTokens().Any(token => token.IsKind(
                    SyntaxKind.GlobalKeyword)
                    || token.IsKind(SyntaxKind.StaticKeyword)
                    || token.IsKind(SyntaxKind.UnsafeKeyword));
                if (directive.Alias is not null
                    || disallowedToken
                    || directive.Name is null
                    || directive.Name.DescendantNodesAndSelf()
                        .Any(node => node is AliasQualifiedNameSyntax)
                    || directive.Parent is not CompilationUnitSyntax
                        and not BaseNamespaceDeclarationSyntax
                    || imported is not INamespaceSymbol importedNamespace
                    || IsMpkNamespace(importedNamespace.ToDisplayString()))
                {
                    throw PracticalFailures.Declaration("using_directive");
                }
            }

            if (root.DescendantNodes().Any(node => node is ExternAliasDirectiveSyntax
                or UsingStatementSyntax)
                || root.DescendantNodes().OfType<LocalDeclarationStatementSyntax>()
                    .Any(local => local.UsingKeyword.RawKind != 0
                        || local.AwaitKeyword.RawKind != 0))
            {
                throw PracticalFailures.Declaration("using_directive");
            }

            int nullableDirectives = 0;
            foreach (DirectiveTriviaSyntax directive in root.DescendantTrivia(descendIntoTrivia: true)
                .Select(trivia => trivia.GetStructure())
                .OfType<DirectiveTriviaSyntax>())
            {
                if (directive is not NullableDirectiveTriviaSyntax nullable
                    || !nullable.IsActive
                    || !nullable.SettingToken.IsKind(SyntaxKind.EnableKeyword)
                    || nullable.TargetToken.RawKind != 0
                    || checked(++nullableDirectives) != 1)
                {
                    throw PracticalFailures.Declaration("source_directive");
                }

                SyntaxToken firstToken = root.GetFirstToken(includeZeroWidth: true);
                if (nullable.Span.End > firstToken.SpanStart)
                {
                    throw PracticalFailures.Declaration("source_directive");
                }
            }
        }
    }

    private static bool IsMpkNamespace(string value) =>
        string.Equals(value, "Mpk", StringComparison.OrdinalIgnoreCase)
        || value.StartsWith("Mpk.", StringComparison.OrdinalIgnoreCase);

    private static void ValidateExpressionBodies(SyntaxState state)
    {
        foreach (SyntaxTree tree in state.Trees)
        {
            foreach (ArrowExpressionClauseSyntax arrow in tree.GetRoot(CancellationToken.None)
                .DescendantNodes()
                .OfType<ArrowExpressionClauseSyntax>())
            {
                bool admitted = arrow.Parent is MethodDeclarationSyntax
                    or PropertyDeclarationSyntax
                    || arrow.Parent is AccessorDeclarationSyntax accessor
                        && accessor.IsKind(SyntaxKind.GetAccessorDeclaration);
                if (!admitted)
                {
                    throw PracticalFailures.Declaration("expression_body_kind");
                }
            }
        }
    }

    private static void ValidateVarContexts(SyntaxState state)
    {
        foreach (SyntaxTree tree in state.Trees)
        {
            SemanticModel model = state.Compilation.GetSemanticModel(tree, false);
            SyntaxNode root = tree.GetRoot(CancellationToken.None);
            foreach (IdentifierNameSyntax identifier in root.DescendantNodes()
                .OfType<IdentifierNameSyntax>()
                .Where(identifier => string.Equals(
                    identifier.Identifier.ValueText,
                    "var",
                    StringComparison.Ordinal)))
            {
                VariableDeclarationSyntax? declaration = identifier.AncestorsAndSelf()
                    .OfType<VariableDeclarationSyntax>()
                    .FirstOrDefault(candidate => candidate.Type.DescendantNodesAndSelf()
                        .Contains(identifier));
                if (declaration is not null)
                {
                    if (declaration.Type != identifier)
                    {
                        throw PracticalFailures.Declaration("var_shape");
                    }

                    ValidateVarDeclaration(declaration, model);
                    continue;
                }

                if (identifier.Parent is ForEachStatementSyntax
                        or DeclarationExpressionSyntax
                        or DeclarationPatternSyntax)
                {
                    throw PracticalFailures.Declaration("var_context");
                }
            }
        }
    }

    private static void ValidateVarDeclaration(
        VariableDeclarationSyntax declaration,
        SemanticModel model)
    {
        ILocalSymbol? local = declaration.Variables.Count == 1
            ? model.GetDeclaredSymbol(
                declaration.Variables[0],
                CancellationToken.None) as ILocalSymbol
            : null;
        ISymbol? resolvedVar = model.GetSymbolInfo(
            declaration.Type,
            CancellationToken.None).Symbol;
        if (declaration.Parent is not LocalDeclarationStatementSyntax
                and not ForStatementSyntax
            || declaration.Parent is LocalDeclarationStatementSyntax localDeclaration
                && (localDeclaration.UsingKeyword.RawKind != 0
                    || localDeclaration.AwaitKeyword.RawKind != 0)
            || declaration.Variables.Count != 1
            || declaration.Variables[0].Initializer is null
            || local is null
            || resolvedVar is ITypeSymbol resolvedType
                && string.Equals(resolvedType.Name, "var", StringComparison.Ordinal)
            || model.GetAliasInfo(declaration.Type, CancellationToken.None) is not null)
        {
            throw PracticalFailures.Declaration("var_shape");
        }

        ExpressionSyntax initializer = declaration.Variables[0].Initializer!.Value;
        ValidateImplicitArrayInference(initializer, model);
        if (initializer.DescendantNodesAndSelf().Any(node =>
            node is AnonymousObjectCreationExpressionSyntax
                or ImplicitObjectCreationExpressionSyntax
                or ImplicitStackAllocArrayCreationExpressionSyntax
                or CollectionExpressionSyntax
            || node.IsKind(SyntaxKind.DefaultLiteralExpression)))
        {
            throw PracticalFailures.Type("target_typed_inference");
        }

        TypeInfo initializerType = model.GetTypeInfo(initializer, CancellationToken.None);
        ITypeSymbol? exactInitializer = initializerType.Type ?? initializerType.ConvertedType;
        if (exactInitializer is null
            || !SymbolEqualityComparer.Default.Equals(local.Type, exactInitializer))
        {
            throw PracticalFailures.Type("var_inference_type");
        }

    }

    private static void ValidateImplicitArrayInference(
        ExpressionSyntax initializer,
        SemanticModel model)
    {
        foreach (ImplicitArrayCreationExpressionSyntax implicitArray in initializer
            .DescendantNodesAndSelf()
            .OfType<ImplicitArrayCreationExpressionSyntax>())
        {
            TypeInfo arrayInformation = model.GetTypeInfo(
                implicitArray,
                CancellationToken.None);
            if ((arrayInformation.Type ?? arrayInformation.ConvertedType)
                    is not IArrayTypeSymbol arrayType
                || !arrayType.IsSZArray
                || arrayType.Rank != 1)
            {
                throw PracticalFailures.Type("target_typed_inference");
            }

            foreach (ExpressionSyntax element in implicitArray.Initializer.Expressions)
            {
                TypeInfo elementInformation = model.GetTypeInfo(
                    element,
                    CancellationToken.None);
                if (elementInformation.Type is null
                    || !SymbolEqualityComparer.IncludeNullability.Equals(
                        elementInformation.Type,
                        arrayType.ElementType))
                {
                    throw PracticalFailures.Type("target_typed_inference");
                }
            }
        }
    }

    private sealed class SyntaxState
    {
        internal SyntaxState(
            CSharpCompilation compilation,
            ImmutableArray<SyntaxTree> trees)
        {
            Compilation = compilation;
            Trees = trees;
        }

        internal CSharpCompilation Compilation { get; }

        internal ImmutableArray<SyntaxTree> Trees { get; }
    }

    private sealed class SyntaxReferenceRecord
    {
        internal SyntaxReferenceRecord(
            string path,
            long sizeBytes,
            string sha256,
            PortableExecutableReference reference)
        {
            Path = path;
            SizeBytes = sizeBytes;
            Sha256 = sha256;
            Reference = reference;
        }

        internal string Path { get; }

        internal long SizeBytes { get; }

        internal string Sha256 { get; }

        internal PortableExecutableReference Reference { get; }
    }

    private sealed class PracticalSyntaxModel
    {
        private readonly SyntaxState state;
        private readonly PracticalSourceClosure closure;
        private readonly Dictionary<ISymbol, CallableRecord> callables =
            new Dictionary<ISymbol, CallableRecord>(SymbolEqualityComparer.Default);
        private readonly Dictionary<ITypeSymbol, PracticalNormalizedType> normalizedTypes =
            new Dictionary<ITypeSymbol, PracticalNormalizedType>(
                SymbolEqualityComparer.IncludeNullability);
        private readonly Dictionary<ITypeSymbol, PracticalNormalizedType> normalizedReturnTypes =
            new Dictionary<ITypeSymbol, PracticalNormalizedType>(
                SymbolEqualityComparer.IncludeNullability);
        private readonly Dictionary<ITypeSymbol, PracticalNormalizedType> normalizedAnnotatedTypes =
            new Dictionary<ITypeSymbol, PracticalNormalizedType>(
                SymbolEqualityComparer.IncludeNullability);
        private readonly Dictionary<ITypeSymbol, PracticalNormalizedType> normalizedNotAnnotatedTypes =
            new Dictionary<ITypeSymbol, PracticalNormalizedType>(
                SymbolEqualityComparer.IncludeNullability);

        internal PracticalSyntaxModel(SyntaxState state, PracticalSourceClosure closure)
        {
            this.state = state;
            this.closure = closure;
        }

        internal PracticalNormalizedSyntax Build()
        {
            CollectCallables();
            ValidateCallableClosure();
            var normalizedCallables = new List<PracticalNormalizedCallable>(callables.Count);
            var exactTypes = new List<PracticalExactTypeBinding>();
            foreach (CallableRecord callable in callables.Values.OrderBy(
                value => value.Id,
                StringComparer.Ordinal))
            {
                callable.CollectLocals(exactTypes);
                normalizedCallables.Add(new PracticalNormalizedCallable(
                    callable.Id,
                    callable.CanonicalBody()));
            }

            PracticalNormalizedCallable[] callableArray = normalizedCallables.ToArray();
            PracticalExactTypeBinding[] exactTypeArray = exactTypes
                .OrderBy(binding => binding.CallableId, StringComparer.Ordinal)
                .ThenBy(binding => binding.LocalOrdinal)
                .ToArray();
            byte[] canonical = CanonicalArtifact(callableArray, exactTypeArray);
            return new PracticalNormalizedSyntax(
                closure,
                callableArray,
                exactTypeArray,
                canonical);
        }

        private void CollectCallables()
        {
            foreach (SyntaxTree tree in state.Trees)
            {
                SemanticModel model = state.Compilation.GetSemanticModel(tree, false);
                SyntaxNode root = tree.GetRoot(CancellationToken.None);
                foreach (MethodDeclarationSyntax syntax in root.DescendantNodes()
                    .OfType<MethodDeclarationSyntax>())
                {
                    IMethodSymbol symbol = model.GetDeclaredSymbol(syntax, CancellationToken.None)
                        ?? throw PracticalFailures.Declaration("callable_shape");
                    AddCallable(new CallableRecord(
                        this,
                        CallableId(symbol, "method", symbol.Name),
                        syntax,
                        symbol,
                        model));
                }

                foreach (ConstructorDeclarationSyntax syntax in root.DescendantNodes()
                    .OfType<ConstructorDeclarationSyntax>())
                {
                    IMethodSymbol symbol = model.GetDeclaredSymbol(syntax, CancellationToken.None)
                        ?? throw PracticalFailures.Declaration("callable_shape");
                    AddCallable(new CallableRecord(
                        this,
                        CallableId(symbol, "constructor", symbol.ContainingType.Name),
                        syntax,
                        symbol,
                        model));
                }

                foreach (PropertyDeclarationSyntax syntax in root.DescendantNodes()
                    .OfType<PropertyDeclarationSyntax>())
                {
                    IPropertySymbol property = model.GetDeclaredSymbol(syntax, CancellationToken.None)
                        ?? throw PracticalFailures.Declaration("callable_shape");
                    IMethodSymbol getter = property.GetMethod
                        ?? throw PracticalFailures.Declaration("property_shape");
                    AddCallable(new CallableRecord(
                        this,
                        CallableId(getter, "method", "get_" + property.Name),
                        syntax,
                        getter,
                        model));
                }
            }
        }

        private void AddCallable(CallableRecord callable)
        {
            if (!callables.TryAdd(callable.Symbol, callable))
            {
                throw PracticalFailures.Declaration("declaration_identity_collision");
            }
        }

        private void ValidateCallableClosure()
        {
            string[] expected = closure.Declarations
                .Where(declaration => declaration.Kind != PracticalDeclarationKind.Type
                    && PracticalIdentity.IsSourceId(declaration.Id))
                .Select(declaration => declaration.Id)
                .OrderBy(value => value, StringComparer.Ordinal)
                .ToArray();
            string[] observed = callables.Values
                .Select(callable => callable.Id)
                .OrderBy(value => value, StringComparer.Ordinal)
                .ToArray();
            if (!expected.SequenceEqual(observed, StringComparer.Ordinal))
            {
                throw PracticalFailures.Protocol("normalized_callable_closure");
            }
        }

        private string CallableId(IMethodSymbol method, string kind, string name)
        {
            string owner = PracticalIdentity.SourceTypeId(
                method.ContainingNamespace.ToDisplayString(),
                method.ContainingType.Name);
            string[] parameters = method.Parameters
                .Select(parameter => NormalizeType(parameter.Type).Id)
                .ToArray();
            string result = kind == "constructor"
                ? owner
                : NormalizeType(method.ReturnType, allowVoid: true).Id;
            return PracticalIdentity.CallableId(
                kind,
                method.ContainingNamespace.ToDisplayString(),
                owner,
                name,
                parameters,
                result);
        }

        private PracticalNormalizedType NormalizeType(
            ITypeSymbol type,
            bool allowVoid = false)
        {
            Dictionary<ITypeSymbol, PracticalNormalizedType> cache = allowVoid
                ? normalizedReturnTypes
                : normalizedTypes;
            if (!cache.TryGetValue(type, out PracticalNormalizedType? normalized))
            {
                normalized = PracticalExactTypeNormalizer.Normalize(
                    type,
                    state.Compilation,
                    allowVoid);
                cache.Add(type, normalized);
            }

            return normalized;
        }

        private PracticalNormalizedType NormalizeInferredType(
            ITypeSymbol type,
            NullableAnnotation topLevelNullability)
        {
            Dictionary<ITypeSymbol, PracticalNormalizedType> cache = topLevelNullability switch
            {
                NullableAnnotation.None => normalizedTypes,
                NullableAnnotation.Annotated => normalizedAnnotatedTypes,
                NullableAnnotation.NotAnnotated => normalizedNotAnnotatedTypes,
                _ => throw PracticalFailures.Type("nullable_inference"),
            };
            if (!cache.TryGetValue(type, out PracticalNormalizedType? normalized))
            {
                normalized = PracticalExactTypeNormalizer.Normalize(
                    type,
                    state.Compilation,
                    topLevelNullability: topLevelNullability);
                cache.Add(type, normalized);
            }

            return normalized;
        }

        private byte[] CanonicalArtifact(
            IReadOnlyList<PracticalNormalizedCallable> normalizedCallables,
            IReadOnlyList<PracticalExactTypeBinding> exactTypes)
        {
            using var output = new MemoryStream();
            using (var writer = CanonicalWriter(output))
            {
                writer.WriteStartObject();
                writer.WritePropertyName("call_edges");
                WriteEdges(writer, closure.CallEdges);
                writer.WritePropertyName("callables");
                writer.WriteStartArray();
                foreach (PracticalNormalizedCallable callable in normalizedCallables)
                {
                    writer.WriteStartObject();
                    writer.WriteString(
                        "body",
                        Encoding.UTF8.GetString(callable.CopyBodyBytes()));
                    writer.WriteString("body_sha256", callable.BodySha256);
                    writer.WriteString("id", callable.Id);
                    writer.WriteEndObject();
                }
                writer.WriteEndArray();
                writer.WritePropertyName("declarations");
                writer.WriteStartArray();
                foreach (PracticalDeclaration declaration in closure.Declarations
                    .OrderBy(value => value.Id, StringComparer.Ordinal)
                    .ThenBy(value => value.Kind))
                {
                    writer.WriteStartObject();
                    writer.WriteString("id", declaration.Id);
                    writer.WriteString("kind", declaration.Kind.ToString());
                    writer.WriteEndObject();
                }
                writer.WriteEndArray();
                writer.WritePropertyName("exact_types");
                writer.WriteStartArray();
                foreach (PracticalExactTypeBinding binding in exactTypes)
                {
                    writer.WriteStartObject();
                    writer.WriteString("callable_id", binding.CallableId);
                    writer.WriteNumber("local_ordinal", binding.LocalOrdinal);
                    writer.WriteString("type", binding.Type.CanonicalKey);
                    writer.WriteEndObject();
                }
                writer.WriteEndArray();
                writer.WriteString("schema", "mpk.csharp_practical.normalized_syntax.v1");
                writer.WriteNumber(
                    "source_data_exception_type_count",
                    closure.SourceDataExceptionTypeCount);
                writer.WritePropertyName("type_edges");
                WriteEdges(writer, closure.TypeEdges);
                writer.WriteEndObject();
            }

            return output.ToArray();
        }

        private static void WriteEdges(
            Utf8JsonWriter writer,
            IEnumerable<PracticalGraphEdge> edges)
        {
            writer.WriteStartArray();
            foreach (PracticalGraphEdge edge in edges
                .OrderBy(value => value.SourceId, StringComparer.Ordinal)
                .ThenBy(value => value.TargetId, StringComparer.Ordinal))
            {
                writer.WriteStartObject();
                writer.WriteString("source", edge.SourceId);
                writer.WriteString("target", edge.TargetId);
                writer.WriteEndObject();
            }
            writer.WriteEndArray();
        }

        private sealed class CallableRecord
        {
            private readonly Dictionary<ISymbol, int> localOrdinals =
                new Dictionary<ISymbol, int>(SymbolEqualityComparer.Default);
            private readonly PracticalSyntaxModel syntaxModel;

            internal CallableRecord(
                PracticalSyntaxModel syntaxModel,
                string id,
                SyntaxNode syntax,
                IMethodSymbol symbol,
                SemanticModel model)
            {
                this.syntaxModel = syntaxModel;
                Id = id;
                Syntax = syntax;
                Symbol = symbol;
                Model = model;
            }

            internal string Id { get; }

            internal SyntaxNode Syntax { get; }

            internal IMethodSymbol Symbol { get; }

            internal SemanticModel Model { get; }

            internal void CollectLocals(List<PracticalExactTypeBinding> bindings)
            {
                foreach (VariableDeclaratorSyntax variable in Syntax.DescendantNodes()
                    .OfType<VariableDeclaratorSyntax>())
                {
                    if (Model.GetDeclaredSymbol(variable, CancellationToken.None)
                        is not ILocalSymbol local)
                    {
                        continue;
                    }

                    int ordinal = localOrdinals.Count;
                    if (!localOrdinals.TryAdd(local, ordinal))
                    {
                        throw PracticalFailures.Declaration("local_identity");
                    }

                    bindings.Add(new PracticalExactTypeBinding(
                        Id,
                        ordinal,
                        NormalizeLocalType(variable, local)));
                }
            }

            private PracticalNormalizedType NormalizeLocalType(
                VariableDeclaratorSyntax variable,
                ILocalSymbol local)
            {
                if (variable.Parent is VariableDeclarationSyntax declaration
                    && declaration.Type is IdentifierNameSyntax identifier
                    && string.Equals(
                        identifier.Identifier.ValueText,
                        "var",
                        StringComparison.Ordinal)
                    && variable.Initializer is not null)
                {
                    TypeInfo information = Model.GetTypeInfo(
                        variable.Initializer.Value,
                        CancellationToken.None);
                    ITypeSymbol inferred = information.Type
                        ?? information.ConvertedType
                        ?? throw PracticalFailures.Type("exact_type");
                    NullableAnnotation nullability = information.Type is null
                        ? information.ConvertedNullability.Annotation
                        : information.Nullability.Annotation;
                    return syntaxModel.NormalizeInferredType(
                        inferred,
                        nullability);
                }

                return syntaxModel.NormalizeType(local.Type);
            }

            internal byte[] CanonicalBody()
            {
                using var output = new MemoryStream();
                using (var writer = CanonicalWriter(output))
                {
                    writer.WriteStartArray();
                    switch (Syntax)
                    {
                        case MethodDeclarationSyntax method:
                            WriteMethodBody(writer, method);
                            break;
                        case ConstructorDeclarationSyntax constructor:
                            WriteConstructorBody(writer, constructor);
                            break;
                        case PropertyDeclarationSyntax property:
                            WritePropertyBody(writer, property);
                            break;
                        default:
                            throw PracticalFailures.Protocol("normalized_callable_shape");
                    }
                    writer.WriteEndArray();
                }

                return output.ToArray();
            }

            private void WriteMethodBody(
                Utf8JsonWriter writer,
                MethodDeclarationSyntax method)
            {
                if (method.ExpressionBody is not null)
                {
                    IOperation expression = Operation(method.ExpressionBody.Expression);
                    WriteSyntheticStatement(
                        writer,
                        Symbol.ReturnsVoid ? "ExpressionStatement" : "Return",
                        expression);
                    return;
                }

                WriteBlock(writer, method.Body);
            }

            private void WriteConstructorBody(
                Utf8JsonWriter writer,
                ConstructorDeclarationSyntax constructor)
            {
                if (constructor.ExpressionBody is not null)
                {
                    throw PracticalFailures.Declaration("expression_body_kind");
                }

                if (constructor.Initializer is not null)
                {
                    WriteSyntheticNode(writer, "ConstructorInitializer", 1);
                    WriteOperation(writer, Operation(constructor.Initializer));
                }

                WriteBlock(writer, constructor.Body);
            }

            private void WritePropertyBody(
                Utf8JsonWriter writer,
                PropertyDeclarationSyntax property)
            {
                if (property.ExpressionBody is not null)
                {
                    WriteSyntheticStatement(
                        writer,
                        "Return",
                        Operation(property.ExpressionBody.Expression));
                    return;
                }

                AccessorDeclarationSyntax? getter = property.AccessorList?.Accessors
                    .SingleOrDefault(accessor => accessor.IsKind(
                        SyntaxKind.GetAccessorDeclaration));
                if (getter is null)
                {
                    throw PracticalFailures.Declaration("property_shape");
                }

                if (getter.ExpressionBody is not null)
                {
                    WriteSyntheticStatement(
                        writer,
                        "Return",
                        Operation(getter.ExpressionBody.Expression));
                    return;
                }

                if (getter.Body is not null)
                {
                    WriteBlock(writer, getter.Body);
                }
            }

            private void WriteBlock(Utf8JsonWriter writer, BlockSyntax? block)
            {
                if (block is null)
                {
                    return;
                }

                if (Model.GetOperation(block, CancellationToken.None)
                    is not IBlockOperation operation)
                {
                    throw PracticalFailures.Protocol("normalized_operation");
                }

                foreach (IOperation statement in operation.Operations)
                {
                    WriteOperation(writer, statement);
                }
            }

            private IOperation Operation(SyntaxNode syntax) =>
                Model.GetOperation(syntax, CancellationToken.None)
                ?? throw PracticalFailures.Protocol("normalized_operation");

            private void WriteSyntheticStatement(
                Utf8JsonWriter writer,
                string kind,
                IOperation value)
            {
                WriteSyntheticNode(writer, kind, 1);
                WriteOperation(writer, value);
            }

            private static void WriteSyntheticNode(
                Utf8JsonWriter writer,
                string kind,
                int childCount)
            {
                writer.WriteStartObject();
                writer.WriteNull("constant");
                writer.WriteNumber("child_count", childCount);
                writer.WriteBoolean("implicit", false);
                writer.WriteString("kind", kind);
                writer.WriteString("symbol", string.Empty);
                writer.WriteString("traits", string.Empty);
                writer.WriteNull("type");
                writer.WriteEndObject();
            }

            private void WriteOperation(Utf8JsonWriter writer, IOperation operation)
            {
                var pending = new Stack<IOperation>();
                pending.Push(operation);
                while (pending.Count != 0)
                {
                    IOperation current = pending.Pop();
                    if (current.Kind == OperationKind.Invalid)
                    {
                        throw PracticalFailures.Protocol("normalized_operation");
                    }

                    IOperation[] children = current.ChildOperations.ToArray();
                    WriteOperationNode(writer, current, children.Length);
                    for (int index = children.Length - 1; index >= 0; index--)
                    {
                        pending.Push(children[index]);
                    }
                }
            }

            private void WriteOperationNode(
                Utf8JsonWriter writer,
                IOperation operation,
                int childCount)
            {
                writer.WriteStartObject();
                WriteConstant(writer, operation);
                writer.WriteNumber("child_count", childCount);
                writer.WriteBoolean("implicit", operation.IsImplicit);
                writer.WriteString("kind", operation.Kind.ToString());
                writer.WriteString("symbol", SymbolKey(operation));
                writer.WriteString("traits", Traits(operation));
                if (operation.Type is null)
                {
                    writer.WriteNull("type");
                }
                else
                {
                    writer.WriteString(
                        "type",
                        CanonicalOperationType(operation));
                }
                writer.WriteEndObject();
            }

            private string CanonicalOperationType(IOperation operation)
            {
                ITypeSymbol type = operation.Type!;
                // W06: Roslyn inserts object reference conversions for a
                // class/array null comparison. Retain the exact operand's
                // nullable value type; object itself never enters the model.
                if (type.SpecialType == SpecialType.System_Object
                    && operation is IConversionOperation { IsImplicit: true }
                    && operation.Parent is IBinaryOperation binary
                    && binary.OperatorMethod is null
                    && binary.OperatorKind is BinaryOperatorKind.Equals or BinaryOperatorKind.NotEquals
                    && ((binary.LeftOperand.ConstantValue.HasValue && binary.LeftOperand.ConstantValue.Value is null)
                        || (binary.RightOperand.ConstantValue.HasValue && binary.RightOperand.ConstantValue.Value is null)))
                {
                    foreach (IOperation operand in new[] { binary.LeftOperand, binary.RightOperand })
                    {
                        IOperation exact = operand;
                        while (exact is IConversionOperation { IsImplicit: true } conversion) { exact = conversion.Operand; }
                        if (exact.Type is ITypeSymbol reference && reference.IsReferenceType
                            && reference.SpecialType != SpecialType.System_Object)
                        { return syntaxModel.NormalizeType(reference.WithNullableAnnotation(NullableAnnotation.Annotated)).CanonicalKey; }
                    }
                }

                // Roslyn's default-value operation can erase the reference
                // annotation. Recover it from the exact admitted type syntax,
                // never from a target or nullable-flow guess.
                if (operation is IDefaultValueOperation && type.IsReferenceType
                    && operation.Syntax is DefaultExpressionSyntax exactDefault)
                {
                    return syntaxModel.NormalizeType(type.WithNullableAnnotation(
                        exactDefault.Type is NullableTypeSyntax
                            ? NullableAnnotation.Annotated : NullableAnnotation.NotAnnotated)).CanonicalKey;
                }
                // W05: the implicit initializer receiver is the fresh, non-null
                // object of its enclosing exact new expression. Roslyn omits
                // that annotation on the initializer and receiver operations.
                if (type.IsReferenceType && (operation is IObjectOrCollectionInitializerOperation
                    || operation is IInstanceReferenceOperation { ReferenceKind: InstanceReferenceKind.ImplicitReceiver }))
                {
                    for (IOperation? parent = operation.Parent; parent is not null; parent = parent.Parent)
                    {
                        if (parent is IObjectCreationOperation creation
                            && SymbolEqualityComparer.Default.Equals(creation.Type, type))
                        { return syntaxModel.NormalizeType(type.WithNullableAnnotation(NullableAnnotation.NotAnnotated)).CanonicalKey; }
                    }
                }
                if (IsIntrinsicArgumentCarrier(type, "StringComparison"))
                {
                    return "intrinsic_argument:System.StringComparison";
                }

                if (IsIntrinsicArgumentCarrier(type, "MidpointRounding"))
                {
                    return "intrinsic_argument:System.MidpointRounding";
                }

                return syntaxModel.NormalizeType(type, allowVoid: true).CanonicalKey;
            }

            private static bool IsIntrinsicArgumentCarrier(
                ITypeSymbol type,
                string metadataName) =>
                type is INamedTypeSymbol named
                && named.Arity == 0
                && named.DeclaringSyntaxReferences.IsEmpty
                && string.Equals(named.MetadataName, metadataName, StringComparison.Ordinal)
                && string.Equals(
                    named.ContainingNamespace.ToDisplayString(),
                    "System",
                    StringComparison.Ordinal)
                && string.Equals(
                    named.ContainingAssembly?.Identity.Name,
                    "System.Runtime",
                    StringComparison.Ordinal);

            private static void WriteConstant(
                Utf8JsonWriter writer,
                IOperation operation)
            {
                if (!operation.ConstantValue.HasValue)
                {
                    writer.WriteNull("constant");
                    return;
                }

                object? value = operation.ConstantValue.Value;
                string encoded = value switch
                {
                    null => "null",
                    bool boolean => boolean ? "bool:true" : "bool:false",
                    string text => "string:" + text,
                    char character => "char:"
                        + ((int)character).ToString(CultureInfo.InvariantCulture),
                    float number => "f32:"
                        + BitConverter.SingleToUInt32Bits(number).ToString(
                            "x8",
                            CultureInfo.InvariantCulture),
                    double number => "f64:"
                        + BitConverter.DoubleToUInt64Bits(number).ToString(
                            "x16",
                            CultureInfo.InvariantCulture),
                    decimal number => "decimal:"
                        + string.Join(
                            ",",
                            decimal.GetBits(number).Select(part => part.ToString(
                                "x8",
                                CultureInfo.InvariantCulture))),
                    IFormattable formattable => value.GetType().FullName + ":"
                        + formattable.ToString(null, CultureInfo.InvariantCulture),
                    _ => value.GetType().FullName + ":" + value,
                };
                writer.WriteString("constant", encoded);
            }

            private string SymbolKey(IOperation operation)
            {
                return operation switch
                {
                    IInvocationOperation value => SymbolKey(value.TargetMethod),
                    IObjectCreationOperation value => SymbolKey(value.Constructor),
                    IPropertyReferenceOperation value => SymbolKey(value.Property),
                    IFieldReferenceOperation value => SymbolKey(value.Field),
                    IMethodReferenceOperation value => SymbolKey(value.Method),
                    IEventReferenceOperation value => SymbolKey(value.Event),
                    ILocalReferenceOperation value => LocalKey(value.Local),
                    IVariableDeclaratorOperation value => LocalKey(value.Symbol),
                    IParameterReferenceOperation value => "parameter:"
                        + value.Parameter.Ordinal.ToString(CultureInfo.InvariantCulture),
                    IArgumentOperation value when value.Parameter is not null => "argument:"
                        + value.Parameter.Ordinal.ToString(CultureInfo.InvariantCulture),
                    IConversionOperation value when value.OperatorMethod is not null =>
                        SymbolKey(value.OperatorMethod),
                    IBinaryOperation value when value.OperatorMethod is not null =>
                        SymbolKey(value.OperatorMethod),
                    IUnaryOperation value when value.OperatorMethod is not null =>
                        SymbolKey(value.OperatorMethod),
                    _ => string.Empty,
                };
            }

            private string LocalKey(ILocalSymbol local)
            {
                if (!localOrdinals.TryGetValue(local, out int ordinal))
                {
                    throw PracticalFailures.Protocol("normalized_local");
                }

                return "local:" + ordinal.ToString(CultureInfo.InvariantCulture);
            }

            private string SymbolKey(ISymbol? symbol)
            {
                if (symbol is null)
                {
                    return string.Empty;
                }

                if (SymbolEqualityComparer.Default.Equals(
                    symbol.ContainingAssembly,
                    Model.Compilation.Assembly))
                {
                    if (symbol is IMethodSymbol method)
                    {
                        string kind = method.MethodKind == MethodKind.Constructor
                            ? "constructor"
                            : "method";
                        string name = kind == "constructor"
                            ? method.ContainingType.Name
                            : method.MethodKind == MethodKind.PropertyGet
                                ? "get_" + method.AssociatedSymbol!.Name
                                : method.Name;
                        string owner = PracticalIdentity.SourceTypeId(
                            method.ContainingNamespace.ToDisplayString(),
                            method.ContainingType.Name);
                        string[] parameters = method.Parameters.Select(parameter =>
                            syntaxModel.NormalizeType(parameter.Type).Id).ToArray();
                        string result = kind == "constructor"
                            ? owner
                            : syntaxModel.NormalizeType(
                                method.ReturnType,
                                allowVoid: true).Id;
                        return PracticalIdentity.CallableId(
                            kind,
                            method.ContainingNamespace.ToDisplayString(),
                            owner,
                            name,
                            parameters,
                            result);
                    }

                    string ownerId = PracticalIdentity.SourceTypeId(
                        symbol.ContainingNamespace.ToDisplayString(),
                        symbol.ContainingType?.Name
                            ?? throw PracticalFailures.Protocol("normalized_symbol"));
                    return ownerId + "." + symbol.MetadataName;
                }

                return (symbol.ContainingAssembly?.Identity.Name ?? string.Empty)
                    + "|"
                    + symbol.ToDisplayString(SymbolDisplayFormat.CSharpErrorMessageFormat);
            }

            private static string Traits(IOperation operation) => operation switch
            {
                IBinaryOperation value => value.OperatorKind + "|"
                    + value.IsChecked + "|" + value.IsLifted,
                IUnaryOperation value => value.OperatorKind + "|"
                    + value.IsChecked + "|" + value.IsLifted,
                IConversionOperation value => string.Join(
                    "|",
                    value.IsChecked,
                    value.IsTryCast,
                    value.Conversion.Exists,
                    value.Conversion.IsIdentity,
                    value.Conversion.IsImplicit,
                    value.Conversion.IsNumeric,
                    value.Conversion.IsNullable,
                    value.Conversion.IsReference,
                    value.Conversion.IsUserDefined),
                IArgumentOperation value => value.ArgumentKind + "|"
                    + (value.Parameter?.Ordinal.ToString(CultureInfo.InvariantCulture) ?? ""),
                IInstanceReferenceOperation value => value.ReferenceKind.ToString(),
                IBranchOperation value => value.BranchKind.ToString(),
                ILoopOperation value => value.LoopKind.ToString(),
                IConditionalOperation value => value.IsRef.ToString(),
                IArrayCreationOperation value => value.DimensionSizes.Length.ToString(
                    CultureInfo.InvariantCulture),
                _ => string.Empty,
            };
        }
    }

    private static Utf8JsonWriter CanonicalWriter(Stream output) => new Utf8JsonWriter(
        output,
        new JsonWriterOptions
        {
            Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping,
            Indented = false,
            SkipValidation = false,
        });
}
