// Disposable CSHARP-03-T01-W04 public Roslyn API probe; never a frontend.
#nullable enable

using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Threading;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.FlowAnalysis;
using Microsoft.CodeAnalysis.Operations;
using Microsoft.CodeAnalysis.Text;

internal static class DataConstructionProbe
{
    private const string RawSchema = "mpk.csharp_practical.t01_w04.roslyn_data_probe.raw.v0";
    private const string WorkItem = "CSHARP-03-T01-W04";
    private const string MarkerPrefix = "/*@shape:";
    private const string MarkerSuffix = "*/";

    private static readonly SymbolDisplayFormat DisplayFormat =
        SymbolDisplayFormat.CSharpErrorMessageFormat.WithMiscellaneousOptions(
            SymbolDisplayFormat.CSharpErrorMessageFormat.MiscellaneousOptions
            | SymbolDisplayMiscellaneousOptions.IncludeNullableReferenceTypeModifier
            | SymbolDisplayMiscellaneousOptions.EscapeKeywordIdentifiers);

    private sealed record ProbeCase(string Id, string Disposition, string Source);
    private sealed record EmittedObservation(
        object? Record,
        IReadOnlyDictionary<string, object?> TypesByDisplay);

    public static int Main(string[] args)
    {
        if (args.Length != 1)
        {
            Console.Error.Write("CSHARP_PRACTICAL_DATA_PROBE_USAGE\n");
            return 64;
        }

        try
        {
            CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;
            CultureInfo.CurrentUICulture = CultureInfo.InvariantCulture;
            ImmutableArray<MetadataReference> references = LoadReferences(args[0]);
            List<object?> cases = new();
            foreach (ProbeCase probeCase in Cases())
            {
                cases.Add(ObserveCase(probeCase, references));
            }

            Dictionary<string, object?> root = Obj(
                ("cases", cases),
                ("compiler", Obj(
                    ("architecture", RuntimeInformation.ProcessArchitecture.ToString()),
                    ("language", LanguageNames.CSharp),
                    ("language_version", LanguageVersion.CSharp14.ToDisplayString()),
                    ("nullable_context", NullableContextOptions.Enable.ToString()),
                    ("reference_count", references.Length),
                    ("roslyn_common", AssemblyIdentity(typeof(Compilation).Assembly)),
                    ("roslyn_csharp", AssemblyIdentity(typeof(CSharpCompilation).Assembly)),
                    ("runtime_version", Environment.Version.ToString()))),
                ("schema", RawSchema),
                ("work_item", WorkItem));

            Console.OutputEncoding = new UTF8Encoding(false, true);
            Console.Write(JsonSerializer.Serialize(root));
            Console.Write('\n');
            return 0;
        }
        catch (ProbeFailure failure)
        {
            Console.Error.Write("CSHARP_PRACTICAL_DATA_PROBE_" + failure.Code + "\n");
            return 65;
        }
        catch (Exception)
        {
            Console.Error.Write("CSHARP_PRACTICAL_DATA_PROBE_UNEXPECTED\n");
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

        ImmutableArray<MetadataReference>.Builder builder = ImmutableArray.CreateBuilder<MetadataReference>(paths.Length);
        foreach (string path in paths)
        {
            builder.Add(MetadataReference.CreateFromFile(path));
        }
        return builder.MoveToImmutable();
    }

    private static object ObserveCase(ProbeCase probeCase, ImmutableArray<MetadataReference> references)
    {
        CSharpParseOptions parseOptions = new(
            languageVersion: LanguageVersion.CSharp14,
            documentationMode: DocumentationMode.None,
            kind: SourceCodeKind.Regular);
        SourceText text = SourceText.From(
            probeCase.Source,
            new UTF8Encoding(false, true),
            SourceHashAlgorithm.Sha256);
        string path = "src/" + probeCase.Id.Replace('.', '_') + ".cs";
        SyntaxTree tree = CSharpSyntaxTree.ParseText(text, parseOptions, path, CancellationToken.None);
        CSharpCompilationOptions compilationOptions = new(
            OutputKind.DynamicallyLinkedLibrary,
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
        CSharpCompilation compilation = CSharpCompilation.Create(
            "probe_" + probeCase.Id.Replace('.', '_'),
            new[] { tree },
            references,
            compilationOptions);
        SemanticModel model = compilation.GetSemanticModel(tree, ignoreAccessibility: false);
        SyntaxNode root = tree.GetRoot(CancellationToken.None);
        Diagnostic[] diagnostics = compilation.GetDiagnostics(CancellationToken.None)
            .OrderBy(DiagnosticSortKey, StringComparer.Ordinal)
            .ToArray();
        bool hasErrors = diagnostics.Any(diagnostic => diagnostic.Severity == DiagnosticSeverity.Error);
        if (probeCase.Disposition == "admitted_shape" && hasErrors)
        {
            string errorIds = string.Join(
                "_",
                diagnostics
                    .Where(diagnostic => diagnostic.Severity == DiagnosticSeverity.Error)
                    .Select(diagnostic => diagnostic.Id)
                    .Distinct(StringComparer.Ordinal)
                    .OrderBy(id => id, StringComparer.Ordinal));
            throw new ProbeFailure(
                "ADMITTED_CASE_DIAGNOSTIC_"
                + probeCase.Id.Replace('-', '_').ToUpperInvariant()
                + "_"
                + errorIds);
        }

        List<object?> syntax = ObserveSyntax(root);
        List<object?> semantic = ObserveSemanticNodes(root, model);
        List<IOperation> roots = OperationRoots(root, model);
        List<object?> operations = roots.Select(ObserveOperation).Cast<object?>().ToList();
        List<object?> cfgs = ObserveControlFlowGraphs(roots);
        List<object?> types = ObserveSourceTypes(root, model);
        EmittedObservation emitted = ObserveEmittedMetadata(
            compilation,
            compilationOptions,
            references,
            hasErrors);
        List<object?> targets = ObserveTargets(
            probeCase.Source,
            root,
            model,
            roots,
            emitted.Record,
            emitted.TypesByDisplay);
        if (targets.Count == 0)
        {
            throw new ProbeFailure("MISSING_TARGET");
        }

        return Obj(
            ("compiler_outcome", hasErrors ? "error" : "success"),
            ("control_flow_graphs", cfgs),
            ("diagnostics", diagnostics.Select(ObserveDiagnostic).Cast<object?>().ToList()),
            ("disposition", probeCase.Disposition),
            ("emitted_metadata", emitted.Record),
            ("id", probeCase.Id),
            ("operation_roots", operations),
            ("semantic_nodes", semantic),
            ("source", probeCase.Source),
            ("source_types", types),
            ("source_utf8_sha256", Hex(SHA256.HashData(Encoding.UTF8.GetBytes(probeCase.Source)))),
            ("syntax", syntax),
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
                    ("depth", Depth(node)),
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
                    ("depth", token.Parent is null ? 0 : Depth(token.Parent) + 1),
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

    private static int Depth(SyntaxNode node)
    {
        int depth = 0;
        for (SyntaxNode? current = node.Parent; current is not null; current = current.Parent)
        {
            depth++;
        }
        return depth;
    }

    private static List<object?> ObserveSemanticNodes(SyntaxNode root, SemanticModel model)
    {
        List<object?> result = new();
        foreach (SyntaxNode node in root.DescendantNodesAndSelf(descendIntoTrivia: true))
        {
            ISymbol? declared = model.GetDeclaredSymbol(node, CancellationToken.None);
            SymbolInfo symbolInfo = default;
            TypeInfo typeInfo = default;
            Conversion? conversion = null;
            bool hasSymbolInfo = node is ExpressionSyntax || node is TypeSyntax || node is ConstructorInitializerSyntax;
            bool hasTypeInfo = node is ExpressionSyntax || node is TypeSyntax;
            if (hasSymbolInfo)
            {
                symbolInfo = model.GetSymbolInfo(node, CancellationToken.None);
            }
            if (hasTypeInfo)
            {
                typeInfo = model.GetTypeInfo(node, CancellationToken.None);
            }
            if (node is ExpressionSyntax expression)
            {
                conversion = model.GetConversion(expression, CancellationToken.None);
            }
            IOperation? operation = model.GetOperation(node, CancellationToken.None);
            if (declared is null
                && (!hasSymbolInfo || (symbolInfo.Symbol is null && symbolInfo.CandidateSymbols.IsEmpty))
                && (!hasTypeInfo || (typeInfo.Type is null && typeInfo.ConvertedType is null))
                && operation is null)
            {
                continue;
            }
            result.Add(Obj(
                ("candidate_reason", hasSymbolInfo ? symbolInfo.CandidateReason.ToString() : null),
                ("candidate_symbols", hasSymbolInfo
                    ? symbolInfo.CandidateSymbols.Select(SymbolIdentity).Cast<object?>().ToList()
                    : new List<object?>()),
                ("conversion", conversion.HasValue ? ObserveConversion(conversion.Value) : null),
                ("converted_type", hasTypeInfo ? TypeIdentity(typeInfo.ConvertedType) : null),
                ("declared_symbol", SymbolIdentity(declared)),
                ("kind", node.Kind().ToString()),
                ("operation", operation is null ? null : OperationSummary(operation)),
                ("span", Span(node.Span)),
                ("symbol", hasSymbolInfo ? SymbolIdentity(symbolInfo.Symbol) : null),
                ("type", hasTypeInfo ? TypeIdentity(typeInfo.Type) : null)));
        }
        return result;
    }

    private static List<IOperation> OperationRoots(SyntaxNode root, SemanticModel model)
    {
        Dictionary<string, IOperation> result = new(StringComparer.Ordinal);
        foreach (SyntaxNode node in root.DescendantNodesAndSelf(descendIntoTrivia: true))
        {
            IOperation? operation = model.GetOperation(node, CancellationToken.None);
            if (operation is null || operation.Parent is not null)
            {
                continue;
            }
            string key = operation.Kind + ":" + operation.Syntax.Span.Start.ToString(CultureInfo.InvariantCulture)
                + ":" + operation.Syntax.Span.Length.ToString(CultureInfo.InvariantCulture);
            result.TryAdd(key, operation);
        }
        return result.Values
            .OrderBy(operation => operation.Syntax.Span.Start)
            .ThenBy(operation => operation.Syntax.Span.Length)
            .ThenBy(operation => operation.Kind.ToString(), StringComparer.Ordinal)
            .ToList();
    }

    private static object ObserveOperation(IOperation operation)
    {
        List<object?> children = operation.ChildOperations.Select(ObserveOperation).Cast<object?>().ToList();
        return Obj(
            ("children", children),
            ("constant", ObserveConstant(operation.ConstantValue)),
            ("details", OperationDetails(operation)),
            ("is_implicit", operation.IsImplicit),
            ("kind", operation.Kind.ToString()),
            ("language", operation.Language),
            ("span", Span(operation.Syntax.Span)),
            ("syntax_kind", operation.Syntax.Kind().ToString()),
            ("type", TypeIdentity(operation.Type)));
    }

    private static object OperationSummary(IOperation operation)
    {
        return Obj(
            ("is_implicit", operation.IsImplicit),
            ("kind", operation.Kind.ToString()),
            ("parent_kind", operation.Parent?.Kind.ToString()),
            ("span", Span(operation.Syntax.Span)),
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
            IMethodReferenceOperation value => Obj(("method", SymbolIdentity(value.Method))),
            IEventReferenceOperation value => Obj(("event", SymbolIdentity(value.Event))),
            IConversionOperation value => Obj(
                ("conversion", ObserveCommonConversion(value.Conversion)),
                ("is_checked", value.IsChecked),
                ("is_try_cast", value.IsTryCast),
                ("operator_method", SymbolIdentity(value.OperatorMethod))),
            IBinaryOperation value => Obj(
                ("is_checked", value.IsChecked),
                ("is_lifted", value.IsLifted),
                ("operator_kind", value.OperatorKind.ToString()),
                ("operator_method", SymbolIdentity(value.OperatorMethod))),
            IUnaryOperation value => Obj(
                ("is_checked", value.IsChecked),
                ("is_lifted", value.IsLifted),
                ("operator_kind", value.OperatorKind.ToString()),
                ("operator_method", SymbolIdentity(value.OperatorMethod))),
            IVariableDeclaratorOperation value => Obj(("symbol", SymbolIdentity(value.Symbol))),
            ILocalReferenceOperation value => Obj(("is_declaration", value.IsDeclaration), ("local", SymbolIdentity(value.Local))),
            IParameterReferenceOperation value => Obj(("parameter", SymbolIdentity(value.Parameter))),
            IArrayCreationOperation value => Obj(("dimension_count", value.DimensionSizes.Length)),
            IArrayElementReferenceOperation value => Obj(("index_count", value.Indices.Length)),
            IArgumentOperation value => Obj(
                ("argument_kind", value.ArgumentKind.ToString()),
                ("in_conversion", ObserveCommonConversion(value.InConversion)),
                ("out_conversion", ObserveCommonConversion(value.OutConversion)),
                ("parameter", SymbolIdentity(value.Parameter))),
            _ => Obj(),
        };
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
            ("value", ConstantText(value)));
    }

    private static string? ConstantText(object? value)
    {
        return value switch
        {
            null => null,
            char character => ((int)character).ToString(CultureInfo.InvariantCulture),
            float single => BitConverter.SingleToUInt32Bits(single).ToString("x8", CultureInfo.InvariantCulture),
            double number => BitConverter.DoubleToUInt64Bits(number).ToString("x16", CultureInfo.InvariantCulture),
            decimal number => string.Join(",", decimal.GetBits(number).Select(part => part.ToString("x8", CultureInfo.InvariantCulture))),
            IFormattable formattable => formattable.ToString(null, CultureInfo.InvariantCulture),
            _ => value.ToString(),
        };
    }

    private static object ObserveConversion(Conversion conversion)
    {
        return Obj(
            ("exists", conversion.Exists),
            ("is_boxing", conversion.IsBoxing),
            ("is_constant_expression", conversion.IsConstantExpression),
            ("is_default_literal", conversion.IsDefaultLiteral),
            ("is_dynamic", conversion.IsDynamic),
            ("is_enumeration", conversion.IsEnumeration),
            ("is_explicit", conversion.IsExplicit),
            ("is_identity", conversion.IsIdentity),
            ("is_implicit", conversion.IsImplicit),
            ("is_nullable", conversion.IsNullable),
            ("is_null_literal", conversion.IsNullLiteral),
            ("is_numeric", conversion.IsNumeric),
            ("is_pointer", conversion.IsPointer),
            ("is_reference", conversion.IsReference),
            ("is_tuple", conversion.IsTupleConversion),
            ("is_unboxing", conversion.IsUnboxing),
            ("is_user_defined", conversion.IsUserDefined),
            ("method", SymbolIdentity(conversion.MethodSymbol)));
    }

    private static object ObserveCommonConversion(CommonConversion conversion)
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

    private static List<object?> ObserveControlFlowGraphs(IEnumerable<IOperation> roots)
    {
        List<object?> result = new();
        foreach (IOperation root in roots)
        {
            ControlFlowGraph? graph = root switch
            {
                IMethodBodyOperation method => ControlFlowGraph.Create(method, CancellationToken.None),
                IConstructorBodyOperation constructor => ControlFlowGraph.Create(constructor, CancellationToken.None),
                _ => null,
            };
            if (graph is not null)
            {
                result.Add(ObserveControlFlowGraph(graph));
            }
        }
        return result;
    }

    private static object ObserveControlFlowGraph(ControlFlowGraph graph)
    {
        Dictionary<ControlFlowRegion, int> regionIds = new(ReferenceEqualityComparer.Instance);
        AssignRegionIds(graph.Root, regionIds);
        List<object?> blocks = new();
        foreach (BasicBlock block in graph.Blocks)
        {
            blocks.Add(Obj(
                ("branch_value", block.BranchValue is null ? null : OperationSummary(block.BranchValue)),
                ("conditional_successor", ObserveBranch(block.ConditionalSuccessor, regionIds)),
                ("condition_kind", block.ConditionKind.ToString()),
                ("enclosing_region", regionIds[block.EnclosingRegion]),
                ("fallthrough_successor", ObserveBranch(block.FallThroughSuccessor, regionIds)),
                ("is_reachable", block.IsReachable),
                ("kind", block.Kind.ToString()),
                ("operations", block.Operations.Select(OperationSummary).Cast<object?>().ToList()),
                ("ordinal", block.Ordinal)));
        }
        return Obj(
            ("blocks", blocks),
            ("original_operation", OperationSummary(graph.OriginalOperation)),
            ("regions", ObserveRegion(graph.Root, regionIds)));
    }

    private static void AssignRegionIds(ControlFlowRegion region, Dictionary<ControlFlowRegion, int> ids)
    {
        ids.Add(region, ids.Count);
        foreach (ControlFlowRegion nested in region.NestedRegions)
        {
            AssignRegionIds(nested, ids);
        }
    }

    private static object ObserveRegion(ControlFlowRegion region, Dictionary<ControlFlowRegion, int> ids)
    {
        return Obj(
            ("capture_ids", region.CaptureIds.Select(id => id.ToString()).Cast<object?>().ToList()),
            ("exception_type", ObserveSymbol(region.ExceptionType)),
            ("first_block", region.FirstBlockOrdinal),
            ("id", ids[region]),
            ("kind", region.Kind.ToString()),
            ("last_block", region.LastBlockOrdinal),
            ("locals", region.Locals.Select(ObserveSymbol).Cast<object?>().ToList()),
            ("nested", region.NestedRegions.Select(item => ObserveRegion(item, ids)).Cast<object?>().ToList()));
    }

    private static object? ObserveBranch(ControlFlowBranch? branch, Dictionary<ControlFlowRegion, int> ids)
    {
        if (branch is null)
        {
            return null;
        }
        return Obj(
            ("destination", branch.Destination?.Ordinal),
            ("entering_regions", branch.EnteringRegions.Select(region => ids[region]).Cast<object?>().ToList()),
            ("finally_regions", branch.FinallyRegions.Select(region => ids[region]).Cast<object?>().ToList()),
            ("leaving_regions", branch.LeavingRegions.Select(region => ids[region]).Cast<object?>().ToList()),
            ("semantics", branch.Semantics.ToString()));
    }

    private static List<object?> ObserveSourceTypes(SyntaxNode root, SemanticModel model)
    {
        List<object?> result = new();
        foreach (BaseTypeDeclarationSyntax declaration in root.DescendantNodes().OfType<BaseTypeDeclarationSyntax>())
        {
            if (model.GetDeclaredSymbol(declaration, CancellationToken.None) is not INamedTypeSymbol type)
            {
                continue;
            }
            List<ISymbol> members = type.GetMembers()
                .OrderBy(symbol => symbol.Locations.Any(location => location.IsInSource) ? 0 : 1)
                .ThenBy(symbol => symbol.Locations.FirstOrDefault(location => location.IsInSource)?.SourceSpan.Start ?? int.MaxValue)
                .ThenBy(symbol => symbol.Kind.ToString(), StringComparer.Ordinal)
                .ThenBy(symbol => symbol.MetadataName, StringComparer.Ordinal)
                .ToList();
            result.Add(Obj(
                ("attributes", ObserveAttributes(type.GetAttributes())),
                ("members", members.Select(ObserveSymbol).Cast<object?>().ToList()),
                ("symbol", ObserveSymbol(type))));
        }
        return result;
    }

    private static List<object?> ObserveTargets(
        string source,
        SyntaxNode root,
        SemanticModel model,
        IReadOnlyList<IOperation> operationRoots,
        object? emittedMetadata,
        IReadOnlyDictionary<string, object?> emittedTypes)
    {
        List<object?> result = new();
        HashSet<string> ids = new(StringComparer.Ordinal);
        int search = 0;
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
            SyntaxNode? target = root.DescendantNodesAndSelf(descendIntoTrivia: true)
                .Where(node => node != root && node.SpanStart >= after && !node.IsMissing)
                .OrderBy(node => node.SpanStart)
                .ThenByDescending(node => node.Span.Length)
                .FirstOrDefault();
            if (target is null)
            {
                throw new ProbeFailure("MARKER_TARGET");
            }
            IOperation? operation = model.GetOperation(target, CancellationToken.None);
            ISymbol? declared = model.GetDeclaredSymbol(target, CancellationToken.None);
            SymbolInfo symbolInfo = target is ExpressionSyntax || target is TypeSyntax || target is ConstructorInitializerSyntax
                ? model.GetSymbolInfo(target, CancellationToken.None)
                : default;
            TypeInfo typeInfo = target is ExpressionSyntax || target is TypeSyntax
                ? model.GetTypeInfo(target, CancellationToken.None)
                : default;
            Conversion? conversion = target is ExpressionSyntax expression
                ? model.GetConversion(expression, CancellationToken.None)
                : null;
            INamedTypeSymbol? relatedType = declared switch
            {
                INamedTypeSymbol type => type,
                _ => declared?.ContainingType,
            };
            IOperation? flowRoot = operationRoots.FirstOrDefault(candidate =>
                candidate.Syntax.Span.Contains(target.Span));
            object? emittedType = null;
            if (relatedType is not null
                && (id.StartsWith("compiler_marker.", StringComparison.Ordinal)
                    || id.StartsWith("synthesized.", StringComparison.Ordinal)))
            {
                emittedTypes.TryGetValue(
                    relatedType.ToDisplayString(DisplayFormat),
                    out object? emittedSourceType);
                emittedType = Obj(
                    ("compiler_metadata", emittedMetadata),
                    ("source_type", emittedSourceType));
            }
            result.Add(Obj(
                ("candidate_reason", symbolInfo.CandidateReason.ToString()),
                ("candidate_symbols", symbolInfo.CandidateSymbols.Select(ObserveSymbol).Cast<object?>().ToList()),
                ("conversion", conversion.HasValue ? ObserveConversion(conversion.Value) : null),
                ("converted_type", ObserveSymbol(typeInfo.ConvertedType)),
                ("declared_symbol", ObserveSymbol(declared)),
                ("enclosing_flow_root", flowRoot is null ? null : OperationSummary(flowRoot)),
                ("emitted_type", emittedType),
                ("marker_span", Span(new TextSpan(start, after - start))),
                ("operation", operation is null ? null : ObserveTargetOperation(operation)),
                ("related_type_members", relatedType is null
                    ? new List<object?>()
                    : relatedType.GetMembers().Select(ObserveSymbol).Cast<object?>().ToList()),
                ("shape_id", id),
                ("symbol", ObserveSymbol(symbolInfo.Symbol)),
                ("syntax", Obj(
                    ("contains_diagnostics", target.ContainsDiagnostics),
                    ("full_span", Span(target.FullSpan)),
                    ("is_missing", target.IsMissing),
                    ("kind", target.Kind().ToString()),
                    ("raw_kind", target.RawKind),
                    ("span", Span(target.Span)))),
                ("type", ObserveSymbol(typeInfo.Type))));
            search = after;
        }
        return result;
    }

    private static EmittedObservation ObserveEmittedMetadata(
        CSharpCompilation compilation,
        CSharpCompilationOptions options,
        ImmutableArray<MetadataReference> references,
        bool hasErrors)
    {
        Dictionary<string, object?> typesByDisplay = new(StringComparer.Ordinal);
        if (hasErrors)
        {
            return new EmittedObservation(null, typesByDisplay);
        }
        using MemoryStream stream = new();
        var emit = compilation.Emit(stream, cancellationToken: CancellationToken.None);
        if (!emit.Success)
        {
            throw new ProbeFailure("EMIT");
        }
        byte[] image = stream.ToArray();
        PortableExecutableReference emittedReference = MetadataReference.CreateFromImage(
            ImmutableArray.Create(image));
        CSharpCompilation inspection = CSharpCompilation.Create(
            compilation.AssemblyName + "_inspection",
            references: references.Add(emittedReference),
            options: options);
        if (inspection.GetAssemblyOrModuleSymbol(emittedReference) is not IAssemblySymbol assembly)
        {
            throw new ProbeFailure("EMITTED_SYMBOL");
        }
        List<object?> typeRecords = new();
        foreach (INamedTypeSymbol type in MetadataTypes(assembly.GlobalNamespace)
                     .OrderBy(item => item.ToDisplayString(DisplayFormat), StringComparer.Ordinal))
        {
            object record = Obj(
                ("attributes", ObserveAttributes(type.GetAttributes())),
                ("members", type.GetMembers()
                    .OrderBy(member => member.Kind.ToString(), StringComparer.Ordinal)
                    .ThenBy(member => member.MetadataName, StringComparer.Ordinal)
                    .Select(ObserveSymbol)
                    .Cast<object?>()
                    .ToList()),
                ("symbol", ObserveSymbol(type)));
            typeRecords.Add(record);
            typesByDisplay.Add(type.ToDisplayString(DisplayFormat), record);
        }
        object result = Obj(
            ("assembly", ObserveSymbol(assembly)),
            ("emit_diagnostics", emit.Diagnostics
                .OrderBy(DiagnosticSortKey, StringComparer.Ordinal)
                .Select(ObserveDiagnostic)
                .Cast<object?>()
                .ToList()),
            ("pe_sha256", Hex(SHA256.HashData(image))),
            ("pe_size_bytes", image.Length),
            ("raw_metadata", ObserveRawMetadata(image)),
            ("types", typeRecords));
        return new EmittedObservation(result, typesByDisplay);
    }

    private static object ObserveRawMetadata(byte[] image)
    {
        using MemoryStream stream = new(image, writable: false);
        using PEReader pe = new(stream, PEStreamOptions.LeaveOpen);
        MetadataReader reader = pe.GetMetadataReader();
        Dictionary<int, string> parentNames = MetadataParentNames(reader);
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
                    ("parent_token", MetadataToken(value.Parent)),
                    ("signature", Hex(reader.GetBlobBytes(value.Signature))),
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
                    ("constructor_token", MetadataToken(value.Constructor)),
                    ("parent", MetadataEntityName(reader, value.Parent, parentNames)),
                    ("parent_kind", value.Parent.Kind.ToString()),
                    ("parent_token", MetadataToken(value.Parent)),
                    ("token", MetadataTokens.GetToken(handle)),
                    ("value", Hex(reader.GetBlobBytes(value.Value))));
            })
            .Cast<object?>()
            .ToList();
        return Obj(
            ("custom_attributes", customAttributes),
            ("member_references", memberReferences),
            ("type_references", typeReferences));
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
                FieldDefinition field = reader.GetFieldDefinition(fieldHandle);
                names.Add(
                    MetadataTokens.GetToken(fieldHandle),
                    typeName + "." + reader.GetString(field.Name));
            }
            foreach (MethodDefinitionHandle methodHandle in type.GetMethods())
            {
                MethodDefinition method = reader.GetMethodDefinition(methodHandle);
                string methodName = reader.GetString(method.Name);
                names.Add(MetadataTokens.GetToken(methodHandle), typeName + "." + methodName);
                foreach (ParameterHandle parameterHandle in method.GetParameters())
                {
                    Parameter parameter = reader.GetParameter(parameterHandle);
                    names.Add(
                        MetadataTokens.GetToken(parameterHandle),
                        typeName + "." + methodName + "(" + reader.GetString(parameter.Name) + ")");
                }
            }
            foreach (PropertyDefinitionHandle propertyHandle in type.GetProperties())
            {
                PropertyDefinition property = reader.GetPropertyDefinition(propertyHandle);
                names.Add(
                    MetadataTokens.GetToken(propertyHandle),
                    typeName + "." + reader.GetString(property.Name));
            }
            foreach (EventDefinitionHandle eventHandle in type.GetEvents())
            {
                EventDefinition eventDefinition = reader.GetEventDefinition(eventHandle);
                names.Add(
                    MetadataTokens.GetToken(eventHandle),
                    typeName + "." + reader.GetString(eventDefinition.Name));
            }
        }
        return names;
    }

    private static string MetadataEntityName(
        MetadataReader reader,
        EntityHandle handle,
        IReadOnlyDictionary<int, string> parentNames)
    {
        int token = MetadataToken(handle);
        if (parentNames.TryGetValue(token, out string? value))
        {
            return value;
        }
        return handle.Kind switch
        {
            HandleKind.TypeReference => MetadataTypeReferenceName(
                reader,
                (TypeReferenceHandle)handle),
            HandleKind.TypeDefinition => MetadataTypeDefinitionName(
                reader,
                (TypeDefinitionHandle)handle),
            HandleKind.MemberReference => MetadataMemberReferenceName(
                reader,
                (MemberReferenceHandle)handle,
                parentNames),
            HandleKind.AssemblyReference => reader.GetString(
                reader.GetAssemblyReference((AssemblyReferenceHandle)handle).Name),
            HandleKind.ModuleReference => reader.GetString(
                reader.GetModuleReference((ModuleReferenceHandle)handle).Name),
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
            + "."
            + reader.GetString(value.Name);
    }

    private static string MetadataTypeReferenceName(
        MetadataReader reader,
        TypeReferenceHandle handle)
    {
        TypeReference value = reader.GetTypeReference(handle);
        string name = reader.GetString(value.Name);
        string namespaceName = reader.GetString(value.Namespace);
        return namespaceName.Length == 0 ? name : namespaceName + "." + name;
    }

    private static string MetadataTypeDefinitionName(
        MetadataReader reader,
        TypeDefinitionHandle handle)
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

    private static IEnumerable<INamedTypeSymbol> MetadataTypes(INamespaceSymbol root)
    {
        foreach (INamedTypeSymbol type in root.GetTypeMembers())
        {
            foreach (INamedTypeSymbol value in MetadataTypeAndNested(type))
            {
                yield return value;
            }
        }
        foreach (INamespaceSymbol child in root.GetNamespaceMembers()
                     .OrderBy(item => item.Name, StringComparer.Ordinal))
        {
            foreach (INamedTypeSymbol type in MetadataTypes(child))
            {
                yield return type;
            }
        }
    }

    private static IEnumerable<INamedTypeSymbol> MetadataTypeAndNested(INamedTypeSymbol root)
    {
        yield return root;
        foreach (INamedTypeSymbol child in root.GetTypeMembers()
                     .OrderBy(item => item.MetadataName, StringComparer.Ordinal))
        {
            foreach (INamedTypeSymbol type in MetadataTypeAndNested(child))
            {
                yield return type;
            }
        }
    }

    private static object ObserveTargetOperation(IOperation operation)
    {
        return Obj(
            ("children", operation.ChildOperations.Select(OperationSummary).Cast<object?>().ToList()),
            ("constant", ObserveConstant(operation.ConstantValue)),
            ("details", OperationDetails(operation)),
            ("is_implicit", operation.IsImplicit),
            ("kind", operation.Kind.ToString()),
            ("span", Span(operation.Syntax.Span)),
            ("syntax_kind", operation.Syntax.Kind().ToString()),
            ("type", TypeIdentity(operation.Type)));
    }

    private static object? ObserveSymbol(ISymbol? symbol)
    {
        if (symbol is null)
        {
            return null;
        }
        List<object?> customModifiers = new();
        List<object?> parameters = new();
        string? methodKind = null;
        int? arity = null;
        object? returnType = null;
        bool? returnsVoid = null;
        bool? isRequired = null;
        bool? isReadOnly = null;
        bool? isConst = null;
        object? associatedSymbol = null;
        List<object?> accessors = new();
        object? baseType = null;
        List<object?> interfaces = new();
        List<object?> typeArguments = new();
        string? specialType = null;
        string? typeKind = null;
        string? nullableAnnotation = null;
        int? arrayRank = null;
        bool? isSzArray = null;
        object? elementType = null;

        if (symbol is IMethodSymbol method)
        {
            methodKind = method.MethodKind.ToString();
            arity = method.Arity;
            returnType = TypeIdentity(method.ReturnType);
            returnsVoid = method.ReturnsVoid;
            customModifiers = ObserveCustomModifiers(method.ReturnTypeCustomModifiers);
            parameters = method.Parameters.Select(ObserveParameter).Cast<object?>().ToList();
        }
        else if (symbol is IPropertySymbol property)
        {
            returnType = TypeIdentity(property.Type);
            isRequired = property.IsRequired;
            isReadOnly = property.IsReadOnly;
            parameters = property.Parameters.Select(ObserveParameter).Cast<object?>().ToList();
            accessors = new[] { property.GetMethod, property.SetMethod }
                .Where(accessor => accessor is not null)
                .Select(accessor => ObserveSymbol(accessor!))
                .Cast<object?>()
                .ToList();
        }
        else if (symbol is IFieldSymbol field)
        {
            returnType = TypeIdentity(field.Type);
            isRequired = field.IsRequired;
            isReadOnly = field.IsReadOnly;
            isConst = field.IsConst;
            associatedSymbol = SymbolIdentity(field.AssociatedSymbol);
            customModifiers = ObserveCustomModifiers(field.CustomModifiers);
        }
        else if (symbol is IParameterSymbol parameter)
        {
            returnType = TypeIdentity(parameter.Type);
            customModifiers = ObserveCustomModifiers(parameter.CustomModifiers);
        }
        else if (symbol is ILocalSymbol local)
        {
            returnType = TypeIdentity(local.Type);
            isConst = local.IsConst;
        }

        if (symbol is ITypeSymbol type)
        {
            typeKind = type.TypeKind.ToString();
            nullableAnnotation = type.NullableAnnotation.ToString();
            specialType = type.SpecialType.ToString();
            baseType = TypeIdentity(type.BaseType);
            interfaces = type.AllInterfaces.Select(TypeIdentity).Cast<object?>().ToList();
        }
        if (symbol is INamedTypeSymbol named)
        {
            arity = named.Arity;
            typeArguments = named.TypeArguments.Select(TypeIdentity).Cast<object?>().ToList();
        }
        if (symbol is IArrayTypeSymbol array)
        {
            arrayRank = array.Rank;
            isSzArray = array.IsSZArray;
            elementType = TypeIdentity(array.ElementType);
        }

        return Obj(
            ("accessors", accessors),
            ("accessibility", symbol.DeclaredAccessibility.ToString()),
            ("arity", arity),
            ("array_rank", arrayRank),
            ("associated_symbol", associatedSymbol),
            ("attributes", ObserveAttributes(symbol.GetAttributes())),
            ("base_type", baseType),
            ("can_be_referenced_by_name", symbol.CanBeReferencedByName),
            ("containing_assembly", symbol.ContainingAssembly?.Identity.ToString()),
            ("containing_symbol", SymbolIdentity(symbol.ContainingSymbol)),
            ("custom_modifiers", customModifiers),
            ("display", symbol.ToDisplayString(DisplayFormat)),
            ("element_type", elementType),
            ("interfaces", interfaces),
            ("is_abstract", symbol.IsAbstract),
            ("is_const", isConst),
            ("is_extern", symbol.IsExtern),
            ("is_implicit", symbol.IsImplicitlyDeclared),
            ("is_override", symbol.IsOverride),
            ("is_read_only", isReadOnly),
            ("is_required", isRequired),
            ("is_sealed", symbol.IsSealed),
            ("is_static", symbol.IsStatic),
            ("is_sz_array", isSzArray),
            ("is_virtual", symbol.IsVirtual),
            ("kind", symbol.Kind.ToString()),
            ("locations", ObserveLocations(symbol.Locations)),
            ("metadata_name", symbol.MetadataName),
            ("method_kind", methodKind),
            ("name", symbol.Name),
            ("nullable_annotation", nullableAnnotation),
            ("original_definition", SymbolIdentity(symbol.OriginalDefinition)),
            ("parameters", parameters),
            ("return_type", returnType),
            ("returns_void", returnsVoid),
            ("special_type", specialType),
            ("type_arguments", typeArguments),
            ("type_kind", typeKind));
    }

    private static object ObserveParameter(IParameterSymbol parameter)
    {
        return Obj(
            ("custom_modifiers", ObserveCustomModifiers(parameter.CustomModifiers)),
            ("has_explicit_default", parameter.HasExplicitDefaultValue),
            ("name", parameter.Name),
            ("ordinal", parameter.Ordinal),
            ("ref_custom_modifiers", ObserveCustomModifiers(parameter.RefCustomModifiers)),
            ("ref_kind", parameter.RefKind.ToString()),
            ("type", TypeIdentity(parameter.Type)));
    }

    private static List<object?> ObserveCustomModifiers(ImmutableArray<CustomModifier> modifiers)
    {
        return modifiers.Select(modifier => Obj(
            ("is_optional", modifier.IsOptional),
            ("modifier", TypeIdentity(modifier.Modifier)))).Cast<object?>().ToList();
    }

    private static List<object?> ObserveAttributes(ImmutableArray<AttributeData> attributes)
    {
        return attributes
            .OrderBy(attribute => attribute.AttributeClass?.ToDisplayString(DisplayFormat), StringComparer.Ordinal)
            .ThenBy(attribute => attribute.ApplicationSyntaxReference?.Span.Start ?? int.MaxValue)
            .Select(attribute => Obj(
                ("attribute_class", TypeIdentity(attribute.AttributeClass)),
                ("constructor", SymbolIdentity(attribute.AttributeConstructor)),
                ("constructor_arguments", attribute.ConstructorArguments.Select(ObserveTypedConstant).Cast<object?>().ToList()),
                ("is_source", attribute.ApplicationSyntaxReference is not null),
                ("named_arguments", attribute.NamedArguments
                    .OrderBy(pair => pair.Key, StringComparer.Ordinal)
                    .Select(pair => Obj(("name", pair.Key), ("value", ObserveTypedConstant(pair.Value))))
                    .Cast<object?>().ToList())))
            .Cast<object?>()
            .ToList();
    }

    private static object ObserveTypedConstant(TypedConstant constant)
    {
        return Obj(
            ("is_null", constant.IsNull),
            ("kind", constant.Kind.ToString()),
            ("type", TypeIdentity(constant.Type)),
            ("value", constant.Kind == TypedConstantKind.Array
                ? constant.Values.Select(ObserveTypedConstant).Cast<object?>().ToList()
                : ConstantText(constant.Value)));
    }

    private static List<object?> ObserveLocations(ImmutableArray<Location> locations)
    {
        return locations.Select(location => Obj(
            ("kind", location.Kind.ToString()),
            ("path", location.IsInSource ? location.SourceTree?.FilePath : null),
            ("span", location.IsInSource ? Span(location.SourceSpan) : null))).Cast<object?>().ToList();
    }

    private static object? TypeIdentity(ITypeSymbol? type)
    {
        if (type is null)
        {
            return null;
        }
        return Obj(
            ("display", type.ToDisplayString(DisplayFormat)),
            ("nullable_annotation", type.NullableAnnotation.ToString()),
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
            ("display", symbol.ToDisplayString(DisplayFormat)),
            ("kind", symbol.Kind.ToString()),
            ("metadata_name", symbol.MetadataName));
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

    private static IReadOnlyList<ProbeCase> Cases()
    {
        return new[]
        {
            new ProbeCase("concise-and-name-resolution", "admitted_shape", """
                /*@shape:nullable.directive.file_wide_enable*/
                #nullable enable
                /*@shape:using.namespace.compilation_unit*/using System;
                namespace Probe
                {
                    /*@shape:using.namespace.namespace_scope*/using System.Globalization;
                    public static class Concise
                    {
                        /*@shape:expression_body.method*/public static int Identity(int value) => value;
                        /*@shape:expression_body.getter*/public static int Answer => 42;
                        public static int Local()
                        {
                            /*@shape:var.local*/var value = 3;
                            return value;
                        }
                    }
                }
                """),
            new ProbeCase("data-declarations", "admitted_shape", """
                namespace Probe
                {
                    /*@shape:enum.default_underlying_int*/public enum State { None = 0, Ready = 1, /*@shape:enum.alias_member*/AlsoReady = 1 }
                    /*@shape:enum.explicit_underlying_byte*/public enum Code : byte { Zero = 0, One = 1 }
                    /*@shape:readonly_struct.declaration*/public readonly struct Amount
                    {
                        /*@shape:field.readonly*/private readonly int _value;
                        /*@shape:property.getter_only_auto*/public int Value { get; }
                        /*@shape:property.explicit_getter*/public int Doubled { get { return _value * 2; } }
                        /*@shape:constructor.primary*/public Amount(int value) { _value = value; Value = value; }
                        /*@shape:constructor.same_type_delegation*/public Amount(byte value) : this((int)value) { }
                        /*@shape:instance_method.declaration*/public int Add(int delta) { return _value + delta; }
                    }
                    /*@shape:sealed_class.declaration*/public sealed class Customer
                    {
                        public string Name { get; }
                        public Customer(string name) { Name = name; }
                        public int Pick(int value) { return value; }
                        public long Pick(long value) { return value; }
                        public int Use(Amount amount)
                        {
                            int first = /*@shape:instance_method.call*/amount.Add(1);
                            return first + /*@shape:overload_resolution.int*/Pick(2);
                        }
                    }
                }
                """),
            new ProbeCase("initialization-and-synthesis", "admitted_shape", """
                namespace Probe
                {
                    public sealed class Draft
                    {
                        /*@shape:init.auto_property*//*@shape:compiler_marker.init_modreq*//*@shape:synthesized.auto_backing_field*//*@shape:synthesized.property_accessors*/public int Count { get; init; }
                    }
                    public sealed class RequiredDraft
                    {
                        /*@shape:required.init_property*//*@shape:compiler_marker.required_attributes*/public required string Name { get; init; }
                    }
                    /*@shape:synthesized.default_constructor*/public sealed class Empty { }
                    public static class Factory
                    {
                        public static RequiredDraft Make()
                        {
                            return /*@shape:object_initializer.creation*/new RequiredDraft { Name = "ready" };
                        }
                    }
                }
                """),
            new ProbeCase("nullable", "admitted_shape", """
                #nullable enable
                namespace Probe
                {
                    public sealed class Child
                    {
                        public string Name { get; }
                        public Child(string name) { Name = name; }
                    }
                    public sealed class Parent
                    {
                        public Child Child { get; }
                        public Parent(Child child) { Child = child; }
                    }
                    public static class NullableData
                    {
                        public static /*@shape:nullable.reference_annotation*/string? Maybe(bool present) { return present ? "yes" : null; }
                        public static /*@shape:nullable.value_shorthand*/int? MaybeInt(int value) { return /*@shape:conversion.nullable*/value; }
                        public static bool Has(int? value) { return /*@shape:nullable.has_value*/value.HasValue; }
                        public static int Read(int? value) { return value.HasValue ? /*@shape:nullable.value*/value.Value : 0; }
                        public static int Defaulted(int? value) { return /*@shape:nullable.get_value_or_default*/value.GetValueOrDefault(); }
                        public static int Fallback(int? value) { return /*@shape:nullable.get_value_or_default_fallback*/value.GetValueOrDefault(7); }
                        public static Child? Access(Parent? value) { return /*@shape:conditional_access.member*/value?.Child; }
                        public static string CoalesceRef(string? value) { return /*@shape:coalesce.reference*/value ?? "fallback"; }
                        public static int CoalesceValue(int? value) { return /*@shape:coalesce.value*/value ?? 7; }
                        public static int ExactDefault() { return /*@shape:default.exact_type*/default(int); }
                    }
                }
                """),
            new ProbeCase("conversions-and-arrays", "admitted_shape", """
                namespace Probe
                {
                    public static class Arrays
                    {
                        public static int Identity(int value) { int copy = /*@shape:conversion.identity*/value; return copy; }
                        public static long Widen(int value) { return /*@shape:conversion.implicit_numeric*/value; }
                        public static int Narrow(long value) { return /*@shape:conversion.explicit_numeric*/(int)value; }
                        public static int[] Allocate(int length) { return /*@shape:array.creation.length*/new int[length]; }
                        public static int[] Explicit() { return /*@shape:array.initializer.explicit*/new int[] { 1, 2 }; }
                        public static int[] Local() { /*@shape:array.initializer.local*/int[] values = { 1, 2 }; return values; }
                        public static int[] Implicit() { return /*@shape:array.initializer.implicit_exact*/new[] { 1, 2 }; }
                        public static int Read(int[] values, int index)
                        {
                            int length = /*@shape:array.length*/values.Length;
                            return length + /*@shape:array.index.read*/values[index];
                        }
                        public static int[] Write(int[] values, int index, int value)
                        {
                            /*@shape:array.index.write*/values[index] = value;
                            return values;
                        }
                        public static int[]? ArrayMetadata(int[]? values) { return /*@shape:intrinsic.array.incidental_generic_metadata*/values; }
                        public static bool IsNull(int[]? values)
                        {
                            return /*@shape:array.null_equality*/values == null;
                        }
                    }
                }
                """),
            new ProbeCase("strings", "admitted_shape", """
                using System;
                namespace Probe
                {
                    public static class Strings
                    {
                        public static string Ordinary() { return /*@shape:string.literal.ordinary*/"alpha"; }
                        public static string Verbatim() { return /*@shape:string.literal.verbatim*/@"alpha\\beta"; }
                        public static string Escaped() { return /*@shape:string.literal.escaped*/"a\\uD800"; }
                        public static string StringMetadata(string value) { return /*@shape:intrinsic.string.incidental_generic_metadata*/value; }
                        public static int Length(string value) { return /*@shape:string.length*/value.Length; }
                        public static char Index(string value, int index) { return /*@shape:string.index*/value[index]; }
                        public static bool Equal(string? left, string? right) { return /*@shape:string.equality*/left == right; }
                        public static bool NotEqual(string? left, string? right) { return /*@shape:string.inequality*/left != right; }
                        public static bool OrdinalEqual(string? left, string? right) { return /*@shape:string.ordinal_equals*/string.Equals(left, right, /*@shape:intrinsic.string_comparison.ordinal*/StringComparison.Ordinal); }
                        public static int Compare(string? left, string? right) { return /*@shape:string.compare.ordinal*/string.Compare(left, right, StringComparison.Ordinal); }
                        public static bool Starts(string value, string prefix) { return /*@shape:string.startswith.ordinal*/value.StartsWith(prefix, StringComparison.Ordinal); }
                        public static bool Ends(string value, string suffix) { return /*@shape:string.endswith.ordinal*/value.EndsWith(suffix, StringComparison.Ordinal); }
                        public static bool Contains(string value, string part) { return /*@shape:string.contains.ordinal*/value.Contains(part, StringComparison.Ordinal); }
                        public static string AddSs(string left, string right) { return /*@shape:string.concat.operator.string_string*/left + right; }
                        public static string AddSc(string left, char right) { return /*@shape:string.concat.operator.string_char*/left + right; }
                        public static string AddCs(char left, string right) { return /*@shape:string.concat.operator.char_string*/left + right; }
                        public static string Concat2(string a, string b) { return /*@shape:string.concat.two*/string.Concat(a, b); }
                        public static string Concat3(string a, string b, string c) { return /*@shape:string.concat.three*/string.Concat(a, b, c); }
                        public static string Concat4(string a, string b, string c, string d) { return /*@shape:string.concat.four*/string.Concat(a, b, c, d); }
                        public static string Slice(string value, int start, int length) { return /*@shape:string.substring.start_length*/value.Substring(start, length); }
                        public static bool Empty(string? value) { return /*@shape:string.is_null_or_empty*/string.IsNullOrEmpty(value); }
                        public static string InterpolateString(string value) { return /*@shape:string.interpolation.string*/$"prefix:{value}"; }
                        public static string InterpolateChar(char value) { return /*@shape:string.interpolation.char*/$"prefix:{value}"; }
                    }
                }
                """),
            new ProbeCase("floating-and-decimal-intrinsics", "admitted_shape", """
                using System;
                namespace Probe
                {
                    public static class Numbers
                    {
                        public static bool SingleNaN(float value) { return /*@shape:intrinsic.single.is_nan*/float.IsNaN(value); }
                        public static bool SingleInfinity(float value) { return /*@shape:intrinsic.single.is_infinity*/float.IsInfinity(value); }
                        public static bool SingleFinite(float value) { return /*@shape:intrinsic.single.is_finite*/float.IsFinite(value); }
                        public static float SingleAbs(float value) { return /*@shape:intrinsic.single.abs*/MathF.Abs(value); }
                        public static float SingleMin(float left, float right) { return /*@shape:intrinsic.single.min*/MathF.Min(left, right); }
                        public static float SingleMax(float left, float right) { return /*@shape:intrinsic.single.max*/MathF.Max(left, right); }
                        public static bool DoubleNaN(double value) { return /*@shape:intrinsic.double.is_nan*/double.IsNaN(value); }
                        public static bool DoubleInfinity(double value) { return /*@shape:intrinsic.double.is_infinity*/double.IsInfinity(value); }
                        public static bool DoubleFinite(double value) { return /*@shape:intrinsic.double.is_finite*/double.IsFinite(value); }
                        public static double DoubleAbs(double value) { return /*@shape:intrinsic.double.abs*/Math.Abs(value); }
                        public static double DoubleMin(double left, double right) { return /*@shape:intrinsic.double.min*/Math.Min(left, right); }
                        public static double DoubleMax(double left, double right) { return /*@shape:intrinsic.double.max*/Math.Max(left, right); }
                        public static decimal Round(decimal value) { return /*@shape:intrinsic.decimal.round*/decimal.Round(value); }
                        public static decimal RoundDigits(decimal value, int digits) { return /*@shape:intrinsic.decimal.round_digits*/decimal.Round(value, digits); }
                        public static decimal RoundMode(decimal value) { return /*@shape:intrinsic.decimal.round_mode*/decimal.Round(value, /*@shape:intrinsic.midpoint_rounding.to_even*/MidpointRounding.ToEven); }
                        public static decimal RoundDigitsMode(decimal value, int digits) { return /*@shape:intrinsic.decimal.round_digits_mode*/decimal.Round(value, digits, MidpointRounding.AwayFromZero); }
                        public static decimal Truncate(decimal value) { return /*@shape:intrinsic.decimal.truncate*/decimal.Truncate(value); }
                        public static decimal Floor(decimal value) { return /*@shape:intrinsic.decimal.floor*/decimal.Floor(value); }
                        public static decimal Ceiling(decimal value) { return /*@shape:intrinsic.decimal.ceiling*/decimal.Ceiling(value); }
                    }
                }
                """),
            new ProbeCase("business-data-intrinsics", "admitted_shape", """
                using System;
                namespace Probe
                {
                    public static class BusinessData
                    {
                        public static DateOnly Date(int year, int month, int day) { return /*@shape:intrinsic.date_only.constructor*//*@shape:intrinsic.date_only.incidental_generic_metadata*/new DateOnly(year, month, day); }
                        public static int DateParts(DateOnly value) { return /*@shape:intrinsic.date_only.year*/value.Year + /*@shape:intrinsic.date_only.month*/value.Month + /*@shape:intrinsic.date_only.day*/value.Day + /*@shape:intrinsic.date_only.day_number*/value.DayNumber; }
                        public static DayOfWeek Weekday(DateOnly value) { return /*@shape:intrinsic.date_only.day_of_week*/value.DayOfWeek; }
                        public static bool DateLess(DateOnly left, DateOnly right) { return /*@shape:intrinsic.date_only.compare*/left < right; }
                        public static DateOnly AddDays(DateOnly value, int days) { return /*@shape:intrinsic.date_only.add_days*/value.AddDays(days); }
                        public static DateOnly AddMonths(DateOnly value, int months) { return /*@shape:intrinsic.date_only.add_months*/value.AddMonths(months); }
                        public static DateOnly AddYears(DateOnly value, int years) { return /*@shape:intrinsic.date_only.add_years*/value.AddYears(years); }
                        public static TimeOnly Time(int hour, int minute, int second, int millisecond) { return /*@shape:intrinsic.time_only.constructor*/new TimeOnly(hour, minute, second, millisecond); }
                        public static long TimeParts(TimeOnly value) { return /*@shape:intrinsic.time_only.hour*/value.Hour + /*@shape:intrinsic.time_only.minute*/value.Minute + /*@shape:intrinsic.time_only.second*/value.Second + /*@shape:intrinsic.time_only.millisecond*/value.Millisecond + /*@shape:intrinsic.time_only.ticks*/value.Ticks; }
                        public static bool TimeLess(TimeOnly left, TimeOnly right) { return /*@shape:intrinsic.time_only.compare*/left < right; }
                        public static TimeSpan TimeDifference(TimeOnly left, TimeOnly right) { return /*@shape:intrinsic.time_only.subtract*/left - right; }
                        public static TimeOnly TimeAdd(TimeOnly value, TimeSpan duration) { return /*@shape:intrinsic.time_only.add_duration*/value.Add(duration); }
                        public static TimeSpan Duration(long ticks) { return /*@shape:intrinsic.time_span.constructor*/new TimeSpan(ticks); }
                        public static long DurationParts(TimeSpan value) { return /*@shape:intrinsic.time_span.days*/value.Days + /*@shape:intrinsic.time_span.hours*/value.Hours + /*@shape:intrinsic.time_span.minutes*/value.Minutes + /*@shape:intrinsic.time_span.seconds*/value.Seconds + /*@shape:intrinsic.time_span.milliseconds*/value.Milliseconds + /*@shape:intrinsic.time_span.ticks*/value.Ticks; }
                        public static bool DurationLess(TimeSpan left, TimeSpan right) { return /*@shape:intrinsic.time_span.compare*/left < right; }
                        public static TimeSpan DurationAdd(TimeSpan left, TimeSpan right) { return /*@shape:intrinsic.time_span.add*/left + right; }
                        public static TimeSpan DurationSubtract(TimeSpan left, TimeSpan right) { return /*@shape:intrinsic.time_span.subtract*/left - right; }
                        public static TimeSpan DurationNegate(TimeSpan value) { return /*@shape:intrinsic.time_span.negate*/-value; }
                        public static Guid EmptyGuid() { return /*@shape:intrinsic.guid.empty*/Guid.Empty; }
                        public static bool GuidEqual(Guid left, Guid right) { return /*@shape:intrinsic.guid.equality*/left == right; }
                        public static bool GuidNotEqual(Guid left, Guid right) { return /*@shape:intrinsic.guid.inequality*/left != right; }
                        public static int GuidCompare(Guid left, Guid right) { return /*@shape:intrinsic.guid.compare_to*/left.CompareTo(right); }
                    }
                }
                """),
            new ProbeCase("near-miss-declarations", "rejected_near_miss", """
                using System;
                namespace Probe
                {
                    /*@shape:near_miss.struct.mutable*/public struct Mutable { public int Value; }
                    /*@shape:near_miss.class.unsealed*/public class Open { }
                    public sealed class Base { }
                    /*@shape:near_miss.class.source_base*/public sealed class Derived : Base { }
                    public sealed class Settable { /*@shape:near_miss.property.setter*/public int Value { get; set; } }
                    public sealed class CustomInit { private int _value; /*@shape:near_miss.property.custom_init*/public int Value { get { return _value; } init { _value = value; } } }
                    public static class GenericCalls
                    {
                        public static T /*@shape:near_miss.generic.method*/Identity<T>(T value) { return value; }
                        public static int UserConversion(long value) { return /*@shape:near_miss.conversion.user_defined*/new Convertible(value); }
                    }
                    public readonly struct Convertible
                    {
                        private readonly long _value;
                        public Convertible(long value) { _value = value; }
                        public static implicit operator int(Convertible value) { return (int)value._value; }
                    }
                }
                """),
            new ProbeCase("near-miss-concise-and-construction", "rejected_near_miss", """
                /*@shape:near_miss.using.global*/
                global using System;
                /*@shape:near_miss.using.static_directive*/
                using static System.Math;
                /*@shape:near_miss.using.alias_directive*/
                using Text = System.String;
                namespace Probe
                {
                    public sealed class Data
                    {
                        public int Value { get; init; }
                        /*@shape:near_miss.expression_body.constructor*/public Data(int value) => Value = value;
                    }
                    public sealed class RequiredWrong { /*@shape:near_miss.required.non_init*/public required string Name { get; set; } }
                    public static class Cases
                    {
                        public static object Anonymous() { /*@shape:near_miss.var.anonymous*/var value = new { X = 1 }; return value; }
                        public static dynamic Dynamic() { /*@shape:near_miss.var.dynamic*/var value = (dynamic)1; return value; }
                        public static int Multiple() { /*@shape:near_miss.var.multiple_declarators*/var first = 1, second = 2; return first + second; }
                        public static Data TargetTyped() { return /*@shape:near_miss.constructor.target_typed_new*/new(1); }
                        public static Data Named() { return /*@shape:near_miss.constructor.named_argument*/new Data(value: 1); }
                        public static Data ConstructorRewrite() { return /*@shape:near_miss.object_initializer.constructor_rewrite*/new Data(1) { Value = 2 }; }
                        public static Data DuplicateTarget() { return /*@shape:near_miss.object_initializer.duplicate_target*/new Data(1) { Value = 2, Value = 3 }; }
                    }
                    public static class Directives
                    {
                        public static double StaticUsing(double value) { return /*@shape:near_miss.using.static*/Abs(value); }
                        public static /*@shape:near_miss.using.alias*/Text Alias(Text value) { return value; }
                        public static void Disposal() { /*@shape:near_miss.using.disposal*/using var stream = new System.IO.MemoryStream(); }
                    }
                }
                """),
            new ProbeCase("near-miss-nullable", "rejected_near_miss", """
                /*@shape:near_miss.nullable.directive_disable*/
                #nullable disable
                namespace Probe
                {
                    public sealed class Child { public string Name { get; } = "x"; }
                    public sealed class Parent { public Child Child { get; } = new Child(); }
                    public static class NullableCases
                    {
                        public static /*@shape:near_miss.nullable.explicit_system_nullable*/System.Nullable<int> Explicit(int value) { return value; }
                        public static int? Construct(int value) { return /*@shape:near_miss.nullable.new_nullable*/new int?(value); }
                        public static string? Chain(Parent? value) { return /*@shape:near_miss.conditional_access.chain*/value?.Child?.Name; }
                        public static int? ValueMember(int? value) { return /*@shape:near_miss.conditional_access.value_member*/value?.GetHashCode(); }
                        public static int Assign(ref int? value) { return /*@shape:near_miss.coalesce.assignment*/value ??= 1; }
                    }
                }
                """),
            new ProbeCase("near-miss-arrays-and-collections", "rejected_near_miss", """
                using System;
                using System.Collections.Generic;
                using System.Collections.Immutable;
                using System.Linq;
                namespace Probe
                {
                    public static class Collections
                    {
                        public static int[,] Multi() { return /*@shape:near_miss.array.multidimensional*/new int[1, 1]; }
                        public static int[][] Jagged() { return /*@shape:near_miss.array.jagged*/new int[1][]; }
                        public static int[] Empty() { return /*@shape:near_miss.collection.array_empty_generic_call*/Array.Empty<int>(); }
                        public static long[] BestCommon() { return /*@shape:near_miss.array.implicit_best_common_type*/new[] { 1L, 2 }; }
                        public static int Slice(int[] values) { return /*@shape:near_miss.array.range*/values[1..].Length; }
                        public static int[] Expression() { return /*@shape:near_miss.array.collection_expression*/[1, 2]; }
                        public static Span<int> Stack() { return /*@shape:near_miss.array.stackalloc*/stackalloc int[2]; }
                        public static List<int> List() { var value = new List<int>(); /*@shape:near_miss.collection.list_add*/value.Add(1); return value; }
                        public static bool Dictionary(Dictionary<int, int> value) { return /*@shape:near_miss.collection.dictionary_contains_key*/value.ContainsKey(1); }
                        public static bool Set(HashSet<int> value) { return /*@shape:near_miss.collection.hash_set_add*/value.Add(1); }
                        public static int[] Linq(int[] value) { return /*@shape:near_miss.collection.linq_select*/value.Select(item => item + 1).ToArray(); }
                        public static ImmutableArray<int> Immutable() { return /*@shape:near_miss.collection.immutable_array_create*/ImmutableArray.Create(1); }
                        public static IEnumerable<char> StringInterface(/*@shape:near_miss.intrinsic.string_generic_interface*/string value) { return value; }
                        public static IList<int> ArrayInterface(/*@shape:near_miss.intrinsic.array_generic_interface*/int[] value) { return value; }
                    }
                }
                """),
            new ProbeCase("near-miss-strings", "rejected_near_miss", """
                using System;
                namespace Probe
                {
                    public static class StringCases
                    {
                        public static string Numeric(int value) { return /*@shape:near_miss.string.interpolation_numeric*/$"{value}"; }
                        public static string Alignment(string value) { return /*@shape:near_miss.string.interpolation_alignment*/$"{value,8}"; }
                        public static string Format(int value) { return /*@shape:near_miss.string.interpolation_format*/$"{value:D2}"; }
                        public static int CharAdd(char left, char right) { return /*@shape:near_miss.string.concat_char_char*/left + right; }
                        public static string ObjectAdd(string left, object right) { return /*@shape:near_miss.string.concat_object*/left + right; }
                        public static int Compare(string left, string right) { return /*@shape:near_miss.string.compare_nonordinal*/string.Compare(left, right); }
                        public static string General(object value) { return /*@shape:near_miss.string.general_to_string*/value.ToString()!; }
                    }
                }
                """),
            new ProbeCase("near-miss-directives", "rejected_near_miss", """
                /*@shape:near_miss.nullable.directive_annotations_only*/
                #nullable enable annotations
                namespace Probe
                {
                    public static class Scoped
                    {
                        /*@shape:near_miss.nullable.directive_restore*/
                        #nullable restore
                        public static /*@shape:near_miss.nullable.directive_scoped*/string? Value() { return null; }
                    }
                }
                """),
        };
    }

    private sealed class ProbeFailure : Exception
    {
        internal ProbeFailure(string code) => Code = code;
        internal string Code { get; }
    }
}
