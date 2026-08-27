using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

internal static class CSharpLowering
{
    internal static LoweredClosure Lower(
        Selection selection,
        SubsetClosure closure,
        ContractSet contracts)
    {
        ValidateInputs(selection, closure, contracts);
        var functions = new LoweredFunction[closure.Methods.Length];
        for (int index = 0; index < closure.Methods.Length; index++)
        {
            SubsetMethod method = closure.Methods[index];
            if (method.Callees.Length != 0)
            {
                // T10 is the sole owner of CallStatic lowering.
                throw LoweringFailure.Operation();
            }

            functions[index] = new LoweringMethodBuilder(method).Build();
        }

        var lowered = new LoweredClosure(selection.Sha256, functions);
        LoweringValidator.Validate(lowered);
        return lowered;
    }

    private static void ValidateInputs(
        Selection selection,
        SubsetClosure closure,
        ContractSet contracts)
    {
        if (!string.Equals(selection.Sha256, contracts.SelectionSha256, StringComparison.Ordinal)
            || closure.Methods.Length != contracts.Contracts.Count
            || closure.SelectedRoots.Length != selection.Raw.Methods.Count)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
        }

        for (int index = 0; index < closure.SelectedRoots.Length; index++)
        {
            if (!string.Equals(
                closure.SelectedRoots[index],
                selection.Raw.Methods[index],
                StringComparison.Ordinal))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
            }
        }

        for (int index = 0; index < closure.Methods.Length; index++)
        {
            if (!string.Equals(
                closure.Methods[index].CanonicalId,
                contracts.Contracts[index].Normalized.FunctionId,
                StringComparison.Ordinal))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
            }
        }
    }
}

internal static class LoweringFailure
{
    internal static FrontendFailure Operation()
    {
        return FrontendFailure.Rejected("lowering", "CSHARP_LOWERING_OPERATION");
    }

    internal static FrontendFailure ControlFlow()
    {
        return FrontendFailure.Rejected("lowering", "CSHARP_LOWERING_CFG");
    }
}

internal sealed class LoweringMethodBuilder
{
    private readonly SubsetMethod method;
    private readonly List<BlockBuilder> blocks = new List<BlockBuilder>();
    private readonly Dictionary<ILocalSymbol, LocalBuilder> locals =
        new Dictionary<ILocalSymbol, LocalBuilder>(SymbolEqualityComparer.Default);
    private readonly Dictionary<IParameterSymbol, ValueBuilder> parameters =
        new Dictionary<IParameterSymbol, ValueBuilder>(SymbolEqualityComparer.Default);
    private readonly List<LocalBuilder> orderedLocals = new List<LocalBuilder>();
    private readonly HashSet<LoweredFeature> features = new HashSet<LoweredFeature>();

    internal LoweringMethodBuilder(SubsetMethod method)
    {
        this.method = method;
    }

    internal LoweredFunction Build()
    {
        LoweringCfgAdapter.Validate(method);
        InitializeBindings();
        BlockBuilder entry = NewBlock();
        var environment = new Dictionary<ILocalSymbol, ValueBuilder>(
            SymbolEqualityComparer.Default);
        FlowState? final = LowerBlock(
            method.Declaration.Body ?? throw LoweringFailure.ControlFlow(),
            new FlowState(entry, environment));
        if (final is not null || blocks.Any(block => block.Terminator is null))
        {
            throw LoweringFailure.ControlFlow();
        }

        LoweredFunction function = Freeze();
        LoweringValidator.Validate(function);
        return function;
    }

    private void InitializeBindings()
    {
        for (int index = 0; index < method.Symbol.Parameters.Length; index++)
        {
            IParameterSymbol parameter = method.Symbol.Parameters[index];
            if (parameter.Ordinal != index)
            {
                throw LoweringFailure.Operation();
            }

            SubsetValueType type = ExactType(parameter.Type);
            parameters.Add(parameter, ValueBuilder.Named("arg" + index.ToString(CultureInfo.InvariantCulture), type));
        }

        IEnumerable<VariableDeclaratorSyntax> declarations = method.Declaration.Body!
            .DescendantNodes(descendIntoTrivia: false)
            .OfType<VariableDeclaratorSyntax>()
            .OrderBy(declaration => declaration.SpanStart);
        int localIndex = 0;
        foreach (VariableDeclaratorSyntax declaration in declarations)
        {
            IOperation operation = Operation(declaration);
            if (operation is not IVariableDeclaratorOperation declarator
                || !ReferenceEquals(operation.Syntax, declaration)
                || locals.ContainsKey(declarator.Symbol))
            {
                throw LoweringFailure.Operation();
            }

            var local = new LocalBuilder(
                declarator.Symbol,
                "local" + localIndex.ToString(CultureInfo.InvariantCulture),
                ExactType(declarator.Symbol.Type),
                declaration.SpanStart);
            locals.Add(declarator.Symbol, local);
            orderedLocals.Add(local);
            localIndex++;
        }

        if (orderedLocals.Count != 0)
        {
            features.Add(LoweredFeature.MutableLocal);
        }
    }

    private FlowState? LowerBlock(BlockSyntax block, FlowState state)
    {
        FlowState? current = state;
        foreach (StatementSyntax statement in block.Statements)
        {
            if (current is null)
            {
                throw LoweringFailure.ControlFlow();
            }

            current = LowerStatement(statement, current);
        }

        return current;
    }

    private FlowState? LowerStatement(StatementSyntax statement, FlowState state)
    {
        switch (statement)
        {
            case BlockSyntax block:
                return LowerBlock(block, state);
            case LocalDeclarationStatementSyntax local:
                return LowerLocal(local, state);
            case ExpressionStatementSyntax expression:
                return LowerExpressionStatement(expression, state);
            case CheckedStatementSyntax checkedStatement:
                return LowerBlock(checkedStatement.Block, state);
            case IfStatementSyntax conditional:
                return LowerIf(conditional, state);
            case ReturnStatementSyntax returned:
                return LowerReturn(returned, state);
            default:
                throw LoweringFailure.Operation();
        }
    }

