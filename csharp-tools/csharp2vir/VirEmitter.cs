using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

internal static class CSharpVirEmitter
{
    internal static CanonicalArtifact Emit(
        Selection selection,
        LoweredClosure closure,
        ContractSet contracts)
    {
        LoweringValidator.Validate(closure);
        ValidateLinks(selection, closure, contracts);
        byte[] payload = WriteModule(selection, closure, contracts, null);
        string hash = EmissionCanonical.Hash(CSharpEmissionProfiles.VirHashDomain, payload);
        byte[] canonical = WriteModule(selection, closure, contracts, hash);
        return new CanonicalArtifact(CSharpEmissionProfiles.VirSchema, hash, canonical);
    }

    private static void ValidateLinks(
        Selection selection,
        LoweredClosure closure,
        ContractSet contracts)
    {
        if (!string.Equals(selection.Sha256, closure.SelectionSha256, StringComparison.Ordinal)
            || !string.Equals(selection.Sha256, contracts.SelectionSha256, StringComparison.Ordinal)
            || closure.Functions.Count != contracts.Contracts.Count)
        {
            throw EmissionFailure.Internal();
        }

        for (int index = 0; index < closure.Functions.Count; index++)
        {
            LoweredFunction function = closure.Functions[index];
            NormalizedContract contract = contracts.Contracts[index].Normalized;
            if (!string.Equals(function.Id, contract.FunctionId, StringComparison.Ordinal)
                || !string.Equals(function.ContractHash, contract.ContractHash, StringComparison.Ordinal)
                || !string.Equals(selection.Raw.Compilation, contract.UnitId, StringComparison.Ordinal))
            {
                throw EmissionFailure.Internal();
            }
        }
    }

    private static byte[] WriteModule(
        Selection selection,
        LoweredClosure closure,
        ContractSet contracts,
        string? hash)
    {
        return EmissionCanonical.Write(writer =>
        {
            writer.WriteStartObject();
            writer.WriteString("schema", CSharpEmissionProfiles.VirSchema);
            EmissionCanonical.WriteSemanticContext(writer);
            writer.WritePropertyName("units");
            writer.WriteStartArray();
            WriteUnit(writer, selection, closure, contracts);
            writer.WriteEndArray();
            if (hash is not null)
            {
                writer.WriteString("vir_hash", hash);
            }

            writer.WriteEndObject();
        });
    }

    private static void WriteUnit(
        Utf8JsonWriter writer,
        Selection selection,
        LoweredClosure closure,
        ContractSet contracts)
    {
        writer.WriteStartObject();
        writer.WritePropertyName("const_decls");
        writer.WriteStartArray();
        writer.WriteEndArray();
        writer.WritePropertyName("functions");
        writer.WriteStartArray();
        for (int index = 0; index < closure.Functions.Count; index++)
        {
            WriteFunction(
                writer,
                selection.Raw.Compilation,
                closure.Functions[index],
                contracts.Contracts[index].Normalized);
        }

        writer.WriteEndArray();
        writer.WriteString("id", selection.Raw.Compilation);
        writer.WriteString("name", selection.Raw.Compilation);
        writer.WritePropertyName("type_decls");
        writer.WriteStartArray();
        writer.WriteEndArray();
        writer.WriteEndObject();
    }

    private static void WriteFunction(
        Utf8JsonWriter writer,
        string unitId,
        LoweredFunction function,
        NormalizedContract contract)
    {
        writer.WriteStartObject();
        writer.WritePropertyName("blocks");
        writer.WriteStartArray();
        foreach (LoweredBlock block in function.Blocks)
        {
            WriteBlock(writer, block);
        }

        writer.WriteEndArray();
        writer.WritePropertyName("contracts");
        EmissionCanonical.WriteRaw(writer, contract.CanonicalBytes);
        writer.WritePropertyName("features_used");
        writer.WriteStartArray();
        foreach (LoweredFeature feature in function.Features)
        {
            writer.WriteStringValue(Feature(feature));
        }

        writer.WriteEndArray();
        writer.WriteString("id", function.Id);
        WriteBindings(writer, "locals", function.Locals);
        writer.WriteString("name", function.Name);
        WriteBindings(writer, "params", function.Parameters);
        WriteBindings(writer, "results", function.Results);
        writer.WriteString("unit_id", unitId);
        writer.WriteEndObject();
    }

