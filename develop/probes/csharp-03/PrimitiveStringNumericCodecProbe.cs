// Disposable CSHARP-03-T01-W07 pinned-runtime probe; never a frontend or library.
#nullable enable

using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Numerics;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

internal static class PrimitiveStringNumericCodecProbe
{
    private const string RawSchema =
        "mpk.csharp_practical.t01_w07.runtime_semantics_probe.raw.v0";
    private const string WorkItem = "CSHARP-03-T01-W07";
    private const int TextBound = 16_384;

    private static readonly string[] TextPrecedence =
    {
        "exception.null_receiver",
        "exception.null_argument",
        "exception.range",
        "obligation.output_bound",
    };

    private static readonly string[] ParsePrecedence =
    {
        "parse_error.input_bound",
        "parse_error.syntax",
        "parse_error.noncanonical",
        "parse_error.scale_precision",
        "parse_error.range",
    };

    private static readonly string[] NumericPrecedence =
    {
        "exception.division_by_zero",
        "exception.overflow",
    };

    private static readonly string[] FormatPrecedence =
    {
        "sidecar.unknown_codec",
        "sidecar.unknown_rounding_mode",
        "parse_error.scale_precision",
        "obligation.output_bound",
    };

    private static readonly string[] SidecarPrecedence =
    {
        "sidecar.unknown_codec",
        "sidecar.unknown_rounding_mode",
        "parse_error.input_bound",
        "parse_error.syntax",
        "parse_error.noncanonical",
        "parse_error.scale_precision",
        "parse_error.range",
        "obligation.output_bound",
    };

    private static readonly List<Dictionary<string, object?>> Vectors = new();

    private sealed record Observation(
        string Kind,
        string ResultEncoding,
        string Value,
        string? Exception);

    private sealed record CodecOutcome(
        string Kind,
        string ResultEncoding,
        string Value,
        string? ErrorId);

    private sealed record IntegerSpec(
        string Id,
        bool Signed,
        BigInteger Minimum,
        BigInteger Maximum);

    public static int Main(string[] args)
    {
        if (args.Length != 1)
        {
            Console.Error.Write("CSHARP_PRACTICAL_RUNTIME_PROBE_USAGE\n");
            return 64;
        }

        try
        {
            CultureInfo culture = CreateCulture(args[0]);
            CultureInfo.CurrentCulture = culture;
            CultureInfo.CurrentUICulture = culture;
            CultureInfo.DefaultThreadCurrentCulture = culture;
            CultureInfo.DefaultThreadCurrentUICulture = culture;

            AddStringVectors();
            AddCodecVectors();
            AddFloatingVectors();
            AddDecimalVectors();
            AddPrecedenceVectors();
            Vectors.Sort(
                (left, right) => StringComparer.Ordinal.Compare(
                    (string)left["id"]!,
                    (string)right["id"]!));

            Dictionary<string, object?> root = Obj(
                ("culture", Obj(
                    ("date_separator_utf16", Utf16(culture.DateTimeFormat.DateSeparator)),
                    ("decimal_separator_utf16", Utf16(culture.NumberFormat.NumberDecimalSeparator)),
                    ("group_separator_utf16", Utf16(culture.NumberFormat.NumberGroupSeparator)),
                    ("negative_sign_utf16", Utf16(culture.NumberFormat.NegativeSign)),
                    ("profile", args[0]),
                    ("short_date_pattern_utf16", Utf16(culture.DateTimeFormat.ShortDatePattern)),
                    ("time_separator_utf16", Utf16(culture.DateTimeFormat.TimeSeparator)))),
                ("runtime", Obj(
                    ("architecture", RuntimeInformation.ProcessArchitecture.ToString()),
                    ("framework_description", RuntimeInformation.FrameworkDescription),
                    ("runtime_version", Environment.Version.ToString()))),
                ("schema", RawSchema),
                ("vectors", Vectors),
                ("work_item", WorkItem));
            Console.OutputEncoding = new UTF8Encoding(false, true);
            Console.Write(JsonSerializer.Serialize(root));
            Console.Write('\n');
            return 0;
        }
        catch (ProbeFailure failure)
        {
            Console.Error.Write("CSHARP_PRACTICAL_RUNTIME_PROBE_" + failure.Code + "\n");
            return 65;
        }
        catch (Exception failure)
        {
            Console.Error.Write(
                "CSHARP_PRACTICAL_RUNTIME_PROBE_UNEXPECTED: "
                + failure.GetType().FullName
                + ": "
                + failure.Message
                + "\n"
                + failure.StackTrace
                + "\n");
            return 70;
        }
    }

    private static CultureInfo CreateCulture(string profile)
    {
        CultureInfo culture = (CultureInfo)CultureInfo.InvariantCulture.Clone();
        switch (profile)
        {
            case "hostile-arabic":
                culture.NumberFormat.NumberDecimalSeparator = "\u066b";
                culture.NumberFormat.NumberGroupSeparator = "\u066c";
                culture.NumberFormat.NegativeSign = "\u2212";
                culture.NumberFormat.PositiveSign = "\u002b\u002b";
                culture.NumberFormat.CurrencySymbol = "\u00a4A";
                culture.DateTimeFormat.DateSeparator = "*";
                culture.DateTimeFormat.TimeSeparator = "!";
                culture.DateTimeFormat.ShortDatePattern = "dd*MM*yyyy";
                break;
            case "hostile-comma":
                culture.NumberFormat.NumberDecimalSeparator = ",";
                culture.NumberFormat.NumberGroupSeparator = ".";
                culture.NumberFormat.NegativeSign = "~";
                culture.NumberFormat.PositiveSign = "!";
                culture.NumberFormat.CurrencySymbol = "XYZ";
                culture.DateTimeFormat.DateSeparator = ".";
                culture.DateTimeFormat.TimeSeparator = "-";
                culture.DateTimeFormat.ShortDatePattern = "yyyy.MM.dd";
                break;
            case "hostile-swap":
                culture.NumberFormat.NumberDecimalSeparator = ";";
                culture.NumberFormat.NumberGroupSeparator = ",";
                culture.NumberFormat.NegativeSign = "NEG";
                culture.NumberFormat.PositiveSign = "POS";
                culture.NumberFormat.CurrencySymbol = "CUR";
                culture.DateTimeFormat.DateSeparator = "_";
                culture.DateTimeFormat.TimeSeparator = ".";
                culture.DateTimeFormat.ShortDatePattern = "MM_dd_yyyy";
                break;
            default:
                throw new ProbeFailure("CULTURE_PROFILE");
        }
        return CultureInfo.ReadOnly(culture);
    }

    private static void AddStringVectors()
    {
        const string sequenceDomain =
            "null or 0..16384 UTF-16 code units as stated per operation; lone surrogates are code units";
        const string nonNullDomain =
            "non-null 0..16384 UTF-16 code units; indices and lengths are exact int32 values";
        string pair = "\ud83d\ude00";
        string loneHigh = "\ud800";
        string loneLow = "\udfff";

        AddMeasuredValue(
            "string.literal.empty",
            "string.utf16",
            "string.literal.decode",
            sequenceDomain,
            new[] { "literal=empty" },
            "utf16_hex",
            () => Utf16(string.Empty));
        AddMeasuredValue(
            "string.literal.bmp",
            "string.utf16",
            "string.literal.decode",
            sequenceDomain,
            new[] { "literal=0041,3042" },
            "utf16_hex",
            () => Utf16("A\u3042"));
        AddMeasuredValue(
            "string.literal.surrogate_pair",
            "string.utf16",
            "string.literal.decode",
            sequenceDomain,
            new[] { "literal=d83d,de00" },
            "utf16_hex",
            () => Utf16(pair));
        AddMeasuredValue(
            "string.literal.lone_high",
            "string.utf16",
            "string.literal.decode",
            sequenceDomain,
            new[] { "literal=d800" },
            "utf16_hex",
            () => Utf16(loneHigh));
        AddMeasuredValue(
            "string.literal.lone_low",
            "string.utf16",
            "string.literal.decode",
            sequenceDomain,
            new[] { "literal=dfff" },
            "utf16_hex",
            () => Utf16(loneLow));

        foreach ((string id, string value) in new[]
        {
            ("empty", string.Empty),
            ("pair", pair),
            ("lone_high", loneHigh),
        })
        {
            AddMeasuredValue(
                "string.length." + id,
                "string.utf16",
                "string.length",
                nonNullDomain,
                new[] { "value_utf16=" + Utf16(value) },
                "signed_decimal",
                () => value.Length.ToString(CultureInfo.InvariantCulture));
        }
        AddMeasuredError(
            "string.length.null_receiver",
            "string.null",
            "string.length",
            nonNullDomain,
            new[] { "value=null" },
            "exception.null_receiver",
            () =>
            {
                string? value = null;
                return value!.Length.ToString(CultureInfo.InvariantCulture);
            });

        AddMeasuredValue(
            "string.index.pair_high",
            "string.utf16",
            "string.index",
            nonNullDomain,
            new[] { "value_utf16=" + Utf16(pair), "index=0" },
            "u16_hex",
            () => U16(pair[0]));
        AddMeasuredValue(
            "string.index.pair_low",
            "string.utf16",
            "string.index",
            nonNullDomain,
            new[] { "value_utf16=" + Utf16(pair), "index=1" },
            "u16_hex",
            () => U16(pair[1]));
        AddMeasuredValue(
            "string.index.lone_high",
            "string.utf16",
            "string.index",
            nonNullDomain,
            new[] { "value_utf16=" + Utf16(loneHigh), "index=0" },
            "u16_hex",
            () => U16(loneHigh[0]));
        AddMeasuredError(
            "string.index.negative",
            "string.range",
            "string.index",
            nonNullDomain,
            new[] { "value_utf16=0061", "index=-1" },
            "exception.range",
            () => U16("a"[-1]));
        AddMeasuredError(
            "string.index.at_length",
            "string.range",
            "string.index",
            nonNullDomain,
            new[] { "value_utf16=0061", "index=1" },
            "exception.range",
            () => U16("a"[1]));
        AddMeasuredError(
            "string.index.null_receiver",
            "string.null",
            "string.index",
            nonNullDomain,
            new[] { "value=null", "index=0" },
            "exception.null_receiver",
            () =>
            {
                string? value = null;
                return U16(value![0]);
            });

        AddBinaryStringBoolean(
            "string.equality.equal",
            "string.equality.operator",
            "0061",
            "0061",
            () => "a" == new string(new[] { 'a' }));
        AddBinaryStringBoolean(
            "string.equality.case",
            "string.equality.operator",
            "0041",
            "0061",
            () => "A" == "a");
        AddBinaryStringBoolean(
            "string.equality.null_null",
            "string.equality.operator",
            "null",
            "null",
            () => (string?)null == (string?)null);
        AddBinaryStringBoolean(
            "string.equality.null_empty",
            "string.equality.operator",
            "null",
            "",
            () => (string?)null == string.Empty);
        AddBinaryStringBoolean(
            "string.inequality.null_empty",
            "string.inequality.operator",
            "null",
            "",
            () => (string?)null != string.Empty);
        AddBinaryStringBoolean(
            "string.equals_ordinal.lone_equal",
            "string.equals.ordinal",
            "d800",
            "d800",
            () => string.Equals(loneHigh, "\ud800", StringComparison.Ordinal));
        AddBinaryStringBoolean(
            "string.equals_ordinal.null_empty",
            "string.equals.ordinal",
            "null",
            "",
            () => string.Equals(null, string.Empty, StringComparison.Ordinal));

        foreach ((string id, string? left, string? right) in new[]
        {
            ("null_null", null, null),
            ("null_value", null, "a"),
            ("value_null", "a", null),
            ("case", "A", "a"),
            ("surrogate", loneHigh, loneLow),
        })
        {
            string? capturedLeft = left;
            string? capturedRight = right;
            AddMeasuredValue(
                "string.compare_ordinal." + id,
                "string.ordinal",
                "string.compare.ordinal",
                sequenceDomain,
                new[]
                {
                    "left_utf16=" + Utf16(capturedLeft),
                    "right_utf16=" + Utf16(capturedRight),
                },
                "comparison_sign",
                () => Sign(string.Compare(capturedLeft, capturedRight, StringComparison.Ordinal)));
        }

        AddOrdinalPredicateVectors(
            "starts_with",
            "abc",
            "a",
            "Abc",
            "a",
            (receiver, argument) => receiver.StartsWith(argument, StringComparison.Ordinal));
        AddOrdinalPredicateVectors(
            "ends_with",
            "abc",
            "c",
            "abC",
            "c",
            (receiver, argument) => receiver.EndsWith(argument, StringComparison.Ordinal));
        AddOrdinalPredicateVectors(
            "contains",
            "abc",
            "b",
            "aBc",
            "b",
            (receiver, argument) => receiver.Contains(argument, StringComparison.Ordinal));

        AddConcatVectors();

        AddMeasuredValue(
            "string.substring.middle",
            "string.substring",
            "string.substring.start_length",
            nonNullDomain,
            new[] { "value_utf16=006100620063", "start=1", "length=1" },
            "utf16_hex",
            () => Utf16("abc".Substring(1, 1)));
        AddMeasuredValue(
            "string.substring.lone",
            "string.substring",
            "string.substring.start_length",
            nonNullDomain,
            new[] { "value_utf16=" + Utf16(pair), "start=0", "length=1" },
            "utf16_hex",
            () => Utf16(pair.Substring(0, 1)));
        AddMeasuredValue(
            "string.substring.empty_at_end",
            "string.substring",
            "string.substring.start_length",
            nonNullDomain,
            new[] { "value_utf16=0061", "start=1", "length=0" },
            "utf16_hex",
            () => Utf16("a".Substring(1, 0)));
        AddMeasuredError(
            "string.substring.negative_start",
            "string.range",
            "string.substring.start_length",
            nonNullDomain,
            new[] { "value_utf16=0061", "start=-1", "length=1" },
            "exception.range",
            () => Utf16("a".Substring(-1, 1)));
        AddMeasuredError(
            "string.substring.excess_length",
            "string.range",
            "string.substring.start_length",
            nonNullDomain,
            new[] { "value_utf16=0061", "start=0", "length=2" },
            "exception.range",
            () => Utf16("a".Substring(0, 2)));
        AddMeasuredError(
            "string.substring.null_receiver",
            "string.null",
            "string.substring.start_length",
            nonNullDomain,
            new[] { "value=null", "start=0", "length=0" },
            "exception.null_receiver",
            () =>
            {
                string? value = null;
                return Utf16(value!.Substring(0, 0));
            });

        foreach ((string id, string? value) in new[]
        {
            ("null", null),
            ("empty", string.Empty),
            ("value", "a"),
        })
        {
            string? captured = value;
            AddMeasuredValue(
                "string.is_null_or_empty." + id,
                "string.null",
                "string.is_null_or_empty",
                sequenceDomain,
                new[] { "value_utf16=" + Utf16(captured) },
                "bool",
                () => Bool(string.IsNullOrEmpty(captured)));
        }

        foreach ((string id, string? value) in new[]
        {
            ("null", null),
            ("empty", string.Empty),
            ("exact", "case"),
            ("case_miss", "Case"),
            ("default", "other"),
        })
        {
            string? captured = value;
            AddMeasuredValue(
                "string.switch_constant." + id,
                "string.ordinal",
                "string.switch.constant",
                sequenceDomain,
                new[] { "value_utf16=" + Utf16(captured) },
                "signed_decimal",
                () => (captured switch
                {
                    null => -1,
                    "" => 0,
                    "case" => 1,
                    _ => 2,
                }).ToString(CultureInfo.InvariantCulture));
        }

        string interpolationValue = "A";
        char interpolationChar = '\ud800';
        AddMeasuredValue(
            "string.interpolation.string_char",
            "string.interpolation",
            "string.interpolation.restricted",
            "every hole has static type string or char, with no alignment or format; normalized result <=16384 UTF-16 units",
            new[] { "prefix_utf16=005b", "string_utf16=0041", "char=d800", "suffix_utf16=005d" },
            "utf16_hex",
            () => Utf16($"[{interpolationValue}{interpolationChar}]"));
        AddRejected(
            "string.interpolation.numeric",
            "string.interpolation",
            "string.interpolation.rejected_numeric",
            "any interpolation hole whose static type is not string or char is outside the profile",
            new[] { "type=decimal", "value=negative_1234_5" },
            "source_rejection.interpolation_hole_type",
            "utf16_hex",
            () => Utf16($"{-1234.5m}"),
            true);
        AddRejected(
            "string.interpolation.alignment",
            "string.interpolation",
            "string.interpolation.rejected_alignment",
            "any alignment component is outside the profile",
            new[] { "type=string", "alignment=3", "value_utf16=0061" },
            "source_rejection.interpolation_alignment",
            "utf16_hex",
            () => Utf16($"{"a",3}"),
            false);
        AddRejected(
            "string.interpolation.format",
            "string.interpolation",
            "string.interpolation.rejected_format",
            "any format component is outside the profile",
            new[] { "type=char", "format_utf16=0047", "value=0061" },
            "source_rejection.interpolation_format",
            "utf16_hex",
            () => Utf16($"{'a':G}"),
            false);
        AddRejected(
            "string.case_conversion.ambient",
            "string.culture_rejection",
            "string.rejected_culture_operation",
            "culture-sensitive string transformations are outside the profile",
            new[] { "operation=to_upper", "value_utf16=0069" },
            "source_rejection.culture_sensitive_string",
            "utf16_hex",
            () => Utf16("i".ToUpper()),
            true);
    }

