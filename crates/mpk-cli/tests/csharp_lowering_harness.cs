using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.Operations;

namespace Mpk.CSharp2Vir;

internal static class LoweringHarness
{
    private const string SourcePath = "src/Case.cs";
    private const string ContractPath = "contracts/case.json";

    public static int Main(string[] args)
    {
        if (args.Length != 1)
        {
            Console.Error.Write("CSHARP_LOWERING_TEST_USAGE\n");
            return 1;
        }

        try
        {
            string referencePackRoot = args[0];
            TypeMappingsAreExact(referencePackRoot);
            ConstantsAreExplicitAndBounded(referencePackRoot);
            NonCallOperationMappingsAreComplete(referencePackRoot);
            RoslynCheckedStatesAreExact(referencePackRoot);
            ConversionRulesAreExact(referencePackRoot);
            ControlFlowAndEvaluationAreDeterministic(referencePackRoot);
            RequiredChecksAreExactAndClosed(referencePackRoot);
            SemanticRowsAreOwned(referencePackRoot);
            CallStaticIsT10Owned(referencePackRoot);
            return 0;
        }
        catch (HarnessFailure failure)
        {
            Console.Error.Write("CSHARP_LOWERING_TEST_" + failure.Code + "\n");
            return 1;
        }
        catch (Exception)
        {
            Console.Error.Write("CSHARP_LOWERING_TEST_UNEXPECTED\n");
            return 1;
        }
    }

