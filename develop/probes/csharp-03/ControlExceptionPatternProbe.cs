// Disposable CSHARP-03-T01-W05 public Roslyn API probe; never a frontend.
#nullable enable

using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
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

internal static class ControlExceptionPatternProbe
{
    private const string RawSchema = "mpk.csharp_practical.t01_w05.roslyn_control_probe.raw.v0";
    private const string WorkItem = "CSHARP-03-T01-W05";
    private const string MarkerPrefix = "/*@shape:";
    private const string MarkerSuffix = "*/";

    private static readonly SymbolDisplayFormat DisplayFormat =
        SymbolDisplayFormat.CSharpErrorMessageFormat.WithMiscellaneousOptions(
            SymbolDisplayFormat.CSharpErrorMessageFormat.MiscellaneousOptions
            | SymbolDisplayMiscellaneousOptions.IncludeNullableReferenceTypeModifier
            | SymbolDisplayMiscellaneousOptions.EscapeKeywordIdentifiers);

    private sealed record ProbeCase(string Id, string Disposition, string Source);

    public static int Main(string[] args)
    {
        if (args.Length != 1)
        {
            Console.Error.Write("CSHARP_PRACTICAL_CONTROL_PROBE_USAGE\n");
            return 64;
        }

        try
        {
            CultureInfo.CurrentCulture = CultureInfo.InvariantCulture;
            CultureInfo.CurrentUICulture = CultureInfo.InvariantCulture;
            ImmutableArray<MetadataReference> references = LoadReferences(args[0]);
            List<object?> cases = Cases()
                .Select(probeCase => ObserveCase(probeCase, references))
                .Cast<object?>()
                .ToList();
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
            Console.Error.Write("CSHARP_PRACTICAL_CONTROL_PROBE_" + failure.Code + "\n");
            return 65;
        }
        catch (Exception)
        {
            Console.Error.Write("CSHARP_PRACTICAL_CONTROL_PROBE_UNEXPECTED\n");
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

    private static object ObserveCase(
        ProbeCase probeCase,
        ImmutableArray<MetadataReference> references)
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
        SyntaxTree tree = CSharpSyntaxTree.ParseText(
            text,
            parseOptions,
            path,
            CancellationToken.None);
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
        if (probeCase.Disposition == "admitted_shape" && diagnostics.Length != 0)
        {
            string diagnosticIds = string.Join(
                "_",
                diagnostics.Select(diagnostic => diagnostic.Id)
                    .Distinct(StringComparer.Ordinal)
                    .OrderBy(id => id, StringComparer.Ordinal));
            throw new ProbeFailure(
                "ADMITTED_CASE_DIAGNOSTIC_"
                + probeCase.Id.Replace('-', '_').ToUpperInvariant()
                + "_"
                + diagnosticIds);
        }

        List<IOperation> roots = OperationRoots(root, model);
        List<object?> decisionGraphs = ObserveDecisionGraphs(probeCase.Id, roots);
        List<object?> exceptionRegions = ObserveExceptionRegions(probeCase.Id, roots);
        List<object?> abruptCompletions = ObserveAbruptCompletions(probeCase.Id, roots);
        List<object?> targets = ObserveTargets(probeCase.Source, root, model);
        if (targets.Count == 0)
        {
            throw new ProbeFailure("MISSING_TARGET");
        }
        return Obj(
            ("abrupt_completions", abruptCompletions),
            ("compiler_outcome", hasErrors ? "error" : "success"),
            ("control_flow_graphs", ObserveControlFlowGraphs(probeCase.Id, roots, hasErrors)),
            ("decision_graphs", decisionGraphs),
            ("diagnostics", diagnostics.Select(ObserveDiagnostic).Cast<object?>().ToList()),
            ("disposition", probeCase.Disposition),
            ("exception_regions", exceptionRegions),
            ("id", probeCase.Id),
            ("operation_roots", roots.Select(ObserveOperation).Cast<object?>().ToList()),
            ("source", probeCase.Source),
            ("source_order", ObserveSourceOrder(targets, decisionGraphs, exceptionRegions, abruptCompletions)),
            ("source_utf8_sha256", Hex(SHA256.HashData(Encoding.UTF8.GetBytes(probeCase.Source)))),
            ("syntax", ObserveSyntax(root)),
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
            string key = operation.Kind + ":"
                + operation.Syntax.Span.Start.ToString(CultureInfo.InvariantCulture)
                + ":"
                + operation.Syntax.Span.Length.ToString(CultureInfo.InvariantCulture);
            result.TryAdd(key, operation);
        }
        return result.Values
            .OrderBy(operation => operation.Syntax.Span.Start)
            .ThenBy(operation => operation.Syntax.Span.Length)
            .ThenBy(operation => operation.Kind.ToString(), StringComparer.Ordinal)
            .ToList();
    }

    private static IEnumerable<IOperation> Operations(IOperation root)
    {
        yield return root;
        foreach (IOperation child in root.ChildOperations)
        {
            foreach (IOperation nested in Operations(child))
            {
                yield return nested;
            }
        }
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
            ("syntax_kind", operation.Syntax.Kind().ToString()),
            ("type", TypeIdentity(operation.Type)));
    }

