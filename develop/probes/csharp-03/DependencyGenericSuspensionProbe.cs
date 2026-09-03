// Disposable CSHARP-03-T01-W06 public Roslyn API probe; never a frontend.
#nullable enable

using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Threading;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Operations;
using Microsoft.CodeAnalysis.Text;

internal static class DependencyGenericSuspensionProbe
{
    private const string RawSchema = "mpk.csharp_practical.t01_w06.roslyn_exclusion_probe.raw.v0";
    private const string WorkItem = "CSHARP-03-T01-W06";
    private const string MarkerPrefix = "/*@shape:";
    private const string MarkerSuffix = "*/";

    private static readonly SymbolDisplayFormat DisplayFormat =
        SymbolDisplayFormat.CSharpErrorMessageFormat.WithMiscellaneousOptions(
            SymbolDisplayFormat.CSharpErrorMessageFormat.MiscellaneousOptions
            | SymbolDisplayMiscellaneousOptions.IncludeNullableReferenceTypeModifier
            | SymbolDisplayMiscellaneousOptions.EscapeKeywordIdentifiers);

    private sealed record ProbeCase(
        string Id,
        string Disposition,
        string Source,
        string[] ExtraReferenceIds,
        bool RunGenerator,
        bool CaptureEmit);

    private sealed record SyntheticReference(
        string Id,
        string Origin,
        string VirtualPath,
        PortableExecutableReference Reference,
        object Observation);

    private sealed record EmittedObservation(
        object? Record,
        Dictionary<string, List<object?>> Evidence);

    public static int Main(string[] args)
    {
        if (args.Length != 1)
        {
            Console.Error.Write("CSHARP_PRACTICAL_EXCLUSION_PROBE_USAGE\n");
            return 64;
        }

        try
        {
            CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;
            CultureInfo.CurrentUICulture = CultureInfo.InvariantCulture;
            ImmutableArray<MetadataReference> references = LoadReferences(args[0]);
            IReadOnlyDictionary<string, SyntheticReference> synthetic =
                BuildSyntheticReferences(references);
            List<object?> cases = Cases()
                .Select(probeCase => ObserveCase(probeCase, references, synthetic))
                .Cast<object?>()
                .ToList();
            Dictionary<string, object?> root = Obj(
                ("cases", cases),
                ("compiler", Obj(
                    ("architecture", RuntimeInformation.ProcessArchitecture.ToString()),
                    ("base_reference_count", references.Length),
                    ("language", LanguageNames.CSharp),
                    ("language_version", LanguageVersion.CSharp14.ToDisplayString()),
                    ("nullable_context", NullableContextOptions.Enable.ToString()),
                    ("roslyn_common", AssemblyIdentity(typeof(Compilation).Assembly)),
                    ("roslyn_csharp", AssemblyIdentity(typeof(CSharpCompilation).Assembly)),
                    ("runtime_version", Environment.Version.ToString()))),
                ("schema", RawSchema),
                ("synthetic_references", synthetic.Values
                    .OrderBy(value => value.Id, StringComparer.Ordinal)
                    .Select(value => value.Observation)
                    .Cast<object?>()
                    .ToList()),
                ("work_item", WorkItem));
            Console.OutputEncoding = new UTF8Encoding(false, true);
            Console.Write(JsonSerializer.Serialize(root));
            Console.Write('\n');
            return 0;
        }
        catch (ProbeFailure failure)
        {
            Console.Error.Write("CSHARP_PRACTICAL_EXCLUSION_PROBE_" + failure.Code + "\n");
            return 65;
        }
        catch (Exception failure)
        {
            Console.Error.Write(
                "CSHARP_PRACTICAL_EXCLUSION_PROBE_UNEXPECTED: "
                + failure.GetType().FullName
                + ": "
                + failure.Message
                + "\n"
                + failure.StackTrace
                + "\n");
            return 70;
        }
    }

    private static Dictionary<string, object?> AssemblyIdentity(System.Reflection.Assembly assembly)
    {
        System.Reflection.AssemblyName name = assembly.GetName();
        return Obj(
            ("culture", string.IsNullOrEmpty(name.CultureName) ? "neutral" : name.CultureName),
            ("name", name.Name),
            ("public_key_token", Convert.ToHexString(name.GetPublicKeyToken() ?? Array.Empty<byte>()).ToLowerInvariant()),
            ("version", name.Version?.ToString()));
    }

    private static ImmutableArray<MetadataReference> LoadReferences(string referenceRoot)
    {
        string root = Path.GetFullPath(referenceRoot);
        string[] paths = Directory.GetFiles(root, "*.dll", SearchOption.TopDirectoryOnly);
        Array.Sort(paths, StringComparer.Ordinal);
        if (paths.Length != 167)
        {
            throw new ProbeFailure("REFERENCE_COUNT");
        }
        ImmutableArray<MetadataReference>.Builder builder =
            ImmutableArray.CreateBuilder<MetadataReference>(paths.Length);
        foreach (string path in paths)
        {
            builder.Add(MetadataReference.CreateFromFile(path));
        }
        return builder.MoveToImmutable();
    }

    private static IReadOnlyDictionary<string, SyntheticReference> BuildSyntheticReferences(
        ImmutableArray<MetadataReference> references)
    {
        SyntheticReference[] values =
        {
            BuildSyntheticReference(
                "mpk-package",
                "package",
                "/virtual/packages/Mpk.Package/1.0.0/lib/net10.0/Mpk.Package.Dependency.dll",
                "Mpk.Package.Dependency",
                """
                using System;
                namespace Mpk.Package
                {
                    [AttributeUsage(AttributeTargets.All)]
                    public sealed class MarkerAttribute : Attribute { }
                    public interface IContract { }
                    public class Base { }
                    public static class Api { public static int Value => 7; }
                }
                """,
                references),
            BuildSyntheticReference(
                "mpk-project",
                "project",
                "/virtual/projects/Mpk.Project.Dependency/bin/Mpk.Project.Dependency.dll",
                "Mpk.Project.Dependency",
                """
                namespace Mpk.Project
                {
                    public static class Api { public static int Value => 11; }
                }
                """,
                references),
            BuildSyntheticReference(
                "ambient-project",
                "ambient",
                "/virtual/ambient/Vendor.Ambient.Dependency.dll",
                "Vendor.Ambient.Dependency",
                """
                namespace Vendor.Ambient
                {
                    public static class Api { public static int Value => 13; }
                }
                """,
                references),
        };
        return values.ToDictionary(value => value.Id, StringComparer.Ordinal);
    }

    private static SyntheticReference BuildSyntheticReference(
        string id,
        string origin,
        string virtualPath,
        string assemblyName,
        string source,
        ImmutableArray<MetadataReference> references)
    {
        CSharpParseOptions parseOptions = ParseOptions();
        SyntaxTree tree = CSharpSyntaxTree.ParseText(
            SourceText.From(source, new UTF8Encoding(false, true), SourceHashAlgorithm.Sha256),
            parseOptions,
            "/virtual/reference-source/" + id + ".cs",
            CancellationToken.None);
        CSharpCompilation compilation = CSharpCompilation.Create(
            assemblyName,
            new[] { tree },
            references,
            CompilationOptions(OutputKind.DynamicallyLinkedLibrary));
        using MemoryStream stream = new();
        var emit = compilation.Emit(stream, cancellationToken: CancellationToken.None);
        if (!emit.Success || emit.Diagnostics.Any(value => value.Severity != DiagnosticSeverity.Hidden))
        {
            throw new ProbeFailure("SYNTHETIC_REFERENCE_EMIT");
        }
        byte[] image = stream.ToArray();
        PortableExecutableReference reference = MetadataReference.CreateFromImage(
            ImmutableArray.Create(image),
            filePath: virtualPath);
        return new SyntheticReference(
            id,
            origin,
            virtualPath,
            reference,
            Obj(
                ("assembly_name", assemblyName),
                ("id", id),
                ("origin", origin),
                ("pe_sha256", Hex(SHA256.HashData(image))),
                ("pe_size_bytes", image.Length),
                ("source_sha256", Hex(SHA256.HashData(Encoding.UTF8.GetBytes(source)))),
                ("virtual_path", virtualPath)));
    }

    private static CSharpParseOptions ParseOptions()
    {
        return new CSharpParseOptions(
            languageVersion: LanguageVersion.CSharp14,
            documentationMode: DocumentationMode.None,
            kind: SourceCodeKind.Regular);
    }

    private static CSharpCompilationOptions CompilationOptions(OutputKind outputKind)
    {
        return new CSharpCompilationOptions(
            outputKind,
            platform: Platform.X64,
            optimizationLevel: OptimizationLevel.Release,
            checkOverflow: true,
            allowUnsafe: false,
            nullableContextOptions: NullableContextOptions.Enable,
            concurrentBuild: false,
            deterministic: true,
            metadataImportOptions: MetadataImportOptions.Public,
            warningLevel: 4,
            reportSuppressedDiagnostics: false);
    }

