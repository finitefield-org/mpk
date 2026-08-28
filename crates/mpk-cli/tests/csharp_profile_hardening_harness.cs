using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Numerics;
using System.Reflection;
using System.Runtime.Loader;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace Mpk.CSharp2Vir;

internal static partial class FrontendVectorHarness
{
    private static DifferentialResult ExecuteDifferential(
        string id,
        string methodId,
        AcceptedExecution execution)
    {
        Equal("10.0.11", Environment.Version.ToString(3), id + "_DIFFERENTIAL_RUNTIME");
        LoweredFunction function = execution.Lowered.Functions
            .Single(candidate => string.Equals(candidate.Id, methodId, StringComparison.Ordinal));
        List<Scalar[]> inputs = DifferentialInputs(id, function);
        Check(inputs.Count > 0 && inputs.Count <= 16, id + "_DIFFERENTIAL_BOUND");

        using var image = new MemoryStream();
        Microsoft.CodeAnalysis.Emit.EmitResult emitted =
            execution.Compilation.Compilation.Emit(image);
        Check(emitted.Success, id + "_DIFFERENTIAL_EMIT");
        image.Position = 0;
        var loadContext = new AssemblyLoadContext(
            "mpk-csharp-differential-" + id,
            isCollectible: true);
        var ledger = new StringBuilder();
        try
        {
            Assembly assembly = loadContext.LoadFromStream(image);
            CanonicalMethodId parsed = SelectionCodec.ParseMethodId(methodId);
            string typeName = parsed.NamespaceName + "." + parsed.StaticType;
            Type runtimeType = assembly.GetType(typeName, throwOnError: true, ignoreCase: false)
                ?? throw new VectorFailure(id + "_DIFFERENTIAL_TYPE");
            MethodInfo method = runtimeType
                .GetMethods(BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Static)
                .Single(candidate =>
                    string.Equals(candidate.Name, parsed.Method, StringComparison.Ordinal)
                    && RuntimeSignatureMatches(candidate, parsed));

            foreach (Scalar[] input in inputs)
            {
                DifferentialOutcome runtime = InvokeRuntime(method, function, input);
                DifferentialOutcome lowered = InvokeLowered(execution.Lowered, methodId, input);
                Equal(runtime, lowered, id + "_DIFFERENTIAL_OUTCOME");
                ledger.Append(id).Append('|');
                foreach (Scalar value in input)
                {
                    ledger.Append(ScalarText(value)).Append(',');
                }

                ledger.Append("=>").Append(OutcomeText(runtime)).Append('\n');
            }
        }
        finally
        {
            loadContext.Unload();
        }

        return new DifferentialResult(
            id,
            inputs.Count,
            RawSha256(StrictUtf8.GetBytes(ledger.ToString())));
    }

    private static bool RuntimeSignatureMatches(
        MethodInfo method,
        CanonicalMethodId expected)
    {
        ParameterInfo[] parameters = method.GetParameters();
        if (parameters.Length != expected.ParameterTypes.Count
            || !string.Equals(
                RuntimeTypeId(method.ReturnType),
                expected.ResultType,
                StringComparison.Ordinal))
        {
            return false;
        }

        for (int index = 0; index < parameters.Length; index++)
        {
            if (!string.Equals(
                RuntimeTypeId(parameters[index].ParameterType),
                expected.ParameterTypes[index],
                StringComparison.Ordinal))
            {
                return false;
            }
        }

        return true;
    }

    private static string RuntimeTypeId(Type type)
    {
        if (type == typeof(bool))
        {
            return "bool";
        }

        if (type == typeof(int))
        {
            return "i32";
        }

        if (type == typeof(uint))
        {
            return "u32";
        }

        if (type == typeof(long))
        {
            return "i64";
        }

        return type == typeof(ulong) ? "u64" : string.Empty;
    }

    private static DifferentialOutcome InvokeRuntime(
        MethodInfo method,
        LoweredFunction function,
        IReadOnlyList<Scalar> input)
    {
        try
        {
            object?[] arguments = input.Select(ScalarObject).ToArray();
            object result = method.Invoke(null, arguments)
                ?? throw new VectorFailure("DIFFERENTIAL_NULL_RESULT");
            return DifferentialOutcome.Returned(
                ScalarFromObject(function.Results[0].Type, result));
        }
        catch (TargetInvocationException error) when (error.InnerException is OverflowException)
        {
            return DifferentialOutcome.Trapped("overflow");
        }
        catch (TargetInvocationException error) when (error.InnerException is DivideByZeroException)
        {
            return DifferentialOutcome.Trapped("division-by-zero");
        }
    }