    private FlowState LowerLocal(LocalDeclarationStatementSyntax statement, FlowState state)
    {
        if (statement.Declaration.Variables.Count != 1)
        {
            throw LoweringFailure.Operation();
        }

        VariableDeclaratorSyntax declaration = statement.Declaration.Variables[0];
        if (declaration.Initializer is null
            || Operation(declaration) is not IVariableDeclaratorOperation operation
            || !locals.TryGetValue(operation.Symbol, out LocalBuilder? local))
        {
            throw LoweringFailure.Operation();
        }

        ExpressionResult result = LowerExpression(declaration.Initializer.Value, state);
        result = ApplyContextConversion(
            operation.Initializer?.Value ?? throw LoweringFailure.Operation(),
            result,
            Origin(declaration.Initializer.Value));
        if (result.Value.Type != local.Type)
        {
            throw LoweringFailure.Operation();
        }

        ValueBuilder assigned;
        if (result.Value.Kind == ValueBuilderKind.Boolean
            || result.Value.Kind == ValueBuilderKind.Integer)
        {
            assigned = EmitConst(result.Flow.Block, result.Value, Origin(declaration.Initializer.Value));
        }
        else if (result.Value.Definition is InstructionBuilder)
        {
            assigned = result.Value;
        }
        else
        {
            assigned = EmitCopy(
                result.Flow.Block,
                local,
                result.Value,
                Origin(declaration.Initializer.Value));
        }

        result.Flow.Environment[local.Symbol] = assigned;
        return result.Flow;
    }

    private FlowState LowerExpressionStatement(ExpressionStatementSyntax statement, FlowState state)
    {
        if (statement.Expression is not AssignmentExpressionSyntax assignment
            || !assignment.IsKind(SyntaxKind.SimpleAssignmentExpression)
            || Operation(assignment) is not ISimpleAssignmentOperation operation
            || operation.Target is not ILocalReferenceOperation target
            || !locals.TryGetValue(target.Local, out LocalBuilder? local))
        {
            // Invocation statements are admitted by T07 but remain T10-owned.
            throw LoweringFailure.Operation();
        }

        ExpressionResult result = LowerExpression(assignment.Right, state);
        result = ApplyContextConversion(operation.Value, result, Origin(assignment.Right));
        if (result.Value.Type != local.Type)
        {
            throw LoweringFailure.Operation();
        }

        ValueBuilder copied = EmitCopy(
            result.Flow.Block,
            local,
            result.Value,
            Origin(assignment));
        result.Flow.Environment[local.Symbol] = copied;
        return result.Flow;
    }

    private FlowState? LowerIf(IfStatementSyntax conditional, FlowState state)
    {
        if (Operation(conditional) is not IConditionalOperation operation
            || operation.Type is not null)
        {
            throw LoweringFailure.Operation();
        }

        ExpressionResult condition = LowerExpression(conditional.Condition, state);
        if (condition.Value.Type != SubsetValueType.Bool)
        {
            throw LoweringFailure.Operation();
        }

        features.Add(LoweredFeature.Branch);
        BlockBuilder whenFalse = NewBlock();
        BlockBuilder whenTrue = NewBlock();
        SetBranch(
            condition.Flow.Block,
            condition.Value,
            whenFalse,
            whenTrue,
            Origin(conditional.Condition));

        var incoming = new Dictionary<ILocalSymbol, ValueBuilder>(
            condition.Flow.Environment,
            SymbolEqualityComparer.Default);
        FlowState? falseFlow = conditional.Else is null
            ? new FlowState(
                whenFalse,
                new Dictionary<ILocalSymbol, ValueBuilder>(incoming, SymbolEqualityComparer.Default))
            : LowerStatement(
                conditional.Else.Statement,
                new FlowState(
                    whenFalse,
                    new Dictionary<ILocalSymbol, ValueBuilder>(incoming, SymbolEqualityComparer.Default)));
        FlowState? trueFlow = LowerStatement(
            conditional.Statement,
            new FlowState(
                whenTrue,
                new Dictionary<ILocalSymbol, ValueBuilder>(incoming, SymbolEqualityComparer.Default)));

        return MergeFlows(falseFlow, trueFlow, incoming.Keys, Origin(conditional));
    }

    private FlowState? LowerReturn(ReturnStatementSyntax returned, FlowState state)
    {
        if (returned.Expression is null
            || Operation(returned) is not IReturnOperation operation
            || operation.ReturnedValue is null)
        {
            throw LoweringFailure.Operation();
        }

        ExpressionResult value = LowerExpression(returned.Expression, state);
        value = ApplyContextConversion(
            operation.ReturnedValue,
            value,
            Origin(returned.Expression));
        SubsetValueType resultType = ExactType(method.Symbol.ReturnType);
        if (value.Value.Type != resultType)
        {
            throw LoweringFailure.Operation();
        }

        SetReturn(value.Flow.Block, value.Value, Origin(returned));
        return null;
    }

    private ExpressionResult LowerExpression(ExpressionSyntax expression, FlowState state)
    {
        switch (expression)
        {
            case ParenthesizedExpressionSyntax parenthesized:
                return LowerExpression(parenthesized.Expression, state);
            case CheckedExpressionSyntax checkedExpression:
                return LowerExpression(checkedExpression.Expression, state);
            case LiteralExpressionSyntax literal:
                return new ExpressionResult(
                    state,
                    EmitConst(state.Block, Literal(literal), Origin(literal)));
            case IdentifierNameSyntax identifier:
                return new ExpressionResult(state, Reference(identifier, state.Environment));
            case PrefixUnaryExpressionSyntax unary:
                return LowerUnary(unary, state);
            case BinaryExpressionSyntax binary:
                return LowerBinary(binary, state);
            case CastExpressionSyntax cast:
                return LowerConversion(cast, state);
            case ConditionalExpressionSyntax conditional:
                return LowerConditional(conditional, state);
            case InvocationExpressionSyntax:
                throw LoweringFailure.Operation();
            default:
                throw LoweringFailure.Operation();
        }
    }

