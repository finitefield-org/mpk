using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.FlowAnalysis;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

internal static class LoweringCfgAdapter
{
    internal static void Validate(SubsetMethod method)
    {
        ControlFlowGraph graph = method.ControlFlowGraph;
        if (!ReferenceEquals(graph.OriginalOperation, method.Body)
            || graph.Blocks.Length != method.CfgBlockCount
            || graph.Blocks.Length < 2)
        {
            throw LoweringFailure.ControlFlow();
        }

        int expectedConditions = method.Declaration.Body!
            .DescendantNodes(descendIntoTrivia: false)
            .Count(node => node is IfStatementSyntax
                || node is ConditionalExpressionSyntax
                || (node is BinaryExpressionSyntax binary
                    && (binary.IsKind(SyntaxKind.LogicalAndExpression)
                        || binary.IsKind(SyntaxKind.LogicalOrExpression))));
        int actualConditions = 0;
        var graphOperations = new HashSet<IOperation>(ReferenceOperationComparer.Instance);
        foreach (BasicBlock block in graph.Blocks)
        {
            foreach (IOperation operation in block.Operations)
            {
                Collect(operation, graphOperations);
            }

            if (block.BranchValue is not null)
            {
                Collect(block.BranchValue, graphOperations);
            }

            if (block.ConditionKind != ControlFlowConditionKind.None)
            {
                actualConditions++;
                if (block.BranchValue is null
                    || ExactType(block.BranchValue.Type, method) != SubsetValueType.Bool
                    || block.ConditionalSuccessor?.Destination is null
                    || block.FallThroughSuccessor?.Destination is null
                    || !HasSourceCondition(block.BranchValue.Syntax))
                {
                    throw LoweringFailure.ControlFlow();
                }
            }
        }

        if (actualConditions != expectedConditions)
        {
            throw LoweringFailure.ControlFlow();
        }

        foreach (IOperation operation in graphOperations)
        {
            if ((operation is IFlowCaptureOperation || operation is IFlowCaptureReferenceOperation)
                && !HasFlowCaptureOwner(operation.Syntax))
            {
                throw LoweringFailure.ControlFlow();
            }
        }

        ValidateAcyclic(graph);
        ValidateBreadthFirstReachability(graph);
    }

    private static void Collect(IOperation operation, HashSet<IOperation> operations)
    {
        if (!operations.Add(operation))
        {
            return;
        }

        foreach (IOperation child in operation.ChildOperations)
        {
            Collect(child, operations);
        }
    }

    private static bool HasSourceCondition(SyntaxNode syntax)
    {
        foreach (SyntaxNode ancestor in syntax.AncestorsAndSelf())
        {
            if (ancestor is IfStatementSyntax conditional
                && conditional.Condition.Span.Contains(syntax.Span))
            {
                return true;
            }

            if (ancestor is ConditionalExpressionSyntax expression
                && expression.Condition.Span.Contains(syntax.Span))
            {
                return true;
            }

            if (ancestor is BinaryExpressionSyntax binary
                && (binary.IsKind(SyntaxKind.LogicalAndExpression)
                    || binary.IsKind(SyntaxKind.LogicalOrExpression)))
            {
                return true;
            }
        }

        return false;
    }

    private static bool HasFlowCaptureOwner(SyntaxNode syntax)
    {
        return syntax.AncestorsAndSelf().Any(ancestor =>
            ancestor is ConditionalExpressionSyntax
            || ancestor is InvocationExpressionSyntax
            || (ancestor is BinaryExpressionSyntax binary
                && (binary.IsKind(SyntaxKind.LogicalAndExpression)
                    || binary.IsKind(SyntaxKind.LogicalOrExpression))));
    }

    private static void ValidateAcyclic(ControlFlowGraph graph)
    {
        var states = new byte[graph.Blocks.Length];
        Visit(graph.Blocks[0], states);
        if (states.Any(state => state != 2))
        {
            throw LoweringFailure.ControlFlow();
        }
    }

    private static void Visit(BasicBlock block, byte[] states)
    {
        int index = block.Ordinal;
        if (index < 0 || index >= states.Length)
        {
            throw LoweringFailure.ControlFlow();
        }

        if (states[index] == 1)
        {
            throw LoweringFailure.ControlFlow();
        }

        if (states[index] == 2)
        {
            return;
        }

        states[index] = 1;
        foreach (BasicBlock successor in SuccessorsFalseThenTrue(block))
        {
            Visit(successor, states);
        }

        states[index] = 2;
    }

    private static void ValidateBreadthFirstReachability(ControlFlowGraph graph)
    {
        var queue = new Queue<BasicBlock>();
        var seen = new HashSet<BasicBlock>();
        queue.Enqueue(graph.Blocks[0]);
        seen.Add(graph.Blocks[0]);
        while (queue.Count != 0)
        {
            BasicBlock block = queue.Dequeue();
            foreach (BasicBlock successor in SuccessorsFalseThenTrue(block))
            {
                if (seen.Add(successor))
                {
                    queue.Enqueue(successor);
                }
            }
        }

        if (seen.Count != graph.Blocks.Length)
        {
            throw LoweringFailure.ControlFlow();
        }
    }

    private static IEnumerable<BasicBlock> SuccessorsFalseThenTrue(BasicBlock block)
    {
        if (block.ConditionKind == ControlFlowConditionKind.None)
        {
            if (block.FallThroughSuccessor?.Destination is BasicBlock only)
            {
                yield return only;
            }

            yield break;
        }

        BasicBlock conditional = block.ConditionalSuccessor?.Destination
            ?? throw LoweringFailure.ControlFlow();
        BasicBlock fallthrough = block.FallThroughSuccessor?.Destination
            ?? throw LoweringFailure.ControlFlow();
        if (block.ConditionKind == ControlFlowConditionKind.WhenTrue)
        {
            yield return fallthrough;
            yield return conditional;
        }
        else if (block.ConditionKind == ControlFlowConditionKind.WhenFalse)
        {
            yield return conditional;
            yield return fallthrough;
        }
        else
        {
            throw LoweringFailure.ControlFlow();
        }
    }

