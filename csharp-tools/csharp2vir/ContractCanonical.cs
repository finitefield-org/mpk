using System;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

internal static class ContractHashing
{
    internal const string SidecarDomain = "MPK-CSHARP-CONTRACT-SIDECAR-0.1";
    internal const string NormalizedContractDomain = "MPK-CONTRACT-1.0";
    private const string SelectionDomain = "MPK-CSHARP-SELECTION-0.1";

    internal static string TypedSha256(string domain, ReadOnlySpan<byte> payload)
    {
        try
        {
            byte[] prefix = Encoding.ASCII.GetBytes(domain + "\0");
            var preimage = new byte[checked(prefix.Length + payload.Length)];
            Buffer.BlockCopy(prefix, 0, preimage, 0, prefix.Length);
            payload.CopyTo(preimage.AsSpan(prefix.Length));
            return Convert.ToHexString(SHA256.HashData(preimage)).ToLowerInvariant();
        }
        catch (Exception error) when (
            error is ArgumentException
            || error is CryptographicException
            || error is EncoderFallbackException
            || error is OverflowException)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
        }
    }

    internal static void ValidateSelectionLink(Selection selection, CapturedSnapshot snapshot)
    {
        ValidateSelectionHash(selection);
        ValidateSelectionHash(snapshot.Selection);
        if (!string.Equals(selection.Sha256, snapshot.Selection.Sha256, StringComparison.Ordinal)
            || !CryptographicOperations.FixedTimeEquals(
                selection.CanonicalBytes,
                snapshot.Selection.CanonicalBytes))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
        }
    }

    internal static void ValidateSidecarHash(ParsedContractSidecar sidecar)
    {
        byte[] canonical = ContractCanonical.WriteSidecar(sidecar);
        string computed = TypedSha256(SidecarDomain, canonical);
        if (!CryptographicOperations.FixedTimeEquals(canonical, sidecar.CanonicalBytes)
            || !string.Equals(computed, sidecar.SidecarSha256, StringComparison.Ordinal))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
        }
    }

    private static void ValidateSelectionHash(Selection selection)
    {
        byte[] canonical;
        try
        {
            canonical = SelectionCodec.CanonicalBytes(selection.Raw);
        }
        catch (Exception error) when (
            error is ArgumentException
            || error is EncoderFallbackException
            || error is InvalidOperationException
            || error is OverflowException)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
        }

        string computed = TypedSha256(SelectionDomain, canonical);
        if (!CryptographicOperations.FixedTimeEquals(canonical, selection.CanonicalBytes)
            || !string.Equals(computed, selection.Sha256, StringComparison.Ordinal))
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
        }
    }
}

internal static class ContractCanonical
{
    internal static byte[] WriteSidecar(ParsedContractSidecar sidecar)
    {
        return Write(writer =>
        {
            // RFC 8785/JCS member order. All accepted contract strings are ASCII.
            writer.WriteStartObject();
            writer.WriteString("abrupt_completion", "forbidden");
            WriteSidecarExpressions(writer, "ensures", sidecar.Ensures);
            writer.WriteString("method", sidecar.Method);
            writer.WritePropertyName("modifies");
            writer.WriteStartArray();
            writer.WriteEndArray();
            WriteSidecarExpressions(writer, "requires", sidecar.Requires);
            writer.WriteString("schema", FrontendConstants.ContractSchema);
            writer.WriteString("semantic_profile", FrontendConstants.SemanticProfile);
            writer.WriteString("termination", "total");
            writer.WriteEndObject();
        });
    }

