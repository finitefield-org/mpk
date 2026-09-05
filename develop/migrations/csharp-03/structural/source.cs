namespace Business;
public enum Choice { Zero = 0, Alias = 0, One = 1 }
public readonly struct Scalars {
    public readonly bool Bool;
    public readonly sbyte I8;
    public readonly byte U8;
    public readonly short I16;
    public readonly ushort U16;
    public readonly int I32;
    public readonly uint U32;
    public readonly long I64;
    public readonly ulong U64;
    public readonly char Char;
    public readonly float F32;
    public readonly double F64;
    public readonly decimal Decimal;
    public readonly string? Text;
    public readonly System.DateOnly Date;
    public readonly System.TimeOnly Time;
    public readonly System.TimeSpan Duration;
    public readonly System.Guid Guid;
    public readonly System.DayOfWeek Day;
    public readonly Choice Choice;
}
public sealed class Product {
    public required Scalars Value {get;init;}
    public int Number {get;init;}
    public bool Same(Product other) { return Number == other.Number && Value.I32 == other.Value.I32; }
}
public sealed class Key { public int Z {get;init;} public int A {get;init;} }
public readonly struct Containers {
    public readonly Key? Key;
    public readonly Scalars? Optional;
    public readonly int[]? Sequence;
    public readonly Product? Reference;
}
public static class Entry {
    public static int Run(Containers input) {
        var first = new Product { Value = new Scalars(), Number = 1 };
        var second = new Product { Value = new Scalars(), Number = 1 };
        return first.Same(second) && input.Reference == null ? 1 : 0;
    }
}