    private static SubsetValueType ExactType(ITypeSymbol? symbol, SubsetMethod method)
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
}

internal static class LoweringValidator
{
    internal static void Validate(LoweredClosure closure)
    {
        if (!IsLowercaseSha256(closure.SelectionSha256)
            || closure.Functions.Count == 0)
        {
            throw LoweringFailure.ControlFlow();
        }

        var ids = new HashSet<string>(StringComparer.Ordinal);
        foreach (LoweredFunction function in closure.Functions)
        {
            if (!ids.Add(function.Id))
            {
                throw LoweringFailure.ControlFlow();
            }

            Validate(function);
        }

        ValidateCallClosure(closure);
    }

    internal static void Validate(LoweredFunction function)
    {
        CanonicalMethodId methodId;
        try
        {
            methodId = SelectionCodec.ParseMethodId(function.Id);
        }
        catch (SelectionSyntaxFailure)
        {
            throw LoweringFailure.Operation();
        }

        if (!string.Equals(methodId.Method, function.Name, StringComparison.Ordinal)
            || !IsLowercaseSha256(function.ContractHash))
        {
            throw LoweringFailure.Operation();
        }

        ValidateOrigin(function.Origin);
        var types = new Dictionary<string, SubsetValueType>(StringComparer.Ordinal);
        ValidateBindings(function.Parameters, "arg", allowEmpty: true, types);
        ValidateBindings(function.Results, "result", allowEmpty: false, types);
        ValidateBindings(function.Locals, "local", allowEmpty: true, types);
        if (function.Results.Count != 1
            || !string.Equals(function.Results[0].Id, "result0", StringComparison.Ordinal)
            || function.Blocks.Count == 0)
        {
            throw LoweringFailure.ControlFlow();
        }

        if (methodId.ParameterTypes.Count != function.Parameters.Count
            || TypeFromToken(methodId.ResultType) != function.Results[0].Type)
        {
            throw LoweringFailure.Operation();
        }

        for (int index = 0; index < function.Parameters.Count; index++)
        {
            if (TypeFromToken(methodId.ParameterTypes[index]) != function.Parameters[index].Type)
            {
                throw LoweringFailure.Operation();
            }
        }

        int parameterIndex = 0;
        int instructionIndex = 0;
        var instructionById = new Dictionary<string, LoweredInstruction>(StringComparer.Ordinal);
        for (int blockIndex = 0; blockIndex < function.Blocks.Count; blockIndex++)
        {
            LoweredBlock block = function.Blocks[blockIndex];
            if (!string.Equals(
                block.Label,
                "bb" + blockIndex.ToString(CultureInfo.InvariantCulture),
                StringComparison.Ordinal))
            {
                throw LoweringFailure.ControlFlow();
            }

            foreach (LoweredBinding parameter in block.Parameters)
            {
                string expected = "p" + parameterIndex.ToString(CultureInfo.InvariantCulture);
                if (!string.Equals(parameter.Id, expected, StringComparison.Ordinal)
                    || !types.TryAdd(parameter.Id, parameter.Type))
                {
                    throw LoweringFailure.ControlFlow();
                }

                parameterIndex++;
            }

            foreach (LoweredInstruction instruction in block.Instructions)
            {
                string expected = "t" + instructionIndex.ToString(CultureInfo.InvariantCulture);
                if (!string.Equals(instruction.Id, expected, StringComparison.Ordinal)
                    || !types.TryAdd(instruction.Id, instruction.Type)
                    || !instructionById.TryAdd(instruction.Id, instruction))
                {
                    throw LoweringFailure.Operation();
                }

                instructionIndex++;
            }
        }

        // Operation shape wins over required-check diagnostics.
        foreach (LoweredBlock block in function.Blocks)
        {
            foreach (LoweredInstruction instruction in block.Instructions)
            {
                ValidateOrigin(instruction.Origin);
                ValidateInstruction(instruction, instructionById, types);
            }
        }

        foreach (LoweredBlock block in function.Blocks)
        {
            foreach (LoweredInstruction instruction in block.Instructions)
            {
                ValidateInstructionChecks(instruction);
            }
        }

        ValidateRequiredCheckLedger(function);
        ValidateGraph(function, types);
        ValidateFeatures(function);
    }

    internal static LoweredRequiredCheck[] CanonicalRequiredChecks(
        IReadOnlyList<LoweredBlock> blocks)
    {
        var checks = new List<LoweredRequiredCheck>();
        foreach (LoweredBlock block in blocks)
        {
            foreach (LoweredInstruction instruction in block.Instructions)
            {
                foreach (LoweredSafetyCheck check in instruction.SafetyChecks)
                {
                    checks.Add(new LoweredRequiredCheck(instruction.Id, check));
                }
            }
        }

        checks.Sort(CompareRequiredChecks);
        return checks.ToArray();
    }