    internal static byte[] WriteNormalized(NormalizedContract contract, bool includeHash)
    {
        return Write(writer =>
        {
            // contract_hash participates in canonical transport, but not its own hash payload.
            writer.WriteStartObject();
            if (includeHash)
            {
                writer.WriteString("contract_hash", contract.ContractHash);
            }

            WriteNormalizedExpressions(writer, "ensures", contract.Ensures);
            writer.WriteString("function_id", contract.FunctionId);
            writer.WritePropertyName("loops");
            writer.WriteStartArray();
            writer.WriteEndArray();
            writer.WritePropertyName("modifies");
            writer.WriteStartArray();
            writer.WriteEndArray();
            writer.WriteString("panic", "forbidden");
            WriteNormalizedExpressions(writer, "requires", contract.Requires);
            WriteSemanticContext(writer);
            writer.WriteString("termination", "total");
            writer.WriteString("unit_id", contract.UnitId);
            writer.WriteEndObject();
        });
    }

    private static byte[] Write(Action<Utf8JsonWriter> action)
    {
        try
        {
            using var output = new MemoryStream();
            using (var writer = new Utf8JsonWriter(
                output,
                new JsonWriterOptions
                {
                    Encoder = JavaScriptEncoder.UnsafeRelaxedJsonEscaping,
                    Indented = false,
                }))
            {
                action(writer);
            }

            return output.ToArray();
        }
        catch (FrontendFailure)
        {
            throw;
        }
        catch (Exception error) when (
            error is ArgumentException
            || error is InvalidOperationException
            || error is NotSupportedException
            || error is OverflowException)
        {
            throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
        }
    }

    private static void WriteSidecarExpressions(
        Utf8JsonWriter writer,
        string property,
        System.Collections.Generic.IReadOnlyList<ContractExpression> expressions)
    {
        writer.WritePropertyName(property);
        writer.WriteStartArray();
        foreach (ContractExpression expression in expressions)
        {
            WriteSidecarExpression(writer, expression);
        }

        writer.WriteEndArray();
    }

    private static void WriteSidecarExpression(Utf8JsonWriter writer, ContractExpression expression)
    {
        writer.WriteStartObject();
        switch (expression.Kind)
        {
            case ContractExpressionKind.Parameter:
                writer.WriteString("parameter", RequiredText(expression));
                break;
            case ContractExpressionKind.Result:
                writer.WriteNumber("result", 0);
                break;
            case ContractExpressionKind.Boolean:
                writer.WriteBoolean("bool", expression.Boolean);
                break;
            case ContractExpressionKind.Integer:
                writer.WritePropertyName("int");
                writer.WriteStartObject();
                writer.WriteString("decimal", RequiredText(expression));
                writer.WriteString("type", IntegerToken(expression.IntegerType));
                writer.WriteEndObject();
                break;
            case ContractExpressionKind.Unary:
            case ContractExpressionKind.Nary:
            case ContractExpressionKind.Binary:
                writer.WritePropertyName("args");
                writer.WriteStartArray();
                foreach (ContractExpression argument in expression.Arguments)
                {
                    WriteSidecarExpression(writer, argument);
                }

                writer.WriteEndArray();
                writer.WriteString("op", RequiredText(expression));
                break;
            default:
                throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
        }

        writer.WriteEndObject();
    }

    private static void WriteNormalizedExpressions(
        Utf8JsonWriter writer,
        string property,
        System.Collections.Generic.IReadOnlyList<NormalizedContractExpression> expressions)
    {
        writer.WritePropertyName(property);
        writer.WriteStartArray();
        foreach (NormalizedContractExpression expression in expressions)
        {
            WriteNormalizedExpression(writer, expression);
        }

        writer.WriteEndArray();
    }