    private static object ObserveCase(
        ProbeCase probeCase,
        ImmutableArray<MetadataReference> baseReferences,
        IReadOnlyDictionary<string, SyntheticReference> synthetic)
    {
        CSharpParseOptions parseOptions = ParseOptions();
        SourceText text = SourceText.From(
            probeCase.Source,
            new UTF8Encoding(false, true),
            SourceHashAlgorithm.Sha256);
        string path = "src/" + probeCase.Id + ".cs";
        SyntaxTree selectedTree = CSharpSyntaxTree.ParseText(
            text,
            parseOptions,
            path,
            CancellationToken.None);
        ImmutableArray<MetadataReference> references = baseReferences.AddRange(
            probeCase.ExtraReferenceIds.Select(id => synthetic[id].Reference));
        CSharpCompilation compilation = CSharpCompilation.Create(
            "probe_" + probeCase.Id.Replace('-', '_'),
            new[] { selectedTree },
            references,
            CompilationOptions(OutputKind.DynamicallyLinkedLibrary));
        List<object?> generatedSources = new();
        List<Diagnostic> generatorDiagnostics = new();
        if (probeCase.RunGenerator)
        {
            GeneratorDriver driver = CSharpGeneratorDriver.Create(
                new ISourceGenerator[] { new FixtureGenerator() },
                parseOptions: parseOptions);
            driver = driver.RunGeneratorsAndUpdateCompilation(
                compilation,
                out Compilation updated,
                out ImmutableArray<Diagnostic> generatedDiagnostics,
                CancellationToken.None);
            compilation = (CSharpCompilation)updated;
            generatorDiagnostics.AddRange(generatedDiagnostics);
            GeneratorDriverRunResult run = driver.GetRunResult();
            foreach (GeneratedSourceResult generated in run.Results
                         .SelectMany(result => result.GeneratedSources)
                         .OrderBy(result => result.HintName, StringComparer.Ordinal))
            {
                string generatedText = generated.SourceText.ToString();
                generatedSources.Add(Obj(
                    ("hint_name", generated.HintName),
                    ("path", generated.SyntaxTree.FilePath),
                    ("source", generatedText),
                    ("source_utf8_sha256", Hex(SHA256.HashData(Encoding.UTF8.GetBytes(generatedText))))));
            }
            if (generatedSources.Count == 0)
            {
                throw new ProbeFailure("GENERATOR_OUTPUT");
            }
        }

        Diagnostic[] diagnostics = compilation.GetDiagnostics(CancellationToken.None)
            .Concat(generatorDiagnostics)
            .OrderBy(DiagnosticSortKey, StringComparer.Ordinal)
            .ToArray();
        bool hasErrors = diagnostics.Any(value => value.Severity == DiagnosticSeverity.Error);
        if (probeCase.Disposition == "admitted_exception_observation" && diagnostics.Length != 0)
        {
            throw new ProbeFailure("ADMITTED_EXCEPTION_DIAGNOSTIC");
        }
        SemanticModel model = compilation.GetSemanticModel(selectedTree, ignoreAccessibility: false);
        SyntaxNode selectedRoot = selectedTree.GetRoot(CancellationToken.None);
        List<IOperation> operationRoots = OperationRoots(compilation);
        EmittedObservation emitted = probeCase.CaptureEmit
            ? ObserveEmittedMetadata(compilation, hasErrors)
            : new EmittedObservation(null, new Dictionary<string, List<object?>>(StringComparer.Ordinal));
        if (generatedSources.Count != 0)
        {
            emitted.Evidence.Add("generated_source", generatedSources);
        }
        List<object?> targets = ObserveTargets(
            probeCase.Source,
            selectedRoot,
            model,
            probeCase.Disposition,
            emitted.Evidence);
        if (targets.Count == 0 || targets.Count != probeCase.Source.Split(MarkerPrefix).Length - 1)
        {
            throw new ProbeFailure("TARGET_COUNT");
        }
        return Obj(
            ("compiler_outcome", hasErrors ? "error" : "success"),
            ("diagnostics", diagnostics.Select(ObserveDiagnostic).Cast<object?>().ToList()),
            ("disposition", probeCase.Disposition),
            ("emitted_metadata", emitted.Record),
            ("extra_references", probeCase.ExtraReferenceIds
                .Select(id => synthetic[id].Observation)
                .Cast<object?>()
                .ToList()),
            ("generated_sources", generatedSources),
            ("id", probeCase.Id),
            ("operation_roots", operationRoots.Select(ObserveOperation).Cast<object?>().ToList()),
            ("source", probeCase.Source),
            ("source_order", targets.Cast<Dictionary<string, object?>>()
                .Select((target, index) => Obj(
                    ("shape_id", target["shape_id"]),
                    ("source_ordinal", index),
                    ("start", ((Dictionary<string, object?>)target["marker_span"]!)["start"])))
                .Cast<object?>()
                .ToList()),
            ("source_utf8_sha256", Hex(SHA256.HashData(Encoding.UTF8.GetBytes(probeCase.Source)))),
            ("syntax", ObserveSyntax(selectedRoot)),
            ("targets", targets));
    }

    private static string DiagnosticSortKey(Diagnostic diagnostic)
    {
        FileLinePositionSpan line = diagnostic.Location.GetLineSpan();
        return string.Join(
            "|",
            line.Path,
            diagnostic.Location.SourceSpan.Start.ToString("D10", CultureInfo.InvariantCulture),
            diagnostic.Location.SourceSpan.Length.ToString("D10", CultureInfo.InvariantCulture),
            diagnostic.Severity,
            diagnostic.Id);
    }

    private static object ObserveDiagnostic(Diagnostic diagnostic)
    {
        Location location = diagnostic.Location;
        return Obj(
            ("id", diagnostic.Id),
            ("is_suppressed", diagnostic.IsSuppressed),
            ("location_kind", location.Kind.ToString()),
            ("severity", diagnostic.Severity.ToString()),
            ("span", location.IsInSource ? Span(location.SourceSpan) : null),
            ("warning_level", diagnostic.WarningLevel));
    }

    private static List<object?> ObserveSyntax(SyntaxNode root)
    {
        List<object?> result = new();
        int ordinal = 0;
        foreach (SyntaxNodeOrToken item in root.DescendantNodesAndTokensAndSelf(descendIntoTrivia: true))
        {
            if (item.IsNode)
            {
                SyntaxNode node = item.AsNode()!;
                result.Add(Obj(
                    ("contains_diagnostics", node.ContainsDiagnostics),
                    ("full_span", Span(node.FullSpan)),
                    ("is_missing", node.IsMissing),
                    ("item", "node"),
                    ("kind", node.Kind().ToString()),
                    ("ordinal", ordinal++),
                    ("raw_kind", node.RawKind),
                    ("span", Span(node.Span))));
            }
            else
            {
                SyntaxToken token = item.AsToken();
                result.Add(Obj(
                    ("contains_diagnostics", token.ContainsDiagnostics),
                    ("full_span", Span(token.FullSpan)),
                    ("is_missing", token.IsMissing),
                    ("item", "token"),
                    ("kind", token.Kind().ToString()),
                    ("ordinal", ordinal++),
                    ("raw_kind", token.RawKind),
                    ("span", Span(token.Span)),
                    ("text", token.Text),
                    ("value_text", token.ValueText)));
            }
        }
        return result;
    }

    private static List<IOperation> OperationRoots(CSharpCompilation compilation)
    {
        Dictionary<string, IOperation> roots = new(StringComparer.Ordinal);
        foreach (SyntaxTree tree in compilation.SyntaxTrees.OrderBy(value => value.FilePath, StringComparer.Ordinal))
        {
            SemanticModel model = compilation.GetSemanticModel(tree, ignoreAccessibility: false);
            SyntaxNode root = tree.GetRoot(CancellationToken.None);
            foreach (SyntaxNode node in root.DescendantNodesAndSelf(descendIntoTrivia: true))
            {
                IOperation? operation = model.GetOperation(node, CancellationToken.None);
                if (operation is null || operation.Parent is not null)
                {
                    continue;
                }
                string key = tree.FilePath + ":" + operation.Kind + ":"
                    + operation.Syntax.Span.Start.ToString(CultureInfo.InvariantCulture) + ":"
                    + operation.Syntax.Span.Length.ToString(CultureInfo.InvariantCulture);
                roots.TryAdd(key, operation);
            }
        }
        return roots.Values
            .OrderBy(value => value.Syntax.SyntaxTree.FilePath, StringComparer.Ordinal)
            .ThenBy(value => value.Syntax.Span.Start)
            .ThenBy(value => value.Syntax.Span.Length)
            .ThenBy(value => value.Kind.ToString(), StringComparer.Ordinal)
            .ToList();
    }

    private static object ObserveOperation(IOperation operation)
    {
        return Obj(
            ("children", operation.ChildOperations.Select(ObserveOperation).Cast<object?>().ToList()),
            ("constant", ObserveConstant(operation.ConstantValue)),
            ("details", OperationDetails(operation)),
            ("is_implicit", operation.IsImplicit),
            ("kind", operation.Kind.ToString()),
            ("language", operation.Language),
            ("source_path", operation.Syntax.SyntaxTree.FilePath),
            ("span", Span(operation.Syntax.Span)),
            ("syntax_kind", operation.Syntax.Kind().ToString()),
            ("type", TypeIdentity(operation.Type)));
    }