    internal static string ProfileOperation(LoweredInstruction instruction)
    {
        return instruction.Kind switch
        {
            LoweredInstructionKind.Const => "Const",
            LoweredInstructionKind.Copy => "Copy",
            LoweredInstructionKind.Convert => "Convert",
            LoweredInstructionKind.CallStatic => "CallStatic",
            LoweredInstructionKind.Unary => instruction.UnaryOperator switch
            {
                LoweredUnaryOperator.BoolNot => "bool_not",
                LoweredUnaryOperator.BvNeg => "bv_neg",
                LoweredUnaryOperator.BvNot => "bv_not",
                _ => throw LoweringFailure.Operation(),
            },
            LoweredInstructionKind.Binary when instruction.IsShiftCountMask =>
                "bv_and(count," + (instruction.Operands[1].Text
                    ?? throw LoweringFailure.Operation()) + ")",
            LoweredInstructionKind.Binary => instruction.BinaryOperator switch
            {
                LoweredBinaryOperator.Eq => "eq",
                LoweredBinaryOperator.NotEq => "not_eq",
                LoweredBinaryOperator.BvAdd => "bv_add",
                LoweredBinaryOperator.BvSub => "bv_sub",
                LoweredBinaryOperator.BvMul => "bv_mul",
                LoweredBinaryOperator.BvSdiv => "bv_sdiv",
                LoweredBinaryOperator.BvSrem => "bv_srem",
                LoweredBinaryOperator.BvUdiv => "bv_udiv",
                LoweredBinaryOperator.BvUrem => "bv_urem",
                LoweredBinaryOperator.BvAnd => "bv_and",
                LoweredBinaryOperator.BvOr => "bv_or",
                LoweredBinaryOperator.BvXor => "bv_xor",
                LoweredBinaryOperator.BvShl => "bv_shl",
                LoweredBinaryOperator.BvAshr => "bv_ashr",
                LoweredBinaryOperator.BvLshr => "bv_lshr",
                LoweredBinaryOperator.SignedLt => "signed_lt",
                LoweredBinaryOperator.SignedLe => "signed_le",
                LoweredBinaryOperator.SignedGt => "signed_gt",
                LoweredBinaryOperator.SignedGe => "signed_ge",
                LoweredBinaryOperator.UnsignedLt => "unsigned_lt",
                LoweredBinaryOperator.UnsignedLe => "unsigned_le",
                LoweredBinaryOperator.UnsignedGt => "unsigned_gt",
                LoweredBinaryOperator.UnsignedGe => "unsigned_ge",
                _ => throw LoweringFailure.Operation(),
            },
            _ => throw LoweringFailure.Operation(),
        };
    }

    private static void ValidateBindings(
        IReadOnlyList<LoweredBinding> bindings,
        string prefix,
        bool allowEmpty,
        Dictionary<string, SubsetValueType> types)
    {
        if (!allowEmpty && bindings.Count == 0)
        {
            throw LoweringFailure.ControlFlow();
        }

        for (int index = 0; index < bindings.Count; index++)
        {
            LoweredBinding binding = bindings[index];
            string expected = prefix + index.ToString(CultureInfo.InvariantCulture);
            if (!string.Equals(binding.Id, expected, StringComparison.Ordinal)
                || !types.TryAdd(binding.Id, binding.Type))
            {
                throw LoweringFailure.ControlFlow();
            }
        }
    }

    private static void ValidateInstruction(
        LoweredInstruction instruction,
        IReadOnlyDictionary<string, LoweredInstruction> instructionById,
        IReadOnlyDictionary<string, SubsetValueType> types)
    {
        foreach (LoweredValue operand in instruction.Operands)
        {
            ValidateValue(operand, types);
        }

        if (instruction.Kind != LoweredInstructionKind.CallStatic
            && (instruction.Function is not null || instruction.ContractHash is not null))
        {
            throw LoweringFailure.Operation();
        }

        switch (instruction.Kind)
        {
            case LoweredInstructionKind.Const:
                RequireInstructionDefaults(instruction);
                if (instruction.Operands.Count != 1
                    || (instruction.Operands[0].Kind != LoweredValueKind.Boolean
                        && instruction.Operands[0].Kind != LoweredValueKind.Integer)
                    || instruction.Operands[0].Type != instruction.Type)
                {
                    throw LoweringFailure.Operation();
                }

                return;
            case LoweredInstructionKind.Copy:
                if (instruction.Target is null
                    || instruction.Operands.Count != 1
                    || instruction.Operands[0].Type != instruction.Type
                    || !instruction.Target.StartsWith("local", StringComparison.Ordinal)
                    || !types.TryGetValue(instruction.Target, out SubsetValueType targetType)
                    || targetType != instruction.Type
                    || instruction.ConversionForm != LoweredConversionForm.None
                    || instruction.OverflowContext != ExplicitOverflowContext.None
                    || instruction.IsShiftCountMask)
                {
                    throw LoweringFailure.Operation();
                }

                return;
            case LoweredInstructionKind.Unary:
                ValidateUnary(instruction);
                return;
            case LoweredInstructionKind.Binary:
                ValidateBinary(instruction, instructionById);
                return;
            case LoweredInstructionKind.Convert:
                ValidateConversion(instruction);
                return;
            case LoweredInstructionKind.CallStatic:
                ValidateCall(instruction);
                return;
            default:
                throw LoweringFailure.Operation();
        }
    }

    private static void RequireInstructionDefaults(LoweredInstruction instruction)
    {
        if (instruction.Target is not null
            || instruction.ConversionForm != LoweredConversionForm.None
            || instruction.OverflowContext != ExplicitOverflowContext.None
            || instruction.IsShiftCountMask)
        {
            throw LoweringFailure.Operation();
        }
    }

    private static void ValidateCall(LoweredInstruction instruction)
    {
        if (instruction.Target is not null
            || instruction.Function is null
            || instruction.ContractHash is null
            || !IsLowercaseSha256(instruction.ContractHash)
            || instruction.ConversionForm != LoweredConversionForm.None
            || instruction.OverflowContext != ExplicitOverflowContext.None
            || instruction.IsShiftCountMask
            || instruction.SafetyChecks.Count != 0)
        {
            throw LoweringFailure.Operation();
        }

        CanonicalMethodId target;
        try
        {
            target = SelectionCodec.ParseMethodId(instruction.Function);
        }
        catch (SelectionSyntaxFailure)
        {
            throw LoweringFailure.Operation();
        }

        if (target.ParameterTypes.Count != instruction.Operands.Count
            || TypeFromToken(target.ResultType) != instruction.Type)
        {
            throw LoweringFailure.Operation();
        }

        for (int index = 0; index < instruction.Operands.Count; index++)
        {
            if (TypeFromToken(target.ParameterTypes[index]) != instruction.Operands[index].Type)
            {
                throw LoweringFailure.Operation();
            }
        }
    }

