using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Runtime.CompilerServices;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.FlowAnalysis;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

internal enum SubsetValueType
{
    Bool,
    I32,
    U32,
    I64,
    U64,
}

internal static class SubsetLimits
{
    internal const uint MethodClosureMaximum = FrontendLimits.MethodClosureMaximum;
    internal const uint SyntaxNodesMaximum = FrontendLimits.SyntaxNodesMaximum;
    internal const uint OperationsPerMethodMaximum = FrontendLimits.OperationsPerMethodMaximum;
    internal const uint OperationsPerClosureMaximum = FrontendLimits.OperationsPerClosureMaximum;
    internal const uint CfgBlocksPerMethodMaximum = FrontendLimits.CfgBlocksPerMethodMaximum;
    internal const uint CfgBlocksPerClosureMaximum = FrontendLimits.CfgBlocksPerClosureMaximum;

    internal static uint Add(uint current, uint increment, uint maximum, string code)
    {
        uint result;
        try
        {
            result = checked(current + increment);
        }
        catch (OverflowException)
        {
            throw FrontendFailure.Rejected("subset", code);
        }

        if (result > maximum)
        {
            throw FrontendFailure.Rejected("subset", code);
        }

        return result;
    }
}

internal sealed class SubsetMethod
{
    internal SubsetMethod(
        string canonicalId,
        MethodDeclarationSyntax declaration,
        IMethodSymbol symbol,
        SemanticModel semanticModel,
        IMethodBodyOperation body,
        ControlFlowGraph controlFlowGraph,
        ImmutableArray<string> callees,
        uint operationCount,
        uint cfgBlockCount)
    {
        CanonicalId = canonicalId;
        Declaration = declaration;
        Symbol = symbol;
        SemanticModel = semanticModel;
        Body = body;
        ControlFlowGraph = controlFlowGraph;
        Callees = callees;
        OperationCount = operationCount;
        CfgBlockCount = cfgBlockCount;
    }

    internal string CanonicalId { get; }

    internal MethodDeclarationSyntax Declaration { get; }

    internal IMethodSymbol Symbol { get; }

    internal SemanticModel SemanticModel { get; }

    internal IMethodBodyOperation Body { get; }

    internal ControlFlowGraph ControlFlowGraph { get; }

    internal ImmutableArray<string> Callees { get; }

    internal uint OperationCount { get; }

    internal uint CfgBlockCount { get; }
}

internal sealed class DeclaredSubsetMethod
{
    internal DeclaredSubsetMethod(
        string canonicalId,
        MethodDeclarationSyntax declaration,
        IMethodSymbol symbol,
        SemanticModel semanticModel)
    {
        CanonicalId = canonicalId;
        Declaration = declaration;
        Symbol = symbol;
        SemanticModel = semanticModel;
    }

    internal string CanonicalId { get; }

    internal MethodDeclarationSyntax Declaration { get; }

    internal IMethodSymbol Symbol { get; }

    internal SemanticModel SemanticModel { get; }
}

internal sealed class SubsetClosure
{
    internal SubsetClosure(
        ImmutableArray<string> selectedRoots,
        ImmutableArray<SubsetMethod> methods,
        uint syntaxNodeCount,
        uint operationCount,
        uint cfgBlockCount)
    {
        SelectedRoots = selectedRoots;
        Methods = methods;
        SyntaxNodeCount = syntaxNodeCount;
        OperationCount = operationCount;
        CfgBlockCount = cfgBlockCount;
    }

    internal ImmutableArray<string> SelectedRoots { get; }

    // Methods are stored in deterministic callee-first order. Canonical method
    // ID breaks ties between otherwise independent closure members.
    internal ImmutableArray<SubsetMethod> Methods { get; }

    internal uint SyntaxNodeCount { get; }

    internal uint OperationCount { get; }

    internal uint CfgBlockCount { get; }
}

internal sealed class ReferenceOperationComparer : IEqualityComparer<IOperation>
{
    internal static readonly ReferenceOperationComparer Instance = new ReferenceOperationComparer();

    private ReferenceOperationComparer()
    {
    }

    public bool Equals(IOperation? left, IOperation? right)
    {
        return ReferenceEquals(left, right);
    }

    public int GetHashCode(IOperation value)
    {
        return RuntimeHelpers.GetHashCode(value);
    }
}