    private static object OperationDetails(IOperation operation)
    {
        return operation switch
        {
            IInvocationOperation value => Obj(("target", SymbolIdentity(value.TargetMethod))),
            IObjectCreationOperation value => Obj(("constructor", SymbolIdentity(value.Constructor))),
            IPropertyReferenceOperation value => Obj(("property", SymbolIdentity(value.Property))),
            IFieldReferenceOperation value => Obj(("field", SymbolIdentity(value.Field))),
            IConversionOperation value => Obj(
                ("conversion", ConversionIdentity(value.Conversion)),
                ("operator_method", SymbolIdentity(value.OperatorMethod))),
            IAwaitOperation value => Obj(("awaited", TypeIdentity(value.Operation.Type))),
            IReturnOperation value => Obj(("has_value", value.ReturnedValue is not null)),
            IAnonymousFunctionOperation value => Obj(("symbol", SymbolIdentity(value.Symbol))),
            ILocalFunctionOperation value => Obj(("symbol", SymbolIdentity(value.Symbol))),
            IAttributeOperation value => Obj(("operation", ObserveOperation(value.Operation))),
            _ => Obj(),
        };
    }

    private static object ConversionIdentity(CommonConversion conversion)
    {
        return Obj(
            ("exists", conversion.Exists),
            ("is_identity", conversion.IsIdentity),
            ("is_implicit", conversion.IsImplicit),
            ("is_nullable", conversion.IsNullable),
            ("is_numeric", conversion.IsNumeric),
            ("is_reference", conversion.IsReference),
            ("is_user_defined", conversion.IsUserDefined),
            ("method", SymbolIdentity(conversion.MethodSymbol)));
    }

    private static object? ObserveConstant(Optional<object?> constant)
    {
        if (!constant.HasValue)
        {
            return null;
        }
        object? value = constant.Value;
        return Obj(
            ("runtime_type", value?.GetType().FullName),
            ("value", value switch
            {
                null => null,
                char character => ((int)character).ToString(CultureInfo.InvariantCulture),
                IFormattable formattable => formattable.ToString(null, CultureInfo.InvariantCulture),
                _ => value.ToString(),
            }));
    }

    private static List<object?> ObserveTargets(
        string source,
        SyntaxNode root,
        SemanticModel model,
        string disposition,
        IReadOnlyDictionary<string, List<object?>> emittedEvidence)
    {
        List<object?> result = new();
        HashSet<string> ids = new(StringComparer.Ordinal);
        int search = 0;
        int ordinal = 0;
        while (true)
        {
            int start = source.IndexOf(MarkerPrefix, search, StringComparison.Ordinal);
            if (start < 0)
            {
                break;
            }
            int nameStart = start + MarkerPrefix.Length;
            int end = source.IndexOf(MarkerSuffix, nameStart, StringComparison.Ordinal);
            if (end < 0)
            {
                throw new ProbeFailure("MARKER_END");
            }
            string id = source[nameStart..end];
            if (id.Length == 0 || !ids.Add(id))
            {
                throw new ProbeFailure("MARKER_ID");
            }
            int after = end + MarkerSuffix.Length;
            SyntaxToken nextToken = root.DescendantTokens(descendIntoTrivia: true)
                .FirstOrDefault(token => !token.IsMissing && token.SpanStart >= after);
            if (nextToken.RawKind == 0)
            {
                throw new ProbeFailure("MARKER_TOKEN");
            }
            int targetStart = nextToken.SpanStart;
            List<SyntaxNode> candidates = root.DescendantNodesAndSelf(descendIntoTrivia: true)
                .Where(node => node != root && node.SpanStart == targetStart && !node.IsMissing)
                .ToList();
            SyntaxNode? target = candidates
                .Where(node => PreferredTarget(id, node))
                .OrderByDescending(node => node.Span.Length)
                .FirstOrDefault();
            target ??= candidates
                .OrderByDescending(node => node.Span.Length)
                .FirstOrDefault();
            if (target is null)
            {
                throw new ProbeFailure("MARKER_TARGET");
            }
            SyntaxNode bindingNode = BindingNode(target);
            ISymbol? declared = model.GetDeclaredSymbol(target, CancellationToken.None);
            SymbolInfo symbolInfo = model.GetSymbolInfo(bindingNode, CancellationToken.None);
            TypeInfo typeInfo = model.GetTypeInfo(bindingNode, CancellationToken.None);
            IOperation? operation = model.GetOperation(target, CancellationToken.None)
                ?? (ReferenceEquals(bindingNode, target)
                    ? null
                    : model.GetOperation(bindingNode, CancellationToken.None));
            string family = FamilyForShape(id);
            string outcome = id.StartsWith("exception.", StringComparison.Ordinal)
                ? "admitted_exception"
                : "rejected";
            if ((disposition == "admitted_exception_observation") != (outcome == "admitted_exception"))
            {
                throw new ProbeFailure("TARGET_DISPOSITION");
            }
            string? evidenceKey = EvidenceKey(id);
            List<object?> evidence = evidenceKey is not null
                && emittedEvidence.TryGetValue(evidenceKey, out List<object?>? found)
                    ? found
                    : new List<object?>();
            if (evidenceKey is not null && evidence.Count == 0)
            {
                throw new ProbeFailure("EMITTED_EVIDENCE");
            }
            result.Add(Obj(
                ("candidate_reason", symbolInfo.CandidateReason.ToString()),
                ("candidate_symbols", symbolInfo.CandidateSymbols.Select(SymbolIdentity).Cast<object?>().ToList()),
                ("converted_type", TypeIdentity(typeInfo.ConvertedType)),
                ("declared_symbol", SymbolIdentity(declared)),
                ("emitted_evidence", evidence),
                ("enclosing_symbol", SymbolIdentity(model.GetEnclosingSymbol(target.SpanStart, CancellationToken.None))),
                ("family", family),
                ("generic_facts", GenericFacts(
                    target,
                    declared,
                    symbolInfo.Symbol,
                    typeInfo.Type,
                    typeInfo.ConvertedType)),
                ("marker_span", Span(new TextSpan(start, after - start))),
                ("operation", operation is null ? null : ObserveOperation(operation)),
                ("profile_outcome", outcome),
                ("shape_id", id),
                ("source_ordinal", ordinal++),
                ("symbol", SymbolIdentity(symbolInfo.Symbol)),
                ("syntax", Obj(
                    ("contains_diagnostics", target.ContainsDiagnostics),
                    ("full_span", Span(target.FullSpan)),
                    ("is_missing", target.IsMissing),
                    ("kind", target.Kind().ToString()),
                    ("raw_kind", target.RawKind),
                    ("span", Span(target.Span)))),
                ("type", TypeIdentity(typeInfo.Type))));
            search = after;
        }
        return result;
    }

    private static SyntaxNode BindingNode(SyntaxNode target)
    {
        return target switch
        {
            UsingDirectiveSyntax value when value.Name is not null => value.Name,
            BaseTypeSyntax value => value.Type,
            AttributeTargetSpecifierSyntax value => value.Identifier.Parent ?? value,
            _ => target,
        };
    }

    private static bool PreferredTarget(string id, SyntaxNode node)
    {
        if (id.StartsWith("near_miss.attribute.", StringComparison.Ordinal)
            || id == "near_miss.dependency.attribute.mpk")
        {
            return node is AttributeSyntax;
        }
        if (id == "near_miss.dependency.namespace.import")
        {
            return node is UsingDirectiveSyntax;
        }
        if (id == "near_miss.dependency.namespace.spoof")
        {
            return node is BaseNamespaceDeclarationSyntax;
        }
        if (id is "near_miss.dependency.interface.mpk" or "near_miss.dependency.base_type.mpk")
        {
            return node is BaseTypeSyntax;
        }
        if (id.StartsWith("near_miss.generic.declaration.class", StringComparison.Ordinal))
        {
            return node is ClassDeclarationSyntax;
        }
        if (id.StartsWith("near_miss.generic.declaration.struct", StringComparison.Ordinal))
        {
            return node is StructDeclarationSyntax;
        }
        if (id.StartsWith("near_miss.generic.declaration.interface", StringComparison.Ordinal))
        {
            return node is InterfaceDeclarationSyntax;
        }
        if (id.StartsWith("near_miss.generic.declaration.delegate", StringComparison.Ordinal))
        {
            return node is DelegateDeclarationSyntax;
        }
        if (id.StartsWith("near_miss.generic.method.declaration", StringComparison.Ordinal))
        {
            return node is MethodDeclarationSyntax;
        }
        if (id.StartsWith("near_miss.generic.type_parameter.", StringComparison.Ordinal)
            || id.StartsWith("near_miss.generic.variance.", StringComparison.Ordinal)
            || id == "near_miss.attribute.source.type_parameter")
        {
            return node is TypeParameterSyntax;
        }
        if (id.StartsWith("near_miss.generic.constraint.", StringComparison.Ordinal))
        {
            return node is TypeParameterConstraintSyntax;
        }
        if (id.Contains(".declaration.", StringComparison.Ordinal)
            || id.Contains(".state_machine", StringComparison.Ordinal))
        {
            return node is MethodDeclarationSyntax or LocalFunctionStatementSyntax
                or ParenthesizedLambdaExpressionSyntax;
        }
        if (id.StartsWith("near_miss.iterator.yield.", StringComparison.Ordinal))
        {
            return node is YieldStatementSyntax;
        }
        if (id.StartsWith("near_miss.async.await.", StringComparison.Ordinal)
            || id.StartsWith("near_miss.iterator.async.await", StringComparison.Ordinal))
        {
            return node is AwaitExpressionSyntax or ForEachStatementSyntax;
        }
        if (id == "exception.compiler_metadata.required_attributes")
        {
            return node is PropertyDeclarationSyntax;
        }
        if (id == "exception.compiler_metadata.init_modreq")
        {
            return node is AccessorDeclarationSyntax;
        }
        if (id == "exception.nullable.shorthand.local_type")
        {
            return node is NullableTypeSyntax;
        }
        return false;
    }