    private ExpressionResult LowerUnary(PrefixUnaryExpressionSyntax syntax, FlowState state)
    {
        IOperation untyped = Operation(syntax);
        if (syntax.IsKind(SyntaxKind.UnaryMinusExpression)
            && syntax.Operand is LiteralExpressionSyntax
            && untyped.ConstantValue.HasValue)
        {
            return new ExpressionResult(
                state,
                EmitConst(state.Block, Literal(untyped), Origin(syntax)));
        }

        if (untyped is not IUnaryOperation operation
            || !ReferenceEquals(operation.Syntax, syntax))
        {
            throw LoweringFailure.Operation();
        }

        ExpressionResult operand = LowerExpression(syntax.Operand, state);
        SubsetValueType resultType = ExactType(operation.Type);
        LoweredUnaryOperator lowered;
        ExplicitOverflowContext context = ExplicitOverflowContext.None;
        LoweredSafetyCheck[] checks = Array.Empty<LoweredSafetyCheck>();
        switch (operation.OperatorKind)
        {
            case UnaryOperatorKind.Not
                when operand.Value.Type == SubsetValueType.Bool
                    && resultType == SubsetValueType.Bool
                    && !operation.IsChecked:
                lowered = LoweredUnaryOperator.BoolNot;
                break;
            case UnaryOperatorKind.BitwiseNegation
                when SubsetTypeRules.IsInteger(operand.Value.Type)
                    && resultType == operand.Value.Type
                    && !operation.IsChecked:
                lowered = LoweredUnaryOperator.BvNot;
                break;
            case UnaryOperatorKind.Minus
                when (resultType == SubsetValueType.I32 || resultType == SubsetValueType.I64)
                    && resultType == operand.Value.Type:
                lowered = LoweredUnaryOperator.BvNeg;
                context = SubsetBodies.ContextFor(syntax);
                if (context == ExplicitOverflowContext.None
                    || operation.IsChecked != (context == ExplicitOverflowContext.Checked))
                {
                    throw LoweringFailure.Operation();
                }

                if (context == ExplicitOverflowContext.Checked)
                {
                    checks = new[]
                    {
                        SafetyCheck(
                            LoweredSafetyCheckKind.IntegerNoOverflow,
                            LoweredCheckOperation.Neg,
                            resultType),
                    };
                }

                break;
            default:
                throw LoweringFailure.Operation();
        }

        ValueBuilder value = EmitUnary(
            operand.Flow.Block,
            lowered,
            resultType,
            context,
            operand.Value,
            checks,
            Origin(syntax));
        return new ExpressionResult(operand.Flow, value);
    }

    private ExpressionResult LowerBinary(BinaryExpressionSyntax syntax, FlowState state)
    {
        if (Operation(syntax) is not IBinaryOperation operation
            || !ReferenceEquals(operation.Syntax, syntax))
        {
            throw LoweringFailure.Operation();
        }

        if (operation.OperatorKind == BinaryOperatorKind.ConditionalAnd
            || operation.OperatorKind == BinaryOperatorKind.ConditionalOr)
        {
            return LowerShortCircuit(syntax, operation, state);
        }

        ExpressionResult left = LowerExpression(syntax.Left, state);
        ExpressionResult right = LowerExpression(syntax.Right, left.Flow);
        if (left.Value.Type != ExactType(operation.LeftOperand.Type)
            || right.Value.Type != ExactType(operation.RightOperand.Type))
        {
            throw LoweringFailure.Operation();
        }

        return LowerEagerBinary(syntax, operation, left.Value, right);
    }

    private ExpressionResult LowerEagerBinary(
        BinaryExpressionSyntax syntax,
        IBinaryOperation operation,
        ValueBuilder left,
        ExpressionResult right)
    {
        SubsetValueType resultType = ExactType(operation.Type);
        ExplicitOverflowContext context = ExplicitOverflowContext.None;
        LoweredSafetyCheck[] checks = Array.Empty<LoweredSafetyCheck>();
        LoweredBinaryOperator lowered;
        switch (operation.OperatorKind)
        {
            case BinaryOperatorKind.Equals:
                lowered = LoweredBinaryOperator.Eq;
                break;
            case BinaryOperatorKind.NotEquals:
                lowered = LoweredBinaryOperator.NotEq;
                break;
            case BinaryOperatorKind.LessThan:
                lowered = SignedComparison(left.Type)
                    ? LoweredBinaryOperator.SignedLt
                    : LoweredBinaryOperator.UnsignedLt;
                break;
            case BinaryOperatorKind.LessThanOrEqual:
                lowered = SignedComparison(left.Type)
                    ? LoweredBinaryOperator.SignedLe
                    : LoweredBinaryOperator.UnsignedLe;
                break;
            case BinaryOperatorKind.GreaterThan:
                lowered = SignedComparison(left.Type)
                    ? LoweredBinaryOperator.SignedGt
                    : LoweredBinaryOperator.UnsignedGt;
                break;
            case BinaryOperatorKind.GreaterThanOrEqual:
                lowered = SignedComparison(left.Type)
                    ? LoweredBinaryOperator.SignedGe
                    : LoweredBinaryOperator.UnsignedGe;
                break;
            case BinaryOperatorKind.Add:
            case BinaryOperatorKind.Subtract:
            case BinaryOperatorKind.Multiply:
                context = ArithmeticContext(operation, syntax);
                lowered = operation.OperatorKind switch
                {
                    BinaryOperatorKind.Add => LoweredBinaryOperator.BvAdd,
                    BinaryOperatorKind.Subtract => LoweredBinaryOperator.BvSub,
                    BinaryOperatorKind.Multiply => LoweredBinaryOperator.BvMul,
                    _ => throw LoweringFailure.Operation(),
                };
                if (context == ExplicitOverflowContext.Checked)
                {
                    LoweredCheckOperation checkOperation = operation.OperatorKind switch
                    {
                        BinaryOperatorKind.Add => LoweredCheckOperation.Add,
                        BinaryOperatorKind.Subtract => LoweredCheckOperation.Sub,
                        BinaryOperatorKind.Multiply => LoweredCheckOperation.Mul,
                        _ => throw LoweringFailure.Operation(),
                    };
                    checks = new[]
                    {
                        SafetyCheck(
                            LoweredSafetyCheckKind.IntegerNoOverflow,
                            checkOperation,
                            left.Type),
                    };
                }

                break;
            case BinaryOperatorKind.Divide:
            case BinaryOperatorKind.Remainder:
                context = ArithmeticContext(operation, syntax);
                bool signed = IsSigned(left.Type);
                bool divide = operation.OperatorKind == BinaryOperatorKind.Divide;
                lowered = (signed, divide) switch
                {
                    (true, true) => LoweredBinaryOperator.BvSdiv,
                    (true, false) => LoweredBinaryOperator.BvSrem,
                    (false, true) => LoweredBinaryOperator.BvUdiv,
                    (false, false) => LoweredBinaryOperator.BvUrem,
                };
                LoweredCheckOperation divrem = divide
                    ? LoweredCheckOperation.Div
                    : LoweredCheckOperation.Rem;
                checks = signed
                    ? new[]
                    {
                        SafetyCheck(LoweredSafetyCheckKind.DivisorNonzero, divrem, left.Type),
                        SafetyCheck(
                            LoweredSafetyCheckKind.SignedDivremRepresentable,
                            divrem,
                            left.Type),
                    }
                    : new[]
                    {
                        SafetyCheck(LoweredSafetyCheckKind.DivisorNonzero, divrem, left.Type),
                    };
                break;
            case BinaryOperatorKind.And:
                lowered = LoweredBinaryOperator.BvAnd;
                break;
            case BinaryOperatorKind.Or:
                lowered = LoweredBinaryOperator.BvOr;
                break;
            case BinaryOperatorKind.ExclusiveOr:
                lowered = LoweredBinaryOperator.BvXor;
                break;
            case BinaryOperatorKind.LeftShift:
            case BinaryOperatorKind.RightShift:
                return LowerShift(syntax, operation, left, right);
            default:
                throw LoweringFailure.Operation();
        }

        ValueBuilder value = EmitBinary(
            right.Flow.Block,
            lowered,
            resultType,
            context,
            shiftCountMask: false,
            left,
            right.Value,
            checks,
            Origin(syntax));
        return new ExpressionResult(right.Flow, value);
    }

