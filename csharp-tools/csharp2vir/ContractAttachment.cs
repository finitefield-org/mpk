using System;
using System.Collections.Generic;
using Microsoft.CodeAnalysis;

namespace Mpk.CSharp2Vir;

internal static class CSharpContracts
{
    internal static ContractSet Attach(
        Selection selection,
        CapturedSnapshot snapshot,
        SubsetClosure closure)
    {
        ContractHashing.ValidateSelectionLink(selection, snapshot);
        if (closure.SelectedRoots.Length != selection.Raw.Methods.Count)
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

        var closureById = new Dictionary<string, SubsetMethod>(StringComparer.Ordinal);
        foreach (SubsetMethod method in closure.Methods)
        {
            if (!closureById.TryAdd(method.CanonicalId, method))
            {
                throw FrontendFailure.Internal("subset");
            }
        }

        if (closureById.Count == 0)
        {
            throw FrontendFailure.Internal("subset");
        }

        var parsedByMethod = new Dictionary<string, ParsedAttachment>(StringComparer.Ordinal);
        var closureCounter = new ContractClosureCounter();
        foreach (string path in selection.Raw.Contracts)
        {
            CapturedFile file = snapshot.Find(CapturedInputKind.Contract, path);
            ParsedContractSidecar sidecar = ContractSidecarParser.Parse(
                file.Bytes,
                closureCounter);
            if (!closureById.ContainsKey(sidecar.Method))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_UNUSED");
            }

            if (!parsedByMethod.TryAdd(
                sidecar.Method,
                new ParsedAttachment(path, file.Sha256, sidecar)))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_DUPLICATE");
            }
        }

        foreach (SubsetMethod method in closure.Methods)
        {
            if (!parsedByMethod.ContainsKey(method.CanonicalId))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_MISSING");
            }
        }

        var attached = new AttachedContract[closure.Methods.Length];
        for (int index = 0; index < closure.Methods.Length; index++)
        {
            SubsetMethod method = closure.Methods[index];
            ParsedAttachment parsed = parsedByMethod[method.CanonicalId];
            ContractHashing.ValidateSidecarHash(parsed.Sidecar);
            NormalizedContract normalized = ContractTypeChecker.Normalize(
                selection.Raw.Compilation,
                method,
                parsed.Sidecar);
            attached[index] = new AttachedContract(
                parsed.Path,
                parsed.RawInputSha256,
                parsed.Sidecar,
                normalized);
        }

        return new ContractSet(selection.Sha256, attached);
    }

    private sealed class ParsedAttachment
    {
        internal ParsedAttachment(
            string path,
            string rawInputSha256,
            ParsedContractSidecar sidecar)
        {
            Path = path;
            RawInputSha256 = rawInputSha256;
            Sidecar = sidecar;
        }

        internal string Path { get; }

        internal string RawInputSha256 { get; }

        internal ParsedContractSidecar Sidecar { get; }
    }
}

internal static class ContractTypeChecker
{
    private sealed class ParameterBinding
    {
        internal ParameterBinding(int index, SubsetValueType type)
        {
            Index = index;
            Type = type;
        }

        internal int Index { get; }

        internal SubsetValueType Type { get; }
    }

    internal static NormalizedContract Normalize(
        string compilation,
        SubsetMethod method,
        ParsedContractSidecar sidecar)
    {
        if (!string.Equals(method.CanonicalId, sidecar.Method, StringComparison.Ordinal))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_IDENTITY");
        }

        var parameters = new Dictionary<string, ParameterBinding>(StringComparer.Ordinal);
        for (int index = 0; index < method.Symbol.Parameters.Length; index++)
        {
            IParameterSymbol parameter = method.Symbol.Parameters[index];
            if (parameter.Ordinal != index
                || !parameters.TryAdd(
                    parameter.Name,
                    new ParameterBinding(index, ExactType(parameter.Type))))
            {
                throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_TYPE");
            }
        }