    private static DifferentialOutcome InvokeLowered(
        LoweredClosure closure,
        string methodId,
        IReadOnlyList<Scalar> input)
    {
        try
        {
            var functions = closure.Functions.ToDictionary(
                function => function.Id,
                StringComparer.Ordinal);
            return DifferentialOutcome.Returned(
                EvaluateFunction(functions, methodId, input, depth: 0));
        }
        catch (DifferentialTrap trap)
        {
            return DifferentialOutcome.Trapped(trap.Category);
        }
    }

    private static Scalar EvaluateFunction(
        IReadOnlyDictionary<string, LoweredFunction> functions,
        string methodId,
        IReadOnlyList<Scalar> arguments,
        int depth)
    {
        if (depth > 8
            || !functions.TryGetValue(methodId, out LoweredFunction? function)
            || function.Parameters.Count != arguments.Count)
        {
            throw new VectorFailure("DIFFERENTIAL_CALL_CLOSURE");
        }

        var values = new Dictionary<string, Scalar>(StringComparer.Ordinal);
        for (int index = 0; index < arguments.Count; index++)
        {
            RequireType(arguments[index], function.Parameters[index].Type);
            values.Add(function.Parameters[index].Id, arguments[index]);
        }

        var blocks = function.Blocks.ToDictionary(block => block.Label, StringComparer.Ordinal);
        LoweredBlock current = function.Blocks[0];
        Scalar[] incoming = Array.Empty<Scalar>();
        int steps = 0;
        while (true)
        {
            if (++steps > 1_024 || current.Parameters.Count != incoming.Length)
            {
                throw new VectorFailure("DIFFERENTIAL_CONTROL_FLOW");
            }

            for (int index = 0; index < incoming.Length; index++)
            {
                RequireType(incoming[index], current.Parameters[index].Type);
                values[current.Parameters[index].Id] = incoming[index];
            }

            foreach (LoweredInstruction instruction in current.Instructions)
            {
                Scalar[] operands = instruction.Operands
                    .Select(operand => ResolveValue(operand, values))
                    .ToArray();
                ApplySafetyChecks(instruction, operands);
                Scalar result = instruction.Kind switch
                {
                    LoweredInstructionKind.Const => operands.Single(),
                    LoweredInstructionKind.Copy => operands.Single(),
                    LoweredInstructionKind.Unary => EvaluateUnary(instruction, operands.Single()),
                    LoweredInstructionKind.Binary => EvaluateBinary(instruction, operands[0], operands[1]),
                    LoweredInstructionKind.Convert => ConvertScalar(operands.Single(), instruction.Type),
                    LoweredInstructionKind.CallStatic => EvaluateFunction(
                        functions,
                        instruction.Function ?? throw new VectorFailure("DIFFERENTIAL_CALL"),
                        operands,
                        depth + 1),
                    _ => throw new VectorFailure("DIFFERENTIAL_INSTRUCTION"),
                };
                RequireType(result, instruction.Type);
                values.Add(instruction.Id, result);
                if (instruction.Kind == LoweredInstructionKind.Copy)
                {
                    values[instruction.Target
                        ?? throw new VectorFailure("DIFFERENTIAL_COPY_TARGET")] = result;
                }
            }

            LoweredTerminator terminator = current.Terminator;
            switch (terminator.Kind)
            {
                case LoweredTerminatorKind.Return:
                    if (terminator.Values.Count != 1)
                    {
                        throw new VectorFailure("DIFFERENTIAL_RETURN");
                    }

                    Scalar returned = ResolveValue(terminator.Values[0], values);
                    RequireType(returned, function.Results[0].Type);
                    return returned;
                case LoweredTerminatorKind.Jump:
                    incoming = terminator.FalseArguments
                        .Select(value => ResolveValue(value, values))
                        .ToArray();
                    current = FindBlock(blocks, terminator.FalseTarget);
                    break;
                case LoweredTerminatorKind.Branch:
                    Scalar condition = ResolveValue(
                        terminator.Condition
                            ?? throw new VectorFailure("DIFFERENTIAL_BRANCH_CONDITION"),
                        values);
                    RequireType(condition, SubsetValueType.Bool);
                    bool takeTrue = condition.Bits != 0;
                    incoming = (takeTrue
                            ? terminator.TrueArguments
                            : terminator.FalseArguments)
                        .Select(value => ResolveValue(value, values))
                        .ToArray();
                    current = FindBlock(
                        blocks,
                        takeTrue ? terminator.TrueTarget : terminator.FalseTarget);
                    break;
                default:
                    throw new VectorFailure("DIFFERENTIAL_TERMINATOR");
            }
        }
    }