    private ExpressionResult LowerShift(
        BinaryExpressionSyntax syntax,
        IBinaryOperation operation,
        ValueBuilder left,
        ExpressionResult right)
    {
        if (!SubsetTypeRules.IsInteger(left.Type)
            || right.Value.Type != SubsetValueType.I32
            || ExactType(operation.Type) != left.Type
            || operation.IsChecked)
        {
            throw LoweringFailure.Operation();
        }

        int mask = Width(left.Type) - 1;
        ValueBuilder maskValue = ValueBuilder.Integer(
            mask.ToString(CultureInfo.InvariantCulture),
            SubsetValueType.I32);
        ValueBuilder masked = EmitBinary(
            right.Flow.Block,
            LoweredBinaryOperator.BvAnd,
            SubsetValueType.I32,
            ExplicitOverflowContext.None,
            shiftCountMask: true,
            right.Value,
            maskValue,
            Array.Empty<LoweredSafetyCheck>(),
            Origin(syntax.Right));
        LoweredBinaryOperator shifted = operation.OperatorKind switch
        {
            BinaryOperatorKind.LeftShift => LoweredBinaryOperator.BvShl,
            BinaryOperatorKind.RightShift when IsSigned(left.Type) => LoweredBinaryOperator.BvAshr,
            BinaryOperatorKind.RightShift => LoweredBinaryOperator.BvLshr,
            _ => throw LoweringFailure.Operation(),
        };
        ValueBuilder result = EmitBinary(
            right.Flow.Block,
            shifted,
            left.Type,
            ExplicitOverflowContext.None,
            shiftCountMask: false,
            left,
            masked,
            Array.Empty<LoweredSafetyCheck>(),
            Origin(syntax));
        return new ExpressionResult(right.Flow, result);
    }

    private ExpressionResult LowerShortCircuit(
        BinaryExpressionSyntax syntax,
        IBinaryOperation operation,
        FlowState state)
    {
        if (ExactType(operation.Type) != SubsetValueType.Bool
            || ExactType(operation.LeftOperand.Type) != SubsetValueType.Bool
            || ExactType(operation.RightOperand.Type) != SubsetValueType.Bool
            || operation.IsChecked)
        {
            throw LoweringFailure.Operation();
        }

        ExpressionResult left = LowerExpression(syntax.Left, state);
        features.Add(LoweredFeature.Branch);
        BlockBuilder whenFalse = NewBlock();
        BlockBuilder whenTrue = NewBlock();
        SetBranch(left.Flow.Block, left.Value, whenFalse, whenTrue, Origin(syntax));
        BlockBuilder join = NewBlock();
        ParameterBuilder parameter = join.AddParameter(SubsetValueType.Bool);

        if (operation.OperatorKind == BinaryOperatorKind.ConditionalAnd)
        {
            SetJump(
                whenFalse,
                join,
                new[] { ValueBuilder.Boolean(false) },
                Origin(syntax));
            ExpressionResult right = LowerExpression(
                syntax.Right,
                new FlowState(
                    whenTrue,
                    new Dictionary<ILocalSymbol, ValueBuilder>(
                        left.Flow.Environment,
                        SymbolEqualityComparer.Default)));
            SetJump(right.Flow.Block, join, new[] { right.Value }, Origin(syntax));
        }
        else
        {
            ExpressionResult right = LowerExpression(
                syntax.Right,
                new FlowState(
                    whenFalse,
                    new Dictionary<ILocalSymbol, ValueBuilder>(
                        left.Flow.Environment,
                        SymbolEqualityComparer.Default)));
            SetJump(right.Flow.Block, join, new[] { right.Value }, Origin(syntax));
            SetJump(
                whenTrue,
                join,
                new[] { ValueBuilder.Boolean(true) },
                Origin(syntax));
        }

        return new ExpressionResult(
            new FlowState(
                join,
                new Dictionary<ILocalSymbol, ValueBuilder>(
                    left.Flow.Environment,
                    SymbolEqualityComparer.Default)),
            ValueBuilder.Defined(parameter));
    }

    private ExpressionResult LowerConversion(CastExpressionSyntax syntax, FlowState state)
    {
        if (Operation(syntax) is not IConversionOperation operation
            || !ReferenceEquals(operation.Syntax, syntax))
        {
            throw LoweringFailure.Operation();
        }

        ExpressionResult operand = LowerExpression(syntax.Expression, state);
        SubsetValueType source = ExactType(operation.Operand.Type);
        SubsetValueType destination = ExactType(operation.Type);
        if (operand.Value.Type != source)
        {
            throw LoweringFailure.Operation();
        }

        if (source == destination)
        {
            if (!operation.Conversion.IsIdentity || operation.IsChecked)
            {
                throw LoweringFailure.Operation();
            }

            return operand;
        }

        if (!operation.Conversion.IsNumeric
            || operation.IsChecked
            || SubsetBodies.ContextFor(syntax) != ExplicitOverflowContext.Unchecked)
        {
            throw LoweringFailure.Operation();
        }

        features.Add(LoweredFeature.Conversion);
        ValueBuilder converted = EmitConvert(
            operand.Flow.Block,
            destination,
            LoweredConversionForm.ExplicitUnchecked,
            operand.Value,
            Origin(syntax));
        return new ExpressionResult(operand.Flow, converted);
    }