    private static void ValidateUnary(LoweredInstruction instruction)
    {
        if (instruction.Target is not null
            || instruction.Operands.Count != 1
            || instruction.ConversionForm != LoweredConversionForm.None
            || instruction.IsShiftCountMask)
        {
            throw LoweringFailure.Operation();
        }

        SubsetValueType operand = instruction.Operands[0].Type;
        switch (instruction.UnaryOperator)
        {
            case LoweredUnaryOperator.BoolNot:
                if (operand != SubsetValueType.Bool
                    || instruction.Type != SubsetValueType.Bool
                    || instruction.OverflowContext != ExplicitOverflowContext.None)
                {
                    throw LoweringFailure.Operation();
                }

                return;
            case LoweredUnaryOperator.BvNot:
                if (!SubsetTypeRules.IsInteger(operand)
                    || instruction.Type != operand
                    || instruction.OverflowContext != ExplicitOverflowContext.None)
                {
                    throw LoweringFailure.Operation();
                }

                return;
            case LoweredUnaryOperator.BvNeg:
                if ((operand != SubsetValueType.I32 && operand != SubsetValueType.I64)
                    || instruction.Type != operand
                    || (instruction.OverflowContext != ExplicitOverflowContext.Checked
                        && instruction.OverflowContext != ExplicitOverflowContext.Unchecked))
                {
                    throw LoweringFailure.Operation();
                }

                return;
            default:
                throw LoweringFailure.Operation();
        }
    }

    private static void ValidateBinary(
        LoweredInstruction instruction,
        IReadOnlyDictionary<string, LoweredInstruction> instructionById)
    {
        if (instruction.Target is not null
            || instruction.Operands.Count != 2
            || instruction.ConversionForm != LoweredConversionForm.None)
        {
            throw LoweringFailure.Operation();
        }

        SubsetValueType left = instruction.Operands[0].Type;
        SubsetValueType right = instruction.Operands[1].Type;
        if (instruction.IsShiftCountMask)
        {
            if (instruction.BinaryOperator != LoweredBinaryOperator.BvAnd
                || instruction.Type != SubsetValueType.I32
                || left != SubsetValueType.I32
                || right != SubsetValueType.I32
                || instruction.Operands[1].Kind != LoweredValueKind.Integer
                || (instruction.Operands[1].Text != "31" && instruction.Operands[1].Text != "63")
                || instruction.OverflowContext != ExplicitOverflowContext.None)
            {
                throw LoweringFailure.Operation();
            }

            return;
        }

        switch (instruction.BinaryOperator)
        {
            case LoweredBinaryOperator.Eq:
            case LoweredBinaryOperator.NotEq:
                RequireSameOperands(left, right);
                RequireResult(instruction, SubsetValueType.Bool, ExplicitOverflowContext.None);
                return;
            case LoweredBinaryOperator.SignedLt:
            case LoweredBinaryOperator.SignedLe:
            case LoweredBinaryOperator.SignedGt:
            case LoweredBinaryOperator.SignedGe:
                RequireSameOperands(left, right);
                if ((left != SubsetValueType.I32 && left != SubsetValueType.I64)
                    || instruction.Type != SubsetValueType.Bool
                    || instruction.OverflowContext != ExplicitOverflowContext.None)
                {
                    throw LoweringFailure.Operation();
                }

                return;
            case LoweredBinaryOperator.UnsignedLt:
            case LoweredBinaryOperator.UnsignedLe:
            case LoweredBinaryOperator.UnsignedGt:
            case LoweredBinaryOperator.UnsignedGe:
                RequireSameOperands(left, right);
                if ((left != SubsetValueType.U32 && left != SubsetValueType.U64)
                    || instruction.Type != SubsetValueType.Bool
                    || instruction.OverflowContext != ExplicitOverflowContext.None)
                {
                    throw LoweringFailure.Operation();
                }

                return;
            case LoweredBinaryOperator.BvAdd:
            case LoweredBinaryOperator.BvSub:
            case LoweredBinaryOperator.BvMul:
            case LoweredBinaryOperator.BvSdiv:
            case LoweredBinaryOperator.BvSrem:
            case LoweredBinaryOperator.BvUdiv:
            case LoweredBinaryOperator.BvUrem:
                RequireSameIntegerResult(instruction, left, right);
                if (instruction.OverflowContext != ExplicitOverflowContext.Checked
                    && instruction.OverflowContext != ExplicitOverflowContext.Unchecked)
                {
                    throw LoweringFailure.Operation();
                }

                bool signedOperation = instruction.BinaryOperator == LoweredBinaryOperator.BvSdiv
                    || instruction.BinaryOperator == LoweredBinaryOperator.BvSrem;
                bool unsignedOperation = instruction.BinaryOperator == LoweredBinaryOperator.BvUdiv
                    || instruction.BinaryOperator == LoweredBinaryOperator.BvUrem;
                if ((signedOperation && !LoweringMethodBuilder.IsSigned(left))
                    || (unsignedOperation && LoweringMethodBuilder.IsSigned(left)))
                {
                    throw LoweringFailure.Operation();
                }

                return;
            case LoweredBinaryOperator.BvAnd:
            case LoweredBinaryOperator.BvOr:
            case LoweredBinaryOperator.BvXor:
                RequireSameIntegerResult(instruction, left, right);
                if (instruction.OverflowContext != ExplicitOverflowContext.None)
                {
                    throw LoweringFailure.Operation();
                }

                return;
            case LoweredBinaryOperator.BvShl:
            case LoweredBinaryOperator.BvAshr:
            case LoweredBinaryOperator.BvLshr:
                string? maskId = instruction.Operands[1].Text;
                if (!SubsetTypeRules.IsInteger(left)
                    || right != SubsetValueType.I32
                    || instruction.Type != left
                    || instruction.OverflowContext != ExplicitOverflowContext.None
                    || instruction.Operands[1].Kind != LoweredValueKind.Variable
                    || maskId is null
                    || !instructionById.TryGetValue(
                        maskId,
                        out LoweredInstruction? mask)
                    || !mask.IsShiftCountMask
                    || mask.Operands[1].Text != (LoweringMethodBuilder.Width(left) - 1)
                        .ToString(CultureInfo.InvariantCulture)
                    || (instruction.BinaryOperator == LoweredBinaryOperator.BvAshr
                        && !LoweringMethodBuilder.IsSigned(left))
                    || (instruction.BinaryOperator == LoweredBinaryOperator.BvLshr
                        && LoweringMethodBuilder.IsSigned(left)))
                {
                    throw LoweringFailure.Operation();
                }

                return;
            default:
                throw LoweringFailure.Operation();
        }
    }