    private static LoweredBlock FindBlock(
        IReadOnlyDictionary<string, LoweredBlock> blocks,
        string? label)
    {
        if (label is null || !blocks.TryGetValue(label, out LoweredBlock? block))
        {
            throw new VectorFailure("DIFFERENTIAL_BLOCK");
        }

        return block;
    }

    private static Scalar ResolveValue(
        LoweredValue value,
        IReadOnlyDictionary<string, Scalar> values)
    {
        Scalar result = value.Kind switch
        {
            LoweredValueKind.Variable => values.TryGetValue(
                value.Text ?? string.Empty,
                out Scalar found)
                    ? found
                    : throw new VectorFailure("DIFFERENTIAL_VALUE"),
            LoweredValueKind.Boolean => BooleanScalar(value.Boolean),
            LoweredValueKind.Integer => IntegerScalar(
                value.Type,
                BigInteger.Parse(
                    value.Text ?? throw new VectorFailure("DIFFERENTIAL_LITERAL"),
                    CultureInfo.InvariantCulture)),
            _ => throw new VectorFailure("DIFFERENTIAL_VALUE_KIND"),
        };
        RequireType(result, value.Type);
        return result;
    }

    private static void ApplySafetyChecks(
        LoweredInstruction instruction,
        IReadOnlyList<Scalar> operands)
    {
        foreach (LoweredSafetyCheck check in instruction.SafetyChecks)
        {
            switch (check.Kind)
            {
                case LoweredSafetyCheckKind.IntegerNoOverflow:
                    BigInteger left = MathematicalValue(operands[0]);
                    BigInteger result = check.Operation switch
                    {
                        LoweredCheckOperation.Add => left + MathematicalValue(operands[1]),
                        LoweredCheckOperation.Sub => left - MathematicalValue(operands[1]),
                        LoweredCheckOperation.Mul => left * MathematicalValue(operands[1]),
                        LoweredCheckOperation.Neg => -left,
                        _ => throw new VectorFailure("DIFFERENTIAL_OVERFLOW_CHECK"),
                    };
                    BigInteger minimum = check.Signed
                        ? -(BigInteger.One << (check.Width - 1))
                        : BigInteger.Zero;
                    BigInteger maximum = check.Signed
                        ? (BigInteger.One << (check.Width - 1)) - 1
                        : (BigInteger.One << check.Width) - 1;
                    if (result < minimum || result > maximum)
                    {
                        throw new DifferentialTrap("overflow");
                    }

                    break;
                case LoweredSafetyCheckKind.DivisorNonzero:
                    if (operands.Count != 2 || operands[1].Bits == 0)
                    {
                        throw new DifferentialTrap("division-by-zero");
                    }

                    break;
                case LoweredSafetyCheckKind.SignedDivremRepresentable:
                    if (operands.Count != 2)
                    {
                        throw new VectorFailure("DIFFERENTIAL_DIVREM_CHECK");
                    }

                    BigInteger signedLeft = MathematicalValue(operands[0]);
                    BigInteger signedRight = MathematicalValue(operands[1]);
                    BigInteger signedMinimum = -(BigInteger.One << (check.Width - 1));
                    if (signedLeft == signedMinimum && signedRight == -1)
                    {
                        throw new DifferentialTrap("overflow");
                    }

                    break;
                default:
                    throw new VectorFailure("DIFFERENTIAL_SAFETY_CHECK");
            }
        }
    }

    private static Scalar EvaluateUnary(LoweredInstruction instruction, Scalar value)
    {
        return instruction.UnaryOperator switch
        {
            LoweredUnaryOperator.BoolNot => BooleanScalar(value.Bits == 0),
            LoweredUnaryOperator.BvNeg => IntegerScalar(
                instruction.Type,
                -MathematicalValue(value)),
            LoweredUnaryOperator.BvNot => new Scalar(
                instruction.Type,
                (~value.Bits) & TypeMask(instruction.Type)),
            _ => throw new VectorFailure("DIFFERENTIAL_UNARY"),
        };
    }