    private ExpressionResult ApplyContextConversion(
        IOperation operation,
        ExpressionResult value,
        LoweredOrigin origin)
    {
        if (operation is not IConversionOperation conversion)
        {
            if (operation.Type is null || value.Value.Type != ExactType(operation.Type))
            {
                throw LoweringFailure.Operation();
            }

            return value;
        }

        SubsetValueType source = ExactType(conversion.Operand.Type);
        SubsetValueType destination = ExactType(conversion.Type);
        if (!conversion.IsImplicit)
        {
            if (value.Value.Type != destination)
            {
                throw LoweringFailure.Operation();
            }

            return value;
        }

        if (source == destination)
        {
            if (!conversion.Conversion.IsIdentity || value.Value.Type != destination)
            {
                throw LoweringFailure.Operation();
            }

            return value;
        }

        bool accepted = (source == SubsetValueType.I32 && destination == SubsetValueType.I64)
            || (source == SubsetValueType.U32 && destination == SubsetValueType.I64)
            || (source == SubsetValueType.U32 && destination == SubsetValueType.U64);
        if (!accepted
            || !conversion.Conversion.IsNumeric
            || conversion.IsChecked
            || value.Value.Type != source)
        {
            throw LoweringFailure.Operation();
        }

        features.Add(LoweredFeature.Conversion);
        ValueBuilder converted = EmitConvert(
            value.Flow.Block,
            destination,
            LoweredConversionForm.Implicit,
            value.Value,
            origin);
        return new ExpressionResult(value.Flow, converted);
    }

    private ExpressionResult LowerConditional(ConditionalExpressionSyntax syntax, FlowState state)
    {
        if (Operation(syntax) is not IConditionalOperation operation
            || operation.Type is null
            || operation.WhenFalse is null)
        {
            throw LoweringFailure.Operation();
        }

        ExpressionResult condition = LowerExpression(syntax.Condition, state);
        SubsetValueType resultType = ExactType(operation.Type);
        features.Add(LoweredFeature.Branch);
        BlockBuilder whenFalse = NewBlock();
        BlockBuilder whenTrue = NewBlock();
        SetBranch(condition.Flow.Block, condition.Value, whenFalse, whenTrue, Origin(syntax.Condition));
        BlockBuilder join = NewBlock();
        ParameterBuilder parameter = join.AddParameter(resultType);

        ExpressionResult falseValue = LowerExpression(
            syntax.WhenFalse,
            new FlowState(
                whenFalse,
                new Dictionary<ILocalSymbol, ValueBuilder>(
                    condition.Flow.Environment,
                    SymbolEqualityComparer.Default)));
        ExpressionResult trueValue = LowerExpression(
            syntax.WhenTrue,
            new FlowState(
                whenTrue,
                new Dictionary<ILocalSymbol, ValueBuilder>(
                    condition.Flow.Environment,
                    SymbolEqualityComparer.Default)));
        if (falseValue.Value.Type != resultType || trueValue.Value.Type != resultType)
        {
            throw LoweringFailure.Operation();
        }

        SetJump(falseValue.Flow.Block, join, new[] { falseValue.Value }, Origin(syntax));
        SetJump(trueValue.Flow.Block, join, new[] { trueValue.Value }, Origin(syntax));
        return new ExpressionResult(
            new FlowState(
                join,
                new Dictionary<ILocalSymbol, ValueBuilder>(
                    condition.Flow.Environment,
                    SymbolEqualityComparer.Default)),
            ValueBuilder.Defined(parameter));
    }

    private FlowState? MergeFlows(
        FlowState? falseFlow,
        FlowState? trueFlow,
        IEnumerable<ILocalSymbol> incomingSymbols,
        LoweredOrigin origin)
    {
        if (falseFlow is null)
        {
            return trueFlow;
        }

        if (trueFlow is null)
        {
            return falseFlow;
        }

        BlockBuilder join = NewBlock();
        var falseArguments = new List<ValueBuilder>();
        var trueArguments = new List<ValueBuilder>();
        var merged = new Dictionary<ILocalSymbol, ValueBuilder>(SymbolEqualityComparer.Default);
        foreach (ILocalSymbol symbol in incomingSymbols.OrderBy(symbol => locals[symbol].Index))
        {
            if (!falseFlow.Environment.TryGetValue(symbol, out ValueBuilder? falseValue)
                || !trueFlow.Environment.TryGetValue(symbol, out ValueBuilder? trueValue))
            {
                throw LoweringFailure.ControlFlow();
            }

            if (falseValue.SameAs(trueValue))
            {
                merged.Add(symbol, falseValue);
                continue;
            }

            ParameterBuilder parameter = join.AddParameter(locals[symbol].Type);
            falseArguments.Add(falseValue);
            trueArguments.Add(trueValue);
            merged.Add(symbol, ValueBuilder.Defined(parameter));
        }

        SetJump(falseFlow.Block, join, falseArguments.ToArray(), origin);
        SetJump(trueFlow.Block, join, trueArguments.ToArray(), origin);
        return new FlowState(join, merged);
    }

    private ValueBuilder Reference(
        IdentifierNameSyntax syntax,
        IReadOnlyDictionary<ILocalSymbol, ValueBuilder> environment)
    {
        IOperation operation = Operation(syntax);
        if (operation is IParameterReferenceOperation parameter
            && parameters.TryGetValue(parameter.Parameter, out ValueBuilder? argument))
        {
            return argument;
        }

        if (operation is ILocalReferenceOperation local
            && environment.TryGetValue(local.Local, out ValueBuilder? value))
        {
            return value;
        }

        throw LoweringFailure.Operation();
    }

    private ValueBuilder Literal(LiteralExpressionSyntax syntax)
    {
        return Literal(Operation(syntax));
    }

    private ValueBuilder Literal(IOperation operation)
    {
        if (!operation.ConstantValue.HasValue || operation.Type is null)
        {
            throw LoweringFailure.Operation();
        }

        SubsetValueType type = ExactType(operation.Type);
        object? value = operation.ConstantValue.Value;
        if (type == SubsetValueType.Bool && value is bool boolean)
        {
            return ValueBuilder.Boolean(boolean);
        }

        string? decimalValue = value switch
        {
            int signed32 => signed32.ToString(CultureInfo.InvariantCulture),
            uint unsigned32 => unsigned32.ToString(CultureInfo.InvariantCulture),
            long signed64 => signed64.ToString(CultureInfo.InvariantCulture),
            ulong unsigned64 => unsigned64.ToString(CultureInfo.InvariantCulture),
            _ => null,
        };
        if (decimalValue is null || !SubsetTypeRules.IsInteger(type))
        {
            throw LoweringFailure.Operation();
        }

        return ValueBuilder.Integer(decimalValue, type);
    }

    private ValueBuilder EmitConst(
        BlockBuilder block,
        ValueBuilder literal,
        LoweredOrigin origin)
    {
        var instruction = new InstructionBuilder(
            LoweredInstructionKind.Const,
            literal.Type,
            null,
            default,
            default,
            LoweredConversionForm.None,
            ExplicitOverflowContext.None,
            shiftCountMask: false,
            new[] { literal },
            Array.Empty<LoweredSafetyCheck>(),
            origin);
        block.AddInstruction(instruction);
        return ValueBuilder.Defined(instruction);
    }

