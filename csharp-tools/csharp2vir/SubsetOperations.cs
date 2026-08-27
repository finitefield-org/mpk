using System;
using System.Collections.Generic;
using System.Collections.Immutable;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.FlowAnalysis;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

internal sealed class SubsetBodyAnalysis
{
    internal SubsetBodyAnalysis(
        IMethodBodyOperation body,
        ControlFlowGraph graph,
        ImmutableArray<string> callees,
        uint operationCount,
        uint cfgBlockCount)
    {
        Body = body;
        Graph = graph;
        Callees = callees;
        OperationCount = operationCount;
        CfgBlockCount = cfgBlockCount;
    }

    internal IMethodBodyOperation Body { get; }

    internal ControlFlowGraph Graph { get; }

    internal ImmutableArray<string> Callees { get; }

    internal uint OperationCount { get; }

    internal uint CfgBlockCount { get; }
}

internal enum ExplicitOverflowContext
{
    None,
    Checked,
    Unchecked,
}

internal static class SubsetBodies
{
    internal static SubsetBodyAnalysis Validate(
        DeclaredSubsetMethod method,
        ImmutableArray<DeclaredSubsetMethod> declaredMethods,
        ref uint closureOperationCount,
        ref uint closureCfgBlockCount)
    {
        IMethodBodyOperation body = RoslynPublicApi.GetMethodBodyOperation(
            method.SemanticModel,
            method.Declaration,
            "typecheck");
        ValidateTypecheckSyntax(method, body);
        ValidateSyntax(method);
        ControlFlowGraph graph = RoslynPublicApi.CreateControlFlowGraph(body, "subset");
        uint cfgBlocks = SubsetLimits.Add(
            0,
            checked((uint)graph.Blocks.Length),
            SubsetLimits.CfgBlocksPerMethodMaximum,
            "CSHARP_LIMIT_CFG_BLOCKS_PER_METHOD");
        closureCfgBlockCount = SubsetLimits.Add(
            closureCfgBlockCount,
            cfgBlocks,
            SubsetLimits.CfgBlocksPerClosureMaximum,
            "CSHARP_LIMIT_CFG_BLOCKS_PER_CLOSURE");

        ValidateGraph(graph);
        var operations = new HashSet<IOperation>(ReferenceOperationComparer.Instance);
        var operationOrder = new List<IOperation>();
        uint operationCount = 0;
        AddOperationTree(
            body,
            operations,
            operationOrder,
            ref operationCount,
            ref closureOperationCount);
        foreach (BasicBlock block in graph.Blocks)
        {
            foreach (IOperation operation in block.Operations)
            {
                AddOperationTree(
                    operation,
                    operations,
                    operationOrder,
                    ref operationCount,
                    ref closureOperationCount);
            }

            if (block.BranchValue is not null)
            {
                AddOperationTree(
                    block.BranchValue,
                    operations,
                    operationOrder,
                    ref operationCount,
                    ref closureOperationCount);
            }
        }

        var callees = new SortedSet<string>(StringComparer.Ordinal);
        foreach (IOperation operation in operationOrder)
        {
            ValidateOperation(operation, method, declaredMethods, callees);
        }

        ValidateDefiniteAssignment(graph, method);
        return new SubsetBodyAnalysis(
            body,
            graph,
            ImmutableArray.CreateRange(callees),
            operationCount,
            cfgBlocks);
    }

    private static void ValidateTypecheckSyntax(
        DeclaredSubsetMethod method,
        IMethodBodyOperation body)
    {
        BlockSyntax syntax = method.Declaration.Body
            ?? throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
        foreach (LocalDeclarationStatementSyntax local in syntax
            .DescendantNodes(descendIntoTrivia: false)
            .OfType<LocalDeclarationStatementSyntax>())
        {
            if (local.Declaration.Type is IdentifierNameSyntax inferred
                && string.Equals(inferred.Identifier.ValueText, "var", StringComparison.Ordinal))
            {
                continue;
            }

            _ = SubsetTypeRules.Validate(local.Declaration.Type, method.SemanticModel);
        }

        var negativeOperands = new HashSet<LiteralExpressionSyntax>();
        foreach (PrefixUnaryExpressionSyntax unary in syntax
            .DescendantNodes(descendIntoTrivia: false)
            .OfType<PrefixUnaryExpressionSyntax>())
        {
            if (unary.IsKind(SyntaxKind.UnaryMinusExpression)
                && unary.Operand is LiteralExpressionSyntax literal)
            {
                negativeOperands.Add(literal);
                ValidateLiteral(literal, method, negative: true, signedExpression: unary);
                continue;
            }

            IOperation operation = RoslynPublicApi.GetOperation(method.SemanticModel, unary, "typecheck")
                ?? throw FrontendFailure.Toolchain("typecheck", "CSHARP_TOOLCHAIN_ADAPTER");
            if (operation.ConstantValue.HasValue)
            {
                throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
            }
        }

        foreach (LiteralExpressionSyntax literal in syntax
            .DescendantNodes(descendIntoTrivia: false)
            .OfType<LiteralExpressionSyntax>())
        {
            if (!negativeOperands.Contains(literal))
            {
                ValidateLiteral(literal, method, negative: false, signedExpression: null);
            }
        }

        foreach (BinaryExpressionSyntax binary in syntax
            .DescendantNodes(descendIntoTrivia: false)
            .OfType<BinaryExpressionSyntax>())
        {
            IOperation operation = RoslynPublicApi.GetOperation(method.SemanticModel, binary, "typecheck")
                ?? throw FrontendFailure.Toolchain("typecheck", "CSHARP_TOOLCHAIN_ADAPTER");
            if (operation.ConstantValue.HasValue)
            {
                throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
            }
        }

        ValidateConvertedConstants(body);
    }

    private static void ValidateConvertedConstants(IOperation operation)
    {
        if (operation is IConversionOperation conversion
            && !conversion.Conversion.IsIdentity
            && conversion.Operand.ConstantValue.HasValue)
        {
            throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
        }

        foreach (IOperation child in operation.ChildOperations)
        {
            ValidateConvertedConstants(child);
        }
    }