    private static void WriteBindings(
        Utf8JsonWriter writer,
        string property,
        IReadOnlyList<LoweredBinding> bindings)
    {
        writer.WritePropertyName(property);
        writer.WriteStartArray();
        foreach (LoweredBinding binding in bindings)
        {
            EmissionCanonical.WriteBinding(writer, binding);
        }

        writer.WriteEndArray();
    }

    private static void WriteBlock(Utf8JsonWriter writer, LoweredBlock block)
    {
        writer.WriteStartObject();
        writer.WritePropertyName("instructions");
        writer.WriteStartArray();
        foreach (LoweredInstruction instruction in block.Instructions)
        {
            WriteInstruction(writer, instruction);
        }

        writer.WriteEndArray();
        writer.WriteString("label", block.Label);
        WriteBindings(writer, "parameters", block.Parameters);
        writer.WritePropertyName("terminator");
        WriteTerminator(writer, block.Terminator);
        writer.WriteEndObject();
    }

    private static void WriteInstruction(Utf8JsonWriter writer, LoweredInstruction instruction)
    {
        writer.WriteStartObject();
        switch (instruction.Kind)
        {
            case LoweredInstructionKind.Const:
                writer.WriteString("id", instruction.Id);
                writer.WriteString("kind", "Const");
                WriteSafetyChecks(writer, instruction.SafetyChecks);
                WriteInstructionType(writer, instruction.Type);
                writer.WritePropertyName("value");
                WriteLiteral(writer, instruction.Operands.Single());
                break;
            case LoweredInstructionKind.Copy:
                writer.WriteString("id", instruction.Id);
                writer.WriteString("kind", "Copy");
                WriteSafetyChecks(writer, instruction.SafetyChecks);
                writer.WriteString("target", instruction.Target);
                WriteInstructionType(writer, instruction.Type);
                writer.WritePropertyName("value");
                EmissionCanonical.WriteValue(writer, instruction.Operands.Single());
                break;
            case LoweredInstructionKind.Binary:
                writer.WriteString("id", instruction.Id);
                writer.WriteString("kind", "BinOp");
                writer.WritePropertyName("lhs");
                EmissionCanonical.WriteValue(writer, instruction.Operands[0]);
                writer.WriteString("op", BinaryOperator(instruction.BinaryOperator));
                writer.WritePropertyName("rhs");
                EmissionCanonical.WriteValue(writer, instruction.Operands[1]);
                WriteSafetyChecks(writer, instruction.SafetyChecks);
                WriteInstructionType(writer, instruction.Type);
                break;
            case LoweredInstructionKind.Unary:
                writer.WriteString("id", instruction.Id);
                writer.WriteString("kind", "UnaryOp");
                writer.WriteString("op", UnaryOperator(instruction.UnaryOperator));
                WriteSafetyChecks(writer, instruction.SafetyChecks);
                WriteInstructionType(writer, instruction.Type);
                writer.WritePropertyName("value");
                EmissionCanonical.WriteValue(writer, instruction.Operands.Single());
                break;
            case LoweredInstructionKind.Convert:
                writer.WriteString("id", instruction.Id);
                writer.WriteString("kind", "Convert");
                WriteSafetyChecks(writer, instruction.SafetyChecks);
                WriteInstructionType(writer, instruction.Type);
                writer.WritePropertyName("value");
                EmissionCanonical.WriteValue(writer, instruction.Operands.Single());
                break;
            case LoweredInstructionKind.CallStatic:
                writer.WritePropertyName("args");
                WriteValues(writer, instruction.Operands);
                writer.WriteString(
                    "contract_hash",
                    instruction.ContractHash ?? throw EmissionFailure.Internal());
                writer.WriteString(
                    "function",
                    instruction.Function ?? throw EmissionFailure.Internal());
                writer.WriteString("id", instruction.Id);
                writer.WriteString("kind", "CallStatic");
                WriteSafetyChecks(writer, instruction.SafetyChecks);
                WriteInstructionType(writer, instruction.Type);
                break;
            default:
                throw EmissionFailure.Internal();
        }

        writer.WriteEndObject();
    }

    private static void WriteInstructionType(Utf8JsonWriter writer, SubsetValueType type)
    {
        writer.WritePropertyName("type");
        EmissionCanonical.WriteType(writer, type);
    }

    private static void WriteLiteral(Utf8JsonWriter writer, LoweredValue value)
    {
        if (value.Kind == LoweredValueKind.Variable)
        {
            throw EmissionFailure.Internal();
        }

        EmissionCanonical.WriteValue(writer, value);
    }