    private ValueBuilder EmitCopy(
        BlockBuilder block,
        LocalBuilder target,
        ValueBuilder value,
        LoweredOrigin origin)
    {
        var instruction = new InstructionBuilder(
            LoweredInstructionKind.Copy,
            target.Type,
            target.Id,
            default,
            default,
            LoweredConversionForm.None,
            ExplicitOverflowContext.None,
            shiftCountMask: false,
            new[] { value },
            Array.Empty<LoweredSafetyCheck>(),
            origin);
        block.AddInstruction(instruction);
        return ValueBuilder.Defined(instruction);
    }

    private ValueBuilder EmitUnary(
        BlockBuilder block,
        LoweredUnaryOperator operation,
        SubsetValueType type,
        ExplicitOverflowContext context,
        ValueBuilder value,
        LoweredSafetyCheck[] checks,
        LoweredOrigin origin)
    {
        var instruction = new InstructionBuilder(
            LoweredInstructionKind.Unary,
            type,
            null,
            operation,
            default,
            LoweredConversionForm.None,
            context,
            shiftCountMask: false,
            new[] { value },
            checks,
            origin);
        block.AddInstruction(instruction);
        return ValueBuilder.Defined(instruction);
    }

    private ValueBuilder EmitBinary(
        BlockBuilder block,
        LoweredBinaryOperator operation,
        SubsetValueType type,
        ExplicitOverflowContext context,
        bool shiftCountMask,
        ValueBuilder left,
        ValueBuilder right,
        LoweredSafetyCheck[] checks,
        LoweredOrigin origin)
    {
        var instruction = new InstructionBuilder(
            LoweredInstructionKind.Binary,
            type,
            null,
            default,
            operation,
            LoweredConversionForm.None,
            context,
            shiftCountMask,
            new[] { left, right },
            checks,
            origin);
        block.AddInstruction(instruction);
        return ValueBuilder.Defined(instruction);
    }

    private ValueBuilder EmitConvert(
        BlockBuilder block,
        SubsetValueType type,
        LoweredConversionForm form,
        ValueBuilder value,
        LoweredOrigin origin)
    {
        var instruction = new InstructionBuilder(
            LoweredInstructionKind.Convert,
            type,
            null,
            default,
            default,
            form,
            ExplicitOverflowContext.None,
            shiftCountMask: false,
            new[] { value },
            Array.Empty<LoweredSafetyCheck>(),
            origin);
        block.AddInstruction(instruction);
        return ValueBuilder.Defined(instruction);
    }

    private BlockBuilder NewBlock()
    {
        var block = new BlockBuilder();
        blocks.Add(block);
        return block;
    }

    private static void SetReturn(
        BlockBuilder block,
        ValueBuilder value,
        LoweredOrigin origin)
    {
        block.SetTerminator(TerminatorBuilder.Return(value, origin));
    }

    private static void SetJump(
        BlockBuilder block,
        BlockBuilder target,
        ValueBuilder[] arguments,
        LoweredOrigin origin)
    {
        block.SetTerminator(TerminatorBuilder.Jump(target, arguments, origin));
    }

    private static void SetBranch(
        BlockBuilder block,
        ValueBuilder condition,
        BlockBuilder whenFalse,
        BlockBuilder whenTrue,
        LoweredOrigin origin)
    {
        block.SetTerminator(TerminatorBuilder.Branch(
            condition,
            whenFalse,
            whenTrue,
            origin));
    }

    private LoweredFunction Freeze()
    {
        IReadOnlyList<BlockBuilder> canonicalBlocks = CanonicalBlockOrder();
        for (int index = 0; index < canonicalBlocks.Count; index++)
        {
            canonicalBlocks[index].ResolvedLabel = "bb" + index.ToString(CultureInfo.InvariantCulture);
        }

        int parameterIndex = 0;
        foreach (BlockBuilder block in canonicalBlocks)
        {
            foreach (ParameterBuilder parameter in block.Parameters)
            {
                parameter.ResolvedId = "p" + parameterIndex.ToString(CultureInfo.InvariantCulture);
                parameterIndex++;
            }
        }

        int temporaryIndex = 0;
        foreach (BlockBuilder block in canonicalBlocks)
        {
            foreach (InstructionBuilder instruction in block.Instructions)
            {
                instruction.ResolvedId = "t" + temporaryIndex.ToString(CultureInfo.InvariantCulture);
                temporaryIndex++;
            }
        }

        var loweredBlocks = canonicalBlocks.Select(block => block.Freeze()).ToArray();
        var loweredParameters = method.Symbol.Parameters
            .Select(parameter => new LoweredBinding(
                "arg" + parameter.Ordinal.ToString(CultureInfo.InvariantCulture),
                ExactType(parameter.Type)))
            .ToArray();
        var loweredResults = new[]
        {
            new LoweredBinding("result0", ExactType(method.Symbol.ReturnType)),
        };
        var loweredLocals = orderedLocals
            .Select(local => new LoweredBinding(local.Id, local.Type))
            .ToArray();
        LoweredRequiredCheck[] checks = LoweringValidator.CanonicalRequiredChecks(loweredBlocks);
        LoweredFeature[] loweredFeatures = features.OrderBy(feature => feature).ToArray();
        return new LoweredFunction(
            method.CanonicalId,
            loweredParameters,
            loweredResults,
            loweredLocals,
            loweredBlocks,
            checks,
            loweredFeatures);
    }

    private IReadOnlyList<BlockBuilder> CanonicalBlockOrder()
    {
        if (blocks.Count == 0)
        {
            throw LoweringFailure.ControlFlow();
        }

        var ordered = new List<BlockBuilder>(blocks.Count);
        var seen = new HashSet<BlockBuilder>();
        var queue = new Queue<BlockBuilder>();
        queue.Enqueue(blocks[0]);
        seen.Add(blocks[0]);
        while (queue.Count != 0)
        {
            BlockBuilder block = queue.Dequeue();
            ordered.Add(block);
            TerminatorBuilder terminator = block.Terminator
                ?? throw LoweringFailure.ControlFlow();
            foreach (BlockBuilder successor in terminator.SuccessorsFalseThenTrue())
            {
                if (seen.Add(successor))
                {
                    queue.Enqueue(successor);
                }
            }
        }

        if (ordered.Count != blocks.Count)
        {
            throw LoweringFailure.ControlFlow();
        }

        return ordered;
    }