    private static Scalar EvaluateBinary(
        LoweredInstruction instruction,
        Scalar left,
        Scalar right)
    {
        ulong mask = instruction.Type == SubsetValueType.Bool
            ? 1
            : TypeMask(instruction.Type);
        return instruction.BinaryOperator switch
        {
            LoweredBinaryOperator.Eq => BooleanScalar(left.Bits == right.Bits),
            LoweredBinaryOperator.NotEq => BooleanScalar(left.Bits != right.Bits),
            LoweredBinaryOperator.BvAdd => new Scalar(
                instruction.Type,
                unchecked(left.Bits + right.Bits) & mask),
            LoweredBinaryOperator.BvSub => new Scalar(
                instruction.Type,
                unchecked(left.Bits - right.Bits) & mask),
            LoweredBinaryOperator.BvMul => new Scalar(
                instruction.Type,
                unchecked(left.Bits * right.Bits) & mask),
            LoweredBinaryOperator.BvSdiv => SignedDivrem(
                instruction.Type,
                left,
                right,
                divide: true),
            LoweredBinaryOperator.BvSrem => SignedDivrem(
                instruction.Type,
                left,
                right,
                divide: false),
            LoweredBinaryOperator.BvUdiv => new Scalar(
                instruction.Type,
                (left.Bits / right.Bits) & mask),
            LoweredBinaryOperator.BvUrem => new Scalar(
                instruction.Type,
                (left.Bits % right.Bits) & mask),
            LoweredBinaryOperator.BvAnd => new Scalar(
                instruction.Type,
                (left.Bits & right.Bits) & mask),
            LoweredBinaryOperator.BvOr => new Scalar(
                instruction.Type,
                (left.Bits | right.Bits) & mask),
            LoweredBinaryOperator.BvXor => new Scalar(
                instruction.Type,
                (left.Bits ^ right.Bits) & mask),
            LoweredBinaryOperator.BvShl => new Scalar(
                instruction.Type,
                (left.Bits << ShiftCount(right)) & mask),
            LoweredBinaryOperator.BvAshr => ArithmeticShiftRight(
                instruction.Type,
                left,
                right),
            LoweredBinaryOperator.BvLshr => new Scalar(
                instruction.Type,
                (left.Bits >> ShiftCount(right)) & mask),
            LoweredBinaryOperator.SignedLt => BooleanScalar(
                MathematicalValue(left) < MathematicalValue(right)),
            LoweredBinaryOperator.SignedLe => BooleanScalar(
                MathematicalValue(left) <= MathematicalValue(right)),
            LoweredBinaryOperator.SignedGt => BooleanScalar(
                MathematicalValue(left) > MathematicalValue(right)),
            LoweredBinaryOperator.SignedGe => BooleanScalar(
                MathematicalValue(left) >= MathematicalValue(right)),
            LoweredBinaryOperator.UnsignedLt => BooleanScalar(left.Bits < right.Bits),
            LoweredBinaryOperator.UnsignedLe => BooleanScalar(left.Bits <= right.Bits),
            LoweredBinaryOperator.UnsignedGt => BooleanScalar(left.Bits > right.Bits),
            LoweredBinaryOperator.UnsignedGe => BooleanScalar(left.Bits >= right.Bits),
            _ => throw new VectorFailure("DIFFERENTIAL_BINARY"),
        };
    }

    private static Scalar SignedDivrem(
        SubsetValueType type,
        Scalar left,
        Scalar right,
        bool divide)
    {
        if (type == SubsetValueType.I32)
        {
            int leftValue = unchecked((int)(uint)left.Bits);
            int rightValue = unchecked((int)(uint)right.Bits);
            int result = divide ? leftValue / rightValue : leftValue % rightValue;
            return new Scalar(type, unchecked((uint)result));
        }

        long signedLeft = unchecked((long)left.Bits);
        long signedRight = unchecked((long)right.Bits);
        long signedResult = divide ? signedLeft / signedRight : signedLeft % signedRight;
        return new Scalar(type, unchecked((ulong)signedResult));
    }

    private static Scalar ArithmeticShiftRight(
        SubsetValueType type,
        Scalar value,
        Scalar count)
    {
        int shift = ShiftCount(count);
        return type switch
        {
            SubsetValueType.I32 => new Scalar(
                type,
                unchecked((uint)(unchecked((int)(uint)value.Bits) >> shift))),
            SubsetValueType.I64 => new Scalar(
                type,
                unchecked((ulong)(unchecked((long)value.Bits) >> shift))),
            _ => throw new VectorFailure("DIFFERENTIAL_ARITHMETIC_SHIFT"),
        };
    }