    private static void ValidateConversion(LoweredInstruction instruction)
    {
        if (instruction.Target is not null
            || instruction.Operands.Count != 1
            || instruction.Operands[0].Type == instruction.Type
            || !SubsetTypeRules.IsInteger(instruction.Operands[0].Type)
            || !SubsetTypeRules.IsInteger(instruction.Type)
            || instruction.OverflowContext != ExplicitOverflowContext.None
            || instruction.IsShiftCountMask)
        {
            throw LoweringFailure.Operation();
        }

        SubsetValueType source = instruction.Operands[0].Type;
        SubsetValueType destination = instruction.Type;
        if (instruction.ConversionForm == LoweredConversionForm.Implicit)
        {
            bool accepted = (source == SubsetValueType.I32 && destination == SubsetValueType.I64)
                || (source == SubsetValueType.U32 && destination == SubsetValueType.I64)
                || (source == SubsetValueType.U32 && destination == SubsetValueType.U64);
            if (!accepted)
            {
                throw LoweringFailure.Operation();
            }
        }
        else if (instruction.ConversionForm != LoweredConversionForm.ExplicitUnchecked)
        {
            throw LoweringFailure.Operation();
        }
    }

    private static void ValidateInstructionChecks(LoweredInstruction instruction)
    {
        LoweredSafetyCheck[] expected = ExpectedChecks(instruction);
        CompareCheckSequences(expected, instruction.SafetyChecks);
    }

    private static LoweredSafetyCheck[] ExpectedChecks(LoweredInstruction instruction)
    {
        if (instruction.Kind == LoweredInstructionKind.Unary
            && instruction.UnaryOperator == LoweredUnaryOperator.BvNeg
            && instruction.OverflowContext == ExplicitOverflowContext.Checked)
        {
            return new[]
            {
                Check(
                    LoweredSafetyCheckKind.IntegerNoOverflow,
                    LoweredCheckOperation.Neg,
                    instruction.Type),
            };
        }

        if (instruction.Kind != LoweredInstructionKind.Binary)
        {
            return Array.Empty<LoweredSafetyCheck>();
        }

        if (instruction.OverflowContext == ExplicitOverflowContext.Checked)
        {
            LoweredCheckOperation overflow = instruction.BinaryOperator switch
            {
                LoweredBinaryOperator.BvAdd => LoweredCheckOperation.Add,
                LoweredBinaryOperator.BvSub => LoweredCheckOperation.Sub,
                LoweredBinaryOperator.BvMul => LoweredCheckOperation.Mul,
                _ => LoweredCheckOperation.None,
            };
            if (overflow != LoweredCheckOperation.None)
            {
                return new[]
                {
                    Check(
                        LoweredSafetyCheckKind.IntegerNoOverflow,
                        overflow,
                        instruction.Type),
                };
            }
        }

        LoweredCheckOperation divrem = instruction.BinaryOperator switch
        {
            LoweredBinaryOperator.BvSdiv or LoweredBinaryOperator.BvUdiv =>
                LoweredCheckOperation.Div,
            LoweredBinaryOperator.BvSrem or LoweredBinaryOperator.BvUrem =>
                LoweredCheckOperation.Rem,
            _ => LoweredCheckOperation.None,
        };
        if (divrem == LoweredCheckOperation.None)
        {
            return Array.Empty<LoweredSafetyCheck>();
        }

        bool signed = instruction.BinaryOperator == LoweredBinaryOperator.BvSdiv
            || instruction.BinaryOperator == LoweredBinaryOperator.BvSrem;
        return signed
            ? new[]
            {
                Check(LoweredSafetyCheckKind.DivisorNonzero, divrem, instruction.Type),
                Check(
                    LoweredSafetyCheckKind.SignedDivremRepresentable,
                    divrem,
                    instruction.Type),
            }
            : new[]
            {
                Check(LoweredSafetyCheckKind.DivisorNonzero, divrem, instruction.Type),
            };
    }

    private static void CompareCheckSequences(
        IReadOnlyList<LoweredSafetyCheck> expected,
        IReadOnlyList<LoweredSafetyCheck> actual)
    {
        foreach (LoweredSafetyCheck check in expected)
        {
            int expectedCount = expected.Count(candidate => SameCheck(candidate, check));
            int actualCount = actual.Count(candidate => SameCheck(candidate, check));
            if (actualCount < expectedCount)
            {
                throw FrontendFailure.Rejected("lowering", "CSHARP_LOWERING_CHECK_MISSING");
            }
        }

        foreach (LoweredSafetyCheck check in actual)
        {
            int expectedCount = expected.Count(candidate => SameCheck(candidate, check));
            int actualCount = actual.Count(candidate => SameCheck(candidate, check));
            if (actualCount > expectedCount)
            {
                throw FrontendFailure.Rejected("lowering", "CSHARP_LOWERING_CHECK_EXTRA");
            }
        }

        if (expected.Count != actual.Count)
        {
            throw FrontendFailure.Rejected("lowering", "CSHARP_LOWERING_CHECK_EXTRA");
        }

        for (int index = 0; index < expected.Count; index++)
        {
            if (!SameCheck(expected[index], actual[index]))
            {
                throw FrontendFailure.Rejected("lowering", "CSHARP_LOWERING_CHECK_ORDER");
            }
        }
    }

