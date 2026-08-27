using System;
using System.Collections.Generic;
using System.Globalization;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

internal sealed class ContractClosureCounter
{
    private uint nodes;

    internal uint Nodes => nodes;

    internal void AddNode()
    {
        if (nodes == ContractLimits.NodesPerClosureMaximum)
        {
            throw FrontendFailure.Rejected(
                "subset",
                "CSHARP_LIMIT_CONTRACT_NODES_PER_CLOSURE");
        }

        nodes++;
    }
}

internal static class ContractLimits
{
    internal const uint ClausesMaximum = FrontendLimits.ContractClausesMaximum;
    internal const uint NodesPerMethodMaximum = FrontendLimits.ContractNodesPerMethodMaximum;
    internal const uint NodesPerClosureMaximum = FrontendLimits.ContractNodesPerClosureMaximum;
    internal const uint ExpressionDepthMaximum = FrontendLimits.ContractDepthMaximum;
    internal const int OperatorArgumentsMaximum = 64;
}

internal sealed class ContractMethodCounter
{
    private readonly ContractClosureCounter closure;
    private uint clauses;
    private uint nodes;

    internal ContractMethodCounter(ContractClosureCounter closure)
    {
        this.closure = closure;
    }

    internal uint Nodes => nodes;

    internal void AddClause()
    {
        if (clauses == ContractLimits.ClausesMaximum)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_LIMIT_CONTRACT_CLAUSES");
        }

        clauses++;
    }

    internal void AddNode(uint depth)
    {
        if (depth > ContractLimits.ExpressionDepthMaximum)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_LIMIT_CONTRACT_DEPTH");
        }

        if (nodes == ContractLimits.NodesPerMethodMaximum)
        {
            throw FrontendFailure.Rejected(
                "subset",
                "CSHARP_LIMIT_CONTRACT_NODES_PER_METHOD");
        }

        closure.AddNode();
        nodes++;
    }
}

internal static class ContractSidecarParser
{
    [Flags]
    private enum TopFields : ushort
    {
        None = 0,
        Schema = 1 << 0,
        SemanticProfile = 1 << 1,
        Method = 1 << 2,
        Requires = 1 << 3,
        Ensures = 1 << 4,
        Modifies = 1 << 5,
        AbruptCompletion = 1 << 6,
        Termination = 1 << 7,
        All = Schema | SemanticProfile | Method | Requires | Ensures | Modifies
            | AbruptCompletion | Termination,
    }

    [Flags]
    private enum ExpressionFields : byte
    {
        None = 0,
        Parameter = 1 << 0,
        Result = 1 << 1,
        Boolean = 1 << 2,
        Integer = 1 << 3,
        Operation = 1 << 4,
        Arguments = 1 << 5,
    }

    [Flags]
    private enum IntegerFields : byte
    {
        None = 0,
        Decimal = 1 << 0,
        Type = 1 << 1,
        All = Decimal | Type,
    }

    internal static ParsedContractSidecar Parse(
        ReadOnlySpan<byte> input,
        ContractClosureCounter closureCounter)
    {
        try
        {
            if (input.Length >= 3
                && input[0] == 0xef
                && input[1] == 0xbb
                && input[2] == 0xbf)
            {
                throw JsonFailure();
            }

            var reader = new Utf8JsonReader(
                input,
                isFinalBlock: true,
                new JsonReaderState(new JsonReaderOptions
                {
                    AllowTrailingCommas = false,
                    CommentHandling = JsonCommentHandling.Disallow,
                    MaxDepth = 128,
                }));
            if (!reader.Read() || reader.TokenType != JsonTokenType.StartObject)
            {
                throw JsonFailure();
            }

            ParsedContractSidecar sidecar = ParseRoot(ref reader, closureCounter);
            if (reader.Read())
            {
                throw JsonFailure();
            }

            return sidecar;
        }
        catch (FrontendFailure)
        {
            throw;
        }
        catch (Exception error) when (
            error is JsonException
            || error is FormatException
            || error is InvalidOperationException)
        {
            throw JsonFailure();
        }
    }