    private static int ShiftCount(Scalar value)
    {
        RequireType(value, SubsetValueType.I32);
        return unchecked((int)(uint)value.Bits);
    }

    private static Scalar ConvertScalar(Scalar value, SubsetValueType target)
    {
        if (target == SubsetValueType.Bool || value.Type == SubsetValueType.Bool)
        {
            if (target == value.Type)
            {
                return value;
            }

            throw new VectorFailure("DIFFERENTIAL_CONVERSION_TYPE");
        }

        return IntegerScalar(target, MathematicalValue(value));
    }

    private static Scalar IntegerScalar(SubsetValueType type, BigInteger value)
    {
        int width = TypeWidth(type);
        BigInteger modulus = BigInteger.One << width;
        BigInteger reduced = value % modulus;
        if (reduced < 0)
        {
            reduced += modulus;
        }

        return new Scalar(type, (ulong)reduced);
    }

    private static Scalar BooleanScalar(bool value)
    {
        return new Scalar(SubsetValueType.Bool, value ? 1UL : 0UL);
    }

    private static BigInteger MathematicalValue(Scalar value)
    {
        return value.Type switch
        {
            SubsetValueType.I32 => unchecked((int)(uint)value.Bits),
            SubsetValueType.U32 => (uint)value.Bits,
            SubsetValueType.I64 => unchecked((long)value.Bits),
            SubsetValueType.U64 => value.Bits,
            _ => throw new VectorFailure("DIFFERENTIAL_INTEGER_TYPE"),
        };
    }

    private static int TypeWidth(SubsetValueType type)
    {
        return type switch
        {
            SubsetValueType.I32 or SubsetValueType.U32 => 32,
            SubsetValueType.I64 or SubsetValueType.U64 => 64,
            _ => throw new VectorFailure("DIFFERENTIAL_WIDTH"),
        };
    }

    private static ulong TypeMask(SubsetValueType type)
    {
        return TypeWidth(type) == 32 ? uint.MaxValue : ulong.MaxValue;
    }

    private static void RequireType(Scalar value, SubsetValueType expected)
    {
        if (value.Type != expected
            || (expected == SubsetValueType.Bool && value.Bits > 1)
            || (TypeWidthOrZero(expected) == 32 && value.Bits > uint.MaxValue))
        {
            throw new VectorFailure("DIFFERENTIAL_SCALAR_TYPE");
        }
    }

    private static int TypeWidthOrZero(SubsetValueType type)
    {
        return type == SubsetValueType.Bool ? 0 : TypeWidth(type);
    }

    private static object ScalarObject(Scalar value)
    {
        return value.Type switch
        {
            SubsetValueType.Bool => value.Bits != 0,
            SubsetValueType.I32 => unchecked((int)(uint)value.Bits),
            SubsetValueType.U32 => (uint)value.Bits,
            SubsetValueType.I64 => unchecked((long)value.Bits),
            SubsetValueType.U64 => value.Bits,
            _ => throw new VectorFailure("DIFFERENTIAL_RUNTIME_ARGUMENT"),
        };
    }

    private static Scalar ScalarFromObject(SubsetValueType type, object value)
    {
        return type switch
        {
            SubsetValueType.Bool when value is bool boolean => BooleanScalar(boolean),
            SubsetValueType.I32 when value is int signed32 =>
                new Scalar(type, unchecked((uint)signed32)),
            SubsetValueType.U32 when value is uint unsigned32 => new Scalar(type, unsigned32),
            SubsetValueType.I64 when value is long signed64 =>
                new Scalar(type, unchecked((ulong)signed64)),
            SubsetValueType.U64 when value is ulong unsigned64 => new Scalar(type, unsigned64),
            _ => throw new VectorFailure("DIFFERENTIAL_RUNTIME_RESULT"),
        };
    }