    private static void WriteValues(
        Utf8JsonWriter writer,
        IReadOnlyList<LoweredValue> values)
    {
        writer.WriteStartArray();
        foreach (LoweredValue value in values)
        {
            EmissionCanonical.WriteValue(writer, value);
        }

        writer.WriteEndArray();
    }

    private static void WriteSafetyChecks(
        Utf8JsonWriter writer,
        IReadOnlyList<LoweredSafetyCheck> checks)
    {
        writer.WritePropertyName("safety_checks");
        writer.WriteStartArray();
        foreach (LoweredSafetyCheck check in checks)
        {
            writer.WriteStartObject();
            switch (check.Kind)
            {
                case LoweredSafetyCheckKind.IntegerNoOverflow:
                    writer.WriteString("kind", "integer_no_overflow");
                    writer.WriteString("operation", CheckOperation(check.Operation));
                    writer.WriteBoolean("signed", check.Signed);
                    break;
                case LoweredSafetyCheckKind.DivisorNonzero:
                    writer.WriteString("kind", "divisor_nonzero");
                    break;
                case LoweredSafetyCheckKind.SignedDivremRepresentable:
                    writer.WriteString("kind", "signed_divrem_representable");
                    writer.WriteString("operation", CheckOperation(check.Operation));
                    break;
                default:
                    throw EmissionFailure.Internal();
            }

            writer.WriteEndObject();
        }

        writer.WriteEndArray();
    }

    private static void WriteTerminator(
        Utf8JsonWriter writer,
        LoweredTerminator terminator)
    {
        writer.WriteStartObject();
        switch (terminator.Kind)
        {
            case LoweredTerminatorKind.Return:
                writer.WriteString("kind", "Return");
                writer.WritePropertyName("values");
                WriteValues(writer, terminator.Values);
                break;
            case LoweredTerminatorKind.Jump:
                writer.WritePropertyName("args");
                WriteValues(writer, terminator.FalseArguments);
                writer.WriteString("kind", "Jump");
                writer.WriteString(
                    "label",
                    terminator.FalseTarget ?? throw EmissionFailure.Internal());
                break;
            case LoweredTerminatorKind.Branch:
                writer.WritePropertyName("cond");
                EmissionCanonical.WriteValue(
                    writer,
                    terminator.Condition ?? throw EmissionFailure.Internal());
                writer.WritePropertyName("else_args");
                WriteValues(writer, terminator.FalseArguments);
                writer.WriteString(
                    "else_label",
                    terminator.FalseTarget ?? throw EmissionFailure.Internal());
                writer.WriteString("kind", "Branch");
                writer.WritePropertyName("then_args");
                WriteValues(writer, terminator.TrueArguments);
                writer.WriteString(
                    "then_label",
                    terminator.TrueTarget ?? throw EmissionFailure.Internal());
                break;
            default:
                throw EmissionFailure.Internal();
        }

        writer.WriteEndObject();
    }

    private static string Feature(LoweredFeature feature)
    {
        return feature switch
        {
            LoweredFeature.Branch => "branch",
            LoweredFeature.CallStatic => "call_static",
            LoweredFeature.Conversion => "conversion",
            LoweredFeature.MutableLocal => "mutable_local",
            _ => throw EmissionFailure.Internal(),
        };
    }

    private static string UnaryOperator(LoweredUnaryOperator operation)
    {
        return operation switch
        {
            LoweredUnaryOperator.BoolNot => "not",
            LoweredUnaryOperator.BvNeg => "bv_neg",
            LoweredUnaryOperator.BvNot => "bv_not",
            _ => throw EmissionFailure.Internal(),
        };
    }

    private static string BinaryOperator(LoweredBinaryOperator operation)
    {
        return operation switch
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
            _ => throw EmissionFailure.Internal(),
        };
    }

    private static string CheckOperation(LoweredCheckOperation operation)
    {
        return operation switch
        {
            LoweredCheckOperation.Add => "add",
            LoweredCheckOperation.Sub => "sub",
            LoweredCheckOperation.Mul => "mul",
            LoweredCheckOperation.Neg => "neg",
            LoweredCheckOperation.Div => "div",
            LoweredCheckOperation.Rem => "rem",
            _ => throw EmissionFailure.Internal(),
        };
    }
}