    private static string FamilyForShape(string id)
    {
        (string Prefix, string Family)[] prefixes =
        {
            ("exception.compiler_metadata.", "exception.compiler_metadata"),
            ("exception.incidental.", "exception.incidental_metadata"),
            ("exception.nullable.", "exception.nullable_shorthand"),
            ("exception.array.", "exception.array_non_generic"),
            ("near_miss.dependency.generated_source.", "dependency.generated_source"),
            ("near_miss.dependency.namespace.", "dependency.namespace"),
            ("near_miss.dependency.package.", "dependency.package"),
            ("near_miss.dependency.assembly.", "dependency.assembly"),
            ("near_miss.dependency.attribute.", "dependency.attribute"),
            ("near_miss.dependency.interface.", "dependency.interface"),
            ("near_miss.dependency.base_type.", "dependency.base_type"),
            ("near_miss.dependency.project.", "dependency.project"),
            ("near_miss.dependency.ambient.", "dependency.ambient"),
            ("near_miss.attribute.compiler_marker.", "attribute.compiler_marker_spelling"),
            ("near_miss.attribute.source.", "attribute.source_written"),
            ("near_miss.generic.declaration.", "generic.declaration"),
            ("near_miss.generic.method.", "generic.method"),
            ("near_miss.generic.type_parameter.", "generic.type_parameter"),
            ("near_miss.generic.constraint.", "generic.constraint"),
            ("near_miss.generic.variance.", "generic.variance"),
            ("near_miss.generic.explicit_call.", "generic.explicit_call"),
            ("near_miss.generic.inferred_call.", "generic.inferred_call"),
            ("near_miss.generic.closed_use.", "generic.closed_use"),
            ("near_miss.generic.framework_type.", "generic.framework_type"),
            ("near_miss.generic.open_type.", "generic.open_type"),
            ("near_miss.generic.explicit_nullable.", "generic.explicit_nullable"),
            ("near_miss.generic.unsupported_nullable.", "generic.unsupported_nullable"),
            ("near_miss.generic.transitive_metadata.", "generic.transitive_metadata"),
            ("near_miss.iterator.async.", "iterator.async"),
            ("near_miss.iterator.declaration.", "iterator.declaration"),
            ("near_miss.iterator.yield.", "iterator.yield"),
            ("near_miss.iterator.protocol.", "iterator.protocol"),
            ("near_miss.iterator.state_machine", "iterator.state_machine"),
            ("near_miss.async.declaration.", "async.declaration"),
            ("near_miss.async.await.", "async.await"),
            ("near_miss.async.task.", "async.task"),
            ("near_miss.async.value_task.", "async.value_task"),
            ("near_miss.async.awaiter.", "async.awaiter"),
            ("near_miss.async.cancellation.", "async.cancellation"),
            ("near_miss.async.parallel.", "async.parallel"),
            ("near_miss.async.state_machine", "async.state_machine"),
        };
        foreach ((string prefix, string family) in prefixes)
        {
            if (id.StartsWith(prefix, StringComparison.Ordinal))
            {
                return family;
            }
        }
        throw new ProbeFailure("SHAPE_FAMILY");
    }

    private static string? EvidenceKey(string id)
    {
        if (id == "exception.compiler_metadata.required_attributes")
        {
            return "required_metadata";
        }
        if (id == "exception.compiler_metadata.init_modreq")
        {
            return "init_metadata";
        }
        if (id.Contains(".state_machine", StringComparison.Ordinal))
        {
            return "state_machine";
        }
        if (id.StartsWith("near_miss.dependency.generated_source.", StringComparison.Ordinal))
        {
            return "generated_source";
        }
        if (id.StartsWith("near_miss.dependency.", StringComparison.Ordinal)
            && !id.StartsWith("near_miss.dependency.namespace.", StringComparison.Ordinal))
        {
            return "assembly_references";
        }
        return null;
    }

    private static object GenericFacts(
        SyntaxNode target,
        ISymbol? declared,
        ISymbol? selected,
        ITypeSymbol? type,
        ITypeSymbol? convertedType)
    {
        ISymbol? symbol = declared ?? selected;
        INamedTypeSymbol? namedType = new INamedTypeSymbol?[]
        {
            type as INamedTypeSymbol,
            convertedType as INamedTypeSymbol,
            symbol as INamedTypeSymbol,
            (symbol as IMethodSymbol)?.ReturnType as INamedTypeSymbol,
        }.FirstOrDefault(value =>
            value?.OriginalDefinition.SpecialType == SpecialType.System_Nullable_T);
        bool isNullableValue = namedType is not null;
        return Obj(
            ("constructed_nullable_value_type", isNullableValue),
            ("immediate_specialization", isNullableValue
                ? Obj(
                    ("payload", TypeSummary(namedType!.TypeArguments[0])),
                    ("residual_type_parameter", namedType.TypeArguments[0].TypeKind == TypeKind.TypeParameter),
                    ("shape", "option"))
                : null),
            ("source_contains_generic_name", target.DescendantNodesAndSelf().Any(node => node is GenericNameSyntax)),
            ("source_contains_nullable_shorthand", target.DescendantNodesAndSelf().Any(node => node is NullableTypeSyntax)),
            ("source_contains_type_parameter", target.DescendantNodesAndSelf().Any(node => node is TypeParameterSyntax)),
            ("symbol_arity", symbol switch
            {
                INamedTypeSymbol value => value.Arity,
                IMethodSymbol value => value.Arity,
                _ => 0,
            }),
            ("symbol_is_generic", symbol switch
            {
                INamedTypeSymbol value => value.IsGenericType,
                IMethodSymbol value => value.IsGenericMethod,
                _ => false,
            }),
            ("type_arguments", symbol switch
            {
                INamedTypeSymbol value => value.TypeArguments.Select(TypeSummary).Cast<object?>().ToList(),
                IMethodSymbol value => value.TypeArguments.Select(TypeSummary).Cast<object?>().ToList(),
                _ => new List<object?>(),
            }),
            ("type_parameters", symbol switch
            {
                INamedTypeSymbol value => value.TypeParameters.Select(TypeParameterIdentity).Cast<object?>().ToList(),
                IMethodSymbol value => value.TypeParameters.Select(TypeParameterIdentity).Cast<object?>().ToList(),
                ITypeParameterSymbol value => new List<object?> { TypeParameterIdentity(value) },
                _ => new List<object?>(),
            }));
    }

    private static object TypeParameterIdentity(ITypeParameterSymbol value)
    {
        return Obj(
            ("constraint_types", value.ConstraintTypes.Select(TypeSummary).Cast<object?>().ToList()),
            ("has_constructor_constraint", value.HasConstructorConstraint),
            ("has_notnull_constraint", value.HasNotNullConstraint),
            ("has_reference_type_constraint", value.HasReferenceTypeConstraint),
            ("has_unmanaged_type_constraint", value.HasUnmanagedTypeConstraint),
            ("has_value_type_constraint", value.HasValueTypeConstraint),
            ("name", value.Name),
            ("nullable_annotation", value.ReferenceTypeConstraintNullableAnnotation.ToString()),
            ("ordinal", value.Ordinal),
            ("variance", value.Variance.ToString()));
    }

    private static object? TypeIdentity(ITypeSymbol? type)
    {
        if (type is null)
        {
            return null;
        }
        return Obj(
            ("arity", type is INamedTypeSymbol named ? named.Arity : 0),
            ("base_type", type.BaseType is null ? null : TypeSummary(type.BaseType)),
            ("containing_assembly", type.ContainingAssembly?.Identity.ToString()),
            ("display", type.ToDisplayString(DisplayFormat)),
            ("interfaces", type.AllInterfaces
                .OrderBy(value => value.ToDisplayString(DisplayFormat), StringComparer.Ordinal)
                .Select(TypeSummary)
                .Cast<object?>()
                .ToList()),
            ("is_reference_type", type.IsReferenceType),
            ("is_value_type", type.IsValueType),
            ("metadata_name", type.MetadataName),
            ("nullable_annotation", type.NullableAnnotation.ToString()),
            ("original_definition", type.OriginalDefinition.ToDisplayString(DisplayFormat)),
            ("special_type", type.SpecialType.ToString()),
            ("type_arguments", type is INamedTypeSymbol value
                ? value.TypeArguments.Select(TypeSummary).Cast<object?>().ToList()
                : new List<object?>()),
            ("type_kind", type.TypeKind.ToString()));
    }

    private static object TypeSummary(ITypeSymbol type)
    {
        return Obj(
            ("containing_assembly", type.ContainingAssembly?.Identity.ToString()),
            ("display", type.ToDisplayString(DisplayFormat)),
            ("metadata_name", type.MetadataName),
            ("nullable_annotation", type.NullableAnnotation.ToString()),
            ("original_definition", type.OriginalDefinition.ToDisplayString(DisplayFormat)),
            ("special_type", type.SpecialType.ToString()),
            ("type_kind", type.TypeKind.ToString()));
    }