    private static List<Scalar[]> DifferentialInputs(string id, LoweredFunction function)
    {
        var result = new List<Scalar[]>();
        var seen = new HashSet<string>(StringComparer.Ordinal);
        if (function.Parameters.Count == 0)
        {
            AddDifferentialInput(result, seen, Array.Empty<Scalar>());
        }
        else
        {
            for (int round = 0; round < 5; round++)
            {
                var values = new Scalar[function.Parameters.Count];
                for (int index = 0; index < values.Length; index++)
                {
                    Scalar[] candidates = DifferentialValues(function.Parameters[index].Type);
                    values[index] = candidates[(round + index) % candidates.Length];
                }

                AddDifferentialInput(result, seen, values);
            }
        }

        if (id is "arithmetic.checked" or "arithmetic.unchecked")
        {
            AddDifferentialInput(result, seen, new[]
            {
                IntegerScalar(SubsetValueType.I32, int.MaxValue),
                IntegerScalar(SubsetValueType.I32, 1),
                IntegerScalar(SubsetValueType.I32, 1),
            });
        }

        if (id is "division.signed_checked" or "division.signed_unchecked")
        {
            AddDifferentialInput(result, seen, new[]
            {
                IntegerScalar(SubsetValueType.I32, int.MinValue),
                IntegerScalar(SubsetValueType.I32, -1),
            });
            AddDifferentialInput(result, seen, new[]
            {
                IntegerScalar(SubsetValueType.I32, 1),
                IntegerScalar(SubsetValueType.I32, 0),
            });
        }

        if (id == "division.unsigned")
        {
            AddDifferentialInput(result, seen, new[]
            {
                IntegerScalar(SubsetValueType.U32, uint.MaxValue),
                IntegerScalar(SubsetValueType.U32, 0),
            });
        }

        if (id == "shift.i32_mask")
        {
            AddDifferentialInput(result, seen, new[]
            {
                IntegerScalar(SubsetValueType.I32, -1),
                IntegerScalar(SubsetValueType.I32, 32),
            });
        }

        if (id == "shift.i64_mask")
        {
            AddDifferentialInput(result, seen, new[]
            {
                IntegerScalar(SubsetValueType.I64, -1),
                IntegerScalar(SubsetValueType.I32, 64),
            });
        }

        return result;
    }

    private static Scalar[] DifferentialValues(SubsetValueType type)
    {
        return type switch
        {
            SubsetValueType.Bool => new[]
            {
                BooleanScalar(false),
                BooleanScalar(true),
                BooleanScalar(false),
                BooleanScalar(true),
                BooleanScalar(false),
            },
            SubsetValueType.I32 => new[]
            {
                IntegerScalar(type, 0),
                IntegerScalar(type, 1),
                IntegerScalar(type, -1),
                IntegerScalar(type, int.MinValue),
                IntegerScalar(type, int.MaxValue),
            },
            SubsetValueType.U32 => new[]
            {
                IntegerScalar(type, 0),
                IntegerScalar(type, 1),
                IntegerScalar(type, uint.MaxValue),
                IntegerScalar(type, 0x80000000U),
                IntegerScalar(type, 31),
            },
            SubsetValueType.I64 => new[]
            {
                IntegerScalar(type, 0),
                IntegerScalar(type, 1),
                IntegerScalar(type, -1),
                IntegerScalar(type, long.MinValue),
                IntegerScalar(type, long.MaxValue),
            },
            SubsetValueType.U64 => new[]
            {
                IntegerScalar(type, 0),
                IntegerScalar(type, 1),
                IntegerScalar(type, ulong.MaxValue),
                IntegerScalar(type, BigInteger.One << 63),
                IntegerScalar(type, 63),
            },
            _ => throw new VectorFailure("DIFFERENTIAL_INPUT_TYPE"),
        };
    }

    private static void AddDifferentialInput(
        List<Scalar[]> destination,
        HashSet<string> seen,
        Scalar[] values)
    {
        string key = string.Join(",", values.Select(ScalarText));
        if (seen.Add(key))
        {
            destination.Add(values);
        }
    }

    private static string ScalarText(Scalar value)
    {
        if (value.Type == SubsetValueType.Bool)
        {
            return "bool:" + (value.Bits == 0 ? "false" : "true");
        }

        return value.Type.ToString().ToLowerInvariant() + ":"
            + MathematicalValue(value).ToString(CultureInfo.InvariantCulture);
    }

    private static string OutcomeText(DifferentialOutcome outcome)
    {
        return outcome.Trap is null
            ? "return:" + ScalarText(
                outcome.Value ?? throw new VectorFailure("DIFFERENTIAL_OUTCOME_VALUE"))
            : "trap:" + outcome.Trap;
    }

