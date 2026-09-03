// Private runtime observations only. No MPK source API or production assembly.
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Numerics;
using System.Text.Json;

internal static class FoundationDataProbe
{
    private static readonly CultureInfo Invariant = CultureInfo.InvariantCulture;
    private static readonly List<string> Trace = new();
    private static string S(long value) => value.ToString(Invariant);
    private static string B(bool value) => value ? "true" : "false";
    private static long L(string text) => long.Parse(text, Invariant);
    private static int I(string text) => int.Parse(text, Invariant);
    private static int? N(string text) => text == "none" ? null : I(text);
    private static string NText(int? value) => value.HasValue ? S(value.Value) : "none";
    private static string BoolText(bool? value) => value.HasValue ? (value.Value ? "1" : "0") : "none";
    private static string[] DateValue(DateOnly value) => new[] { S(value.Year), S(value.Month), S(value.Day), S(value.DayNumber), S((int)value.DayOfWeek) };
    private static string[] TimeValue(TimeOnly value) => new[] { S(value.Ticks), S(value.Hour), S(value.Minute), S(value.Second), S(value.Millisecond) };
    private static string[] Compare(long a, long b) => new[] { S(Math.Sign(a.CompareTo(b))), B(a == b), B(a != b), B(a < b), B(a <= b), B(a > b), B(a >= b) };