    private static object? SymbolIdentity(ISymbol? symbol)
    {
        if (symbol is null)
        {
            return null;
        }
        return Obj(
            ("arity", symbol switch
            {
                INamedTypeSymbol value => value.Arity,
                IMethodSymbol value => value.Arity,
                _ => 0,
            }),
            ("attributes", ObserveAttributes(symbol.GetAttributes())),
            ("containing_assembly", symbol.ContainingAssembly?.Identity.ToString()),
            ("display", symbol.ToDisplayString(DisplayFormat)),
            ("is_implicit", symbol.IsImplicitlyDeclared),
            ("kind", symbol.Kind.ToString()),
            ("locations", symbol.Locations
                .OrderBy(value => value.SourceTree?.FilePath, StringComparer.Ordinal)
                .ThenBy(value => value.SourceSpan.Start)
                .Select(ObserveLocation)
                .Cast<object?>()
                .ToList()),
            ("metadata_name", symbol.MetadataName),
            ("original_definition", symbol.OriginalDefinition.ToDisplayString(DisplayFormat)),
            ("type_arguments", symbol switch
            {
                INamedTypeSymbol value => value.TypeArguments.Select(TypeSummary).Cast<object?>().ToList(),
                IMethodSymbol value => value.TypeArguments.Select(TypeSummary).Cast<object?>().ToList(),
                _ => new List<object?>(),
            }),
            ("type_parameters", symbol switch
            {
                INamedTypeSymbol value => value.TypeParameters.Select(TypeParameterIdentity).Cast<object?>().ToList(),
                IMethodSymbol value => value.TypeParameters.Select(TypeParameterIdentity).Cast<object?>().ToList(),
                ITypeParameterSymbol value => new List<object?> { TypeParameterIdentity(value) },
                _ => new List<object?>(),
            }));
    }

    private static object ObserveLocation(Location location)
    {
        string? path = location.SourceTree?.FilePath;
        return Obj(
            ("kind", location.Kind.ToString()),
            ("origin", path is null ? "metadata" : path.StartsWith("src/", StringComparison.Ordinal) ? "selected" : "generated"),
            ("path", path),
            ("span", location.IsInSource ? Span(location.SourceSpan) : null));
    }

    private static List<object?> ObserveAttributes(ImmutableArray<AttributeData> attributes)
    {
        return attributes
            .OrderBy(value => value.AttributeClass?.ToDisplayString(DisplayFormat), StringComparer.Ordinal)
            .ThenBy(value => value.ApplicationSyntaxReference?.Span.Start ?? int.MaxValue)
            .Select(value => Obj(
                ("attribute_class", value.AttributeClass?.ToDisplayString(DisplayFormat)),
                ("constructor", value.AttributeConstructor?.ToDisplayString(DisplayFormat)),
                ("is_source", value.ApplicationSyntaxReference is not null),
                ("source_path", value.ApplicationSyntaxReference?.SyntaxTree.FilePath),
                ("source_span", value.ApplicationSyntaxReference is null
                    ? null
                    : Span(value.ApplicationSyntaxReference.Span))))
            .Cast<object?>()
            .ToList();
    }

    private static EmittedObservation ObserveEmittedMetadata(
        CSharpCompilation compilation,
        bool hasErrors)
    {
        Dictionary<string, List<object?>> evidence = new(StringComparer.Ordinal);
        if (hasErrors)
        {
            return new EmittedObservation(null, evidence);
        }
        using MemoryStream stream = new();
        var emit = compilation.Emit(stream, cancellationToken: CancellationToken.None);
        if (!emit.Success)
        {
            throw new ProbeFailure("EMIT");
        }
        byte[] image = stream.ToArray();
        using MemoryStream metadataStream = new(image, writable: false);
        using PEReader pe = new(metadataStream, PEStreamOptions.LeaveOpen);
        MetadataReader reader = pe.GetMetadataReader();
        Dictionary<int, string> parentNames = MetadataParentNames(reader);

        List<object?> assemblyReferences = reader.AssemblyReferences
            .Select(handle =>
            {
                AssemblyReference value = reader.GetAssemblyReference(handle);
                return Obj(
                    ("culture", value.Culture.IsNil ? "neutral" : reader.GetString(value.Culture)),
                    ("name", reader.GetString(value.Name)),
                    ("public_key_or_token", Hex(reader.GetBlobBytes(value.PublicKeyOrToken))),
                    ("token", MetadataTokens.GetToken(handle)),
                    ("version", value.Version.ToString()));
            })
            .Cast<object?>()
            .ToList();
        List<object?> typeReferences = reader.TypeReferences
            .Select(handle =>
            {
                TypeReference value = reader.GetTypeReference(handle);
                return Obj(
                    ("identity", MetadataTypeReferenceName(reader, handle)),
                    ("resolution_scope_kind", value.ResolutionScope.Kind.ToString()),
                    ("resolution_scope_token", MetadataToken(value.ResolutionScope)),
                    ("token", MetadataTokens.GetToken(handle)));
            })
            .Cast<object?>()
            .ToList();
        List<object?> memberReferences = reader.MemberReferences
            .Select(handle =>
            {
                MemberReference value = reader.GetMemberReference(handle);
                return Obj(
                    ("name", reader.GetString(value.Name)),
                    ("parent", MetadataEntityName(reader, value.Parent, parentNames)),
                    ("parent_kind", value.Parent.Kind.ToString()),
                    ("signature", Hex(reader.GetBlobBytes(value.Signature))),
                    ("token", MetadataTokens.GetToken(handle)));
            })
            .Cast<object?>()
            .ToList();
        List<object?> typeDefinitions = reader.TypeDefinitions
            .Select(handle =>
            {
                TypeDefinition value = reader.GetTypeDefinition(handle);
                return Obj(
                    ("attributes", value.Attributes.ToString()),
                    ("base_type", MetadataEntityName(reader, value.BaseType, parentNames)),
                    ("fields", value.GetFields().Select(fieldHandle =>
                    {
                        FieldDefinition field = reader.GetFieldDefinition(fieldHandle);
                        return Obj(
                            ("attributes", field.Attributes.ToString()),
                            ("name", reader.GetString(field.Name)),
                            ("signature", Hex(reader.GetBlobBytes(field.Signature))),
                            ("token", MetadataTokens.GetToken(fieldHandle)));
                    }).Cast<object?>().ToList()),
                    ("methods", value.GetMethods().Select(methodHandle =>
                    {
                        MethodDefinition method = reader.GetMethodDefinition(methodHandle);
                        return Obj(
                            ("attributes", method.Attributes.ToString()),
                            ("impl_attributes", method.ImplAttributes.ToString()),
                            ("name", reader.GetString(method.Name)),
                            ("signature", Hex(reader.GetBlobBytes(method.Signature))),
                            ("token", MetadataTokens.GetToken(methodHandle)));
                    }).Cast<object?>().ToList()),
                    ("name", reader.GetString(value.Name)),
                    ("namespace", reader.GetString(value.Namespace)),
                    ("token", MetadataTokens.GetToken(handle)));
            })
            .Cast<object?>()
            .ToList();
        List<object?> customAttributes = reader.CustomAttributes
            .Select(handle =>
            {
                CustomAttribute value = reader.GetCustomAttribute(handle);
                return Obj(
                    ("constructor", MetadataEntityName(reader, value.Constructor, parentNames)),
                    ("constructor_kind", value.Constructor.Kind.ToString()),
                    ("parent", MetadataEntityName(reader, value.Parent, parentNames)),
                    ("parent_kind", value.Parent.Kind.ToString()),
                    ("token", MetadataTokens.GetToken(handle)),
                    ("value", Hex(reader.GetBlobBytes(value.Value))));
            })
            .Cast<object?>()
            .ToList();
        evidence.Add("assembly_references", assemblyReferences);
        evidence.Add(
            "required_metadata",
            customAttributes.Cast<Dictionary<string, object?>>()
                .Where(value => ((string)value["constructor"]!).Contains("RequiredMemberAttribute", StringComparison.Ordinal)
                    || ((string)value["constructor"]!).Contains("CompilerFeatureRequiredAttribute", StringComparison.Ordinal))
                .Cast<object?>()
                .ToList());
        evidence.Add(
            "init_metadata",
            typeReferences.Cast<Dictionary<string, object?>>()
                .Where(value => ((string)value["identity"]!).EndsWith(".IsExternalInit", StringComparison.Ordinal))
                .Concat(memberReferences.Cast<Dictionary<string, object?>>()
                    .Where(value => ((string)value["name"]!).StartsWith("set_", StringComparison.Ordinal)))
                .Cast<object?>()
                .ToList());
        evidence.Add(
            "state_machine",
            typeDefinitions.Cast<Dictionary<string, object?>>()
                .Where(value => ((string)value["name"]!).Contains("d__", StringComparison.Ordinal))
                .Concat(customAttributes.Cast<Dictionary<string, object?>>()
                    .Where(value => ((string)value["constructor"]!).Contains("StateMachineAttribute", StringComparison.Ordinal)))
                .Cast<object?>()
                .ToList());
        object record = Obj(
            ("assembly_references", assemblyReferences),
            ("custom_attributes", customAttributes),
            ("emit_diagnostics", emit.Diagnostics
                .OrderBy(DiagnosticSortKey, StringComparer.Ordinal)
                .Select(ObserveDiagnostic)
                .Cast<object?>()
                .ToList()),
            ("member_references", memberReferences),
            ("pe_sha256", Hex(SHA256.HashData(image))),
            ("pe_size_bytes", image.Length),
            ("type_definitions", typeDefinitions),
            ("type_references", typeReferences));
        return new EmittedObservation(record, evidence);
    }