    private static void AddBinaryStringBoolean(
        string id,
        string operation,
        string left,
        string right,
        Func<bool> action)
    {
        AddMeasuredValue(
            id,
            "string.ordinal",
            operation,
            "two null or bounded UTF-16 string values; equality is ordinal and null differs from empty",
            new[] { "left_utf16=" + left, "right_utf16=" + right },
            "bool",
            () => Bool(action()));
    }

    private static void AddOrdinalPredicateVectors(
        string name,
        string trueReceiver,
        string trueArgument,
        string falseReceiver,
        string falseArgument,
        Func<string, string, bool> operation)
    {
        const string domain =
            "non-null bounded UTF-16 receiver and non-null bounded UTF-16 argument; StringComparison.Ordinal appears only in this argument position";
        AddMeasuredValue(
            "string." + name + ".true",
            "string.ordinal",
            "string." + name + ".ordinal",
            domain,
            new[]
            {
                "receiver_utf16=" + Utf16(trueReceiver),
                "argument_utf16=" + Utf16(trueArgument),
            },
            "bool",
            () => Bool(operation(trueReceiver, trueArgument)));
        AddMeasuredValue(
            "string." + name + ".case_false",
            "string.ordinal",
            "string." + name + ".ordinal",
            domain,
            new[]
            {
                "receiver_utf16=" + Utf16(falseReceiver),
                "argument_utf16=" + Utf16(falseArgument),
            },
            "bool",
            () => Bool(operation(falseReceiver, falseArgument)));
        AddMeasuredError(
            "string." + name + ".null_receiver",
            "string.null",
            "string." + name + ".ordinal",
            domain,
            new[] { "receiver=null", "argument_utf16=0061" },
            "exception.null_receiver",
            () => operation(null!, "a").ToString());
        AddMeasuredError(
            "string." + name + ".null_argument",
            "string.null",
            "string." + name + ".ordinal",
            domain,
            new[] { "receiver_utf16=0061", "argument=null" },
            "exception.null_argument",
            () => operation("a", null!).ToString());
    }

    private static void AddConcatVectors()
    {
        const string domain =
            "the listed arity and static operand types only; each string is null or bounded UTF-16, each char is one uint16, and result length <=16384";
        foreach ((string id, string? left, string? right) in new[]
        {
            ("values", "a", "b"),
            ("left_null", null, "b"),
            ("right_null", "a", null),
            ("both_null", null, null),
        })
        {
            string? capturedLeft = left;
            string? capturedRight = right;
            AddMeasuredValue(
                "string.concat.operator.string_string." + id,
                "string.concat",
                "string.concat.operator.string_string",
                domain,
                new[]
                {
                    "left_utf16=" + Utf16(capturedLeft),
                    "right_utf16=" + Utf16(capturedRight),
                },
                "utf16_hex",
                () => Utf16(capturedLeft + capturedRight));
            AddMeasuredValue(
                "string.concat.string2." + id,
                "string.concat",
                "string.concat.string2",
                domain,
                new[]
                {
                    "arg0_utf16=" + Utf16(capturedLeft),
                    "arg1_utf16=" + Utf16(capturedRight),
                },
                "utf16_hex",
                () => Utf16(string.Concat(capturedLeft, capturedRight)));
        }

        string? nullString = null;
        char lone = '\ud800';
        AddMeasuredValue(
            "string.concat.operator.string_char.value",
            "string.concat",
            "string.concat.operator.string_char",
            domain,
            new[] { "left_utf16=0061", "right_char=d800" },
            "utf16_hex",
            () => Utf16("a" + lone));
        AddMeasuredValue(
            "string.concat.operator.string_char.null",
            "string.concat",
            "string.concat.operator.string_char",
            domain,
            new[] { "left=null", "right_char=d800" },
            "utf16_hex",
            () => Utf16(nullString + lone));
        AddMeasuredValue(
            "string.concat.operator.char_string.value",
            "string.concat",
            "string.concat.operator.char_string",
            domain,
            new[] { "left_char=d800", "right_utf16=0061" },
            "utf16_hex",
            () => Utf16(lone + "a"));
        AddMeasuredValue(
            "string.concat.operator.char_string.null",
            "string.concat",
            "string.concat.operator.char_string",
            domain,
            new[] { "left_char=d800", "right=null" },
            "utf16_hex",
            () => Utf16(lone + nullString));

        AddMeasuredValue(
            "string.concat.string3.null_middle",
            "string.concat",
            "string.concat.string3",
            domain,
            new[] { "arg0_utf16=0061", "arg1=null", "arg2_utf16=0062" },
            "utf16_hex",
            () => Utf16(string.Concat("a", null, "b")));
        AddMeasuredValue(
            "string.concat.string4.null_edges",
            "string.concat",
            "string.concat.string4",
            domain,
            new[] { "arg0=null", "arg1_utf16=0061", "arg2_utf16=0062", "arg3=null" },
            "utf16_hex",
            () => Utf16(string.Concat(null, "a", "b", null)));

        AddRejected(
            "string.concat.char_char",
            "string.concat",
            "string.concat.rejected_char_char",
            "char + char has no string operand and is outside the profile",
            new[] { "left_char=0061", "right_char=0062" },
            "source_rejection.concat_char_char",
            "signed_decimal",
            () => ('a' + 'b').ToString(CultureInfo.InvariantCulture),
            false);
        AddRejected(
            "string.concat.object",
            "string.concat",
            "string.concat.rejected_object",
            "object conversion, boxing, and implicit ToString concatenation are outside the profile",
            new[] { "left_utf16=0078", "right_static_type=object", "right_value=int32_1" },
            "source_rejection.concat_object_conversion",
            "utf16_hex",
            () => Utf16("x" + (object)1),
            true);
    }

    private static void AddCodecVectors()
    {
        AddIntegerCodecVectors();
        AddDecimalCodecVectors();
        AddDateTimeCodecVectors();
        AddDurationInstantCodecVectors();
        AddFloatingCodecVectors();
        AddGuidCodecVectors();
        AddCodecRoundTripVectors();
    }

    private static void AddIntegerCodecVectors()
    {
        IntegerSpec[] specs =
        {
            new("i8", true, sbyte.MinValue, sbyte.MaxValue),
            new("u8", false, byte.MinValue, byte.MaxValue),
            new("i16", true, short.MinValue, short.MaxValue),
            new("u16", false, ushort.MinValue, ushort.MaxValue),
            new("i32", true, int.MinValue, int.MaxValue),
            new("u32", false, uint.MinValue, uint.MaxValue),
            new("i64", true, long.MinValue, long.MaxValue),
            new("u64", false, ulong.MinValue, ulong.MaxValue),
        };
        foreach (IntegerSpec spec in specs)
        {
            List<(string Id, string Text)> cases = new()
            {
                ("zero", "0"),
                ("one", "1"),
                ("maximum", AsciiInteger(spec.Maximum)),
                ("overflow_high", AsciiInteger(spec.Maximum + BigInteger.One)),
                ("plus", "+1"),
                ("plus_malformed", "+x"),
                ("leading_zero", "01"),
                ("negative_zero", "-0"),
                ("whitespace", " 1"),
                ("separator", "1,0"),
                ("exponent", "1e0"),
                ("non_ascii_digit", "\u0661"),
            };
            if (spec.Signed)
            {
                cases.Add(("minimum", AsciiInteger(spec.Minimum)));
                cases.Add(("negative_one", "-1"));
                cases.Add(("overflow_low", AsciiInteger(spec.Minimum - BigInteger.One)));
                cases.Add(("hostile_negative", "~1"));
            }
            else
            {
                cases.Add(("negative", "-1"));
            }
            cases.Add(("over_bound", new string('9', TextBound + 1)));

            foreach ((string id, string input) in cases)
            {
                CodecOutcome outcome = ParseInteger(input, spec);
                AddCodec(
                    "codec.integer." + spec.Id + ".parse." + id,
                    "codec.integer",
                    "codec.integer." + spec.Id + ".parse",
                    (spec.Signed ? "signed" : "unsigned")
                        + " canonical ASCII base-10 in ["
                        + AsciiInteger(spec.Minimum)
                        + ","
                        + AsciiInteger(spec.Maximum)
                        + "] and <=16384 UTF-16 units; no plus, whitespace, separator, exponent, non-ASCII digit, leading zero, or negative zero",
                    new[] { InputDescription(input) },
                    outcome,
                    Observe(() => BclIntegerParse(spec.Id, input), "bcl_general_parse"),
                    true);
            }

            IEnumerable<string> formats = spec.Signed
                ? new[]
                {
                    AsciiInteger(spec.Minimum),
                    "0",
                    AsciiInteger(spec.Maximum),
                }
                : new[] { "0", AsciiInteger(spec.Maximum) };
            foreach (string canonical in formats)
            {
                string id = canonical.StartsWith("-", StringComparison.Ordinal)
                    ? "minimum"
                    : canonical == "0" ? "zero" : "maximum";
                AddCodec(
                    "codec.integer." + spec.Id + ".format." + id,
                    "codec.integer",
                    "codec.integer." + spec.Id + ".format",
                    "every value of " + spec.Id + "; output is the unique canonical ASCII base-10 spelling and <=16384 UTF-16 units",
                    new[] { "value=" + canonical },
                    new CodecOutcome("value", "ascii", canonical, null),
                    Observe(() => Utf16(BclIntegerFormat(spec.Id, canonical)), "utf16_hex"),
                    true);
            }
        }
    }

    private static CodecOutcome ParseInteger(string input, IntegerSpec spec)
    {
        if (input.Length > TextBound)
        {
            return CodecError("parse_error.input_bound");
        }
        if (input.Length == 0)
        {
            return CodecError("parse_error.syntax");
        }
        bool negative = input[0] == '-';
        bool positiveSign = input[0] == '+';
        if (negative && !spec.Signed)
        {
            return CodecError("parse_error.syntax");
        }
        int start = negative || positiveSign ? 1 : 0;
        if (start == input.Length)
        {
            return CodecError("parse_error.syntax");
        }
        for (int index = start; index < input.Length; index++)
        {
            if (input[index] < '0' || input[index] > '9')
            {
                return CodecError("parse_error.syntax");
            }
        }
        if (positiveSign || (input.Length - start > 1 && input[start] == '0'))
        {
            return CodecError("parse_error.noncanonical");
        }
        if (negative && input[start] == '0' && input.Length - start == 1)
        {
            return CodecError("parse_error.noncanonical");
        }
        BigInteger magnitude = BigInteger.Zero;
        for (int index = start; index < input.Length; index++)
        {
            magnitude = magnitude * 10 + (input[index] - '0');
        }
        BigInteger value = negative ? -magnitude : magnitude;
        if (value < spec.Minimum || value > spec.Maximum)
        {
            return CodecError("parse_error.range");
        }
        return new CodecOutcome("value", "ascii", input, null);
    }