        SubsetValueType resultType = ExactType(method.Symbol.ReturnType);
        var requires = new NormalizedContractExpression[sidecar.Requires.Count];
        for (int index = 0; index < sidecar.Requires.Count; index++)
        {
            NormalizedContractExpression expression = NormalizeExpression(
                sidecar.Requires[index],
                parameters,
                resultType,
                allowResult: false);
            RequireBoolean(expression);
            requires[index] = expression;
        }

        var ensures = new NormalizedContractExpression[sidecar.Ensures.Count];
        for (int index = 0; index < sidecar.Ensures.Count; index++)
        {
            NormalizedContractExpression expression = NormalizeExpression(
                sidecar.Ensures[index],
                parameters,
                resultType,
                allowResult: true);
            RequireBoolean(expression);
            ensures[index] = expression;
        }

        return new NormalizedContract(compilation, sidecar.Method, requires, ensures);
    }

    private static NormalizedContractExpression NormalizeExpression(
        ContractExpression expression,
        IReadOnlyDictionary<string, ParameterBinding> parameters,
        SubsetValueType resultType,
        bool allowResult)
    {
        switch (expression.Kind)
        {
            case ContractExpressionKind.Parameter:
            {
                string name = expression.Text ?? throw HashFailure();
                if (!parameters.TryGetValue(name, out ParameterBinding? parameter))
                {
                    throw TypeFailure();
                }

                return NormalizedContractExpression.Variable(parameter.Index, parameter.Type);
            }
            case ContractExpressionKind.Result:
                if (!allowResult)
                {
                    throw TypeFailure();
                }

                return NormalizedContractExpression.Result(resultType);
            case ContractExpressionKind.Boolean:
                return NormalizedContractExpression.BooleanLiteral(expression.Boolean);
            case ContractExpressionKind.Integer:
                if (!SubsetTypeRules.IsInteger(expression.IntegerType))
                {
                    throw TypeFailure();
                }

                return NormalizedContractExpression.IntegerLiteral(
                    expression.Text ?? throw HashFailure(),
                    expression.IntegerType);
            case ContractExpressionKind.Unary:
                return NormalizeUnary(expression, parameters, resultType, allowResult);
            case ContractExpressionKind.Nary:
                return NormalizeNary(expression, parameters, resultType, allowResult);
            case ContractExpressionKind.Binary:
                return NormalizeBinary(expression, parameters, resultType, allowResult);
            default:
                throw HashFailure();
        }
    }

    private static NormalizedContractExpression NormalizeUnary(
        ContractExpression expression,
        IReadOnlyDictionary<string, ParameterBinding> parameters,
        SubsetValueType resultType,
        bool allowResult)
    {
        if (expression.Arguments.Count != 1 || expression.Text is null)
        {
            throw OperatorFailure();
        }

        NormalizedContractExpression operand = NormalizeExpression(
            expression.Arguments[0],
            parameters,
            resultType,
            allowResult);
        SubsetValueType type = expression.Text switch
        {
            "not" when operand.Type == SubsetValueType.Bool => SubsetValueType.Bool,
            "bv_neg" when SubsetTypeRules.IsInteger(operand.Type) => operand.Type,
            "bv_not" when SubsetTypeRules.IsInteger(operand.Type) => operand.Type,
            "not" or "bv_neg" or "bv_not" => throw TypeFailure(),
            _ => throw OperatorFailure(),
        };
        return NormalizedContractExpression.Operation(
            NormalizedContractExpressionKind.Unary,
            type,
            expression.Text,
            new[] { operand });
    }

    private static NormalizedContractExpression NormalizeNary(
        ContractExpression expression,
        IReadOnlyDictionary<string, ParameterBinding> parameters,
        SubsetValueType resultType,
        bool allowResult)
    {
        if (expression.Text is not ("and" or "or")
            || expression.Arguments.Count < 2
            || expression.Arguments.Count > ContractLimits.OperatorArgumentsMaximum)
        {
            throw OperatorFailure();
        }

        var arguments = new NormalizedContractExpression[expression.Arguments.Count];
        for (int index = 0; index < expression.Arguments.Count; index++)
        {
            NormalizedContractExpression argument = NormalizeExpression(
                expression.Arguments[index],
                parameters,
                resultType,
                allowResult);
            RequireBoolean(argument);
            arguments[index] = argument;
        }

        return NormalizedContractExpression.Operation(
            NormalizedContractExpressionKind.Nary,
            SubsetValueType.Bool,
            expression.Text,
            arguments);
    }

    private static NormalizedContractExpression NormalizeBinary(
        ContractExpression expression,
        IReadOnlyDictionary<string, ParameterBinding> parameters,
        SubsetValueType resultType,
        bool allowResult)
    {
        if (expression.Arguments.Count != 2 || expression.Text is null)
        {
            throw OperatorFailure();
        }

        NormalizedContractExpression left = NormalizeExpression(
            expression.Arguments[0],
            parameters,
            resultType,
            allowResult);
        NormalizedContractExpression right = NormalizeExpression(
            expression.Arguments[1],
            parameters,
            resultType,
            allowResult);
        SubsetValueType type;
        switch (expression.Text)
        {
            case "eq":
            case "not_eq":
                RequireSame(left, right);
                type = SubsetValueType.Bool;
                break;
            case "signed_lt":
            case "signed_le":
            case "signed_gt":
            case "signed_ge":
                RequireSameInteger(left, right);
                if (!SubsetTypeRules.IsSigned(left.Type))
                {
                    throw TypeFailure();
                }

                type = SubsetValueType.Bool;
                break;
            case "unsigned_lt":
            case "unsigned_le":
            case "unsigned_gt":
            case "unsigned_ge":
                RequireSameInteger(left, right);
                if (SubsetTypeRules.IsSigned(left.Type))
                {
                    throw TypeFailure();
                }

                type = SubsetValueType.Bool;
                break;
            case "bv_add":
            case "bv_sub":
            case "bv_mul":
            case "bv_and":
            case "bv_or":
            case "bv_xor":
                RequireSameInteger(left, right);
                type = left.Type;
                break;
            case "bv_shl":
                RequireInteger(left);
                RequireInteger(right);
                type = left.Type;
                break;
            case "bv_ashr":
                RequireInteger(left);
                RequireInteger(right);
                if (!SubsetTypeRules.IsSigned(left.Type))
                {
                    throw TypeFailure();
                }

                type = left.Type;
                break;
            case "bv_lshr":
                RequireInteger(left);
                RequireInteger(right);
                if (SubsetTypeRules.IsSigned(left.Type))
                {
                    throw TypeFailure();
                }

                type = left.Type;
                break;
            default:
                throw OperatorFailure();
        }

        return NormalizedContractExpression.Operation(
            NormalizedContractExpressionKind.Binary,
            type,
            expression.Text,
            new[] { left, right });
    }

    private static SubsetValueType ExactType(ITypeSymbol type)
    {
        return type.SpecialType switch
        {
            SpecialType.System_Boolean => SubsetValueType.Bool,
            SpecialType.System_Int32 => SubsetValueType.I32,
            SpecialType.System_UInt32 => SubsetValueType.U32,
            SpecialType.System_Int64 => SubsetValueType.I64,
            SpecialType.System_UInt64 => SubsetValueType.U64,
            _ => throw TypeFailure(),
        };
    }

    private static void RequireBoolean(NormalizedContractExpression expression)
    {
        if (expression.Type != SubsetValueType.Bool)
        {
            throw TypeFailure();
        }
    }

    private static void RequireInteger(NormalizedContractExpression expression)
    {
        if (!SubsetTypeRules.IsInteger(expression.Type))
        {
            throw TypeFailure();
        }
    }

    private static void RequireSame(
        NormalizedContractExpression left,
        NormalizedContractExpression right)
    {
        if (left.Type != right.Type)
        {
            throw TypeFailure();
        }
    }

    private static void RequireSameInteger(
        NormalizedContractExpression left,
        NormalizedContractExpression right)
    {
        RequireInteger(left);
        RequireInteger(right);
        RequireSame(left, right);
    }

    private static FrontendFailure TypeFailure()
    {
        return FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_TYPE");
    }

    private static FrontendFailure OperatorFailure()
    {
        return FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_OPERATOR");
    }

    private static FrontendFailure HashFailure()
    {
        return FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
    }
}