    private static List<FuzzResult> ExecuteFuzz(
        JsonElement profile,
        string referencePackRoot,
        string manifestPath)
    {
        byte[] manifestBytes = File.ReadAllBytes(manifestPath);
        using JsonDocument manifest = JsonDocument.Parse(
            manifestBytes,
            new JsonDocumentOptions
            {
                AllowTrailingCommas = false,
                CommentHandling = JsonCommentHandling.Disallow,
                MaxDepth = 32,
            });
        JsonElement root = manifest.RootElement;
        Equal("mpk.csharp.fuzz_seeds.v0", Text(root, "schema"), "FUZZ_SCHEMA");
        JsonElement targets = Property(root, "targets");
        string[] expectedTargets =
        {
            "compiler_output",
            "contract",
            "parser",
            "protocol",
            "resource",
        };
        Check(
            expectedTargets.SequenceEqual(
                targets.EnumerateObject().Select(property => property.Name),
                StringComparer.Ordinal),
            "FUZZ_TARGETS");
        string manifestDirectory = Path.GetDirectoryName(Path.GetFullPath(manifestPath))
            ?? throw new VectorFailure("FUZZ_MANIFEST_PATH");
        var results = new List<FuzzResult>();
        foreach (string target in expectedTargets)
        {
            JsonElement records = Property(targets, target);
            Equal(2, records.GetArrayLength(), "FUZZ_SEED_COUNT_" + target);
            int mutationCount = 0;
            var ledger = new StringBuilder();
            string previous = string.Empty;
            foreach (JsonElement record in records.EnumerateArray())
            {
                string relative = Text(record, "path");
                Check(
                    relative.Length > 0
                    && relative.IndexOfAny(new[] { '/', '\\' }) < 0
                    && string.CompareOrdinal(previous, relative) < 0,
                    "FUZZ_SEED_PATH_" + target);
                previous = relative;
                string path = Path.Combine(manifestDirectory, "seeds", target, relative);
                byte[] seed = File.ReadAllBytes(path);
                Equal((ulong)seed.Length, Integer(record, "size_bytes"), "FUZZ_SEED_SIZE_" + target);
                Equal(Text(record, "sha256"), RawSha256(seed), "FUZZ_SEED_HASH_" + target);
                int mutationIndex = 0;
                foreach (byte[] mutation in MutateSeed(seed))
                {
                    string outcome = ExerciseFuzzTarget(
                        target,
                        mutation,
                        profile,
                        referencePackRoot);
                    ledger.Append(relative)
                        .Append('|')
                        .Append(mutationIndex.ToString(CultureInfo.InvariantCulture))
                        .Append('|')
                        .Append(RawSha256(mutation))
                        .Append('|')
                        .Append(outcome)
                        .Append('\n');
                    mutationIndex++;
                    mutationCount++;
                }

                Equal(6, mutationIndex, "FUZZ_MUTATIONS_PER_SEED_" + target);
            }

            results.Add(new FuzzResult(
                target,
                records.GetArrayLength(),
                mutationCount,
                RawSha256(StrictUtf8.GetBytes(ledger.ToString()))));
        }

        return results;
    }

    private static IEnumerable<byte[]> MutateSeed(byte[] seed)
    {
        yield return (byte[])seed.Clone();
        yield return Array.Empty<byte>();
        yield return seed.AsSpan(0, seed.Length / 2).ToArray();
        var appended = new byte[checked(seed.Length + 1)];
        Buffer.BlockCopy(seed, 0, appended, 0, seed.Length);
        appended[^1] = 0;
        yield return appended;
        var flipped = (byte[])seed.Clone();
        if (flipped.Length != 0)
        {
            flipped[flipped.Length / 2] ^= 0x80;
        }

        yield return flipped;
        var duplicated = new byte[checked(seed.Length * 2)];
        Buffer.BlockCopy(seed, 0, duplicated, 0, seed.Length);
        Buffer.BlockCopy(seed, 0, duplicated, seed.Length, seed.Length);
        yield return duplicated;
    }

    private static string ExerciseFuzzTarget(
        string target,
        byte[] input,
        JsonElement profile,
        string referencePackRoot)
    {
        try
        {
            return target switch
            {
                "parser" => ExerciseParser(input),
                "contract" => ExerciseContract(input),
                "protocol" => ExerciseProtocol(input, profile),
                "compiler_output" => ExerciseCompilerOutput(input, referencePackRoot),
                "resource" => ExerciseResource(input),
                _ => throw new VectorFailure("FUZZ_TARGET"),
            };
        }
        catch (FrontendFailure failure)
        {
            return "frontend:"
                + FrontendDiagnosticRegistry.StatusText(failure.Status)
                + ":" + failure.Phase + ":" + failure.Code;
        }
        catch (SelectionSyntaxFailure)
        {
            return "malformed:selection";
        }
        catch (JsonException)
        {
            return "malformed:json";
        }
        catch (InvalidOperationException)
        {
            return "malformed:shape";
        }
        catch (KeyNotFoundException)
        {
            return "malformed:missing";
        }
        catch (FormatException)
        {
            return "malformed:number";
        }
        catch (OverflowException)
        {
            return "malformed:overflow";
        }
    }