    private static string BclIntegerParse(string id, string input)
    {
        NumberStyles style = NumberStyles.Integer;
        IFormatProvider culture = CultureInfo.CurrentCulture;
        return id switch
        {
            "i8" => sbyte.TryParse(input, style, culture, out sbyte value)
                ? value.ToString(CultureInfo.InvariantCulture)
                : "parse_false",
            "u8" => byte.TryParse(input, style, culture, out byte value)
                ? value.ToString(CultureInfo.InvariantCulture)
                : "parse_false",
            "i16" => short.TryParse(input, style, culture, out short value)
                ? value.ToString(CultureInfo.InvariantCulture)
                : "parse_false",
            "u16" => ushort.TryParse(input, style, culture, out ushort value)
                ? value.ToString(CultureInfo.InvariantCulture)
                : "parse_false",
            "i32" => int.TryParse(input, style, culture, out int value)
                ? value.ToString(CultureInfo.InvariantCulture)
                : "parse_false",
            "u32" => uint.TryParse(input, style, culture, out uint value)
                ? value.ToString(CultureInfo.InvariantCulture)
                : "parse_false",
            "i64" => long.TryParse(input, style, culture, out long value)
                ? value.ToString(CultureInfo.InvariantCulture)
                : "parse_false",
            "u64" => ulong.TryParse(input, style, culture, out ulong value)
                ? value.ToString(CultureInfo.InvariantCulture)
                : "parse_false",
            _ => throw new ProbeFailure("INTEGER_TYPE"),
        };
    }

    private static string BclIntegerFormat(string id, string canonical)
    {
        CultureInfo invariant = CultureInfo.InvariantCulture;
        return id switch
        {
            "i8" => sbyte.Parse(canonical, invariant).ToString(),
            "u8" => byte.Parse(canonical, invariant).ToString(),
            "i16" => short.Parse(canonical, invariant).ToString(),
            "u16" => ushort.Parse(canonical, invariant).ToString(),
            "i32" => int.Parse(canonical, invariant).ToString(),
            "u32" => uint.Parse(canonical, invariant).ToString(),
            "i64" => long.Parse(canonical, invariant).ToString(),
            "u64" => ulong.Parse(canonical, invariant).ToString(),
            _ => throw new ProbeFailure("INTEGER_TYPE"),
        };
    }

    private static void AddDecimalCodecVectors()
    {
        foreach ((string id, string input) in new[]
        {
            ("zero", "0"),
            ("negative", "-12.34"),
            ("maximum", "79228162514264337593543950335"),
            ("minimum_fraction", "0.0000000000000000000000000001"),
            ("plus", "+1"),
            ("plus_malformed", "+x"),
            ("leading_zero", "01"),
            ("negative_zero", "-0"),
            ("empty_fraction", "1."),
            ("trailing_zero", "1.20"),
            ("scale_29", "0.00000000000000000000000000001"),
            ("coefficient_overflow", "79228162514264337593543950336"),
            ("comma", "1,5"),
            ("whitespace", " 1"),
            ("over_bound", new string('9', TextBound + 1)),
        })
        {
            CodecOutcome outcome = ParseDecimal(input, null);
            AddCodec(
                "codec.decimal.normalized.parse." + id,
                "codec.decimal",
                "codec.decimal.normalized.parse",
                "canonical ASCII fixed-point decimal with optional minus only for a negative value, canonical integer part, optional nonzero-final fractional part, scale 0..28, 96-bit coefficient, and <=16384 UTF-16 units",
                new[] { InputDescription(input) },
                outcome,
                Observe(() => BclDecimalParse(input), "decimal_bits"),
                true);
        }

        foreach (decimal value in new[]
        {
            decimal.Zero,
            -12.3400m,
            decimal.MaxValue,
            new decimal(1, 0, 0, false, 28),
        })
        {
            string canonical = FormatDecimalNormalized(value);
            AddCodec(
                "codec.decimal.normalized.format." + SafeId(canonical),
                "codec.decimal",
                "codec.decimal.normalized.format",
                "every decimal value; normalize value-equivalent trailing-zero scale and negative zero, emit unique ASCII fixed-point text, and satisfy output bound",
                new[] { "value=" + DecimalEncoding(value) },
                new CodecOutcome("value", "ascii", canonical, null),
                Observe(() => Utf16(value.ToString()), "utf16_hex"),
                true);
        }

        foreach ((string id, string input, int scale) in new[]
        {
            ("scale0", "12", 0),
            ("scale2", "12.30", 2),
            ("scale28", "0.0000000000000000000000000001", 28),
            ("maximum_scale2", "79228162514264337593543950335.00", 2),
            ("maximum_scale28", "79228162514264337593543950335.0000000000000000000000000000", 28),
            ("unrepresentable_coefficient", "7922816251426433759354395033.51", 2),
            ("wrong_digits", "12.3", 2),
            ("trailing_zero_allowed", "12.30", 2),
            ("leading_zero", "012.30", 2),
            ("negative_zero", "-0.00", 2),
            ("plus", "+12.30", 2),
            ("plus_malformed", "+x", 2),
            ("syntax", "12,30", 2),
            ("over_bound", new string('9', TextBound + 1), 2),
            ("scale_outside", "12.30", 29),
            ("syntax_before_scale", "12x", 29),
            ("noncanonical_before_scale", "+1", 29),
            ("range", "79228162514264337593543950336.00", 2),
        })
        {
            CodecOutcome outcome = ParseDecimal(input, scale);
            AddCodec(
                "codec.decimal.fixed.parse." + id,
                "codec.decimal",
                "codec.decimal.fixed.parse",
                "scale is 0..28; text is canonical ASCII fixed-point with exactly scale fractional digits, canonical integer part, non-negative zero, exact 96-bit representability after removing only trailing-zero padding when necessary, and <=16384 UTF-16 units",
                new[] { InputDescription(input), "scale=" + scale.ToString(CultureInfo.InvariantCulture) },
                outcome,
                Observe(() => BclDecimalParse(input), "decimal_bits"),
                true);
        }

        decimal[] values = { 1.245m, 1.255m, -1.245m, -1.255m };
        MidpointRounding[] modes =
        {
            MidpointRounding.ToEven,
            MidpointRounding.AwayFromZero,
            MidpointRounding.ToZero,
            MidpointRounding.ToNegativeInfinity,
            MidpointRounding.ToPositiveInfinity,
        };
        foreach (decimal value in values)
        {
            foreach (MidpointRounding mode in modes)
            {
                decimal rounded = decimal.Round(value, 2, mode);
                string canonical = FormatDecimalFixed(rounded, 2);
                AddCodec(
                    "codec.decimal.fixed.format."
                        + SafeId(FormatDecimalNormalized(value))
                        + "."
                        + mode.ToString().ToLowerInvariant(),
                    "codec.decimal",
                    "codec.decimal.fixed.format",
                    "decimal value, scale 0..28, and one of ToEven/AwayFromZero/ToZero/ToNegativeInfinity/ToPositiveInfinity; round with the selected mode, emit exactly scale fractional ASCII digits, and satisfy output bound",
                    new[]
                    {
                        "value=" + DecimalEncoding(value),
                        "scale=2",
                        "rounding=" + mode,
                    },
                    new CodecOutcome("value", "ascii", canonical, null),
                    Observe(() => Utf16(value.ToString("F2")), "utf16_hex"),
                    true);
            }
        }
    }

    private static CodecOutcome ParseDecimal(string input, int? fixedScale)
    {
        return ParseDecimal(input, fixedScale, out _);
    }

    private static CodecOutcome ParseDecimal(
        string input,
        int? fixedScale,
        out decimal parsedValue)
    {
        parsedValue = decimal.Zero;
        if (input.Length > TextBound)
        {
            return CodecError("parse_error.input_bound");
        }
        if (input.Length == 0)
        {
            return CodecError("parse_error.syntax");
        }
        bool negative = input[0] == '-';
        bool positiveSign = input[0] == '+';
        int start = negative || positiveSign ? 1 : 0;
        int dot = -1;
        for (int index = start; index < input.Length; index++)
        {
            char character = input[index];
            if (character == '.')
            {
                if (dot >= 0)
                {
                    return CodecError("parse_error.syntax");
                }
                dot = index;
            }
            else if (character < '0' || character > '9')
            {
                return CodecError("parse_error.syntax");
            }
        }
        int integerEnd = dot < 0 ? input.Length : dot;
        if (integerEnd <= start || dot == input.Length - 1)
        {
            return CodecError("parse_error.syntax");
        }
        int integerDigits = integerEnd - start;
        if (positiveSign || (integerDigits > 1 && input[start] == '0'))
        {
            return CodecError("parse_error.noncanonical");
        }
        int scale = dot < 0 ? 0 : input.Length - dot - 1;
        if (fixedScale is null && scale > 0 && input[^1] == '0')
        {
            return CodecError("parse_error.noncanonical");
        }
        if (negative && IsAllZeroDigits(input, start))
        {
            return CodecError("parse_error.noncanonical");
        }
        if (fixedScale is < 0 or > 28
            || (fixedScale is int required && scale != required))
        {
            return CodecError("parse_error.scale_precision");
        }
        if (scale > 28)
        {
            return CodecError("parse_error.scale_precision");
        }

        BigInteger coefficient = BigInteger.Zero;
        for (int index = start; index < input.Length; index++)
        {
            if (input[index] != '.')
            {
                coefficient = coefficient * 10 + (input[index] - '0');
            }
        }
        BigInteger maximum = (BigInteger.One << 96) - BigInteger.One;
        while (coefficient > maximum && scale > 0 && coefficient % 10 == 0)
        {
            coefficient /= 10;
            scale--;
        }
        if (coefficient > maximum)
        {
            return CodecError("parse_error.range");
        }
        decimal value = DecimalFromParts(coefficient, negative, scale);
        parsedValue = value;
        return new CodecOutcome("value", "decimal_bits", DecimalEncoding(value), null);
    }

    private static bool IsAllZeroDigits(string input, int start)
    {
        for (int index = start; index < input.Length; index++)
        {
            if (input[index] != '.' && input[index] != '0')
            {
                return false;
            }
        }
        return true;
    }

    private static decimal DecimalFromParts(BigInteger coefficient, bool negative, int scale)
    {
        uint low = (uint)(coefficient & uint.MaxValue);
        uint middle = (uint)((coefficient >> 32) & uint.MaxValue);
        uint high = (uint)((coefficient >> 64) & uint.MaxValue);
        return new decimal(
            unchecked((int)low),
            unchecked((int)middle),
            unchecked((int)high),
            negative,
            (byte)scale);
    }

    private static string BclDecimalParse(string input)
    {
        return decimal.TryParse(input, NumberStyles.Number, CultureInfo.CurrentCulture, out decimal value)
            ? DecimalEncoding(value)
            : "parse_false";
    }

    private static string FormatDecimalNormalized(decimal value)
    {
        int[] bits = decimal.GetBits(value);
        bool negative = (bits[3] & unchecked((int)0x80000000)) != 0;
        int scale = (bits[3] >> 16) & 0xff;
        BigInteger coefficient =
            unchecked((uint)bits[0])
            | ((BigInteger)unchecked((uint)bits[1]) << 32)
            | ((BigInteger)unchecked((uint)bits[2]) << 64);
        while (scale > 0 && coefficient % 10 == 0)
        {
            coefficient /= 10;
            scale--;
        }
        if (coefficient.IsZero)
        {
            return "0";
        }
        string digits = AsciiInteger(coefficient);
        string body;
        if (scale == 0)
        {
            body = digits;
        }
        else if (digits.Length <= scale)
        {
            body = "0." + new string('0', scale - digits.Length) + digits;
        }
        else
        {
            body = digits.Insert(digits.Length - scale, ".");
        }
        return negative ? "-" + body : body;
    }

    private static string FormatDecimalFixed(decimal value, int scale)
    {
        decimal rounded = decimal.Round(value, scale, MidpointRounding.ToEven);
        string normalized = FormatDecimalNormalized(rounded);
        bool negative = normalized.StartsWith("-", StringComparison.Ordinal);
        string body = negative ? normalized[1..] : normalized;
        int dot = body.IndexOf('.', StringComparison.Ordinal);
        string integer = dot < 0 ? body : body[..dot];
        string fraction = dot < 0 ? string.Empty : body[(dot + 1)..];
        string result = scale == 0
            ? integer
            : integer + "." + fraction.PadRight(scale, '0');
        return negative && result != "0" ? "-" + result : result;
    }

