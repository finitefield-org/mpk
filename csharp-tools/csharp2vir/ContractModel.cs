using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace Mpk.CSharp2Vir;

internal enum ContractExpressionKind
{
    Parameter,
    Result,
    Boolean,
    Integer,
    Unary,
    Nary,
    Binary,
}

internal sealed class ContractExpression
{
    private readonly ReadOnlyCollection<ContractExpression> arguments;

    private ContractExpression(
        ContractExpressionKind kind,
        string? text,
        bool boolean,
        SubsetValueType integerType,
        ContractExpression[] arguments)
    {
        Kind = kind;
        Text = text;
        Boolean = boolean;
        IntegerType = integerType;
        this.arguments = Array.AsReadOnly((ContractExpression[])arguments.Clone());
    }

    internal ContractExpressionKind Kind { get; }

    // Parameter name, canonical decimal, or operator according to Kind.
    internal string? Text { get; }

    internal bool Boolean { get; }

    internal SubsetValueType IntegerType { get; }

    internal IReadOnlyList<ContractExpression> Arguments => arguments;

    internal static ContractExpression Parameter(string name)
    {
        return new ContractExpression(
            ContractExpressionKind.Parameter,
            name,
            false,
            default,
            Array.Empty<ContractExpression>());
    }

    internal static ContractExpression Result()
    {
        return new ContractExpression(
            ContractExpressionKind.Result,
            null,
            false,
            default,
            Array.Empty<ContractExpression>());
    }

    internal static ContractExpression BooleanLiteral(bool value)
    {
        return new ContractExpression(
            ContractExpressionKind.Boolean,
            null,
            value,
            default,
            Array.Empty<ContractExpression>());
    }

    internal static ContractExpression IntegerLiteral(string decimalValue, SubsetValueType type)
    {
        return new ContractExpression(
            ContractExpressionKind.Integer,
            decimalValue,
            false,
            type,
            Array.Empty<ContractExpression>());
    }

    internal static ContractExpression Operation(
        ContractExpressionKind kind,
        string operation,
        ContractExpression[] arguments)
    {
        return new ContractExpression(kind, operation, false, default, arguments);
    }
}

internal sealed class ParsedContractSidecar
{
    private readonly ReadOnlyCollection<ContractExpression> requires;
    private readonly ReadOnlyCollection<ContractExpression> ensures;
    private readonly byte[] canonicalBytes;

    internal ParsedContractSidecar(
        string method,
        ContractExpression[] requires,
        ContractExpression[] ensures,
        uint nodeCount)
    {
        Method = method;
        this.requires = Array.AsReadOnly((ContractExpression[])requires.Clone());
        this.ensures = Array.AsReadOnly((ContractExpression[])ensures.Clone());
        NodeCount = nodeCount;
        canonicalBytes = ContractCanonical.WriteSidecar(this);
        SidecarSha256 = ContractHashing.TypedSha256(
            ContractHashing.SidecarDomain,
            canonicalBytes);
    }

    internal string Method { get; }

    internal IReadOnlyList<ContractExpression> Requires => requires;

    internal IReadOnlyList<ContractExpression> Ensures => ensures;

    internal uint NodeCount { get; }

    internal ReadOnlySpan<byte> CanonicalBytes => canonicalBytes;

    internal string SidecarSha256 { get; }
}

internal enum NormalizedContractExpressionKind
{
    Variable,
    Result,
    Boolean,
    Integer,
    Unary,
    Nary,
    Binary,
}

internal sealed class NormalizedContractExpression
{
    private readonly ReadOnlyCollection<NormalizedContractExpression> arguments;

    private NormalizedContractExpression(
        NormalizedContractExpressionKind kind,
        SubsetValueType type,
        string? text,
        int index,
        bool boolean,
        NormalizedContractExpression[] arguments)
    {
        Kind = kind;
        Type = type;
        Text = text;
        Index = index;
        Boolean = boolean;
        this.arguments = Array.AsReadOnly((NormalizedContractExpression[])arguments.Clone());
    }

    internal NormalizedContractExpressionKind Kind { get; }

    internal SubsetValueType Type { get; }

    // Canonical decimal or operator according to Kind.
    internal string? Text { get; }