    private static void ValidateSyntax(DeclaredSubsetMethod method)
    {
        BlockSyntax body = method.Declaration.Body
            ?? throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_DECLARATION");
        var names = new HashSet<string>(StringComparer.Ordinal);
        foreach (ParameterSyntax parameter in method.Declaration.ParameterList.Parameters)
        {
            names.Add(parameter.Identifier.ValueText);
        }

        ValidateBlock(body, method, names);
        if (!HasFinalReturn(body))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
        }
    }

    private static bool HasFinalReturn(BlockSyntax block)
    {
        if (block.Statements.Count == 0)
        {
            return false;
        }

        return block.Statements[^1] switch
        {
            ReturnStatementSyntax => true,
            BlockSyntax nested => HasFinalReturn(nested),
            CheckedStatementSyntax checkedStatement => HasFinalReturn(checkedStatement.Block),
            _ => false,
        };
    }

    private static void ValidateBlock(
        BlockSyntax block,
        DeclaredSubsetMethod method,
        HashSet<string> names)
    {
        foreach (StatementSyntax statement in block.Statements)
        {
            ValidateStatement(statement, method, names);
        }
    }

    private static void ValidateStatement(
        StatementSyntax statement,
        DeclaredSubsetMethod method,
        HashSet<string> names)
    {
        switch (statement)
        {
            case BlockSyntax block:
                ValidateBlock(block, method, names);
                return;
            case LocalDeclarationStatementSyntax local:
                ValidateLocalDeclaration(local, method, names);
                return;
            case ExpressionStatementSyntax expressionStatement:
                if (expressionStatement.Expression is AssignmentExpressionSyntax assignment
                    && assignment.IsKind(SyntaxKind.SimpleAssignmentExpression))
                {
                    if (assignment.Left is not IdentifierNameSyntax target)
                    {
                        throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_PURITY");
                    }

                    SubsetDeclarations.ValidateIdentifier(target.Identifier, "CSHARP_SUBSET_CONTROL_FLOW");
                    ValidateExpression(assignment.Right, method);
                    return;
                }

                if (expressionStatement.Expression is InvocationExpressionSyntax invocation)
                {
                    ValidateExpression(invocation, method);
                    return;
                }

                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
            case CheckedStatementSyntax checkedStatement:
                ValidateBlock(checkedStatement.Block, method, names);
                return;
            case IfStatementSyntax conditional:
                ValidateExpression(conditional.Condition, method);
                ValidateStatement(conditional.Statement, method, names);
                if (conditional.Else is not null)
                {
                    ValidateStatement(conditional.Else.Statement, method, names);
                }

                return;
            case ReturnStatementSyntax returned:
                if (returned.Expression is null)
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
                }

                ValidateExpression(returned.Expression, method);
                return;
            case ThrowStatementSyntax:
            case TryStatementSyntax:
            case LockStatementSyntax:
            case UsingStatementSyntax:
            case FixedStatementSyntax:
            case UnsafeStatementSyntax:
            case YieldStatementSyntax:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_ABRUPT");
            case ForStatementSyntax:
            case ForEachStatementSyntax:
            case WhileStatementSyntax:
            case DoStatementSyntax:
            case SwitchStatementSyntax:
            case GotoStatementSyntax:
            case LabeledStatementSyntax:
            case BreakStatementSyntax:
            case ContinueStatementSyntax:
            case LocalFunctionStatementSyntax:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
            default:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
        }
    }

    private static void ValidateLocalDeclaration(
        LocalDeclarationStatementSyntax local,
        DeclaredSubsetMethod method,
        HashSet<string> names)
    {
        if (!local.AwaitKeyword.IsKind(SyntaxKind.None)
            || !local.UsingKeyword.IsKind(SyntaxKind.None))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_ABRUPT");
        }

        if (local.Modifiers.Count != 0
            || local.Declaration.Variables.Count != 1)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
        }

        if (local.Declaration.Type is IdentifierNameSyntax inferred
            && string.Equals(inferred.Identifier.ValueText, "var", StringComparison.Ordinal))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
        }

        _ = SubsetTypeRules.Validate(local.Declaration.Type, method.SemanticModel);
        VariableDeclaratorSyntax variable = local.Declaration.Variables[0];
        SubsetDeclarations.ValidateIdentifier(variable.Identifier, "CSHARP_SUBSET_CONTROL_FLOW");
        if (variable.ArgumentList is not null
            || variable.Initializer is null
            || !names.Add(variable.Identifier.ValueText))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
        }

        ValidateExpression(variable.Initializer.Value, method);
    }

    private static void ValidateExpression(ExpressionSyntax expression, DeclaredSubsetMethod method)
    {
        switch (expression)
        {
            case ParenthesizedExpressionSyntax parenthesized:
                ValidateExpression(parenthesized.Expression, method);
                return;
            case LiteralExpressionSyntax literal:
                ValidateLiteral(literal, method, negative: false, signedExpression: null);
                return;
            case PrefixUnaryExpressionSyntax unary:
                if (unary.IsKind(SyntaxKind.UnaryMinusExpression)
                    && unary.Operand is LiteralExpressionSyntax negativeLiteral)
                {
                    ValidateLiteral(negativeLiteral, method, negative: true, signedExpression: unary);
                    return;
                }

                if (!unary.IsKind(SyntaxKind.LogicalNotExpression)
                    && !unary.IsKind(SyntaxKind.BitwiseNotExpression)
                    && !unary.IsKind(SyntaxKind.UnaryMinusExpression))
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
                }

                if (unary.IsKind(SyntaxKind.UnaryMinusExpression)
                    && ContextFor(unary) == ExplicitOverflowContext.None)
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OVERFLOW_CONTEXT");
                }

                ValidateExpression(unary.Operand, method);
                return;
            case BinaryExpressionSyntax binary:
                ValidateBinarySyntax(binary, method);
                ValidateExpression(binary.Left, method);
                ValidateExpression(binary.Right, method);
                return;
            case CheckedExpressionSyntax checkedExpression:
                ValidateExpression(checkedExpression.Expression, method);
                return;
            case CastExpressionSyntax cast:
                try
                {
                    _ = SubsetTypeRules.Validate(cast.Type, method.SemanticModel);
                }
                catch (FrontendFailure failure) when (
                    failure.Status == FrontendStatus.Rejected
                    && string.Equals(failure.Code, "CSHARP_SUBSET_TYPE", StringComparison.Ordinal))
                {
                    TypeInfo target = RoslynPublicApi.GetTypeInfo(
                        method.SemanticModel,
                        cast.Type,
                        "typecheck");
                    try
                    {
                        _ = SubsetTypeRules.ValidateSymbol(
                            target.Type,
                            method.SemanticModel.Compilation);
                    }
                    catch (FrontendFailure targetFailure) when (
                        targetFailure.Status == FrontendStatus.Rejected
                        && string.Equals(
                            targetFailure.Code,
                            "CSHARP_SUBSET_TYPE",
                            StringComparison.Ordinal))
                    {
                        throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONVERSION");
                    }

                    throw;
                }

                ValidateExpression(cast.Expression, method);
                return;
            case ConditionalExpressionSyntax conditional:
                ValidateExpression(conditional.Condition, method);
                ValidateExpression(conditional.WhenTrue, method);
                ValidateExpression(conditional.WhenFalse, method);
                return;
            case InvocationExpressionSyntax invocation:
                ValidateInvocationSyntax(invocation, method);
                return;
            case IdentifierNameSyntax identifier:
                SubsetDeclarations.ValidateIdentifier(identifier.Identifier, "CSHARP_SUBSET_OPERATION");
                ValidateValueReference(identifier, method);
                return;
            case AssignmentExpressionSyntax:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
            case ThrowExpressionSyntax:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_ABRUPT");
            case ObjectCreationExpressionSyntax:
            case ImplicitObjectCreationExpressionSyntax:
            case ArrayCreationExpressionSyntax:
            case ImplicitArrayCreationExpressionSyntax:
            case StackAllocArrayCreationExpressionSyntax:
            case CollectionExpressionSyntax:
            case AnonymousObjectCreationExpressionSyntax:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_PURITY");
            case MemberAccessExpressionSyntax:
            case ElementAccessExpressionSyntax:
            case ConditionalAccessExpressionSyntax:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_PURITY");
            case DefaultExpressionSyntax:
            case ImplicitStackAllocArrayCreationExpressionSyntax:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_INITIALIZATION");
            case LambdaExpressionSyntax:
            case QueryExpressionSyntax:
            case SwitchExpressionSyntax:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
            default:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
        }
    }

    private static void ValidateValueReference(
        IdentifierNameSyntax identifier,
        DeclaredSubsetMethod method)
    {
        SymbolInfo information = RoslynPublicApi.GetSymbolInfo(
            method.SemanticModel,
            identifier,
            "typecheck");
        if (information.CandidateReason != CandidateReason.None
            || information.CandidateSymbols.Length != 0
            || (information.Symbol is not IParameterSymbol
                && information.Symbol is not ILocalSymbol))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_PURITY");
        }

        _ = SubsetTypeRules.ValidateSymbol(
            (information.Symbol as IParameterSymbol)?.Type
                ?? ((ILocalSymbol)information.Symbol).Type,
            method.SemanticModel.Compilation);
    }

    private static void ValidateInvocationSyntax(
        InvocationExpressionSyntax invocation,
        DeclaredSubsetMethod method)
    {
        ValidateCallTargetSyntax(invocation.Expression);

        foreach (ArgumentSyntax argument in invocation.ArgumentList.Arguments)
        {
            if (argument.NameColon is not null
                || !argument.RefKindKeyword.IsKind(SyntaxKind.None))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
            }

            ValidateExpression(argument.Expression, method);
        }
    }

    private static void ValidateCallTargetSyntax(ExpressionSyntax expression)
    {
        switch (expression)
        {
            case IdentifierNameSyntax identifier:
                SubsetDeclarations.ValidateIdentifier(identifier.Identifier, "CSHARP_SUBSET_CALL");
                return;
            case MemberAccessExpressionSyntax member
                when member.IsKind(SyntaxKind.SimpleMemberAccessExpression)
                    && member.Name is IdentifierNameSyntax:
                ValidateCallTargetSyntax(member.Expression);
                SubsetDeclarations.ValidateIdentifier(member.Name.Identifier, "CSHARP_SUBSET_CALL");
                return;
            default:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
        }
    }

    private static void ValidateBinarySyntax(
        BinaryExpressionSyntax binary,
        DeclaredSubsetMethod method)
    {
        IOperation operation = RoslynPublicApi.GetOperation(method.SemanticModel, binary, "typecheck")
            ?? throw FrontendFailure.Toolchain("typecheck", "CSHARP_TOOLCHAIN_ADAPTER");
        if (operation.ConstantValue.HasValue)
        {
            throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
        }

        if (binary.IsKind(SyntaxKind.UnsignedRightShiftExpression))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
        }

        bool contextRequired = binary.IsKind(SyntaxKind.AddExpression)
            || binary.IsKind(SyntaxKind.SubtractExpression)
            || binary.IsKind(SyntaxKind.MultiplyExpression)
            || binary.IsKind(SyntaxKind.DivideExpression)
            || binary.IsKind(SyntaxKind.ModuloExpression);
        if (contextRequired && ContextFor(binary) == ExplicitOverflowContext.None)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OVERFLOW_CONTEXT");
        }
    }

    private static void ValidateLiteral(
        LiteralExpressionSyntax literal,
        DeclaredSubsetMethod method,
        bool negative,
        PrefixUnaryExpressionSyntax? signedExpression)
    {
        if (literal.IsKind(SyntaxKind.TrueLiteralExpression)
            || literal.IsKind(SyntaxKind.FalseLiteralExpression))
        {
            if (negative || !string.Equals(literal.Token.Text, literal.Token.ValueText, StringComparison.Ordinal))
            {
                throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
            }

            return;
        }

        if (!literal.IsKind(SyntaxKind.NumericLiteralExpression))
        {
            throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
        }

        string token = literal.Token.Text;
        int digitLength = token.Length;
        string suffix = string.Empty;
        if (token.EndsWith("UL", StringComparison.Ordinal))
        {
            suffix = "UL";
            digitLength -= 2;
        }
        else if (token.EndsWith("U", StringComparison.Ordinal)
            || token.EndsWith("L", StringComparison.Ordinal))
        {
            suffix = token[^1..];
            digitLength--;
        }

        if (digitLength == 0)
        {
            throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
        }

        for (int index = 0; index < digitLength; index++)
        {
            if (token[index] < '0' || token[index] > '9')
            {
                throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
            }
        }

        ExpressionSyntax complete = signedExpression is null ? literal : signedExpression;
        IOperation operation = RoslynPublicApi.GetOperation(method.SemanticModel, complete, "typecheck")
            ?? throw FrontendFailure.Toolchain("typecheck", "CSHARP_TOOLCHAIN_ADAPTER");
        SubsetValueType type = SubsetTypeRules.ValidateSymbol(
            operation.Type,
            method.SemanticModel.Compilation);
        if (!operation.ConstantValue.HasValue || type == SubsetValueType.Bool)
        {
            throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
        }

        string expectedSuffix = type switch
        {
            SubsetValueType.I32 => string.Empty,
            SubsetValueType.U32 => "U",
            SubsetValueType.I64 => "L",
            SubsetValueType.U64 => "UL",
            _ => throw FrontendFailure.Internal("typecheck"),
        };
        if (!string.Equals(suffix, expectedSuffix, StringComparison.Ordinal)
            || (negative && type != SubsetValueType.I32 && type != SubsetValueType.I64))
        {
            throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
        }
    }

    private static void AddOperationTree(
        IOperation operation,
        HashSet<IOperation> operations,
        List<IOperation> operationOrder,
        ref uint operationCount,
        ref uint closureOperationCount)
    {
        if (operations.Contains(operation))
        {
            return;
        }

        operationCount = SubsetLimits.Add(
            operationCount,
            1,
            SubsetLimits.OperationsPerMethodMaximum,
            "CSHARP_LIMIT_OPERATIONS_PER_METHOD");
        closureOperationCount = SubsetLimits.Add(
            closureOperationCount,
            1,
            SubsetLimits.OperationsPerClosureMaximum,
            "CSHARP_LIMIT_OPERATIONS_PER_CLOSURE");
        operations.Add(operation);
        operationOrder.Add(operation);

        foreach (IOperation child in operation.ChildOperations)
        {
            AddOperationTree(
                child,
                operations,
                operationOrder,
                ref operationCount,
                ref closureOperationCount);
        }
    }

    private static void ValidateOperation(
        IOperation operation,
        DeclaredSubsetMethod method,
        ImmutableArray<DeclaredSubsetMethod> declaredMethods,
        SortedSet<string> callees)
    {
        if (operation.Kind == OperationKind.Invalid
            || operation.Syntax.SyntaxTree is null
            || !ReferenceEquals(operation.Syntax.SyntaxTree, method.Declaration.SyntaxTree))
        {
            throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
        }

        if (operation.Type is not null
            && operation is not IConversionOperation
            && operation is not IInvocationOperation)
        {
            _ = SubsetTypeRules.ValidateSymbol(
                operation.Type,
                method.SemanticModel.Compilation);
        }

        switch (operation)
        {
            case IMethodBodyOperation:
            case IBlockOperation:
            case IVariableDeclarationGroupOperation:
            case IVariableDeclarationOperation:
            case IVariableInitializerOperation:
            case IExpressionStatementOperation:
            case IReturnOperation:
            case IParenthesizedOperation:
                return;
            case IConditionalOperation conditional:
                ValidateConditional(conditional, method);
                return;
            case IVariableDeclaratorOperation declarator:
                ValidateLocalSymbol(declarator.Symbol, method);
                if (!declarator.IgnoredArguments.IsEmpty || declarator.Initializer is null)
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
                }

                return;
            case ILiteralOperation literal:
                if (!literal.ConstantValue.HasValue)
                {
                    throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
                }

                return;
            case IParameterReferenceOperation parameter:
                ValidateParameterReference(parameter, method);
                return;
            case ILocalReferenceOperation local:
                ValidateLocalSymbol(local.Local, method);
                return;
            case ISimpleAssignmentOperation assignment:
                if (assignment.Target is not ILocalReferenceOperation target)
                {
                    if (assignment.Target is IParameterReferenceOperation)
                    {
                        throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
                    }

                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_PURITY");
                }

                ValidateLocalSymbol(target.Local, method);
                return;
            case IConversionOperation conversion:
                ValidateConversion(conversion, method);
                return;
            case IUnaryOperation unary:
                ValidateUnary(unary, method);
                return;
            case IBinaryOperation binary:
                ValidateBinary(binary, method);
                return;
            case IInvocationOperation invocation:
                callees.Add(ValidateInvocation(invocation, method, declaredMethods));
                return;
            case IArgumentOperation argument:
                if (argument.ArgumentKind != ArgumentKind.Explicit
                    || argument.Parameter is null
                    || !argument.InConversion.Exists
                    || !argument.InConversion.IsIdentity
                    || !argument.OutConversion.Exists
                    || !argument.OutConversion.IsIdentity)
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
                }

                return;
            case IFlowCaptureOperation:
            case IFlowCaptureReferenceOperation:
                ValidateFlowCapture(operation);
                return;
            case IFieldReferenceOperation:
            case IPropertyReferenceOperation:
            case IEventReferenceOperation:
            case IArrayElementReferenceOperation:
            case IObjectCreationOperation:
            case IAnonymousObjectCreationOperation:
            case IArrayCreationOperation:
            case IDynamicObjectCreationOperation:
            case IDynamicMemberReferenceOperation:
            case IInstanceReferenceOperation:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_PURITY");
            case IThrowOperation:
            case ITryOperation:
            case ILockOperation:
            case IUsingOperation:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_ABRUPT");
            case ILoopOperation:
            case ISwitchOperation:
            case IBranchOperation:
            case ILabeledOperation:
            case ILocalFunctionOperation:
            case IAnonymousFunctionOperation:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
            default:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
        }
    }

    private static void ValidateUnary(IUnaryOperation operation, DeclaredSubsetMethod method)
    {
        if (operation.OperatorMethod is not null)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
        }

        SubsetValueType operand = SubsetTypeRules.ValidateSymbol(
            operation.Operand.Type,
            method.SemanticModel.Compilation);
        SubsetValueType result = SubsetTypeRules.ValidateSymbol(
            operation.Type,
            method.SemanticModel.Compilation);
        if (operand != result)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
        }

        if (operation.Syntax is PrefixUnaryExpressionSyntax syntax
            && syntax.Operand is LiteralExpressionSyntax
            && syntax.IsKind(SyntaxKind.UnaryMinusExpression)
            && operation.ConstantValue.HasValue)
        {
            if (result != SubsetValueType.I32 && result != SubsetValueType.I64)
            {
                throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
            }

            return;
        }

        switch (operation.OperatorKind)
        {
            case UnaryOperatorKind.Not when result == SubsetValueType.Bool:
            case UnaryOperatorKind.BitwiseNegation when SubsetTypeRules.IsInteger(result):
                if (operation.IsChecked)
                {
                    throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
                }

                return;
            case UnaryOperatorKind.Minus
                when result == SubsetValueType.I32 || result == SubsetValueType.I64:
                ExplicitOverflowContext context = ContextFor(operation.Syntax);
                if (context == ExplicitOverflowContext.None)
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OVERFLOW_CONTEXT");
                }

                if (operation.IsChecked != (context == ExplicitOverflowContext.Checked))
                {
                    throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
                }

                if (operation.ConstantValue.HasValue)
                {
                    throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
                }

                return;
            default:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
        }
    }

    private static void ValidateConditional(
        IConditionalOperation operation,
        DeclaredSubsetMethod method)
    {
        if (SubsetTypeRules.ValidateSymbol(
                operation.Condition.Type,
                method.SemanticModel.Compilation) != SubsetValueType.Bool)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
        }

        if (operation.Syntax is IfStatementSyntax)
        {
            if (operation.Type is not null)
            {
                throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
            }

            return;
        }

        IOperation? whenFalseOperation = operation.WhenFalse;
        if (operation.Syntax is not ConditionalExpressionSyntax syntax
            || operation.Type is null
            || operation.WhenTrue.Type is null
            || whenFalseOperation is null
            || whenFalseOperation.Type is null)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
        }

        SubsetValueType result = SubsetTypeRules.ValidateSymbol(
            operation.Type,
            method.SemanticModel.Compilation);
        SubsetValueType whenTrue = SubsetTypeRules.ValidateSymbol(
            operation.WhenTrue.Type,
            method.SemanticModel.Compilation);
        SubsetValueType whenFalse = SubsetTypeRules.ValidateSymbol(
            whenFalseOperation.Type,
            method.SemanticModel.Compilation);
        SubsetValueType sourceTrue = SubsetTypeRules.ValidateSymbol(
            RoslynPublicApi.GetTypeInfo(method.SemanticModel, syntax.WhenTrue, "typecheck").Type,
            method.SemanticModel.Compilation);
        SubsetValueType sourceFalse = SubsetTypeRules.ValidateSymbol(
            RoslynPublicApi.GetTypeInfo(method.SemanticModel, syntax.WhenFalse, "typecheck").Type,
            method.SemanticModel.Compilation);
        if (result != whenTrue
            || result != whenFalse
            || sourceTrue != sourceFalse
            || sourceTrue != result)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
        }
    }

    private static void ValidateBinary(IBinaryOperation operation, DeclaredSubsetMethod method)
    {
        if (operation.OperatorMethod is not null)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
        }

        SubsetValueType left = SubsetTypeRules.ValidateSymbol(
            operation.LeftOperand.Type,
            method.SemanticModel.Compilation);
        SubsetValueType right = SubsetTypeRules.ValidateSymbol(
            operation.RightOperand.Type,
            method.SemanticModel.Compilation);
        SubsetValueType result = SubsetTypeRules.ValidateSymbol(
            operation.Type,
            method.SemanticModel.Compilation);
        bool shift = operation.OperatorKind == BinaryOperatorKind.LeftShift
            || operation.OperatorKind == BinaryOperatorKind.RightShift;
        if ((!shift && left != right)
            || (shift && right != SubsetValueType.I32))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
        }

        if (operation.Syntax is not BinaryExpressionSyntax syntax)
        {
            throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
        }

        TypeInfo sourceLeft = RoslynPublicApi.GetTypeInfo(method.SemanticModel, syntax.Left, "typecheck");
        TypeInfo sourceRight = RoslynPublicApi.GetTypeInfo(method.SemanticModel, syntax.Right, "typecheck");
        SubsetValueType unconvertedLeft = SubsetTypeRules.ValidateSymbol(
            sourceLeft.Type,
            method.SemanticModel.Compilation);
        SubsetValueType unconvertedRight = SubsetTypeRules.ValidateSymbol(
            sourceRight.Type,
            method.SemanticModel.Compilation);
        if ((!shift && unconvertedLeft != unconvertedRight)
            || (shift && unconvertedRight != SubsetValueType.I32))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
        }

        if (operation.ConstantValue.HasValue)
        {
            throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
        }

        switch (operation.OperatorKind)
        {
            case BinaryOperatorKind.Equals:
            case BinaryOperatorKind.NotEquals:
                Require(!operation.IsChecked, "CSHARP_SUBSET_OPERATION");
                Require(result == SubsetValueType.Bool, "CSHARP_SUBSET_OPERATION");
                return;
            case BinaryOperatorKind.LessThan:
            case BinaryOperatorKind.LessThanOrEqual:
            case BinaryOperatorKind.GreaterThan:
            case BinaryOperatorKind.GreaterThanOrEqual:
                Require(!operation.IsChecked, "CSHARP_SUBSET_OPERATION");
                Require(
                    SubsetTypeRules.IsInteger(left) && result == SubsetValueType.Bool,
                    "CSHARP_SUBSET_OPERATION");
                return;
            case BinaryOperatorKind.ConditionalAnd:
            case BinaryOperatorKind.ConditionalOr:
                Require(!operation.IsChecked, "CSHARP_SUBSET_OPERATION");
                Require(left == SubsetValueType.Bool && result == SubsetValueType.Bool, "CSHARP_SUBSET_OPERATION");
                return;
            case BinaryOperatorKind.And:
            case BinaryOperatorKind.Or:
            case BinaryOperatorKind.ExclusiveOr:
                Require(!operation.IsChecked, "CSHARP_SUBSET_OPERATION");
                if (left == SubsetValueType.Bool)
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
                }

                Require(result == left, "CSHARP_SUBSET_OPERATION");
                return;
            case BinaryOperatorKind.LeftShift:
            case BinaryOperatorKind.RightShift:
                Require(!operation.IsChecked, "CSHARP_SUBSET_OPERATION");
                Require(
                    SubsetTypeRules.IsInteger(left)
                        && right == SubsetValueType.I32
                        && result == left,
                    "CSHARP_SUBSET_OPERATION");
                return;
            case BinaryOperatorKind.Add:
            case BinaryOperatorKind.Subtract:
            case BinaryOperatorKind.Multiply:
            case BinaryOperatorKind.Divide:
            case BinaryOperatorKind.Remainder:
                Require(SubsetTypeRules.IsInteger(left) && result == left, "CSHARP_SUBSET_OPERATION");
                ValidateArithmeticContext(operation);
                return;
            default:
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OPERATION");
        }
    }

    private static void ValidateArithmeticContext(IBinaryOperation operation)
    {
        ExplicitOverflowContext context = ContextFor(operation.Syntax);
        if (context == ExplicitOverflowContext.None)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_OVERFLOW_CONTEXT");
        }

        bool expectedChecked = context == ExplicitOverflowContext.Checked;
        if (operation.OperatorKind == BinaryOperatorKind.Remainder)
        {
            expectedChecked = false;
        }

        if (operation.IsChecked != expectedChecked)
        {
            throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
        }
    }

    private static void ValidateConversion(
        IConversionOperation operation,
        DeclaredSubsetMethod method)
    {
        SubsetValueType source;
        SubsetValueType destination;
        try
        {
            source = SubsetTypeRules.ValidateSymbol(
                operation.Operand.Type,
                method.SemanticModel.Compilation);
            destination = SubsetTypeRules.ValidateSymbol(
                operation.Type,
                method.SemanticModel.Compilation);
        }
        catch (FrontendFailure failure) when (
            failure.Status == FrontendStatus.Rejected
            && string.Equals(failure.Code, "CSHARP_SUBSET_TYPE", StringComparison.Ordinal))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONVERSION");
        }
        if (operation.OperatorMethod is not null
            || operation.ConstrainedToType is not null
            || operation.IsTryCast
            || !operation.Conversion.Exists
            || operation.Conversion.IsUserDefined
            || operation.Conversion.MethodSymbol is not null)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONVERSION");
        }

        if (operation.Operand.ConstantValue.HasValue)
        {
            throw FrontendFailure.Rejected("typecheck", "CSHARP_SUBSET_LITERAL");
        }

        bool explicitInSource = operation.Syntax is CastExpressionSyntax;
        ExpressionSyntax sourceSyntax = operation.Operand.Syntax as ExpressionSyntax
            ?? throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
        Conversion classified = RoslynPublicApi.ClassifyConversion(
            method.SemanticModel,
            sourceSyntax,
            operation.Type!,
            explicitInSource,
            "subset");
        if (!classified.Exists
            || classified.IsUserDefined
            || classified.MethodSymbol is not null
            || classified.IsIdentity != operation.Conversion.IsIdentity
            || classified.IsNumeric != operation.Conversion.IsNumeric)
        {
            throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
        }

        if (source == destination)
        {
            if (!operation.Conversion.IsIdentity || operation.IsChecked)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONVERSION");
            }

            return;
        }

        if (!operation.Conversion.IsNumeric)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONVERSION");
        }

        if (operation.IsImplicit)
        {
            bool accepted = (source == SubsetValueType.I32 && destination == SubsetValueType.I64)
                || (source == SubsetValueType.U32 && destination == SubsetValueType.I64)
                || (source == SubsetValueType.U32 && destination == SubsetValueType.U64);
            if (!accepted || !classified.IsImplicit || operation.IsChecked)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONVERSION");
            }

            return;
        }

        if (!explicitInSource || (!classified.IsExplicit && !classified.IsImplicit))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONVERSION");
        }

        ExplicitOverflowContext context = ContextFor(operation.Syntax);
        if (operation.IsChecked || context == ExplicitOverflowContext.Checked)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CHECKED_CONVERSION");
        }

        if (context != ExplicitOverflowContext.Unchecked)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONVERSION");
        }
    }

    private static string ValidateInvocation(
        IInvocationOperation operation,
        DeclaredSubsetMethod method,
        ImmutableArray<DeclaredSubsetMethod> declaredMethods)
    {
        if (operation.Instance is not null
            || operation.TargetMethod.MethodKind != MethodKind.Ordinary
            || !operation.TargetMethod.IsStatic
            || operation.TargetMethod.IsGenericMethod
            || operation.TargetMethod.IsExtensionMethod
            || operation.TargetMethod.ReducedFrom is not null
            || operation.TargetMethod.OriginalDefinition is null
            || operation.Arguments.Length != operation.TargetMethod.Parameters.Length
            || operation.Syntax is not InvocationExpressionSyntax syntax)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
        }

        SymbolInfo information = RoslynPublicApi.GetSymbolInfo(
            method.SemanticModel,
            syntax.Expression,
            "subset");
        if (information.CandidateReason != CandidateReason.None
            || information.CandidateSymbols.Length != 0
            || information.Symbol is not IMethodSymbol resolved
            || !SymbolEqualityComparer.Default.Equals(resolved, operation.TargetMethod))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
        }

        DeclaredSubsetMethod? target = null;
        foreach (DeclaredSubsetMethod candidate in declaredMethods)
        {
            if (SymbolEqualityComparer.Default.Equals(candidate.Symbol, operation.TargetMethod)
                || SymbolEqualityComparer.Default.Equals(candidate.Symbol, operation.TargetMethod.OriginalDefinition))
            {
                if (target is not null)
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
                }

                target = candidate;
            }
        }

        if (target is null
            || operation.TargetMethod.DeclaringSyntaxReferences.Length != 1
            || !SymbolEqualityComparer.Default.Equals(
                operation.TargetMethod.ContainingAssembly,
                method.SemanticModel.Compilation.Assembly))
        {
            string namespaceName = operation.TargetMethod.ContainingNamespace?.ToDisplayString() ?? string.Empty;
            string typeName = operation.TargetMethod.ContainingType?.Name ?? string.Empty;
            if (namespaceName.StartsWith("System.IO", StringComparison.Ordinal)
                || namespaceName.StartsWith("System.Reflection", StringComparison.Ordinal)
                || namespaceName.StartsWith("System.Threading", StringComparison.Ordinal)
                || namespaceName.StartsWith("System.Runtime.InteropServices", StringComparison.Ordinal)
                || (string.Equals(namespaceName, "System", StringComparison.Ordinal)
                    && (string.Equals(typeName, "Console", StringComparison.Ordinal)
                        || string.Equals(typeName, "Environment", StringComparison.Ordinal)
                        || string.Equals(typeName, "DateTime", StringComparison.Ordinal)
                        || string.Equals(typeName, "DateTimeOffset", StringComparison.Ordinal)
                        || string.Equals(typeName, "Random", StringComparison.Ordinal))))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_PURITY");
            }

            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
        }

        for (int index = 0; index < operation.Arguments.Length; index++)
        {
            IArgumentOperation argument = operation.Arguments[index];
            if (argument.ArgumentKind != ArgumentKind.Explicit
                || argument.Parameter is null
                || argument.Parameter.Ordinal != index
                || argument.Syntax is not ArgumentSyntax argumentSyntax
                || argumentSyntax.NameColon is not null
                || !argumentSyntax.RefKindKeyword.IsKind(SyntaxKind.None)
                || !argument.InConversion.Exists
                || !argument.InConversion.IsIdentity
                || !argument.OutConversion.Exists
                || !argument.OutConversion.IsIdentity)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CALL");
            }
        }

        return target.CanonicalId;
    }

    private static void ValidateParameterReference(
        IParameterReferenceOperation operation,
        DeclaredSubsetMethod method)
    {
        if (!SymbolEqualityComparer.Default.Equals(operation.Parameter.ContainingSymbol, method.Symbol)
            || operation.Parameter.RefKind != RefKind.None
            || operation.Parameter.IsOptional
            || operation.Parameter.IsParams)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_PURITY");
        }
    }

    private static void ValidateLocalSymbol(ILocalSymbol local, DeclaredSubsetMethod method)
    {
        if (local.IsConst
            || local.RefKind != RefKind.None
            || local.DeclaringSyntaxReferences.Length != 1
            || !ReferenceEquals(local.DeclaringSyntaxReferences[0].SyntaxTree, method.Declaration.SyntaxTree))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
        }

        _ = SubsetTypeRules.ValidateSymbol(local.Type, method.SemanticModel.Compilation);
    }

    private static void ValidateFlowCapture(IOperation operation)
    {
        foreach (SyntaxNode ancestor in operation.Syntax.AncestorsAndSelf())
        {
            if (ancestor is ConditionalExpressionSyntax
                || ancestor is InvocationExpressionSyntax
                || (ancestor is BinaryExpressionSyntax binary
                    && (binary.IsKind(SyntaxKind.LogicalAndExpression)
                        || binary.IsKind(SyntaxKind.LogicalOrExpression))))
            {
                return;
            }
        }

        throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
    }

    private static void ValidateGraph(ControlFlowGraph graph)
    {
        if (graph.Blocks.Length < 2
            || graph.Blocks[0].Kind != BasicBlockKind.Entry
            || graph.Blocks[^1].Kind != BasicBlockKind.Exit
            || graph.Root.Kind != ControlFlowRegionKind.Root
            || graph.Root.EnclosingRegion is not null)
        {
            throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
        }

        ValidateRegion(graph.Root, root: true);
        for (int index = 0; index < graph.Blocks.Length; index++)
        {
            BasicBlock block = graph.Blocks[index];
            BasicBlockKind expected = index == 0
                ? BasicBlockKind.Entry
                : index == graph.Blocks.Length - 1
                    ? BasicBlockKind.Exit
                    : BasicBlockKind.Block;
            if (block.Ordinal != index || block.Kind != expected || !block.IsReachable)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
            }

            if (block.Kind == BasicBlockKind.Exit)
            {
                if (block.FallThroughSuccessor is not null
                    || block.ConditionalSuccessor is not null
                    || block.ConditionKind != ControlFlowConditionKind.None
                    || block.BranchValue is not null)
                {
                    throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
                }

                continue;
            }

            if (block.FallThroughSuccessor is null)
            {
                throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
            }

            ValidateBranch(block.FallThroughSuccessor);
            if (block.ConditionKind == ControlFlowConditionKind.None)
            {
                if (block.ConditionalSuccessor is not null)
                {
                    throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
                }
            }
            else
            {
                if (block.ConditionalSuccessor is null || block.BranchValue is null)
                {
                    throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
                }

                ValidateBranch(block.ConditionalSuccessor);
            }
        }
    }

    private static void ValidateRegion(ControlFlowRegion region, bool root)
    {
        if ((root && region.Kind != ControlFlowRegionKind.Root)
            || (!root && region.Kind != ControlFlowRegionKind.LocalLifetime)
            || region.ExceptionType is not null
            || region.LocalFunctions.Length != 0)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_ABRUPT");
        }

        foreach (ControlFlowRegion nested in region.NestedRegions)
        {
            ValidateRegion(nested, root: false);
        }
    }

    private static void ValidateBranch(ControlFlowBranch branch)
    {
        if ((branch.Semantics != ControlFlowBranchSemantics.Regular
                && branch.Semantics != ControlFlowBranchSemantics.Return)
            || branch.Destination is null
            || !branch.FinallyRegions.IsEmpty)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_ABRUPT");
        }

        foreach (ControlFlowRegion region in branch.EnteringRegions)
        {
            if (region.Kind != ControlFlowRegionKind.LocalLifetime)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_ABRUPT");
            }
        }

        foreach (ControlFlowRegion region in branch.LeavingRegions)
        {
            if (region.Kind != ControlFlowRegionKind.LocalLifetime)
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_ABRUPT");
            }
        }
    }

    private static void ValidateDefiniteAssignment(
        ControlFlowGraph graph,
        DeclaredSubsetMethod method)
    {
        var universe = new HashSet<ISymbol>(SymbolEqualityComparer.Default);
        foreach (VariableDeclaratorSyntax declaration in method.Declaration.Body!
            .DescendantNodes(descendIntoTrivia: false)
            .OfType<VariableDeclaratorSyntax>())
        {
            IOperation operation = RoslynPublicApi.GetOperation(
                method.SemanticModel,
                declaration,
                "subset")
                ?? throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
            if (operation is not IVariableDeclaratorOperation declarator)
            {
                throw FrontendFailure.Toolchain("subset", "CSHARP_TOOLCHAIN_ADAPTER");
            }

            universe.Add(declarator.Symbol);
        }

        int count = graph.Blocks.Length;
        var uses = new HashSet<ISymbol>[count];
        var definitions = new HashSet<ISymbol>[count];
        for (int index = 0; index < count; index++)
        {
            uses[index] = new HashSet<ISymbol>(SymbolEqualityComparer.Default);
            definitions[index] = new HashSet<ISymbol>(SymbolEqualityComparer.Default);
            foreach (IOperation operation in graph.Blocks[index].Operations)
            {
                CollectFlow(operation, universe, definitions[index], uses[index], assignmentTarget: false);
            }

            if (graph.Blocks[index].BranchValue is not null)
            {
                CollectFlow(
                    graph.Blocks[index].BranchValue!,
                    universe,
                    definitions[index],
                    uses[index],
                    assignmentTarget: false);
            }
        }

        var incoming = new HashSet<ISymbol>[count];
        var outgoing = new HashSet<ISymbol>[count];
        incoming[0] = new HashSet<ISymbol>(SymbolEqualityComparer.Default);
        outgoing[0] = new HashSet<ISymbol>(definitions[0], SymbolEqualityComparer.Default);
        for (int index = 1; index < count; index++)
        {
            incoming[index] = new HashSet<ISymbol>(universe, SymbolEqualityComparer.Default);
            outgoing[index] = new HashSet<ISymbol>(universe, SymbolEqualityComparer.Default);
        }

        bool changed;
        do
        {
            changed = false;
            for (int index = 1; index < count; index++)
            {
                BasicBlock block = graph.Blocks[index];
                if (block.Predecessors.Length == 0)
                {
                    throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
                }

                var nextIncoming = new HashSet<ISymbol>(
                    outgoing[block.Predecessors[0].Source.Ordinal],
                    SymbolEqualityComparer.Default);
                for (int predecessor = 1; predecessor < block.Predecessors.Length; predecessor++)
                {
                    nextIncoming.IntersectWith(outgoing[block.Predecessors[predecessor].Source.Ordinal]);
                }

                var nextOutgoing = new HashSet<ISymbol>(nextIncoming, SymbolEqualityComparer.Default);
                nextOutgoing.UnionWith(definitions[index]);
                if (!incoming[index].SetEquals(nextIncoming))
                {
                    incoming[index] = nextIncoming;
                    changed = true;
                }

                if (!outgoing[index].SetEquals(nextOutgoing))
                {
                    outgoing[index] = nextOutgoing;
                    changed = true;
                }
            }
        }
        while (changed);

        for (int index = 0; index < count; index++)
        {
            if (!uses[index].IsSubsetOf(incoming[index]))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_SUBSET_CONTROL_FLOW");
            }
        }
    }

    private static void CollectFlow(
        IOperation operation,
        HashSet<ISymbol> universe,
        HashSet<ISymbol> definitions,
        HashSet<ISymbol> uses,
        bool assignmentTarget)
    {
        if (operation is ISimpleAssignmentOperation assignment
            && assignment.Target is ILocalReferenceOperation target)
        {
            CollectFlow(assignment.Value, universe, definitions, uses, assignmentTarget: false);
            if (universe.Contains(target.Local))
            {
                definitions.Add(target.Local);
            }

            return;
        }

        if (operation is IVariableDeclaratorOperation declarator)
        {
            if (declarator.Initializer is not null)
            {
                CollectFlow(declarator.Initializer, universe, definitions, uses, assignmentTarget: false);
            }

            if (universe.Contains(declarator.Symbol))
            {
                definitions.Add(declarator.Symbol);
            }

            return;
        }

        if (operation is ILocalReferenceOperation local
            && !assignmentTarget
            && universe.Contains(local.Local)
            && !definitions.Contains(local.Local))
        {
            uses.Add(local.Local);
        }

        foreach (IOperation child in operation.ChildOperations)
        {
            CollectFlow(child, universe, definitions, uses, assignmentTarget: false);
        }
    }

    internal static ExplicitOverflowContext ContextFor(SyntaxNode syntax)
    {
        foreach (SyntaxNode ancestor in syntax.AncestorsAndSelf())
        {
            if (ancestor is CheckedExpressionSyntax expression)
            {
                return expression.Keyword.IsKind(SyntaxKind.CheckedKeyword)
                    ? ExplicitOverflowContext.Checked
                    : ExplicitOverflowContext.Unchecked;
            }

            if (ancestor is CheckedStatementSyntax statement)
            {
                return statement.Keyword.IsKind(SyntaxKind.CheckedKeyword)
                    ? ExplicitOverflowContext.Checked
                    : ExplicitOverflowContext.Unchecked;
            }
        }

        return ExplicitOverflowContext.None;
    }

    private static void Require(bool condition, string code)
    {
        if (!condition)
        {
            throw FrontendFailure.Rejected("subset", code);
        }
    }
}