    private static void AddDateTimeCodecVectors()
    {
        foreach ((string id, string input) in new[]
        {
            ("leap", "2024-02-29"),
            ("minimum", "0001-01-01"),
            ("maximum", "9999-12-31"),
            ("invalid_day", "2023-02-29"),
            ("year_zero", "0000-01-01"),
            ("slash", "2024/02/29"),
            ("short", "2024-2-29"),
            ("whitespace", " 2024-02-29"),
            ("non_ascii_digit", "\u0662\u0660\u0662\u0664-02-29"),
            ("over_bound", new string('9', TextBound + 1)),
        })
        {
            AddCodec(
                "codec.date.parse." + id,
                "codec.date_time",
                "codec.date.parse",
                "exactly 10 ASCII units yyyy-MM-dd, Gregorian year 0001..9999, and a valid calendar date; input <=16384 UTF-16 units",
                new[] { InputDescription(input) },
                ParseDate(input),
                Observe(() => BclDateParse(input), "date_day_number"),
                true);
        }
        foreach (DateOnly value in new[]
        {
            DateOnly.MinValue,
            new DateOnly(2024, 2, 29),
            DateOnly.MaxValue,
        })
        {
            string candidate = Four(value.Year) + "-" + Two(value.Month) + "-" + Two(value.Day);
            AddCodec(
                "codec.date.format." + candidate.Replace('-', '_'),
                "codec.date_time",
                "codec.date.format",
                "every DateOnly value; output exactly 10 ASCII units yyyy-MM-dd and satisfy output bound",
                new[] { "day_number=" + value.DayNumber.ToString(CultureInfo.InvariantCulture) },
                new CodecOutcome("value", "ascii", candidate, null),
                Observe(() => Utf16(value.ToString()), "utf16_hex"),
                true);
        }

        foreach ((string id, string input) in new[]
        {
            ("minimum", "00:00:00.0000000"),
            ("maximum", "23:59:59.9999999"),
            ("sample", "12:34:56.1234567"),
            ("hour_24", "24:00:00.0000000"),
            ("second_60", "23:59:60.0000000"),
            ("six_fraction", "12:34:56.123456"),
            ("comma", "12:34:56,1234567"),
            ("short", "1:34:56.1234567"),
            ("over_bound", new string('9', TextBound + 1)),
        })
        {
            AddCodec(
                "codec.time.parse." + id,
                "codec.date_time",
                "codec.time.parse",
                "exactly 16 ASCII units HH:mm:ss.fffffff, 24-hour range 00:00:00.0000000 through 23:59:59.9999999, and input <=16384 UTF-16 units",
                new[] { InputDescription(input) },
                ParseTime(input),
                Observe(() => BclTimeParse(input), "time_ticks"),
                true);
        }
        foreach (TimeOnly value in new[]
        {
            TimeOnly.MinValue,
            new TimeOnly(12, 34, 56).Add(TimeSpan.FromTicks(1_234_567)),
            TimeOnly.MaxValue,
        })
        {
            string candidate = FormatTime(value.Ticks);
            AddCodec(
                "codec.time.format." + value.Ticks.ToString("d19", CultureInfo.InvariantCulture),
                "codec.date_time",
                "codec.time.format",
                "every TimeOnly tick value 0..863999999999; output exactly 16 ASCII units HH:mm:ss.fffffff and satisfy output bound",
                new[] { "ticks=" + value.Ticks.ToString(CultureInfo.InvariantCulture) },
                new CodecOutcome("value", "ascii", candidate, null),
                Observe(() => Utf16(value.ToString()), "utf16_hex"),
                true);
        }
    }

    private static CodecOutcome ParseDate(string input)
    {
        if (input.Length > TextBound)
        {
            return CodecError("parse_error.input_bound");
        }
        if (input.Length != 10 || input[4] != '-' || input[7] != '-')
        {
            return CodecError("parse_error.syntax");
        }
        if (!AsciiDigitsExcept(input, 4, 7))
        {
            return CodecError("parse_error.syntax");
        }
        int year = Digits(input, 0, 4);
        int month = Digits(input, 5, 2);
        int day = Digits(input, 8, 2);
        if (year < 1 || month < 1 || month > 12)
        {
            return CodecError("parse_error.range");
        }
        int[] days = { 31, IsLeapYear(year) ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 };
        if (day < 1 || day > days[month - 1])
        {
            return CodecError("parse_error.range");
        }
        DateOnly value = new(year, month, day);
        return new CodecOutcome(
            "value",
            "date_day_number",
            AsciiInteger(value.DayNumber),
            null);
    }

    private static string BclDateParse(string input)
    {
        return DateOnly.TryParse(input, CultureInfo.CurrentCulture, DateTimeStyles.None, out DateOnly value)
            ? value.DayNumber.ToString(CultureInfo.InvariantCulture)
            : "parse_false";
    }

    private static CodecOutcome ParseTime(string input)
    {
        if (input.Length > TextBound)
        {
            return CodecError("parse_error.input_bound");
        }
        if (input.Length != 16 || input[2] != ':' || input[5] != ':' || input[8] != '.')
        {
            return CodecError("parse_error.syntax");
        }
        if (!AsciiDigitsExcept(input, 2, 5, 8))
        {
            return CodecError("parse_error.syntax");
        }
        int hour = Digits(input, 0, 2);
        int minute = Digits(input, 3, 2);
        int second = Digits(input, 6, 2);
        int fraction = Digits(input, 9, 7);
        if (hour > 23 || minute > 59 || second > 59)
        {
            return CodecError("parse_error.range");
        }
        long ticks = (((hour * 60L + minute) * 60L) + second) * 10_000_000L + fraction;
        return new CodecOutcome(
            "value",
            "time_ticks",
            AsciiInteger(ticks),
            null);
    }

    private static string BclTimeParse(string input)
    {
        return TimeOnly.TryParse(input, CultureInfo.CurrentCulture, DateTimeStyles.None, out TimeOnly value)
            ? value.Ticks.ToString(CultureInfo.InvariantCulture)
            : "parse_false";
    }

    private static void AddDurationInstantCodecVectors()
    {
        AddSignedCarrierCodec("duration_ticks", "duration ticks");
        AddSignedCarrierCodec("unix_milliseconds", "Unix instant milliseconds");
    }

    private static void AddSignedCarrierCodec(string id, string description)
    {
        IntegerSpec spec = new("i64", true, long.MinValue, long.MaxValue);
        foreach ((string caseId, string input) in new[]
        {
            ("minimum", AsciiInteger(long.MinValue)),
            ("negative", "-1"),
            ("zero", "0"),
            ("maximum", AsciiInteger(long.MaxValue)),
            ("plus", "+1"),
            ("leading_zero", "01"),
            ("negative_zero", "-0"),
            ("whitespace", " 1"),
            ("overflow", "9223372036854775808"),
            ("over_bound", new string('9', TextBound + 1)),
        })
        {
            AddCodec(
                "codec." + id + ".parse." + caseId,
                "codec.duration_instant",
                "codec." + id + ".parse",
                "canonical signed ASCII base-10 int64 " + description + "; no plus, whitespace, separator, exponent, leading zero, or negative zero; input <=16384 UTF-16 units",
                new[] { InputDescription(input) },
                ParseInteger(input, spec),
                Observe(() => BclIntegerParse("i64", input), "bcl_general_parse"),
                true);
        }
        foreach (long value in new[] { long.MinValue, -1L, 0L, long.MaxValue })
        {
            string candidate = AsciiInteger(value);
            AddCodec(
                "codec." + id + ".format." + SafeId(candidate),
                "codec.duration_instant",
                "codec." + id + ".format",
                "every int64 " + description + "; output unique canonical signed ASCII base-10 and satisfy output bound",
                new[] { "value=" + candidate },
                new CodecOutcome("value", "ascii", candidate, null),
                Observe(() => Utf16(value.ToString()), "utf16_hex"),
                true);
        }
    }

    private static void AddFloatingCodecVectors()
    {
        foreach ((string id, string input) in new[]
        {
            ("positive_zero", "00000000"),
            ("negative_zero", "80000000"),
            ("positive_infinity", "7f800000"),
            ("quiet_nan", "7fc12345"),
            ("signaling_nan", "7fa12345"),
            ("uppercase", "7FC12345"),
            ("prefix", "0x00000000"),
            ("short", "0000000"),
            ("invalid", "0000000g"),
            ("over_bound", new string('9', TextBound + 1)),
        })
        {
            AddCodec(
                "codec.binary32.parse." + id,
                "codec.floating_bits",
                "codec.binary32.parse",
                "exactly 8 lowercase ASCII hexadecimal digits without prefix; every 32-bit payload including signed zero, infinities, and NaNs is admitted",
                new[] { InputDescription(input) },
                ParseHexBits(input, 8),
                Observe(() => BclHexParse32(input), "ieee_binary32_bits"),
                false);
        }
        foreach (string bits in new[] { "00000000", "80000000", "7f800000", "7fc12345", "7fa12345" })
        {
            uint raw = uint.Parse(bits, NumberStyles.AllowHexSpecifier, CultureInfo.InvariantCulture);
            float value = BitConverter.Int32BitsToSingle(unchecked((int)raw));
            AddCodec(
                "codec.binary32.format." + bits,
                "codec.floating_bits",
                "codec.binary32.format",
                "every IEEE binary32 bit pattern; output exactly 8 lowercase hexadecimal digits without prefix",
                new[] { "bits=" + bits },
                new CodecOutcome("value", "ascii", bits, null),
                Observe(() => Bits(value), "ieee_binary32_bits"),
                false);
        }

        foreach ((string id, string input) in new[]
        {
            ("positive_zero", "0000000000000000"),
            ("negative_zero", "8000000000000000"),
            ("positive_infinity", "7ff0000000000000"),
            ("quiet_nan", "7ff8123456789abc"),
            ("signaling_nan", "7ff0123456789abc"),
            ("uppercase", "7FF8123456789ABC"),
            ("prefix", "0x0000000000000000"),
            ("short", "000000000000000"),
            ("invalid", "000000000000000g"),
            ("over_bound", new string('9', TextBound + 1)),
        })
        {
            AddCodec(
                "codec.binary64.parse." + id,
                "codec.floating_bits",
                "codec.binary64.parse",
                "exactly 16 lowercase ASCII hexadecimal digits without prefix; every 64-bit payload including signed zero, infinities, and NaNs is admitted",
                new[] { InputDescription(input) },
                ParseHexBits(input, 16),
                Observe(() => BclHexParse64(input), "ieee_binary64_bits"),
                false);
        }
        foreach (string bits in new[]
        {
            "0000000000000000",
            "8000000000000000",
            "7ff0000000000000",
            "7ff8123456789abc",
            "7ff0123456789abc",
        })
        {
            ulong raw = ulong.Parse(bits, NumberStyles.AllowHexSpecifier, CultureInfo.InvariantCulture);
            double value = BitConverter.Int64BitsToDouble(unchecked((long)raw));
            AddCodec(
                "codec.binary64.format." + bits,
                "codec.floating_bits",
                "codec.binary64.format",
                "every IEEE binary64 bit pattern; output exactly 16 lowercase hexadecimal digits without prefix",
                new[] { "bits=" + bits },
                new CodecOutcome("value", "ascii", bits, null),
                Observe(() => Bits(value), "ieee_binary64_bits"),
                false);
        }
    }

    private static CodecOutcome ParseHexBits(string input, int digits)
    {
        if (input.Length > TextBound)
        {
            return CodecError("parse_error.input_bound");
        }
        if (input.Length != digits)
        {
            return CodecError("parse_error.syntax");
        }
        bool uppercase = false;
        foreach (char character in input)
        {
            if (character is >= 'A' and <= 'F')
            {
                uppercase = true;
            }
            else if (!(character is >= '0' and <= '9') && !(character is >= 'a' and <= 'f'))
            {
                return CodecError("parse_error.syntax");
            }
        }
        if (uppercase)
        {
            return CodecError("parse_error.noncanonical");
        }
        return new CodecOutcome(
            "value",
            digits == 8 ? "ieee_binary32_bits" : "ieee_binary64_bits",
            input,
            null);
    }

    private static string BclHexParse32(string input)
    {
        return uint.TryParse(input, NumberStyles.AllowHexSpecifier, CultureInfo.CurrentCulture, out uint raw)
            ? Bits(BitConverter.Int32BitsToSingle(unchecked((int)raw)))
            : "parse_false";
    }

    private static string BclHexParse64(string input)
    {
        return ulong.TryParse(input, NumberStyles.AllowHexSpecifier, CultureInfo.CurrentCulture, out ulong raw)
            ? Bits(BitConverter.Int64BitsToDouble(unchecked((long)raw)))
            : "parse_false";
    }

    private static void AddGuidCodecVectors()
    {
        const string n = "00112233445566778899aabbccddeeff";
        const string d = "00112233-4455-6677-8899-aabbccddeeff";
        foreach ((string format, string canonical) in new[] { ("n", n), ("d", d) })
        {
            foreach ((string id, string input) in new[]
            {
                ("zero", format == "n" ? new string('0', 32) : "00000000-0000-0000-0000-000000000000"),
                ("sample", canonical),
                ("uppercase", canonical.ToUpperInvariant()),
                ("wrong_shape", format == "n" ? d : n),
                ("braced", "{" + d + "}"),
                ("invalid", format == "n" ? n[..31] + "g" : d[..35] + "g"),
                ("over_bound", new string('9', TextBound + 1)),
            })
            {
                AddCodec(
                    "codec.guid." + format + ".parse." + id,
                    "codec.guid",
                    "codec.guid." + format + ".parse",
                    format == "n"
                        ? "exactly 32 lowercase ASCII hexadecimal digits in GUID N order"
                        : "exactly 36 ASCII units in lowercase GUID D form with hyphens at 8/13/18/23",
                    new[] { InputDescription(input) },
                    ParseGuid(input, format),
                    Observe(() => BclGuidParse(input, format), "guid_n_ascii"),
                    false);
            }
            Guid value = Guid.ParseExact(canonical, format);
            AddCodec(
                "codec.guid." + format + ".format.sample",
                "codec.guid",
                "codec.guid." + format + ".format",
                "every 128-bit GUID value; output only the exact lowercase " + format.ToUpperInvariant() + " spelling",
                new[] { "guid_n=" + n },
                new CodecOutcome("value", "ascii", format == "n" ? n : d, null),
                Observe(() => value.ToString(format, CultureInfo.CurrentCulture), "ascii"),
                false);
        }
    }