    private static void ValidateRequiredCheckLedger(LoweredFunction function)
    {
        LoweredRequiredCheck[] expected = CanonicalRequiredChecks(function.Blocks);
        Dictionary<RequiredCheckKey, int> expectedCounts = CountRequiredChecks(expected);
        Dictionary<RequiredCheckKey, int> actualCounts = CountRequiredChecks(
            function.RequiredChecks);
        foreach ((RequiredCheckKey check, int expectedCount) in expectedCounts)
        {
            actualCounts.TryGetValue(check, out int actualCount);
            if (actualCount < expectedCount)
            {
                throw FrontendFailure.Rejected("lowering", "CSHARP_LOWERING_CHECK_MISSING");
            }
        }

        foreach ((RequiredCheckKey check, int actualCount) in actualCounts)
        {
            expectedCounts.TryGetValue(check, out int expectedCount);
            if (actualCount > expectedCount)
            {
                throw FrontendFailure.Rejected("lowering", "CSHARP_LOWERING_CHECK_EXTRA");
            }
        }

        if (expected.Length != function.RequiredChecks.Count)
        {
            throw FrontendFailure.Rejected("lowering", "CSHARP_LOWERING_CHECK_EXTRA");
        }

        for (int index = 0; index < expected.Length; index++)
        {
            if (!SameRequiredCheck(expected[index], function.RequiredChecks[index]))
            {
                throw FrontendFailure.Rejected("lowering", "CSHARP_LOWERING_CHECK_ORDER");
            }
        }
    }

    private static Dictionary<RequiredCheckKey, int> CountRequiredChecks(
        IEnumerable<LoweredRequiredCheck> checks)
    {
        var counts = new Dictionary<RequiredCheckKey, int>();
        foreach (LoweredRequiredCheck check in checks)
        {
            var key = new RequiredCheckKey(
                check.InstructionId,
                check.Check.Kind,
                check.Check.Operation,
                check.Check.Width,
                check.Check.Signed);
            counts.TryGetValue(key, out int count);
            counts[key] = checked(count + 1);
        }

        return counts;
    }

    private static void ValidateGraph(
        LoweredFunction function,
        IReadOnlyDictionary<string, SubsetValueType> types)
    {
        var blocks = function.Blocks.ToDictionary(block => block.Label, StringComparer.Ordinal);
        var incoming = function.Blocks.ToDictionary(
            block => block.Label,
            _ => 0,
            StringComparer.Ordinal);
        foreach (LoweredBlock block in function.Blocks)
        {
            ValidateOrigin(block.Terminator.Origin);
            switch (block.Terminator.Kind)
            {
                case LoweredTerminatorKind.Return:
                    if (block.Terminator.Condition is not null
                        || block.Terminator.FalseTarget is not null
                        || block.Terminator.TrueTarget is not null
                        || block.Terminator.FalseArguments.Count != 0
                        || block.Terminator.TrueArguments.Count != 0
                        || block.Terminator.Values.Count != 1
                        || block.Terminator.Values[0].Type != function.Results[0].Type)
                    {
                        throw LoweringFailure.ControlFlow();
                    }

                    ValidateValue(block.Terminator.Values[0], types);
                    break;
                case LoweredTerminatorKind.Jump:
                    if (block.Terminator.Condition is not null
                        || block.Terminator.FalseTarget is null
                        || block.Terminator.TrueTarget is not null
                        || block.Terminator.TrueArguments.Count != 0
                        || block.Terminator.Values.Count != 0)
                    {
                        throw LoweringFailure.ControlFlow();
                    }

                    ValidateEdge(
                        block.Terminator.FalseTarget,
                        block.Terminator.FalseArguments,
                        blocks,
                        incoming,
                        types);
                    break;
                case LoweredTerminatorKind.Branch:
                    if (block.Terminator.Condition is null
                        || block.Terminator.Condition.Type != SubsetValueType.Bool
                        || block.Terminator.FalseTarget is null
                        || block.Terminator.TrueTarget is null
                        || block.Terminator.Values.Count != 0)
                    {
                        throw LoweringFailure.ControlFlow();
                    }

                    ValidateValue(block.Terminator.Condition, types);
                    ValidateEdge(
                        block.Terminator.FalseTarget,
                        block.Terminator.FalseArguments,
                        blocks,
                        incoming,
                        types);
                    ValidateEdge(
                        block.Terminator.TrueTarget,
                        block.Terminator.TrueArguments,
                        blocks,
                        incoming,
                        types);
                    break;
                default:
                    throw LoweringFailure.ControlFlow();
            }
        }

        if (function.Blocks[0].Parameters.Count != 0
            || incoming[function.Blocks[0].Label] != 0
            || function.Blocks.Skip(1).Any(block => incoming[block.Label] == 0))
        {
            throw LoweringFailure.ControlFlow();
        }

        var order = new List<string>();
        var queue = new Queue<string>();
        var seen = new HashSet<string>(StringComparer.Ordinal);
        queue.Enqueue("bb0");
        seen.Add("bb0");
        while (queue.Count != 0)
        {
            string label = queue.Dequeue();
            order.Add(label);
            LoweredTerminator terminator = blocks[label].Terminator;
            foreach (string successor in SuccessorsFalseThenTrue(terminator))
            {
                if (seen.Add(successor))
                {
                    queue.Enqueue(successor);
                }
            }
        }

        if (order.Count != function.Blocks.Count)
        {
            throw LoweringFailure.ControlFlow();
        }

        for (int index = 0; index < order.Count; index++)
        {
            if (!string.Equals(
                order[index],
                "bb" + index.ToString(CultureInfo.InvariantCulture),
                StringComparison.Ordinal))
            {
                throw LoweringFailure.ControlFlow();
            }
        }

        ValidateAcyclic(function, blocks);
    }

