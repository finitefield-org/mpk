using System;
using System.Threading;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.FlowAnalysis;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

internal static class RoslynPublicApi
{
    internal static SemanticModel GetSemanticModel(
        RoslynCompilationSession session,
        SyntaxTree syntaxTree)
    {
        return GetSemanticModel(session, syntaxTree, "lowering");
    }

    internal static SemanticModel GetSemanticModel(
        RoslynCompilationSession session,
        SyntaxTree syntaxTree,
        string phase)
    {
        try
        {
            SemanticModel model = session.Compilation.GetSemanticModel(
                syntaxTree,
                ignoreAccessibility: false);
            if (!ReferenceEquals(model.SyntaxTree, syntaxTree)
                || !ReferenceEquals(model.Compilation, session.Compilation)
                || model.IgnoresAccessibility
                || model.IsSpeculativeSemanticModel
                || model.ParentModel is not null
                || !string.Equals(model.Language, LanguageNames.CSharp, StringComparison.Ordinal))
            {
                throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
            }

            return model;
        }
        catch (FrontendFailure)
        {
            throw;
        }
        catch (Exception error) when (IsAdapterException(error))
        {
            throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    internal static IMethodSymbol GetDeclaredSymbol(
        SemanticModel semanticModel,
        MethodDeclarationSyntax declaration)
    {
        return GetDeclaredSymbol(semanticModel, declaration, "lowering");
    }

    internal static IMethodSymbol GetDeclaredSymbol(
        SemanticModel semanticModel,
        MethodDeclarationSyntax declaration,
        string phase)
    {
        try
        {
            return semanticModel.GetDeclaredSymbol(declaration, CancellationToken.None)
                ?? throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
        }
        catch (FrontendFailure)
        {
            throw;
        }
        catch (Exception error) when (IsAdapterException(error))
        {
            throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    internal static SymbolInfo GetSymbolInfo(
        SemanticModel semanticModel,
        ExpressionSyntax expression)
    {
        return GetSymbolInfo(semanticModel, expression, "lowering");
    }

    internal static SymbolInfo GetSymbolInfo(
        SemanticModel semanticModel,
        ExpressionSyntax expression,
        string phase)
    {
        try
        {
            return semanticModel.GetSymbolInfo(expression, CancellationToken.None);
        }
        catch (Exception error) when (IsAdapterException(error))
        {
            throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    internal static TypeInfo GetTypeInfo(
        SemanticModel semanticModel,
        ExpressionSyntax expression)
    {
        return GetTypeInfo(semanticModel, expression, "lowering");
    }

    internal static TypeInfo GetTypeInfo(
        SemanticModel semanticModel,
        ExpressionSyntax expression,
        string phase)
    {
        try
        {
            return semanticModel.GetTypeInfo(expression, CancellationToken.None);
        }
        catch (Exception error) when (IsAdapterException(error))
        {
            throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    internal static Conversion ClassifyConversion(
        SemanticModel semanticModel,
        ExpressionSyntax expression,
        ITypeSymbol destination,
        bool isExplicitInSource)
    {
        return ClassifyConversion(
            semanticModel,
            expression,
            destination,
            isExplicitInSource,
            "lowering");
    }

    internal static Conversion ClassifyConversion(
        SemanticModel semanticModel,
        ExpressionSyntax expression,
        ITypeSymbol destination,
        bool isExplicitInSource,
        string phase)
    {
        try
        {
            return semanticModel.ClassifyConversion(
                expression,
                destination,
                isExplicitInSource);
        }
        catch (Exception error) when (IsAdapterException(error))
        {
            throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    internal static IOperation? GetOperation(
        SemanticModel semanticModel,
        SyntaxNode syntax)
    {
        return GetOperation(semanticModel, syntax, "lowering");
    }

    internal static IOperation? GetOperation(
        SemanticModel semanticModel,
        SyntaxNode syntax,
        string phase)
    {
        try
        {
            return semanticModel.GetOperation(syntax, CancellationToken.None);
        }
        catch (Exception error) when (IsAdapterException(error))
        {
            throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    internal static IMethodBodyOperation GetMethodBodyOperation(
        SemanticModel semanticModel,
        MethodDeclarationSyntax declaration)
    {
        return GetMethodBodyOperation(semanticModel, declaration, "lowering");
    }

    internal static IMethodBodyOperation GetMethodBodyOperation(
        SemanticModel semanticModel,
        MethodDeclarationSyntax declaration,
        string phase)
    {
        IOperation? operation = GetOperation(semanticModel, declaration, phase);
        if (operation is not IMethodBodyOperation methodBody
            || methodBody.Parent is not null
            || !ReferenceEquals(methodBody.Syntax, declaration))
        {
            throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
        }

        return methodBody;
    }

    internal static ControlFlowGraph CreateControlFlowGraph(IMethodBodyOperation methodBody)
    {
        return CreateControlFlowGraph(methodBody, "lowering");
    }

    internal static ControlFlowGraph CreateControlFlowGraph(
        IMethodBodyOperation methodBody,
        string phase)
    {
        try
        {
            ControlFlowGraph graph = ControlFlowGraph.Create(methodBody, CancellationToken.None);
            if (!ReferenceEquals(graph.OriginalOperation, methodBody) || graph.Parent is not null)
            {
                throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
            }

            return graph;
        }
        catch (FrontendFailure)
        {
            throw;
        }
        catch (Exception error) when (IsAdapterException(error))
        {
            throw FrontendFailure.Toolchain(phase, "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    private static bool IsAdapterException(Exception error)
    {
        return error is ArgumentException
            || error is InvalidOperationException
            || error is NotSupportedException;
    }
}
