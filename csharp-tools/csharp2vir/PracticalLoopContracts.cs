using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Globalization;
using System.Linq;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

// W01 source half of the private pre-lowering contract gate. This captures
// original Roslyn sites and lexical facts, never CFGs or frontend success.
// Native attachment uses the retained method sidecar and the shared 33-arm
// expression validator. W02 consumes the result and owns ownership/CFG proofs.
internal static class CSharpPracticalLoopContracts
{
    internal static byte[] Capture(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs, ImmutableArray<MetadataReference> references)
    {
        try { return CaptureCore(selection, inputs, references); }
        catch (PracticalCaptureFailure) { throw; }
        catch (Exception) { throw PracticalFailures.Protocol("loop_contract_capture"); }
    }
    private static byte[] CaptureCore(PracticalSourceSelection selection,
        IEnumerable<PracticalCapturedInput> inputs, ImmutableArray<MetadataReference> references)
    {
        CSharpCompilation? compilation = null;
        var closure = CSharpPracticalCapture.Validate(selection, inputs, references,
            validateDataDeclarations: c => { compilation = c; CSharpPracticalSyntaxNormalizer.ValidateLoopContractPrerequisites(c); },
            validateDataLimits: CheckLimits, allowLoopContractForeach: true);
        var methods = new List<object>();
        foreach (var declaration in closure.ReachableDeclarations.Where(d =>
            d.Id.StartsWith("mpk.csharp.source.", StringComparison.Ordinal)
            && d.Kind is PracticalDeclarationKind.Method or PracticalDeclarationKind.Constructor or PracticalDeclarationKind.Property))
        {
            var source = closure.Sources[declaration.SourceOrdinal];
            var tree = compilation!.SyntaxTrees.Single(t => t.FilePath == source.Path);
            var syntax = tree.GetRoot().DescendantNodes().Where(n => n is BaseMethodDeclarationSyntax or PropertyDeclarationSyntax).Single(n =>
                source.ByteOffset(n.SpanStart) == declaration.StartByte && source.ByteOffset(n.Span.End) == declaration.EndByte);
            var model = compilation.GetSemanticModel(tree);
            var symbol = model.GetDeclaredSymbol(syntax);
            var method = symbol is IPropertySymbol property ? property.GetMethod! : (IMethodSymbol)symbol!;
            var loops = syntax.DescendantNodes().Where(IsLoop).ToArray();
            var locals = new Dictionary<ISymbol, string>(SymbolEqualityComparer.Default);
            // Match the data normalizer's existing declarator ordinals; foreach
            // iteration bindings follow them in source order for the W02 handoff.
            foreach (var node in syntax.DescendantNodes().OfType<VariableDeclaratorSyntax>())
                if (model.GetDeclaredSymbol(node) is ILocalSymbol local)
                    locals.Add(local, "local:" + locals.Count.ToString(CultureInfo.InvariantCulture));
            foreach (var node in loops.OfType<ForEachStatementSyntax>())
                locals.Add(model.GetDeclaredSymbol(node)!, "local:" + locals.Count.ToString(CultureInfo.InvariantCulture));
            string Binding(ISymbol symbol) => symbol is IParameterSymbol parameter
                ? "parameter:" + parameter.Ordinal.ToString(CultureInfo.InvariantCulture)
                : locals.TryGetValue(symbol, out var id) ? id : throw Failure("binding");
            string Type(ITypeSymbol type) => PracticalExactTypeNormalizer.Normalize(type, compilation).Id;
            object Bind(ISymbol symbol) => new { id = Binding(symbol), type_id = Type(symbol is ILocalSymbol l ? l.Type : ((IParameterSymbol)symbol).Type) };
            var parameters = method.Parameters.Select(Bind).ToList();
            if (!method.IsStatic)
                parameters.Insert(0, new { id = "this", type_id = Type(method.ContainingType.WithNullableAnnotation(NullableAnnotation.NotAnnotated)) });
            string LoopId(SyntaxNode loop) => declaration.Id + "#loop#" + Array.IndexOf(loops, loop).ToString("D4", CultureInfo.InvariantCulture);
            var rows = new List<object>();
            foreach (var loop in loops)
            {
                if (loop is ForEachVariableStatementSyntax) throw Failure("foreach_shape");
                string[] borrows = Array.Empty<string>();
                if (loop is ForEachStatementSyntax each)
                {
                    var type = model.GetTypeInfo(each.Expression).Type;
                    if (type is not IArrayTypeSymbol { Rank: 1, IsSZArray: true }
                        && type?.SpecialType != SpecialType.System_String) throw Failure("foreach_shape");
                    if (!each.AwaitKeyword.IsKind(SyntaxKind.None) || each.Type is RefTypeSyntax) throw Failure("foreach_shape");
                    var bound = model.GetSymbolInfo(each.Expression).Symbol;
                    if (bound is ILocalSymbol or IParameterSymbol) borrows = new[] { Binding(bound) };
                }
                var assigned = model.AnalyzeDataFlow(loop)!.DefinitelyAssignedOnEntry.ToHashSet(SymbolEqualityComparer.Default);
                var visible = model.LookupSymbols(loop.SpanStart).ToHashSet(SymbolEqualityComparer.Default);
                var scoped = locals.Keys.Where(s => assigned.Contains(s) && visible.Contains(s)).ToHashSet(SymbolEqualityComparer.Default);
                if (loop is ForStatementSyntax { Declaration: not null } forLoop)
                    foreach (var variable in forLoop.Declaration.Variables.Where(v => v.Initializer is not null))
                        scoped.Add(model.GetDeclaredSymbol(variable)!);
                var variables = parameters.Concat(scoped.OrderBy(Binding, StringComparer.Ordinal).Select(Bind)).ToArray();
                var modifies = new SortedSet<string>(StringComparer.Ordinal);
                var arrayWrites = new SortedSet<string>(StringComparer.Ordinal);
                foreach (var node in loop.DescendantNodesAndSelf())
                {
                    var operation = model.GetOperation(node);
                    IOperation? target = operation switch {
                        IAssignmentOperation assignment => assignment.Target,
                        IIncrementOrDecrementOperation increment => increment.Target,
                        _ => null,
                    };
                    if (target is ILocalReferenceOperation local) modifies.Add(Binding(local.Local));
                    else if (target is IParameterReferenceOperation parameter) modifies.Add(Binding(parameter.Parameter));
                    else if (target is IArrayElementReferenceOperation element)
                    {
                        IOperation array = element.ArrayReference;
                        while (array is IConversionOperation conversion) array = conversion.Operand;
                        string id = array switch {
                            ILocalReferenceOperation l => Binding(l.Local),
                            IParameterReferenceOperation p => Binding(p.Parameter),
                            _ => throw Failure("array_ownership_target"),
                        };
                        modifies.Add("construction:" + id); arrayWrites.Add(id);
                    }
                    else if (target is not null) throw Failure("modifies_target");
                    if (node is VariableDeclaratorSyntax { Initializer: not null } v && model.GetDeclaredSymbol(v) is ILocalSymbol ls)
                        modifies.Add(Binding(ls));
                    if (node is ForEachStatementSyntax fe) modifies.Add(Binding(model.GetDeclaredSymbol(fe)!));
                }
                // Borrow conflicts are retained for W02's alias-aware ownership
                // validation; a modifies clause is never a write permission.
                var exits = new List<object>();
                foreach (var node in loop.DescendantNodes().Where(n => n is BreakStatementSyntax or ContinueStatementSyntax or ReturnStatementSyntax or ThrowStatementSyntax))
                {
                    var nearest = node.Ancestors().FirstOrDefault(IsLoop);
                    string kind, target;
                    if (node is ReturnStatementSyntax) { kind = "return"; target = declaration.Id; }
                    else if (node is ThrowStatementSyntax) { kind = "throw"; target = declaration.Id; }
                    else {
                        if (node is BreakStatementSyntax && node.Ancestors().First(n => IsLoop(n) || n is SwitchStatementSyntax) is SwitchStatementSyntax) continue;
                        if (nearest is null) throw Failure("abrupt_target");
                        kind = node is BreakStatementSyntax ? "break" : "continue"; target = LoopId(nearest);
                        // An inner break/continue does not exit its ancestors.
                        if (nearest != loop) continue;
                    }
                    exits.Add(new { kind, target, start_byte = source.ByteOffset(node.SpanStart), end_byte = source.ByteOffset(node.Span.End) });
                }
                var parent = loop.Ancestors().FirstOrDefault(IsLoop);
                rows.Add(new {
                    loop_id = LoopId(loop), parent = parent is null ? null : LoopId(parent),
                    kind = loop switch { ForStatementSyntax => "for", WhileStatementSyntax => "while", DoStatementSyntax => "do", _ => "foreach" },
                    start_byte = source.ByteOffset(loop.SpanStart), end_byte = source.ByteOffset(loop.Span.End),
                    variables, modifies = modifies.ToArray(), read_borrows = borrows, array_writes = arrayWrites.ToArray(), exits,
                });
            }
            methods.Add(new {
                callable_id = declaration.Id, source_path = source.Path, source_content_sha256 = source.RawSha256,
                start_byte = declaration.StartByte, end_byte = declaration.EndByte,
                parameters, locals = locals.Keys.OrderBy(Binding, StringComparer.Ordinal).Select(Bind).ToArray(),
                result_type_id = PracticalExactTypeNormalizer.Normalize(method.ReturnType, compilation, true).Id,
                allocations = syntax.DescendantNodes().OfType<VariableDeclaratorSyntax>()
                    .Where(v => v.Initializer?.Value is ArrayCreationExpressionSyntax)
                    .Select(v => {
                        var creation = (ArrayCreationExpressionSyntax)v.Initializer!.Value;
                        var local = (ILocalSymbol)model.GetDeclaredSymbol(v)!;
                        var length = creation.Type.RankSpecifiers.Single().Sizes.Single();
                        var lengthSymbol = model.GetSymbolInfo(length).Symbol;
                        return new { binding_id = Binding(local), type_id = Type(local.Type),
                            start_byte = source.ByteOffset(creation.SpanStart), end_byte = source.ByteOffset(creation.Span.End),
                            length_start_byte = source.ByteOffset(length.SpanStart), length_end_byte = source.ByteOffset(length.Span.End),
                            length_binding = lengthSymbol is ILocalSymbol or IParameterSymbol ? Binding(lengthSymbol) : null };
                    }).ToArray(),
                loops = rows,
            });
        }
        return JsonSerializer.SerializeToUtf8Bytes(new {
            compilation_id = selection.CompilationId,
            selected_root_ids = selection.SelectedRootIds,
            sources = closure.Sources.Select(s => new { path = s.Path, raw_sha256 = s.RawSha256 }),
            methods,
        });
    }
    private static bool IsLoop(SyntaxNode node) => node is ForStatementSyntax or WhileStatementSyntax
        or DoStatementSyntax or ForEachStatementSyntax or ForEachVariableStatementSyntax;
    private static PracticalCaptureFailure Failure(string code) =>
        new(7, PracticalDiagnosticFamily.CSHARP_PRACTICAL_LOOP_CONTRACT, "loop_contract_" + code);
    private static void CheckLimits(CSharpCompilation compilation)
    {
        foreach (var tree in compilation.SyntaxTrees)
            foreach (var method in tree.GetRoot().DescendantNodes().Where(n => n is BaseMethodDeclarationSyntax or PropertyDeclarationSyntax))
            {
                var loops = method.DescendantNodes().Where(IsLoop).ToArray();
                if (loops.Length > 32) throw PracticalFailures.Limit("loops_per_method");
                if (loops.Any(l => l.Ancestors().Count(IsLoop) >= 8)) throw PracticalFailures.Limit("loop_nesting");
            }
    }
}