    private static Dictionary<int, string> MetadataParentNames(MetadataReader reader)
    {
        Dictionary<int, string> names = new();
        foreach (TypeDefinitionHandle handle in reader.TypeDefinitions)
        {
            TypeDefinition type = reader.GetTypeDefinition(handle);
            string typeName = MetadataTypeDefinitionName(reader, handle);
            names.Add(MetadataTokens.GetToken(handle), typeName);
            foreach (FieldDefinitionHandle fieldHandle in type.GetFields())
            {
                names.Add(
                    MetadataTokens.GetToken(fieldHandle),
                    typeName + "." + reader.GetString(reader.GetFieldDefinition(fieldHandle).Name));
            }
            foreach (MethodDefinitionHandle methodHandle in type.GetMethods())
            {
                names.Add(
                    MetadataTokens.GetToken(methodHandle),
                    typeName + "." + reader.GetString(reader.GetMethodDefinition(methodHandle).Name));
            }
            foreach (PropertyDefinitionHandle propertyHandle in type.GetProperties())
            {
                names.Add(
                    MetadataTokens.GetToken(propertyHandle),
                    typeName + "." + reader.GetString(reader.GetPropertyDefinition(propertyHandle).Name));
            }
            foreach (EventDefinitionHandle eventHandle in type.GetEvents())
            {
                names.Add(
                    MetadataTokens.GetToken(eventHandle),
                    typeName + "." + reader.GetString(reader.GetEventDefinition(eventHandle).Name));
            }
        }
        return names;
    }

    private static string MetadataEntityName(
        MetadataReader reader,
        EntityHandle handle,
        IReadOnlyDictionary<int, string> parentNames)
    {
        if (handle.IsNil)
        {
            return "nil";
        }
        int token = MetadataToken(handle);
        if (parentNames.TryGetValue(token, out string? value))
        {
            return value;
        }
        return handle.Kind switch
        {
            HandleKind.TypeReference => MetadataTypeReferenceName(reader, (TypeReferenceHandle)handle),
            HandleKind.TypeDefinition => MetadataTypeDefinitionName(reader, (TypeDefinitionHandle)handle),
            HandleKind.MemberReference => MetadataMemberReferenceName(reader, (MemberReferenceHandle)handle, parentNames),
            HandleKind.AssemblyReference => reader.GetString(reader.GetAssemblyReference((AssemblyReferenceHandle)handle).Name),
            HandleKind.ModuleReference => reader.GetString(reader.GetModuleReference((ModuleReferenceHandle)handle).Name),
            _ => handle.Kind + ":" + token.ToString("x8", CultureInfo.InvariantCulture),
        };
    }

    private static string MetadataMemberReferenceName(
        MetadataReader reader,
        MemberReferenceHandle handle,
        IReadOnlyDictionary<int, string> parentNames)
    {
        MemberReference value = reader.GetMemberReference(handle);
        return MetadataEntityName(reader, value.Parent, parentNames)
            + "." + reader.GetString(value.Name);
    }

    private static string MetadataTypeReferenceName(MetadataReader reader, TypeReferenceHandle handle)
    {
        TypeReference value = reader.GetTypeReference(handle);
        string name = reader.GetString(value.Name);
        string namespaceName = reader.GetString(value.Namespace);
        return namespaceName.Length == 0 ? name : namespaceName + "." + name;
    }

    private static string MetadataTypeDefinitionName(MetadataReader reader, TypeDefinitionHandle handle)
    {
        TypeDefinition value = reader.GetTypeDefinition(handle);
        string name = reader.GetString(value.Name);
        string namespaceName = reader.GetString(value.Namespace);
        return namespaceName.Length == 0 ? name : namespaceName + "." + name;
    }

    private static int MetadataToken(EntityHandle handle)
    {
        return handle.IsNil ? 0 : MetadataTokens.GetToken(handle);
    }

    private static Dictionary<string, object?> Span(TextSpan span)
    {
        return Obj(("end", span.End), ("length", span.Length), ("start", span.Start));
    }

    private static Dictionary<string, object?> Obj(params (string Key, object? Value)[] fields)
    {
        Dictionary<string, object?> result = new(StringComparer.Ordinal);
        foreach ((string key, object? value) in fields)
        {
            result.Add(key, value);
        }
        return result;
    }

    private static string Hex(byte[] bytes) => Convert.ToHexString(bytes).ToLowerInvariant();

    private sealed class FixtureGenerator : ISourceGenerator
    {
        public void Initialize(GeneratorInitializationContext context) { }

        public void Execute(GeneratorExecutionContext context)
        {
            context.AddSource(
                "GeneratedDependency.g.cs",
                SourceText.From(
                    """
                    global using System;
                    namespace Generated.Dependency
                    {
                        public static class Bridge
                        {
                            public static int Value() => 17;
                        }
                    }
                    """,
                    new UTF8Encoding(false, true),
                    SourceHashAlgorithm.Sha256));
        }
    }