    private static CodecOutcome ParseGuid(string input, string format)
    {
        if (input.Length > TextBound)
        {
            return CodecError("parse_error.input_bound");
        }
        int expectedLength = format == "n" ? 32 : 36;
        if (input.Length != expectedLength)
        {
            return CodecError("parse_error.syntax");
        }
        bool uppercase = false;
        for (int index = 0; index < input.Length; index++)
        {
            bool hyphen = format == "d" && index is 8 or 13 or 18 or 23;
            if (hyphen)
            {
                if (input[index] != '-')
                {
                    return CodecError("parse_error.syntax");
                }
                continue;
            }
            char character = input[index];
            if (character is >= 'A' and <= 'F')
            {
                uppercase = true;
            }
            else if (!(character is >= '0' and <= '9') && !(character is >= 'a' and <= 'f'))
            {
                return CodecError("parse_error.syntax");
            }
        }
        if (uppercase)
        {
            return CodecError("parse_error.noncanonical");
        }
        string n = format == "n" ? input : input.Replace("-", string.Empty, StringComparison.Ordinal);
        return new CodecOutcome("value", "guid_n_ascii", n, null);
    }

    private static string BclGuidParse(string input, string format)
    {
        return Guid.TryParseExact(input, format, out Guid value)
            ? value.ToString("N", CultureInfo.InvariantCulture)
            : "parse_false";
    }

    private static void AddCodecRoundTripVectors()
    {
        IntegerSpec[] integerSpecs =
        {
            new("i8", true, sbyte.MinValue, sbyte.MaxValue),
            new("u8", false, byte.MinValue, byte.MaxValue),
            new("i16", true, short.MinValue, short.MaxValue),
            new("u16", false, ushort.MinValue, ushort.MaxValue),
            new("i32", true, int.MinValue, int.MaxValue),
            new("u32", false, uint.MinValue, uint.MaxValue),
            new("i64", true, long.MinValue, long.MaxValue),
            new("u64", false, ulong.MinValue, ulong.MaxValue),
        };
        foreach (IntegerSpec spec in integerSpecs)
        {
            string[] values = spec.Signed
                ? new[]
                {
                    AsciiInteger(spec.Minimum),
                    "0",
                    AsciiInteger(spec.Maximum),
                }
                : new[] { "0", AsciiInteger(spec.Maximum) };
            foreach (string canonical in values)
            {
                string captured = canonical;
                AddRoundTrip(
                    "codec.integer."
                        + spec.Id
                        + ".roundtrip."
                        + SafeId(captured),
                    "codec.integer",
                    "codec.integer." + spec.Id + ".roundtrip",
                    "every " + spec.Id + " value and every successfully parsed canonical spelling",
                    new[] { "canonical=" + captured },
                    () =>
                    {
                        CodecOutcome parsed = ParseInteger(captured, spec);
                        return parsed.Kind == "value" && parsed.Value == captured;
                    },
                    () => BclIntegerExactRoundTrip(spec.Id, captured));
            }
        }

        foreach ((string id, decimal value) in new[]
        {
            ("zero", decimal.Zero),
            ("negative_zero_scale28", new decimal(0, 0, 0, true, 28)),
            ("negative", -12.3400m),
            ("maximum", decimal.MaxValue),
            ("minimum_fraction", new decimal(1, 0, 0, false, 28)),
        })
        {
            decimal captured = value;
            AddRoundTrip(
                "codec.decimal.normalized.roundtrip." + id,
                "codec.decimal",
                "codec.decimal.normalized.roundtrip",
                "every decimal value modulo value-based scale and zero-sign equivalence, and every successfully parsed normalized spelling",
                new[] { "value=" + DecimalEncoding(captured) },
                () => DecimalNormalizedRoundTrip(captured),
                () => BclDecimalNormalizedRoundTrip(captured));
        }

        foreach (MidpointRounding mode in new[]
        {
            MidpointRounding.ToEven,
            MidpointRounding.AwayFromZero,
            MidpointRounding.ToZero,
            MidpointRounding.ToNegativeInfinity,
            MidpointRounding.ToPositiveInfinity,
        })
        {
            MidpointRounding capturedMode = mode;
            foreach ((string id, decimal value, int scale) in new[]
            {
                ("n1_255", -1.255m, 2),
                ("1_245", 1.245m, 2),
                ("integer_scale2", 1m, 2),
                ("maximum_scale2", decimal.MaxValue, 2),
                ("maximum_scale28", decimal.MaxValue, 28),
                ("minimum_scale28", decimal.MinValue, 28),
                ("negative_zero_scale28", new decimal(0, 0, 0, true, 28), 28),
                ("least_fraction_scale28", new decimal(1, 0, 0, false, 28), 28),
                ("round_to_zero_scale0", new decimal(1, 0, 0, false, 28), 0),
            })
            {
                decimal capturedValue = value;
                int capturedScale = scale;
                AddRoundTrip(
                    "codec.decimal.fixed.roundtrip."
                        + id
                        + "."
                        + mode.ToString().ToLowerInvariant(),
                    "codec.decimal",
                    "codec.decimal.fixed.roundtrip",
                    "every decimal value, scale 0..28, and allowlisted rounding mode; parsing formatted text returns the explicitly rounded value and reformatting is byte-identical",
                    new[]
                    {
                        "value=" + DecimalEncoding(capturedValue),
                        "scale=" + AsciiInteger(capturedScale),
                        "rounding=" + capturedMode,
                    },
                    () => DecimalFixedRoundTrip(capturedValue, capturedScale, capturedMode),
                    () => BclDecimalFixedRoundTrip(capturedValue, capturedScale, capturedMode));
            }
        }

        foreach (DateOnly value in new[]
        {
            DateOnly.MinValue,
            new DateOnly(2024, 2, 29),
            DateOnly.MaxValue,
        })
        {
            DateOnly captured = value;
            AddRoundTrip(
                "codec.date.roundtrip." + captured.DayNumber.ToString("d7", CultureInfo.InvariantCulture),
                "codec.date_time",
                "codec.date.roundtrip",
                "every DateOnly value and every successfully parsed yyyy-MM-dd spelling",
                new[] { "day_number=" + captured.DayNumber.ToString(CultureInfo.InvariantCulture) },
                () => DateRoundTrip(captured),
                () => DateOnly.TryParseExact(
                    captured.ToString("yyyy-MM-dd", CultureInfo.InvariantCulture),
                    "yyyy-MM-dd",
                    CultureInfo.InvariantCulture,
                    DateTimeStyles.None,
                    out DateOnly parsed) && parsed == captured);
        }

        foreach (TimeOnly value in new[]
        {
            TimeOnly.MinValue,
            new TimeOnly(12, 34, 56).Add(TimeSpan.FromTicks(1_234_567)),
            TimeOnly.MaxValue,
        })
        {
            TimeOnly captured = value;
            AddRoundTrip(
                "codec.time.roundtrip." + captured.Ticks.ToString("d12", CultureInfo.InvariantCulture),
                "codec.date_time",
                "codec.time.roundtrip",
                "every TimeOnly value and every successfully parsed HH:mm:ss.fffffff spelling",
                new[] { "ticks=" + captured.Ticks.ToString(CultureInfo.InvariantCulture) },
                () => TimeRoundTrip(captured),
                () => TimeOnly.TryParseExact(
                    captured.ToString("HH:mm:ss.fffffff", CultureInfo.InvariantCulture),
                    "HH:mm:ss.fffffff",
                    CultureInfo.InvariantCulture,
                    DateTimeStyles.None,
                    out TimeOnly parsed) && parsed == captured);
        }

        IntegerSpec signed64 = new("i64", true, long.MinValue, long.MaxValue);
        foreach (string carrier in new[]
        {
            AsciiInteger(long.MinValue),
            "-1",
            "0",
            AsciiInteger(long.MaxValue),
        })
        {
            string captured = carrier;
            foreach (string codec in new[] { "duration_ticks", "unix_milliseconds" })
            {
                AddRoundTrip(
                    "codec." + codec + ".roundtrip." + SafeId(captured),
                    "codec.duration_instant",
                    "codec." + codec + ".roundtrip",
                    "every signed int64 carrier value and every successfully parsed canonical spelling",
                    new[] { "canonical=" + captured },
                    () =>
                    {
                        CodecOutcome parsed = ParseInteger(captured, signed64);
                        return parsed.Kind == "value" && parsed.Value == captured;
                    },
                    () => long.TryParse(
                        captured,
                        NumberStyles.AllowLeadingSign,
                        CultureInfo.InvariantCulture,
                        out long value) && value.ToString(CultureInfo.InvariantCulture) == captured);
            }
        }

        foreach (string bits in new[] { "00000000", "80000000", "7f800000", "7fc12345", "7fa12345" })
        {
            string captured = bits;
            AddRoundTrip(
                "codec.binary32.roundtrip." + captured,
                "codec.floating_bits",
                "codec.binary32.roundtrip",
                "every IEEE binary32 bit pattern, including both zero signs, infinities, and every NaN payload",
                new[] { "bits=" + captured },
                () => ParseHexBits(captured, 8).Value == captured,
                () => BclHexParse32(captured) == captured);
        }
        foreach (string bits in new[]
        {
            "0000000000000000",
            "8000000000000000",
            "7ff0000000000000",
            "7ff8123456789abc",
            "7ff0123456789abc",
        })
        {
            string captured = bits;
            AddRoundTrip(
                "codec.binary64.roundtrip." + captured,
                "codec.floating_bits",
                "codec.binary64.roundtrip",
                "every IEEE binary64 bit pattern, including both zero signs, infinities, and every NaN payload",
                new[] { "bits=" + captured },
                () => ParseHexBits(captured, 16).Value == captured,
                () => BclHexParse64(captured) == captured);
        }

        const string guidN = "00112233445566778899aabbccddeeff";
        const string guidD = "00112233-4455-6677-8899-aabbccddeeff";
        foreach ((string format, string canonical) in new[] { ("n", guidN), ("d", guidD) })
        {
            string capturedFormat = format;
            string captured = canonical;
            AddRoundTrip(
                "codec.guid." + format + ".roundtrip.sample",
                "codec.guid",
                "codec.guid." + format + ".roundtrip",
                "every GUID value and every successfully parsed exact lowercase "
                    + format.ToUpperInvariant()
                    + " spelling",
                new[] { "canonical=" + captured },
                () =>
                {
                    CodecOutcome parsed = ParseGuid(captured, capturedFormat);
                    string reformatted = FormatGuid(parsed.Value, capturedFormat);
                    return parsed.Kind == "value"
                        && parsed.Value == guidN
                        && reformatted == captured;
                },
                () => Guid.TryParseExact(captured, capturedFormat, out Guid value)
                    && value.ToString(capturedFormat, CultureInfo.InvariantCulture) == captured);
        }
    }

    private static void AddRoundTrip(
        string id,
        string family,
        string operation,
        string domain,
        string[] inputs,
        Func<bool> candidate,
        Func<bool> differential)
    {
        bool candidateResult = candidate();
        bool differentialResult = differential();
        if (!candidateResult || !differentialResult)
        {
            throw new ProbeFailure("ROUNDTRIP");
        }
        AddVector(
            id,
            family,
            operation,
            domain,
            inputs,
            "candidate_admitted",
            new CodecOutcome("value", "bool", "true", null),
            new Observation("value", "bool", "true", null),
            false,
            Array.Empty<string>());
    }

    private static bool BclIntegerExactRoundTrip(string id, string canonical)
    {
        CultureInfo invariant = CultureInfo.InvariantCulture;
        return id switch
        {
            "i8" => sbyte.TryParse(canonical, NumberStyles.AllowLeadingSign, invariant, out sbyte value)
                && value.ToString(invariant) == canonical,
            "u8" => byte.TryParse(canonical, NumberStyles.None, invariant, out byte value)
                && value.ToString(invariant) == canonical,
            "i16" => short.TryParse(canonical, NumberStyles.AllowLeadingSign, invariant, out short value)
                && value.ToString(invariant) == canonical,
            "u16" => ushort.TryParse(canonical, NumberStyles.None, invariant, out ushort value)
                && value.ToString(invariant) == canonical,
            "i32" => int.TryParse(canonical, NumberStyles.AllowLeadingSign, invariant, out int value)
                && value.ToString(invariant) == canonical,
            "u32" => uint.TryParse(canonical, NumberStyles.None, invariant, out uint value)
                && value.ToString(invariant) == canonical,
            "i64" => long.TryParse(canonical, NumberStyles.AllowLeadingSign, invariant, out long value)
                && value.ToString(invariant) == canonical,
            "u64" => ulong.TryParse(canonical, NumberStyles.None, invariant, out ulong value)
                && value.ToString(invariant) == canonical,
            _ => throw new ProbeFailure("INTEGER_TYPE"),
        };
    }

    private static string FormatGuid(string n, string format)
    {
        if (n.Length != 32)
        {
            throw new ProbeFailure("GUID_BITS");
        }
        return format == "n"
            ? n
            : n[..8]
                + "-"
                + n[8..12]
                + "-"
                + n[12..16]
                + "-"
                + n[16..20]
                + "-"
                + n[20..];
    }

    private static bool DecimalNormalizedRoundTrip(decimal value)
    {
        string formatted = FormatDecimalNormalized(value);
        CodecOutcome parsed = ParseDecimal(formatted, null, out decimal reparsed);
        return parsed.Kind == "value"
            && reparsed == value
            && FormatDecimalNormalized(reparsed) == formatted;
    }

    private static bool DecimalFixedRoundTrip(
        decimal value,
        int scale,
        MidpointRounding mode)
    {
        decimal rounded = decimal.Round(value, scale, mode);
        string formatted = FormatDecimalFixed(rounded, scale);
        CodecOutcome parsed = ParseDecimal(formatted, scale, out decimal reparsed);
        return parsed.Kind == "value"
            && reparsed == rounded
            && FormatDecimalFixed(reparsed, scale) == formatted;
    }

