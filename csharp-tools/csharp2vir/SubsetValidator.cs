using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using Microsoft.CodeAnalysis;

namespace Mpk.CSharp2Vir;

internal static class CSharpSubset
{
    internal static SubsetClosure Validate(
        Selection selection,
        RoslynCompilationSession session)
    {
        SubsetDeclarationSet declarations = SubsetDeclarations.Validate(selection, session);
        var declaredById = new Dictionary<string, DeclaredSubsetMethod>(StringComparer.Ordinal);
        var analyses = new Dictionary<string, SubsetBodyAnalysis>(StringComparer.Ordinal);
        foreach (DeclaredSubsetMethod method in declarations.Methods)
        {
            declaredById.Add(method.CanonicalId, method);
        }

        var selected = new HashSet<string>(declarations.SelectedRoots, StringComparer.Ordinal);
        var states = new Dictionary<string, byte>(StringComparer.Ordinal);
        uint closureCount = 0;
        uint operationCount = 0;
        uint cfgBlockCount = 0;
        foreach (string root in declarations.SelectedRoots)
        {
            Visit(
                root,
                declaredById,
                declarations.Methods,
                analyses,
                states,
                ref closureCount,
                ref operationCount,
                ref cfgBlockCount);
        }

        if (analyses.Count != declarations.Methods.Length)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
        }

        List<string> ordered = CanonicalOrder(analyses);
        var methods = ImmutableArray.CreateBuilder<SubsetMethod>(ordered.Count);
        foreach (string id in ordered)
        {
            DeclaredSubsetMethod declared = declaredById[id];
            SubsetBodyAnalysis analysis = analyses[id];
            if (!selected.Contains(id)
                && declared.Symbol.DeclaredAccessibility != Accessibility.Private
                && declared.Symbol.DeclaredAccessibility != Accessibility.Internal)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
            }

            methods.Add(new SubsetMethod(
                id,
                declared.Declaration,
                declared.Symbol,
                declared.SemanticModel,
                analysis.Body,
                analysis.Graph,
                analysis.Callees,
                analysis.OperationCount,
                analysis.CfgBlockCount));
        }

        return new SubsetClosure(
            declarations.SelectedRoots,
            methods.MoveToImmutable(),
            declarations.SyntaxNodeCount,
            operationCount,
            cfgBlockCount);
    }

    private static void Visit(
        string id,
        IReadOnlyDictionary<string, DeclaredSubsetMethod> declared,
        ImmutableArray<DeclaredSubsetMethod> allMethods,
        Dictionary<string, SubsetBodyAnalysis> analyses,
        Dictionary<string, byte> states,
        ref uint closureCount,
        ref uint operationCount,
        ref uint cfgBlockCount)
    {
        if (!declared.TryGetValue(id, out DeclaredSubsetMethod? method))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
        }

        if (states.TryGetValue(id, out byte state))
        {
            if (state == 1)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
            }

            return;
        }

        closureCount = SubsetLimits.Add(
            closureCount,
            1,
            SubsetLimits.MethodClosureMaximum,
            "CSHARP_LIMIT_METHOD_CLOSURE");
        states.Add(id, 1);
        SubsetBodyAnalysis analysis = SubsetBodies.Validate(
            method,
            allMethods,
            ref operationCount,
            ref cfgBlockCount);
        analyses.Add(id, analysis);
        foreach (string callee in analysis.Callees)
        {
            Visit(
                callee,
                declared,
                allMethods,
                analyses,
                states,
                ref closureCount,
                ref operationCount,
                ref cfgBlockCount);
        }

        states[id] = 2;
    }

    private static List<string> CanonicalOrder(
        IReadOnlyDictionary<string, SubsetBodyAnalysis> analyses)
    {
        var unresolved = new Dictionary<string, int>(StringComparer.Ordinal);
        var callers = new Dictionary<string, SortedSet<string>>(StringComparer.Ordinal);
        var ids = new SortedSet<string>(analyses.Keys, StringComparer.Ordinal);
        foreach (string id in ids)
        {
            callers.Add(id, new SortedSet<string>(StringComparer.Ordinal));
        }

        foreach (string id in ids)
        {
            ImmutableArray<string> callees = analyses[id].Callees;
            unresolved.Add(id, callees.Length);
            foreach (string callee in callees)
            {
                if (!callers.TryGetValue(callee, out SortedSet<string>? calleeCallers))
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
                }

                calleeCallers.Add(id);
            }
        }

        var ready = new SortedSet<string>(StringComparer.Ordinal);
        foreach (string id in ids)
        {
            if (unresolved[id] == 0)
            {
                ready.Add(id);
            }
        }

        var ordered = new List<string>(analyses.Count);
        while (ready.Count != 0)
        {
            string id = ready.Min
                ?? throw FrontendFailure.Internal("subset");
            ready.Remove(id);
            ordered.Add(id);
            foreach (string caller in callers[id])
            {
                int remaining = unresolved[caller] - 1;
                unresolved[caller] = remaining;
                if (remaining == 0)
                {
                    ready.Add(caller);
                }
            }
        }

        if (ordered.Count != analyses.Count)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
        }

        return ordered;
    }
}