    private static object OperationDetails(IOperation operation)
    {
        return operation switch
        {
            ILoopOperation value => Obj(
                ("continue_label", value.ContinueLabel.Name),
                ("exit_label", value.ExitLabel?.Name),
                ("loop_kind", value.LoopKind.ToString())),
            IBranchOperation value => Obj(
                ("branch_kind", value.BranchKind.ToString()),
                ("target", value.Target.Name)),
            ISwitchOperation value => Obj(
                ("case_count", value.Cases.Length),
                ("exit_label", value.ExitLabel.Name)),
            ISwitchExpressionOperation value => Obj(
                ("arm_count", value.Arms.Length),
                ("is_exhaustive", value.IsExhaustive)),
            ISwitchCaseOperation value => Obj(
                ("body_count", value.Body.Length),
                ("clause_count", value.Clauses.Length),
                ("local_count", value.Locals.Length)),
            ISwitchExpressionArmOperation value => Obj(("local_count", value.Locals.Length)),
            IPatternOperation value => Obj(
                ("input_type", TypeIdentity(value.InputType)),
                ("narrowed_type", TypeIdentity(value.NarrowedType))),
            ITryOperation value => Obj(
                ("catch_count", value.Catches.Length),
                ("exit_label", value.ExitLabel?.Name),
                ("has_finally", value.Finally is not null)),
            ICatchClauseOperation value => Obj(
                ("exception_type", TypeIdentity(value.ExceptionType)),
                ("has_filter", value.Filter is not null),
                ("local_count", value.Locals.Length)),
            IThrowOperation value => Obj(("is_rethrow", value.Exception is null)),
            IReturnOperation value => Obj(("has_value", value.ReturnedValue is not null)),
            IInvocationOperation value => Obj(("target", SymbolIdentity(value.TargetMethod))),
            IObjectCreationOperation value => Obj(("constructor", SymbolIdentity(value.Constructor))),
            IPropertyReferenceOperation value => Obj(("property", SymbolIdentity(value.Property))),
            IFieldReferenceOperation value => Obj(("field", SymbolIdentity(value.Field))),
            ILocalReferenceOperation value => Obj(("local", SymbolIdentity(value.Local))),
            IParameterReferenceOperation value => Obj(("parameter", SymbolIdentity(value.Parameter))),
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
            ("value", value switch
            {
                null => null,
                char character => ((int)character).ToString(CultureInfo.InvariantCulture),
                IFormattable formattable => formattable.ToString(null, CultureInfo.InvariantCulture),
                _ => value.ToString(),
            }));
    }

    private static List<object?> ObserveDecisionGraphs(string caseId, IEnumerable<IOperation> roots)
    {
        List<object?> result = new();
        int graphOrdinal = 0;
        foreach (IOperation root in roots)
        {
            List<IOperation> selected = Operations(root)
                .Where(IsDecisionOperation)
                .OrderBy(operation => operation.Syntax.Span.Start)
                .ThenByDescending(operation => operation.Syntax.Span.Length)
                .ThenBy(operation => operation.Kind.ToString(), StringComparer.Ordinal)
                .ToList();
            if (selected.Count == 0)
            {
                continue;
            }
            string graphId = caseId + "#decision-graph#"
                + graphOrdinal.ToString("D4", CultureInfo.InvariantCulture);
            Dictionary<IOperation, string> ids = new(ReferenceEqualityComparer.Instance);
            for (int index = 0; index < selected.Count; index++)
            {
                ids.Add(selected[index], graphId + "#node#"
                    + index.ToString("D4", CultureInfo.InvariantCulture));
            }
            List<object?> nodes = new();
            List<object?> edges = new();
            for (int index = 0; index < selected.Count; index++)
            {
                IOperation operation = selected[index];
                IOperation? parent = operation.Parent;
                while (parent is not null && !ids.ContainsKey(parent))
                {
                    parent = parent.Parent;
                }
                string? parentId = parent is null ? null : ids[parent];
                nodes.Add(Obj(
                    ("details", OperationDetails(operation)),
                    ("id", ids[operation]),
                    ("is_implicit", operation.IsImplicit),
                    ("operation_kind", operation.Kind.ToString()),
                    ("parent_id", parentId),
                    ("source_ordinal", index),
                    ("span", Span(operation.Syntax.Span)),
                    ("syntax_kind", operation.Syntax.Kind().ToString())));
                if (parentId is not null)
                {
                    edges.Add(Obj(
                        ("from", parentId),
                        ("kind", "operation_parent"),
                        ("to", ids[operation])));
                }
            }
            result.Add(Obj(
                ("edges", edges),
                ("id", graphId),
                ("nodes", nodes),
                ("root", OperationSummary(root)),
                ("source_ordinal", graphOrdinal)));
            graphOrdinal++;
        }
        return result;
    }

    private static bool IsDecisionOperation(IOperation operation)
    {
        return operation is ILoopOperation
            or IBranchOperation
            or ISwitchOperation
            or ISwitchExpressionOperation
            or ISwitchCaseOperation
            or ISwitchExpressionArmOperation
            or IPatternOperation
            || IsGuardOrFilterOperation(operation);
    }

    private static bool IsGuardOrFilterOperation(IOperation operation)
    {
        return operation.Parent switch
        {
            ICatchClauseOperation catchClause => ReferenceEquals(catchClause.Filter, operation),
            IPatternCaseClauseOperation caseClause => ReferenceEquals(caseClause.Guard, operation),
            ISwitchExpressionArmOperation arm => ReferenceEquals(arm.Guard, operation),
            _ => false,
        };
    }