    private static ParsedContractSidecar ParseRoot(
        ref Utf8JsonReader reader,
        ContractClosureCounter closureCounter)
    {
        TopFields fields = TopFields.None;
        string? schema = null;
        string? semanticProfile = null;
        string? method = null;
        string? abruptCompletion = null;
        string? termination = null;
        ContractExpression[]? requires = null;
        ContractExpression[]? ensures = null;
        var counter = new ContractMethodCounter(closureCounter);

        while (Read(ref reader) && reader.TokenType != JsonTokenType.EndObject)
        {
            RequireToken(ref reader, JsonTokenType.PropertyName);
            string property = RequiredString(ref reader);
            ReadRequired(ref reader);
            switch (property)
            {
                case "schema":
                    AddField(ref fields, TopFields.Schema);
                    schema = ReadStringValue(ref reader);
                    break;
                case "semantic_profile":
                    AddField(ref fields, TopFields.SemanticProfile);
                    semanticProfile = ReadStringValue(ref reader);
                    break;
                case "method":
                    AddField(ref fields, TopFields.Method);
                    method = ReadStringValue(ref reader);
                    break;
                case "requires":
                    AddField(ref fields, TopFields.Requires);
                    requires = ParseClauseArray(ref reader, counter);
                    break;
                case "ensures":
                    AddField(ref fields, TopFields.Ensures);
                    ensures = ParseClauseArray(ref reader, counter);
                    break;
                case "modifies":
                    AddField(ref fields, TopFields.Modifies);
                    ParseEmptyModifies(ref reader);
                    break;
                case "abrupt_completion":
                    AddField(ref fields, TopFields.AbruptCompletion);
                    abruptCompletion = ReadStringValue(ref reader);
                    break;
                case "termination":
                    AddField(ref fields, TopFields.Termination);
                    termination = ReadStringValue(ref reader);
                    break;
                default:
                    throw ShapeFailure();
            }
        }

        if (reader.TokenType != JsonTokenType.EndObject || fields != TopFields.All)
        {
            throw ShapeFailure();
        }

        if (!string.Equals(schema, FrontendConstants.ContractSchema, StringComparison.Ordinal)
            || !string.Equals(
                semanticProfile,
                FrontendConstants.SemanticProfile,
                StringComparison.Ordinal)
            || !string.Equals(abruptCompletion, "forbidden", StringComparison.Ordinal)
            || !string.Equals(termination, "total", StringComparison.Ordinal)
            || ensures is null
            || ensures.Length == 0
            || requires is null
            || method is null)
        {
            throw IdentityFailure();
        }

        try
        {
            CanonicalMethodId parsed = SelectionCodec.ParseMethodId(method);
            if (!string.Equals(parsed.Canonical, method, StringComparison.Ordinal))
            {
                throw IdentityFailure();
            }
        }
        catch (SelectionSyntaxFailure)
        {
            throw IdentityFailure();
        }

        return new ParsedContractSidecar(method, requires, ensures, counter.Nodes);
    }

    private static ContractExpression[] ParseClauseArray(
        ref Utf8JsonReader reader,
        ContractMethodCounter counter)
    {
        RequireToken(ref reader, JsonTokenType.StartArray);
        var expressions = new List<ContractExpression>();
        while (Read(ref reader) && reader.TokenType != JsonTokenType.EndArray)
        {
            counter.AddClause();
            ContractExpression expression = ParseExpression(ref reader, counter, depth: 1);
            expressions.Add(expression);
        }

        if (reader.TokenType != JsonTokenType.EndArray)
        {
            throw JsonFailure();
        }

        return expressions.ToArray();
    }