    // Parameter or result index according to Kind.
    internal int Index { get; }

    internal bool Boolean { get; }

    internal IReadOnlyList<NormalizedContractExpression> Arguments => arguments;

    internal static NormalizedContractExpression Variable(int index, SubsetValueType type)
    {
        return new NormalizedContractExpression(
            NormalizedContractExpressionKind.Variable,
            type,
            null,
            index,
            false,
            Array.Empty<NormalizedContractExpression>());
    }

    internal static NormalizedContractExpression Result(SubsetValueType type)
    {
        return new NormalizedContractExpression(
            NormalizedContractExpressionKind.Result,
            type,
            null,
            0,
            false,
            Array.Empty<NormalizedContractExpression>());
    }

    internal static NormalizedContractExpression BooleanLiteral(bool value)
    {
        return new NormalizedContractExpression(
            NormalizedContractExpressionKind.Boolean,
            SubsetValueType.Bool,
            null,
            0,
            value,
            Array.Empty<NormalizedContractExpression>());
    }

    internal static NormalizedContractExpression IntegerLiteral(
        string decimalValue,
        SubsetValueType type)
    {
        return new NormalizedContractExpression(
            NormalizedContractExpressionKind.Integer,
            type,
            decimalValue,
            0,
            false,
            Array.Empty<NormalizedContractExpression>());
    }

    internal static NormalizedContractExpression Operation(
        NormalizedContractExpressionKind kind,
        SubsetValueType type,
        string operation,
        NormalizedContractExpression[] arguments)
    {
        return new NormalizedContractExpression(kind, type, operation, 0, false, arguments);
    }
}

internal sealed class NormalizedContract
{
    private readonly ReadOnlyCollection<NormalizedContractExpression> requires;
    private readonly ReadOnlyCollection<NormalizedContractExpression> ensures;
    private readonly byte[] hashPayloadBytes;
    private readonly byte[] canonicalBytes;

    internal NormalizedContract(
        string unitId,
        string functionId,
        NormalizedContractExpression[] requires,
        NormalizedContractExpression[] ensures)
    {
        UnitId = unitId;
        FunctionId = functionId;
        this.requires = Array.AsReadOnly((NormalizedContractExpression[])requires.Clone());
        this.ensures = Array.AsReadOnly((NormalizedContractExpression[])ensures.Clone());
        hashPayloadBytes = ContractCanonical.WriteNormalized(this, includeHash: false);
        ContractHash = ContractHashing.TypedSha256(
            ContractHashing.NormalizedContractDomain,
            hashPayloadBytes);
        canonicalBytes = ContractCanonical.WriteNormalized(this, includeHash: true);
    }

    internal string UnitId { get; }

    internal string FunctionId { get; }

    internal IReadOnlyList<NormalizedContractExpression> Requires => requires;

    internal IReadOnlyList<NormalizedContractExpression> Ensures => ensures;

    internal ReadOnlySpan<byte> HashPayloadBytes => hashPayloadBytes;

    internal ReadOnlySpan<byte> CanonicalBytes => canonicalBytes;

    internal string ContractHash { get; }
}

internal sealed class AttachedContract
{
    internal AttachedContract(
        string normalizedPath,
        string rawInputSha256,
        ParsedContractSidecar sidecar,
        NormalizedContract normalized)
    {
        NormalizedPath = normalizedPath;
        RawInputSha256 = rawInputSha256;
        Sidecar = sidecar;
        Normalized = normalized;
    }

    internal string NormalizedPath { get; }

    // This remains input identity only. It is never substituted for a typed contract hash.
    internal string RawInputSha256 { get; }

    internal ParsedContractSidecar Sidecar { get; }

    internal NormalizedContract Normalized { get; }
}

internal sealed class ContractSet
{
    private readonly ReadOnlyCollection<AttachedContract> contracts;

    internal ContractSet(string selectionSha256, AttachedContract[] contracts)
    {
        SelectionSha256 = selectionSha256;
        this.contracts = Array.AsReadOnly((AttachedContract[])contracts.Clone());
    }

    internal string SelectionSha256 { get; }

    // Contracts are in the closure's deterministic callee-first order.
    internal IReadOnlyList<AttachedContract> Contracts => contracts;
}