    private static void WriteNormalizedExpression(
        Utf8JsonWriter writer,
        NormalizedContractExpression expression)
    {
        writer.WriteStartObject();
        switch (expression.Kind)
        {
            case NormalizedContractExpressionKind.Variable:
                writer.WriteString("var", "arg" + expression.Index.ToString(
                    System.Globalization.CultureInfo.InvariantCulture));
                break;
            case NormalizedContractExpressionKind.Result:
                writer.WriteNumber("result", 0);
                break;
            case NormalizedContractExpressionKind.Boolean:
                writer.WriteBoolean("bool", expression.Boolean);
                break;
            case NormalizedContractExpressionKind.Integer:
                writer.WritePropertyName("int");
                writer.WriteStartObject();
                writer.WriteBoolean("signed", SubsetTypeRules.IsSigned(expression.Type));
                writer.WriteString("value", RequiredText(expression));
                writer.WriteNumber(
                    "width",
                    expression.Type == SubsetValueType.I32 || expression.Type == SubsetValueType.U32
                        ? 32
                        : 64);
                writer.WriteEndObject();
                break;
            case NormalizedContractExpressionKind.Unary:
                writer.WriteString("op", RequiredText(expression));
                writer.WritePropertyName("value");
                WriteNormalizedExpression(writer, expression.Arguments[0]);
                break;
            case NormalizedContractExpressionKind.Nary:
                writer.WritePropertyName("args");
                writer.WriteStartArray();
                foreach (NormalizedContractExpression argument in expression.Arguments)
                {
                    WriteNormalizedExpression(writer, argument);
                }

                writer.WriteEndArray();
                writer.WriteString("op", RequiredText(expression));
                break;
            case NormalizedContractExpressionKind.Binary:
                writer.WritePropertyName("lhs");
                WriteNormalizedExpression(writer, expression.Arguments[0]);
                writer.WriteString("op", RequiredText(expression));
                writer.WritePropertyName("rhs");
                WriteNormalizedExpression(writer, expression.Arguments[1]);
                break;
            default:
                throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
        }

        writer.WriteEndObject();
    }

    internal static void WriteSemanticContext(Utf8JsonWriter writer)
    {
        writer.WritePropertyName("semantic_context");
        writer.WriteStartObject();
        writer.WriteString("profile_entry_sha256", FrontendConstants.ProfileEntrySha256);
        writer.WritePropertyName("profile_registry");
        writer.WriteStartObject();
        writer.WriteString("id", FrontendConstants.ProfileRegistryId);
        writer.WriteString("registry_sha256", FrontendConstants.ProfileRegistrySha256);
        writer.WriteNumber("revision", FrontendConstants.ProfileRegistryRevision);
        writer.WriteString("schema", FrontendConstants.ProfileRegistryId);
        writer.WriteEndObject();
        writer.WritePropertyName("semantic_parameters");
        writer.WriteStartObject();
        writer.WriteString("schema", FrontendConstants.SemanticParametersSchema);
        writer.WritePropertyName("value");
        writer.WriteStartObject();
        writer.WriteBoolean("check_overflow_default", false);
        writer.WriteString("documentation_mode", "none");
        writer.WriteString("language_version", "14.0");
        writer.WriteString("nullable_context", "disable");
        writer.WriteString("optimization", "release");
        writer.WriteString("platform", "x64");
        writer.WriteNumber("pointer_width", 64);
        writer.WritePropertyName("preprocessor_symbols");
        writer.WriteStartArray();
        writer.WriteEndArray();
        writer.WriteString("source_kind", "regular");
        writer.WriteString("target_framework", "net10.0");
        writer.WriteString("target_id", FrontendConstants.TargetId);
        writer.WriteBoolean("unsafe", false);
        writer.WriteEndObject();
        writer.WriteEndObject();
        writer.WriteString("semantic_profile", FrontendConstants.SemanticProfile);
        writer.WriteString("source_language", "csharp");
        writer.WriteEndObject();
    }

    private static string RequiredText(ContractExpression expression)
    {
        return expression.Text
            ?? throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
    }

    private static string RequiredText(NormalizedContractExpression expression)
    {
        return expression.Text
            ?? throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH");
    }

    private static string IntegerToken(SubsetValueType type)
    {
        return type switch
        {
            SubsetValueType.I32 => "i32",
            SubsetValueType.U32 => "u32",
            SubsetValueType.I64 => "i64",
            SubsetValueType.U64 => "u64",
            _ => throw FrontendFailure.Rejected("subset", "CSHARP_CONTRACT_HASH"),
        };
    }
}