    private static List<object?> ObserveExceptionRegions(string caseId, IEnumerable<IOperation> roots)
    {
        List<ITryOperation> tries = roots
            .SelectMany(Operations)
            .OfType<ITryOperation>()
            .OrderBy(operation => operation.Syntax.Span.Start)
            .ThenBy(operation => operation.Syntax.Span.Length)
            .ToList();
        List<object?> result = new();
        for (int index = 0; index < tries.Count; index++)
        {
            ITryOperation operation = tries[index];
            string id = caseId + "#exception-region#"
                + index.ToString("D4", CultureInfo.InvariantCulture);
            List<object?> catches = operation.Catches
                .OrderBy(catchClause => catchClause.Syntax.Span.Start)
                .Select((catchClause, catchIndex) => Obj(
                    ("exception_declaration", catchClause.ExceptionDeclarationOrExpression is null
                        ? null
                        : ObserveOperation(catchClause.ExceptionDeclarationOrExpression)),
                    ("exception_type", TypeIdentity(catchClause.ExceptionType)),
                    ("filter", catchClause.Filter is null ? null : ObserveOperation(catchClause.Filter)),
                    ("handler", ObserveOperation(catchClause.Handler)),
                    ("locals", catchClause.Locals.Select(SymbolIdentity).Cast<object?>().ToList()),
                    ("source_ordinal", catchIndex),
                    ("span", Span(catchClause.Syntax.Span))))
                .Cast<object?>()
                .ToList();
            List<object?> throws = Operations(operation)
                .OfType<IThrowOperation>()
                .OrderBy(throwOperation => throwOperation.Syntax.Span.Start)
                .Select((throwOperation, throwIndex) => Obj(
                    ("is_rethrow", throwOperation.Exception is null),
                    ("operation", ObserveOperation(throwOperation)),
                    ("source_ordinal", throwIndex),
                    ("span", Span(throwOperation.Syntax.Span))))
                .Cast<object?>()
                .ToList();
            result.Add(Obj(
                ("body", ObserveOperation(operation.Body)),
                ("catches", catches),
                ("finally", operation.Finally is null ? null : ObserveOperation(operation.Finally)),
                ("handler_search_order", catches.Select((_, catchIndex) => catchIndex).Cast<object?>().ToList()),
                ("id", id),
                ("nesting_depth", EnclosingTryDepth(operation)),
                ("source_ordinal", index),
                ("span", Span(operation.Syntax.Span)),
                ("throws", throws)));
        }
        return result;
    }

    private static int EnclosingTryDepth(ITryOperation operation)
    {
        int depth = 0;
        for (IOperation? parent = operation.Parent; parent is not null; parent = parent.Parent)
        {
            if (parent is ITryOperation)
            {
                depth++;
            }
        }
        return depth;
    }

    private static List<object?> ObserveAbruptCompletions(string caseId, IEnumerable<IOperation> roots)
    {
        List<IOperation> operations = roots
            .SelectMany(Operations)
            .Where(operation => operation is IBranchOperation or IReturnOperation or IThrowOperation)
            .OrderBy(operation => operation.Syntax.Span.Start)
            .ThenBy(operation => operation.Syntax.Span.Length)
            .ThenBy(operation => operation.Kind.ToString(), StringComparer.Ordinal)
            .ToList();
        List<object?> result = new();
        for (int index = 0; index < operations.Count; index++)
        {
            IOperation operation = operations[index];
            result.Add(Obj(
                ("completion_kind", operation switch
                {
                    IBranchOperation branch => branch.BranchKind.ToString(),
                    IReturnOperation => "Return",
                    IThrowOperation value => value.Exception is null ? "Rethrow" : "Throw",
                    _ => throw new ProbeFailure("ABRUPT_KIND"),
                }),
                ("id", caseId + "#abrupt#" + index.ToString("D4", CultureInfo.InvariantCulture)),
                ("operation", ObserveOperation(operation)),
                ("source_ordinal", index),
                ("span", Span(operation.Syntax.Span))));
        }
        return result;
    }

    private static List<object?> ObserveControlFlowGraphs(
        string caseId,
        IEnumerable<IOperation> roots,
        bool hasCompilerErrors)
    {
        List<object?> result = new();
        int graphOrdinal = 0;
        foreach (IOperation root in roots)
        {
            ControlFlowGraph? graph;
            try
            {
                graph = root switch
                {
                    IMethodBodyOperation method => ControlFlowGraph.Create(method, CancellationToken.None),
                    IConstructorBodyOperation constructor => ControlFlowGraph.Create(constructor, CancellationToken.None),
                    _ => null,
                };
            }
            catch (ArgumentException)
            {
                if (!hasCompilerErrors)
                {
                    throw new ProbeFailure("CFG_CREATE");
                }
                graph = null;
            }
            if (graph is null)
            {
                continue;
            }
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
            result.Add(Obj(
                ("blocks", blocks),
                ("id", caseId + "#cfg#" + graphOrdinal.ToString("D4", CultureInfo.InvariantCulture)),
                ("original_operation", OperationSummary(graph.OriginalOperation)),
                ("regions", ObserveRegion(graph.Root, regionIds)),
                ("source_ordinal", graphOrdinal)));
            graphOrdinal++;
        }
        return result;
    }

    private static void AssignRegionIds(
        ControlFlowRegion region,
        Dictionary<ControlFlowRegion, int> ids)
    {
        ids.Add(region, ids.Count);
        foreach (ControlFlowRegion nested in region.NestedRegions)
        {
            AssignRegionIds(nested, ids);
        }
    }