    private IOperation Operation(SyntaxNode syntax)
    {
        return RoslynPublicApi.GetOperation(method.SemanticModel, syntax, "lowering")
            ?? throw FrontendFailure.Toolchain("lowering", "CSHARP_TOOLCHAIN_ADAPTER");
    }

    private SubsetValueType ExactType(ITypeSymbol? symbol)
    {
        try
        {
            return SubsetTypeRules.ValidateSymbol(
                symbol,
                method.SemanticModel.Compilation,
                "lowering");
        }
        catch (FrontendFailure)
        {
            throw LoweringFailure.Operation();
        }
    }

    private static ExplicitOverflowContext ArithmeticContext(
        IBinaryOperation operation,
        BinaryExpressionSyntax syntax)
    {
        ExplicitOverflowContext context = SubsetBodies.ContextFor(syntax);
        if (context == ExplicitOverflowContext.None)
        {
            throw LoweringFailure.Operation();
        }

        bool expected = context == ExplicitOverflowContext.Checked;
        if (operation.OperatorKind == BinaryOperatorKind.Remainder)
        {
            expected = false;
        }

        if (operation.IsChecked != expected)
        {
            throw FrontendFailure.Toolchain("lowering", "CSHARP_TOOLCHAIN_ADAPTER");
        }

        return context;
    }

    private static bool SignedComparison(SubsetValueType type)
    {
        if (!SubsetTypeRules.IsInteger(type))
        {
            throw LoweringFailure.Operation();
        }

        return IsSigned(type);
    }

    internal static int Width(SubsetValueType type)
    {
        return type switch
        {
            SubsetValueType.I32 or SubsetValueType.U32 => 32,
            SubsetValueType.I64 or SubsetValueType.U64 => 64,
            _ => throw LoweringFailure.Operation(),
        };
    }

    internal static bool IsSigned(SubsetValueType type)
    {
        return type switch
        {
            SubsetValueType.I32 or SubsetValueType.I64 => true,
            SubsetValueType.U32 or SubsetValueType.U64 => false,
            _ => throw LoweringFailure.Operation(),
        };
    }

    private static LoweredSafetyCheck SafetyCheck(
        LoweredSafetyCheckKind kind,
        LoweredCheckOperation operation,
        SubsetValueType type)
    {
        return new LoweredSafetyCheck(kind, operation, Width(type), IsSigned(type));
    }

    private static LoweredOrigin Origin(SyntaxNode syntax)
    {
        if (syntax.SyntaxTree is null || syntax.Span.Length <= 0)
        {
            throw LoweringFailure.Operation();
        }

        string path = syntax.SyntaxTree.FilePath;
        if (string.IsNullOrEmpty(path))
        {
            throw LoweringFailure.Operation();
        }

        return new LoweredOrigin(path, syntax.Span.Start, syntax.Span.End);
    }

    private sealed class LocalBuilder
    {
        internal LocalBuilder(
            ILocalSymbol symbol,
            string id,
            SubsetValueType type,
            int index)
        {
            Symbol = symbol;
            Id = id;
            Type = type;
            Index = index;
        }

        internal ILocalSymbol Symbol { get; }

        internal string Id { get; }

        internal SubsetValueType Type { get; }

        internal int Index { get; }
    }

    private sealed class FlowState
    {
        internal FlowState(
            BlockBuilder block,
            Dictionary<ILocalSymbol, ValueBuilder> environment)
        {
            Block = block;
            Environment = environment;
        }

        internal BlockBuilder Block { get; }

        internal Dictionary<ILocalSymbol, ValueBuilder> Environment { get; }
    }

    private sealed class ExpressionResult
    {
        internal ExpressionResult(FlowState flow, ValueBuilder value)
        {
            Flow = flow;
            Value = value;
        }

        internal FlowState Flow { get; }

        internal ValueBuilder Value { get; }
    }

    private interface IValueDefinition
    {
        string? ResolvedId { get; set; }

        SubsetValueType Type { get; }
    }

    private enum ValueBuilderKind
    {
        Defined,
        Named,
        Boolean,
        Integer,
    }

    private sealed class ValueBuilder
    {
        private ValueBuilder(
            ValueBuilderKind kind,
            SubsetValueType type,
            IValueDefinition? definition,
            string? text,
            bool boolean)
        {
            Kind = kind;
            Type = type;
            Definition = definition;
            Text = text;
            BooleanValue = boolean;
        }

        internal ValueBuilderKind Kind { get; }

        internal SubsetValueType Type { get; }

        internal IValueDefinition? Definition { get; }

        internal string? Text { get; }

        internal bool BooleanValue { get; }

        internal static ValueBuilder Defined(IValueDefinition definition)
        {
            return new ValueBuilder(
                ValueBuilderKind.Defined,
                definition.Type,
                definition,
                null,
                false);
        }

        internal static ValueBuilder Named(string id, SubsetValueType type)
        {
            return new ValueBuilder(ValueBuilderKind.Named, type, null, id, false);
        }

        internal static ValueBuilder Boolean(bool value)
        {
            return new ValueBuilder(
                ValueBuilderKind.Boolean,
                SubsetValueType.Bool,
                null,
                null,
                value);
        }

        internal static ValueBuilder Integer(string value, SubsetValueType type)
        {
            return new ValueBuilder(ValueBuilderKind.Integer, type, null, value, false);
        }

        internal bool SameAs(ValueBuilder other)
        {
            return Kind == other.Kind
                && Type == other.Type
                && ReferenceEquals(Definition, other.Definition)
                && string.Equals(Text, other.Text, StringComparison.Ordinal)
                && BooleanValue == other.BooleanValue;
        }

        internal LoweredValue Freeze()
        {
            return Kind switch
            {
                ValueBuilderKind.Defined => LoweredValue.Variable(
                    Definition?.ResolvedId ?? throw LoweringFailure.ControlFlow(),
                    Type),
                ValueBuilderKind.Named => LoweredValue.Variable(
                    Text ?? throw LoweringFailure.Operation(),
                    Type),
                ValueBuilderKind.Boolean => LoweredValue.BooleanLiteral(BooleanValue),
                ValueBuilderKind.Integer => LoweredValue.IntegerLiteral(
                    Text ?? throw LoweringFailure.Operation(),
                    Type),
                _ => throw LoweringFailure.Operation(),
            };
        }
    }

    private sealed class ParameterBuilder : IValueDefinition
    {
        internal ParameterBuilder(SubsetValueType type)
        {
            Type = type;
        }

        public string? ResolvedId { get; set; }

        public SubsetValueType Type { get; }

        internal LoweredBinding Freeze()
        {
            return new LoweredBinding(
                ResolvedId ?? throw LoweringFailure.ControlFlow(),
                Type);
        }
    }