    private static ContractExpression ParseExpression(
        ref Utf8JsonReader reader,
        ContractMethodCounter counter,
        uint depth)
    {
        RequireToken(ref reader, JsonTokenType.StartObject);
        counter.AddNode(depth);
        ExpressionFields fields = ExpressionFields.None;
        string? parameter = null;
        bool boolean = false;
        string? decimalValue = null;
        SubsetValueType integerType = default;
        string? operation = null;
        ContractExpression[]? arguments = null;

        while (Read(ref reader) && reader.TokenType != JsonTokenType.EndObject)
        {
            RequireToken(ref reader, JsonTokenType.PropertyName);
            string property = RequiredString(ref reader);
            ReadRequired(ref reader);
            switch (property)
            {
                case "parameter":
                    AddField(ref fields, ExpressionFields.Parameter);
                    parameter = ReadStringValue(ref reader);
                    break;
                case "result":
                    AddField(ref fields, ExpressionFields.Result);
                    ParseResult(ref reader);
                    break;
                case "bool":
                    AddField(ref fields, ExpressionFields.Boolean);
                    if (reader.TokenType != JsonTokenType.True
                        && reader.TokenType != JsonTokenType.False)
                    {
                        throw ShapeFailure();
                    }

                    boolean = reader.GetBoolean();
                    break;
                case "int":
                    AddField(ref fields, ExpressionFields.Integer);
                    (decimalValue, integerType) = ParseInteger(ref reader);
                    break;
                case "op":
                    AddField(ref fields, ExpressionFields.Operation);
                    operation = ReadStringValue(ref reader);
                    break;
                case "args":
                    AddField(ref fields, ExpressionFields.Arguments);
                    arguments = ParseArguments(ref reader, counter, depth);
                    break;
                default:
                    throw ShapeFailure();
            }
        }

        if (reader.TokenType != JsonTokenType.EndObject)
        {
            throw JsonFailure();
        }

        return fields switch
        {
            ExpressionFields.Parameter when parameter is not null
                => ContractExpression.Parameter(parameter),
            ExpressionFields.Result => ContractExpression.Result(),
            ExpressionFields.Boolean => ContractExpression.BooleanLiteral(boolean),
            ExpressionFields.Integer when decimalValue is not null
                => ContractExpression.IntegerLiteral(decimalValue, integerType),
            ExpressionFields.Operation | ExpressionFields.Arguments
                when operation is not null && arguments is not null
                => BuildOperation(operation, arguments),
            _ => throw ShapeFailure(),
        };
    }

    private static ContractExpression[] ParseArguments(
        ref Utf8JsonReader reader,
        ContractMethodCounter counter,
        uint parentDepth)
    {
        RequireToken(ref reader, JsonTokenType.StartArray);
        var arguments = new List<ContractExpression>();
        while (Read(ref reader) && reader.TokenType != JsonTokenType.EndArray)
        {
            if (arguments.Count == ContractLimits.OperatorArgumentsMaximum)
            {
                throw OperatorFailure();
            }

            ContractExpression expression = ParseExpression(
                ref reader,
                counter,
                checked(parentDepth + 1));
            arguments.Add(expression);
        }

        if (reader.TokenType != JsonTokenType.EndArray)
        {
            throw JsonFailure();
        }

        return arguments.ToArray();
    }

    private static ContractExpression BuildOperation(
        string operation,
        ContractExpression[] arguments)
    {
        switch (operation)
        {
            case "not":
            case "bv_neg":
            case "bv_not":
                if (arguments.Length != 1)
                {
                    throw OperatorFailure();
                }

                return ContractExpression.Operation(
                    ContractExpressionKind.Unary,
                    operation,
                    arguments);
            case "and":
            case "or":
                if (arguments.Length < 2 || arguments.Length > ContractLimits.OperatorArgumentsMaximum)
                {
                    throw OperatorFailure();
                }

                return ContractExpression.Operation(
                    ContractExpressionKind.Nary,
                    operation,
                    arguments);
            case "eq":
            case "not_eq":
            case "signed_lt":
            case "signed_le":
            case "signed_gt":
            case "signed_ge":
            case "unsigned_lt":
            case "unsigned_le":
            case "unsigned_gt":
            case "unsigned_ge":
            case "bv_add":
            case "bv_sub":
            case "bv_mul":
            case "bv_and":
            case "bv_or":
            case "bv_xor":
            case "bv_shl":
            case "bv_ashr":
            case "bv_lshr":
                if (arguments.Length != 2)
                {
                    throw OperatorFailure();
                }

                return ContractExpression.Operation(
                    ContractExpressionKind.Binary,
                    operation,
                    arguments);
            default:
                throw OperatorFailure();
        }
    }