    private static void ValidateEdge(
        string target,
        IReadOnlyList<LoweredValue> arguments,
        IReadOnlyDictionary<string, LoweredBlock> blocks,
        Dictionary<string, int> incoming,
        IReadOnlyDictionary<string, SubsetValueType> types)
    {
        if (!blocks.TryGetValue(target, out LoweredBlock? destination)
            || arguments.Count != destination.Parameters.Count)
        {
            throw LoweringFailure.ControlFlow();
        }

        incoming[target]++;
        for (int index = 0; index < arguments.Count; index++)
        {
            ValidateValue(arguments[index], types);
            if (arguments[index].Type != destination.Parameters[index].Type)
            {
                throw LoweringFailure.ControlFlow();
            }
        }
    }

    private static IEnumerable<string> SuccessorsFalseThenTrue(LoweredTerminator terminator)
    {
        if (terminator.FalseTarget is not null)
        {
            yield return terminator.FalseTarget;
        }

        if (terminator.TrueTarget is not null)
        {
            yield return terminator.TrueTarget;
        }
    }

    private static void ValidateAcyclic(
        LoweredFunction function,
        IReadOnlyDictionary<string, LoweredBlock> blocks)
    {
        var state = function.Blocks.ToDictionary(
            block => block.Label,
            _ => (byte)0,
            StringComparer.Ordinal);
        Visit("bb0", blocks, state);
        if (state.Values.Any(value => value != 2))
        {
            throw LoweringFailure.ControlFlow();
        }
    }

    private static void Visit(
        string label,
        IReadOnlyDictionary<string, LoweredBlock> blocks,
        Dictionary<string, byte> state)
    {
        if (state[label] == 1)
        {
            throw LoweringFailure.ControlFlow();
        }

        if (state[label] == 2)
        {
            return;
        }

        state[label] = 1;
        foreach (string successor in SuccessorsFalseThenTrue(blocks[label].Terminator))
        {
            Visit(successor, blocks, state);
        }

        state[label] = 2;
    }

    private static void ValidateFeatures(LoweredFunction function)
    {
        var expected = new HashSet<LoweredFeature>();
        if (function.Locals.Count != 0)
        {
            expected.Add(LoweredFeature.MutableLocal);
        }

        foreach (LoweredBlock block in function.Blocks)
        {
            if (block.Terminator.Kind == LoweredTerminatorKind.Branch
                || block.Parameters.Count != 0)
            {
                expected.Add(LoweredFeature.Branch);
            }

            if (block.Instructions.Any(instruction =>
                instruction.Kind == LoweredInstructionKind.Convert))
            {
                expected.Add(LoweredFeature.Conversion);
            }

            if (block.Instructions.Any(instruction =>
                instruction.Kind == LoweredInstructionKind.CallStatic))
            {
                expected.Add(LoweredFeature.CallStatic);
            }
        }

        LoweredFeature[] ordered = expected.OrderBy(feature => feature).ToArray();
        if (ordered.Length != function.Features.Count)
        {
            throw LoweringFailure.Operation();
        }

        for (int index = 0; index < ordered.Length; index++)
        {
            if (ordered[index] != function.Features[index])
            {
                throw LoweringFailure.Operation();
            }
        }
    }

    private static void ValidateValue(
        LoweredValue value,
        IReadOnlyDictionary<string, SubsetValueType> types)
    {
        switch (value.Kind)
        {
            case LoweredValueKind.Variable:
                if (value.Text is null
                    || !types.TryGetValue(value.Text, out SubsetValueType type)
                    || type != value.Type)
                {
                    throw LoweringFailure.Operation();
                }

                return;
            case LoweredValueKind.Boolean:
                if (value.Type != SubsetValueType.Bool || value.Text is not null)
                {
                    throw LoweringFailure.Operation();
                }

                return;
            case LoweredValueKind.Integer:
                if (!SubsetTypeRules.IsInteger(value.Type)
                    || value.Text is null
                    || !IsCanonicalDecimal(value.Text)
                    || !IsInRange(value.Text, value.Type))
                {
                    throw LoweringFailure.Operation();
                }

                return;
            default:
                throw LoweringFailure.Operation();
        }
    }

    private static void ValidateOrigin(LoweredOrigin origin)
    {
        if (string.IsNullOrEmpty(origin.NormalizedPath)
            || origin.Utf16Start < 0
            || origin.Utf16End <= origin.Utf16Start)
        {
            throw LoweringFailure.Operation();
        }
    }

    private static void ValidateCallClosure(LoweredClosure closure)
    {
        var functions = closure.Functions
            .Select((function, index) => (function, index))
            .ToDictionary(pair => pair.function.Id, StringComparer.Ordinal);
        var callersByCallee = functions.Keys.ToDictionary(
            id => id,
            _ => new SortedSet<string>(StringComparer.Ordinal),
            StringComparer.Ordinal);
        var remaining = functions.Keys.ToDictionary(
            id => id,
            _ => 0,
            StringComparer.Ordinal);

        foreach (LoweredFunction caller in closure.Functions)
        {
            var callees = new SortedSet<string>(StringComparer.Ordinal);
            foreach (LoweredInstruction call in caller.Blocks
                .SelectMany(block => block.Instructions)
                .Where(instruction => instruction.Kind == LoweredInstructionKind.CallStatic))
            {
                if (call.Function is null
                    || call.ContractHash is null
                    || !functions.TryGetValue(call.Function, out var callee)
                    || !string.Equals(
                        call.ContractHash,
                        callee.function.ContractHash,
                        StringComparison.Ordinal)
                    || callee.index >= functions[caller.Id].index)
                {
                    throw LoweringFailure.Operation();
                }

                callees.Add(call.Function);
            }

            remaining[caller.Id] = callees.Count;
            foreach (string callee in callees)
            {
                callersByCallee[callee].Add(caller.Id);
            }
        }

        var ready = new SortedSet<string>(
            remaining.Where(pair => pair.Value == 0).Select(pair => pair.Key),
            StringComparer.Ordinal);
        var expected = new List<string>(closure.Functions.Count);
        while (ready.Count != 0)
        {
            string next = ready.Min ?? throw LoweringFailure.ControlFlow();
            ready.Remove(next);
            expected.Add(next);
            foreach (string caller in callersByCallee[next])
            {
                remaining[caller]--;
                if (remaining[caller] == 0)
                {
                    ready.Add(caller);
                }
            }
        }

        if (expected.Count != closure.Functions.Count
            || !expected.SequenceEqual(
                closure.Functions.Select(function => function.Id),
                StringComparer.Ordinal))
        {
            throw LoweringFailure.ControlFlow();
        }
    }