    private static string ExerciseParser(byte[] input)
    {
        string source = SourceTransport.Decode(input);
        Selection selection = FuzzSelection();
        RoslynSourceSession parsed = RoslynSessionFactory.Parse(
            selection,
            Sources("src/Seed.cs", source));
        return "accepted:" + parsed.SyntaxTrees.Length.ToString(CultureInfo.InvariantCulture);
    }

    private static string ExerciseContract(byte[] input)
    {
        ParsedContractSidecar parsed = ContractSidecarParser.Parse(
            input,
            new ContractClosureCounter());
        return "accepted:" + parsed.Method;
    }

    private static string ExerciseProtocol(byte[] input, JsonElement profile)
    {
        using JsonDocument document = JsonDocument.Parse(
            input,
            new JsonDocumentOptions
            {
                AllowTrailingCommas = false,
                CommentHandling = JsonCommentHandling.Disallow,
                MaxDepth = 64,
            });
        JsonElement root = document.RootElement;
        if (!root.TryGetProperty("status", out JsonElement statusElement)
            || !root.TryGetProperty("phase", out JsonElement phaseElement)
            || statusElement.GetString() is not string status
            || phaseElement.GetString() is not string phase)
        {
            throw new InvalidOperationException("protocol seed shape");
        }

        JsonElement issues = status == "rejected"
            ? root.GetProperty("rejected_features")
            : root.GetProperty("diagnostics");
        string code = issues[0].GetProperty("code").GetString()
            ?? throw new InvalidOperationException("protocol seed code");
        FrontendFailure failure = CreateFailure(status, phase, code);
        byte[] emitted = CSharpFrontendFailureEmitter.Emit(FailureRequest(profile), failure);
        using JsonDocument roundTrip = JsonDocument.Parse(emitted.AsMemory(0, emitted.Length - 1));
        return "emitted:" + RawSha256(emitted);
    }

    private static string ExerciseCompilerOutput(byte[] input, string referencePackRoot)
    {
        string source = SourceTransport.Decode(input);
        Selection selection = FuzzSelection();
        RoslynSourceSession parsed = RoslynSessionFactory.Parse(
            selection,
            Sources("src/Seed.cs", source));
        RoslynCompilationSession compiled = RoslynSessionFactory.Compile(
            selection,
            parsed,
            referencePackRoot);
        return "accepted:" + compiled.Diagnostics.Length.ToString(CultureInfo.InvariantCulture);
    }

    private static string ExerciseResource(byte[] input)
    {
        using JsonDocument document = JsonDocument.Parse(
            input,
            new JsonDocumentOptions
            {
                AllowTrailingCommas = false,
                CommentHandling = JsonCommentHandling.Disallow,
                MaxDepth = 8,
            });
        JsonElement root = document.RootElement;
        string id = root.GetProperty("id").GetString()
            ?? throw new InvalidOperationException("resource ID");
        string phase = root.GetProperty("phase").GetString()
            ?? throw new InvalidOperationException("resource phase");
        ulong observed = root.GetProperty("observed").GetUInt64();
        ulong accepted = FrontendLimits.Validate(id, observed, phase);
        return "accepted:" + accepted.ToString(CultureInfo.InvariantCulture);
    }

    private static Selection FuzzSelection()
    {
        return SelectionCodec.Validate(new RawSelection(
            "fuzz",
            new[] { "src/Seed.cs" },
            new[] { "contracts/seed.json" },
            new[] { "Fuzz.Seed::F(i32)->i32" }));
    }

    private readonly record struct Scalar(SubsetValueType Type, ulong Bits);

    private sealed record DifferentialOutcome(string? Trap, Scalar? Value)
    {
        internal static DifferentialOutcome Returned(Scalar value)
        {
            return new DifferentialOutcome(null, value);
        }

        internal static DifferentialOutcome Trapped(string category)
        {
            return new DifferentialOutcome(category, null);
        }
    }

    private sealed class DifferentialTrap : Exception
    {
        internal DifferentialTrap(string category)
            : base(category)
        {
            Category = category;
        }

        internal string Category { get; }
    }
}
