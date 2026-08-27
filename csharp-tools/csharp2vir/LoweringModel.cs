using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;

namespace Mpk.CSharp2Vir;

internal enum LoweredValueKind
{
    Variable,
    Boolean,
    Integer,
}

internal sealed class LoweredValue
{
    private LoweredValue(
        LoweredValueKind kind,
        SubsetValueType type,
        string? text,
        bool boolean)
    {
        Kind = kind;
        Type = type;
        Text = text;
        Boolean = boolean;
    }

    internal LoweredValueKind Kind { get; }

    internal SubsetValueType Type { get; }

    // Variable ID or canonical decimal according to Kind.
    internal string? Text { get; }

    internal bool Boolean { get; }

    internal static LoweredValue Variable(string id, SubsetValueType type)
    {
        return new LoweredValue(LoweredValueKind.Variable, type, id, false);
    }

    internal static LoweredValue BooleanLiteral(bool value)
    {
        return new LoweredValue(LoweredValueKind.Boolean, SubsetValueType.Bool, null, value);
    }

    internal static LoweredValue IntegerLiteral(string value, SubsetValueType type)
    {
        return new LoweredValue(LoweredValueKind.Integer, type, value, false);
    }
}

internal sealed class LoweredOrigin
{
    internal LoweredOrigin(string normalizedPath, int utf16Start, int utf16End)
    {
        NormalizedPath = normalizedPath;
        Utf16Start = utf16Start;
        Utf16End = utf16End;
    }

    internal string NormalizedPath { get; }

    internal int Utf16Start { get; }

    internal int Utf16End { get; }
}

internal sealed class LoweredBinding
{
    internal LoweredBinding(string id, SubsetValueType type)
    {
        Id = id;
        Type = type;
    }

    internal string Id { get; }

    internal SubsetValueType Type { get; }
}

internal enum LoweredInstructionKind
{
    Const,
    Copy,
    Unary,
    Binary,
    Convert,
}

internal enum LoweredUnaryOperator
{
    BoolNot,
    BvNeg,
    BvNot,
}

internal enum LoweredBinaryOperator
{
    Eq,
    NotEq,
    BvAdd,
    BvSub,
    BvMul,
    BvSdiv,
    BvSrem,
    BvUdiv,
    BvUrem,
    BvAnd,
    BvOr,
    BvXor,
    BvShl,
    BvAshr,
    BvLshr,
    SignedLt,
    SignedLe,
    SignedGt,
    SignedGe,
    UnsignedLt,
    UnsignedLe,
    UnsignedGt,
    UnsignedGe,
}

internal enum LoweredConversionForm
{
    None,
    Implicit,
    ExplicitUnchecked,
}

internal enum LoweredSafetyCheckKind
{
    IntegerNoOverflow,
    DivisorNonzero,
    SignedDivremRepresentable,
}

internal enum LoweredCheckOperation
{
    None,
    Add,
    Sub,
    Mul,
    Neg,
    Div,
    Rem,
}

internal sealed class LoweredSafetyCheck
{
    internal LoweredSafetyCheck(
        LoweredSafetyCheckKind kind,
        LoweredCheckOperation operation,
        int width,
        bool signed)
    {
        Kind = kind;
        Operation = operation;
        Width = width;
        Signed = signed;
    }

    internal LoweredSafetyCheckKind Kind { get; }

    internal LoweredCheckOperation Operation { get; }

    internal int Width { get; }

    internal bool Signed { get; }
}

internal sealed class LoweredInstruction
{
    private readonly ReadOnlyCollection<LoweredValue> operands;
    private readonly ReadOnlyCollection<LoweredSafetyCheck> safetyChecks;

    internal LoweredInstruction(
        string id,
        LoweredInstructionKind kind,
        SubsetValueType type,
        string? target,
        LoweredUnaryOperator unaryOperator,
        LoweredBinaryOperator binaryOperator,
        LoweredConversionForm conversionForm,
        ExplicitOverflowContext overflowContext,
        bool shiftCountMask,
        LoweredValue[] operands,
        LoweredSafetyCheck[] safetyChecks,
        LoweredOrigin origin)
    {
        Id = id;
        Kind = kind;
        Type = type;
        Target = target;
        UnaryOperator = unaryOperator;
        BinaryOperator = binaryOperator;
        ConversionForm = conversionForm;
        OverflowContext = overflowContext;
        IsShiftCountMask = shiftCountMask;
        this.operands = Array.AsReadOnly((LoweredValue[])operands.Clone());
        this.safetyChecks = Array.AsReadOnly((LoweredSafetyCheck[])safetyChecks.Clone());
        Origin = origin;
    }

    internal string Id { get; }

    internal LoweredInstructionKind Kind { get; }

    internal SubsetValueType Type { get; }

    internal string? Target { get; }

    internal LoweredUnaryOperator UnaryOperator { get; }

    internal LoweredBinaryOperator BinaryOperator { get; }

    internal LoweredConversionForm ConversionForm { get; }

    internal ExplicitOverflowContext OverflowContext { get; }

    internal bool IsShiftCountMask { get; }

    internal IReadOnlyList<LoweredValue> Operands => operands;

