using System;
using System.IO;
using System.Security.Cryptography;
using System.Text;
using System.Text.Encodings.Web;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

internal static class EmissionCanonical
{
    internal static byte[] Write(Action<Utf8JsonWriter> write)
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
                    SkipValidation = false,
                }))
            {
                write(writer);
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
            throw EmissionFailure.Internal();
        }
    }

    internal static string Hash(string domain, ReadOnlySpan<byte> canonicalPayload)
    {
        try
        {
            byte[] prefix = Encoding.ASCII.GetBytes(domain + "\0");
            var preimage = new byte[checked(prefix.Length + canonicalPayload.Length)];
            Buffer.BlockCopy(prefix, 0, preimage, 0, prefix.Length);
            canonicalPayload.CopyTo(preimage.AsSpan(prefix.Length));
            return Convert.ToHexString(SHA256.HashData(preimage)).ToLowerInvariant();
        }
        catch (Exception error) when (
            error is ArgumentException
            || error is CryptographicException
            || error is EncoderFallbackException
            || error is OverflowException)
        {
            throw EmissionFailure.Internal();
        }
    }

    internal static void WriteRaw(Utf8JsonWriter writer, ReadOnlySpan<byte> canonicalValue)
    {
        if (canonicalValue.Length == 0)
        {
            throw EmissionFailure.Internal();
        }

        writer.WriteRawValue(canonicalValue, skipInputValidation: false);
    }

    internal static void WriteSemanticContext(Utf8JsonWriter writer)
    {
        ContractCanonical.WriteSemanticContext(writer);
    }

    internal static void WriteSelection(Utf8JsonWriter writer, Selection selection)
    {
        WriteRaw(writer, selection.CanonicalBytes);
    }

    internal static void WriteType(Utf8JsonWriter writer, SubsetValueType type)
    {
        writer.WriteStartObject();
        if (type == SubsetValueType.Bool)
        {
            writer.WriteString("kind", "bool");
        }
        else
        {
            writer.WriteString("kind", "bv");
            writer.WriteBoolean("signed", LoweringMethodBuilder.IsSigned(type));
            writer.WriteNumber("width", LoweringMethodBuilder.Width(type));
        }

        writer.WriteEndObject();
    }

    internal static void WriteBinding(Utf8JsonWriter writer, LoweredBinding binding)
    {
        writer.WriteStartObject();
        writer.WriteString("id", binding.Id);
        writer.WritePropertyName("type");
        WriteType(writer, binding.Type);
        writer.WriteEndObject();
    }

    internal static void WriteValue(Utf8JsonWriter writer, LoweredValue value)
    {
        writer.WriteStartObject();
        switch (value.Kind)
        {
            case LoweredValueKind.Variable:
                writer.WriteString("var", value.Text ?? throw EmissionFailure.Internal());
                break;
            case LoweredValueKind.Boolean:
                writer.WriteBoolean("bool", value.Boolean);
                break;
            case LoweredValueKind.Integer:
                writer.WritePropertyName("int");
                writer.WriteStartObject();
                writer.WriteBoolean("signed", LoweringMethodBuilder.IsSigned(value.Type));
                writer.WriteString("value", value.Text ?? throw EmissionFailure.Internal());
                writer.WriteNumber("width", LoweringMethodBuilder.Width(value.Type));
                writer.WriteEndObject();
                break;
            default:
                throw EmissionFailure.Internal();
        }

        writer.WriteEndObject();
    }
}