    private static (string Decimal, SubsetValueType Type) ParseInteger(
        ref Utf8JsonReader reader)
    {
        RequireToken(ref reader, JsonTokenType.StartObject);
        IntegerFields fields = IntegerFields.None;
        string? decimalValue = null;
        string? typeToken = null;
        while (Read(ref reader) && reader.TokenType != JsonTokenType.EndObject)
        {
            RequireToken(ref reader, JsonTokenType.PropertyName);
            string property = RequiredString(ref reader);
            ReadRequired(ref reader);
            switch (property)
            {
                case "decimal":
                    AddField(ref fields, IntegerFields.Decimal);
                    decimalValue = ReadStringValue(ref reader);
                    break;
                case "type":
                    AddField(ref fields, IntegerFields.Type);
                    typeToken = ReadStringValue(ref reader);
                    break;
                default:
                    throw ShapeFailure();
            }
        }

        if (reader.TokenType != JsonTokenType.EndObject
            || fields != IntegerFields.All
            || decimalValue is null
            || typeToken is null)
        {
            throw ShapeFailure();
        }

        SubsetValueType type = typeToken switch
        {
            "i32" => SubsetValueType.I32,
            "u32" => SubsetValueType.U32,
            "i64" => SubsetValueType.I64,
            "u64" => SubsetValueType.U64,
            _ => throw TypeFailure(),
        };
        ValidateCanonicalInteger(decimalValue, type);
        return (decimalValue, type);
    }

    private static void ValidateCanonicalInteger(string value, SubsetValueType type)
    {
        if (value.Length == 0)
        {
            throw TypeFailure();
        }

        int firstDigit = value[0] == '-' ? 1 : 0;
        if (firstDigit == 1 && !SubsetTypeRules.IsSigned(type))
        {
            throw TypeFailure();
        }

        if (firstDigit == value.Length
            || (value[firstDigit] == '0' && value.Length != 1)
            || value[firstDigit] < '0'
            || value[firstDigit] > '9')
        {
            throw TypeFailure();
        }

        for (int index = firstDigit + 1; index < value.Length; index++)
        {
            if (value[index] < '0' || value[index] > '9')
            {
                throw TypeFailure();
            }
        }

        bool fits = type switch
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
        if (!fits)
        {
            throw TypeFailure();
        }
    }

    private static void ParseResult(ref Utf8JsonReader reader)
    {
        if (reader.TokenType != JsonTokenType.Number
            || reader.HasValueSequence
            || !reader.ValueSpan.SequenceEqual("0"u8))
        {
            throw TypeFailure();
        }
    }

    private static void ParseEmptyModifies(ref Utf8JsonReader reader)
    {
        RequireToken(ref reader, JsonTokenType.StartArray);
        ReadRequired(ref reader);
        if (reader.TokenType != JsonTokenType.EndArray)
        {
            throw IdentityFailure();
        }
    }

    private static string ReadStringValue(ref Utf8JsonReader reader)
    {
        RequireToken(ref reader, JsonTokenType.String);
        return RequiredString(ref reader);
    }

    private static string RequiredString(ref Utf8JsonReader reader)
    {
        return reader.GetString() ?? throw ShapeFailure();
    }

    private static bool Read(ref Utf8JsonReader reader)
    {
        return reader.Read();
    }

    private static void ReadRequired(ref Utf8JsonReader reader)
    {
        if (!reader.Read())
        {
            throw JsonFailure();
        }
    }

    private static void RequireToken(ref Utf8JsonReader reader, JsonTokenType expected)
    {
        if (reader.TokenType != expected)
        {
            throw ShapeFailure();
        }
    }

    private static void AddField(ref TopFields fields, TopFields field)
    {
        if ((fields & field) != 0)
        {
            throw DuplicateFailure();
        }

        fields |= field;
    }

    private static void AddField(ref ExpressionFields fields, ExpressionFields field)
    {
        if ((fields & field) != 0)
        {
            throw DuplicateFailure();
        }

        fields |= field;
    }

    private static void AddField(ref IntegerFields fields, IntegerFields field)
    {
        if ((fields & field) != 0)
        {
            throw DuplicateFailure();
        }

        fields |= field;
    }

    private static FrontendFailure JsonFailure()
    {
        return FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_JSON");
    }

    private static FrontendFailure ShapeFailure()
    {
        return FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_SHAPE");
    }

    private static FrontendFailure IdentityFailure()
    {
        return FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_IDENTITY");
    }

    private static FrontendFailure DuplicateFailure()
    {
        return FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_DUPLICATE");
    }

    private static FrontendFailure TypeFailure()
    {
        return FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_TYPE");
    }

    private static FrontendFailure OperatorFailure()
    {
        return FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_OPERATOR");
    }
}