    private sealed class InstructionBuilder : IValueDefinition
    {
        private readonly ValueBuilder[] operands;
        private readonly LoweredSafetyCheck[] safetyChecks;

        internal InstructionBuilder(
            LoweredInstructionKind kind,
            SubsetValueType type,
            string? target,
            LoweredUnaryOperator unaryOperator,
            LoweredBinaryOperator binaryOperator,
            LoweredConversionForm conversionForm,
            ExplicitOverflowContext overflowContext,
            bool shiftCountMask,
            ValueBuilder[] operands,
            LoweredSafetyCheck[] safetyChecks,
            LoweredOrigin origin)
        {
            Kind = kind;
            Type = type;
            Target = target;
            UnaryOperator = unaryOperator;
            BinaryOperator = binaryOperator;
            ConversionForm = conversionForm;
            OverflowContext = overflowContext;
            IsShiftCountMask = shiftCountMask;
            this.operands = (ValueBuilder[])operands.Clone();
            this.safetyChecks = (LoweredSafetyCheck[])safetyChecks.Clone();
            Origin = origin;
        }

        public string? ResolvedId { get; set; }

        public SubsetValueType Type { get; }

        internal LoweredInstructionKind Kind { get; }

        internal string? Target { get; }

        internal LoweredUnaryOperator UnaryOperator { get; }

        internal LoweredBinaryOperator BinaryOperator { get; }

        internal LoweredConversionForm ConversionForm { get; }

        internal ExplicitOverflowContext OverflowContext { get; }

        internal bool IsShiftCountMask { get; }

        internal LoweredOrigin Origin { get; }

        internal LoweredInstruction Freeze()
        {
            return new LoweredInstruction(
                ResolvedId ?? throw LoweringFailure.ControlFlow(),
                Kind,
                Type,
                Target,
                UnaryOperator,
                BinaryOperator,
                ConversionForm,
                OverflowContext,
                IsShiftCountMask,
                operands.Select(operand => operand.Freeze()).ToArray(),
                safetyChecks,
                Origin);
        }
    }

    private sealed class BlockBuilder
    {
        private readonly List<ParameterBuilder> parameters = new List<ParameterBuilder>();
        private readonly List<InstructionBuilder> instructions = new List<InstructionBuilder>();

        internal string? ResolvedLabel { get; set; }

        internal IReadOnlyList<ParameterBuilder> Parameters => parameters;

        internal IReadOnlyList<InstructionBuilder> Instructions => instructions;

        internal TerminatorBuilder? Terminator { get; private set; }

        internal ParameterBuilder AddParameter(SubsetValueType type)
        {
            var parameter = new ParameterBuilder(type);
            parameters.Add(parameter);
            return parameter;
        }

        internal void AddInstruction(InstructionBuilder instruction)
        {
            if (Terminator is not null)
            {
                throw LoweringFailure.ControlFlow();
            }

            instructions.Add(instruction);
        }

        internal void SetTerminator(TerminatorBuilder terminator)
        {
            if (Terminator is not null)
            {
                throw LoweringFailure.ControlFlow();
            }

            Terminator = terminator;
        }

        internal LoweredBlock Freeze()
        {
            return new LoweredBlock(
                ResolvedLabel ?? throw LoweringFailure.ControlFlow(),
                parameters.Select(parameter => parameter.Freeze()).ToArray(),
                instructions.Select(instruction => instruction.Freeze()).ToArray(),
                (Terminator ?? throw LoweringFailure.ControlFlow()).Freeze());
        }
    }

    private sealed class TerminatorBuilder
    {
        private readonly ValueBuilder[] falseArguments;
        private readonly ValueBuilder[] trueArguments;
        private readonly ValueBuilder[] values;

        private TerminatorBuilder(
            LoweredTerminatorKind kind,
            ValueBuilder? condition,
            BlockBuilder? falseTarget,
            ValueBuilder[] falseArguments,
            BlockBuilder? trueTarget,
            ValueBuilder[] trueArguments,
            ValueBuilder[] values,
            LoweredOrigin origin)
        {
            Kind = kind;
            Condition = condition;
            FalseTarget = falseTarget;
            this.falseArguments = (ValueBuilder[])falseArguments.Clone();
            TrueTarget = trueTarget;
            this.trueArguments = (ValueBuilder[])trueArguments.Clone();
            this.values = (ValueBuilder[])values.Clone();
            Origin = origin;
        }

        internal LoweredTerminatorKind Kind { get; }

        internal ValueBuilder? Condition { get; }

        internal BlockBuilder? FalseTarget { get; }

        internal BlockBuilder? TrueTarget { get; }

        internal LoweredOrigin Origin { get; }

        internal static TerminatorBuilder Return(ValueBuilder value, LoweredOrigin origin)
        {
            return new TerminatorBuilder(
                LoweredTerminatorKind.Return,
                null,
                null,
                Array.Empty<ValueBuilder>(),
                null,
                Array.Empty<ValueBuilder>(),
                new[] { value },
                origin);
        }

        internal static TerminatorBuilder Jump(
            BlockBuilder target,
            ValueBuilder[] arguments,
            LoweredOrigin origin)
        {
            return new TerminatorBuilder(
                LoweredTerminatorKind.Jump,
                null,
                target,
                arguments,
                null,
                Array.Empty<ValueBuilder>(),
                Array.Empty<ValueBuilder>(),
                origin);
        }

        internal static TerminatorBuilder Branch(
            ValueBuilder condition,
            BlockBuilder whenFalse,
            BlockBuilder whenTrue,
            LoweredOrigin origin)
        {
            return new TerminatorBuilder(
                LoweredTerminatorKind.Branch,
                condition,
                whenFalse,
                Array.Empty<ValueBuilder>(),
                whenTrue,
                Array.Empty<ValueBuilder>(),
                Array.Empty<ValueBuilder>(),
                origin);
        }

        internal IEnumerable<BlockBuilder> SuccessorsFalseThenTrue()
        {
            if (FalseTarget is not null)
            {
                yield return FalseTarget;
            }

            if (TrueTarget is not null)
            {
                yield return TrueTarget;
            }
        }

        internal LoweredTerminator Freeze()
        {
            return new LoweredTerminator(
                Kind,
                Condition?.Freeze(),
                FalseTarget?.ResolvedLabel,
                falseArguments.Select(argument => argument.Freeze()).ToArray(),
                TrueTarget?.ResolvedLabel,
                trueArguments.Select(argument => argument.Freeze()).ToArray(),
                values.Select(value => value.Freeze()).ToArray(),
                Origin);
        }
    }
}