    private static object ObserveRegion(
        ControlFlowRegion region,
        Dictionary<ControlFlowRegion, int> ids)
    {
        return Obj(
            ("capture_ids", region.CaptureIds.Select(id => id.ToString()).Cast<object?>().ToList()),
            ("exception_type", TypeIdentity(region.ExceptionType)),
            ("first_block", region.FirstBlockOrdinal),
            ("id", ids[region]),
            ("kind", region.Kind.ToString()),
            ("last_block", region.LastBlockOrdinal),
            ("locals", region.Locals.Select(SymbolIdentity).Cast<object?>().ToList()),
            ("nested", region.NestedRegions.Select(item => ObserveRegion(item, ids)).Cast<object?>().ToList()));
    }

    private static object? ObserveBranch(
        ControlFlowBranch? branch,
        Dictionary<ControlFlowRegion, int> ids)
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

    private static List<object?> ObserveTargets(
        string source,
        SyntaxNode root,
        SemanticModel model)
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
            SyntaxToken nextToken = root
                .DescendantTokens(descendIntoTrivia: true)
                .FirstOrDefault(token => !token.IsMissing && token.SpanStart >= after);
            if (nextToken.RawKind == 0)
            {
                throw new ProbeFailure("MARKER_TOKEN");
            }
            int targetStart = nextToken.SpanStart;
            IEnumerable<SyntaxNode> candidates = root
                .DescendantNodesAndSelf(descendIntoTrivia: true)
                .Where(node => node != root && node.SpanStart == targetStart && !node.IsMissing);
            SyntaxKind? preferredSyntaxKind = PreferredSyntaxKind(id);
            SyntaxNode? target = preferredSyntaxKind.HasValue
                ? candidates
                    .Where(node => node.Kind() == preferredSyntaxKind.Value)
                    .OrderBy(node => node.SpanStart)
                    .ThenByDescending(node => node.Span.Length)
                    .FirstOrDefault()
                : null;
            target ??= PreferPatternTarget(id)
                ? candidates
                    .Where(node => model.GetOperation(node, CancellationToken.None) is IPatternOperation)
                    .OrderBy(node => node.SpanStart)
                    .ThenByDescending(node => node.Span.Length)
                    .FirstOrDefault()
                : null;
            target ??= candidates
                .OrderBy(node => node.SpanStart)
                .ThenByDescending(node => node.Span.Length)
                .FirstOrDefault();
            if (target is null)
            {
                throw new ProbeFailure("MARKER_TARGET");
            }
            IOperation? operation = model.GetOperation(target, CancellationToken.None);
            SymbolInfo symbolInfo = target is ExpressionSyntax || target is TypeSyntax
                ? model.GetSymbolInfo(target, CancellationToken.None)
                : default;
            TypeInfo typeInfo = target is ExpressionSyntax || target is TypeSyntax
                ? model.GetTypeInfo(target, CancellationToken.None)
                : default;
            result.Add(Obj(
                ("candidate_reason", symbolInfo.CandidateReason.ToString()),
                ("candidate_symbols", symbolInfo.CandidateSymbols.Select(SymbolIdentity).Cast<object?>().ToList()),
                ("marker_span", Span(new TextSpan(start, after - start))),
                ("operation", operation is null ? null : ObserveOperation(operation)),
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

    private static SyntaxKind? PreferredSyntaxKind(string id)
    {
        return id switch
        {
            "exception.propagation.call_to_handler" => SyntaxKind.InvocationExpression,
            "pattern.and" => SyntaxKind.AndPattern,
            "pattern.parenthesized" => SyntaxKind.ParenthesizedPattern,
            "pattern.relational.greater" => SyntaxKind.RelationalPattern,
            _ => null,
        };
    }

    private static bool PreferPatternTarget(string id)
    {
        if (id.StartsWith("pattern.", StringComparison.Ordinal))
        {
            return !id.StartsWith("pattern.guard.", StringComparison.Ordinal)
                && id != "pattern.list.is_expression";
        }
        return id is "near_miss.pattern.positional_deconstruct"
            or "near_miss.pattern.open_hierarchy"
            or "near_miss.pattern.extension_deconstruct";
    }

    private static List<object?> ObserveSourceOrder(
        IEnumerable<object?> targets,
        IEnumerable<object?> decisionGraphs,
        IEnumerable<object?> exceptionRegions,
        IEnumerable<object?> abruptCompletions)
    {
        List<(int Start, string Category, string Id)> entries = new();
        foreach (Dictionary<string, object?> target in targets.Cast<Dictionary<string, object?>>())
        {
            Dictionary<string, object?> span = (Dictionary<string, object?>)target["marker_span"]!;
            entries.Add(((int)span["start"]!, "target", (string)target["shape_id"]!));
        }
        foreach (Dictionary<string, object?> graph in decisionGraphs.Cast<Dictionary<string, object?>>())
        {
            Dictionary<string, object?> root = (Dictionary<string, object?>)graph["root"]!;
            Dictionary<string, object?> span = (Dictionary<string, object?>)root["span"]!;
            entries.Add(((int)span["start"]!, "decision_graph", (string)graph["id"]!));
        }
        foreach (Dictionary<string, object?> region in exceptionRegions.Cast<Dictionary<string, object?>>())
        {
            Dictionary<string, object?> span = (Dictionary<string, object?>)region["span"]!;
            entries.Add(((int)span["start"]!, "exception_region", (string)region["id"]!));
        }
        foreach (Dictionary<string, object?> abrupt in abruptCompletions.Cast<Dictionary<string, object?>>())
        {
            Dictionary<string, object?> span = (Dictionary<string, object?>)abrupt["span"]!;
            entries.Add(((int)span["start"]!, "abrupt_completion", (string)abrupt["id"]!));
        }
        return entries
            .OrderBy(entry => entry.Start)
            .ThenBy(entry => entry.Category, StringComparer.Ordinal)
            .ThenBy(entry => entry.Id, StringComparer.Ordinal)
            .Select((entry, index) => Obj(
                ("category", entry.Category),
                ("id", entry.Id),
                ("source_ordinal", index),
                ("start", entry.Start)))
            .Cast<object?>()
            .ToList();
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
            new ProbeCase("admitted-loops", "admitted_shape", """
                namespace Probe
                {
                    public static class Loops
                    {
                        public static int While(int[] values)
                        {
                            int index = 0;
                            int total = 0;
                            /*@shape:loop.while.statement*/while (index < values.Length)
                            {
                                index++;
                                if (index == 2) { /*@shape:abrupt.continue.while*/continue; }
                                if (index == 5) { /*@shape:abrupt.break.while*/break; }
                                total += values[index - 1];
                            }
                            /*@shape:abrupt.return.after_while*/return total;
                        }
                        public static int Do(int value)
                        {
                            int count = 0;
                            /*@shape:loop.do.statement*/do { count++; } while (count < value);
                            return count;
                        }
                        public static int For(int[] values)
                        {
                            int total = 0;
                            /*@shape:loop.for.statement*/for (int index = 0; index < values.Length; index++)
                            {
                                if (values[index] < 0) { /*@shape:abrupt.continue.for*/continue; }
                                total += values[index];
                            }
                            return total;
                        }
                        public static int ForeachArray(int[] values)
                        {
                            int total = 0;
                            /*@shape:loop.foreach.array_explicit*/foreach (int value in values) { total += value; }
                            return total;
                        }
                        public static int ForeachArrayVar(int[] values)
                        {
                            int total = 0;
                            /*@shape:loop.foreach.array_var*/foreach (var value in values) { total += value; }
                            return total;
                        }
                        public static int ForeachString(string value)
                        {
                            int total = 0;
                            /*@shape:loop.foreach.string*/foreach (char character in value) { total += character; }
                            return total;
                        }
                    }
                }
                """),
            new ProbeCase("admitted-switch-statements", "admitted_shape", """
                namespace Probe
                {
                    public enum State { None = 0, Ready = 1, Done = 2 }
                    public static class Switches
                    {
                        public static int Integer(int value, bool enabled)
                        {
                            /*@shape:switch.statement.integer*/switch (value)
                            {
                                case /*@shape:switch.case_clause.constant_integer*/0:
                                    return 0;
                                case /*@shape:pattern.and*//*@shape:pattern.relational.greater*/> 0 and < 10 when /*@shape:pattern.guard.statement*/enabled:
                                    return 1;
                                case /*@shape:pattern.var.statement*/var other when other >= 10:
                                    return 2;
                                default:
                                    return -1;
                            }
                        }
                        public static int Enum(State value)
                        {
                            /*@shape:switch.statement.enum*/switch (value)
                            {
                                case State.None: return 0;
                                case State.Ready: return 1;
                                case State.Done: return 2;
                                default: return -1;
                            }
                        }
                        public static int Text(string? value)
                        {
                            /*@shape:switch.statement.string*/switch (value)
                            {
                                case null: return 0;
                                case "ready": return 1;
                                default: return 2;
                            }
                        }
                    }
                }
                """),
            new ProbeCase("admitted-switch-expressions", "admitted_shape", """
                namespace Probe
                {
                    public static class Expressions
                    {
                        public static int Nullable(int? value, bool enabled)
                        {
                            return /*@shape:switch.expression.nullable*/value switch
                            {
                                /*@shape:pattern.null*/null => 0,
                                /*@shape:pattern.declaration.value*/int number when /*@shape:pattern.guard.expression*/enabled && number > 0 => number,
                                /*@shape:pattern.not_null*/not null => -1,
                            };
                        }
                        public static int Logical(int value)
                        {
                            return /*@shape:switch.expression.logical*/value switch
                            {
                                /*@shape:pattern.parenthesized*/(> 0 and < 10) => 1,
                                /*@shape:pattern.or*/< -10 or > 10 => 2,
                                /*@shape:pattern.not*/not 0 => 3,
                                /*@shape:pattern.discard*/_ => 0,
                            };
                        }
                        public static int Constant(int value)
                        {
                            return value switch
                            {
                                /*@shape:pattern.constant.integer*/0 => 0,
                                _ => 1,
                            };
                        }
                    }
                }
                """),
            new ProbeCase("admitted-type-and-property-patterns", "admitted_shape", """
                namespace Probe
                {
                    public enum Tag { Empty = 0, Value = 1 }
                    public sealed class Customer
                    {
                        public int Level { get; }
                        public Customer(int level) { Level = level; }
                    }
                    public sealed class Outcome
                    {
                        public Tag Tag { get; }
                        public int Payload { get; }
                        public Outcome(Tag tag, int payload) { Tag = tag; Payload = payload; }
                    }
                    public static class Patterns
                    {
                        public static int Type(Customer? value)
                        {
                            return value switch
                            {
                                /*@shape:pattern.type.exact*/Customer => 1,
                                null => 0,
                            };
                        }
                        public static int Declaration(Customer? value)
                        {
                            return value switch
                            {
                                /*@shape:pattern.declaration.reference*/Customer customer => customer.Level,
                                null => 0,
                            };
                        }
                        public static int Property(Customer? value)
                        {
                            return value switch
                            {
                                /*@shape:pattern.property.pure_getter*/Customer { Level: > 0 } => 1,
                                Customer { Level: 0 } => 0,
                                _ => -1,
                            };
                        }
                        public static int BoundTag(Outcome value)
                        {
                            return value switch
                            {
                                /*@shape:pattern.property.bound_tag*/{ Tag: Tag.Value, Payload: var payload } => payload,
                                { Tag: Tag.Empty } => 0,
                                _ => -1,
                            };
                        }
                    }
                }
                """),
            new ProbeCase("admitted-list-patterns", "admitted_shape", """
                namespace Probe
                {
                    public static class Lists
                    {
                        public static int Match(int[] values)
                        {
                            return /*@shape:switch.expression.list*/values switch
                            {
                                /*@shape:pattern.list.empty*/[] => 0,
                                /*@shape:pattern.list.single*/[var first] => first,
                                /*@shape:pattern.list.relational*/[1, > 0] => 2,
                                _ => -1,
                            };
                        }
                        public static bool IsPair(int[] values)
                        {
                            return /*@shape:pattern.list.is_expression*/values is [_, _];
                        }
                    }
                }
                """),
            new ProbeCase("admitted-throws-and-catches", "admitted_shape", """
                using System;
                namespace Probe
                {
                    public sealed class DomainException : Exception
                    {
                        public int Code { get; }
                        public DomainException(int code) /*@shape:exception.source.parameterless_base*/: base() { Code = code; }
                    }
                    public sealed class EmptyDomainException : Exception
                    {
                        /*@shape:exception.source.implicit_parameterless_base*/public EmptyDomainException() { }
                    }
                    public static class Exceptions
                    {
                        public static void ThrowBuiltIn()
                        {
                            /*@shape:exception.throw.builtin_parameterless*/throw new ArgumentException();
                        }
                        public static void ThrowSource(int code)
                        {
                            /*@shape:exception.throw.source_sealed*/throw new DomainException(code);
                        }
                        public static void ThrowImplicitSource()
                        {
                            /*@shape:exception.throw.source_implicit_base*/throw new EmptyDomainException();
                        }
                        public static int CatchPayload(int code)
                        {
                            try { /*@shape:exception.propagation.call_to_handler*/ThrowSource(code); return 0; }
                            /*@shape:exception.catch.typed_source*/catch (DomainException error) { return /*@shape:exception.catch.immutable_payload*/error.Code; }
                        }
                        public static int Ordered(int value)
                        {
                            try { if (value < 0) { throw new ArgumentOutOfRangeException(); } throw new ArgumentException(); }
                            /*@shape:exception.catch.lexical_first*/catch (ArgumentOutOfRangeException) { return 1; }
                            /*@shape:exception.catch.lexical_second*/catch (ArgumentException) { return 2; }
                        }
                    }
                }
                """),
            new ProbeCase("admitted-filters-and-finally", "admitted_shape", """
                using System;
                namespace Probe
                {
                    public static class Regions
                    {
                        private static bool Accept(int value) { return value > 0; }
                        private static void Cleanup() { }
                        public static int Filter(int value)
                        {
                            try { throw new ArgumentException(); }
                            catch (ArgumentException) when (/*@shape:exception.filter.pure_boolean*/Accept(value)) { return 1; }
                            catch (ArgumentException) { return 2; }
                        }
                        public static int FinallyNormal(int value)
                        {
                            int result = value;
                            try { result++; }
                            /*@shape:exception.finally.normal*/finally { result++; }
                            return result;
                        }
                        public static int ReturnThroughFinally(int value)
                        {
                            try { /*@shape:abrupt.return.through_finally*/return value; }
                            /*@shape:exception.finally.preserves_return*/finally { Cleanup(); }
                        }
                        public static int LoopControlThroughFinally()
                        {
                            int result = 0;
                            for (int index = 0; index < 2; index++)
                            {
                                try
                                {
                                    if (index == 0) { /*@shape:abrupt.continue.through_finally*/continue; }
                                    /*@shape:abrupt.break.through_finally*/break;
                                }
                                /*@shape:exception.finally.preserves_loop_control*/finally { Cleanup(); }
                            }
                            return result;
                        }
                        public static int Rethrow()
                        {
                            try { throw new InvalidOperationException(); }
                            catch (InvalidOperationException) { /*@shape:exception.throw.rethrow*/throw; }
                        }
                        public static int NestedSearch()
                        {
                            try
                            {
                                try { throw new ArgumentException(); }
                                /*@shape:exception.unwind.inner_finally*/finally { Cleanup(); }
                            }
                            catch (ArgumentException) when (/*@shape:exception.search.outer_filter_before_inner_finally*/Accept(1)) { return 1; }
                        }
                    }
                }
                """),
            new ProbeCase("admitted-filter-failure-and-abrupt-finally", "admitted_shape", """
                using System;
                namespace Probe
                {
                    public static class FilterFailure
                    {
                        private static bool ThrowsFilter() { throw new InvalidOperationException(); }
                        public static int Evaluate()
                        {
                            try { throw new ArgumentException(); }
                            catch (ArgumentException) when (/*@shape:exception.filter.failure*/ThrowsFilter()) { return 1; }
                            /*@shape:exception.filter.failure_continues_search*/catch (ArgumentException) { return 2; }
                        }
                        public static int FinallyReplacesReturn()
                        {
                            try { /*@shape:abrupt.return.before_finally_throw*/return 1; }
                            finally { /*@shape:exception.finally.throw_replaces_return*/throw new InvalidOperationException(); }
                        }
                    }
                }
                """),
            new ProbeCase("near-miss-loop-control", "rejected_near_miss", """
                using System;
                using System.Collections.Generic;
                namespace Probe
                {
                    public static class LoopRejects
                    {
                        public static int Goto(int value)
                        {
                            if (value > 0) { /*@shape:near_miss.loop.goto*/goto Done; }
                            value++;
                            /*@shape:near_miss.loop.label*/Done: return value;
                        }
                        public static int Framework(List<int> values)
                        {
                            int total = 0;
                            /*@shape:near_miss.loop.foreach_framework_collection*/foreach (int value in values) { total += value; }
                            return total;
                        }
                        public static int Ref(int[] values)
                        {
                            int total = 0;
                            /*@shape:near_miss.loop.foreach_ref*/foreach (ref int value in values.AsSpan()) { total += value; }
                            return total;
                        }
                    }
                }
                """),
            new ProbeCase("near-miss-custom-enumeration", "rejected_near_miss", """
                namespace Probe
                {
                    public sealed class Values
                    {
                        public Cursor GetEnumerator() { return new Cursor(); }
                    }
                    public struct Cursor
                    {
                        public int Current { get; private set; }
                        public bool MoveNext() { Current++; return Current < 2; }
                    }
                    public static class Rejects
                    {
                        public static int Custom(Values values)
                        {
                            int total = 0;
                            /*@shape:near_miss.loop.foreach_custom_protocol*/foreach (int value in values) { total += value; }
                            return total;
                        }
                    }
                }
                """),
            new ProbeCase("near-miss-switch-non-exhaustive", "rejected_near_miss", """
                namespace Probe
                {
                    public static class SwitchRejects
                    {
                        public static int NonExhaustive(int value)
                        {
                            return /*@shape:near_miss.switch.expression_non_exhaustive*/value switch { 0 => 0 };
                        }
                    }
                }
                """),
            new ProbeCase("near-miss-switch-branches", "rejected_near_miss", """
                namespace Probe
                {
                    public static class SwitchRejects
                    {
                        public static int GotoCase(int value)
                        {
                            switch (value)
                            {
                                case 0: /*@shape:near_miss.switch.goto_case*/goto case 1;
                                case 1: return 1;
                                default: /*@shape:near_miss.switch.goto_default*/goto default;
                            }
                        }
                        public static int Fallthrough(int value)
                        {
                            switch (value)
                            {
                                case 0: /*@shape:near_miss.switch.statement_fallthrough*/value++;
                                case 1: return value;
                                default: return -1;
                            }
                        }
                    }
                }
                """),
            new ProbeCase("near-miss-patterns", "rejected_near_miss", """
                namespace Probe
                {
                    public sealed class Positional
                    {
                        public int Value { get; }
                        public Positional(int value) { Value = value; }
                        public void Deconstruct(out int value) { value = Value; }
                    }
                    public class Open
                    {
                        public int Value { get; }
                        public Open(int value) { Value = value; }
                    }
                    public sealed class Extended
                    {
                        public int Value { get; }
                        public Extended(int value) { Value = value; }
                    }
                    public static class Extensions
                    {
                        public static void Deconstruct(this Extended value, out int item) { item = value.Value; }
                    }
                    public static class PatternRejects
                    {
                        public static int PositionalValue(Positional value)
                        {
                            return value switch { /*@shape:near_miss.pattern.positional_deconstruct*/Positional(var item) => item, _ => 0 };
                        }
                        public static int OpenType(Open? value)
                        {
                            return value switch { /*@shape:near_miss.pattern.open_hierarchy*/Open item => item.Value, null => 0 };
                        }
                        public static int Extension(Extended value)
                        {
                            return value switch { /*@shape:near_miss.pattern.extension_deconstruct*/Extended(var item) => item, _ => 0 };
                        }
                        public static bool Dynamic(dynamic value)
                        {
                            return /*@shape:near_miss.pattern.dynamic_input*/value is int;
                        }
                        public static bool Slice(int[] values)
                        {
                            return /*@shape:near_miss.pattern.list_slice*/values is [1, ..];
                        }
                        public static bool StringList(string value)
                        {
                            return /*@shape:near_miss.pattern.list_string*/value is ['a'];
                        }
                    }
                }
                """),
            new ProbeCase("near-miss-pattern-effects", "rejected_near_miss", """
                namespace Probe
                {
                    public sealed class Effectful
                    {
                        private int _value;
                        public int Value { get { _value++; return _value; } }
                    }
                    public readonly struct Comparable
                    {
                        private readonly int _value;
                        public Comparable(int value) { _value = value; }
                        public static bool operator ==(Comparable left, Comparable right) { return left._value == right._value; }
                        public static bool operator !=(Comparable left, Comparable right) { return left._value != right._value; }
                        public override bool Equals(object? value) { return false; }
                        public override int GetHashCode() { return _value; }
                    }
                    public static class Effects
                    {
                        public static bool Getter(Effectful value)
                        {
                            return /*@shape:near_miss.pattern.property_effectful_getter*/value is { Value: > 0 };
                        }
                        public static bool Equality(Comparable left, Comparable right)
                        {
                            return left is var item && /*@shape:near_miss.pattern.guard_user_equality*/item == right;
                        }
                    }
                }
                """),
            new ProbeCase("near-miss-throws", "rejected_near_miss", """
                using System;
                namespace Probe
                {
                    public class OpenException : Exception { }
                    public class BaseException : Exception { }
                    public sealed class IndirectException : BaseException { }
                    public sealed class MessageException : Exception
                    {
                        public MessageException(string message) : base(message) { }
                    }
                    public static class ThrowRejects
                    {
                        public static int Expression(string? value)
                        {
                            return (value ?? /*@shape:near_miss.exception.throw_expression*/throw new ArgumentNullException()).Length;
                        }
                        public static void Null()
                        {
                            /*@shape:near_miss.exception.throw_null*/throw null;
                        }
                        public static void Stored()
                        {
                            Exception error = new ArgumentException();
                            /*@shape:near_miss.exception.throw_stored*/throw error;
                        }
                        public static void Message()
                        {
                            /*@shape:near_miss.exception.builtin_message_constructor*/throw new ArgumentException("message");
                        }
                        public static void ParameterName()
                        {
                            /*@shape:near_miss.exception.builtin_parameter_name_constructor*/throw new ArgumentNullException("value");
                        }
                        public static void Inner()
                        {
                            /*@shape:near_miss.exception.builtin_inner_constructor*/throw new ArgumentException("message", new InvalidOperationException());
                        }
                        public static void Open()
                        {
                            /*@shape:near_miss.exception.source_unsealed*/throw new OpenException();
                        }
                        public static void Indirect()
                        {
                            /*@shape:near_miss.exception.source_indirect_base*/throw new IndirectException();
                        }
                        public static void SourceMessage()
                        {
                            /*@shape:near_miss.exception.source_message_base*/throw new MessageException("message");
                        }
                    }
                }
                """),
            new ProbeCase("near-miss-catches", "rejected_near_miss", """
                using System;
                namespace Probe
                {
                    public static class CatchRejects
                    {
                        public static int CatchAll()
                        {
                            try { throw new ArgumentException(); }
                            /*@shape:near_miss.exception.catch_untyped*/catch { return 0; }
                        }
                        public static int CatchGeneral()
                        {
                            try { throw new ArgumentException(); }
                            /*@shape:near_miss.exception.catch_system_exception*/catch (Exception) { return 0; }
                        }
                        public static int CatchResource()
                        {
                            try { throw new OutOfMemoryException(); }
                            /*@shape:near_miss.exception.catch_resource_exhaustion*/catch (OutOfMemoryException) { return 0; }
                        }
                        public static int CatchStackOverflow()
                        {
                            try { throw new StackOverflowException(); }
                            /*@shape:near_miss.exception.catch_stack_overflow*/catch (StackOverflowException) { return 0; }
                        }
                        public static string ReadRuntimeState()
                        {
                            try { throw new ArgumentException(); }
                            catch (ArgumentException error) { return /*@shape:near_miss.exception.catch_message*/error.Message; }
                        }
                        public static object? ReadInner()
                        {
                            try { throw new ArgumentException(); }
                            catch (ArgumentException error) { return /*@shape:near_miss.exception.catch_inner_exception*/error.InnerException; }
                        }
                        public static string? ReadStack()
                        {
                            try { throw new ArgumentException(); }
                            catch (ArgumentException error) { return /*@shape:near_miss.exception.catch_stack_trace*/error.StackTrace; }
                        }
                        public static object ReadData()
                        {
                            try { throw new ArgumentException(); }
                            catch (ArgumentException error) { return /*@shape:near_miss.exception.catch_data*/error.Data; }
                        }
                        public static int FilterMutation(int value)
                        {
                            try { throw new ArgumentException(); }
                            catch (ArgumentException) when (/*@shape:near_miss.exception.filter_side_effect*/++value > 0) { return value; }
                        }
                    }
                }
                """),
            new ProbeCase("near-miss-rethrow-and-handler-order", "rejected_near_miss", """
                using System;
                namespace Probe
                {
                    public static class InvalidHandlers
                    {
                        public static void Outside()
                        {
                            /*@shape:near_miss.exception.rethrow_outside_catch*/throw;
                        }
                        public static int WrongOrder()
                        {
                            try { throw new ArgumentOutOfRangeException(); }
                            catch (ArgumentException) { return 1; }
                            /*@shape:near_miss.exception.catch_unreachable_order*/catch (ArgumentOutOfRangeException) { return 2; }
                        }
                    }
                }
                """),
            new ProbeCase("near-miss-finally-abrupt", "rejected_near_miss", """
                namespace Probe
                {
                    public static class FinallyRejects
                    {
                        public static int Return()
                        {
                            try { return 1; }
                            finally { /*@shape:near_miss.exception.finally_return*/return 2; }
                        }
                        public static int Break()
                        {
                            while (true)
                            {
                                try { return 1; }
                                finally { /*@shape:near_miss.exception.finally_break*/break; }
                            }
                            return 0;
                        }
                        public static int Continue()
                        {
                            int value = 0;
                            while (value < 1)
                            {
                                try { return value; }
                                finally { value++; /*@shape:near_miss.exception.finally_continue*/continue; }
                            }
                            return value;
                        }
                        public static int Goto()
                        {
                            try { return 1; }
                            finally { /*@shape:near_miss.exception.finally_goto*/goto Done; }
                        Done:
                            return 0;
                        }
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