    private static IReadOnlyList<ProbeCase> Cases()
    {
        return new ProbeCase[]
        {
            new ProbeCase(
                "admitted-closed-exceptions",
                "admitted_exception_observation",
                """
                #nullable enable
                using System;
                namespace Probe
                {
                    public sealed class RequiredData
                    {
                        /*@shape:exception.compiler_metadata.required_attributes*/public required string Name
                        {
                            get;
                            /*@shape:exception.compiler_metadata.init_modreq*/init;
                        }
                    }
                    public static class ClosedExceptions
                    {
                        public static /*@shape:exception.nullable.shorthand.return_type*/int? Some(int value)
                        {
                            /*@shape:exception.nullable.shorthand.local_type*/int? result =
                                /*@shape:exception.nullable.shorthand.implicit_conversion*/value;
                            return result;
                        }
                        public static int? None()
                        {
                            return /*@shape:exception.nullable.shorthand.default*/default(int?);
                        }
                        public static bool Reference(/*@shape:exception.nullable.reference_annotation*/string? value)
                        {
                            return value is null;
                        }
                        public static int Array(/*@shape:exception.array.not_constructed_generic*/int[] values)
                        {
                            return /*@shape:exception.incidental.array_length*/values.Length;
                        }
                        public static int Text(string value)
                        {
                            return /*@shape:exception.incidental.string_length*/value.Length;
                        }
                        public static decimal Round(decimal value)
                        {
                            return /*@shape:exception.incidental.decimal_round*/decimal.Round(value, 2, MidpointRounding.ToEven);
                        }
                        public static DateOnly Date(int year, int month, int day)
                        {
                            return /*@shape:exception.incidental.date_only_constructor*/new DateOnly(year, month, day);
                        }
                        public static RequiredData Required(string name)
                        {
                            return new RequiredData { Name = name };
                        }
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                true),
            new ProbeCase(
                "dependency-mpk-package",
                "rejected_profile_form",
                """
                /*@shape:near_miss.dependency.namespace.import*/using Mpk.Package;
                namespace Probe
                {
                    [/*@shape:near_miss.dependency.attribute.mpk*/Marker]
                    public sealed class Derived
                        : /*@shape:near_miss.dependency.base_type.mpk*/Base,
                          /*@shape:near_miss.dependency.interface.mpk*/IContract
                    {
                    }
                    public static class UsesMpk
                    {
                        public static int ViaPackage()
                        {
                            return /*@shape:near_miss.dependency.package.reference_origin*/Api.Value;
                        }
                        public static int ViaAssembly()
                        {
                            return /*@shape:near_miss.dependency.assembly.selected_symbol*/Mpk.Package.Api.Value;
                        }
                    }
                }
                """,
                new[] { "mpk-package" },
                false,
                true),
            new ProbeCase(
                "dependency-namespace-spoof",
                "rejected_profile_form",
                """
                #warning CSHARP_T01_W06_REJECTED_NAMESPACE_SPOOF
                /*@shape:near_miss.dependency.namespace.spoof*/namespace Mpk.Spoof
                {
                    public static class Api { public static int Value => 19; }
                }
                namespace Probe
                {
                    public static class UsesSpoof
                    {
                        public static int Value() => Mpk.Spoof.Api.Value;
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                false),
            new ProbeCase(
                "dependency-project-and-ambient",
                "rejected_profile_form",
                """
                namespace Probe
                {
                    public static class ExternalUses
                    {
                        public static int Project()
                        {
                            return /*@shape:near_miss.dependency.project.reference_origin*/Mpk.Project.Api.Value;
                        }
                        public static int Ambient()
                        {
                            return /*@shape:near_miss.dependency.ambient.reference*/Vendor.Ambient.Api.Value;
                        }
                    }
                }
                """,
                new[] { "mpk-project", "ambient-project" },
                false,
                true),
            new ProbeCase(
                "dependency-generated-source",
                "rejected_profile_form",
                """
                namespace Probe
                {
                    public static class GeneratedUses
                    {
                        public static int Member()
                        {
                            return /*@shape:near_miss.dependency.generated_source.member*/Generated.Dependency.Bridge.Value();
                        }
                        public static DateOnly GlobalImport()
                        {
                            return /*@shape:near_miss.dependency.generated_source.global_using*/new DateOnly(2026, 9, 4);
                        }
                    }
                }
                """,
                Array.Empty<string>(),
                true,
                true),
            new ProbeCase(
                "source-written-attributes",
                "rejected_profile_form",
                """
                using System;
                [assembly: /*@shape:near_miss.attribute.source.assembly*/Probe.Marker]
                [module: /*@shape:near_miss.attribute.source.module*/Probe.Marker]
                namespace Probe
                {
                    [/*@shape:near_miss.attribute.source.attribute_usage*/AttributeUsage(AttributeTargets.All, AllowMultiple = true)]
                    public sealed class MarkerAttribute : Attribute { }

                    [/*@shape:near_miss.attribute.source.class*/Marker]
                    public class Attributed<[/*@shape:near_miss.attribute.source.type_parameter*/Marker] T>
                    {
                        [/*@shape:near_miss.attribute.source.field*/Marker]
                        private int _field;

                        [/*@shape:near_miss.attribute.source.event*/Marker]
                        public event Action? Changed;

                        [/*@shape:near_miss.attribute.source.property*/Marker]
                        public int Property
                        {
                            [/*@shape:near_miss.attribute.source.accessor_get*/Marker] get;
                            [/*@shape:near_miss.attribute.source.accessor_set*/Marker] set;
                        }

                        [/*@shape:near_miss.attribute.source.constructor*/Marker]
                        public Attributed([/*@shape:near_miss.attribute.source.parameter_constructor*/Marker] int value)
                        {
                            _field = value;
                        }

                        [return: /*@shape:near_miss.attribute.source.return*/Marker]
                        [/*@shape:near_miss.attribute.source.method*/Marker]
                        public int Method([/*@shape:near_miss.attribute.source.parameter_method*/Marker] int value)
                        {
                            Changed?.Invoke();
                            return _field + value;
                        }
                    }

                    [/*@shape:near_miss.attribute.source.struct*/Marker]
                    public struct AttributedStruct { }

                    [/*@shape:near_miss.attribute.source.enum*/Marker]
                    public enum AttributedEnum { Value = 0 }

                    [/*@shape:near_miss.attribute.source.interface*/Marker]
                    public interface AttributedInterface { }

                    [/*@shape:near_miss.attribute.source.delegate*/Marker]
                    public delegate int AttributedDelegate(int value);

                    public static class LocalAttributes
                    {
                        public static int Run()
                        {
                            [/*@shape:near_miss.attribute.source.local_function*/Marker]
                            int Local([/*@shape:near_miss.attribute.source.parameter_local*/Marker] int value) => value;
                            Func<int, int> function =
                                [/*@shape:near_miss.attribute.source.lambda*/Marker]
                                ([/*@shape:near_miss.attribute.source.parameter_lambda*/Marker] int value) => value;
                            return function(Local(1));
                        }
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                true),
            new ProbeCase(
                "source-written-compiler-markers",
                "rejected_profile_form",
                """
                using System.Diagnostics.CodeAnalysis;
                using System.Runtime.CompilerServices;
                namespace Probe
                {
                    [/*@shape:near_miss.attribute.compiler_marker.feature_required*/CompilerFeatureRequired("RequiredMembers")]
                    public sealed class SpoofedMarkers
                    {
                        [/*@shape:near_miss.attribute.compiler_marker.required_member*/RequiredMember]
                        public string Name { get; init; } = "";

                        [/*@shape:near_miss.attribute.compiler_marker.sets_required_members*/SetsRequiredMembers]
                        public SpoofedMarkers() { }
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                false),
            new ProbeCase(
                "generic-declarations",
                "rejected_profile_form",
                """
                using System;
                namespace Probe
                {
                    /*@shape:near_miss.generic.declaration.class*/public class Box<
                        /*@shape:near_miss.generic.type_parameter.type*/T>
                    {
                        public T Value { get; }
                        public Box(T value) { Value = value; }
                    }

                    /*@shape:near_miss.generic.declaration.struct*/public readonly struct Pair<T, U>
                    {
                        public T First { get; }
                        public U Second { get; }
                        public Pair(T first, U second) { First = first; Second = second; }
                    }

                    /*@shape:near_miss.generic.declaration.interface*/public interface Producer<
                        /*@shape:near_miss.generic.variance.out*/out T>
                    {
                        T Produce();
                    }

                    public interface Consumer<
                        /*@shape:near_miss.generic.variance.in*/in T>
                    {
                        void Consume(T value);
                    }

                    /*@shape:near_miss.generic.declaration.delegate*/public delegate TResult Mapper<T, TResult>(T value);

                    public static class GenericMethods
                    {
                        /*@shape:near_miss.generic.method.declaration*/public static T Identity<
                            /*@shape:near_miss.generic.type_parameter.method*/T>(T value)
                        {
                            return value;
                        }
                    }

                    public class Base { }
                    public interface ITag { }
                    public class Constraints<TClass, TNullableClass, TValue, TUnmanaged, TNotNull, TBase, TInterface, TNew, TDependent>
                        where TClass : /*@shape:near_miss.generic.constraint.reference*/class
                        where TNullableClass : /*@shape:near_miss.generic.constraint.reference_nullable*/class?
                        where TValue : /*@shape:near_miss.generic.constraint.value*/struct
                        where TUnmanaged : /*@shape:near_miss.generic.constraint.unmanaged*/unmanaged
                        where TNotNull : /*@shape:near_miss.generic.constraint.notnull*/notnull
                        where TBase : /*@shape:near_miss.generic.constraint.base*/Base
                        where TInterface : /*@shape:near_miss.generic.constraint.interface*/ITag
                        where TNew : /*@shape:near_miss.generic.constraint.constructor*/new()
                        where TDependent : /*@shape:near_miss.generic.constraint.dependent*/TBase
                    {
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                false),
            new ProbeCase(
                "generic-calls-and-constructed-types",
                "rejected_profile_form",
                """
                using System;
                using System.Collections.Generic;
                namespace Probe
                {
                    public sealed class Box<T>
                    {
                        public Box(T value) { Value = value; }
                        public T Value { get; }
                    }
                    public static class Uses
                    {
                        public static T Identity<T>(T value) => value;

                        public static int ExplicitCall(int value) =>
                            /*@shape:near_miss.generic.explicit_call.method*/Identity<int>(value);
                        public static int InferredCall(int value) =>
                            /*@shape:near_miss.generic.inferred_call.method*/Identity(value);
                        public static Box<int> ClosedUser(int value) =>
                            /*@shape:near_miss.generic.closed_use.user_type*/new Box<int>(value);
                        public static Type OpenUser() =>
                            typeof(/*@shape:near_miss.generic.open_type.user_type*/Box<>);

                        public static /*@shape:near_miss.generic.framework_type.list*/List<int> List() => new();
                        public static /*@shape:near_miss.generic.framework_type.dictionary*/Dictionary<string, int> Dictionary() => new();
                        public static /*@shape:near_miss.generic.framework_type.interface*/IEnumerable<int> Interface(int[] values) => values;
                        public static /*@shape:near_miss.generic.framework_type.delegate*/Func<int, int> Delegate() => value => value;
                        public static /*@shape:near_miss.generic.framework_type.tuple*/(int, string) Tuple() => (1, "one");
                        public static /*@shape:near_miss.generic.framework_type.span*/Span<int> Span(int[] values) => values;
                        public static /*@shape:near_miss.generic.framework_type.key_value_pair*/KeyValuePair<string, int> Entry() => new("one", 1);

                        public static /*@shape:near_miss.generic.explicit_nullable.system*/System.Nullable<int> ExplicitSystem(int value) => value;
                        public static /*@shape:near_miss.generic.explicit_nullable.alias*/Nullable<int> ExplicitAlias(int value) => value;
                        public static int? NewNullable(int value) =>
                            /*@shape:near_miss.generic.explicit_nullable.construction*/new int?(value);
                        public static int? CastNullable(int value) =>
                            /*@shape:near_miss.generic.explicit_nullable.cast*/(int?)value;
                        public static /*@shape:near_miss.generic.unsupported_nullable.byref*/ref int? ByRef(ref int? value)
                        {
                            return ref value;
                        }
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                false),
            new ProbeCase(
                "generic-invalid-nullable-payloads",
                "rejected_profile_form",
                """
                using System;
                namespace Probe
                {
                    public static class InvalidNullable
                    {
                        public static /*@shape:near_miss.generic.unsupported_nullable.reference_payload*/System.Nullable<string> ReferencePayload() => default;
                        public static /*@shape:near_miss.generic.unsupported_nullable.ref_struct_payload*/Span<int>? RefStructPayload() => null;
                        public static /*@shape:near_miss.generic.unsupported_nullable.nested*/System.Nullable<int?> Nested() => null;
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                false),
            new ProbeCase(
                "generic-transitive-metadata",
                "rejected_profile_form",
                """
                using System;
                using System.Collections.Generic;
                using System.Linq;
                namespace Probe
                {
                    public static class TransitiveMetadata
                    {
                        public static IEnumerable<char> StringEnumerable(string value) =>
                            /*@shape:near_miss.generic.transitive_metadata.string_interface*/value;
                        public static IList<int> ArrayList(int[] values) =>
                            /*@shape:near_miss.generic.transitive_metadata.array_interface*/values;
                        public static IComparable<string> StringComparable(string value) =>
                            /*@shape:near_miss.generic.transitive_metadata.string_comparable*/value;
                        public static IComparable<decimal> DecimalComparable(decimal value) =>
                            /*@shape:near_miss.generic.transitive_metadata.decimal_comparable*/value;
                        public static IComparable<DateOnly> DateComparable(DateOnly value) =>
                            /*@shape:near_miss.generic.transitive_metadata.date_comparable*/value;
                        public static int Count(int[] values) =>
                            /*@shape:near_miss.generic.transitive_metadata.inferred_linq*/Enumerable.Count(values);
                        public static int[] Empty() =>
                            /*@shape:near_miss.generic.transitive_metadata.explicit_array_empty*/Array.Empty<int>();
                        public static ReadOnlySpan<char> StringSpan(string value) =>
                            /*@shape:near_miss.generic.transitive_metadata.string_span*/value.AsSpan();
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                false),
            new ProbeCase(
                "iterator-forms",
                "rejected_profile_form",
                """
                using System;
                using System.Collections;
                using System.Collections.Generic;
                namespace Probe
                {
                    public static class IteratorForms
                    {
                        /*@shape:near_miss.iterator.declaration.generic*//*@shape:near_miss.iterator.state_machine*/public static IEnumerable<int> Values(int count)
                        {
                            for (int index = 0; index < count; index++)
                            {
                                /*@shape:near_miss.iterator.yield.return*/yield return index;
                            }
                            /*@shape:near_miss.iterator.yield.break*/yield break;
                        }

                        public static /*@shape:near_miss.iterator.protocol.ienumerable_generic*/IEnumerable<int> GenericProtocol(int[] values) => values;

                        /*@shape:near_miss.iterator.declaration.non_generic*/public static /*@shape:near_miss.iterator.protocol.ienumerable_non_generic*/IEnumerable NonGeneric()
                        {
                            yield return 1;
                        }

                        public static /*@shape:near_miss.iterator.protocol.ienumerator_generic*/IEnumerator<int> GenericEnumerator(IEnumerable<int> values) => values.GetEnumerator();
                        public static /*@shape:near_miss.iterator.protocol.ienumerator_non_generic*/IEnumerator NonGenericEnumerator(IEnumerable values) => values.GetEnumerator();

                        /*@shape:near_miss.iterator.declaration.local_function*/public static IEnumerable<int> LocalIterator()
                        {
                            IEnumerable<int> Local()
                            {
                                yield return 1;
                            }
                            return Local();
                        }
                    }

                    /*@shape:near_miss.iterator.protocol.manual_enumerator*/public struct ManualEnumerator : IEnumerator<int>
                    {
                        private int _current;
                        public int Current => _current;
                        object IEnumerator.Current => Current;
                        public bool MoveNext() { _current++; return _current < 2; }
                        public void Reset() { _current = 0; }
                        public void Dispose() { }
                    }

                    public sealed class CustomEnumerable
                    {
                        public ManualEnumerator GetEnumerator() => new();
                    }

                    public static class CustomProtocol
                    {
                        public static int Sum(CustomEnumerable values)
                        {
                            int total = 0;
                            /*@shape:near_miss.iterator.protocol.custom_foreach*/foreach (int value in values)
                            {
                                total += value;
                            }
                            return total;
                        }
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                true),
            new ProbeCase(
                "async-iterator-forms",
                "rejected_profile_form",
                """
                using System.Collections.Generic;
                using System.Threading.Tasks;
                namespace Probe
                {
                    public static class AsyncIterators
                    {
                        /*@shape:near_miss.iterator.async.declaration*//*@shape:near_miss.iterator.async.state_machine*/public static async IAsyncEnumerable<int> Values()
                        {
                            /*@shape:near_miss.iterator.async.await*/await Task.Yield();
                            /*@shape:near_miss.iterator.async.yield_return*/yield return 1;
                        }

                        public static /*@shape:near_miss.iterator.async.iasyncenumerator*/IAsyncEnumerator<int> Enumerator(/*@shape:near_miss.iterator.async.iasyncenumerable*/IAsyncEnumerable<int> values) => values.GetAsyncEnumerator();

                        public static async Task<int> Consume(IAsyncEnumerable<int> values)
                        {
                            int total = 0;
                            /*@shape:near_miss.iterator.async.await_foreach*/await foreach (int value in values)
                            {
                                total += value;
                            }
                            return total;
                        }
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                true),
            new ProbeCase(
                "async-task-and-value-task",
                "rejected_profile_form",
                """
                using System;
                using System.Threading;
                using System.Threading.Tasks;
                namespace Probe
                {
                    public static class AsyncForms
                    {
                        /*@shape:near_miss.async.declaration.task*//*@shape:near_miss.async.state_machine*/public static async /*@shape:near_miss.async.task.non_generic*/Task Wait(Task input)
                        {
                            /*@shape:near_miss.async.await.task*/await input;
                        }

                        /*@shape:near_miss.async.declaration.task_generic*/public static async /*@shape:near_miss.async.task.generic*/Task<int> WaitValue(Task<int> input)
                        {
                            return await input;
                        }

                        /*@shape:near_miss.async.declaration.void*/public static async void Fire(Task input)
                        {
                            await input;
                        }

                        public static async /*@shape:near_miss.async.value_task.non_generic*/ValueTask WaitValueTask(ValueTask input)
                        {
                            /*@shape:near_miss.async.await.value_task*/await input;
                        }

                        public static async /*@shape:near_miss.async.value_task.generic*/ValueTask<int> WaitValueTaskResult(ValueTask<int> input)
                        {
                            return await input;
                        }

                        public static Task Completed() =>
                            /*@shape:near_miss.async.task.factory_completed*/Task.CompletedTask;
                        public static Task<int> FromResult() =>
                            /*@shape:near_miss.async.task.factory_result*/Task.FromResult(1);
                        public static Task RunFactory() =>
                            /*@shape:near_miss.async.task.factory_run*/Task.Run(static () => { });
                        public static Task<Task> Race(Task first, Task second) =>
                            /*@shape:near_miss.async.task.race_when_any*/Task.WhenAny(first, second);
                        public static Task<TaskStatus> Continuation(Task input) =>
                            /*@shape:near_miss.async.task.continuation*/input.ContinueWith(static completed => completed.Status);
                        public static TaskScheduler Scheduler() =>
                            /*@shape:near_miss.async.task.scheduler*/TaskScheduler.Current;
                        public static SynchronizationContext? Context() =>
                            /*@shape:near_miss.async.task.synchronization_context*/SynchronizationContext.Current;
                        public static ParallelLoopResult ParallelLoop() =>
                            /*@shape:near_miss.async.parallel.for*/Parallel.For(0, 2, static _ => { });
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                true),
            new ProbeCase(
                "async-custom-awaiter-and-cancellation",
                "rejected_profile_form",
                """
                using System;
                using System.Runtime.CompilerServices;
                using System.Threading;
                using System.Threading.Tasks;
                namespace Probe
                {
                    /*@shape:near_miss.async.awaiter.custom_type*/public readonly struct CustomAwaiter : INotifyCompletion
                    {
                        public bool IsCompleted => true;
                        public int GetResult() => 23;
                        public void OnCompleted(Action continuation) => continuation();
                    }

                    /*@shape:near_miss.async.awaiter.custom_awaitable*/public readonly struct CustomAwaitable
                    {
                        public CustomAwaiter GetAwaiter() => new();
                    }

                    public static class AwaiterAndCancellation
                    {
                        /*@shape:near_miss.async.declaration.custom_awaiter*//*@shape:near_miss.async.state_machine.custom_awaiter*/public static async Task<int> Custom(CustomAwaitable value)
                        {
                            return /*@shape:near_miss.async.await.custom*/await value;
                        }

                        public static int Cancellation(/*@shape:near_miss.async.cancellation.token*/CancellationToken token)
                        {
                            /*@shape:near_miss.async.cancellation.throw_if_requested*/token.ThrowIfCancellationRequested();
                            return 0;
                        }

                        public static CancellationTokenSource Source() =>
                            /*@shape:near_miss.async.cancellation.source*/new CancellationTokenSource();
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                true),
            new ProbeCase(
                "async-lambda-and-local-function",
                "rejected_profile_form",
                """
                using System;
                using System.Threading.Tasks;
                namespace Probe
                {
                    public static class NestedAsync
                    {
                        public static async Task<int> Run()
                        {
                            Func<Task<int>> lambda =
                                /*@shape:near_miss.async.declaration.lambda*//*@shape:near_miss.async.state_machine.lambda*/async () =>
                                {
                                    /*@shape:near_miss.async.await.lambda*/await Task.Yield();
                                    return 1;
                                };

                            /*@shape:near_miss.async.declaration.local_function*/async Task<int> Local()
                            {
                                /*@shape:near_miss.async.await.local_function*/await Task.Yield();
                                return 2;
                            }

                            return await lambda() + await Local();
                        }
                    }
                }
                """,
                Array.Empty<string>(),
                false,
                true),
        };
    }

    private sealed class ProbeFailure : Exception
    {
        internal ProbeFailure(string code) => Code = code;
        internal string Code { get; }
    }
}