    private static bool BclDecimalNormalizedRoundTrip(decimal value)
    {
        string formatted = FormatDecimalNormalized(value);
        return decimal.TryParse(
                formatted,
                NumberStyles.AllowLeadingSign | NumberStyles.AllowDecimalPoint,
                CultureInfo.InvariantCulture,
                out decimal reparsed)
            && reparsed == value;
    }

    private static bool BclDecimalFixedRoundTrip(
        decimal value,
        int scale,
        MidpointRounding mode)
    {
        decimal rounded = decimal.Round(value, scale, mode);
        string formatted = FormatDecimalFixed(rounded, scale);
        return decimal.TryParse(
                formatted,
                NumberStyles.AllowLeadingSign | NumberStyles.AllowDecimalPoint,
                CultureInfo.InvariantCulture,
                out decimal reparsed)
            && reparsed == rounded
            && reparsed.ToString("F" + AsciiInteger(scale), CultureInfo.InvariantCulture) == formatted;
    }

    private static bool DateRoundTrip(DateOnly value)
    {
        string formatted = Four(value.Year) + "-" + Two(value.Month) + "-" + Two(value.Day);
        CodecOutcome parsed = ParseDate(formatted);
        return parsed.Kind == "value"
            && parsed.Value == AsciiInteger(value.DayNumber);
    }

    private static bool TimeRoundTrip(TimeOnly value)
    {
        string formatted = FormatTime(value.Ticks);
        CodecOutcome parsed = ParseTime(formatted);
        return parsed.Kind == "value"
            && parsed.Value == AsciiInteger(value.Ticks);
    }