    private static SubsetValueType TypeFromToken(string token)
    {
        return token switch
        {
            "bool" => SubsetValueType.Bool,
            "i32" => SubsetValueType.I32,
            "u32" => SubsetValueType.U32,
            "i64" => SubsetValueType.I64,
            "u64" => SubsetValueType.U64,
            _ => throw LoweringFailure.Operation(),
        };
    }

    private static void RequireSameOperands(SubsetValueType left, SubsetValueType right)
    {
        if (left != right)
        {
            throw LoweringFailure.Operation();
        }
    }

    private static void RequireResult(
        LoweredInstruction instruction,
        SubsetValueType type,
        ExplicitOverflowContext context)
    {
        if (instruction.Type != type || instruction.OverflowContext != context)
        {
            throw LoweringFailure.Operation();
        }
    }

    private static void RequireSameIntegerResult(
        LoweredInstruction instruction,
        SubsetValueType left,
        SubsetValueType right)
    {
        if (!SubsetTypeRules.IsInteger(left)
            || left != right
            || instruction.Type != left)
        {
            throw LoweringFailure.Operation();
        }
    }

    private static LoweredSafetyCheck Check(
        LoweredSafetyCheckKind kind,
        LoweredCheckOperation operation,
        SubsetValueType type)
    {
        return new LoweredSafetyCheck(
            kind,
            operation,
            LoweringMethodBuilder.Width(type),
            LoweringMethodBuilder.IsSigned(type));
    }

    private static bool SameCheck(LoweredSafetyCheck left, LoweredSafetyCheck right)
    {
        return left.Kind == right.Kind
            && left.Operation == right.Operation
            && left.Width == right.Width
            && left.Signed == right.Signed;
    }

    private static bool SameRequiredCheck(
        LoweredRequiredCheck left,
        LoweredRequiredCheck right)
    {
        return string.Equals(left.InstructionId, right.InstructionId, StringComparison.Ordinal)
            && SameCheck(left.Check, right.Check);
    }

    private static int CompareRequiredChecks(
        LoweredRequiredCheck left,
        LoweredRequiredCheck right)
    {
        int result = left.Check.Kind.CompareTo(right.Check.Kind);
        if (result != 0)
        {
            return result;
        }

        result = OperationOrder(left.Check.Operation).CompareTo(OperationOrder(right.Check.Operation));
        if (result != 0)
        {
            return result;
        }

        result = left.Check.Width.CompareTo(right.Check.Width);
        if (result != 0)
        {
            return result;
        }

        result = left.Check.Signed.CompareTo(right.Check.Signed);
        if (result != 0)
        {
            return result;
        }

        return CompareIndexedId(left.InstructionId, right.InstructionId, 't');
    }

    private static int OperationOrder(LoweredCheckOperation operation)
    {
        return operation switch
        {
            LoweredCheckOperation.Add or LoweredCheckOperation.Div => 0,
            LoweredCheckOperation.Sub or LoweredCheckOperation.Rem => 1,
            LoweredCheckOperation.Mul => 2,
            LoweredCheckOperation.Neg => 3,
            _ => 4,
        };
    }

    private static int CompareIndexedId(string left, string right, char prefix)
    {
        if (left.Length < 2
            || right.Length < 2
            || left[0] != prefix
            || right[0] != prefix
            || !int.TryParse(left.AsSpan(1), NumberStyles.None, CultureInfo.InvariantCulture, out int leftIndex)
            || !int.TryParse(right.AsSpan(1), NumberStyles.None, CultureInfo.InvariantCulture, out int rightIndex))
        {
            throw LoweringFailure.ControlFlow();
        }

        return leftIndex.CompareTo(rightIndex);
    }

    private static bool IsCanonicalDecimal(string value)
    {
        if (string.Equals(value, "0", StringComparison.Ordinal))
        {
            return true;
        }

        ReadOnlySpan<char> digits = value.AsSpan();
        if (digits.Length != 0 && digits[0] == '-')
        {
            digits = digits[1..];
        }

        return digits.Length != 0
            && digits[0] != '0'
            && !digits.Contains('-')
            && digits.ToString().All(character => character >= '0' && character <= '9');
    }

    private static bool IsInRange(string value, SubsetValueType type)
    {
        return type switch
        {
            SubsetValueType.I32 => int.TryParse(
                value,
                NumberStyles.AllowLeadingSign,
                CultureInfo.InvariantCulture,
                out _),
            SubsetValueType.U32 => uint.TryParse(
                value,
                NumberStyles.None,
                CultureInfo.InvariantCulture,
                out _),
            SubsetValueType.I64 => long.TryParse(
                value,
                NumberStyles.AllowLeadingSign,
                CultureInfo.InvariantCulture,
                out _),
            SubsetValueType.U64 => ulong.TryParse(
                value,
                NumberStyles.None,
                CultureInfo.InvariantCulture,
                out _),
            _ => false,
        };
    }

    internal static bool IsLowercaseSha256(string value)
    {
        return value.Length == 64
            && value.All(character =>
                (character >= '0' && character <= '9')
                || (character >= 'a' && character <= 'f'));
    }

    private readonly record struct RequiredCheckKey(
        string InstructionId,
        LoweredSafetyCheckKind Kind,
        LoweredCheckOperation Operation,
        int Width,
        bool Signed);
}