    internal IReadOnlyList<LoweredSafetyCheck> SafetyChecks => safetyChecks;

    internal LoweredOrigin Origin { get; }
}

internal sealed class LoweredRequiredCheck
{
    internal LoweredRequiredCheck(string instructionId, LoweredSafetyCheck check)
    {
        InstructionId = instructionId;
        Check = check;
    }

    internal string InstructionId { get; }

    internal LoweredSafetyCheck Check { get; }
}

internal enum LoweredTerminatorKind
{
    Return,
    Jump,
    Branch,
}

internal sealed class LoweredTerminator
{
    private readonly ReadOnlyCollection<LoweredValue> values;
    private readonly ReadOnlyCollection<LoweredValue> falseArguments;
    private readonly ReadOnlyCollection<LoweredValue> trueArguments;

    internal LoweredTerminator(
        LoweredTerminatorKind kind,
        LoweredValue? condition,
        string? falseTarget,
        LoweredValue[] falseArguments,
        string? trueTarget,
        LoweredValue[] trueArguments,
        LoweredValue[] values,
        LoweredOrigin origin)
    {
        Kind = kind;
        Condition = condition;
        FalseTarget = falseTarget;
        this.falseArguments = Array.AsReadOnly((LoweredValue[])falseArguments.Clone());
        TrueTarget = trueTarget;
        this.trueArguments = Array.AsReadOnly((LoweredValue[])trueArguments.Clone());
        this.values = Array.AsReadOnly((LoweredValue[])values.Clone());
        Origin = origin;
    }

    internal LoweredTerminatorKind Kind { get; }

    internal LoweredValue? Condition { get; }

    internal string? FalseTarget { get; }

    internal IReadOnlyList<LoweredValue> FalseArguments => falseArguments;

    internal string? TrueTarget { get; }

    internal IReadOnlyList<LoweredValue> TrueArguments => trueArguments;

    internal IReadOnlyList<LoweredValue> Values => values;

    internal LoweredOrigin Origin { get; }
}

internal sealed class LoweredBlock
{
    private readonly ReadOnlyCollection<LoweredBinding> parameters;
    private readonly ReadOnlyCollection<LoweredInstruction> instructions;

    internal LoweredBlock(
        string label,
        LoweredBinding[] parameters,
        LoweredInstruction[] instructions,
        LoweredTerminator terminator)
    {
        Label = label;
        this.parameters = Array.AsReadOnly((LoweredBinding[])parameters.Clone());
        this.instructions = Array.AsReadOnly((LoweredInstruction[])instructions.Clone());
        Terminator = terminator;
    }

    internal string Label { get; }

    internal IReadOnlyList<LoweredBinding> Parameters => parameters;

    internal IReadOnlyList<LoweredInstruction> Instructions => instructions;

    internal LoweredTerminator Terminator { get; }
}

internal enum LoweredFeature
{
    Branch,
    Conversion,
    MutableLocal,
}

internal sealed class LoweredFunction
{
    private readonly ReadOnlyCollection<LoweredBinding> parameters;
    private readonly ReadOnlyCollection<LoweredBinding> results;
    private readonly ReadOnlyCollection<LoweredBinding> locals;
    private readonly ReadOnlyCollection<LoweredBlock> blocks;
    private readonly ReadOnlyCollection<LoweredRequiredCheck> requiredChecks;
    private readonly ReadOnlyCollection<LoweredFeature> features;

    internal LoweredFunction(
        string id,
        LoweredBinding[] parameters,
        LoweredBinding[] results,
        LoweredBinding[] locals,
        LoweredBlock[] blocks,
        LoweredRequiredCheck[] requiredChecks,
        LoweredFeature[] features)
    {
        Id = id;
        this.parameters = Array.AsReadOnly((LoweredBinding[])parameters.Clone());
        this.results = Array.AsReadOnly((LoweredBinding[])results.Clone());
        this.locals = Array.AsReadOnly((LoweredBinding[])locals.Clone());
        this.blocks = Array.AsReadOnly((LoweredBlock[])blocks.Clone());
        this.requiredChecks = Array.AsReadOnly((LoweredRequiredCheck[])requiredChecks.Clone());
        this.features = Array.AsReadOnly((LoweredFeature[])features.Clone());
    }

    internal string Id { get; }

    internal IReadOnlyList<LoweredBinding> Parameters => parameters;

    internal IReadOnlyList<LoweredBinding> Results => results;

    internal IReadOnlyList<LoweredBinding> Locals => locals;

    internal IReadOnlyList<LoweredBlock> Blocks => blocks;

    internal IReadOnlyList<LoweredRequiredCheck> RequiredChecks => requiredChecks;

    internal IReadOnlyList<LoweredFeature> Features => features;
}

internal sealed class LoweredClosure
{
    private readonly ReadOnlyCollection<LoweredFunction> functions;

    internal LoweredClosure(
        string selectionSha256,
        LoweredFunction[] functions)
    {
        SelectionSha256 = selectionSha256;
        this.functions = Array.AsReadOnly((LoweredFunction[])functions.Clone());
    }

    internal string SelectionSha256 { get; }

    // Functions retain the closure's deterministic callee-first order.
    internal IReadOnlyList<LoweredFunction> Functions => functions;
}