    private static void ConstantsAreExplicitAndBounded(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\npublic static class Constants\n{\n"
            + "    public static long F()\n"
            + "    {\n"
            + "        int x = -2147483648;\n"
            + "        return x == -2147483648 ? -9223372036854775808L : 0L;\n"
            + "    }\n"
            + "}\n";
        const string method = "Vector.Constants::F()->i64";
        LoweredFunction function = Lower(
            referencePackRoot,
            "literal-negative-min",
            source,
            method).Function(method);
        LoweredInstruction[] constants = function.Blocks
            .SelectMany(block => block.Instructions)
            .Where(instruction => instruction.Kind == LoweredInstructionKind.Const)
            .ToArray();
        Equal(4, constants.Length, "CONSTANT_COUNT");
        Equal(
            "-2147483648,-2147483648,0,-9223372036854775808",
            string.Join(',', constants.Select(constant => constant.Operands.Single().Text)),
            "CONSTANT_VALUES");

        LoweredInstruction first = constants[0];
        ExpectLoweringRejected(
            () => LoweringValidator.Validate(ReplaceInstruction(
                function,
                first.Id,
                CloneInstruction(
                    first,
                    operands: new[]
                    {
                        LoweredValue.IntegerLiteral("2147483648", SubsetValueType.I32),
                    }))),
            "CSHARP_LOWERING_OPERATION");
    }

    private static void TypeMappingsAreExact(string referencePackRoot)
    {
        var source = new StringBuilder("namespace Vector;\npublic static class Types\n{\n");
        var records = new[]
        {
            (Source: "bool", Token: "bool", Type: SubsetValueType.Bool),
            (Source: "int", Token: "i32", Type: SubsetValueType.I32),
            (Source: "uint", Token: "u32", Type: SubsetValueType.U32),
            (Source: "long", Token: "i64", Type: SubsetValueType.I64),
            (Source: "ulong", Token: "u64", Type: SubsetValueType.U64),
        };
        var methods = new List<string>();
        for (int index = 0; index < records.Length; index++)
        {
            (string sourceType, string token, _) = records[index];
            source.Append(
                $"    public static {sourceType} T{index}({sourceType} x) "
                    + $"{{ {sourceType} y = x; return y; }}\n");
            methods.Add($"Vector.Types::T{index}({token})->{token}");
        }

        source.Append("}\n");
        LoweringContext context = Lower(
            referencePackRoot,
            "type-mappings",
            source.ToString(),
            methods.ToArray());
        for (int index = 0; index < records.Length; index++)
        {
            LoweredFunction function = context.Function(methods[index]);
            Equal(records[index].Type, function.Parameters.Single().Type, "TYPE_PARAMETER_" + index);
            Equal(records[index].Type, function.Results.Single().Type, "TYPE_RESULT_" + index);
            Equal(records[index].Type, function.Locals.Single().Type, "TYPE_LOCAL_" + index);
            Equal("Copy", string.Join(',', InstructionTrace(function)), "TYPE_COPY_" + index);
        }
    }

    private static void NonCallOperationMappingsAreComplete(string referencePackRoot)
    {
        var cases = new[]
        {
            new OperationCase("bool F00(bool x) { return !x; }", "Vector.Operations::F00(bool)->bool", "bool_not"),
            new OperationCase("bool F01(bool x, bool y) { return x == y; }", "Vector.Operations::F01(bool,bool)->bool", "eq"),
            new OperationCase("bool F02(int x, int y) { return x != y; }", "Vector.Operations::F02(i32,i32)->bool", "not_eq"),
            new OperationCase("bool F03(int x, int y) { return x < y; }", "Vector.Operations::F03(i32,i32)->bool", "signed_lt"),
            new OperationCase("bool F04(long x, long y) { return x <= y; }", "Vector.Operations::F04(i64,i64)->bool", "signed_le"),
            new OperationCase("bool F05(int x, int y) { return x > y; }", "Vector.Operations::F05(i32,i32)->bool", "signed_gt"),
            new OperationCase("bool F06(long x, long y) { return x >= y; }", "Vector.Operations::F06(i64,i64)->bool", "signed_ge"),
            new OperationCase("bool F07(uint x, uint y) { return x < y; }", "Vector.Operations::F07(u32,u32)->bool", "unsigned_lt"),
            new OperationCase("bool F08(ulong x, ulong y) { return x <= y; }", "Vector.Operations::F08(u64,u64)->bool", "unsigned_le"),
            new OperationCase("bool F09(uint x, uint y) { return x > y; }", "Vector.Operations::F09(u32,u32)->bool", "unsigned_gt"),
            new OperationCase("bool F10(ulong x, ulong y) { return x >= y; }", "Vector.Operations::F10(u64,u64)->bool", "unsigned_ge"),
            new OperationCase("int F11(int x, int y) { return checked(x + y); }", "Vector.Operations::F11(i32,i32)->i32", "bv_add"),
            new OperationCase("uint F12(uint x, uint y) { return checked(x - y); }", "Vector.Operations::F12(u32,u32)->u32", "bv_sub"),
            new OperationCase("long F13(long x, long y) { return checked(x * y); }", "Vector.Operations::F13(i64,i64)->i64", "bv_mul"),
            new OperationCase("ulong F14(ulong x, ulong y) { return unchecked(x + y); }", "Vector.Operations::F14(u64,u64)->u64", "bv_add"),
            new OperationCase("int F15(int x, int y) { return unchecked(x - y); }", "Vector.Operations::F15(i32,i32)->i32", "bv_sub"),
            new OperationCase("uint F16(uint x, uint y) { return unchecked(x * y); }", "Vector.Operations::F16(u32,u32)->u32", "bv_mul"),
            new OperationCase("int F17(int x) { return checked(-x); }", "Vector.Operations::F17(i32)->i32", "bv_neg"),
            new OperationCase("long F18(long x) { return unchecked(-x); }", "Vector.Operations::F18(i64)->i64", "bv_neg"),
            new OperationCase("int F19(int x, int y) { return checked(x / y); }", "Vector.Operations::F19(i32,i32)->i32", "bv_sdiv"),
            new OperationCase("long F20(long x, long y) { return unchecked(x % y); }", "Vector.Operations::F20(i64,i64)->i64", "bv_srem"),
            new OperationCase("uint F21(uint x, uint y) { return checked(x / y); }", "Vector.Operations::F21(u32,u32)->u32", "bv_udiv"),
            new OperationCase("ulong F22(ulong x, ulong y) { return unchecked(x % y); }", "Vector.Operations::F22(u64,u64)->u64", "bv_urem"),
            new OperationCase("int F23(int x) { return ~x; }", "Vector.Operations::F23(i32)->i32", "bv_not"),
            new OperationCase("uint F24(uint x, uint y) { return x & y; }", "Vector.Operations::F24(u32,u32)->u32", "bv_and"),
            new OperationCase("long F25(long x, long y) { return x | y; }", "Vector.Operations::F25(i64,i64)->i64", "bv_or"),
            new OperationCase("ulong F26(ulong x, ulong y) { return x ^ y; }", "Vector.Operations::F26(u64,u64)->u64", "bv_xor"),
            new OperationCase("int F27(int x, int n) { return x << n; }", "Vector.Operations::F27(i32,i32)->i32", "bv_and(count,31)", "bv_shl"),
            new OperationCase("long F28(long x, int n) { return x << n; }", "Vector.Operations::F28(i64,i32)->i64", "bv_and(count,63)", "bv_shl"),
            new OperationCase("long F29(long x, int n) { return x >> n; }", "Vector.Operations::F29(i64,i32)->i64", "bv_and(count,63)", "bv_ashr"),
            new OperationCase("uint F30(uint x, int n) { return x >> n; }", "Vector.Operations::F30(u32,i32)->u32", "bv_and(count,31)", "bv_lshr"),
            new OperationCase("bool F31(bool x, bool y) { return x && y; }", "Vector.Operations::F31(bool,bool)->bool"),
            new OperationCase("bool F32(bool x, bool y) { return x || y; }", "Vector.Operations::F32(bool,bool)->bool"),
            new OperationCase("int F33(bool c, int x, int y) { return c ? x : y; }", "Vector.Operations::F33(bool,i32,i32)->i32"),
        };
        Equal(34, cases.Length, "OPERATION_CASE_COUNT");

        var functions = new Dictionary<string, LoweredFunction>(StringComparer.Ordinal);
        for (int chunkStart = 0; chunkStart < cases.Length; chunkStart += 17)
        {
            OperationCase[] chunk = cases.Skip(chunkStart).Take(17).ToArray();
            var source = new StringBuilder("namespace Vector;\npublic static class Operations\n{\n");
            foreach (OperationCase operation in chunk)
            {
                source.Append("    public static ").Append(operation.Source).Append('\n');
            }

            source.Append("}\n");
            LoweringContext context = Lower(
                referencePackRoot,
                "operation-mappings-" + chunkStart.ToString(CultureInfo.InvariantCulture),
                source.ToString(),
                chunk.Select(operation => operation.Method).ToArray());
            foreach (OperationCase operation in chunk)
            {
                functions.Add(operation.Method, context.Function(operation.Method));
            }
        }

        for (int index = 0; index < 31; index++)
        {
            Equal(
                string.Join(',', cases[index].ExpectedInstructions),
                string.Join(',', InstructionTrace(functions[cases[index].Method])),
                "OPERATION_MAPPING_" + index);
        }

        for (int index = 31; index < cases.Length; index++)
        {
            LoweredFunction function = functions[cases[index].Method];
            Check(
                function.Blocks.Any(block => block.Terminator.Kind == LoweredTerminatorKind.Branch),
                "OPERATION_BRANCH_" + index);
            Check(
                function.Blocks.Sum(block => block.Parameters.Count) == 1,
                "OPERATION_PARAMETER_" + index);
        }

        Check(
            functions[cases[33].Method].Blocks.Any(block =>
                block.Terminator.Kind == LoweredTerminatorKind.Jump),
            "OPERATION_CONDITIONAL_JUMP");
    }

    private static void RoslynCheckedStatesAreExact(string referencePackRoot)
    {
        var records = new[]
        {
            new CheckedCase("Add", "+", "checked", true, new[] { "int", "uint", "long", "ulong" }),
            new CheckedCase("Subtract", "-", "checked", true, new[] { "int", "uint", "long", "ulong" }),
            new CheckedCase("Multiply", "*", "checked", true, new[] { "int", "uint", "long", "ulong" }),
            new CheckedCase("Minus", "unary-", "checked", true, new[] { "int", "long" }),
            new CheckedCase("Divide", "/", "checked", true, new[] { "int", "uint", "long", "ulong" }),
            new CheckedCase("Remainder", "%", "checked", false, new[] { "int", "uint", "long", "ulong" }),
            new CheckedCase("Add", "+", "unchecked", false, new[] { "int", "uint", "long", "ulong" }),
            new CheckedCase("Subtract", "-", "unchecked", false, new[] { "int", "uint", "long", "ulong" }),
            new CheckedCase("Multiply", "*", "unchecked", false, new[] { "int", "uint", "long", "ulong" }),
            new CheckedCase("Minus", "unary-", "unchecked", false, new[] { "int", "long" }),
            new CheckedCase("Divide", "/", "unchecked", false, new[] { "int", "uint", "long", "ulong" }),
            new CheckedCase("Remainder", "%", "unchecked", false, new[] { "int", "uint", "long", "ulong" }),
        };
        Equal(12, records.Length, "CHECKED_STATE_CASE_COUNT");

        var methods = new List<string>();
        var expected = new List<(string Method, CheckedCase Case, string Declaration)>();
        int methodIndex = 0;
        foreach (CheckedCase record in records)
        {
            foreach (string sourceType in record.OperandTypes)
            {
                string token = TypeToken(sourceType);
                string name = "C" + methodIndex.ToString("D2", CultureInfo.InvariantCulture);
                string expression = record.Source == "unary-"
                    ? $"{record.Context}(-x)"
                    : $"{record.Context}(x {record.Source} y)";
                string parameters = record.Source == "unary-"
                    ? $"{sourceType} x"
                    : $"{sourceType} x, {sourceType} y";
                string method = record.Source == "unary-"
                    ? $"Vector.CheckedStates::{name}({token})->{token}"
                    : $"Vector.CheckedStates::{name}({token},{token})->{token}";
                string declaration = $"    public static {sourceType} {name}({parameters}) "
                    + $"{{ return {expression}; }}\n";
                methods.Add(method);
                expected.Add((method, record, declaration));
                methodIndex++;
            }
        }

        Equal(44, methods.Count, "CHECKED_STATE_EXPANSION_COUNT");
        for (int chunkStart = 0; chunkStart < expected.Count; chunkStart += 22)
        {
            (string Method, CheckedCase Case, string Declaration)[] chunk = expected
                .Skip(chunkStart)
                .Take(22)
                .ToArray();
            var source = new StringBuilder("namespace Vector;\npublic static class CheckedStates\n{\n");
            foreach (var item in chunk)
            {
                source.Append(item.Declaration);
            }

            source.Append("}\n");
            LoweringContext context = Lower(
                referencePackRoot,
                "checked-states-" + chunkStart.ToString(CultureInfo.InvariantCulture),
                source.ToString(),
                chunk.Select(item => item.Method).ToArray());
            foreach (var item in chunk)
            {
                string methodId = item.Method;
                CheckedCase record = item.Case;
                SubsetMethod subset = context.SubsetMethod(methodId);
                IOperation arithmetic = OperationTree(subset.Body)
                    .Single(operation => IsCheckedOperation(operation, record.OperatorKind));
                bool actual = arithmetic switch
                {
                    IBinaryOperation binary => binary.IsChecked,
                    IUnaryOperation unary => unary.IsChecked,
                    _ => throw new HarnessFailure("CHECKED_STATE_KIND"),
                };
                Equal(record.ExpectedIsChecked, actual, "CHECKED_STATE_" + methodId);

                LoweredInstruction lowered = context.Function(methodId).Blocks
                    .SelectMany(block => block.Instructions)
                    .Single(instruction => instruction.Kind == LoweredInstructionKind.Binary
                        || instruction.Kind == LoweredInstructionKind.Unary);
                Equal(
                    record.Context == "checked"
                        ? ExplicitOverflowContext.Checked
                        : ExplicitOverflowContext.Unchecked,
                    lowered.OverflowContext,
                    "CHECKED_CONTEXT_" + methodId);
            }
        }
    }

    private static void ConversionRulesAreExact(string referencePackRoot)
    {
        var records = new List<ConversionCase>
        {
            new ConversionCase("bool", "bool", "identity"),
            new ConversionCase("int", "int", "identity"),
            new ConversionCase("uint", "uint", "identity"),
            new ConversionCase("long", "long", "identity"),
            new ConversionCase("ulong", "ulong", "identity"),
            new ConversionCase("int", "long", "implicit"),
            new ConversionCase("uint", "long", "implicit"),
            new ConversionCase("uint", "ulong", "implicit"),
        };
        foreach (string sourceType in new[] { "int", "uint", "long", "ulong" })
        {
            foreach (string destinationType in new[] { "int", "uint", "long", "ulong" })
            {
                if (!string.Equals(sourceType, destinationType, StringComparison.Ordinal))
                {
                    records.Add(new ConversionCase(sourceType, destinationType, "explicit"));
                }
            }
        }

        Equal(20, records.Count, "CONVERSION_RULE_COUNT");
        var source = new StringBuilder("namespace Vector;\npublic static class Conversions\n{\n");
        var methods = new List<string>();
        for (int index = 0; index < records.Count; index++)
        {
            ConversionCase record = records[index];
            string name = "C" + index.ToString("D2", CultureInfo.InvariantCulture);
            string body = record.Form switch
            {
                "identity" => $"return ({record.DestinationType})x;",
                "implicit" => $"{record.DestinationType} y = x; return y;",
                "explicit" => $"return unchecked(({record.DestinationType})x);",
                _ => throw new HarnessFailure("CONVERSION_FORM"),
            };
            string method = $"Vector.Conversions::{name}({TypeToken(record.SourceType)})"
                + $"->{TypeToken(record.DestinationType)}";
            string declaration =
                $"    public static {record.DestinationType} {name}({record.SourceType} x) "
                    + $"{{ {body} }}\n";
            source.Append(declaration);
            methods.Add(method);
        }

        source.Append("}\n");
        LoweringContext context = Lower(
            referencePackRoot,
            "conversion-rules",
            source.ToString(),
            methods.ToArray());
        for (int index = 0; index < records.Count; index++)
        {
            LoweredInstruction[] conversions = context.Function(methods[index]).Blocks
                .SelectMany(block => block.Instructions)
                .Where(instruction => instruction.Kind == LoweredInstructionKind.Convert)
                .ToArray();
            if (records[index].Form == "identity")
            {
                Equal(0, conversions.Length, "CONVERSION_IDENTITY_" + index);
                continue;
            }

            Equal(1, conversions.Length, "CONVERSION_COUNT_" + index);
            Equal(
                records[index].Form == "implicit"
                    ? LoweredConversionForm.Implicit
                    : LoweredConversionForm.ExplicitUnchecked,
                conversions[0].ConversionForm,
                "CONVERSION_LOWERED_FORM_" + index);
            Equal(
                SourceType(records[index].DestinationType),
                conversions[0].Type,
                "CONVERSION_DESTINATION_" + index);
            Equal(
                SourceType(records[index].SourceType),
                conversions[0].Operands.Single().Type,
                "CONVERSION_SOURCE_" + index);
            Equal(0, conversions[0].SafetyChecks.Count, "CONVERSION_CHECKS_" + index);
        }
    }

    private static void ControlFlowAndEvaluationAreDeterministic(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\npublic static class Control\n{\n"
            + "    public static int Copy(int x) { int y = x; y = unchecked(y + 1); return y; }\n"
            + "    public static int Join(bool c, int a, int b) { int y = a; if (c) { y = a; } else { y = b; } return y; }\n"
            + "    public static int Early(bool c, int a, int b) { if (c) { return a; } return b; }\n"
            + "    public static int Conditional(bool c, int a, int b) { return c ? a : b; }\n"
            + "    public static bool And(bool a, bool b) { return a && b; }\n"
            + "    public static bool Or(bool a, bool b) { return a || b; }\n"
            + "    public static int Order(int a, int b, int c, int d) { return unchecked((a + b) * (c - d)); }\n"
            + "}\n";
        string[] methods =
        {
            "Vector.Control::And(bool,bool)->bool",
            "Vector.Control::Conditional(bool,i32,i32)->i32",
            "Vector.Control::Copy(i32)->i32",
            "Vector.Control::Early(bool,i32,i32)->i32",
            "Vector.Control::Join(bool,i32,i32)->i32",
            "Vector.Control::Or(bool,bool)->bool",
            "Vector.Control::Order(i32,i32,i32,i32)->i32",
        };
        LoweringContext first = Lower(referencePackRoot, "control-flow", source, methods);
        LoweringContext second = Lower(referencePackRoot, "control-flow", source, methods);
        foreach (string method in methods)
        {
            Equal(
                Fingerprint(first.Function(method)),
                Fingerprint(second.Function(method)),
                "CONTROL_REPEAT_" + method);
        }

        Equal(
            "Copy,Const,bv_add,Copy",
            string.Join(',', InstructionTrace(first.Function(methods[2]))),
            "CONTROL_COPY_TRACE");
        Equal(
            "bv_add,bv_sub,bv_mul",
            string.Join(',', InstructionTrace(first.Function(methods[6]))),
            "CONTROL_LEFT_TO_RIGHT");

        LoweredFunction join = first.Function(methods[4]);
        Equal(4, join.Blocks.Count, "CONTROL_JOIN_BLOCKS");
        Equal(LoweredTerminatorKind.Branch, join.Blocks[0].Terminator.Kind, "CONTROL_JOIN_BRANCH");
        Equal("arg2", CopyOperand(join.Blocks[1]), "CONTROL_FALSE_FIRST");
        Equal("arg1", CopyOperand(join.Blocks[2]), "CONTROL_TRUE_SECOND");
        Equal(1, join.Blocks[3].Parameters.Count, "CONTROL_JOIN_PARAMETER");
        Equal("p0", join.Blocks[3].Parameters[0].Id, "CONTROL_PARAMETER_ID");

        LoweredFunction early = first.Function(methods[3]);
        Equal(
            2,
            early.Blocks.Count(block => block.Terminator.Kind == LoweredTerminatorKind.Return),
            "CONTROL_EARLY_RETURNS");
        Check(
            first.Function(methods[0]).Blocks.Sum(block => block.Parameters.Count) == 1,
            "CONTROL_AND_JOIN");
        Check(
            first.Function(methods[5]).Blocks.Sum(block => block.Parameters.Count) == 1,
            "CONTROL_OR_JOIN");
        Check(
            first.Function(methods[1]).Blocks.Any(block =>
                block.Terminator.Kind == LoweredTerminatorKind.Jump),
            "CONTROL_CONDITIONAL_JUMP");
    }

    private static void RequiredChecksAreExactAndClosed(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\npublic static class Checks\n{\n"
            + "    public static int F(int x, int y)\n"
            + "    {\n"
            + "        checked\n"
            + "        {\n"
            + "            int a = x + y;\n"
            + "            int s = a - y;\n"
            + "            int m = s * y;\n"
            + "            int n = -m;\n"
            + "            int d = n / y;\n"
            + "            int r = n % y;\n"
            + "            return unchecked(d + r);\n"
            + "        }\n"
            + "    }\n"
            + "}\n";
        const string method = "Vector.Checks::F(i32,i32)->i32";
        LoweredFunction function = Lower(
            referencePackRoot,
            "required-checks",
            source,
            method).Function(method);
        Equal(
            "IntegerNoOverflow:Add,IntegerNoOverflow:Sub,IntegerNoOverflow:Mul,"
                + "IntegerNoOverflow:Neg,DivisorNonzero:Div,DivisorNonzero:Rem,"
                + "SignedDivremRepresentable:Div,SignedDivremRepresentable:Rem",
            string.Join(',', function.RequiredChecks.Select(check =>
                check.Check.Kind + ":" + check.Check.Operation)),
            "CHECK_CANONICAL_ORDER");
        Check(function.RequiredChecks.All(check =>
            check.Check.Width == 32 && check.Check.Signed), "CHECK_TYPE_CONTEXT");

        LoweredInstruction division = function.Blocks
            .SelectMany(block => block.Instructions)
            .Single(instruction => instruction.BinaryOperator == LoweredBinaryOperator.BvSdiv);
        ExpectLoweringRejected(
            () => LoweringValidator.Validate(ReplaceInstruction(
                function,
                division.Id,
                CloneInstruction(division, checks: division.SafetyChecks.Skip(1).ToArray()))),
            "CSHARP_LOWERING_CHECK_MISSING");
        ExpectLoweringRejected(
            () => LoweringValidator.Validate(ReplaceInstruction(
                function,
                division.Id,
                CloneInstruction(
                    division,
                    checks: division.SafetyChecks
                        .Concat(new[] { division.SafetyChecks[0] })
                        .ToArray()))),
            "CSHARP_LOWERING_CHECK_EXTRA");
        ExpectLoweringRejected(
            () => LoweringValidator.Validate(ReplaceInstruction(
                function,
                division.Id,
                CloneInstruction(division, checks: division.SafetyChecks.Reverse().ToArray()))),
            "CSHARP_LOWERING_CHECK_ORDER");
        ExpectLoweringRejected(
            () => LoweringValidator.Validate(ReplaceInstruction(
                function,
                division.Id,
                CloneInstruction(
                    division,
                    binaryOperator: LoweredBinaryOperator.SignedLt,
                    checks: Array.Empty<LoweredSafetyCheck>()))),
            "CSHARP_LOWERING_OPERATION");
        ExpectLoweringRejected(
            () => LoweringValidator.Validate(CloneFunction(
                function,
                blocks: Array.Empty<LoweredBlock>())),
            "CSHARP_LOWERING_CFG");
        ExpectLoweringRejected(
            () => LoweringValidator.Validate(CloneFunction(
                function,
                requiredChecks: function.RequiredChecks.Skip(1).ToArray())),
            "CSHARP_LOWERING_CHECK_MISSING");
        ExpectLoweringRejected(
            () => LoweringValidator.Validate(CloneFunction(
                function,
                requiredChecks: function.RequiredChecks
                    .Concat(new[] { function.RequiredChecks[0] })
                    .ToArray())),
            "CSHARP_LOWERING_CHECK_EXTRA");
        ExpectLoweringRejected(
            () => LoweringValidator.Validate(CloneFunction(
                function,
                requiredChecks: function.RequiredChecks.Reverse().ToArray())),
            "CSHARP_LOWERING_CHECK_ORDER");
    }

    private static void SemanticRowsAreOwned(string referencePackRoot)
    {
        // The executable checks above establish each row before it enters this
        // closed ownership set; T11 later aggregates all 34 rows.
        var rows = new HashSet<string>(StringComparer.Ordinal)
        {
            "M01", "M02", "M07", "M08", "M09", "M10", "M11", "M12",
            "M13", "M14", "M16", "M18", "M19", "M21", "M29",
        };
        Equal(
            "M01,M02,M07,M08,M09,M10,M11,M12,M13,M14,M16,M18,M19,M21,M29",
            string.Join(',', rows.OrderBy(row => row, StringComparer.Ordinal)),
            "SEMANTIC_ROW_SET");

        // Keep this owner executable rather than a prose-only row claim.
        const string source =
            "namespace Vector;\npublic static class Rows\n{\n"
            + "    public static uint F(bool c, uint x, uint y, int n)\n"
            + "    {\n"
            + "        uint value = unchecked(x + y);\n"
            + "        if (c && x < y) { value = value << n; }\n"
            + "        return value;\n"
            + "    }\n"
            + "}\n";
        const string method = "Vector.Rows::F(bool,u32,u32,i32)->u32";
        LoweredFunction lowered = Lower(
            referencePackRoot,
            "semantic-rows",
            source,
            method).Function(method);
        Check(lowered.Features.Contains(LoweredFeature.Branch), "SEMANTIC_ROWS_BRANCH");
        Check(lowered.Features.Contains(LoweredFeature.MutableLocal), "SEMANTIC_ROWS_LOCAL");
        Check(InstructionTrace(lowered).Contains("bv_add"), "SEMANTIC_ROWS_ARITHMETIC");
        Check(InstructionTrace(lowered).Contains("unsigned_lt"), "SEMANTIC_ROWS_COMPARISON");
        Check(InstructionTrace(lowered).Contains("bv_shl"), "SEMANTIC_ROWS_SHIFT");
    }

    private static void CallStaticIsT10Owned(string referencePackRoot)
    {
        const string source =
            "namespace Vector;\npublic static class Calls\n{\n"
            + "    public static int F(int x) { return G(x); }\n"
            + "    private static int G(int x) { return x; }\n"
            + "}\n";
        LoweringContext context = Lower(
            referencePackRoot,
            "call-boundary",
            source,
            "Vector.Calls::F(i32)->i32");
        LoweredInstruction call = context.Function("Vector.Calls::F(i32)->i32")
            .Blocks.SelectMany(block => block.Instructions)
            .Single(instruction => instruction.Kind == LoweredInstructionKind.CallStatic);
        Equal("Vector.Calls::G(i32)->i32", call.Function, "CALLSTATIC_T10_FUNCTION");
        Check(
            context.Function("Vector.Calls::F(i32)->i32").Features.Contains(
                LoweredFeature.CallStatic),
            "CALLSTATIC_T10_FEATURE");
    }

    private static LoweringContext Lower(
        string referencePackRoot,
        string compilation,
        string source,
        params string[] methods)
    {
        LoweringContext context = Compile(referencePackRoot, compilation, source, methods);
        context.Lowered = CSharpLowering.Lower(context.Selection, context.Closure, context.Contracts);
        return context;
    }

    private static LoweringContext Compile(
        string referencePackRoot,
        string compilation,
        string source,
        params string[] methods)
    {
        Array.Sort(methods, StringComparer.Ordinal);
        Selection selection = SelectionCodec.Validate(new RawSelection(
            compilation,
            new[] { SourcePath },
            new[] { ContractPath },
            methods));
        var sources = new CapturedSourceSet(new[]
        {
            new CapturedSourceText(SourcePath, source),
        });
        RoslynSourceSession sourceSession = RoslynSessionFactory.Parse(selection, sources);
        RoslynCompilationSession compilationSession = RoslynSessionFactory.Compile(
            selection,
            sourceSession,
            referencePackRoot);
        SubsetClosure closure = CSharpSubset.Validate(selection, compilationSession);
        ContractSet contracts = InjectContracts(selection, closure);
        return new LoweringContext(selection, closure, contracts);
    }

    private static ContractSet InjectContracts(Selection selection, SubsetClosure closure)
    {
        var contracts = new AttachedContract[closure.Methods.Length];
        for (int index = 0; index < closure.Methods.Length; index++)
        {
            string method = closure.Methods[index].CanonicalId;
            var sidecar = new ParsedContractSidecar(
                method,
                Array.Empty<ContractExpression>(),
                new[] { ContractExpression.BooleanLiteral(true) },
                1);
            var normalized = new NormalizedContract(
                selection.Raw.Compilation,
                method,
                Array.Empty<NormalizedContractExpression>(),
                new[] { NormalizedContractExpression.BooleanLiteral(true) });
            contracts[index] = new AttachedContract(
                "contracts/injected" + index.ToString("D3", CultureInfo.InvariantCulture) + ".json",
                new string('0', 64),
                sidecar,
                normalized);
        }

        return new ContractSet(selection.Sha256, contracts);
    }

    private static IEnumerable<IOperation> OperationTree(IOperation root)
    {
        yield return root;
        foreach (IOperation child in root.ChildOperations)
        {
            foreach (IOperation descendant in OperationTree(child))
            {
                yield return descendant;
            }
        }
    }

    private static bool IsCheckedOperation(IOperation operation, string kind)
    {
        return operation switch
        {
            IBinaryOperation binary => string.Equals(
                binary.OperatorKind.ToString(),
                kind,
                StringComparison.Ordinal),
            IUnaryOperation unary => string.Equals(
                unary.OperatorKind.ToString(),
                kind,
                StringComparison.Ordinal),
            _ => false,
        };
    }

    private static string[] InstructionTrace(LoweredFunction function)
    {
        return function.Blocks
            .SelectMany(block => block.Instructions)
            .Select(LoweringValidator.ProfileOperation)
            .ToArray();
    }

    private static string Fingerprint(LoweredFunction function)
    {
        var value = new StringBuilder(function.Id)
            .Append("|args=").Append(string.Join(',', function.Parameters.Select(FormatBinding)))
            .Append("|results=").Append(string.Join(',', function.Results.Select(FormatBinding)))
            .Append("|locals=").Append(string.Join(',', function.Locals.Select(FormatBinding)))
            .Append("|features=").Append(string.Join(',', function.Features));
        foreach (LoweredBlock block in function.Blocks)
        {
            value.Append('|').Append(block.Label).Append('(')
                .Append(string.Join(',', block.Parameters.Select(FormatBinding)))
                .Append(')');
            foreach (LoweredInstruction instruction in block.Instructions)
            {
                value.Append(':').Append(instruction.Id).Append('=')
                    .Append(instruction.Kind).Append(':')
                    .Append(instruction.Type).Append(':')
                    .Append(instruction.Target).Append(':')
                    .Append(instruction.UnaryOperator).Append(':')
                    .Append(instruction.BinaryOperator).Append(':')
                    .Append(instruction.ConversionForm).Append(':')
                    .Append(instruction.OverflowContext).Append(':')
                    .Append(instruction.IsShiftCountMask).Append(':')
                    .Append(LoweringValidator.ProfileOperation(instruction)).Append('[')
                    .Append(string.Join(',', instruction.Operands.Select(FormatValue)))
                    .Append("]{")
                    .Append(string.Join(',', instruction.SafetyChecks.Select(FormatCheck)))
                    .Append("}@").Append(FormatOrigin(instruction.Origin));
            }

            value.Append("->").Append(block.Terminator.Kind).Append(':')
                .Append(block.Terminator.Condition is null
                    ? string.Empty
                    : FormatValue(block.Terminator.Condition)).Append(':')
                .Append(block.Terminator.FalseTarget).Append(':')
                .Append(string.Join(',', block.Terminator.FalseArguments.Select(FormatValue))).Append(':')
                .Append(block.Terminator.TrueTarget).Append(':')
                .Append(string.Join(',', block.Terminator.TrueArguments.Select(FormatValue))).Append(':')
                .Append(string.Join(',', block.Terminator.Values.Select(FormatValue)))
                .Append('@').Append(FormatOrigin(block.Terminator.Origin));
        }

        value.Append("|checks=").Append(string.Join(',', function.RequiredChecks.Select(check =>
            check.InstructionId + ":" + FormatCheck(check.Check))));

        return value.ToString();
    }

    private static string FormatBinding(LoweredBinding binding)
    {
        return binding.Id + ":" + binding.Type;
    }

    private static string FormatValue(LoweredValue value)
    {
        string payload = value.Kind switch
        {
            LoweredValueKind.Variable or LoweredValueKind.Integer => value.Text ?? string.Empty,
            LoweredValueKind.Boolean => value.Boolean ? "true" : "false",
            _ => throw new HarnessFailure("VALUE_KIND"),
        };
        return value.Kind + ":" + value.Type + ":" + payload;
    }

    private static string FormatCheck(LoweredSafetyCheck check)
    {
        return check.Kind + ":" + check.Operation + ":" + check.Width + ":" + check.Signed;
    }

    private static string FormatOrigin(LoweredOrigin origin)
    {
        return origin.NormalizedPath + ":" + origin.Utf16Start + ":" + origin.Utf16End;
    }

    private static string CopyOperand(LoweredBlock block)
    {
        LoweredInstruction copy = block.Instructions
            .Single(instruction => instruction.Kind == LoweredInstructionKind.Copy);
        return copy.Operands.Single().Text ?? throw new HarnessFailure("COPY_OPERAND");
    }

    private static LoweredInstruction CloneInstruction(
        LoweredInstruction source,
        LoweredBinaryOperator? binaryOperator = null,
        LoweredSafetyCheck[]? checks = null,
        LoweredValue[]? operands = null)
    {
        return new LoweredInstruction(
            source.Id,
            source.Kind,
            source.Type,
            source.Target,
            source.UnaryOperator,
            binaryOperator ?? source.BinaryOperator,
            source.ConversionForm,
            source.OverflowContext,
            source.IsShiftCountMask,
            operands ?? source.Operands.ToArray(),
            checks ?? source.SafetyChecks.ToArray(),
            source.Origin,
            source.Function,
            source.ContractHash);
    }

    private static LoweredFunction ReplaceInstruction(
        LoweredFunction function,
        string id,
        LoweredInstruction replacement)
    {
        LoweredBlock[] blocks = function.Blocks.Select(block => new LoweredBlock(
            block.Label,
            block.Parameters.ToArray(),
            block.Instructions
                .Select(instruction => string.Equals(instruction.Id, id, StringComparison.Ordinal)
                    ? replacement
                    : instruction)
                .ToArray(),
            block.Terminator)).ToArray();
        return CloneFunction(function, blocks: blocks);
    }

    private static LoweredFunction CloneFunction(
        LoweredFunction function,
        LoweredBlock[]? blocks = null,
        LoweredRequiredCheck[]? requiredChecks = null)
    {
        return new LoweredFunction(
            function.Id,
            function.Name,
            function.ContractHash,
            function.Origin,
            function.Parameters.ToArray(),
            function.Results.ToArray(),
            function.Locals.ToArray(),
            blocks ?? function.Blocks.ToArray(),
            requiredChecks ?? function.RequiredChecks.ToArray(),
            function.Features.ToArray());
    }

    private static void ExpectLoweringRejected(Action action, string code)
    {
        try
        {
            action();
        }
        catch (FrontendFailure failure)
        {
            if (failure.Status != FrontendStatus.Rejected
                || !string.Equals(failure.Phase, "lowering", StringComparison.Ordinal)
                || !string.Equals(failure.Code, code, StringComparison.Ordinal))
            {
                throw new HarnessFailure(
                    code + "_GOT_" + failure.Status + "_" + failure.Phase + "_" + failure.Code);
            }

            return;
        }

        throw new HarnessFailure(code + "_ACCEPTED");
    }

    private static string TypeToken(string sourceType)
    {
        return sourceType switch
        {
            "bool" => "bool",
            "int" => "i32",
            "uint" => "u32",
            "long" => "i64",
            "ulong" => "u64",
            _ => throw new HarnessFailure("TYPE_TOKEN"),
        };
    }

    private static SubsetValueType SourceType(string sourceType)
    {
        return sourceType switch
        {
            "bool" => SubsetValueType.Bool,
            "int" => SubsetValueType.I32,
            "uint" => SubsetValueType.U32,
            "long" => SubsetValueType.I64,
            "ulong" => SubsetValueType.U64,
            _ => throw new HarnessFailure("SOURCE_TYPE"),
        };
    }

    private static void Check(bool condition, string code)
    {
        if (!condition)
        {
            throw new HarnessFailure(code);
        }
    }

    private static void Equal<T>(T expected, T actual, string code)
    {
        if (!EqualityComparer<T>.Default.Equals(expected, actual))
        {
            throw new HarnessFailure(code);
        }
    }

    private sealed class OperationCase
    {
        internal OperationCase(string source, string method, params string[] expectedInstructions)
        {
            Source = source;
            Method = method;
            ExpectedInstructions = expectedInstructions;
        }

        internal string Source { get; }

        internal string Method { get; }

        internal string[] ExpectedInstructions { get; }
    }

    private sealed class CheckedCase
    {
        internal CheckedCase(
            string operatorKind,
            string source,
            string context,
            bool expectedIsChecked,
            string[] operandTypes)
        {
            OperatorKind = operatorKind;
            Source = source;
            Context = context;
            ExpectedIsChecked = expectedIsChecked;
            OperandTypes = operandTypes;
        }

        internal string OperatorKind { get; }

        internal string Source { get; }

        internal string Context { get; }

        internal bool ExpectedIsChecked { get; }

        internal string[] OperandTypes { get; }
    }

    private sealed class ConversionCase
    {
        internal ConversionCase(string sourceType, string destinationType, string form)
        {
            SourceType = sourceType;
            DestinationType = destinationType;
            Form = form;
        }

        internal string SourceType { get; }

        internal string DestinationType { get; }

        internal string Form { get; }
    }

    private sealed class LoweringContext
    {
        internal LoweringContext(
            Selection selection,
            SubsetClosure closure,
            ContractSet contracts)
        {
            Selection = selection;
            Closure = closure;
            Contracts = contracts;
        }

        internal Selection Selection { get; }

        internal SubsetClosure Closure { get; }

        internal ContractSet Contracts { get; }

        internal LoweredClosure? Lowered { get; set; }

        internal LoweredFunction Function(string id)
        {
            return (Lowered ?? throw new HarnessFailure("LOWERING_MISSING"))
                .Functions.Single(function => string.Equals(function.Id, id, StringComparison.Ordinal));
        }

        internal SubsetMethod SubsetMethod(string id)
        {
            return Closure.Methods.Single(method =>
                string.Equals(method.CanonicalId, id, StringComparison.Ordinal));
        }
    }

    private sealed class HarnessFailure : Exception
    {
        internal HarnessFailure(string code)
            : base(code)
        {
            Code = code;
        }

        internal string Code { get; }
    }
}