    private readonly struct Data
    {
        internal readonly int Number;
        internal readonly bool Flag;
        internal readonly string? Text;
        internal Data(int number) { Number = number; Flag = true; Text = "x"; }
    }
    private sealed class Built
    {
        internal Built(int value) { Trace.Add("constructor"); if (value != 1) throw new ArgumentException(); }
        internal int Left { get; init; }
        internal required int Right { get; init; }
        internal int Total => Left + Right;
        internal int Sum(int value) { Trace.Add("method"); return Left + Right + value; }
        internal string Text => "present";
    }
    private sealed class Node { internal string Text => "present"; }
    private static int Mark(string name, int value) { Trace.Add(name); return value; }
    private static Built Receiver(Built value) { Trace.Add("receiver"); return value; }
    private static int? NullLeft() { Trace.Add("left"); return null; }
    private static Node? NullReceiver() { Trace.Add("receiver"); return null; }
    private static string[] Source(string operation)
    {
        Trace.Clear();
        switch (operation)
        {
            case "source.construction_order":
            {
                Built value = new Built(Mark("argument", 1)) { Left = Mark("left", 2), Right = Mark("right", 3) };
                int total = Mark("getter", value.Total);
                int called = Receiver(value).Sum(Mark("call_argument", 13));
                return new[] { string.Join(",", Trace), S(total + called) };
            }
            case "source.null_short_circuit":
            {
                int coalesced = NullLeft() ?? Mark("fallback", 7);
                string? text = NullReceiver()?.Text;
                return new[] { string.Join(",", Trace), S(coalesced), text ?? "none" };
            }
            case "source.null_call_order":
            {
                Built? absent = null;
                try { absent!.Sum(Mark("argument", 1)); }
                catch (NullReferenceException error) { return new[] { string.Join(",", Trace), error.GetType().FullName! }; }
                throw new InvalidOperationException();
            }
            case "source.struct_default":
            {
                Data value = default(Data);
                return new[] { S(value.Number), B(value.Flag), value.Text ?? "none" };
            }
            case "source.array_default":
                return new[] { S((new int[1])[0]), (new string?[1])[0] ?? "none", S((new Data[1])[0].Number) };
            case "source.array_negative":
            {
                int length = I("-1");
                return new[] { S(new int[length].Length) };
            }
            case "source.array_index":
            {
                int index = I("1");
                return new[] { S((new int[1])[index]) };
            }
            case "source.array_two_pass":
            {
                int[] input = { 1, 2, 3, 4 };
                int count = 0;
                foreach (int item in input) { Trace.Add(S(item)); if (item % 2 == 0) count++; }
                int[] output = new int[count];
                int next = 0;
                foreach (int item in input) if (item % 2 == 0) { Trace.Add(S(item)); output[next++] = item; }
                return new[] { S(output.Length), S(output[0]), S(output[1]), string.Join(",", Trace) };
            }
            default: throw new InvalidOperationException("unknown source case");
        }
    }
    private static string[] Evaluate(string operation, string[] a)
    {
        if (operation.StartsWith("source.", StringComparison.Ordinal)) return Source(operation);
        if (operation.StartsWith("lifted.", StringComparison.Ordinal)) return Lifted(operation.Split('.'), a);
        if (operation.StartsWith("money.", StringComparison.Ordinal)) return Money(operation.Split('.')[1], a);
        if (operation.StartsWith("instant.", StringComparison.Ordinal))
        {
            long left = L(a[0]), right = L(a[1]);
            string name = operation.Split('.')[1];
            if (name == "compare") return Compare(left, right);
            if (name != "difference" && right % 10000 != 0) return new[] { "error", "precision" };
            BigInteger result = name == "difference" ? ((BigInteger)left - right) * 10000 : (BigInteger)left + (name == "add_duration" ? (BigInteger)right / 10000 : -(BigInteger)right / 10000);
            return result < long.MinValue || result > long.MaxValue ? new[] { "error", "range" } : new[] { "ok", S((long)result) };
        }
        switch (operation)
        {
            case "date.construct": return DateValue(new DateOnly(I(a[0]), I(a[1]), I(a[2])));
            case "date.add_days": return DateValue(DateOnly.FromDayNumber(I(a[0])).AddDays(I(a[1])));
            case "date.add_months": return DateValue(DateOnly.FromDayNumber(I(a[0])).AddMonths(I(a[1])));
            case "date.add_years": return DateValue(DateOnly.FromDayNumber(I(a[0])).AddYears(I(a[1])));
            case "date.compare":
            {
                DateOnly x = DateOnly.FromDayNumber(I(a[0])), y = DateOnly.FromDayNumber(I(a[1]));
                return new[] { S(Math.Sign(x.CompareTo(y))), B(x == y), B(x != y), B(x < y), B(x <= y), B(x > y), B(x >= y) };
            }
            case "time.construct": return TimeValue(new TimeOnly(L(a[0])));
            case "time.add_duration": return TimeValue(new TimeOnly(L(a[0])).Add(new TimeSpan(L(a[1]))));
            case "time.subtract": return new[] { S((new TimeOnly(L(a[0])) - new TimeOnly(L(a[1]))).Ticks) };
            case "time.compare":
            {
                TimeOnly x = new(L(a[0])), y = new(L(a[1]));
                return new[] { S(Math.Sign(x.CompareTo(y))), B(x == y), B(x != y), B(x < y), B(x <= y), B(x > y), B(x >= y) };
            }
            case "duration.construct":
            {
                TimeSpan x = new(L(a[0]));
                return new[] { S(x.Ticks), S(x.Days), S(x.Hours), S(x.Minutes), S(x.Seconds), S(x.Milliseconds) };
            }
            case "duration.add": return new[] { S((new TimeSpan(L(a[0])) + new TimeSpan(L(a[1]))).Ticks) };
            case "duration.subtract": return new[] { S((new TimeSpan(L(a[0])) - new TimeSpan(L(a[1]))).Ticks) };
            case "duration.negate": return new[] { S((-new TimeSpan(L(a[0]))).Ticks) };
            case "duration.compare":
            {
                TimeSpan x = new(L(a[0])), y = new(L(a[1]));
                return new[] { S(Math.Sign(x.CompareTo(y))), B(x == y), B(x != y), B(x < y), B(x <= y), B(x > y), B(x >= y) };
            }
            case "guid.empty": return new[] { Guid.Empty.ToString("N", Invariant) };
            case "guid.compare":
            {
                Guid x = Guid.ParseExact(a[0], "N"), y = Guid.ParseExact(a[1], "N");
                return new[] { S(Math.Sign(x.CompareTo(y))), B(x == y), B(x != y) };
            }
            case "nullable.inspect":
            {
                int? x = N(a[0]);
                return new[] { B(x.HasValue), S(x.GetValueOrDefault()), S(x.GetValueOrDefault(7)) };
            }
            case "nullable.value": return new[] { S(N(a[0])!.Value) };
            case "nullable.add": return new[] { NText(checked(N(a[0]) + N(a[1]))) };
            case "nullable.divide": return new[] { NText(N(a[0]) / N(a[1])) };
            case "nullable.compare":
            {
                int? x = N(a[0]), y = N(a[1]);
                return new[] { B(x == y), B(x != y), B(x < y), B(x <= y), B(x > y), B(x >= y) };
            }
            case "nullable.boolean":
            {
                bool? x = a[0] == "none" ? null : a[0] == "1", y = a[1] == "none" ? null : a[1] == "1";
                return new[] { BoolText(x & y), BoolText(x | y), BoolText(!x), B(x == y), B(x != y) };
            }
            default: throw new InvalidOperationException("unknown operation");
        }
    }
    private static string DecimalValue(decimal value) => value == 0 ? "0" : value.ToString("0.############################", Invariant);
    private static string[] Money(string operation, string[] a)
    {
        decimal left = decimal.Parse(a[0], Invariant);
        string currency = a[1];
        if (operation == "create")
        {
            int scale = I(a[2]);
            if (currency != "AAA" && currency != "BBB") return new[] { "error", "invalid_currency" };
            if (scale < 0 || scale > 28) return new[] { "error", "invalid_scale" };
            if (decimal.Round(left, scale, MidpointRounding.ToEven) != left) return new[] { "error", "invalid_precision" };
            return new[] { "ok", DecimalValue(left), currency };
        }
        decimal right = decimal.Parse(a[2], Invariant);
        if (operation == "equal") return new[] { B(currency == a[3] && left == right) };
        if (operation == "compare")
        {
            int compared = string.CompareOrdinal(currency, a[3]);
            return new[] { S(Math.Sign(compared == 0 ? decimal.Compare(left, right) : compared)) };
        }
        try
        {
            decimal result;
            if (operation == "add" || operation == "subtract" || operation == "amount_compare")
            {
                if (currency != a[3]) return new[] { "error", "currency_mismatch" };
                if (operation == "amount_compare") return new[] { "ok", S(Math.Sign(decimal.Compare(left, right))) };
                result = operation == "add" ? left + right : left - right;
            }
            else
            {
                int scale = I(a[3]), mode = I(a[4]);
                if (scale < 0 || scale > 28) return new[] { "error", "invalid_scale" };
                if (mode < 0 || mode > 4) return new[] { "error", "invalid_rounding" };
                if (operation == "divide" && right == 0) return new[] { "error", "division_by_zero" };
                result = operation == "multiply" ? left * right : left / right;
                result = decimal.Round(result, scale, (MidpointRounding)mode);
            }
            return new[] { "ok", DecimalValue(result), currency };
        }
        catch (OverflowException) { return new[] { "error", "decimal_overflow" }; }
    }
    private static string[] Lifted(string[] op, string[] a)
    {
        string name = op[2];
        string right = a.Length == 1 ? a[0] : a[1];
        switch (op[1])
        {
            case "i32":
            {
                int? x = N(a[0]), y = N(right);
                if (name == "compare") return new[] { B(x == y), B(x != y), B(x < y), B(x <= y), B(x > y), B(x >= y) };
                int? z = name switch { "add" => x + y, "subtract" => x - y, "multiply" => x * y, "divide" => x / y, "remainder" => x % y, "negate" => -x, "plus" => +x, _ => throw new Exception() };
                return new[] { z.HasValue ? S(z.Value) : "none" };
            }
            case "i64":
            {
                long? x = a[0] == "none" ? null : L(a[0]), y = right == "none" ? null : L(right);
                if (name == "compare") return new[] { B(x == y), B(x != y), B(x < y), B(x <= y), B(x > y), B(x >= y) };
                long? z = name switch { "add" => x + y, "subtract" => x - y, "multiply" => x * y, "divide" => x / y, "remainder" => x % y, "negate" => -x, "plus" => +x, _ => throw new Exception() };
                return new[] { z.HasValue ? S(z.Value) : "none" };
            }
            case "decimal":
            {
                decimal? x = a[0] == "none" ? null : decimal.Parse(a[0], Invariant), y = right == "none" ? null : decimal.Parse(right, Invariant);
                if (name == "compare") return new[] { B(x == y), B(x != y), B(x < y), B(x <= y), B(x > y), B(x >= y) };
                decimal? z = name switch { "add" => x + y, "subtract" => x - y, "multiply" => x * y, "divide" => x / y, "remainder" => x % y, "negate" => -x, "plus" => +x, _ => throw new Exception() };
                return new[] { z.HasValue ? z.Value.ToString("G29", Invariant) : "none" };
            }
            case "f32":
            {
                float? x = a[0] == "none" ? null : float.Parse(a[0], Invariant), y = right == "none" ? null : float.Parse(right, Invariant);
                if (name == "compare") return new[] { B(x == y), B(x != y), B(x < y), B(x <= y), B(x > y), B(x >= y) };
                float? z = name switch { "add" => x + y, "subtract" => x - y, "multiply" => x * y, "divide" => x / y, "remainder" => x % y, "negate" => -x, "plus" => +x, _ => throw new Exception() };
                return new[] { z.HasValue ? unchecked((uint)BitConverter.SingleToInt32Bits(z.Value)).ToString("x8", Invariant) : "none" };
            }
            case "f64":
            {
                double? x = a[0] == "none" ? null : double.Parse(a[0], Invariant), y = right == "none" ? null : double.Parse(right, Invariant);
                if (name == "compare") return new[] { B(x == y), B(x != y), B(x < y), B(x <= y), B(x > y), B(x >= y) };
                double? z = name switch { "add" => x + y, "subtract" => x - y, "multiply" => x * y, "divide" => x / y, "remainder" => x % y, "negate" => -x, "plus" => +x, _ => throw new Exception() };
                return new[] { z.HasValue ? unchecked((ulong)BitConverter.DoubleToInt64Bits(z.Value)).ToString("x16", Invariant) : "none" };
            }
            default: throw new Exception();
        }
    }
    public static int Main(string[] args)
    {
        CultureInfo culture = (CultureInfo)Invariant.Clone();
        culture.NumberFormat.NegativeSign = args[1] == "hostile-comma" ? "~" : "negative";
        culture.NumberFormat.NumberDecimalSeparator = args[1] == "hostile-comma" ? "," : ":";
        culture.DateTimeFormat.DateSeparator = "!";
        CultureInfo.CurrentCulture = culture;
        CultureInfo.CurrentUICulture = culture;
        using JsonDocument document = JsonDocument.Parse(File.ReadAllBytes(args[0]));
        List<object> rows = new();
        foreach (JsonElement row in document.RootElement.EnumerateArray())
        {
            string operation = row.GetProperty("operation").GetString()!;
            string[] input = row.GetProperty("inputs").EnumerateArray().Select(x => x.GetString()!).ToArray();
            object outcome;
            try { outcome = new { kind = "value", value = Evaluate(operation, input) }; }
            catch (Exception error) { outcome = new { kind = "exception", value = new[] { error.GetType().FullName! } }; }
            rows.Add(new { id = row.GetProperty("id").GetString(), operation, inputs = input, observed = outcome });
        }
        Console.WriteLine(JsonSerializer.Serialize(new { runtime = Environment.Version.ToString(), culture = args[1], vectors = rows }));
        return 0;
    }
}