    private static void AddFloatingVectors()
    {
        uint[] floatBits =
        {
            0xff800000,
            0xbf800000,
            0x80000000,
            0x00000000,
            0x00000001,
            0x3f800000,
            0x7f800000,
            0x7fc12345,
            0x7fa12345,
        };
        ulong[] doubleBits =
        {
            0xfff0000000000000,
            0xbff0000000000000,
            0x8000000000000000,
            0x0000000000000000,
            0x0000000000000001,
            0x3ff0000000000000,
            0x7ff0000000000000,
            0x7ff8123456789abc,
            0x7ff0123456789abc,
        };

        string floatDomain =
            "all IEEE binary32 values represented by exact 32-bit payloads; round-to-nearest ties-to-even, no fast-math, FMA, or extended precision";
        string doubleDomain =
            "all IEEE binary64 values represented by exact 64-bit payloads; round-to-nearest ties-to-even, no fast-math, FMA, or extended precision";
        for (int leftIndex = 0; leftIndex < floatBits.Length; leftIndex++)
        {
            float left = BitConverter.Int32BitsToSingle(unchecked((int)floatBits[leftIndex]));
            string leftBits = Bits(left);
            AddFloatingUnary("single", leftIndex, leftBits, floatDomain, "plus", () => +left);
            AddFloatingUnary("single", leftIndex, leftBits, floatDomain, "negate", () => -left);
            AddFloatingUnary("single", leftIndex, leftBits, floatDomain, "abs", () => MathF.Abs(left));
            AddFloatingPredicate("single", leftIndex, leftBits, floatDomain, "is_nan", () => float.IsNaN(left));
            AddFloatingPredicate("single", leftIndex, leftBits, floatDomain, "is_infinity", () => float.IsInfinity(left));
            AddFloatingPredicate("single", leftIndex, leftBits, floatDomain, "is_finite", () => float.IsFinite(left));
            for (int rightIndex = 0; rightIndex < floatBits.Length; rightIndex++)
            {
                float right = BitConverter.Int32BitsToSingle(unchecked((int)floatBits[rightIndex]));
                string rightBits = Bits(right);
                AddFloatingBinary("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "add", () => left + right);
                AddFloatingBinary("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "subtract", () => left - right);
                AddFloatingBinary("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "multiply", () => left * right);
                AddFloatingBinary("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "divide", () => left / right);
                AddFloatingBinary("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "remainder", () => left % right);
                AddFloatingBinary("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "min", () => MathF.Min(left, right));
                AddFloatingBinary("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "max", () => MathF.Max(left, right));
                AddFloatingComparison("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "equal", () => left == right);
                AddFloatingComparison("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "not_equal", () => left != right);
                AddFloatingComparison("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "less", () => left < right);
                AddFloatingComparison("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "less_equal", () => left <= right);
                AddFloatingComparison("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "greater", () => left > right);
                AddFloatingComparison("single", leftIndex, rightIndex, leftBits, rightBits, floatDomain, "greater_equal", () => left >= right);
            }
        }

        for (int leftIndex = 0; leftIndex < doubleBits.Length; leftIndex++)
        {
            double left = BitConverter.Int64BitsToDouble(unchecked((long)doubleBits[leftIndex]));
            string leftBits = Bits(left);
            AddFloatingUnary("double", leftIndex, leftBits, doubleDomain, "plus", () => +left);
            AddFloatingUnary("double", leftIndex, leftBits, doubleDomain, "negate", () => -left);
            AddFloatingUnary("double", leftIndex, leftBits, doubleDomain, "abs", () => Math.Abs(left));
            AddFloatingPredicate("double", leftIndex, leftBits, doubleDomain, "is_nan", () => double.IsNaN(left));
            AddFloatingPredicate("double", leftIndex, leftBits, doubleDomain, "is_infinity", () => double.IsInfinity(left));
            AddFloatingPredicate("double", leftIndex, leftBits, doubleDomain, "is_finite", () => double.IsFinite(left));
            for (int rightIndex = 0; rightIndex < doubleBits.Length; rightIndex++)
            {
                double right = BitConverter.Int64BitsToDouble(unchecked((long)doubleBits[rightIndex]));
                string rightBits = Bits(right);
                AddFloatingBinary("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "add", () => left + right);
                AddFloatingBinary("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "subtract", () => left - right);
                AddFloatingBinary("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "multiply", () => left * right);
                AddFloatingBinary("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "divide", () => left / right);
                AddFloatingBinary("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "remainder", () => left % right);
                AddFloatingBinary("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "min", () => Math.Min(left, right));
                AddFloatingBinary("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "max", () => Math.Max(left, right));
                AddFloatingComparison("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "equal", () => left == right);
                AddFloatingComparison("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "not_equal", () => left != right);
                AddFloatingComparison("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "less", () => left < right);
                AddFloatingComparison("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "less_equal", () => left <= right);
                AddFloatingComparison("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "greater", () => left > right);
                AddFloatingComparison("double", leftIndex, rightIndex, leftBits, rightBits, doubleDomain, "greater_equal", () => left >= right);
            }
        }

        AddMeasuredValue(
            "numeric.conversion.int32_to_single.max",
            "floating.conversion",
            "numeric.conversion.int32_to_single",
            "every int32 input; explicit conversion to IEEE binary32 under round-to-nearest ties-to-even",
            new[] { "value=2147483647" },
            "ieee_binary32_bits",
            () => Bits((float)int.MaxValue));
        AddMeasuredValue(
            "numeric.conversion.int64_to_double.max",
            "floating.conversion",
            "numeric.conversion.int64_to_double",
            "every int64 input; explicit conversion to IEEE binary64 under round-to-nearest ties-to-even",
            new[] { "value=9223372036854775807" },
            "ieee_binary64_bits",
            () => Bits((double)long.MaxValue));
        AddMeasuredValue(
            "numeric.conversion.single_to_double.nan",
            "floating.conversion",
            "numeric.conversion.single_to_double",
            "every IEEE binary32 input; explicit widening conversion to binary64 with pinned NaN payload behavior",
            new[] { "bits=7fc12345" },
            "ieee_binary64_bits",
            () => Bits((double)BitConverter.Int32BitsToSingle(unchecked((int)0x7fc12345))));
        AddMeasuredValue(
            "numeric.conversion.double_to_single.nan",
            "floating.conversion",
            "numeric.conversion.double_to_single",
            "every IEEE binary64 input; explicit narrowing conversion to binary32 with pinned rounding and NaN payload behavior",
            new[] { "bits=7ff8123456789abc" },
            "ieee_binary32_bits",
            () => Bits((float)BitConverter.Int64BitsToDouble(unchecked((long)0x7ff8123456789abc))));
        AddMeasuredError(
            "numeric.conversion.single_to_int32.nan",
            "floating.conversion",
            "numeric.conversion.single_to_int32.checked",
            "IEEE binary32 values whose truncation toward zero is in int32 range; checked conversion otherwise overflows",
            new[] { "bits=7fc00000" },
            "exception.overflow",
            () =>
            {
                float value = float.NaN;
                return checked((int)value).ToString(CultureInfo.InvariantCulture);
            });
        AddMeasuredValue(
            "numeric.conversion.single_to_int32.fraction",
            "floating.conversion",
            "numeric.conversion.single_to_int32.checked",
            "IEEE binary32 values whose truncation toward zero is in int32 range; checked conversion otherwise overflows",
            new[] { "bits=3ff33333" },
            "signed_decimal",
            () => checked((int)1.9f).ToString(CultureInfo.InvariantCulture));
        AddMeasuredError(
            "numeric.conversion.double_to_int64.infinity",
            "floating.conversion",
            "numeric.conversion.double_to_int64.checked",
            "IEEE binary64 values whose truncation toward zero is in int64 range; checked conversion otherwise overflows",
            new[] { "bits=7ff0000000000000" },
            "exception.overflow",
            () =>
            {
                double value = double.PositiveInfinity;
                return checked((long)value).ToString(CultureInfo.InvariantCulture);
            });
        AddMeasuredValue(
            "numeric.conversion.double_to_int64.fraction",
            "floating.conversion",
            "numeric.conversion.double_to_int64.checked",
            "IEEE binary64 values whose truncation toward zero is in int64 range; checked conversion otherwise overflows",
            new[] { "bits=bffe666666666666" },
            "signed_decimal",
            () => checked((long)-1.9d).ToString(CultureInfo.InvariantCulture));

        AddRejected(
            "floating.general_format.single",
            "floating.culture_rejection",
            "floating.rejected_general_format",
            "general floating formatting and ToString are outside the profile; only the exact bit codec is admitted",
            new[] { "type=single", "bits=c49a5000" },
            "source_rejection.general_format",
            "utf16_hex",
            () => Utf16((-1234.5f).ToString()),
            true);
        AddRejected(
            "floating.general_parse.double",
            "floating.culture_rejection",
            "floating.rejected_general_parse",
            "general floating parsing is outside the profile; only the exact bit codec is admitted",
            new[] { "type=double", "text_utf16=002d0031003200330034002e0035" },
            "source_rejection.general_parse",
            "ieee_binary64_bits",
            () => double.TryParse("-1234.5", out double value) ? Bits(value) : "parse_false",
            true);
    }

    private static void AddFloatingUnary(
        string type,
        int index,
        string bits,
        string domain,
        string operation,
        Func<float> action)
    {
        AddMeasuredValue(
            "floating." + type + "." + operation + ".v" + index.ToString("d2", CultureInfo.InvariantCulture),
            "floating." + type,
            "floating." + type + "." + operation,
            domain,
            new[] { "operand_bits=" + bits },
            "ieee_binary32_bits",
            () => Bits(action()));
    }

    private static void AddFloatingUnary(
        string type,
        int index,
        string bits,
        string domain,
        string operation,
        Func<double> action)
    {
        AddMeasuredValue(
            "floating." + type + "." + operation + ".v" + index.ToString("d2", CultureInfo.InvariantCulture),
            "floating." + type,
            "floating." + type + "." + operation,
            domain,
            new[] { "operand_bits=" + bits },
            "ieee_binary64_bits",
            () => Bits(action()));
    }

    private static void AddFloatingPredicate(
        string type,
        int index,
        string bits,
        string domain,
        string operation,
        Func<bool> action)
    {
        AddMeasuredValue(
            "floating." + type + "." + operation + ".v" + index.ToString("d2", CultureInfo.InvariantCulture),
            "floating." + type,
            "floating." + type + "." + operation,
            domain,
            new[] { "operand_bits=" + bits },
            "bool",
            () => Bool(action()));
    }

    private static void AddFloatingBinary(
        string type,
        int leftIndex,
        int rightIndex,
        string leftBits,
        string rightBits,
        string domain,
        string operation,
        Func<float> action)
    {
        AddMeasuredValue(
            PairId(type, operation, leftIndex, rightIndex),
            "floating." + type,
            "floating." + type + "." + operation,
            domain,
            new[] { "left_bits=" + leftBits, "right_bits=" + rightBits },
            "ieee_binary32_bits",
            () => Bits(action()));
    }

    private static void AddFloatingBinary(
        string type,
        int leftIndex,
        int rightIndex,
        string leftBits,
        string rightBits,
        string domain,
        string operation,
        Func<double> action)
    {
        AddMeasuredValue(
            PairId(type, operation, leftIndex, rightIndex),
            "floating." + type,
            "floating." + type + "." + operation,
            domain,
            new[] { "left_bits=" + leftBits, "right_bits=" + rightBits },
            "ieee_binary64_bits",
            () => Bits(action()));
    }

    private static void AddFloatingComparison(
        string type,
        int leftIndex,
        int rightIndex,
        string leftBits,
        string rightBits,
        string domain,
        string operation,
        Func<bool> action)
    {
        AddMeasuredValue(
            PairId(type, operation, leftIndex, rightIndex),
            "floating." + type,
            "floating." + type + "." + operation,
            domain,
            new[] { "left_bits=" + leftBits, "right_bits=" + rightBits },
            "bool",
            () => Bool(action()));
    }

    private static string PairId(string type, string operation, int left, int right)
    {
        return "floating."
            + type
            + "."
            + operation
            + ".v"
            + left.ToString("d2", CultureInfo.InvariantCulture)
            + ".v"
            + right.ToString("d2", CultureInfo.InvariantCulture);
    }

    private static void AddDecimalVectors()
    {
        decimal[] values =
        {
            -1.5m,
            -1m,
            new decimal(1, 0, 0, true, 28),
            new decimal(0, 0, 0, true, 0),
            decimal.Zero,
            new decimal(1, 0, 0, false, 28),
            1m,
            1.5m,
        };
        const string domain =
            ".NET decimal sign plus 96-bit coefficient and scale 0..28; exact checked runtime arithmetic and value-based comparison";
        for (int leftIndex = 0; leftIndex < values.Length; leftIndex++)
        {
            decimal left = values[leftIndex];
            AddDecimalUnary(leftIndex, left, domain, "plus", () => +left);
            AddDecimalUnary(leftIndex, left, domain, "negate", () => -left);
            for (int rightIndex = 0; rightIndex < values.Length; rightIndex++)
            {
                decimal right = values[rightIndex];
                AddDecimalBinary(leftIndex, rightIndex, left, right, domain, "add", () => left + right);
                AddDecimalBinary(leftIndex, rightIndex, left, right, domain, "subtract", () => left - right);
                AddDecimalBinary(leftIndex, rightIndex, left, right, domain, "multiply", () => left * right);
                AddDecimalBinary(leftIndex, rightIndex, left, right, domain, "divide", () => left / right);
                AddDecimalBinary(leftIndex, rightIndex, left, right, domain, "remainder", () => left % right);
                AddDecimalComparison(leftIndex, rightIndex, left, right, domain, "equal", () => left == right);
                AddDecimalComparison(leftIndex, rightIndex, left, right, domain, "not_equal", () => left != right);
                AddDecimalComparison(leftIndex, rightIndex, left, right, domain, "less", () => left < right);
                AddDecimalComparison(leftIndex, rightIndex, left, right, domain, "less_equal", () => left <= right);
                AddDecimalComparison(leftIndex, rightIndex, left, right, domain, "greater", () => left > right);
                AddDecimalComparison(leftIndex, rightIndex, left, right, domain, "greater_equal", () => left >= right);
            }
        }

        foreach ((string id, string operation, Func<decimal> action) in new (string, string, Func<decimal>)[]
        {
            ("max_plus_one", "decimal.add", () => DecimalAdd(decimal.MaxValue, decimal.One)),
            ("min_minus_one", "decimal.subtract", () => DecimalSubtract(decimal.MinValue, decimal.One)),
            ("max_times_two", "decimal.multiply", () => DecimalMultiply(decimal.MaxValue, 2m)),
            ("negate_min", "decimal.negate", () => DecimalNegate(decimal.MinValue)),
            ("max_divide_zero", "decimal.divide", () => DecimalDivide(decimal.MaxValue, decimal.Zero)),
            ("max_divide_fraction", "decimal.divide", () => DecimalDivide(decimal.MaxValue, 0.1m)),
            ("max_remainder_zero", "decimal.remainder", () => DecimalRemainder(decimal.MaxValue, decimal.Zero)),
        })
        {
            AddMeasuredNumeric(
                "decimal.edge." + id,
                "decimal.edge",
                operation,
                domain,
                new[] { "case=" + id },
                "decimal_bits",
                () => DecimalEncoding(action()));
        }

        MidpointRounding[] modes =
        {
            MidpointRounding.ToEven,
            MidpointRounding.AwayFromZero,
            MidpointRounding.ToZero,
            MidpointRounding.ToNegativeInfinity,
            MidpointRounding.ToPositiveInfinity,
        };
        foreach (decimal value in new[] { -2.5m, -1.5m, 1.5m, 2.5m })
        {
            foreach (MidpointRounding mode in modes)
            {
                decimal captured = value;
                MidpointRounding capturedMode = mode;
                AddMeasuredNumeric(
                    "decimal.round."
                        + SafeId(FormatDecimalNormalized(value))
                        + "."
                        + mode.ToString().ToLowerInvariant(),
                    "decimal.rounding",
                    "decimal.round",
                    "decimal value, digits 0..28, and exact allowlisted midpoint mode in the Round argument position",
                    new[] { "value=" + DecimalEncoding(value), "digits=0", "rounding=" + mode },
                    "decimal_bits",
                    () => DecimalEncoding(decimal.Round(captured, 0, capturedMode)));
            }
        }
        AddMeasuredError(
            "decimal.round.digits_negative",
            "decimal.rounding",
            "decimal.round",
            "decimal value, digits 0..28, and exact allowlisted midpoint mode in the Round argument position",
            new[] { "value=" + DecimalEncoding(1.25m), "digits=-1", "rounding=ToEven" },
            "exception.range",
            () => DecimalRound(1.25m, -1, MidpointRounding.ToEven).ToString(CultureInfo.InvariantCulture));
        AddMeasuredError(
            "decimal.round.digits_29",
            "decimal.rounding",
            "decimal.round",
            "decimal value, digits 0..28, and exact allowlisted midpoint mode in the Round argument position",
            new[] { "value=" + DecimalEncoding(1.25m), "digits=29", "rounding=ToEven" },
            "exception.range",
            () => DecimalRound(1.25m, 29, MidpointRounding.ToEven).ToString(CultureInfo.InvariantCulture));
        foreach ((string operation, Func<decimal, decimal> action) in new (string, Func<decimal, decimal>)[]
        {
            ("truncate", decimal.Truncate),
            ("floor", decimal.Floor),
            ("ceiling", decimal.Ceiling),
        })
        {
            foreach (decimal value in new[] { -1.9m, -0.1m, 0.1m, 1.9m })
            {
                decimal captured = value;
                AddMeasuredNumeric(
                    "decimal." + operation + "." + SafeId(FormatDecimalNormalized(value)),
                    "decimal.rounding",
                    "decimal." + operation,
                    "every decimal value; exact System.Decimal " + operation + " value semantics",
                    new[] { "value=" + DecimalEncoding(value) },
                    "decimal_bits",
                    () => DecimalEncoding(action(captured)));
            }
        }

        AddMeasuredValue(
            "decimal.conversion.int64.minimum",
            "decimal.conversion",
            "decimal.conversion.int64_to_decimal",
            "every int64 value; exact integral decimal result",
            new[] { "value=-9223372036854775808" },
            "decimal_bits",
            () => DecimalEncoding((decimal)long.MinValue));
        AddMeasuredValue(
            "decimal.conversion.uint64.maximum",
            "decimal.conversion",
            "decimal.conversion.uint64_to_decimal",
            "every uint64 value; exact integral decimal result",
            new[] { "value=18446744073709551615" },
            "decimal_bits",
            () => DecimalEncoding((decimal)ulong.MaxValue));
        AddMeasuredValue(
            "decimal.conversion.to_int32.fraction",
            "decimal.conversion",
            "decimal.conversion.decimal_to_int32",
            "decimal values whose truncation toward zero lies in int32 range; overflow otherwise",
            new[] { "value=" + DecimalEncoding(-1.9m) },
            "signed_decimal",
            () => ((int)-1.9m).ToString(CultureInfo.InvariantCulture));
        AddMeasuredError(
            "decimal.conversion.to_int32.overflow",
            "decimal.conversion",
            "decimal.conversion.decimal_to_int32",
            "decimal values whose truncation toward zero lies in int32 range; overflow otherwise",
            new[] { "value=" + DecimalEncoding(decimal.MaxValue) },
            "exception.overflow",
            () =>
            {
                decimal value = decimal.MaxValue;
                return ((int)value).ToString(CultureInfo.InvariantCulture);
            });

        AddMeasuredValue(
            "decimal.equivalence.trailing_scale",
            "decimal.representation",
            "decimal.value_equality",
            "all decimal representations; equality compares numeric value rather than coefficient scale or zero sign",
            new[] { "left=" + DecimalEncoding(1m), "right=" + DecimalEncoding(1.00m) },
            "bool",
            () => Bool(1m == 1.00m));
        decimal negativeZero = new(0, 0, 0, true, 28);
        AddMeasuredValue(
            "decimal.equivalence.negative_zero",
            "decimal.representation",
            "decimal.value_equality",
            "all decimal representations; equality compares numeric value rather than coefficient scale or zero sign",
            new[] { "left=" + DecimalEncoding(decimal.Zero), "right=" + DecimalEncoding(negativeZero) },
            "bool",
            () => Bool(decimal.Zero == negativeZero));
        AddRejected(
            "decimal.general_format",
            "decimal.culture_rejection",
            "decimal.rejected_general_format",
            "general decimal formatting and ToString are outside the profile; only exact boundary codecs are admitted",
            new[] { "value=" + DecimalEncoding(-1234.5m) },
            "source_rejection.general_format",
            "utf16_hex",
            () => Utf16((-1234.5m).ToString()),
            true);
        AddRejected(
            "decimal.get_bits.source",
            "decimal.representation",
            "decimal.rejected_representation_inspection",
            "decimal.GetBits and other representation inspection are probe-only and rejected in selected application source",
            new[] { "value=" + DecimalEncoding(1.00m) },
            "source_rejection.decimal_representation_inspection",
            "decimal_bits",
            () => DecimalEncoding(1.00m),
            false);
    }

    private static void AddDecimalUnary(
        int index,
        decimal value,
        string domain,
        string operation,
        Func<decimal> action)
    {
        AddMeasuredNumeric(
            "decimal." + operation + ".v" + index.ToString("d2", CultureInfo.InvariantCulture),
            "decimal.arithmetic",
            "decimal." + operation,
            domain,
            new[] { "operand=" + DecimalEncoding(value) },
            "decimal_bits",
            () => DecimalEncoding(action()));
    }

    private static decimal DecimalAdd(decimal left, decimal right) => left + right;

    private static decimal DecimalSubtract(decimal left, decimal right) => left - right;

    private static decimal DecimalMultiply(decimal left, decimal right) => left * right;

    private static decimal DecimalDivide(decimal left, decimal right) => left / right;

    private static decimal DecimalRemainder(decimal left, decimal right) => left % right;

    private static decimal DecimalNegate(decimal value) => -value;

    private static decimal DecimalRound(decimal value, int digits, MidpointRounding mode) =>
        decimal.Round(value, digits, mode);

    private static void AddDecimalBinary(
        int leftIndex,
        int rightIndex,
        decimal left,
        decimal right,
        string domain,
        string operation,
        Func<decimal> action)
    {
        AddMeasuredNumeric(
            "decimal."
                + operation
                + ".v"
                + leftIndex.ToString("d2", CultureInfo.InvariantCulture)
                + ".v"
                + rightIndex.ToString("d2", CultureInfo.InvariantCulture),
            "decimal.arithmetic",
            "decimal." + operation,
            domain,
            new[] { "left=" + DecimalEncoding(left), "right=" + DecimalEncoding(right) },
            "decimal_bits",
            () => DecimalEncoding(action()));
    }

    private static void AddDecimalComparison(
        int leftIndex,
        int rightIndex,
        decimal left,
        decimal right,
        string domain,
        string operation,
        Func<bool> action)
    {
        AddMeasuredValue(
            "decimal."
                + operation
                + ".v"
                + leftIndex.ToString("d2", CultureInfo.InvariantCulture)
                + ".v"
                + rightIndex.ToString("d2", CultureInfo.InvariantCulture),
            "decimal.comparison",
            "decimal." + operation,
            domain,
            new[] { "left=" + DecimalEncoding(left), "right=" + DecimalEncoding(right) },
            "bool",
            () => Bool(action()));
    }

    private static void AddPrecedenceVectors()
    {
        IntegerSpec signed64 = new("i64", true, long.MinValue, long.MaxValue);
        string overBound = new('\u0661', TextBound + 1);
        const string noncanonicalOverflow = "+9223372036854775808";
        const string decimalOverflow = "79228162514264337593543950336";
        AddCodec(
            "precedence.parser.input_bound_before_syntax",
            "error_precedence",
            "precedence.parser.input_bound_before_syntax",
            "over-bound text is rejected before any code-unit grammar inspection",
            new[] { InputDescription(overBound) },
            ParseInteger(overBound, signed64),
            new Observation("not_applicable", "none", string.Empty, null),
            false);
        AddCodec(
            "precedence.parser.noncanonical_before_range",
            "error_precedence",
            "precedence.parser.noncanonical_before_range",
            "canonical spelling checks precede numeric range checks after ASCII syntax succeeds",
            new[] { InputDescription(noncanonicalOverflow) },
            ParseInteger(noncanonicalOverflow, signed64),
            new Observation("not_applicable", "none", string.Empty, null),
            false);
        AddCodec(
            "precedence.decimal.scale_before_range",
            "error_precedence",
            "precedence.decimal.scale_before_range",
            "fixed-scale argument validation precedes coefficient range evaluation",
            new[] { "scale=29", InputDescription(decimalOverflow) },
            ParseDecimal(decimalOverflow, 29),
            new Observation("not_applicable", "none", string.Empty, null),
            false);
        AddRejectedWithPrecedence(
            "precedence.sidecar.codec_before_rounding",
            "error_precedence",
            "precedence.sidecar.codec_before_rounding",
            "closed sidecar validation rejects an unknown codec before inspecting an unknown rounding mode or input text",
            new[] { "codec=unknown", "rounding=unknown", "scale=2", InputDescription(overBound) },
            ValidateDecimalSidecarThenParse("unknown", "unknown", overBound, 2));
        AddRejectedWithPrecedence(
            "precedence.sidecar.rounding_before_parse",
            "error_precedence",
            "precedence.sidecar.rounding_before_parse",
            "after codec identity succeeds, unknown rounding mode rejects before parser input-bound or grammar checks",
            new[] { "codec=decimal_fixed", "rounding=unknown", "scale=2", InputDescription(overBound) },
            ValidateDecimalSidecarThenParse("decimal_fixed", "unknown", overBound, 2));
    }

    private static CodecOutcome ValidateDecimalSidecarThenParse(
        string codec,
        string rounding,
        string input,
        int scale)
    {
        if (codec != "decimal_fixed")
        {
            return new CodecOutcome("rejected", "none", string.Empty, "sidecar.unknown_codec");
        }
        if (rounding is not ("ToEven" or "AwayFromZero" or "ToZero"
            or "ToNegativeInfinity" or "ToPositiveInfinity"))
        {
            return new CodecOutcome("rejected", "none", string.Empty, "sidecar.unknown_rounding_mode");
        }
        return ParseDecimal(input, scale);
    }

    private static void AddRejectedWithPrecedence(
        string id,
        string family,
        string operation,
        string domain,
        string[] inputs,
        CodecOutcome outcome)
    {
        AddVector(
            id,
            family,
            operation,
            domain,
            inputs,
            "candidate_rejected",
            outcome,
            new Observation("not_applicable", "none", string.Empty, null),
            false,
            SidecarPrecedence);
    }

    private static void AddMeasuredValue(
        string id,
        string family,
        string operation,
        string domain,
        string[] inputs,
        string encoding,
        Func<string> action)
    {
        Observation observed = Observe(action, encoding);
        if (observed.Kind != "value")
        {
            throw new ProbeFailure("UNEXPECTED_RUNTIME_ERROR");
        }
        AddVector(
            id,
            family,
            operation,
            domain,
            inputs,
            "candidate_admitted",
            new CodecOutcome("value", encoding, observed.Value, null),
            observed,
            false,
            family.StartsWith("string.", StringComparison.Ordinal) ? TextPrecedence : NumericPrecedence);
    }

    private static void AddMeasuredNumeric(
        string id,
        string family,
        string operation,
        string domain,
        string[] inputs,
        string encoding,
        Func<string> action)
    {
        Observation observed = Observe(action, encoding);
        CodecOutcome profile = observed.Kind == "value"
            ? new CodecOutcome("value", encoding, observed.Value, null)
            : new CodecOutcome("error", "none", string.Empty, ExceptionError(observed.Exception));
        AddVector(
            id,
            family,
            operation,
            domain,
            inputs,
            "candidate_admitted",
            profile,
            observed,
            false,
            NumericPrecedence);
    }

    private static void AddMeasuredError(
        string id,
        string family,
        string operation,
        string domain,
        string[] inputs,
        string errorId,
        Func<string> action)
    {
        Observation observed = Observe(action, "none");
        if (observed.Kind != "exception" || ExceptionError(observed.Exception) != errorId)
        {
            throw new ProbeFailure("EXPECTED_RUNTIME_ERROR");
        }
        AddVector(
            id,
            family,
            operation,
            domain,
            inputs,
            "candidate_admitted",
            new CodecOutcome("error", "none", string.Empty, errorId),
            observed,
            false,
            family.StartsWith("string.", StringComparison.Ordinal) ? TextPrecedence : NumericPrecedence);
    }

    private static void AddRejected(
        string id,
        string family,
        string operation,
        string domain,
        string[] inputs,
        string errorId,
        string runtimeEncoding,
        Func<string> runtimeAction,
        bool cultureSensitive)
    {
        AddVector(
            id,
            family,
            operation,
            domain,
            inputs,
            "candidate_rejected",
            new CodecOutcome("rejected", "none", string.Empty, errorId),
            Observe(runtimeAction, runtimeEncoding),
            cultureSensitive,
            family.StartsWith("string.", StringComparison.Ordinal) ? TextPrecedence : NumericPrecedence);
    }

    private static void AddCodec(
        string id,
        string family,
        string operation,
        string domain,
        string[] inputs,
        CodecOutcome profile,
        Observation differential,
        bool cultureSensitive)
    {
        AddVector(
            id,
            family,
            operation,
            domain,
            inputs,
            "candidate_admitted",
            profile,
            differential,
            cultureSensitive,
            operation.EndsWith(".format", StringComparison.Ordinal)
                ? FormatPrecedence
                : ParsePrecedence);
    }

    private static void AddVector(
        string id,
        string family,
        string operation,
        string domain,
        string[] inputs,
        string profileOutcome,
        CodecOutcome profile,
        Observation differential,
        bool cultureSensitive,
        string[] precedence)
    {
        string[] exactPrecedence = ExactPrecedence(operation, profile, precedence);
        Vectors.Add(Obj(
            ("accepted_domain", domain),
            ("differential", Obj(
                ("exception", differential.Exception),
                ("kind", differential.Kind),
                ("result_encoding", differential.ResultEncoding),
                ("value", differential.Value))),
            ("error_precedence", exactPrecedence),
            ("family", family),
            ("id", id),
            ("inputs", inputs),
            ("operation", operation),
            ("profile", Obj(
                ("error_id", profile.ErrorId),
                ("kind", profile.Kind),
                ("result_encoding", profile.ResultEncoding),
                ("value", profile.Value))),
            ("profile_outcome", profileOutcome),
            ("runtime_culture_sensitive", cultureSensitive)));
    }

    private static string[] ExactPrecedence(
        string operation,
        CodecOutcome profile,
        string[] explicitPrecedence)
    {
        if (operation.StartsWith("precedence.", StringComparison.Ordinal))
        {
            return explicitPrecedence;
        }
        if (profile.Kind == "rejected" && profile.ErrorId is not null)
        {
            return new[] { profile.ErrorId };
        }
        if (operation.StartsWith("codec.", StringComparison.Ordinal))
        {
            if (operation.EndsWith(".roundtrip", StringComparison.Ordinal))
            {
                return Array.Empty<string>();
            }
            if (operation.EndsWith(".format", StringComparison.Ordinal))
            {
                return new[] { "obligation.output_bound" };
            }
            if (operation.Contains("integer.", StringComparison.Ordinal)
                || operation.StartsWith("codec.duration_ticks.", StringComparison.Ordinal)
                || operation.StartsWith("codec.unix_milliseconds.", StringComparison.Ordinal))
            {
                return new[]
                {
                    "parse_error.input_bound",
                    "parse_error.syntax",
                    "parse_error.noncanonical",
                    "parse_error.range",
                };
            }
            if (operation.StartsWith("codec.decimal.", StringComparison.Ordinal))
            {
                return ParsePrecedence;
            }
            if (operation is "codec.date.parse" or "codec.time.parse")
            {
                return new[]
                {
                    "parse_error.input_bound",
                    "parse_error.syntax",
                    "parse_error.range",
                };
            }
            return new[]
            {
                "parse_error.input_bound",
                "parse_error.syntax",
                "parse_error.noncanonical",
            };
        }
        if (operation is "string.length")
        {
            return new[] { "exception.null_receiver" };
        }
        if (operation is "string.index" or "string.substring.start_length")
        {
            return new[] { "exception.null_receiver", "exception.range" };
        }
        if (operation is "string.starts_with.ordinal"
            or "string.ends_with.ordinal"
            or "string.contains.ordinal")
        {
            return new[] { "exception.null_receiver", "exception.null_argument" };
        }
        if (operation.StartsWith("string.concat.", StringComparison.Ordinal)
            || operation == "string.interpolation.restricted")
        {
            return new[] { "obligation.output_bound" };
        }
        if (operation is "numeric.conversion.single_to_int32.checked"
            or "numeric.conversion.double_to_int64.checked"
            or "decimal.conversion.decimal_to_int32")
        {
            return new[] { "exception.overflow" };
        }
        if (operation is "decimal.add" or "decimal.subtract" or "decimal.multiply")
        {
            return new[] { "exception.overflow" };
        }
        if (operation == "decimal.divide")
        {
            return NumericPrecedence;
        }
        if (operation == "decimal.remainder")
        {
            return new[] { "exception.division_by_zero" };
        }
        if (operation == "decimal.round")
        {
            return new[] { "exception.range" };
        }
        return Array.Empty<string>();
    }

    private static Observation Observe(Func<string> action, string encoding)
    {
        try
        {
            return new Observation("value", encoding, action(), null);
        }
        catch (Exception failure)
        {
            return new Observation(
                "exception",
                "none",
                string.Empty,
                failure.GetType().FullName);
        }
    }

    private static CodecOutcome CodecError(string id) =>
        new("error", "none", string.Empty, id);

    private static string ExceptionError(string? exception)
    {
        return exception switch
        {
            "System.NullReferenceException" => "exception.null_receiver",
            "System.ArgumentNullException" => "exception.null_argument",
            "System.ArgumentOutOfRangeException" => "exception.range",
            "System.IndexOutOfRangeException" => "exception.range",
            "System.OverflowException" => "exception.overflow",
            "System.DivideByZeroException" => "exception.division_by_zero",
            _ => throw new ProbeFailure("EXCEPTION_CLASS"),
        };
    }

    private static string DecimalEncoding(decimal value)
    {
        int[] bits = decimal.GetBits(value);
        uint low = unchecked((uint)bits[0]);
        uint middle = unchecked((uint)bits[1]);
        uint high = unchecked((uint)bits[2]);
        bool negative = (bits[3] & unchecked((int)0x80000000)) != 0;
        int scale = (bits[3] >> 16) & 0xff;
        return "sign="
            + (negative ? "1" : "0")
            + ";scale="
            + scale.ToString("d2", CultureInfo.InvariantCulture)
            + ";coefficient="
            + high.ToString("x8", CultureInfo.InvariantCulture)
            + middle.ToString("x8", CultureInfo.InvariantCulture)
            + low.ToString("x8", CultureInfo.InvariantCulture);
    }

    private static string Bits(float value) =>
        unchecked((uint)BitConverter.SingleToInt32Bits(value)).ToString(
            "x8",
            CultureInfo.InvariantCulture);

    private static string Bits(double value) =>
        unchecked((ulong)BitConverter.DoubleToInt64Bits(value)).ToString(
            "x16",
            CultureInfo.InvariantCulture);

    private static string Utf16(string? value)
    {
        if (value is null)
        {
            return "null";
        }
        StringBuilder builder = new(value.Length * 4);
        foreach (char character in value)
        {
            builder.Append(((ushort)character).ToString("x4", CultureInfo.InvariantCulture));
        }
        return builder.ToString();
    }

    private static string U16(char value) =>
        ((ushort)value).ToString("x4", CultureInfo.InvariantCulture);

    private static string Bool(bool value) => value ? "true" : "false";

    private static string Sign(int value) => value switch
    {
        < 0 => "-1",
        > 0 => "1",
        _ => "0",
    };

    private static string InputDescription(string input)
    {
        if (input.Length > 64 && input.All(character => character == input[0]))
        {
            return "text_utf16_repeat="
                + U16(input[0])
                + ";count="
                + AsciiInteger(input.Length);
        }
        return "text_utf16=" + Utf16(input);
    }

    private static string SafeId(string value)
    {
        StringBuilder builder = new();
        foreach (char character in value)
        {
            builder.Append(character switch
            {
                '-' => 'n',
                '.' => '_',
                >= '0' and <= '9' => character,
                _ => 'x',
            });
        }
        return builder.ToString();
    }

    private static bool AsciiDigitsExcept(string value, params int[] except)
    {
        for (int index = 0; index < value.Length; index++)
        {
            if (except.Contains(index))
            {
                continue;
            }
            if (value[index] < '0' || value[index] > '9')
            {
                return false;
            }
        }
        return true;
    }

    private static int Digits(string value, int start, int length)
    {
        int result = 0;
        for (int index = start; index < start + length; index++)
        {
            result = result * 10 + (value[index] - '0');
        }
        return result;
    }

    private static bool IsLeapYear(int year) =>
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);

    private static string AsciiInteger(BigInteger value)
    {
        if (value.IsZero)
        {
            return "0";
        }
        bool negative = value.Sign < 0;
        BigInteger remaining = negative ? -value : value;
        List<char> reversed = new();
        while (!remaining.IsZero)
        {
            int digit = (int)(remaining % 10);
            reversed.Add((char)('0' + digit));
            remaining /= 10;
        }
        char[] result = new char[reversed.Count + (negative ? 1 : 0)];
        int start = negative ? 1 : 0;
        if (negative)
        {
            result[0] = '-';
        }
        for (int index = 0; index < reversed.Count; index++)
        {
            result[start + index] = reversed[reversed.Count - 1 - index];
        }
        return new string(result);
    }

    private static string AsciiUnsignedFixed(long value, int width)
    {
        if (value < 0 || width < 1)
        {
            throw new ProbeFailure("ASCII_FIXED_DOMAIN");
        }
        char[] result = new char[width];
        long remaining = value;
        for (int index = width - 1; index >= 0; index--)
        {
            result[index] = (char)('0' + remaining % 10);
            remaining /= 10;
        }
        if (remaining != 0)
        {
            throw new ProbeFailure("ASCII_FIXED_RANGE");
        }
        return new string(result);
    }

    private static string Two(int value) => AsciiUnsignedFixed(value, 2);

    private static string Four(int value) => AsciiUnsignedFixed(value, 4);

    private static string FormatTime(long ticks)
    {
        long fraction = ticks % 10_000_000L;
        long seconds = ticks / 10_000_000L;
        long second = seconds % 60;
        long minutes = seconds / 60;
        long minute = minutes % 60;
        long hour = minutes / 60;
        return AsciiUnsignedFixed(hour, 2)
            + ":"
            + AsciiUnsignedFixed(minute, 2)
            + ":"
            + AsciiUnsignedFixed(second, 2)
            + "."
            + AsciiUnsignedFixed(fraction, 7);
    }

    private static Dictionary<string, object?> Obj(
        params (string Key, object? Value)[] pairs)
    {
        Dictionary<string, object?> value = new(StringComparer.Ordinal);
        foreach ((string key, object? item) in pairs)
        {
            if (!value.TryAdd(key, item))
            {
                throw new ProbeFailure("DUPLICATE_KEY");
            }
        }
        return value;
    }

    private sealed class ProbeFailure : Exception
    {
        internal ProbeFailure(string code) => Code = code;
        internal string Code { get; }
    }
}
